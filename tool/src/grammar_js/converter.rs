//! Converter from Grammar.js to Rust-sitter IR

use super::{GrammarJs, Rule as JsRule};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, 
    SymbolId, ProductionId, FieldId, RuleId,
    PrecedenceKind, Associativity, ConflictDeclaration, ConflictResolution,
    ExternalToken
};
use anyhow::{Result, Context};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Converts a Grammar.js structure to Rust-sitter IR
pub struct GrammarJsConverter {
    grammar_js: GrammarJs,
    symbol_names: HashMap<String, SymbolId>,
    pattern_symbols: HashMap<SymbolId, SymbolId>, // Maps pattern rule symbols to their token IDs
    next_symbol_id: usize,
    next_production_id: usize,
    next_field_id: usize,
    fields: IndexMap<FieldId, String>,
}

impl GrammarJsConverter {
    pub fn new(grammar_js: GrammarJs) -> Self {
        Self {
            grammar_js,
            symbol_names: HashMap::new(),
            pattern_symbols: HashMap::new(),
            next_symbol_id: 0,
            next_production_id: 0,
            next_field_id: 0,
            fields: IndexMap::new(),
        }
    }
    
    /// Convert Grammar.js to Rust-sitter Grammar IR
    pub fn convert(mut self) -> Result<Grammar> {
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
        eprintln!("DEBUG converter: Processing extras, count = {}", self.grammar_js.extras.len());
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
        
        Ok(grammar)
    }
    
