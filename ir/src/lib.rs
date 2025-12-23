// IR crate should be safe - no unsafe needed for grammar representation
#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), warn(missing_docs))]

//! Grammar Intermediate Representation for pure-Rust Tree-sitter
//! This module provides GLR-aware data structures for representing Tree-sitter grammars

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error types and Result alias for IR operations.
pub mod error;
pub use error::{IrError, Result as IrResult};

/// Grammar optimization utilities
pub mod optimizer;
pub use optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};

/// Grammar validation utilities
pub mod validation;
pub use validation::{GrammarValidator, ValidationError, ValidationResult, ValidationWarning};

/// Debug macros for development
pub mod debug_macros;
/// Symbol registry for managing grammar symbols
pub mod symbol_registry;
pub use symbol_registry::{SymbolInfo, SymbolRegistry};
/// Builder API for programmatically constructing grammars
pub mod builder;

/// Core grammar representation supporting all Tree-sitter features including GLR
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Grammar {
    /// Grammar name
    pub name: String,
    /// Production rules indexed by left-hand side symbol
    pub rules: IndexMap<SymbolId, Vec<Rule>>,
    /// Token definitions
    pub tokens: IndexMap<SymbolId, Token>,
    /// Precedence declarations
    pub precedences: Vec<Precedence>,
    /// Conflict resolution declarations
    pub conflicts: Vec<ConflictDeclaration>,
    /// External scanner tokens
    pub externals: Vec<ExternalToken>,
    /// Extra tokens (e.g., whitespace, comments)
    pub extras: Vec<SymbolId>,
    /// Field names maintained in lexicographic order
    pub fields: IndexMap<FieldId, String>,
    /// Supertype symbols
    pub supertypes: Vec<SymbolId>,
    /// Rules to inline during generation
    pub inline_rules: Vec<SymbolId>,
    /// Alias sequences for productions
    pub alias_sequences: IndexMap<ProductionId, AliasSequence>,
    /// Maps rule IDs to production IDs
    pub production_ids: IndexMap<RuleId, ProductionId>,
    /// Maximum alias sequence length
    pub max_alias_sequence_length: usize,
    /// Maps symbol IDs to rule names
    pub rule_names: IndexMap<SymbolId, String>,
    /// Centralized symbol registry
    pub symbol_registry: Option<SymbolRegistry>,
}

impl Grammar {
    /// Add a rule to the grammar
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.entry(rule.lhs).or_default().push(rule);
    }

    /// Get all rules for a given LHS symbol
    pub fn get_rules_for_symbol(&self, symbol: SymbolId) -> Option<&Vec<Rule>> {
        self.rules.get(&symbol)
    }

    /// Iterate over all rules in the grammar
    pub fn all_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values().flat_map(|rules| rules.iter())
    }

    /// Get the start symbol (LHS of the first rule)
    pub fn start_symbol(&self) -> Option<SymbolId> {
        // For Tree-sitter compatibility, look for "source_file" symbol
        if let Some(source_file_id) = self.find_symbol_by_name("source_file")
            && self.rules.contains_key(&source_file_id)
        {
            return Some(source_file_id);
        }

        // In rust-sitter, source_file is often just a reference to the actual language type
        // So let's look for the language type that's marked with #[rust_sitter::language]
        // This is typically the first non-terminal that has rules

        // Try common patterns first
        for name in &["Expression", "Statement", "Program", "Module"] {
            if let Some(symbol_id) = self.find_symbol_by_name(name)
                && self.rules.contains_key(&symbol_id)
            {
                return Some(symbol_id);
            }
        }

        // Otherwise, use the first symbol that has rules and isn't a leaf/token
        for (symbol_id, rules) in &self.rules {
            // Skip symbols that look like internal/generated names
            if let Some(name) = self.rule_names.get(symbol_id)
                && !name.contains('_')
                && !rules.is_empty()
            {
                return Some(*symbol_id);
            }
        }

        // Final fallback: just use the first symbol with rules
        self.rules.keys().next().copied()
    }

    /// Find a symbol by its name in rule_names
    pub fn find_symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        for (symbol_id, symbol_name) in &self.rule_names {
            if symbol_name == name {
                return Some(*symbol_id);
            }
        }
        None
    }

    /// Build or get the symbol registry
    pub fn get_or_build_registry(&mut self) -> &SymbolRegistry {
        if self.symbol_registry.is_none() {
            self.symbol_registry = Some(self.build_registry());
        }
        self.symbol_registry.as_ref().unwrap()
    }

    /// Check for empty string terminals (separate from main validate)
    pub fn check_empty_terminals(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for empty string terminals
        for (id, token) in &self.tokens {
            match &token.pattern {
                TokenPattern::String(s) if s.is_empty() => {
                    errors.push(format!(
                        "Token '{}' (id={}) has empty string pattern",
                        token.name, id.0
                    ));
                }
                TokenPattern::Regex(r) if r.is_empty() => {
                    errors.push(format!(
                        "Token '{}' (id={}) has empty regex pattern",
                        token.name, id.0
                    ));
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Build a new symbol registry from the grammar
    pub fn build_registry(&self) -> SymbolRegistry {
        let mut registry = SymbolRegistry::new();

        // Sort tokens deterministically: underscore-prefixed last
        let mut token_entries: Vec<_> = self.tokens.iter().collect();
        token_entries.sort_by_key(|(_id, token)| {
            let name = &token.name;
            (name.starts_with('_'), name.clone())
        });

        // Register all tokens
        for (symbol_id, token) in token_entries {
            let metadata = SymbolMetadata {
                is_visible: !token.name.starts_with('_'),
                is_named: false,
                is_hidden: self.extras.contains(symbol_id),
                is_terminal: true,
            };
            registry.register(&token.name, metadata);
        }

        // Sort non-terminals deterministically
        let mut rule_entries: Vec<_> = self.rule_names.iter().collect();
        rule_entries.sort_by_key(|(_, name)| (*name).clone());

        // Register all non-terminals
        for (symbol_id, name) in rule_entries {
            if !self.tokens.contains_key(symbol_id) {
                let metadata = SymbolMetadata {
                    is_visible: !name.starts_with('_'),
                    is_named: true,
                    is_hidden: name.starts_with('_'),
                    is_terminal: false,
                };
                registry.register(name, metadata);
            }
        }

        // Register externals
        for external in &self.externals {
            let metadata = SymbolMetadata {
                is_visible: true,
                is_named: false,
                is_hidden: false,
                is_terminal: true,
            };
            registry.register(&external.name, metadata);
        }

        registry
    }
}

/// Grammar rule supporting GLR multiple actions per state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// Left-hand side symbol
    pub lhs: SymbolId,
    /// Right-hand side symbols
    pub rhs: Vec<Symbol>,
    /// Precedence if specified
    pub precedence: Option<PrecedenceKind>,
    /// Associativity if specified
    pub associativity: Option<Associativity>,
    /// Field to position mapping
    pub fields: Vec<(FieldId, usize)>,
    /// Production ID
    pub production_id: ProductionId,
}

/// Precedence supporting both static and dynamic precedence (PREC_DYNAMIC)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecedenceKind {
    /// Static precedence
    Static(i16),
    /// Dynamic precedence
    Dynamic(i16),
}

/// Token with fragile flag for lexical vs parse conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// Token name
    pub name: String,
    /// Token pattern (string or regex)
    pub pattern: TokenPattern,
    /// TSFragile flag for lexical vs parse conflicts
    pub fragile: bool,
}

