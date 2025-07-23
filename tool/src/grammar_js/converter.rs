//! Converter from Grammar.js to Rust-sitter IR

use super::{GrammarJs, Rule as JsRule};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, 
    SymbolId, ProductionId, FieldId, RuleId,
    PrecedenceKind, Associativity, ConflictDeclaration, ConflictResolution
};
use anyhow::{Result, Context};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Converts a Grammar.js structure to Rust-sitter IR
pub struct GrammarJsConverter {
    grammar_js: GrammarJs,
    symbol_names: HashMap<String, SymbolId>,
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
            fields: IndexMap::new(),
            supertypes: Vec::new(),
            inline_rules: Vec::new(),
            alias_sequences: IndexMap::new(),
            production_ids: IndexMap::new(),
            max_alias_sequence_length: 0,
            rule_names: IndexMap::new(),
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
        
        // Copy fields
        grammar.fields = self.fields.clone();
        
        Ok(grammar)
    }
    
    fn collect_symbols(&mut self, grammar: &mut Grammar) -> Result<()> {
        // Add all rule names as non-terminals
        for rule_name in self.grammar_js.rules.keys() {
            let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
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
            self.convert_rule_body(grammar, &rule_body, lhs_symbol)?;
        }
        
        eprintln!("Debug: After conversion, grammar has {} IR rules", grammar.rules.len());
        
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
                let rhs = vec![Symbol::Terminal(token_id)];
                self.add_rule(grammar, lhs, rhs, None, None);
            }
            
            JsRule::Symbol { name } => {
                if let Some(&symbol_id) = self.symbol_names.get(name) {
                    let rhs = vec![Symbol::NonTerminal(symbol_id)];
                    self.add_rule(grammar, lhs, rhs, None, None);
                }
            }
            
            JsRule::Seq { members } => {
                let mut rhs = Vec::new();
                for member in members {
                    if let Some(symbol) = self.rule_to_symbol(member) {
                        rhs.push(symbol);
                    }
                }
                self.add_rule(grammar, lhs, rhs, None, None);
            }
            
            JsRule::Choice { members } => {
                // Each choice member becomes a separate rule
                for member in members {
                    self.convert_rule_body(grammar, member, lhs)?;
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
                
                // Convert the content
                if let Some(symbol) = self.rule_to_symbol(content) {
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
                    grammar.rules.insert(lhs, rule);
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
    
    fn rule_to_symbol(&self, rule: &JsRule) -> Option<Symbol> {
        match rule {
            JsRule::Symbol { name } => {
                self.symbol_names.get(name).map(|&id| Symbol::NonTerminal(id))
            }
            _ => None, // Simplified for MVP
        }
    }
    
    fn add_rule(&mut self, grammar: &mut Grammar, lhs: SymbolId, rhs: Vec<Symbol>, 
                precedence: Option<PrecedenceKind>, associativity: Option<Associativity>) {
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
        grammar.rules.insert(lhs, rule);
    }
    
    fn add_repeat_rule(&mut self, grammar: &mut Grammar, content: &JsRule, lhs: SymbolId, _is_repeat1: bool) -> Result<()> {
        if let Some(symbol) = self.rule_to_symbol(content) {
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
                    if let Some(symbol) = self.rule_to_symbol(member) {
                        rhs.push(symbol);
                    }
                }
                self.add_rule(grammar, lhs, rhs, precedence, associativity);
            }
            _ => {
                if let Some(symbol) = self.rule_to_symbol(content) {
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