    fn collect_symbols(&mut self, grammar: &mut Grammar) -> Result<()> {
        // Add all rule names as non-terminals
        for rule_name in self.grammar_js.rules.keys() {
            let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
            eprintln!("Debug: Collecting symbol '{}' as SymbolId({})", rule_name, self.next_symbol_id);
            if rule_name == "source_file" {
                eprintln!("Debug: FOUND source_file! Adding to symbol_names and rule_names as SymbolId({})", symbol_id.0);
            }
            self.symbol_names.insert(rule_name.clone(), symbol_id);
            grammar.rule_names.insert(symbol_id, rule_name.clone());
            self.next_symbol_id += 1;
        }
        
        // Add common terminal tokens
        self.add_terminal_token(grammar, "_STRING", r#""[^"]*""#)?;
        self.add_terminal_token(grammar, "_NUMBER", r"-?\d+(\.\d+)?")?;
        self.add_terminal_token(grammar, "_IDENTIFIER", r"[a-zA-Z_]\w*")?;
        
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
    
    fn add_terminal_token(&mut self, grammar: &mut Grammar, name: &str, pattern: &str) -> Result<()> {
        let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.symbol_names.insert(name.to_string(), symbol_id);
        
        grammar.tokens.insert(symbol_id, Token {
            name: name.to_string(),
            pattern: TokenPattern::Regex(pattern.to_string()),
            fragile: false,
        });
        
        self.next_symbol_id += 1;
        Ok(())
    }
    
    fn convert_rules(&mut self, grammar: &mut Grammar) -> Result<()> {
        // Clone to avoid borrow issues
        let rules: Vec<(String, JsRule)> = self.grammar_js.rules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            
        eprintln!("Debug: Converting {} grammar.js rules", rules.len());
            
        for (rule_name, rule_body) in rules {
            let lhs_symbol = *self.symbol_names.get(&rule_name)
                .context(format!("Symbol {} not found", rule_name))?;
            
            eprintln!("Debug: Converting rule '{}' (symbol {})", rule_name, lhs_symbol.0);
            if rule_name == "source_file" {
                eprintln!("Debug: Converting source_file rule!");
                eprintln!("Debug: source_file rule body: {:?}", rule_body);
            }
            eprintln!("Debug: Rule body type: {:?}", std::mem::discriminant(&rule_body));
            self.convert_rule_body(grammar, &rule_body, lhs_symbol)?;
        }
        
        eprintln!("Debug: After conversion, grammar has {} IR rules", grammar.rules.len());
        
        // Check which symbols are referenced but have no rules
        eprintln!("Debug: Checking for symbols without rules...");
        for (name, &symbol_id) in &self.symbol_names {
            if !grammar.rules.contains_key(&symbol_id) || grammar.rules[&symbol_id].is_empty() {
                eprintln!("  WARNING: Symbol '{}' (SymbolId({})) has no rules!", name, symbol_id.0);
            }
        }
        
        Ok(())
    }
    
    fn convert_rule_body(&mut self, grammar: &mut Grammar, rule: &JsRule, lhs: SymbolId) -> Result<()> {
        match rule {
            JsRule::String { value } => {
                // Create a literal token rule
                let token_id = self.get_or_create_token(grammar, value, TokenPattern::String(value.clone()));
                let rhs = vec![Symbol::Terminal(token_id)];
                self.add_rule(grammar, lhs, rhs, None, None);
            }
            
            JsRule::Pattern { value } => {
                // Create a regex token rule  
                let token_name = format!("_{}", lhs.0); // Generate token name
                let token_id = self.get_or_create_token(grammar, &token_name, TokenPattern::Regex(value.clone()));
                // Track that this symbol is actually a pattern that resolves to a token
                self.pattern_symbols.insert(lhs, token_id);
                let rhs = vec![Symbol::Terminal(token_id)];
                self.add_rule(grammar, lhs, rhs, None, None);
            }
            
            JsRule::Symbol { name } => {
                let lhs_name = self.symbol_names.iter()
                    .find(|(_, id)| **id == lhs)
                    .map(|(n, _)| n.as_str())
                    .unwrap_or("?");
                eprintln!("Debug: Converting SYMBOL rule: {} -> {}", lhs_name, name);
                
                if let Some(&symbol_id) = self.symbol_names.get(name) {
                    eprintln!("Debug: Found symbol {} with ID {}", name, symbol_id.0);
                    let rhs = vec![Symbol::NonTerminal(symbol_id)];
                    eprintln!("Debug: Creating rule SymbolId({}) -> [NonTerminal(SymbolId({}))]", lhs.0, symbol_id.0);
                    self.add_rule(grammar, lhs, rhs, None, None);
                } else {
                    eprintln!("Debug: Symbol {} not found in symbol_names!", name);
                }
            }
            
            JsRule::Seq { members } => {
                let mut rhs = Vec::new();
                for member in members {
                    if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                        rhs.push(symbol);
                    }
                }
                self.add_rule(grammar, lhs, rhs, None, None);
            }
            
            JsRule::Choice { members } => {
                // For CHOICE, we need to create rules: lhs -> member1 | lhs -> member2 | ...
                eprintln!("Debug: Converting CHOICE for {} with {} members", lhs.0, members.len());
                for (i, member) in members.iter().enumerate() {
                    // Convert each member to a symbol
                    eprintln!("Debug: Converting choice member {} for {}", i, lhs.0);
                    if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                        eprintln!("Debug: Adding rule {} -> {:?}", lhs.0, symbol);
                        // Create a rule: lhs -> symbol
                        self.add_rule(grammar, lhs, vec![symbol], None, None);
                    } else {
                        eprintln!("Debug: Failed to convert choice member {} for {}", i, lhs.0);
                    }
                }
            }
            
            JsRule::Optional { value } => {
                // Add rule with the value
                self.convert_rule_body(grammar, value, lhs)?;
                // Add empty rule
                self.add_rule(grammar, lhs, vec![], None, None);
            }
            
            JsRule::Repeat { content } => {
                // Add empty rule for repeat
                self.add_rule(grammar, lhs, vec![], None, None);
                // Add recursive rule
                self.add_repeat_rule(grammar, content, lhs, false)?;
            }
            
            JsRule::Repeat1 { content } => {
                // Add base case
                self.convert_rule_body(grammar, content, lhs)?;
                // Add recursive rule
                self.add_repeat_rule(grammar, content, lhs, true)?;
            }
            
            JsRule::Field { name, content } => {
                // Get or create field ID
                let field_id = self.get_or_create_field(name);
                
                // First, ensure the content rule is converted if it's a symbol
                if let JsRule::Symbol { name: content_name } = content.as_ref() {
                    if let Some(&content_symbol_id) = self.symbol_names.get(content_name) {
                        // Check if this symbol needs its rule converted
                        if let Some(content_rule) = self.grammar_js.rules.get(content_name).cloned() {
                            eprintln!("Debug: Converting nested rule {} for field", content_name);
                            self.convert_rule_body(grammar, &content_rule, content_symbol_id)?;
                        }
                    }
                }
                
                // Convert the content
                eprintln!("Debug: FIELD conversion - lhs: SymbolId({}), field: {}, content: {:?}", lhs.0, name, content);
                if let Some(symbol) = self.rule_to_symbol(grammar, content) {
                    eprintln!("Debug: FIELD resolved to symbol: {:?}", symbol);
                    let rule = Rule {
                        lhs,
                        rhs: vec![symbol],
                        precedence: None,
                        associativity: None,
                        fields: vec![(field_id, 0)],
                        production_id: ProductionId(self.next_production_id.try_into().unwrap()),
                    };
                    self.next_production_id += 1;
                    
                    let rule_id = RuleId(grammar.rules.len().try_into().unwrap());
                    grammar.production_ids.insert(rule_id, rule.production_id);
                    grammar.rules.entry(lhs).or_insert_with(Vec::new).push(rule);
                }
            }
            
            JsRule::Prec { value, content } => {
                let precedence = Some(PrecedenceKind::Static(*value as i16));
                self.convert_rule_with_precedence(grammar, content, lhs, precedence, None)?;
            }
            
            JsRule::PrecLeft { value, content } => {
                let precedence = Some(PrecedenceKind::Static(*value as i16));
                let associativity = Some(Associativity::Left);
                self.convert_rule_with_precedence(grammar, content, lhs, precedence, associativity)?;
            }
            
            JsRule::PrecRight { value, content } => {
                let precedence = Some(PrecedenceKind::Static(*value as i16));
                let associativity = Some(Associativity::Right);
                self.convert_rule_with_precedence(grammar, content, lhs, precedence, associativity)?;
            }
            
            _ => {
                // For other rule types, add a simple rule
                self.add_rule(grammar, lhs, vec![], None, None);
            }
        }
        
        Ok(())
    }
    