/// Token pattern representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenPattern {
    /// String literal pattern
    String(String),
    /// Regular expression pattern
    Regex(String),
}

/// Grammar symbol types representing the building blocks of grammar rules.
///
/// Symbols in a grammar can be either simple (terminals, non-terminals, externals)
/// or complex (optional, repetition, choice, sequence). Complex symbols must be
/// normalized into simple rules before GLR parser generation.
///
/// # Simple Symbols (GLR-Ready)
///
/// These symbols can be directly used in LR(1) item sets and parse table generation:
/// - `Terminal`: Lexical tokens defined in the grammar's token set
/// - `NonTerminal`: Grammar rules that can be expanded into other symbols
/// - `External`: Tokens produced by external scanners (e.g., indentation)
/// - `Epsilon`: Empty production (matches zero symbols)
///
/// # Complex Symbols (Require Normalization)
///
/// These symbols provide syntactic sugar but must be converted to auxiliary rules:
/// - `Optional`: Matches zero or one occurrence of the inner symbol
/// - `Repeat`: Matches zero or more occurrences (Kleene star)
/// - `RepeatOne`: Matches one or more occurrences (Kleene plus)
/// - `Choice`: Matches one of several alternative symbols
/// - `Sequence`: Matches a sequence of symbols in order
///
/// # Normalization Process
///
/// Complex symbols are normalized by [`Grammar::normalize()`] which creates auxiliary
/// non-terminal rules. For example:
///
/// ```text
/// Optional(Terminal("x")) => aux_N -> Terminal("x") | Epsilon
/// Repeat(Terminal("x"))   => aux_N -> aux_N Terminal("x") | Epsilon
/// Choice([A, B, C])       => aux_N -> A | B | C
/// ```
///
/// # Examples
///
/// ```ignore
/// // Simple symbols - ready for GLR parsing
/// let token = Symbol::Terminal(SymbolId(1));
/// let rule_ref = Symbol::NonTerminal(SymbolId(2));
///
/// // Complex symbols - need normalization
/// let optional_comma = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(3))));
/// let repeat_stmt = Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(4))));
/// let choice = Symbol::Choice(vec![
///     Symbol::Terminal(SymbolId(5)),
///     Symbol::Terminal(SymbolId(6)),
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Symbol {
    /// Terminal symbol representing a lexical token.
    ///
    /// Terminals are the atomic units of a language, typically matched by regular
    /// expressions or string literals. Examples: keywords, operators, identifiers.
    Terminal(SymbolId),

    /// Non-terminal symbol representing a grammar rule.
    ///
    /// Non-terminals expand into sequences of other symbols (terminals or non-terminals).
    /// They define the hierarchical structure of the language.
    NonTerminal(SymbolId),

    /// External scanner symbol for context-sensitive lexing.
    ///
    /// External symbols are produced by custom scanner code, enabling parsing of
    /// constructs that cannot be expressed with regular expressions (e.g., Python
    /// indentation, here-documents, template literals).
    External(SymbolId),

    /// Optional symbol matching zero or one occurrence.
    ///
    /// Normalized to: `aux -> inner | Epsilon`
    ///
    /// # Example
    /// ```text
    /// Optional(Terminal("?")) => aux_1000 -> Terminal("?") | Epsilon
    /// ```
    Optional(Box<Symbol>),

    /// Repetition matching zero or more occurrences (Kleene star).
    ///
    /// Normalized to left-recursive rule for parser efficiency:
    /// `aux -> aux inner | Epsilon`
    ///
    /// # Example
    /// ```text
    /// Repeat(Terminal(",")) => aux_1001 -> aux_1001 Terminal(",") | Epsilon
    /// ```
    Repeat(Box<Symbol>),

    /// Repetition matching one or more occurrences (Kleene plus).
    ///
    /// Normalized to: `aux -> aux inner | inner`
    ///
    /// # Example
    /// ```text
    /// RepeatOne(NonTerminal("stmt")) =>
    ///   aux_1002 -> aux_1002 NonTerminal("stmt") | NonTerminal("stmt")
    /// ```
    RepeatOne(Box<Symbol>),

    /// Choice between multiple alternative symbols.
    ///
    /// Normalized to separate rules for each alternative:
    /// `aux -> choice1 | choice2 | choice3`
    ///
    /// # Example
    /// ```text
    /// Choice([A, B, C]) =>
    ///   aux_1003 -> A
    ///   aux_1003 -> B
    ///   aux_1003 -> C
    /// ```
    Choice(Vec<Symbol>),

    /// Sequence of symbols that must appear in order.
    ///
    /// If the sequence contains more than one symbol after normalization,
    /// it is converted to an auxiliary rule: `aux -> symbol1 symbol2 symbol3`
    ///
    /// # Example
    /// ```text
    /// Sequence([A, B, C]) => aux_1004 -> A B C
    /// ```
    Sequence(Vec<Symbol>),

    /// Empty production matching zero symbols.
    ///
    /// Used for optional parts of rules and as the base case for repetitions.
    /// In LR parsing, this creates epsilon transitions.
    Epsilon,
}

