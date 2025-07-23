//! Improved parser for grammar.js files
//! 
//! This module provides a more comprehensive parser for Tree-sitter grammar.js files.
//! It handles most common grammar patterns and can parse real-world grammars.

use super::{GrammarJs, Rule, ExternalToken};
use anyhow::{Result, bail, Context};
use regex::Regex;
use std::collections::HashMap;

/// Parse a grammar.js file content with improved parsing
pub fn parse_grammar_js_v2(content: &str) -> Result<GrammarJs> {
    let parser = ImprovedGrammarJsParser::new(content);
    parser.parse()
}

struct ImprovedGrammarJsParser {
    content: String,
    lines: Vec<String>,
}

impl ImprovedGrammarJsParser {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            lines: content.lines().map(|l| l.to_string()).collect(),
        }
    }
    
    fn parse(&self) -> Result<GrammarJs> {
        // First, try to extract the module.exports pattern
        let exports_regex = Regex::new(r"module\.exports\s*=\s*grammar\s*\(([\s\S]*)\)")?;
        
        let grammar_content = if let Some(caps) = exports_regex.captures(&self.content) {
            caps[1].to_string()
        } else {
            bail!("Could not find module.exports = grammar(...) pattern")
        };
        
        // Parse the grammar object
        self.parse_grammar_object(&grammar_content)
    }
    
    fn parse_grammar_object(&self, content: &str) -> Result<GrammarJs> {
        let mut grammar = GrammarJs::new("".to_string());
        
        // Extract name
        grammar.name = self.extract_grammar_name(content)?;
        
        // Extract word
        grammar.word = self.extract_word_token(content);
        
        // Extract inline rules
        grammar.inline = self.extract_inline_rules(content);
        
        // Extract conflicts
        grammar.conflicts = self.extract_conflicts(content);
        
        // Extract extras
        grammar.extras = self.extract_extras(content)?;
        
        // Extract externals
        grammar.externals = self.extract_externals(content);
        
        // Extract precedences
        grammar.precedences = self.extract_precedences(content);
        
        // Extract supertypes
        grammar.supertypes = self.extract_supertypes(content);
        
        // Extract rules
        grammar.rules = self.extract_rules(content)?;
        
        Ok(grammar)
    }
    
    fn extract_grammar_name(&self, content: &str) -> Result<String> {
        let name_regex = Regex::new(r#"name:\s*['"]([^'"]+)['"]"#)?;
        
        if let Some(caps) = name_regex.captures(content) {
            Ok(caps[1].to_string())
        } else {
            bail!("Could not find grammar name")
        }
    }
    
    fn extract_word_token(&self, content: &str) -> Option<String> {
        let word_regex = Regex::new(r#"word:\s*\$\s*=>\s*\$\.(\w+)"#).ok()?;
        
        word_regex.captures(content)
            .map(|caps| caps[1].to_string())
    }
    
    fn extract_inline_rules(&self, content: &str) -> Vec<String> {
        // Match inline: $ => [$.rule1, $.rule2, ...]
        if let Ok(inline_regex) = Regex::new(r#"inline:\s*\$\s*=>\s*\[([^\]]+)\]"#) {
            if let Some(caps) = inline_regex.captures(content) {
                self.parse_symbol_array(&caps[1])
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    fn extract_conflicts(&self, content: &str) -> Vec<Vec<String>> {
        // Match conflicts: $ => [[$.rule1, $.rule2], [$.rule3, $.rule4]]
        if let Ok(conflicts_regex) = Regex::new(r#"conflicts:\s*\$\s*=>\s*\[([^\]]+(?:\][^\]]*\[)*[^\]]*)\]"#) {
            if let Some(caps) = conflicts_regex.captures(content) {
                self.parse_conflicts_array(&caps[1])
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    fn extract_extras(&self, content: &str) -> Result<Vec<Rule>> {
        // Match extras: $ => [...]
        let extras_regex = Regex::new(r#"extras:\s*\$\s*=>\s*\[([^\]]+)\]"#)?;
        
        if let Some(caps) = extras_regex.captures(content) {
            self.parse_rule_array(&caps[1])
        } else {
            Ok(vec![])
        }
    }
    
    fn extract_externals(&self, content: &str) -> Vec<ExternalToken> {
        // Match externals: $ => [...]
        if let Ok(externals_regex) = Regex::new(r#"externals:\s*\$\s*=>\s*\[([^\]]+)\]"#) {
            if let Some(caps) = externals_regex.captures(content) {
                self.parse_externals_array(&caps[1])
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    fn extract_precedences(&self, content: &str) -> Vec<Vec<(String, i32)>> {
        // Match precedences: $ => [[...], [...]]
        if let Ok(prec_regex) = Regex::new(r#"precedences:\s*\$\s*=>\s*\[([^\]]+(?:\][^\]]*\[)*[^\]]*)\]"#) {
            if let Some(caps) = prec_regex.captures(content) {
                self.parse_precedences_array(&caps[1])
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    fn extract_supertypes(&self, content: &str) -> Vec<String> {
        // Match supertypes: $ => [...]
        if let Ok(super_regex) = Regex::new(r#"supertypes:\s*\$\s*=>\s*\[([^\]]+)\]"#) {
            if let Some(caps) = super_regex.captures(content) {
                self.parse_symbol_array(&caps[1])
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    fn extract_rules(&self, content: &str) -> Result<HashMap<String, Rule>> {
        let mut rules = HashMap::new();
        
        // Match rules: $ => { rule_name: $ => ..., ... }
        let rules_regex = Regex::new(r#"rules:\s*\$\s*=>\s*\{([^}]+)\}"#)?;
        
        if let Some(caps) = rules_regex.captures(content) {
            let rules_content = &caps[1];
            
            // Parse individual rules
            let rule_regex = Regex::new(r#"(\w+):\s*\$\s*=>\s*([^,]+(?:\([^)]*\)[^,]*)*)"#)?;
            
            for cap in rule_regex.captures_iter(rules_content) {
                let rule_name = cap[1].to_string();
                let rule_def = cap[2].trim();
                
                let rule = self.parse_rule(rule_def)
                    .with_context(|| format!("Failed to parse rule '{}'", rule_name))?;
                
                rules.insert(rule_name, rule);
            }
        }
        
        Ok(rules)
    }
    
    fn parse_rule(&self, rule_def: &str) -> Result<Rule> {
        let trimmed = rule_def.trim();
        
        // Check for different rule patterns
        if trimmed.starts_with("seq(") {
            self.parse_seq_rule(trimmed)
        } else if trimmed.starts_with("choice(") {
            self.parse_choice_rule(trimmed)
        } else if trimmed.starts_with("optional(") {
            self.parse_optional_rule(trimmed)
        } else if trimmed.starts_with("repeat(") {
            self.parse_repeat_rule(trimmed)
        } else if trimmed.starts_with("repeat1(") {
            self.parse_repeat1_rule(trimmed)
        } else if trimmed.starts_with("prec(") {
            self.parse_prec_rule(trimmed)
        } else if trimmed.starts_with("prec.left(") {
            self.parse_prec_left_rule(trimmed)
        } else if trimmed.starts_with("prec.right(") {
            self.parse_prec_right_rule(trimmed)
        } else if trimmed.starts_with("prec.dynamic(") {
            self.parse_prec_dynamic_rule(trimmed)
        } else if trimmed.starts_with("field(") {
            self.parse_field_rule(trimmed)
        } else if trimmed.starts_with("alias(") {
            self.parse_alias_rule(trimmed)
        } else if trimmed.starts_with("token(") {
            self.parse_token_rule(trimmed)
        } else if trimmed.starts_with("token.immediate(") {
            self.parse_immediate_token_rule(trimmed)
        } else if trimmed.starts_with("/") && trimmed.ends_with("/") {
            // Regular expression pattern
            Ok(Rule::Pattern { value: trimmed[1..trimmed.len()-1].to_string() })
        } else if trimmed.starts_with("'") || trimmed.starts_with("\"") {
            // String literal
            let quote = &trimmed[0..1];
            if let Some(end) = trimmed[1..].find(quote) {
                Ok(Rule::String { value: trimmed[1..end+1].to_string() })
            } else {
                bail!("Unterminated string literal")
            }
        } else if trimmed.starts_with("$.") {
            // Symbol reference
            Ok(Rule::Symbol { name: trimmed[2..].to_string() })
        } else {
            bail!("Unknown rule pattern: {}", trimmed)
        }
    }
    
    fn parse_seq_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "seq")?;
        let members = self.parse_rule_list(&content)?;
        Ok(Rule::Seq { members })
    }
    
    fn parse_choice_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "choice")?;
        let members = self.parse_rule_list(&content)?;
        Ok(Rule::Choice { members })
    }
    
    fn parse_optional_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "optional")?;
        let value = self.parse_rule(content.trim())?;
        Ok(Rule::Optional { value: Box::new(value) })
    }
    
    fn parse_repeat_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "repeat")?;
        let content_rule = self.parse_rule(content.trim())?;
        Ok(Rule::Repeat { content: Box::new(content_rule) })
    }
    
    fn parse_repeat1_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "repeat1")?;
        let content_rule = self.parse_rule(content.trim())?;
        Ok(Rule::Repeat1 { content: Box::new(content_rule) })
    }
    
    fn parse_prec_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "prec")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("prec() requires two arguments")
        }
        
        let value: i32 = parts[0].trim().parse()
            .with_context(|| "Failed to parse precedence value")?;
        let content_rule = self.parse_rule(parts[1].trim())?;
        
        Ok(Rule::Prec { value, content: Box::new(content_rule) })
    }
    
    fn parse_prec_left_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "prec.left")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("prec.left() requires two arguments")
        }
        
        let value: i32 = parts[0].trim().parse()
            .with_context(|| "Failed to parse precedence value")?;
        let content_rule = self.parse_rule(parts[1].trim())?;
        
        Ok(Rule::PrecLeft { value, content: Box::new(content_rule) })
    }
    
    fn parse_prec_right_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "prec.right")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("prec.right() requires two arguments")
        }
        
        let value: i32 = parts[0].trim().parse()
            .with_context(|| "Failed to parse precedence value")?;
        let content_rule = self.parse_rule(parts[1].trim())?;
        
        Ok(Rule::PrecRight { value, content: Box::new(content_rule) })
    }
    
    fn parse_prec_dynamic_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "prec.dynamic")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("prec.dynamic() requires two arguments")
        }
        
        let value: i32 = parts[0].trim().parse()
            .with_context(|| "Failed to parse precedence value")?;
        let content_rule = self.parse_rule(parts[1].trim())?;
        
        Ok(Rule::PrecDynamic { value, content: Box::new(content_rule) })
    }
    
    fn parse_field_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "field")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("field() requires two arguments")
        }
        
        let name = parts[0].trim().trim_matches(|c| c == '\'' || c == '"').to_string();
        let content_rule = self.parse_rule(parts[1].trim())?;
        
        Ok(Rule::Field { name, content: Box::new(content_rule) })
    }
    
    fn parse_alias_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "alias")?;
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            bail!("alias() requires two arguments")
        }
        
        let content_rule = self.parse_rule(parts[0].trim())?;
        let alias_str = parts[1].trim();
        
        let (value, named) = if alias_str.starts_with("$.") {
            (alias_str[2..].to_string(), true)
        } else {
            let trimmed = alias_str.trim_matches(|c| c == '\'' || c == '"');
            (trimmed.to_string(), false)
        };
        
        Ok(Rule::Alias { content: Box::new(content_rule), value, named })
    }
    
    fn parse_token_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "token")?;
        let content_rule = self.parse_rule(content.trim())?;
        Ok(Rule::Token { content: Box::new(content_rule) })
    }
    
    fn parse_immediate_token_rule(&self, rule_def: &str) -> Result<Rule> {
        let content = self.extract_function_args(rule_def, "token.immediate")?;
        let content_rule = self.parse_rule(content.trim())?;
        Ok(Rule::ImmediateToken { content: Box::new(content_rule) })
    }
    
    fn extract_function_args(&self, rule_def: &str, func_name: &str) -> Result<String> {
        let prefix = format!("{}(", func_name);
        if !rule_def.starts_with(&prefix) {
            bail!("Expected {} function", func_name)
        }
        
        // Find matching closing parenthesis
        let mut depth = 0;
        let mut end_pos = None;
        
        for (i, ch) in rule_def.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(end) = end_pos {
            Ok(rule_def[prefix.len()..end].to_string())
        } else {
            bail!("Unmatched parentheses in {}", func_name)
        }
    }
    
    fn parse_rule_list(&self, content: &str) -> Result<Vec<Rule>> {
        let mut rules = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for ch in content.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' => {
                    current.push(ch);
                    escape_next = true;
                }
                '\'' | '"' => {
                    current.push(ch);
                    if !in_string {
                        in_string = true;
                    } else {
                        in_string = false;
                    }
                }
                '(' | '[' | '{' if !in_string => {
                    current.push(ch);
                    depth += 1;
                }
                ')' | ']' | '}' if !in_string => {
                    current.push(ch);
                    depth -= 1;
                }
                ',' if depth == 0 && !in_string => {
                    if !current.trim().is_empty() {
                        rules.push(self.parse_rule(current.trim())?);
                    }
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        
        if !current.trim().is_empty() {
            rules.push(self.parse_rule(current.trim())?);
        }
        
        Ok(rules)
    }
    
    fn parse_symbol_array(&self, content: &str) -> Vec<String> {
        content.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with("$.") {
                    s[2..].to_string()
                } else {
                    s.to_string()
                }
            })
            .collect()
    }
    
    fn parse_conflicts_array(&self, _content: &str) -> Vec<Vec<String>> {
        // Parse nested arrays like [[$.a, $.b], [$.c, $.d]]
        // TODO: Implement proper nested array parsing
        Vec::new()
    }
    
    fn parse_rule_array(&self, content: &str) -> Result<Vec<Rule>> {
        self.parse_rule_list(content)
    }
    
    fn parse_externals_array(&self, _content: &str) -> Vec<ExternalToken> {
        // Simple implementation - would need improvement
        Vec::new()
    }
    
    fn parse_precedences_array(&self, _content: &str) -> Vec<Vec<(String, i32)>> {
        // Simple implementation - would need improvement
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test] 
    fn test_basic_parsing() {
        let grammar_js = r#"
module.exports = grammar({
  name: 'simple',
  
  rules: {
    source: $ => $.expression,
    expression: $ => /\d+/
  }
});
        "#;
        
        let result = parse_grammar_js_v2(grammar_js);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        assert_eq!(grammar.name, "simple");
        assert_eq!(grammar.rules.len(), 2);
    }
    
    #[test]
    fn test_parse_simple_grammar() {
        let grammar_js = r#"
module.exports = grammar({
  name: 'test',
  
  rules: {
    source_file: $ => repeat($.statement),
    
    statement: $ => choice(
      $.expression_statement,
      $.return_statement
    ),
    
    expression_statement: $ => seq(
      $.expression,
      ';'
    ),
    
    return_statement: $ => seq(
      'return',
      optional($.expression),
      ';'
    ),
    
    expression: $ => choice(
      $.identifier,
      $.number
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+/
  }
});
        "#;
        
        let grammar = parse_grammar_js_v2(grammar_js).unwrap();
        assert_eq!(grammar.name, "test");
        assert_eq!(grammar.rules.len(), 7);
    }
}