    fn get_or_create_string_token(&mut self, grammar: &mut Grammar, value: &str) -> SymbolId {
        // Check if we already have this token
        for (id, token) in &grammar.tokens {
            if let TokenPattern::String(s) = &token.pattern {
                if s == value {
                    return *id;
                }
            }
        }
        
        // Create new token
        let id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.next_symbol_id += 1;
        let token = Token {
            name: format!("\"{}\"", value),
            pattern: TokenPattern::String(value.to_string()),
            fragile: false,
        };
        grammar.tokens.insert(id, token);
        id
    }
    
    fn get_or_create_pattern_token(&mut self, grammar: &mut Grammar, pattern: &str) -> SymbolId {
        // Check if we already have this token
        for (id, token) in &grammar.tokens {
            if let TokenPattern::Regex(p) = &token.pattern {
                if p == pattern {
                    return *id;
                }
            }
        }
        
        // Create new token
        let id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.next_symbol_id += 1;
        let token = Token {
            name: format!("/{}/", pattern),
            pattern: TokenPattern::Regex(pattern.to_string()),
            fragile: false,
        };
        grammar.tokens.insert(id, token);
        id
    }
    
    fn find_extra_symbol(&self, rule: &JsRule, grammar: &Grammar) -> Option<SymbolId> {
        eprintln!("DEBUG find_extra_symbol: rule = {:?}", rule);
        match rule {
            JsRule::Symbol { name } => {
                eprintln!("  Looking for symbol '{}'", name);
                
                // First check if it's directly a token
                if let Some(&symbol_id) = self.symbol_names.get(name) {
                    eprintln!("    Found symbol '{}' with id {:?}", name, symbol_id);
                    
                    // Check if this is actually a token in the grammar
                    if grammar.tokens.contains_key(&symbol_id) {
                        eprintln!("    Symbol is a token, returning {:?}", symbol_id);
                        return Some(symbol_id);
                    }
                    
                    // If it's a rule, we need to check if it's a simple wrapper around a token
                    // For extras like Whitespace that wrap a token pattern
                    if let Some(rules) = grammar.rules.get(&symbol_id) {
                        eprintln!("    Symbol is a rule with {} alternatives", rules.len());
                        // If there's exactly one rule and it's a simple sequence with one token
                        if rules.len() == 1 && rules[0].rhs.len() == 1 {
                            if let Symbol::Terminal(token_id) = &rules[0].rhs[0] {
                                eprintln!("    Rule wraps token {:?}, using that for extra", token_id);
                                return Some(*token_id);
                            }
                        }
                    }
                }
                
                // Fallback: return the symbol itself
                let result = self.symbol_names.get(name).copied();
                eprintln!("  Symbol '{}' -> {:?}", name, result);
                result
            }
            JsRule::Pattern { value } => {
                // Look for a token with matching pattern
                eprintln!("  Looking for pattern '{}' in tokens", value);
                // Special handling for whitespace patterns
                if value.contains(r"\s") {
                    // Look for the whitespace token we added
                    if let Some(&id) = self.symbol_names.get("_WHITESPACE") {
                        eprintln!("    Found whitespace token with id {:?}", id);
                        return Some(id);
                    }
                }
                eprintln!("  Pattern '{}' not found in tokens", value);
                None
            }
            _ => {
                eprintln!("  Unhandled rule type");
                None
            }
        }
    }
    