/// Alias sequence for node renaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasSequence {
    /// Aliases for each position
    pub aliases: Vec<Option<String>>,
}

/// Precedence declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precedence {
    /// Precedence level
    pub level: i16,
    /// Associativity for this level
    pub associativity: Associativity,
    /// Symbols at this precedence level
    pub symbols: Vec<SymbolId>,
}

/// Associativity for conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Associativity {
    /// Left associative
    Left,
    /// Right associative
    Right,
    /// Non-associative
    None,
}

/// Conflict declaration for GLR handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDeclaration {
    /// Conflicting symbols
    pub symbols: Vec<SymbolId>,
    /// Conflict resolution strategy
    pub resolution: ConflictResolution,
}

/// How to resolve conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Resolve by precedence
    Precedence(PrecedenceKind),
    /// Resolve by associativity
    Associativity(Associativity),
    /// Allow GLR fork/merge
    GLR,
}

/// External token declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalToken {
    /// External token name
    pub name: String,
    /// Symbol ID for the external token
    pub symbol_id: SymbolId,
}

// Type-safe IDs
/// Symbol identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub u16);

/// Rule identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RuleId(pub u16);

/// State identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StateId(pub u16);

/// Field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldId(pub u16);

/// Production identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProductionId(pub u16);

// Display implementations for debugging
impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.0)
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule({})", self.0)
    }
}

impl fmt::Display for StateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State({})", self.0)
    }
}

impl fmt::Display for FieldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field({})", self.0)
    }
}

impl fmt::Display for ProductionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Production({})", self.0)
    }
}

/// Metadata for a symbol in the language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMetadata {
    /// Whether the symbol is visible
    pub is_visible: bool,
    /// Whether the symbol is named
    pub is_named: bool,
    /// Whether the symbol is hidden
    pub is_hidden: bool,
    /// Whether the symbol is a terminal
    pub is_terminal: bool,
}

/// Grammar validation and processing
impl Grammar {
    /// Create a new empty grammar
    pub fn new(name: String) -> Self {
        Self {
            name,
            rules: IndexMap::new(),
            tokens: IndexMap::new(),
            precedences: Vec::new(),
            conflicts: Vec::new(),
            externals: Vec::new(),
            extras: Vec::new(),
            fields: IndexMap::new(),
            supertypes: Vec::new(),
            inline_rules: Vec::new(),
            alias_sequences: IndexMap::new(),
            production_ids: IndexMap::new(),
            max_alias_sequence_length: 0,
            rule_names: IndexMap::new(),
            symbol_registry: None,
        }
    }

    /// Extract IR from procedural macro data
    pub fn from_macro_output(data: &str) -> Result<Self, GrammarError> {
        // This will be implemented to parse the output from rust-sitter macros
        serde_json::from_str(data).map_err(GrammarError::ParseError)
    }

