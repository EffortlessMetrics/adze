use anyhow::{bail, Context, Result};
use regex::Regex;
use std::collections::HashMap;

use super::{GrammarJs, Rule};

/// A more robust parser for grammar.js files
pub struct GrammarJsParserV3 {
    content: String,
}

impl GrammarJsParserV3 {
    pub fn new(content: String) -> Self {
        Self { content }
    }
    
    pub fn parse(&self) -> Result<GrammarJs> {
        // First, find the module.exports pattern
        let exports_regex = Regex::new(r"module\.exports\s*=\s*grammar\s*\(")?;
        
        let grammar_content = if let Some(mat) = exports_regex.find(&self.content) {
            // Found the start, now find the matching closing parenthesis
            let start = mat.end();
            let end = self.find_matching_paren(&self.content[start..])?;
            self.content[start..start + end].to_string()
        } else {
            bail!("Could not find module.exports = grammar(...) pattern")
        };
        
        // Parse the grammar content
        self.parse_grammar_content(&grammar_content)
    }
    
    fn parse_grammar_content(&self, content: &str) -> Result<GrammarJs> {
        let mut grammar = GrammarJs {
            name: String::new(),
            word: None,
            rules: HashMap::new(),
            extras: vec![],
            conflicts: vec![],
            externals: vec![],
            inline: vec![],
            supertypes: vec![],
            precedences: vec![],
        };
        
        // Extract name
        grammar.name = self.extract_grammar_name(content)?;
        
        // Extract word token
        grammar.word = self.extract_word_token(content);
        
        // Extract extras
        grammar.extras = self.extract_extras(content)?;
        
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
    
    fn extract_extras(&self, content: &str) -> Result<Vec<Rule>> {
        // Find extras: $ => [
        if let Some(extras_start) = content.find("extras:") {
            let after_extras = &content[extras_start + 7..]; // Skip "extras:"
            let trimmed = after_extras.trim_start();
            
            // Skip $ =>
            if let Some(arrow_pos) = trimmed.find("=>") {
                let after_arrow = trimmed[arrow_pos + 2..].trim_start();
                
                if after_arrow.starts_with('[') {
                    // Extract the array content by matching brackets
                    let array_content = self.extract_balanced_delim(&after_arrow[1..], '[', ']')?;
                    return self.parse_rule_array(&array_content);
                }
            }
        }
        
        Ok(vec![])
    }
    
    fn extract_rules(&self, content: &str) -> Result<HashMap<String, Rule>> {
        let mut rules = HashMap::new();
        
        // Find the rules: section
        if let Some(rules_start) = content.find("rules:") {
            let after_rules = &content[rules_start + 6..]; // Skip "rules:"
            
            // Skip whitespace and find the opening brace
            let trimmed = after_rules.trim_start();
            if !trimmed.starts_with('{') {
                bail!("Expected '{{' after 'rules:'");
            }
            
            // Extract the rules object content by matching braces
            let rules_content = self.extract_balanced_delim(&trimmed[1..], '{', '}')?;
            
            eprintln!("Debug: Found rules content of length {}", rules_content.len());
            
            // Parse individual rules using a more robust approach
            self.parse_rules_object(&rules_content, &mut rules)?;
        }
        
        Ok(rules)
    }
    
    fn parse_rules_object(&self, content: &str, rules: &mut HashMap<String, Rule>) -> Result<()> {
        // Use regex to find all rule definitions
        let rule_regex = Regex::new(r"(\w+):\s*\$\s*=>\s*")?;
        
        let mut _last_end = 0;
        for mat in rule_regex.find_iter(content) {
            // Extract rule name
            let rule_name = content[mat.start()..mat.end()]
                .split(':')
                .next()
                .unwrap()
                .trim()
                .to_string();
            
            // Find the end of this rule by looking for the next rule or end of object
            let rule_start = mat.end();
            let mut rule_end = content.len();
            
            // Look for the next rule
            if let Some(next_match) = rule_regex.find_at(content, rule_start) {
                // Back up to find the comma before the next rule
                let mut pos = next_match.start();
                while pos > rule_start {
                    pos -= 1;
                    if content.chars().nth(pos) == Some(',') {
                        rule_end = pos;
                        break;
                    }
                }
            }
            
            let rule_def = content[rule_start..rule_end].trim();
            let rule_def = rule_def.trim_end_matches(',');
            
            let def_preview = if rule_def.len() > 50 { 
                format!("{}...", &rule_def[..50]) 
            } else { 
                rule_def.to_string() 
            };
            eprintln!("Debug: Parsing rule '{}' with definition: {}", rule_name, def_preview);
            
            let rule = self.parse_rule(rule_def)
                .with_context(|| format!("Failed to parse rule '{}'", rule_name))?;
            
            rules.insert(rule_name, rule);
            _last_end = rule_end;
        }
        
        Ok(())
    }
    
    fn extract_balanced_delim(&self, content: &str, open: char, close: char) -> Result<String> {
        let mut depth = 1;
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();
        
        eprintln!("Debug: extract_balanced_delim called with open='{}' close='{}', content length={}", open, close, chars.len());
        
        while depth > 0 && pos < chars.len() {
            let ch = chars[pos];
            
            // Simple string handling - just skip quoted content
            if ch == '\'' || ch == '"' || ch == '`' {
                let quote = ch;
                pos += 1;
                while pos < chars.len() {
                    if chars[pos] == '\\' {
                        pos += 2; // Skip escaped char
                    } else if chars[pos] == quote {
                        pos += 1;
                        break;
                    } else {
                        pos += 1;
                    }
                }
            } else if ch == '/' && pos + 1 < chars.len() {
                // Handle regex patterns
                if pos > 0 && "[,({:;=\n ".contains(chars[pos - 1]) && chars[pos + 1] != '/' && chars[pos + 1] != '*' {
                    // Likely a regex
                    pos += 1;
                    while pos < chars.len() {
                        if chars[pos] == '\\' {
                            pos += 2;
                        } else if chars[pos] == '/' {
                            pos += 1;
                            break;
                        } else {
                            pos += 1;
                        }
                    }
                } else {
                    pos += 1;
                }
            } else {
                if ch == open {
                    depth += 1;
                } else if ch == close {
                    depth -= 1;
                }
                pos += 1;
            }
        }
        
        if depth == 0 {
            Ok(content[..pos - 1].to_string())
        } else {
            bail!("Unbalanced {} and {} in content", open, close)
        }
    }
    
    fn find_matching_paren(&self, content: &str) -> Result<usize> {
        self.extract_balanced_delim(content, '(', ')')
            .map(|s| s.len() + 1)
    }
    
    fn parse_rule(&self, rule_def: &str) -> Result<Rule> {
        let trimmed = rule_def.trim();
        
        // Handle different rule patterns
        if trimmed.starts_with("seq(") || trimmed.starts_with("choice(") || 
           trimmed.starts_with("repeat(") || trimmed.starts_with("repeat1(") ||
           trimmed.starts_with("optional(") || trimmed.starts_with("field(") ||
           trimmed.starts_with("alias(") || trimmed.starts_with("token(") ||
           trimmed.starts_with("prec(") || trimmed.starts_with("prec.left(") ||
           trimmed.starts_with("prec.right(") {
            // For now, return a placeholder
            Ok(Rule::Seq { members: vec![] })
        } else if trimmed.starts_with("$") {
            // Symbol reference
            Ok(Rule::Symbol { name: trimmed[1..].trim_start_matches('.').to_string() })
        } else if trimmed.starts_with("'") || trimmed.starts_with("\"") {
            // String literal
            let quote = &trimmed[0..1];
            if let Some(end) = trimmed[1..].find(quote) {
                Ok(Rule::String { value: trimmed[1..end + 1].to_string() })
            } else {
                bail!("Unterminated string literal")
            }
        } else if trimmed.starts_with("/") {
            // Regex pattern
            if let Some(end) = trimmed[1..].find('/') {
                Ok(Rule::Pattern { value: trimmed[1..end + 1].to_string() })
            } else {
                bail!("Unterminated regex pattern")
            }
        } else {
            // Unknown pattern - for now return a placeholder
            eprintln!("Warning: Unknown rule pattern: {}", trimmed);
            Ok(Rule::Seq { members: vec![] })
        }
    }
    
    fn parse_rule_array(&self, content: &str) -> Result<Vec<Rule>> {
        let mut rules = vec![];
        
        // Split by commas (simplified - doesn't handle nested commas)
        for part in content.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                rules.push(self.parse_rule(trimmed)?);
            }
        }
        
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let content = r#"
module.exports = grammar({
  name: 'test',
  
  rules: {
    program: $ => $.expression,
    expression: $ => 'hello'
  }
})
"#;
        
        let parser = GrammarJsParserV3::new(content.to_string());
        let result = parser.parse();
        assert!(result.is_ok());
        
        let grammar = result.unwrap();
        assert_eq!(grammar.name, "test");
        assert_eq!(grammar.rules.len(), 2);
    }
}