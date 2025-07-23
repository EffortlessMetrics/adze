// Query language parser
// Parses Tree-sitter's S-expression query syntax

use super::ast::*;
use rust_sitter_ir::{Grammar, SymbolId};
use std::collections::HashMap;

/// Parser for tree-sitter query language
pub struct QueryParser<'a> {
    input: &'a str,
    position: usize,
    grammar: &'a Grammar,
    capture_names: HashMap<String, u32>,
    next_capture_id: u32,
}

impl<'a> QueryParser<'a> {
    /// Create a new query parser
    pub fn new(input: &'a str, grammar: &'a Grammar) -> Self {
        QueryParser {
            input,
            position: 0,
            grammar,
            capture_names: HashMap::new(),
            next_capture_id: 0,
        }
    }
    
    /// Parse the query
    pub fn parse(mut self) -> Result<Query, QueryError> {
        let mut patterns = Vec::new();
        let mut property_settings = Vec::new();
        let mut property_predicates = Vec::new();
        
        self.skip_whitespace();
        
        while !self.is_at_end() {
            let start_byte = self.position;
            
            // Each pattern starts with a '('
            if !self.consume_char('(') {
                return Err(self.syntax_error("Expected '(' to start pattern"));
            }
            
            // Parse the pattern
            let root = self.parse_pattern_node()?;
            let mut predicates = Vec::new();
            
            // Parse predicates
            self.skip_whitespace();
            while self.peek_char() == Some('(') {
                if self.peek_ahead("(#") {
                    self.consume_char('(');
                    self.consume_char('#');
                    
                    let predicate = self.parse_predicate()?;
                    
                    // Handle property settings and predicates
                    match &predicate {
                        Predicate::Set { property, capture, value } => {
                            property_settings.push(PropertySetting {
                                key: property.clone(),
                                value: value.clone(),
                                capture: *capture,
                            });
                        }
                        Predicate::Is { property, capture, value } => {
                            property_predicates.push(PropertyPredicate {
                                key: property.clone(),
                                value: value.clone(),
                                capture: *capture,
                                is_positive: true,
                            });
                        }
                        Predicate::IsNot { property, capture, value } => {
                            property_predicates.push(PropertyPredicate {
                                key: property.clone(),
                                value: value.clone(),
                                capture: *capture,
                                is_positive: false,
                            });
                        }
                        _ => predicates.push(predicate),
                    }
                    
                    if !self.consume_char(')') {
                        return Err(self.syntax_error("Expected ')' after predicate"));
                    }
                } else {
                    break;
                }
                self.skip_whitespace();
            }
            
            if !self.consume_char(')') {
                return Err(self.syntax_error("Expected ')' to close pattern"));
            }
            
            patterns.push(Pattern {
                root,
                predicates,
                start_byte,
            });
            
            self.skip_whitespace();
        }
        
        Ok(Query {
            source: self.input.to_string(),
            patterns,
            capture_names: self.capture_names,
            property_settings,
            property_predicates,
        })
    }
    
    /// Parse a pattern node
    fn parse_pattern_node(&mut self) -> Result<PatternNode, QueryError> {
        self.skip_whitespace();
        
        // Check for opening paren (for grouped nodes)
        let has_paren = self.consume_char('(');
        
        // Parse node type
        let node_type = self.parse_identifier()?;
        
        // Look up symbol in grammar
        let symbol = self.find_symbol(&node_type)?;
        let is_named = self.is_named_symbol(symbol);
        
        let mut node = PatternNode::new(symbol, is_named);
        
        // Parse capture name
        self.skip_whitespace();
        if self.peek_char() == Some('@') {
            self.consume_char('@');
            let capture_name = self.parse_identifier()?;
            let capture_id = self.get_or_create_capture(&capture_name);
            node.capture = Some(capture_id);
        }
        
        // Parse quantifier
        self.skip_whitespace();
        match self.peek_char() {
            Some('?') => {
                self.advance();
                node.quantifier = Quantifier::Optional;
            }
            Some('+') => {
                self.advance();
                node.quantifier = Quantifier::Plus;
            }
            Some('*') => {
                self.advance();
                node.quantifier = Quantifier::Star;
            }
            _ => {}
        }
        
        if has_paren {
            // Parse children and fields
            self.skip_whitespace();
            while self.peek_char() != Some(')') {
                if self.is_at_end() {
                    return Err(self.syntax_error("Unexpected end of input"));
                }
                
                // Check for field
                if self.peek_char() == Some('.') || self.peek_identifier().is_ok() {
                    if let Ok(field_name) = self.peek_field_name() {
                        self.parse_identifier()?; // consume field name
                        self.consume_char(':');
                        let field_node = self.parse_pattern_node()?;
                        node.add_field(field_name, field_node);
                    } else {
                        // Regular child
                        let child = self.parse_pattern_child()?;
                        node.add_child(child);
                    }
                } else {
                    // Regular child
                    let child = self.parse_pattern_child()?;
                    node.add_child(child);
                }
                
                self.skip_whitespace();
            }
            
            if !self.consume_char(')') {
                return Err(self.syntax_error("Expected ')' to close node"));
            }
        }
        
        Ok(node)
    }
    
