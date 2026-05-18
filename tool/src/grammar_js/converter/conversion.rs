use super::{GrammarJsConverter, JsRule};
use adze_ir::{ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, SymbolId};
use anyhow::{Context, Result};
use indexmap::IndexMap;

impl GrammarJsConverter {
    /// Convert Grammar.js to Adze Grammar IR
    pub fn convert(mut self) -> Result<Grammar> {
        eprintln!(
            "DEBUG converter.convert: Starting conversion for grammar '{}'",
            self.grammar_js.name
        );
        eprintln!(
            "DEBUG converter.convert: Grammar.js has {} rules",
            self.grammar_js.rules.len()
        );

        let mut grammar = Grammar {
            name: self.grammar_js.name.clone(),
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
        };

        // First pass: collect all symbols (rules and tokens)
        self.collect_symbols(&mut grammar)?;

        // Convert rules to IR rules
        self.convert_rules(&mut grammar)?;

        // Handle inline rules
        for inline in &self.grammar_js.inline {
            if let Some(&symbol_id) = self.symbol_names.get(inline) {
                grammar.inline_rules.push(symbol_id);
            }
        }

        // Handle externals
        for external in &self.grammar_js.externals {
            if let Some(&symbol_id) = self.symbol_names.get(&external.name) {
                grammar.externals.push(ExternalToken {
                    name: external.name.clone(),
                    symbol_id,
                });
            }
        }

        // Handle conflicts
        for conflict_set in &self.grammar_js.conflicts {
            let mut symbols = Vec::new();
            for rule in conflict_set {
                if let Some(&symbol_id) = self.symbol_names.get(rule) {
                    symbols.push(symbol_id);
                }
            }
            if !symbols.is_empty() {
                grammar.conflicts.push(ConflictDeclaration {
                    symbols,
                    resolution: ConflictResolution::GLR, // Default to GLR handling
                });
            }
        }

        // Handle supertypes
        for supertype in &self.grammar_js.supertypes {
            if let Some(&symbol_id) = self.symbol_names.get(supertype) {
                grammar.supertypes.push(symbol_id);
            }
        }

        // Handle extras
        eprintln!(
            "DEBUG converter: Processing extras, count = {}",
            self.grammar_js.extras.len()
        );
        for extra in &self.grammar_js.extras {
            eprintln!("  Processing extra: {:?}", extra);
            if let Some(symbol_id) = self.find_extra_symbol(extra, &grammar) {
                eprintln!("    Found symbol_id: {:?}", symbol_id);
                grammar.extras.push(symbol_id);
            } else {
                eprintln!("    WARNING: Could not find symbol for extra");
            }
        }

        // Copy fields
        grammar.fields = self.fields.clone();

        eprintln!(
            "DEBUG converter.convert: Final grammar has {} rules",
            grammar.rules.len()
        );
        eprintln!(
            "DEBUG converter.convert: Final grammar has {} tokens",
            grammar.tokens.len()
        );
        eprintln!("DEBUG converter.convert: Final grammar rule_names:");
        for (symbol_id, name) in &grammar.rule_names {
            eprintln!("  SymbolId({}) -> '{}'", symbol_id.0, name);
        }

        // Check what the start symbol will be
        if let Some(start_symbol) = grammar.start_symbol() {
            eprintln!(
                "DEBUG converter.convert: Start symbol is SymbolId({}) -> '{}'",
                start_symbol.0,
                grammar
                    .rule_names
                    .get(&start_symbol)
                    .unwrap_or(&"???".to_string())
            );
        } else {
            eprintln!("DEBUG converter.convert: No start symbol found!");
        }

        Ok(grammar)
    }
    pub(super) fn collect_symbols(&mut self, grammar: &mut Grammar) -> Result<()> {
        // Add all rule names as non-terminals
        for rule_name in self.grammar_js.rules.keys() {
            let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
            eprintln!(
                "Debug: Collecting symbol '{}' as SymbolId({})",
                rule_name, self.next_symbol_id
            );
            if rule_name == "source_file" {
                eprintln!(
                    "Debug: FOUND source_file! Adding to symbol_names and rule_names as SymbolId({})",
                    symbol_id.0
                );
            }
            self.symbol_names.insert(rule_name.clone(), symbol_id);
            grammar.rule_names.insert(symbol_id, rule_name.clone());
            self.next_symbol_id += 1;
        }

        // Add common terminal tokens
        // NOTE: Commented out because these default tokens interfere with custom patterns
        // and cause incorrect lexer generation
        // self.add_terminal_token(grammar, "_STRING", r#""[^"]*""#)?;
        // self.add_terminal_token(grammar, "_NUMBER", r"-?\d+(\.\d+)?")?;
        // self.add_terminal_token(grammar, "_IDENTIFIER", r"[a-zA-Z_]\w*")?;

        // Add whitespace token if in extras
        let has_whitespace = self.grammar_js.extras.iter().any(|extra| {
            if let JsRule::Pattern { value } = extra {
                value.contains(r"\s")
            } else {
                false
            }
        });

        if has_whitespace {
            self.add_terminal_token(grammar, "_WHITESPACE", r"\s+")?;
        }

        // Add external symbols
        for external in &self.grammar_js.externals {
            let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
            self.symbol_names.insert(external.name.clone(), symbol_id);
            self.next_symbol_id += 1;
        }

        Ok(())
    }
    pub(super) fn convert_rules(&mut self, grammar: &mut Grammar) -> Result<()> {
        // Clone to avoid borrow issues
        let rules: Vec<(String, JsRule)> = self
            .grammar_js
            .rules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        eprintln!("Debug: Converting {} grammar.js rules", rules.len());

        for (rule_name, rule_body) in rules {
            let lhs_symbol = *self
                .symbol_names
                .get(&rule_name)
                .context(format!("Symbol {} not found", rule_name))?;

            eprintln!(
                "Debug: Converting rule '{}' (symbol {})",
                rule_name, lhs_symbol.0
            );
            if rule_name == "source_file" {
                eprintln!("Debug: Converting source_file rule!");
                eprintln!("Debug: source_file rule body: {:?}", rule_body);
            }
            eprintln!(
                "Debug: Rule body type: {:?}",
                std::mem::discriminant(&rule_body)
            );
            self.convert_rule_body(grammar, &rule_body, lhs_symbol)?;
        }

        eprintln!(
            "Debug: After conversion, grammar has {} IR rules",
            grammar.rules.len()
        );

        // Check which symbols are referenced but have no rules
        eprintln!("Debug: Checking for symbols without rules...");
        for (name, &symbol_id) in &self.symbol_names {
            if !grammar.rules.contains_key(&symbol_id) || grammar.rules[&symbol_id].is_empty() {
                eprintln!(
                    "  WARNING: Symbol '{}' (SymbolId({})) has no rules!",
                    name, symbol_id.0
                );
            }
        }

        Ok(())
    }
}