    /// Helper to validate a symbol recursively
    fn validate_symbol(&self, symbol: &Symbol) -> Result<(), GrammarError> {
        match symbol {
            Symbol::Terminal(id) | Symbol::NonTerminal(id) => {
                if !self.rules.contains_key(id) && !self.tokens.contains_key(id) {
                    return Err(GrammarError::UnresolvedSymbol(*id));
                }
            }
            Symbol::External(id) => {
                if !self.externals.iter().any(|ext| ext.symbol_id == *id) {
                    return Err(GrammarError::UnresolvedExternalSymbol(*id));
                }
            }
            Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
                self.validate_symbol(inner)?;
            }
            Symbol::Choice(choices) => {
                for s in choices {
                    self.validate_symbol(s)?;
                }
            }
            Symbol::Sequence(seq) => {
                for s in seq {
                    self.validate_symbol(s)?;
                }
            }
            Symbol::Epsilon => {}
        }
        Ok(())
    }

    /// Validate grammar consistency and detect issues
    pub fn validate(&self) -> Result<(), GrammarError> {
        // Validate field name ordering (must be lexicographic)
        let mut field_names: Vec<_> = self.fields.values().collect();
        field_names.sort();
        let expected_order: Vec<_> = self.fields.values().collect();
        if field_names != expected_order {
            return Err(GrammarError::InvalidFieldOrdering);
        }

        // Validate symbol references
        for rule in self.all_rules() {
            for symbol in &rule.rhs {
                self.validate_symbol(symbol)?;
            }
        }

        Ok(())
    }

    /// Apply grammar transformations for better table generation
    pub fn optimize(&mut self) {
        // Remove unused rules
        // Inline simple rules where beneficial
        // Optimize precedence declarations
        // This will be implemented based on Tree-sitter's optimization strategies
    }

    /// Normalize complex symbols by creating auxiliary rules for GLR parsing.
    ///
    /// This method transforms complex symbols (`Optional`, `Repeat`, `RepeatOne`, `Choice`,
    /// `Sequence`) into simple non-terminal rules that can be processed by the GLR parser
    /// generator. Each complex symbol is replaced with a reference to a newly created
    /// auxiliary non-terminal rule.
    ///
    /// # Why Normalization is Necessary
    ///
    /// GLR parser generators operate on LR(1) grammars, which only support simple
    /// production rules of the form `A -> B C D`. Complex symbols like `Optional(X)`
    /// or `Repeat(Y)` are syntactic sugar that must be expanded into standard rules
    /// before FIRST/FOLLOW set computation and LR(1) item set generation.
    ///
    /// # Auxiliary Rule Generation Strategy
    ///
    /// The normalization process creates auxiliary non-terminal symbols with IDs starting
    /// at `max_existing_id + 1000` to avoid conflicts with existing symbols. The auxiliary
    /// rules are named `_auxN` where N is the symbol ID.
    ///
    /// ## Normalization Patterns
    ///
    /// | Complex Symbol | Auxiliary Rules Generated |
    /// |----------------|---------------------------|
    /// | `Optional(X)` | `aux -> X \| Epsilon` |
    /// | `Repeat(X)` | `aux -> aux X \| Epsilon` (left-recursive) |
    /// | `RepeatOne(X)` | `aux -> aux X \| X` (left-recursive) |
    /// | `Choice([X, Y, Z])` | `aux -> X`, `aux -> Y`, `aux -> Z` |
    /// | `Sequence([X, Y])` | `aux -> X Y` (if length > 1) |
    ///
    /// # Idempotency
    ///
    /// This method is idempotent - calling it multiple times has no additional effect
    /// after the first normalization. Normalized symbols (terminals, non-terminals,
    /// externals, epsilon) are left unchanged.
    ///
    /// # Algorithm Details
    ///
    /// 1. **Symbol ID Allocation**: Finds the maximum existing symbol ID and allocates
    ///    auxiliary symbols starting at `max_id + 1000`, bounded by `60000` to stay
    ///    within `u16` range.
    ///
    /// 2. **Recursive Processing**: Processes each rule's right-hand side recursively,
    ///    normalizing nested complex symbols from the inside out.
    ///
    /// 3. **Rule Replacement**: Replaces original rules containing complex symbols with
    ///    normalized versions, preserving precedence, associativity, and field mappings.
    ///
    /// 4. **Registry Update**: Adds auxiliary symbol names to `rule_names` map for
    ///    debugging and error reporting.
    ///
    /// # Production ID Management
    ///
    /// Production IDs are allocated sequentially during normalization to ensure each
    /// rule alternative has a unique identifier. This is critical for parse tree
    /// construction and disambiguation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut grammar = Grammar::new("example".to_string());
    ///
    /// // Before normalization:
    /// // rule: stmt -> Optional(Terminal("if"))
    /// grammar.add_rule(Rule {
    ///     lhs: SymbolId(1),
    ///     rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
    ///     precedence: None,
    ///     associativity: None,
    ///     fields: vec![],
    ///     production_id: ProductionId(0),
    /// });
    ///
    /// grammar.normalize()?;
    ///
    /// // After normalization:
    /// // rule: stmt -> NonTerminal(aux_1000)
    /// // aux_1000 -> Terminal("if")
    /// // aux_1000 -> Epsilon
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `GrammarError` if normalization encounters invalid symbol references
    /// during recursive processing.
    ///
    /// # See Also
    ///
    /// - [`normalize_symbol()`](Self::normalize_symbol) - Normalizes a single symbol
    /// - [`normalize_symbol_list()`](Self::normalize_symbol_list) - Normalizes a list of symbols
    /// - [`Symbol`] - Documentation on symbol types and their normalization patterns
    pub fn normalize(&mut self) -> Result<(), GrammarError> {
        let mut new_rules_to_add = Vec::new();

        // First, find the max symbol ID to avoid conflicts
        let max_id = self
            .rules
            .keys()
            .chain(self.tokens.keys())
            .map(|id| id.0)
            .max()
            .unwrap_or(0);

        // Start auxiliary symbols well above existing ones, but within u16 range
        let mut aux_counter: u16 = (max_id + 1000).min(60000);

        // Process each existing rule and normalize its RHS
        let mut rules_to_replace = Vec::new();
        for (lhs, rules) in &self.rules {
            for rule in rules {
                let (normalized_rhs, new_aux_rules) =
                    self.normalize_symbol_list(&rule.rhs, &mut aux_counter)?;

                // If normalization produced auxiliary rules, we need to replace this rule
                if !new_aux_rules.is_empty() || normalized_rhs != rule.rhs {
                    let new_rule = Rule {
                        lhs: *lhs,
                        rhs: normalized_rhs,
                        precedence: rule.precedence,
                        associativity: rule.associativity,
                        fields: rule.fields.clone(),
                        production_id: rule.production_id,
                    };

                    rules_to_replace.push((*lhs, rule.clone(), new_rule));
                    new_rules_to_add.extend(new_aux_rules);
                }
            }
        }

        // Replace rules with their normalized versions
        for (lhs, old_rule, new_rule) in rules_to_replace {
            if let Some(rules) = self.rules.get_mut(&lhs)
                && let Some(pos) = rules
                    .iter()
                    .position(|r| r.production_id == old_rule.production_id)
            {
                rules[pos] = new_rule;
            }
        }

        // Add all the new auxiliary rules
        for rule in new_rules_to_add {
            let lhs = rule.lhs;
            self.rules.entry(lhs).or_default().push(rule);

            // Add rule name for the auxiliary symbol
            if !self.rule_names.contains_key(&lhs) {
                self.rule_names.insert(lhs, format!("_aux{}", lhs.0));
            }
        }

        Ok(())
    }

    /// Normalize a list of symbols recursively, processing each symbol in sequence.
    ///
    /// This helper function normalizes multiple symbols and collects all generated
    /// auxiliary rules. It is used for processing the right-hand side of rules and
    /// the contents of `Sequence` symbols.
    ///
    /// # Parameters
    ///
    /// - `symbols`: The slice of symbols to normalize
    /// - `aux_counter`: Mutable counter for allocating unique auxiliary symbol IDs
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<Symbol>`: The normalized symbols (complex symbols replaced with aux references)
    /// - `Vec<Rule>`: All auxiliary rules generated during normalization
    ///
    /// # Algorithm
    ///
    /// Iterates through each symbol, calling [`normalize_symbol()`](Self::normalize_symbol)
    /// and accumulating the results. The auxiliary rules from all symbols are combined
    /// into a single vector.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Input: [Terminal(1), Optional(Terminal(2)), NonTerminal(3)]
    /// // Output:
    /// //   normalized: [Terminal(1), NonTerminal(aux_1000), NonTerminal(3)]
    /// //   aux_rules: [aux_1000 -> Terminal(2), aux_1000 -> Epsilon]
    /// ```
    fn normalize_symbol_list(
        &self,
        symbols: &[Symbol],
        aux_counter: &mut u16,
    ) -> Result<(Vec<Symbol>, Vec<Rule>), GrammarError> {
        let mut normalized_symbols = Vec::new();
        let mut auxiliary_rules = Vec::new();

        for symbol in symbols {
            let (norm_symbol, mut aux_rules) = self.normalize_symbol(symbol, aux_counter)?;
            normalized_symbols.push(norm_symbol);
            auxiliary_rules.append(&mut aux_rules);
        }

        Ok((normalized_symbols, auxiliary_rules))
    }

    /// Normalize a single symbol recursively, creating auxiliary rules as needed.
    ///
    /// This is the core normalization function that implements the transformation patterns
    /// for each type of complex symbol. Simple symbols are returned unchanged, while complex
    /// symbols are replaced with references to newly created auxiliary non-terminals.
    ///
    /// # Parameters
    ///
    /// - `symbol`: The symbol to normalize
    /// - `aux_counter`: Mutable counter for allocating unique auxiliary symbol IDs and production IDs
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Symbol`: The normalized symbol (either the original or a reference to an auxiliary rule)
    /// - `Vec<Rule>`: Auxiliary rules generated for this symbol (empty for simple symbols)
    ///
    /// # Normalization Rules by Symbol Type
    ///
    /// ## Simple Symbols (No Transformation)
    ///
    /// - `Terminal(id)`: Returned as-is
    /// - `NonTerminal(id)`: Returned as-is
    /// - `External(id)`: Returned as-is
    /// - `Epsilon`: Returned as-is
    ///
    /// ## Optional(inner)
    ///
    /// Creates auxiliary rules:
    /// ```text
    /// aux_N -> <normalized_inner>
    /// aux_N -> Epsilon
    /// ```
    /// Returns: `NonTerminal(aux_N)`
    ///
    /// ## Repeat(inner) - Zero or More
    ///
    /// Creates left-recursive auxiliary rules for efficient parsing:
    /// ```text
    /// aux_N -> aux_N <normalized_inner>
    /// aux_N -> Epsilon
    /// ```
    /// Returns: `NonTerminal(aux_N)`
    ///
    /// **Why Left Recursion?** Left-recursive rules (`A -> A x`) are more efficient in
    /// LR parsers than right-recursive rules (`A -> x A`) because they build the parse
    /// tree incrementally without deep stack nesting.
    ///
    /// ## RepeatOne(inner) - One or More
    ///
    /// Creates left-recursive auxiliary rules with non-empty base case:
    /// ```text
    /// aux_N -> aux_N <normalized_inner>
    /// aux_N -> <normalized_inner>
    /// ```
    /// Returns: `NonTerminal(aux_N)`
    ///
    /// ## Choice(alternatives)
    ///
    /// Creates one auxiliary rule per alternative:
    /// ```text
    /// aux_N -> <normalized_alt1>
    /// aux_N -> <normalized_alt2>
    /// aux_N -> <normalized_alt3>
    /// ```
    /// Returns: `NonTerminal(aux_N)`
    ///
    /// ## Sequence(symbols)
    ///
    /// Normalizes each symbol in the sequence. If the normalized sequence has:
    /// - **One symbol**: Returns that symbol directly (no auxiliary rule needed)
    /// - **Multiple symbols**: Creates auxiliary rule `aux_N -> s1 s2 s3 ...`
    ///
    /// # Recursive Processing
    ///
    /// Complex symbols can be nested (e.g., `Optional(Repeat(Choice([A, B])))`).
    /// This function processes them recursively, normalizing from the innermost
    /// symbol outward, ensuring all nested complex symbols are eliminated.
    ///
    /// # Production ID Allocation
    ///
    /// Each auxiliary rule is assigned a unique `ProductionId` by incrementing
    /// `aux_counter`. This ensures parse trees can distinguish between different
    /// rule alternatives during disambiguation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Simple symbol - no change
    /// normalize_symbol(&Symbol::Terminal(SymbolId(5)), &mut counter)
    /// // => (Symbol::Terminal(SymbolId(5)), vec![])
    ///
    /// // Optional - creates 2 auxiliary rules
    /// normalize_symbol(&Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(5)))), &mut counter)
    /// // => (Symbol::NonTerminal(SymbolId(1000)), vec![
    /// //      Rule { lhs: SymbolId(1000), rhs: vec![Terminal(5)], ... },
    /// //      Rule { lhs: SymbolId(1000), rhs: vec![Epsilon], ... }
    /// //    ])
    ///
    /// // Nested complex symbols
    /// normalize_symbol(
    ///     &Symbol::Repeat(Box::new(
    ///         Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(5))))
    ///     )),
    ///     &mut counter
    /// )
    /// // First normalizes Optional to aux_1000, then creates Repeat rules for aux_1001
    /// ```
    fn normalize_symbol(
        &self,
        symbol: &Symbol,
        aux_counter: &mut u16,
    ) -> Result<(Symbol, Vec<Rule>), GrammarError> {
        match symbol {
            Symbol::Terminal(_)
            | Symbol::NonTerminal(_)
            | Symbol::External(_)
            | Symbol::Epsilon => {
                // These are already normalized
                Ok((symbol.clone(), Vec::new()))
            }
            Symbol::Optional(inner) => {
                // First recursively normalize the inner symbol
                let (norm_inner, mut inner_rules) = self.normalize_symbol(inner, aux_counter)?;

                // Create aux rule: aux -> inner | ε
                let aux_id = SymbolId(*aux_counter);
                *aux_counter += 1;

                // aux -> inner
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![norm_inner],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter), // Use unique production IDs
                });

                // aux -> ε
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![Symbol::Epsilon],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter + 1),
                });

                *aux_counter += 2; // Used 2 production IDs

                Ok((Symbol::NonTerminal(aux_id), inner_rules))
            }
            Symbol::Repeat(inner) => {
                // First recursively normalize the inner symbol
                let (norm_inner, mut inner_rules) = self.normalize_symbol(inner, aux_counter)?;

                // Create aux rule: aux -> aux inner | ε
                let aux_id = SymbolId(*aux_counter);
                *aux_counter += 1;

                // aux -> aux inner (left-recursive for efficiency)
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![Symbol::NonTerminal(aux_id), norm_inner],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter),
                });

                // aux -> ε
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![Symbol::Epsilon],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter + 1),
                });

                *aux_counter += 2;

                Ok((Symbol::NonTerminal(aux_id), inner_rules))
            }
            Symbol::RepeatOne(inner) => {
                // First recursively normalize the inner symbol
                let (norm_inner, mut inner_rules) = self.normalize_symbol(inner, aux_counter)?;

                // Create aux rule: aux -> aux inner | inner
                let aux_id = SymbolId(*aux_counter);
                *aux_counter += 1;

                // aux -> aux inner (left-recursive)
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![Symbol::NonTerminal(aux_id), norm_inner.clone()],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter),
                });

                // aux -> inner
                inner_rules.push(Rule {
                    lhs: aux_id,
                    rhs: vec![norm_inner],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(*aux_counter + 1),
                });

                *aux_counter += 2;

                Ok((Symbol::NonTerminal(aux_id), inner_rules))
            }
            Symbol::Choice(choices) => {
                // Create aux rules: aux -> choice1 | choice2 | ...
                let aux_id = SymbolId(*aux_counter);
                *aux_counter += 1;

                let mut choice_rules = Vec::new();

                for choice in choices {
                    // Recursively normalize each choice
                    let (norm_choice, mut choice_aux_rules) =
                        self.normalize_symbol(choice, aux_counter)?;
                    choice_rules.append(&mut choice_aux_rules);

                    choice_rules.push(Rule {
                        lhs: aux_id,
                        rhs: vec![norm_choice],
                        precedence: None,
                        associativity: None,
                        fields: vec![],
                        production_id: ProductionId(*aux_counter),
                    });

                    *aux_counter += 1;
                }

                Ok((Symbol::NonTerminal(aux_id), choice_rules))
            }
            Symbol::Sequence(seq) => {
                // Normalize each element of the sequence and flatten
                let (norm_seq, aux_rules) = self.normalize_symbol_list(seq, aux_counter)?;
                // For sequences, we can return the flattened sequence directly
                // But we need to handle the case where there are multiple symbols
                if norm_seq.len() == 1 {
                    Ok((norm_seq.into_iter().next().unwrap(), aux_rules))
                } else {
                    // Create an auxiliary rule for the sequence
                    let aux_id = SymbolId(*aux_counter);
                    *aux_counter += 1;

                    let mut seq_rules = aux_rules;
                    seq_rules.push(Rule {
                        lhs: aux_id,
                        rhs: norm_seq,
                        precedence: None,
                        associativity: None,
                        fields: vec![],
                        production_id: ProductionId(*aux_counter),
                    });

                    *aux_counter += 1;

                    Ok((Symbol::NonTerminal(aux_id), seq_rules))
                }
            }
        }
    }
}

