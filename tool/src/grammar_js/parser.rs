//! Parser for grammar.js files
//! 
//! This module provides a simple parser for Tree-sitter grammar.js files.
//! It starts with regex-based parsing for MVP and can be upgraded to a full JS parser later.

use super::{GrammarJs, Rule};
use anyhow::{Result, bail};
use regex::Regex;
use std::collections::HashMap;

/// Parse a grammar.js file content
pub fn parse_grammar_js(content: &str) -> Result<GrammarJs> {
    // For MVP, we'll use a simplified parser that handles common patterns
    // This will be replaced with a proper JavaScript parser in the future
    
    let parser = SimpleGrammarJsParser::new(content);
    parser.parse()
}

struct SimpleGrammarJsParser {
    content: String,
}

impl SimpleGrammarJsParser {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
    
    fn parse(&self) -> Result<GrammarJs> {
        // Extract the grammar name
        let name = self.extract_name()?;
        let mut grammar = GrammarJs::new(name);
        
        // Extract word token
        grammar.word = self.extract_word();
        
        // Extract inline rules
        grammar.inline = self.extract_inline();
        
        // Extract conflicts
        grammar.conflicts = self.extract_conflicts();
        
        // Extract extras
        grammar.extras = self.extract_extras()?;
        
        // Extract rules
        grammar.rules = self.extract_rules()?;
        
        // Validate the grammar
        grammar.validate()?;
        
        Ok(grammar)
    }
    
