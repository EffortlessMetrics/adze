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
    #[allow(dead_code)]
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
        // Find extras: $ => [
        if let Some(extras_start) = content.find("extras:") {
            let after_extras = &content[extras_start + 7..]; // Skip "extras:"
            let trimmed = after_extras.trim_start();
            
            // Skip $ =>
            if let Some(arrow_pos) = trimmed.find("=>") {
                let after_arrow = trimmed[arrow_pos + 2..].trim_start();
                
                if after_arrow.starts_with('[') {
                    // Extract the array content by matching brackets
                    let array_content = self.extract_balanced_brackets(&after_arrow[1..])?;
                    return self.parse_rule_array(&array_content);
                }
            }
        }
        
        Ok(vec![])
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
        
        // Find the rules: section
        if let Some(rules_start) = content.find("rules:") {
            let after_rules = &content[rules_start + 6..]; // Skip "rules:"
            
            // Skip whitespace and find the opening brace
            let trimmed = after_rules.trim_start();
            if !trimmed.starts_with('{') {
                bail!("Expected '{{' after 'rules:'");
            }
            
            // Extract the rules object content by matching braces
            let rules_content = self.extract_balanced_braces(&trimmed[1..])?;
            
            eprintln!("Debug: Found rules content of length {}", rules_content.len());
            
            // Parse individual rules from the content by finding rule patterns
            let mut remaining = rules_content.as_str();
            while !remaining.trim().is_empty() {
                // Skip whitespace
                remaining = remaining.trim_start();
                
                // Look for rule name
                let rule_regex = Regex::new(r#"^(\w+):\s*\$\s*=>\s*"#)?;
                if let Some(caps) = rule_regex.captures(remaining) {
                    let rule_name = caps[1].to_string();
                    let after_arrow = &remaining[caps[0].len()..];
                    
                    // Extract the rule definition
                    let (rule_def, rest) = self.extract_rule_definition(after_arrow)?;
                    
                    eprintln!("Debug: Found rule '{}' with definition length {}", rule_name, rule_def.len());
                    
                    let rule = self.parse_rule(&rule_def)
                        .with_context(|| format!("Failed to parse rule '{}'", rule_name))?;
                    
                    rules.insert(rule_name, rule);
                    remaining = rest;
                } else {
                    // No more rules found
                    break;
                }
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
        } else if trimmed.starts_with("/") {
            // Regular expression pattern - handle both single-line and multi-line regexes
            if let Some(end_pos) = self.find_regex_end(trimmed) {
                let pattern = trimmed[1..end_pos].to_string();
                Ok(Rule::Pattern { value: pattern })
            } else {
                bail!("Unterminated regex pattern: {}", trimmed)
            }
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
        } else if trimmed.starts_with("commaSep(") {
            // Handle commaSep helper function
            self.parse_comma_sep(trimmed, false)
        } else if trimmed.starts_with("commaSep1(") {
            // Handle commaSep1 helper function
            self.parse_comma_sep(trimmed, true)
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
    
    /// Extract content within balanced braces
    fn extract_balanced_braces(&self, content: &str) -> Result<String> {
        let mut depth = 1;
        let mut end_idx = 0;
        let chars: Vec<char> = content.chars().collect();
        
        while depth > 0 && end_idx < chars.len() {
            match chars[end_idx] {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            end_idx += 1;
        }
        
        if depth == 0 && end_idx > 0 {
            Ok(content[..end_idx - 1].to_string())
        } else {
            bail!("Unbalanced braces in content")
        }
    }
    
    /// Extract content within balanced brackets
    fn extract_balanced_brackets(&self, content: &str) -> Result<String> {
        let mut depth = 1;
        let mut end_idx = 0;
        let chars: Vec<char> = content.chars().collect();
        let mut in_string = false;
        let mut in_regex = false;
        let mut escape_next = false;
        
        while depth > 0 && end_idx < chars.len() {
            if escape_next {
                escape_next = false;
            } else if chars[end_idx] == '\\' {
                escape_next = true;
            } else if !in_regex && (chars[end_idx] == '\'' || chars[end_idx] == '"') {
                in_string = !in_string;
            } else if !in_string && chars[end_idx] == '/' && end_idx > 0 && 
                      (chars[end_idx - 1].is_whitespace() || "[,(".contains(chars[end_idx - 1])) {
                in_regex = true;
            } else if in_regex && chars[end_idx] == '/' && !escape_next {
                in_regex = false;
            } else if !in_string && !in_regex {
                match chars[end_idx] {
                    '[' => depth += 1,
                    ']' => depth -= 1,
                    _ => {}
                }
            }
            end_idx += 1;
        }
        
        if depth == 0 && end_idx > 0 {
            Ok(content[..end_idx - 1].to_string())
        } else {
            bail!("Unbalanced brackets in content")
        }
    }
    
    /// Parse commaSep/commaSep1 helper functions
    fn parse_comma_sep(&self, rule_def: &str, require_one: bool) -> Result<Rule> {
        let func_name = if require_one { "commaSep1" } else { "commaSep" };
        let content = self.extract_function_args(rule_def, func_name)?;
        
        // Parse the inner rule
        let inner_rule = self.parse_rule(&content)?;
        
        if require_one {
            // commaSep1(rule) => seq(rule, repeat(seq(',', rule)))
            Ok(Rule::Seq {
                members: vec![
                    inner_rule.clone(),
                    Rule::Repeat {
                        content: Box::new(Rule::Seq {
                            members: vec![
                                Rule::String { value: ",".to_string() },
                                inner_rule,
                            ]
                        })
                    }
                ]
            })
        } else {
            // commaSep(rule) => optional(commaSep1(rule))
            Ok(Rule::Optional {
                value: Box::new(self.parse_comma_sep(rule_def.replace("commaSep(", "commaSep1(").as_str(), true)?)
            })
        }
    }
    
    /// Find the matching closing parenthesis
    fn find_matching_paren(&self, content: &str) -> Result<usize> {
        let chars: Vec<char> = content.chars().collect();
        let mut depth = 1;
        let mut i = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut escape_next = false;
        
        while i < chars.len() && depth > 0 {
            if escape_next {
                escape_next = false;
            } else if chars[i] == '\\' {
                escape_next = true;
            } else if !in_string && (chars[i] == '\'' || chars[i] == '"' || chars[i] == '`') {
                in_string = true;
                string_char = chars[i];
            } else if in_string && chars[i] == string_char {
                in_string = false;
            } else if !in_string {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }
        
        if depth == 0 {
            Ok(i - 1) // Position of the closing paren
        } else {
            bail!("No matching closing parenthesis found")
        }
    }
    
    /// Find the end position of a regex pattern, handling escaped slashes
    fn find_regex_end(&self, content: &str) -> Option<usize> {
        let chars: Vec<char> = content.chars().collect();
        let mut i = 1; // Skip the initial /
        let mut escaped = false;
        
        while i < chars.len() {
            if escaped {
                escaped = false;
            } else if chars[i] == '\\' {
                escaped = true;
            } else if chars[i] == '/' {
                return Some(i);
            }
            i += 1;
        }
        
        None
    }
    
    /// Extract a complete rule definition (handling nested structures)
    fn extract_rule_definition<'a>(&self, content: &'a str) -> Result<(String, &'a str)> {
        let trimmed = content.trim_start();
        
        // Find the end of this rule definition
        // Rules are typically separated by commas at the top level
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_idx = 0;
        let chars: Vec<char> = trimmed.chars().collect();
        
        while end_idx < chars.len() {
            let ch = chars[end_idx];
            
            if escape_next {
                escape_next = false;
            } else if ch == '\\' {
                escape_next = true;
            } else if ch == '\'' || ch == '"' {
                in_string = !in_string;
            } else if !in_string {
                match ch {
                    '(' | '{' | '[' => depth += 1,
                    ')' | '}' | ']' => depth -= 1,
                    ',' if depth == 0 => {
                        // Found the separator
                        return Ok((trimmed[..end_idx].trim().to_string(), &content[end_idx + 1..]));
                    }
                    _ => {}
                }
            }
            end_idx += 1;
        }
        
        // No comma found, this is the last rule
        Ok((content.trim().to_string(), ""))
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