    fn rule_to_symbol(&mut self, grammar: &mut Grammar, rule: &JsRule) -> Option<Symbol> {
        match rule {
            JsRule::Symbol { name } => {
                eprintln!("Debug: rule_to_symbol for Symbol '{}'", name);
                if let Some(&id) = self.symbol_names.get(name) {
                    eprintln!("Debug:   Found symbol ID {}", id.0);
                    // Check if this symbol is actually a pattern that maps to a token
                    if let Some(&token_id) = self.pattern_symbols.get(&id) {
                        eprintln!("Debug:   Symbol {} is a pattern, returning Terminal({})", id.0, token_id.0);
                        Some(Symbol::Terminal(token_id))
                    } else {
                        eprintln!("Debug:   Symbol {} is not a pattern, returning NonTerminal", id.0);
                        Some(Symbol::NonTerminal(id))
                    }
                } else {
                    eprintln!("Debug:   Symbol '{}' not found in symbol_names", name);
                    None
                }
            }
            JsRule::String { value } => {
                // Create inline token
                Some(Symbol::Terminal(self.get_or_create_string_token(grammar, value)))
            }
            JsRule::Pattern { value } => {
                // Create pattern token
                Some(Symbol::Terminal(self.get_or_create_pattern_token(grammar, value)))
            }
            JsRule::Field { content, .. } => {
                // For fields, return the symbol of the content
                self.rule_to_symbol(grammar, content)
            }
            _ => None, // Other types not yet handled
        }
    }
    
    fn add_rule(&mut self, grammar: &mut Grammar, lhs: SymbolId, rhs: Vec<Symbol>, 
                precedence: Option<PrecedenceKind>, associativity: Option<Associativity>) {
        eprintln!("Debug: Adding rule for SymbolId({}) -> {:?}", lhs.0, rhs);
        let rule = Rule {
            lhs,
            rhs,
            precedence,
            associativity,
            fields: vec![],
            production_id: ProductionId(self.next_production_id.try_into().unwrap()),
        };
        self.next_production_id += 1;
        
        let rule_id = RuleId(grammar.rules.len().try_into().unwrap());
        grammar.production_ids.insert(rule_id, rule.production_id);
        grammar.rules.entry(lhs).or_insert_with(Vec::new).push(rule);
    }
    
    fn add_repeat_rule(&mut self, grammar: &mut Grammar, content: &JsRule, lhs: SymbolId, _is_repeat1: bool) -> Result<()> {
        if let Some(symbol) = self.rule_to_symbol(grammar, content) {
            // Add recursive rule: lhs -> lhs symbol
            let rhs = vec![Symbol::NonTerminal(lhs), symbol];
            self.add_rule(grammar, lhs, rhs, None, None);
        }
        Ok(())
    }
    
    fn convert_rule_with_precedence(&mut self, grammar: &mut Grammar, content: &JsRule, lhs: SymbolId,
                                    precedence: Option<PrecedenceKind>, associativity: Option<Associativity>) -> Result<()> {
        match content {
            JsRule::Seq { members } => {
                let mut rhs = Vec::new();
                for member in members {
                    if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                        rhs.push(symbol);
                    }
                }
                self.add_rule(grammar, lhs, rhs, precedence, associativity);
            }
            _ => {
                if let Some(symbol) = self.rule_to_symbol(grammar, content) {
                    self.add_rule(grammar, lhs, vec![symbol], precedence, associativity);
                }
            }
        }
        Ok(())
    }
    
    fn get_or_create_field(&mut self, name: &str) -> FieldId {
        // Check if field already exists
        for (field_id, field_name) in &self.fields {
            if field_name == name {
                return *field_id;
            }
        }
        
        // Create new field
        let field_id = FieldId(self.next_field_id.try_into().unwrap());
        self.fields.insert(field_id, name.to_string());
        self.next_field_id += 1;
        field_id
    }
    
    fn get_or_create_token(&mut self, grammar: &mut Grammar, name: &str, pattern: TokenPattern) -> SymbolId {
        // Check if token already exists
        if let Some(&symbol_id) = self.symbol_names.get(name) {
            return symbol_id;
        }
        
        // Create new token
        let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.symbol_names.insert(name.to_string(), symbol_id);
        self.next_symbol_id += 1;
        
        let token = Token {
            name: name.to_string(),
            pattern,
            fragile: false,
        };
        grammar.tokens.insert(symbol_id, token);
        
        symbol_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_conversion() {
        let mut grammar_js = GrammarJs::new("test".to_string());
        
        grammar_js.rules.insert(
            "expression".to_string(),
            JsRule::Choice {
                members: vec![
                    JsRule::Symbol { name: "number".to_string() },
                    JsRule::Symbol { name: "identifier".to_string() },
                ]
            }
        );
        
        grammar_js.rules.insert(
            "number".to_string(),
            JsRule::Pattern { value: r"\d+".to_string() }
        );
        
        grammar_js.rules.insert(
            "identifier".to_string(),
            JsRule::Pattern { value: r"[a-zA-Z]+".to_string() }
        );
        
        let converter = GrammarJsConverter::new(grammar_js);
        let grammar = converter.convert().unwrap();
        
        assert_eq!(grammar.name, "test");
        assert!(!grammar.rules.is_empty());
        assert!(!grammar.tokens.is_empty());
    }
}