/// Grammar processing errors
#[derive(Debug, thiserror::Error)]
pub enum GrammarError {
    /// Failed to parse grammar
    #[error("Failed to parse grammar: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Invalid field ordering
    #[error("Invalid field ordering - fields must be in lexicographic order")]
    InvalidFieldOrdering,

    /// Unresolved symbol reference
    #[error("Unresolved symbol reference: {0}")]
    UnresolvedSymbol(SymbolId),

    /// Unresolved external symbol reference
    #[error("Unresolved external symbol reference: {0}")]
    UnresolvedExternalSymbol(SymbolId),

    /// Conflict in grammar
    #[error("Conflict in grammar: {0}")]
    ConflictError(String),

    /// Invalid precedence declaration
    #[error("Invalid precedence declaration: {0}")]
    InvalidPrecedence(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_creation() {
        let grammar = Grammar::new("test".to_string());
        assert_eq!(grammar.name, "test");
        assert!(grammar.rules.is_empty());
        assert!(grammar.tokens.is_empty());
        assert!(grammar.precedences.is_empty());
        assert!(grammar.conflicts.is_empty());
        assert!(grammar.externals.is_empty());
        assert!(grammar.fields.is_empty());
        assert!(grammar.supertypes.is_empty());
        assert!(grammar.inline_rules.is_empty());
        assert!(grammar.alias_sequences.is_empty());
        assert!(grammar.production_ids.is_empty());
        assert_eq!(grammar.max_alias_sequence_length, 0);
    }

    #[test]
    fn test_field_ordering_validation() {
        let mut grammar = Grammar::new("test".to_string());

        // Add fields in non-lexicographic order
        grammar.fields.insert(FieldId(1), "zebra".to_string());
        grammar.fields.insert(FieldId(0), "alpha".to_string());

        // Validation should fail
        assert!(grammar.validate().is_err());

        // Fix the ordering
        grammar.fields.clear();
        grammar.fields.insert(FieldId(0), "alpha".to_string());
        grammar.fields.insert(FieldId(1), "zebra".to_string());

        // Validation should now pass
        assert!(grammar.validate().is_ok());
    }

    #[test]
    fn test_symbol_id_display() {
        let symbol_id = SymbolId(42);
        assert_eq!(format!("{}", symbol_id), "Symbol(42)");

        let rule_id = RuleId(10);
        assert_eq!(format!("{}", rule_id), "Rule(10)");

        let state_id = StateId(5);
        assert_eq!(format!("{}", state_id), "State(5)");

        let field_id = FieldId(3);
        assert_eq!(format!("{}", field_id), "Field(3)");

        let production_id = ProductionId(7);
        assert_eq!(format!("{}", production_id), "Production(7)");
    }

    #[test]
    fn test_precedence_kinds() {
        let static_prec = PrecedenceKind::Static(5);
        let dynamic_prec = PrecedenceKind::Dynamic(10);

        match static_prec {
            PrecedenceKind::Static(level) => assert_eq!(level, 5),
            _ => panic!("Expected static precedence"),
        }

        match dynamic_prec {
            PrecedenceKind::Dynamic(level) => assert_eq!(level, 10),
            _ => panic!("Expected dynamic precedence"),
        }
    }

    #[test]
    fn test_symbol_types() {
        let terminal = Symbol::Terminal(SymbolId(1));
        let non_terminal = Symbol::NonTerminal(SymbolId(2));
        let external = Symbol::External(SymbolId(3));

        match terminal {
            Symbol::Terminal(SymbolId(1)) => {}
            _ => panic!("Expected terminal symbol"),
        }

        match non_terminal {
            Symbol::NonTerminal(SymbolId(2)) => {}
            _ => panic!("Expected non-terminal symbol"),
        }

        match external {
            Symbol::External(SymbolId(3)) => {}
            _ => panic!("Expected external symbol"),
        }

        // Test equality and hashing
        assert_eq!(terminal, Symbol::Terminal(SymbolId(1)));
        assert_ne!(terminal, non_terminal);

        let mut set = std::collections::HashSet::new();
        set.insert(terminal.clone());
        assert!(set.contains(&terminal));
        assert!(!set.contains(&non_terminal));
    }

    #[test]
    fn test_token_patterns() {
        let string_pattern = TokenPattern::String("hello".to_string());
        let regex_pattern = TokenPattern::Regex(r"\d+".to_string());

        match string_pattern {
            TokenPattern::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string pattern"),
        }

        match regex_pattern {
            TokenPattern::Regex(r) => assert_eq!(r, r"\d+"),
            _ => panic!("Expected regex pattern"),
        }
    }

    #[test]
    fn test_associativity() {
        let left = Associativity::Left;
        let right = Associativity::Right;
        let none = Associativity::None;

        assert_eq!(left, Associativity::Left);
        assert_eq!(right, Associativity::Right);
        assert_eq!(none, Associativity::None);

        assert_ne!(left, right);
        assert_ne!(left, none);
        assert_ne!(right, none);
    }

    #[test]
    fn test_conflict_resolution() {
        let precedence_resolution = ConflictResolution::Precedence(PrecedenceKind::Static(5));
        let associativity_resolution = ConflictResolution::Associativity(Associativity::Left);
        let glr_resolution = ConflictResolution::GLR;

        match precedence_resolution {
            ConflictResolution::Precedence(PrecedenceKind::Static(5)) => {}
            _ => panic!("Expected precedence resolution"),
        }

        match associativity_resolution {
            ConflictResolution::Associativity(Associativity::Left) => {}
            _ => panic!("Expected associativity resolution"),
        }

        match glr_resolution {
            ConflictResolution::GLR => {}
            _ => panic!("Expected GLR resolution"),
        }
    }

    #[test]
    fn test_grammar_with_rules_and_tokens() {
        let mut grammar = Grammar::new("test_grammar".to_string());

        // Add a rule: S -> NUMBER
        let rule = Rule {
            lhs: SymbolId(0),                         // S
            rhs: vec![Symbol::Terminal(SymbolId(1))], // NUMBER
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(0), 0)],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        // Add a token
        let token = Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(1), token);