    fn extract_name(&self) -> Result<String> {
        let name_regex = Regex::new(r#"name:\s*['"]([^'"]+)['"]"#)?;
        
        if let Some(caps) = name_regex.captures(&self.content) {
            Ok(caps[1].to_string())
        } else {
            bail!("Could not find grammar name")
        }
    }
    
    fn extract_word(&self) -> Option<String> {
        let word_regex = Regex::new(r#"word:\s*\$\s*=>\s*\$\.(\w+)"#).ok()?;
        
        word_regex.captures(&self.content)
            .map(|caps| caps[1].to_string())
    }
    
    fn extract_inline(&self) -> Vec<String> {
        let inline_regex = Regex::new(r#"inline:\s*\$\s*=>\s*\[([\s\S]*?)\]"#).ok();
        
        if let Some(regex) = inline_regex {
            if let Some(caps) = regex.captures(&self.content) {
                let inline_content = &caps[1];
                return self.parse_array_of_symbols(inline_content);
            }
        }
        
        Vec::new()
    }
    
    fn extract_conflicts(&self) -> Vec<Vec<String>> {
        let conflicts_regex = Regex::new(r#"conflicts:\s*\$\s*=>\s*\[([\s\S]*?)\]"#).ok();
        
        if let Some(regex) = conflicts_regex {
            if let Some(caps) = regex.captures(&self.content) {
                let conflicts_content = &caps[1];
                return self.parse_conflicts_array(conflicts_content);
            }
        }
        
        Vec::new()
    }
    
    fn extract_extras(&self) -> Result<Vec<Rule>> {
        // For MVP, we'll just extract whitespace and comments
        let mut extras = Vec::new();
        
        // Check for common extras pattern
        if self.content.contains("extras: $ => [") {
            // Add whitespace by default
            extras.push(Rule::Pattern { 
                value: r"\s".to_string() 
            });
        }
        
        Ok(extras)
    }
    
    fn extract_rules(&self) -> Result<HashMap<String, Rule>> {
        let mut rules = HashMap::new();
        
        // Extract rules section
        let rules_regex = Regex::new(r#"rules:\s*\{([\s\S]*?)\n\s*\}"#)?;
        
        if let Some(caps) = rules_regex.captures(&self.content) {
            let rules_content = &caps[1];
            
            // Parse individual rules (simplified for MVP)
            let rule_regex = Regex::new(r#"(\w+):\s*\$\s*=>\s*([\s\S]*?)(?=,\s*\w+:|$)"#)?;
            
            for caps in rule_regex.captures_iter(rules_content) {
                let rule_name = caps[1].to_string();
                let rule_body = &caps[2];
                
                let rule = self.parse_rule_body(rule_body)?;
                rules.insert(rule_name, rule);
            }
        }
        
        Ok(rules)
    }
    
    fn parse_rule_body(&self, body: &str) -> Result<Rule> {
        let trimmed = body.trim();
        
        // Check for string literal
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            let value = trimmed[1..trimmed.len()-1].to_string();
            return Ok(Rule::String { value });
        }
        
        // Check for regex pattern
        if trimmed.starts_with('/') && trimmed.contains('/') {
            if let Some(end) = trimmed[1..].find('/') {
                let value = trimmed[1..=end].to_string();
                return Ok(Rule::Pattern { value });
            }
        }
        
        // Check for symbol reference
        if trimmed.starts_with("$.") {
            let symbol_name = trimmed[2..].trim();
            return Ok(Rule::Symbol { 
                name: symbol_name.to_string() 
            });
        }
        
        // Check for seq
        if trimmed.contains("seq(") {
            return Ok(Rule::Seq { 
                members: vec![] // Simplified for MVP
            });
        }
        
        // Check for choice
        if trimmed.contains("choice(") {
            return Ok(Rule::Choice { 
                members: vec![] // Simplified for MVP
            });
        }
        
        // Check for optional
        if trimmed.contains("optional(") {
            return Ok(Rule::Optional {
                value: Box::new(Rule::Blank)
            });
        }
        
        // Check for repeat
        if trimmed.contains("repeat(") && !trimmed.contains("repeat1(") {
            return Ok(Rule::Repeat {
                content: Box::new(Rule::Blank)
            });
        }
        
        // Check for repeat1
        if trimmed.contains("repeat1(") {
            return Ok(Rule::Repeat1 {
                content: Box::new(Rule::Blank)
            });
        }
        
        // Default to blank for now
        Ok(Rule::Blank)
    }
    
    fn parse_array_of_symbols(&self, content: &str) -> Vec<String> {
        let symbol_regex = Regex::new(r#"\$\.(\w+)"#).ok();
        
        if let Some(regex) = symbol_regex {
            regex.captures_iter(content)
                .map(|caps| caps[1].to_string())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    fn parse_conflicts_array(&self, content: &str) -> Vec<Vec<String>> {
        // Simplified parsing for MVP
        let mut conflicts = Vec::new();
        
        // Look for arrays within the conflicts array
        let array_regex = Regex::new(r#"\[([\s\S]*?)\]"#).ok();
        
        if let Some(regex) = array_regex {
            for caps in regex.captures_iter(content) {
                let conflict_set = self.parse_array_of_symbols(&caps[1]);
                if !conflict_set.is_empty() {
                    conflicts.push(conflict_set);
                }
            }
        }
        
        conflicts
    }
}

/// Parse a simple grammar.js example for testing
pub fn parse_json_grammar() -> Result<GrammarJs> {
    let grammar_content = r#"
module.exports = grammar({
  name: 'json',

  extras: $ => [
    /\s/
  ],

  rules: {
    document: $ => $._value,

    _value: $ => choice(
      $.object,
      $.array,
      $.number,
      $.string,
      $.true,
      $.false,
      $.null
    ),

    object: $ => seq(
      '{',
      optional(commaSep1($.pair)),
      '}'
    ),

    pair: $ => seq(
      field('key', $.string),
      ':',
      field('value', $._value)
    ),

    array: $ => seq(
      '[',
      optional(commaSep1($._value)),
      ']'
    ),

    string: $ => seq(
      '"',
      repeat(choice(
        /[^"\\]/,
        /\\./
      )),
      '"'
    ),

    number: $ => /\-?\d+(\.\d+)?([eE][+-]?\d+)?/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null'
  }
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)))
}
"#;

    parse_grammar_js(grammar_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_name() {
        let parser = SimpleGrammarJsParser::new(r#"
            module.exports = grammar({
                name: 'test_lang',
                rules: {}
            });
        "#);
        
        assert_eq!(parser.extract_name().unwrap(), "test_lang");
    }
    
    #[test]
    fn test_extract_word() {
        let parser = SimpleGrammarJsParser::new(r#"
            module.exports = grammar({
                name: 'test',
                word: $ => $.identifier,
                rules: {}
            });
        "#);
        
        assert_eq!(parser.extract_word(), Some("identifier".to_string()));
    }
    
    #[test]
    fn test_parse_simple_grammar() {
        let content = r#"
module.exports = grammar({
  name: 'simple',
  
  rules: {
    source_file: $ => 'hello',
    identifier: $ => /[a-z]+/
  }
});
"#;
        
        let grammar = parse_grammar_js(content).unwrap();
        assert_eq!(grammar.name, "simple");
        assert_eq!(grammar.rules.len(), 2);
    }
}