    /// Parse a pattern child (node or token)
    fn parse_pattern_child(&mut self) -> Result<PatternChild, QueryError> {
        self.skip_whitespace();
        
        if self.peek_char() == Some('"') {
            // String literal (anonymous token)
            let token = self.parse_string()?;
            Ok(PatternChild::Token(token))
        } else {
            // Pattern node
            let node = self.parse_pattern_node()?;
            Ok(PatternChild::Node(node))
        }
    }
    
    /// Parse a predicate
    fn parse_predicate(&mut self) -> Result<Predicate, QueryError> {
        let name = self.parse_identifier()?;
        
        match name.as_str() {
            "eq?" => self.parse_eq_predicate(),
            "not-eq?" => self.parse_not_eq_predicate(),
            "match?" => self.parse_match_predicate(),
            "not-match?" => self.parse_not_match_predicate(),
            "set!" => self.parse_set_directive(),
            "is?" => self.parse_is_predicate(),
            "is-not?" => self.parse_is_not_predicate(),
            "any-of?" => self.parse_any_of_predicate(),
            _ => self.parse_custom_predicate(name),
        }
    }
    
    /// Parse #eq? predicate
    fn parse_eq_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let capture1 = self.parse_capture_ref()?;
        self.skip_whitespace();
        
        if self.peek_char() == Some('@') {
            let capture2 = self.parse_capture_ref()?;
            Ok(Predicate::Eq {
                capture1,
                capture2: Some(capture2),
                value: None,
            })
        } else {
            let value = self.parse_string()?;
            Ok(Predicate::Eq {
                capture1,
                capture2: None,
                value: Some(value),
            })
        }
    }
    
    // Similar implementations for other predicates...
    fn parse_not_eq_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let capture1 = self.parse_capture_ref()?;
        self.skip_whitespace();
        
        if self.peek_char() == Some('@') {
            let capture2 = self.parse_capture_ref()?;
            Ok(Predicate::NotEq {
                capture1,
                capture2: Some(capture2),
                value: None,
            })
        } else {
            let value = self.parse_string()?;
            Ok(Predicate::NotEq {
                capture1,
                capture2: None,
                value: Some(value),
            })
        }
    }
    
    fn parse_match_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let capture = self.parse_capture_ref()?;
        self.skip_whitespace();
        let regex = self.parse_string()?;
        Ok(Predicate::Match { capture, regex })
    }
    
    fn parse_not_match_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let capture = self.parse_capture_ref()?;
        self.skip_whitespace();
        let regex = self.parse_string()?;
        Ok(Predicate::NotMatch { capture, regex })
    }
    
    fn parse_set_directive(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let property = self.parse_identifier()?;
        self.skip_whitespace();
        
        let (capture, value) = if self.peek_char() == Some('@') {
            (Some(self.parse_capture_ref()?), None)
        } else if self.peek_char() == Some('"') {
            (None, Some(self.parse_string()?))
        } else {
            (None, None)
        };
        
        Ok(Predicate::Set { property, capture, value })
    }
    
    fn parse_is_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let property = self.parse_identifier()?;
        self.skip_whitespace();
        
        let (capture, value) = if self.peek_char() == Some('@') {
            (Some(self.parse_capture_ref()?), None)
        } else if self.peek_char() == Some('"') {
            (None, Some(self.parse_string()?))
        } else {
            (None, None)
        };
        
        Ok(Predicate::Is { property, capture, value })
    }
    
    fn parse_is_not_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let property = self.parse_identifier()?;
        self.skip_whitespace();
        
        let (capture, value) = if self.peek_char() == Some('@') {
            (Some(self.parse_capture_ref()?), None)
        } else if self.peek_char() == Some('"') {
            (None, Some(self.parse_string()?))
        } else {
            (None, None)
        };
        
        Ok(Predicate::IsNot { property, capture, value })
    }
    
    fn parse_any_of_predicate(&mut self) -> Result<Predicate, QueryError> {
        self.skip_whitespace();
        let capture = self.parse_capture_ref()?;
        let mut values = Vec::new();
        
        self.skip_whitespace();
        while self.peek_char() == Some('"') {
            values.push(self.parse_string()?);
            self.skip_whitespace();
        }
        
        Ok(Predicate::AnyOf { capture, values })
    }
    
    fn parse_custom_predicate(&mut self, name: String) -> Result<Predicate, QueryError> {
        let mut args = Vec::new();
        
        self.skip_whitespace();
        while self.peek_char() != Some(')') {
            if self.peek_char() == Some('@') {
                let capture = self.parse_capture_ref()?;
                args.push(PredicateArg::Capture(capture));
            } else {
                let string = self.parse_string()?;
                args.push(PredicateArg::String(string));
            }
            self.skip_whitespace();
        }
        
        Ok(Predicate::Custom { name, args })
    }
    
    // Helper methods
    
    fn parse_capture_ref(&mut self) -> Result<u32, QueryError> {
        if !self.consume_char('@') {
            return Err(self.syntax_error("Expected '@' for capture reference"));
        }
        let name = self.parse_identifier()?;
        self.capture_names.get(&name)
            .copied()
            .ok_or_else(|| QueryError::InvalidCapture(name))
    }
    
    fn get_or_create_capture(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.capture_names.get(name) {
            id
        } else {
            let id = self.next_capture_id;
            self.capture_names.insert(name.to_string(), id);
            self.next_capture_id += 1;
            id
        }
    }
    
    fn find_symbol(&self, name: &str) -> Result<SymbolId, QueryError> {
        // Try to find in tokens
        for (&id, token) in &self.grammar.tokens {
            if token.name == name {
                return Ok(id);
            }
        }
        
        // Try to find in rules
        for (&id, _) in &self.grammar.rules {
            if let Some(rule_name) = self.grammar.rule_names.get(&id) {
                if rule_name == name {
                    return Ok(id);
                }
            }
        }
        
        Err(QueryError::UndefinedNodeType(name.to_string()))
    }
    
    fn is_named_symbol(&self, _symbol: SymbolId) -> bool {
        // For now, assume all rule symbols are named
        // and token symbols starting with uppercase are named
        true
    }
    
    fn parse_identifier(&mut self) -> Result<String, QueryError> {
        self.skip_whitespace();
        let start = self.position;
        
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '!' || ch == '?' {
                self.advance();
            } else {
                break;
            }
        }
        
        if self.position == start {
            return Err(self.syntax_error("Expected identifier"));
        }
        
        Ok(self.input[start..self.position].to_string())
    }
    
    fn parse_string(&mut self) -> Result<String, QueryError> {
        if !self.consume_char('"') {
            return Err(self.syntax_error("Expected '\"' to start string"));
        }
        
        let mut result = String::new();
        
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.peek_char() {
                    self.advance();
                    match escaped {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        _ => {
                            result.push('\\');
                            result.push(escaped);
                        }
                    }
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        
        Err(self.syntax_error("Unterminated string"))
    }
    
    fn peek_field_name(&mut self) -> Result<String, QueryError> {
        let saved_pos = self.position;
        let result = self.parse_identifier();
        
        if result.is_ok() && self.peek_char() == Some(':') {
            self.position = saved_pos;
            result
        } else {
            self.position = saved_pos;
            Err(QueryError::SyntaxError {
                position: self.position,
                message: "Not a field name".to_string(),
            })
        }
    }
    
    fn peek_identifier(&mut self) -> Result<String, QueryError> {
        let saved_pos = self.position;
        let result = self.parse_identifier();
        self.position = saved_pos;
        result
    }
    
    fn peek_ahead(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || ch == ';' {
                self.advance();
                if ch == ';' {
                    // Skip comment to end of line
                    while let Some(ch) = self.peek_char() {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
    
    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }
    
    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn advance(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.position += ch.len_utf8();
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
    fn syntax_error(&self, message: &str) -> QueryError {
        QueryError::SyntaxError {
            position: self.position,
            message: message.to_string(),
        }
    }
}