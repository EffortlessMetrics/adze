// Enhanced grammar validation for GLR parser with detailed diagnostics
// This module provides comprehensive validation with helpful error messages

use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, TokenPattern};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Detailed error information with suggestions
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: ErrorLocation,
    pub suggestion: Option<String>,
    pub related: Vec<RelatedInfo>,
}

/// Location information for errors
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub symbol: Option<SymbolId>,
    pub rule_index: Option<usize>,
    pub position: Option<usize>,
    pub description: String,
}

/// Related information for better error context
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub location: String,
    pub message: String,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    EmptyGrammar,
    NoStartSymbol,
    UndefinedSymbol,
    UnreachableSymbol,
    NonProductiveSymbol,
    LeftRecursion,
    AmbiguousGrammar,
    InvalidToken,
    DuplicateRule,
    InvalidField,
    ConflictingPrecedence,
    MissingRequiredSymbol,
    CyclicDependency,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error: {}", self.message)?;
        writeln!(f, "Location: {}", self.location.description)?;

        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "Suggestion: {}", suggestion)?;
        }

        if !self.related.is_empty() {
            writeln!(f, "Related information:")?;
            for info in &self.related {
                writeln!(f, "  - {} at {}", info.message, info.location)?;
            }
        }

        Ok(())
    }
}

/// Enhanced grammar validator with helpful diagnostics
pub struct GLRGrammarValidator {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    symbol_names: HashMap<SymbolId, String>,
}

/// Validation warnings (non-fatal)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
    pub location: String,
    pub suggestion: Option<String>,
}

/// Validation result with detailed information
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub stats: GrammarStats,
    pub suggestions: Vec<String>,
}

/// Grammar statistics
#[derive(Debug, Default)]
pub struct GrammarStats {
    pub total_symbols: usize,
    pub terminal_count: usize,
    pub nonterminal_count: usize,
    pub rule_count: usize,
    pub max_rule_length: usize,
    pub has_left_recursion: bool,
    pub is_ll1: bool,
    pub is_lr1: bool,
    pub requires_glr: bool,
}