        // Add fields in correct order
        grammar.fields.insert(FieldId(0), "left".to_string());
        grammar.fields.insert(FieldId(1), "right".to_string());

        // Validation should pass
        match grammar.validate() {
            Ok(_) => {}
            Err(e) => panic!("Grammar validation failed: {:?}", e),
        }

        assert_eq!(grammar.rules.len(), 1);
        assert_eq!(grammar.tokens.len(), 1);
        assert_eq!(grammar.fields.len(), 2);
    }

    #[test]
    fn test_grammar_validation_unresolved_symbol() {
        let mut grammar = Grammar::new("test".to_string());

        // Add a rule that references a non-existent symbol
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(999))], // Non-existent symbol
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        // Validation should fail
        assert!(grammar.validate().is_err());

        match grammar.validate() {
            Err(GrammarError::UnresolvedSymbol(SymbolId(999))) => {}
            _ => panic!("Expected unresolved symbol error"),
        }
    }

    #[test]
    fn test_grammar_validation_unresolved_external() {
        let mut grammar = Grammar::new("test".to_string());

        // Add a rule that references a non-existent external symbol
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::External(SymbolId(999))], // Non-existent external
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        // Validation should fail
        assert!(grammar.validate().is_err());

        match grammar.validate() {
            Err(GrammarError::UnresolvedExternalSymbol(SymbolId(999))) => {}
            _ => panic!("Expected unresolved external symbol error"),
        }
    }

    #[test]
    fn test_alias_sequence() {
        let alias_seq = AliasSequence {
            aliases: vec![Some("alias1".to_string()), None, Some("alias2".to_string())],
        };

        assert_eq!(alias_seq.aliases.len(), 3);
        assert_eq!(alias_seq.aliases[0], Some("alias1".to_string()));
        assert_eq!(alias_seq.aliases[1], None);
        assert_eq!(alias_seq.aliases[2], Some("alias2".to_string()));
    }

    #[test]
    fn test_external_token() {
        let external_token = ExternalToken {
            name: "HERE_STRING".to_string(),
            symbol_id: SymbolId(42),
        };

        assert_eq!(external_token.name, "HERE_STRING");
        assert_eq!(external_token.symbol_id, SymbolId(42));
    }

    #[test]
    fn test_precedence() {
        let precedence = Precedence {
            level: 10,
            associativity: Associativity::Right,
            symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
        };

        assert_eq!(precedence.level, 10);
        assert_eq!(precedence.associativity, Associativity::Right);
        assert_eq!(precedence.symbols.len(), 3);
        assert!(precedence.symbols.contains(&SymbolId(2)));
    }

    #[test]
    fn test_conflict_declaration() {
        let conflict = ConflictDeclaration {
            symbols: vec![SymbolId(1), SymbolId(2)],
            resolution: ConflictResolution::GLR,
        };

        assert_eq!(conflict.symbols.len(), 2);
        assert!(conflict.symbols.contains(&SymbolId(1)));
        assert!(conflict.symbols.contains(&SymbolId(2)));

        match conflict.resolution {
            ConflictResolution::GLR => {}
            _ => panic!("Expected GLR resolution"),
        }
    }
}