impl GLRGrammarValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            symbol_names: HashMap::new(),
        }
    }

    /// Validate a grammar with comprehensive error reporting
    pub fn validate(&mut self, grammar: &Grammar) -> ValidationResult {
        self.errors.clear();
        self.warnings.clear();
        self.build_symbol_names(grammar);

        let mut stats = GrammarStats::default();

        // Phase 1: Basic structure validation
        self.validate_basic_structure(grammar);

        // Phase 2: Symbol validation
        self.validate_symbols(grammar);

        // Phase 3: Reachability and productivity
        let reachable = self.find_reachable_symbols(grammar);
        let productive = self.find_productive_symbols(grammar);
        self.validate_reachability(&reachable, grammar);
        self.validate_productivity(&productive, grammar);

        // Phase 4: Grammar properties
        self.check_left_recursion(grammar, &mut stats);
        self.check_ambiguity(grammar, &mut stats);

        // Phase 5: Token validation
        self.validate_tokens(grammar);

        // Phase 6: Precedence and associativity
        self.validate_precedence(grammar);

        // Collect statistics
        self.collect_statistics(grammar, &mut stats);

        // Generate suggestions
        let suggestions = self.generate_suggestions(grammar, &stats);

        ValidationResult {
            is_valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
            stats,
            suggestions,
        }
    }

    fn build_symbol_names(&mut self, grammar: &Grammar) {
        // Map tokens
        for (id, token) in &grammar.tokens {
            self.symbol_names.insert(*id, token.name.clone());
        }

        // Map rule names
        for (id, name) in &grammar.rule_names {
            self.symbol_names.insert(*id, name.clone());
        }

        // Map external tokens
        for external in &grammar.externals {
            self.symbol_names
                .insert(external.symbol_id, external.name.clone());
        }
    }

    fn get_symbol_name(&self, id: SymbolId) -> String {
        self.symbol_names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("symbol_{}", id.0))
    }

    fn validate_basic_structure(&mut self, grammar: &Grammar) {
        // Check for empty grammar
        if grammar.rules.is_empty() {
            self.errors.push(ValidationError {
                kind: ErrorKind::EmptyGrammar,
                message: "Grammar has no rules defined".to_string(),
                location: ErrorLocation {
                    symbol: None,
                    rule_index: None,
                    position: None,
                    description: "Grammar definition".to_string(),
                },
                suggestion: Some(
                    "Add at least one rule to define the grammar structure".to_string(),
                ),
                related: vec![],
            });
        }

        // Check for start symbol
        let has_explicit_start = grammar.rules.keys().any(|id| {
            self.get_symbol_name(*id).starts_with("_start") || self.get_symbol_name(*id) == "start"
        });

        if !has_explicit_start && !grammar.rules.is_empty() {
            let first_rule = grammar.rules.keys().next().unwrap();
            self.warnings.push(ValidationWarning {
                message: format!(
                    "No explicit start rule found, using '{}' as start symbol",
                    self.get_symbol_name(*first_rule)
                ),
                location: "Grammar root".to_string(),
                suggestion: Some(
                    "Consider adding an explicit '_start' rule for clarity".to_string(),
                ),
            });
        }
    }

    fn validate_symbols(&mut self, grammar: &Grammar) {
        let mut defined_symbols = HashSet::new();
        let mut used_symbols = HashSet::new();

        // Collect defined symbols
        defined_symbols.extend(grammar.tokens.keys());
        // Add non-terminals (LHS of rules)
        for rules in grammar.rules.values() {
            for rule in rules {
                defined_symbols.insert(rule.lhs);
            }
        }
        for external in &grammar.externals {
            defined_symbols.insert(external.symbol_id);
        }

        // Collect used symbols and check for undefined
        for (_symbol_id, rules) in &grammar.rules {
            for rule in rules {
                for (pos, symbol) in rule.rhs.iter().enumerate() {
                    let symbol_id = match symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
                        Symbol::External(ext) => SymbolId(ext.0),
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => {
                            continue; // Skip complex symbols in reachability analysis
                        }
                    };

                    used_symbols.insert(symbol_id);

                    if !defined_symbols.contains(&symbol_id) {
                        let mut related = vec![];

                        // Find similar symbols
                        let similar =
                            self.find_similar_symbols(&symbol_id, &defined_symbols, grammar);
                        for sim in similar {
                            related.push(RelatedInfo {
                                location: "Symbol definition".to_string(),
                                message: format!("Did you mean '{}'?", self.get_symbol_name(sim)),
                            });
                        }

                        self.errors.push(ValidationError {
                            kind: ErrorKind::UndefinedSymbol,
                            message: format!(
                                "Symbol '{}' is not defined",
                                self.get_symbol_name(symbol_id)
                            ),
                            location: ErrorLocation {
                                symbol: Some(rule.lhs),
                                rule_index: None,
                                position: Some(pos),
                                description: format!(
                                    "In rule '{}' at position {}",
                                    self.get_symbol_name(rule.lhs),
                                    pos
                                ),
                            },
                            suggestion: Some(
                                "Define the symbol as a token or rule before using it".to_string(),
                            ),
                            related,
                        });
                    }
                }
            }
        }

        // Check for duplicate rules
        let mut rule_counts: HashMap<SymbolId, usize> = HashMap::new();
        for rule_id in grammar.rules.keys() {
            *rule_counts.entry(*rule_id).or_insert(0) += 1;
        }

        for (symbol, count) in rule_counts {
            if count > 1 {
                self.errors.push(ValidationError {
                    kind: ErrorKind::DuplicateRule,
                    message: format!(
                        "Symbol '{}' has {} rule definitions",
                        self.get_symbol_name(symbol),
                        count
                    ),
                    location: ErrorLocation {
                        symbol: Some(symbol),
                        rule_index: None,
                        position: None,
                        description: format!("Symbol '{}'", self.get_symbol_name(symbol)),
                    },
                    suggestion: Some(
                        "Each symbol should have exactly one rule definition".to_string(),
                    ),
                    related: vec![],
                });
            }
        }
    }

    fn find_similar_symbols(
        &self,
        target: &SymbolId,
        defined: &HashSet<SymbolId>,
        _grammar: &Grammar,
    ) -> Vec<SymbolId> {
        let target_name = self.get_symbol_name(*target);
        let mut similar = vec![];

        for symbol in defined {
            let name = self.get_symbol_name(*symbol);
            if self.is_similar(&target_name, &name) {
                similar.push(*symbol);
            }
        }

        similar.sort_by_key(|s| {
            let name = self.get_symbol_name(*s);
            self.edit_distance(&target_name, &name)
        });

        similar.truncate(3);
        similar
    }

    fn is_similar(&self, s1: &str, s2: &str) -> bool {
        if s1 == s2 {
            return false;
        }

        let dist = self.edit_distance(s1, s2);
        let max_len = s1.len().max(s2.len());

        // Similar if edit distance is less than 30% of length
        (dist as f64) / (max_len as f64) < 0.3
    }

    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
                }
            }
        }

        dp[len1][len2]
    }

    fn find_reachable_symbols(&self, grammar: &Grammar) -> HashSet<SymbolId> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from the LHS of the first rule (start symbol)
        if let Some(start_rules) = grammar.rules.values().next() {
            if let Some(start_rule) = start_rules.first() {
                let start_symbol = start_rule.lhs;
                queue.push_back(start_symbol);
                reachable.insert(start_symbol);
            }
        }

        while let Some(symbol) = queue.pop_front() {
            // Find all rules with this symbol as LHS
            for rules in grammar.rules.values() {
                for rule in rules {
                    if rule.lhs == symbol {
                        for rhs_symbol in &rule.rhs {
                            let id = match rhs_symbol {
                                Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
                                Symbol::External(ext) => SymbolId(ext.0),
                                Symbol::Optional(_)
                                | Symbol::Repeat(_)
                                | Symbol::RepeatOne(_)
                                | Symbol::Choice(_)
                                | Symbol::Sequence(_)
                                | Symbol::Epsilon => {
                                    continue; // Skip complex symbols
                                }
                            };

                            if reachable.insert(id) {
                                queue.push_back(id);
                            }
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Helper to check if a symbol is productive
    fn is_symbol_productive(symbol: &Symbol, productive: &HashSet<SymbolId>) -> bool {
        match symbol {
            Symbol::Terminal(id) | Symbol::NonTerminal(id) => productive.contains(id),
            Symbol::External(ext) => productive.contains(&SymbolId(ext.0)),
            Symbol::Optional(_) | Symbol::Repeat(_) => true, // Always productive (can be empty)
            Symbol::RepeatOne(inner) => Self::is_symbol_productive(inner, productive),
            Symbol::Choice(choices) => choices
                .iter()
                .any(|s| Self::is_symbol_productive(s, productive)),
            Symbol::Sequence(seq) => seq
                .iter()
                .all(|s| Self::is_symbol_productive(s, productive)),
            Symbol::Epsilon => true,
        }
    }

    fn find_productive_symbols(&self, grammar: &Grammar) -> HashSet<SymbolId> {
        let mut productive = HashSet::new();
        let mut changed = true;

        // All terminals are productive
        productive.extend(grammar.tokens.keys());
        for external in &grammar.externals {
            productive.insert(external.symbol_id);
        }

        // Fixed-point iteration
        while changed {
            changed = false;

            for rules in grammar.rules.values() {
                for rule in rules {
                    let symbol = rule.lhs;
                    if !productive.contains(&symbol) {
                        let all_productive = rule.rhs.iter().all(|sym| {
                            match sym {
                                Symbol::Terminal(id) | Symbol::NonTerminal(id) => {
                                    productive.contains(id)
                                }
                                Symbol::External(ext) => productive.contains(&SymbolId(ext.0)),
                                Symbol::Optional(_) | Symbol::Repeat(_) => true, // Always productive (can be empty)
                                Symbol::RepeatOne(inner) => {
                                    Self::is_symbol_productive(inner, &productive)
                                }
                                Symbol::Choice(choices) => choices
                                    .iter()
                                    .any(|s| Self::is_symbol_productive(s, &productive)),
                                Symbol::Sequence(seq) => seq
                                    .iter()
                                    .all(|s| Self::is_symbol_productive(s, &productive)),
                                Symbol::Epsilon => true,
                            }
                        });

                        if all_productive {
                            productive.insert(symbol);
                            changed = true;
                        }
                    }
                }
            }
        }

        productive
    }

    fn validate_reachability(&mut self, reachable: &HashSet<SymbolId>, grammar: &Grammar) {
        // Check all non-terminals (LHS of rules)
        let mut non_terminals = HashSet::new();
        for rules in grammar.rules.values() {
            for rule in rules {
                non_terminals.insert(rule.lhs);
            }
        }

        for symbol in non_terminals {
            if !reachable.contains(&symbol) {
                self.warnings.push(ValidationWarning {
                    message: format!(
                        "Symbol '{}' is defined but not reachable from start symbol",
                        self.get_symbol_name(symbol)
                    ),
                    location: format!("Rule definition for '{}'", self.get_symbol_name(symbol)),
                    suggestion: Some(
                        "Remove unused rules or ensure they are referenced".to_string(),
                    ),
                });
            }
        }

        for (symbol, _token) in &grammar.tokens {
            if !reachable.contains(symbol) {
                self.warnings.push(ValidationWarning {
                    message: format!(
                        "Token '{}' is defined but never used",
                        self.get_symbol_name(*symbol)
                    ),
                    location: format!("Token definition for '{}'", self.get_symbol_name(*symbol)),
                    suggestion: Some("Remove unused tokens or use them in rules".to_string()),
                });
            }
        }
    }

    fn validate_productivity(&mut self, productive: &HashSet<SymbolId>, grammar: &Grammar) {
        // Check all non-terminals (LHS of rules)
        let mut non_terminals = HashSet::new();
        for rules in grammar.rules.values() {
            for rule in rules {
                non_terminals.insert(rule.lhs);
            }
        }

        for symbol in non_terminals {
            if !productive.contains(&symbol) {
                // Find why it's not productive
                let mut cycle_symbols = vec![];
                self.find_non_productive_cycle(
                    symbol,
                    grammar,
                    &mut cycle_symbols,
                    &mut HashSet::new(),
                );

                let mut related = vec![];
                for sym in &cycle_symbols {
                    related.push(RelatedInfo {
                        location: format!("Symbol '{}'", self.get_symbol_name(*sym)),
                        message: "Part of non-productive cycle".to_string(),
                    });
                }

                self.errors.push(ValidationError {
                    kind: ErrorKind::NonProductiveSymbol,
                    message: format!(
                        "Symbol '{}' cannot derive any terminal strings",
                        self.get_symbol_name(symbol)
                    ),
                    location: ErrorLocation {
                        symbol: Some(symbol),
                        rule_index: None,
                        position: None,
                        description: format!("Rule for '{}'", self.get_symbol_name(symbol)),
                    },
                    suggestion: Some(
                        "Add a rule that derives terminals or break the cycle".to_string(),
                    ),
                    related,
                });
            }
        }
    }

    fn find_non_productive_cycle(
        &self,
        start: SymbolId,
        grammar: &Grammar,
        path: &mut Vec<SymbolId>,
        visited: &mut HashSet<SymbolId>,
    ) -> bool {
        if visited.contains(&start) {
            return path.contains(&start);
        }

        visited.insert(start);
        path.push(start);

        if let Some(rules) = grammar.rules.get(&start) {
            for rule in rules {
                for symbol in &rule.rhs {
                    if let Symbol::NonTerminal(id) = symbol {
                        if self.find_non_productive_cycle(*id, grammar, path, visited) {
                            return true;
                        }
                    }
                }
            }
        }

        path.pop();
        false
    }

    fn check_left_recursion(&mut self, grammar: &Grammar, stats: &mut GrammarStats) {
        for (symbol, rules) in &grammar.rules {
            for rule in rules {
                // Check direct left recursion
                if let Some(Symbol::NonTerminal(first)) = rule.rhs.first() {
                    if first == symbol {
                        stats.has_left_recursion = true;

                        self.warnings.push(ValidationWarning {
                            message: format!(
                                "Direct left recursion in rule for '{}'",
                                self.get_symbol_name(*symbol)
                            ),
                            location: format!(
                                "Rule: {} → {} ...",
                                self.get_symbol_name(*symbol),
                                self.get_symbol_name(*first)
                            ),
                            suggestion: Some(
                                "Consider rewriting to eliminate left recursion or use GLR parsing"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }

            // Check indirect left recursion
            if self.has_indirect_left_recursion(*symbol, grammar) {
                stats.has_left_recursion = true;
                self.warnings.push(ValidationWarning {
                    message: format!(
                        "Indirect left recursion detected for '{}'",
                        self.get_symbol_name(*symbol)
                    ),
                    location: format!("Starting from rule for '{}'", self.get_symbol_name(*symbol)),
                    suggestion: Some(
                        "GLR parsing can handle left recursion efficiently".to_string(),
                    ),
                });
            }
        }
    }

    fn has_indirect_left_recursion(&self, symbol: SymbolId, grammar: &Grammar) -> bool {
        let mut visited = HashSet::new();
        self.can_derive_left(symbol, symbol, grammar, &mut visited)
    }

    fn can_derive_left(
        &self,
        from: SymbolId,
        target: SymbolId,
        grammar: &Grammar,
        visited: &mut HashSet<SymbolId>,
    ) -> bool {
        if visited.contains(&from) {
            return false;
        }
        visited.insert(from);

        if let Some(rules) = grammar.rules.get(&from) {
            for rule in rules {
                if let Some(Symbol::NonTerminal(first)) = rule.rhs.first() {
                    if *first == target {
                        return true;
                    }
                    if self.can_derive_left(*first, target, grammar, visited) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn check_ambiguity(&mut self, grammar: &Grammar, stats: &mut GrammarStats) {
        // Detect ambiguity by analyzing the grammar for conflicts

        // 1. Check for direct ambiguity (multiple rules with same RHS)
        self.check_direct_ambiguity(grammar, stats);

        // 2. Check for shift/reduce and reduce/reduce conflicts
        self.check_lr_conflicts(grammar, stats);

        // 3. Check for common ambiguous patterns
        self.check_ambiguous_patterns(grammar, stats);
    }

    fn check_direct_ambiguity(&mut self, grammar: &Grammar, stats: &mut GrammarStats) {
        // Group rules by their RHS
        let mut rhs_map: HashMap<Vec<Symbol>, Vec<SymbolId>> = HashMap::new();

        for (symbol, rules) in &grammar.rules {
            for rule in rules {
                let rhs = rule.rhs.clone();
                rhs_map.entry(rhs).or_default().push(*symbol);
            }
        }

        // Check for multiple rules with the same RHS
        for (rhs, symbols) in rhs_map {
            if symbols.len() > 1 {
                let rhs_str = rhs
                    .iter()
                    .map(|s| match s {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) => self.get_symbol_name(*id),
                        Symbol::External(ext) => format!("external_{}", ext.0),
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => "<complex>".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                self.warnings.push(ValidationWarning {
                    message: format!(
                        "Direct ambiguity: multiple non-terminals produce '{}'",
                        rhs_str
                    ),
                    location: format!(
                        "Non-terminals: {}",
                        symbols
                            .iter()
                            .map(|s| self.get_symbol_name(*s))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    suggestion: Some(
                        "Consider merging these rules or adding precedence".to_string(),
                    ),
                });

                stats.requires_glr = true;
            }
        }
    }

    fn check_lr_conflicts(&mut self, grammar: &Grammar, stats: &mut GrammarStats) {
        // Check for common LR conflict patterns

        // 1. Check for left-recursive and right-recursive rules for the same non-terminal
        for (symbol, rules) in &grammar.rules {
            for rule in rules {
                let has_left_rec = !rule.rhs.is_empty()
                    && match &rule.rhs[0] {
                        Symbol::NonTerminal(id) => id == symbol,
                        _ => false,
                    };

                let has_right_rec = rule.rhs.len() > 1
                    && match rule.rhs.last() {
                        Some(Symbol::NonTerminal(id)) => id == symbol,
                        _ => false,
                    };

                // Check if there are both left and right recursive rules for the same symbol
                if has_left_rec || has_right_rec {
                    let other_rules: Vec<_> = grammar
                        .rules
                        .iter()
                        .filter(|(s, _)| *s == symbol)
                        .flat_map(|(_, rules)| rules.iter())
                        .filter(|r| {
                            let other_left_rec = !r.rhs.is_empty()
                                && match &r.rhs[0] {
                                    Symbol::NonTerminal(id) => id == symbol,
                                    _ => false,
                                };
                            let other_right_rec = r.rhs.len() > 1
                                && match r.rhs.last() {
                                    Some(Symbol::NonTerminal(id)) => id == symbol,
                                    _ => false,
                                };
                            (has_left_rec && other_right_rec) || (has_right_rec && other_left_rec)
                        })
                        .collect();

                    if !other_rules.is_empty() {
                        self.warnings.push(ValidationWarning {
                            message: format!(
                                "Mixed left/right recursion for '{}'",
                                self.get_symbol_name(*symbol)
                            ),
                            location: format!("Non-terminal '{}'", self.get_symbol_name(*symbol)),
                            suggestion: Some(
                                "This creates shift/reduce conflicts - GLR will handle it"
                                    .to_string(),
                            ),
                        });
                        stats.requires_glr = true;
                    }
                }
            }
        }

        // 2. Check for common prefix ambiguity
        let mut prefix_map: HashMap<Vec<SymbolId>, Vec<(SymbolId, usize)>> = HashMap::new();

        for (symbol, rules) in &grammar.rules {
            for rule in rules {
                for prefix_len in 1..=rule.rhs.len().min(3) {
                    let prefix: Vec<_> = rule
                        .rhs
                        .iter()
                        .take(prefix_len)
                        .map(|s| match s {
                            Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
                            Symbol::External(ext) => SymbolId(ext.0),
                            Symbol::Optional(_)
                            | Symbol::Repeat(_)
                            | Symbol::RepeatOne(_)
                            | Symbol::Choice(_)
                            | Symbol::Sequence(_)
                            | Symbol::Epsilon => {
                                panic!("Complex symbols should be normalized before validation");
                            }
                        })
                        .collect();

                    prefix_map
                        .entry(prefix)
                        .or_default()
                        .push((*symbol, rule.rhs.len()));
                }
            }
        }

        for (prefix, occurrences) in prefix_map {
            if occurrences.len() > 1 {
                // Check if the rules have different lengths (shift/reduce conflict)
                let lengths: HashSet<_> = occurrences.iter().map(|(_, len)| *len).collect();
                if lengths.len() > 1 && prefix.len() < *lengths.iter().min().unwrap() {
                    let prefix_str = prefix
                        .iter()
                        .map(|s| self.get_symbol_name(*s))
                        .collect::<Vec<_>>()
                        .join(" ");

                    self.warnings.push(ValidationWarning {
                        message: format!(
                            "Shift/reduce conflict: rules with common prefix '{}'",
                            prefix_str
                        ),
                        location: format!(
                            "Rules: {}",
                            occurrences
                                .iter()
                                .map(|(s, _)| self.get_symbol_name(*s))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        suggestion: Some("GLR parsing will explore both possibilities".to_string()),
                    });
                    stats.requires_glr = true;
                }
            }
        }
    }

    fn check_ambiguous_patterns(&mut self, grammar: &Grammar, stats: &mut GrammarStats) {
        // Check for classic ambiguous patterns

        // 0. Check for E → E E pattern (highly ambiguous)
        for (_key, rules) in &grammar.rules {
            for rule in rules {
                if rule.rhs.len() == 2 {
                    if let (Symbol::NonTerminal(id1), Symbol::NonTerminal(id2)) =
                        (&rule.rhs[0], &rule.rhs[1])
                    {
                        if *id1 == rule.lhs && *id2 == rule.lhs {
                            self.warnings.push(ValidationWarning {
                                message: format!(
                                    "Highly ambiguous pattern: '{}' → '{}' '{}'",
                                    self.get_symbol_name(rule.lhs),
                                    self.get_symbol_name(rule.lhs),
                                    self.get_symbol_name(rule.lhs)
                                ),
                                location: format!(
                                    "Non-terminal '{}'",
                                    self.get_symbol_name(rule.lhs)
                                ),
                                suggestion: Some(
                                    "This creates extreme ambiguity - GLR required".to_string(),
                                ),
                            });
                            stats.requires_glr = true;
                        }
                    }
                }
            }
        }

        // 1. Dangling else pattern (if-then-else ambiguity)
        for (_symbol, rules) in &grammar.rules {
            for rule in rules {
                let rule_str = rule
                    .rhs
                    .iter()
                    .map(|s| match s {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) => self.get_symbol_name(*id),
                        Symbol::External(ext) => format!("external_{}", ext.0),
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => "<complex>".to_string(),
                    })
                    .collect::<Vec<_>>();

                // Look for patterns like: if expr then stmt | if expr then stmt else stmt
                if rule_str.len() >= 4 {
                    let has_if_pattern = rule_str
                        .windows(4)
                        .any(|w| w[0].contains("if") && w[2].contains("then"));

                    if has_if_pattern {
                        let has_optional_else = grammar.rules.values().any(|rules| {
                            rules.iter().any(|r| {
                                let other_str = r
                                    .rhs
                                    .iter()
                                    .map(|s| match s {
                                        Symbol::Terminal(id) | Symbol::NonTerminal(id) => {
                                            self.get_symbol_name(*id)
                                        }
                                        Symbol::External(ext) => format!("external_{}", ext.0),
                                        Symbol::Optional(_)
                                        | Symbol::Repeat(_)
                                        | Symbol::RepeatOne(_)
                                        | Symbol::Choice(_)
                                        | Symbol::Sequence(_)
                                        | Symbol::Epsilon => "<complex>".to_string(),
                                    })
                                    .collect::<Vec<_>>();

                                other_str.len() > rule_str.len()
                                    && other_str.contains(&"else".to_string())
                            })
                        });

                        if has_optional_else {
                            self.warnings.push(ValidationWarning {
                                message: "Potential 'dangling else' ambiguity detected".to_string(),
                                location: "Conditional statement rules".to_string(),
                                suggestion: Some(
                                    "Use precedence or GLR parsing to resolve".to_string(),
                                ),
                            });
                            stats.requires_glr = true;
                        }
                    }
                }
            }
        }

        // 2. Expression ambiguity (like E → E + E | E * E)
        let mut symbol_binary_ops: HashMap<SymbolId, Vec<&Rule>> = HashMap::new();

        for (_key, rules) in &grammar.rules {
            for rule in rules {
                if rule.rhs.len() == 3 {
                    if let (Symbol::NonTerminal(id1), _, Symbol::NonTerminal(id2)) =
                        (&rule.rhs[0], &rule.rhs[1], &rule.rhs[2])
                    {
                        if *id1 == rule.lhs && *id2 == rule.lhs {
                            symbol_binary_ops.entry(rule.lhs).or_default().push(rule);
                        }
                    }
                }
            }
        }

        for (symbol, binary_ops) in symbol_binary_ops {
            if binary_ops.len() > 1 {
                let symbol_name = self.get_symbol_name(symbol);
                self.warnings.push(ValidationWarning {
                    message: format!(
                        "Expression ambiguity: '{}' has multiple binary operators",
                        symbol_name
                    ),
                    location: format!("Non-terminal '{}'", symbol_name),
                    suggestion: Some("Define precedence levels or use GLR parsing".to_string()),
                });
                stats.requires_glr = true;
            }
        }
    }

    fn validate_tokens(&mut self, grammar: &Grammar) {
        for (symbol, token) in &grammar.tokens {
            match &token.pattern {
                TokenPattern::String(s) if s.is_empty() => {
                    self.errors.push(ValidationError {
                        kind: ErrorKind::InvalidToken,
                        message: format!(
                            "Token '{}' has empty string pattern",
                            self.get_symbol_name(*symbol)
                        ),
                        location: ErrorLocation {
                            symbol: Some(*symbol),
                            rule_index: None,
                            position: None,
                            description: format!("Token '{}'", self.get_symbol_name(*symbol)),
                        },
                        suggestion: Some("Provide a non-empty pattern for the token".to_string()),
                        related: vec![],
                    });
                }
                TokenPattern::Regex(r) => {
                    // Validate regex compilation
                    match regex::Regex::new(r) {
                        Err(e) => {
                            self.errors.push(ValidationError {
                                kind: ErrorKind::InvalidToken,
                                message: format!(
                                    "Invalid regex pattern for token '{}'",
                                    self.get_symbol_name(*symbol)
                                ),
                                location: ErrorLocation {
                                    symbol: Some(*symbol),
                                    rule_index: None,
                                    position: None,
                                    description: format!(
                                        "Token '{}' pattern: {}",
                                        self.get_symbol_name(*symbol),
                                        r
                                    ),
                                },
                                suggestion: Some(format!("Fix regex error: {}", e)),
                                related: vec![],
                            });
                        }
                        Ok(re) => {
                            // Check for common regex issues
                            if r.contains("*+") || r.contains("++") || r.contains("?+") {
                                self.warnings.push(ValidationWarning {
                                    message: format!(
                                        "Possessive quantifiers in token '{}'",
                                        self.get_symbol_name(*symbol)
                                    ),
                                    location: format!("Pattern: {}", r),
                                    suggestion: Some(
                                        "Consider if possessive quantifiers are intended"
                                            .to_string(),
                                    ),
                                });
                            }

                            if !r.starts_with('^') && re.find_at("test", 1).is_some() {
                                self.warnings.push(ValidationWarning {
                                    message: format!(
                                        "Token '{}' pattern may match at non-boundary",
                                        self.get_symbol_name(*symbol)
                                    ),
                                    location: format!("Pattern: {}", r),
                                    suggestion: Some(
                                        "Consider anchoring with ^ for consistent tokenization"
                                            .to_string(),
                                    ),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for overlapping token patterns
        let mut patterns: Vec<(SymbolId, String)> = vec![];
        for (symbol, token) in &grammar.tokens {
            let pattern_str = match &token.pattern {
                TokenPattern::String(s) => regex::escape(s),
                TokenPattern::Regex(r) => r.clone(),
            };
            patterns.push((*symbol, pattern_str));
        }

        for i in 0..patterns.len() {
            for j in i + 1..patterns.len() {
                if self.patterns_overlap(&patterns[i].1, &patterns[j].1) {
                    self.warnings.push(ValidationWarning {
                        message: format!(
                            "Tokens '{}' and '{}' have overlapping patterns",
                            self.get_symbol_name(patterns[i].0),
                            self.get_symbol_name(patterns[j].0)
                        ),
                        location: "Token definitions".to_string(),
                        suggestion: Some(
                            "Ensure token precedence is correctly defined".to_string(),
                        ),
                    });
                }
            }
        }
    }

    fn patterns_overlap(&self, p1: &str, p2: &str) -> bool {
        // Simple overlap detection
        if p1 == p2 {
            return true;
        }

        // Check if one pattern can match what the other matches
        if let (Ok(re1), Ok(re2)) = (regex::Regex::new(p1), regex::Regex::new(p2)) {
            // Test some common cases
            let test_strings = vec!["a", "1", " ", "test", "123", "abc123"];
            for test in test_strings {
                if re1.is_match(test) && re2.is_match(test) {
                    return true;
                }
            }
        }

        false
    }

    fn validate_precedence(&mut self, grammar: &Grammar) {
        // Check for conflicting precedence declarations
        let mut symbol_precedences: HashMap<SymbolId, Vec<(i32, String)>> = HashMap::new();

        for (i, prec) in grammar.precedences.iter().enumerate() {
            for symbol in &prec.symbols {
                symbol_precedences
                    .entry(*symbol)
                    .or_default()
                    .push((prec.level as i32, format!("precedence group {}", i)));
            }
        }

        for (symbol, precs) in symbol_precedences {
            if precs.len() > 1 {
                let levels: HashSet<_> = precs.iter().map(|(l, _)| *l).collect();
                if levels.len() > 1 {
                    self.errors.push(ValidationError {
                        kind: ErrorKind::ConflictingPrecedence,
                        message: format!(
                            "Symbol '{}' has conflicting precedence levels",
                            self.get_symbol_name(symbol)
                        ),
                        location: ErrorLocation {
                            symbol: Some(symbol),
                            rule_index: None,
                            position: None,
                            description: "Precedence declarations".to_string(),
                        },
                        suggestion: Some(
                            "Each symbol should have at most one precedence level".to_string(),
                        ),
                        related: precs
                            .iter()
                            .map(|(level, loc)| RelatedInfo {
                                location: loc.clone(),
                                message: format!("Precedence level {}", level),
                            })
                            .collect(),
                    });
                }
            }
        }
    }

    fn collect_statistics(&self, grammar: &Grammar, stats: &mut GrammarStats) {
        stats.total_symbols = grammar.tokens.len() + grammar.rules.len() + grammar.externals.len();
        stats.terminal_count = grammar.tokens.len();
        stats.nonterminal_count = grammar.rules.len();
        stats.rule_count = grammar.rules.values().map(|rules| rules.len()).sum();

        stats.max_rule_length = grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .map(|r| r.rhs.len())
            .max()
            .unwrap_or(0);

        // Simple LL(1) check - no left recursion and no ambiguity
        stats.is_ll1 = !stats.has_left_recursion && !stats.requires_glr;

        // LR(1) can handle left recursion but not ambiguity
        stats.is_lr1 = !stats.requires_glr;
    }

    fn generate_suggestions(&self, grammar: &Grammar, stats: &GrammarStats) -> Vec<String> {
        let mut suggestions = vec![];

        if stats.has_left_recursion {
            suggestions
                .push("Grammar contains left recursion - GLR parsing is recommended".to_string());
        }

        if stats.requires_glr {
            suggestions.push(
                "Grammar has ambiguities that require GLR parsing for correct handling".to_string(),
            );
        }

        if stats.max_rule_length > 10 {
            suggestions.push(format!("Some rules are very long (max: {} symbols). Consider breaking them down for readability", 
                                   stats.max_rule_length));
        }

        if grammar.precedences.is_empty() && stats.requires_glr {
            suggestions
                .push("Consider adding precedence declarations to resolve ambiguities".to_string());
        }

        if grammar.fields.is_empty() && grammar.rules.len() > 5 {
            suggestions
                .push("Consider adding field names to rules for better AST access".to_string());
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::ProductionId;

    #[test]
    fn test_empty_grammar_error() {
        let grammar = Grammar::new("test".to_string());
        let mut validator = GLRGrammarValidator::new();
        let result = validator.validate(&grammar);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.kind == ErrorKind::EmptyGrammar)
        );
    }

    #[test]
    fn test_undefined_symbol_with_suggestions() {
        let mut grammar = Grammar::new("test".to_string());

        // Define a token
        grammar.tokens.insert(
            SymbolId(1),
            rust_sitter_ir::Token {
                name: "number".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );
        grammar.rule_names.insert(SymbolId(1), "number".to_string());

        // Use undefined symbol "numbr" (typo)
        grammar.rules.insert(
            SymbolId(2),
            vec![Rule {
                lhs: SymbolId(2),
                rhs: vec![Symbol::Terminal(SymbolId(99))], // undefined
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        grammar.rule_names.insert(SymbolId(2), "expr".to_string());
        grammar.rule_names.insert(SymbolId(99), "numbr".to_string()); // typo

        let mut validator = GLRGrammarValidator::new();
        let result = validator.validate(&grammar);

        assert!(!result.is_valid);
        let undefined_error = result
            .errors
            .iter()
            .find(|e| e.kind == ErrorKind::UndefinedSymbol)
            .expect("Should have undefined symbol error");

        // Should suggest "number" as similar to "numbr"
        assert!(!undefined_error.related.is_empty());
    }

    #[test]
    fn test_left_recursion_detection() {
        let mut grammar = Grammar::new("test".to_string());

        // expr → expr + number (left recursive)
        let expr_id = SymbolId(1);
        let plus_id = SymbolId(2);
        let number_id = SymbolId(3);

        grammar.tokens.insert(
            plus_id,
            rust_sitter_ir::Token {
                name: "plus".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        grammar.tokens.insert(
            number_id,
            rust_sitter_ir::Token {
                name: "number".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );

        grammar.rules.insert(
            expr_id,
            vec![Rule {
                lhs: expr_id,
                rhs: vec![
                    Symbol::NonTerminal(expr_id),
                    Symbol::Terminal(plus_id),
                    Symbol::Terminal(number_id),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );

        grammar.rule_names.insert(expr_id, "expr".to_string());

        let mut validator = GLRGrammarValidator::new();
        let result = validator.validate(&grammar);

        assert!(result.stats.has_left_recursion);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("left recursion"))
        );
    }
}
