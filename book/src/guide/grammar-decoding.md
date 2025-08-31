# Grammar Decoding

This guide shows you how to dynamically load and work with Tree-sitter grammars using rust-sitter's grammar decoding capabilities. This is particularly useful for tools, language servers, and applications that need to work with multiple languages at runtime.

## What is Grammar Decoding?

Grammar decoding allows you to:
- **Extract grammar rules** from compiled Tree-sitter languages
- **Reconstruct parse tables** for use with rust-sitter's GLR parser
- **Load languages dynamically** without compile-time grammar definitions
- **Analyze existing grammars** for tooling and research

## Basic Usage

### Prerequisites

Add rust-sitter with the pure-rust feature:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["pure-rust"] }
# Language crates you want to decode
rust-sitter-python = "0.1"
rust-sitter-javascript = "0.1"
```

### Decoding Your First Grammar

```rust
use rust_sitter::decoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get a compiled Tree-sitter language
    let lang = rust_sitter_python::get_language();
    
    // Decode the grammar
    let grammar = decoder::decode_grammar(lang);
    
    // Explore the grammar
    println!("Grammar: {}", grammar.name);
    println!("Rules: {}", grammar.rules.len());
    println!("Fields: {}", grammar.fields.len());
    println!("External tokens: {}", grammar.externals.len());
    
    // List some rules
    for (symbol_id, rules) in grammar.rules.iter().take(5) {
        println!("Symbol {}: {} rules", symbol_id.0, rules.len());
    }
    
    Ok(())
}
```

### Decoding Parse Tables

To use the grammar with rust-sitter's GLR parser, you also need the parse table:

```rust
use rust_sitter::{decoder, glr_parser::GLRParser};

fn decode_and_parse() -> Result<(), Box<dyn std::error::Error>> {
    let lang = rust_sitter_python::get_language();
    
    // Decode both grammar and parse table
    let grammar = decoder::decode_grammar(lang);
    let table = decoder::decode_parse_table(lang);
    
    // Create a GLR parser
    let parser = GLRParser::new(grammar, table);
    
    // Parse some Python code
    let code = r#"
def hello(name):
    print(f"Hello, {name}!")
    return True
"#;
    
    let result = parser.parse(code)?;
    println!("Parse successful: {}", result.is_success());
    
    Ok(())
}
```

## Working with Grammar Information

### Exploring Symbol Metadata

```rust
use rust_sitter::decoder;

fn explore_symbols() {
    let lang = rust_sitter_python::get_language();
    let grammar = decoder::decode_grammar(lang);
    
    // Find specific symbols
    let function_def = grammar.rules.iter()
        .find(|(_, rules)| {
            rules.iter().any(|rule| {
                // Look for function definition patterns
                rule.rhs.len() >= 3
            })
        });
    
    if let Some((symbol_id, rules)) = function_def {
        println!("Found function-like symbol: {}", symbol_id.0);
        for rule in rules {
            println!("  Production: {} -> {} symbols", 
                rule.lhs.0, rule.rhs.len());
        }
    }
}
```

### Field Information

```rust
use rust_sitter::decoder;

fn explore_fields() {
    let lang = rust_sitter_python::get_language();
    let grammar = decoder::decode_grammar(lang);
    
    println!("Available fields:");
    for (field_id, name) in &grammar.fields {
        println!("  {}: {}", field_id.0, name);
    }
    
    // Find rules that use fields
    let mut field_usage = std::collections::HashMap::new();
    for rules in grammar.rules.values() {
        for rule in rules {
            for (pos, field_id) in &rule.field_map {
                *field_usage.entry(*field_id).or_insert(0) += 1;
            }
        }
    }
    
    println!("\nField usage:");
    for (field_id, count) in field_usage {
        if let Some(name) = grammar.fields.get(&field_id) {
            println!("  {}: {} uses", name, count);
        }
    }
}
```

### Token Patterns

```rust
use rust_sitter::decoder;

fn explore_tokens() {
    let lang = rust_sitter_python::get_language();
    let grammar = decoder::decode_grammar(lang);
    
    println!("Token information:");
    for (symbol_id, token) in &grammar.tokens {
        println!("  Token {}: {} ({:?})", 
            symbol_id.0, token.name, token.pattern);
    }
    
    // Show external tokens (if any)
    if !grammar.externals.is_empty() {
        println!("\nExternal tokens:");
        for external in &grammar.externals {
            println!("  {}: {}", external.name, external.value);
        }
    }
}
```

## Multi-Language Support

### Loading Multiple Languages

```rust
use rust_sitter::decoder;
use std::collections::HashMap;

struct LanguageRegistry {
    grammars: HashMap<String, rust_sitter_ir::Grammar>,
    tables: HashMap<String, rust_sitter_glr_core::ParseTable>,
}

impl LanguageRegistry {
    fn new() -> Self {
        Self {
            grammars: HashMap::new(),
            tables: HashMap::new(),
        }
    }
    
    fn register_language(&mut self, name: &str, lang: &rust_sitter::TSLanguage) {
        let grammar = decoder::decode_grammar(lang);
        let table = decoder::decode_parse_table(lang);
        
        self.grammars.insert(name.to_string(), grammar);
        self.tables.insert(name.to_string(), table);
    }
    
    fn parse(&self, language: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let grammar = self.grammars.get(language).ok_or("Language not registered")?;
        let table = self.tables.get(language).ok_or("Parse table not found")?;
        
        let parser = rust_sitter::glr_parser::GLRParser::new(grammar.clone(), table.clone());
        let result = parser.parse(code)?;
        
        println!("Parsed {} code successfully: {}", language, result.is_success());
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = LanguageRegistry::new();
    
    // Register languages
    registry.register_language("python", rust_sitter_python::get_language());
    registry.register_language("javascript", rust_sitter_javascript::get_language());
    
    // Parse different languages
    registry.parse("python", "def hello(): pass")?;
    registry.parse("javascript", "function hello() { return 42; }")?;
    
    Ok(())
}
```

## Advanced Usage

### Grammar Analysis Tools

```rust
use rust_sitter::decoder;
use std::collections::{HashMap, HashSet};

struct GrammarAnalyzer {
    grammar: rust_sitter_ir::Grammar,
}

impl GrammarAnalyzer {
    fn new(lang: &rust_sitter::TSLanguage) -> Self {
        Self {
            grammar: decoder::decode_grammar(lang),
        }
    }
    
    fn analyze_complexity(&self) -> GrammarStats {
        let mut stats = GrammarStats::default();
        
        for rules in self.grammar.rules.values() {
            for rule in rules {
                stats.total_productions += 1;
                stats.max_rhs_length = stats.max_rhs_length.max(rule.rhs.len());
                
                if rule.field_map.is_empty() {
                    stats.productions_without_fields += 1;
                }
                
                if rule.precedence.is_some() {
                    stats.productions_with_precedence += 1;
                }
            }
        }
        
        stats.total_symbols = self.grammar.rules.len();
        stats.total_tokens = self.grammar.tokens.len();
        stats.total_fields = self.grammar.fields.len();
        
        stats
    }
    
    fn find_recursive_rules(&self) -> Vec<rust_sitter_ir::SymbolId> {
        let mut recursive = Vec::new();
        
        for (symbol_id, rules) in &self.grammar.rules {
            for rule in rules {
                // Check if this rule references itself
                for rhs_symbol in &rule.rhs {
                    if let rust_sitter_ir::Symbol::NonTerminal(ref_id) = rhs_symbol {
                        if ref_id == symbol_id {
                            recursive.push(*symbol_id);
                            break;
                        }
                    }
                }
            }
        }
        
        recursive
    }
    
    fn find_ambiguous_rules(&self) -> Vec<rust_sitter_ir::SymbolId> {
        self.grammar.rules.iter()
            .filter(|(_, rules)| rules.len() > 1)
            .map(|(symbol_id, _)| *symbol_id)
            .collect()
    }
}

#[derive(Default, Debug)]
struct GrammarStats {
    total_symbols: usize,
    total_tokens: usize,
    total_fields: usize,
    total_productions: usize,
    max_rhs_length: usize,
    productions_without_fields: usize,
    productions_with_precedence: usize,
}

fn analyze_grammar() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = GrammarAnalyzer::new(rust_sitter_python::get_language());
    
    let stats = analyzer.analyze_complexity();
    println!("Grammar Statistics:");
    println!("  Symbols: {}", stats.total_symbols);
    println!("  Tokens: {}", stats.total_tokens);
    println!("  Fields: {}", stats.total_fields);
    println!("  Productions: {}", stats.total_productions);
    println!("  Max RHS length: {}", stats.max_rhs_length);
    println!("  Productions with precedence: {}", stats.productions_with_precedence);
    
    let recursive = analyzer.find_recursive_rules();
    println!("Recursive rules: {} found", recursive.len());
    
    let ambiguous = analyzer.find_ambiguous_rules();
    println!("Potentially ambiguous symbols: {} found", ambiguous.len());
    
    Ok(())
}
```

### Custom Token Pattern Loading

```rust
use rust_sitter::decoder;
use std::collections::HashMap;
use rust_sitter_ir::TokenPattern;

fn load_enhanced_patterns() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom token patterns
    let mut custom_patterns = HashMap::new();
    
    // Enhanced patterns for Python
    custom_patterns.insert(
        "identifier".to_string(), 
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string())
    );
    custom_patterns.insert(
        "string".to_string(),
        TokenPattern::Regex(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#.to_string())
    );
    custom_patterns.insert(
        "number".to_string(),
        TokenPattern::Regex(r"\d+(\.\d+)?([eE][+-]?\d+)?".to_string())
    );
    
    // Load and enhance the grammar
    let lang = rust_sitter_python::get_language();
    let mut grammar = decoder::decode_grammar(lang);
    
    // Update token patterns with enhanced versions
    for (symbol_id, token) in grammar.tokens.iter_mut() {
        if let Some(enhanced_pattern) = custom_patterns.get(&token.name) {
            token.pattern = enhanced_pattern.clone();
            println!("Enhanced pattern for {}: {:?}", token.name, enhanced_pattern);
        }
    }
    
    // The enhanced grammar can now be used for parsing
    let table = decoder::decode_parse_table(lang);
    let parser = rust_sitter::glr_parser::GLRParser::new(grammar, table);
    
    Ok(())
}
```

## Testing and Validation

### Roundtrip Testing

Verify that decoded grammars work correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter::{decoder, glr_parser::GLRParser};
    
    #[test]
    fn test_python_decoding_roundtrip() {
        let lang = rust_sitter_python::get_language();
        let grammar = decoder::decode_grammar(lang);
        let table = decoder::decode_parse_table(lang);
        
        // Verify basic structure
        assert!(!grammar.rules.is_empty());
        assert!(!grammar.fields.is_empty());
        assert_eq!(table.lex_modes.len(), lang.state_count as usize);
        
        // Test parsing capability
        let parser = GLRParser::new(grammar, table);
        let test_cases = vec![
            "def hello(): pass",
            "x = 42",
            "if True: print('ok')",
            "for i in range(10): pass",
        ];
        
        for test_case in test_cases {
            let result = parser.parse(test_case);
            assert!(result.is_ok(), "Failed to parse: {}", test_case);
        }
    }
    
    #[test]
    fn test_rule_reconstruction_accuracy() {
        let lang = rust_sitter_python::get_language();
        let grammar = decoder::decode_grammar(lang);
        let table = decoder::decode_parse_table(lang);
        
        // Verify rule lengths match between grammar and parse table
        for (i, parse_rule) in table.rules.iter().enumerate() {
            let production_id = rust_sitter_ir::ProductionId(i as u16);
            
            // Find corresponding grammar rule
            let grammar_rule = grammar.rules.values()
                .flat_map(|rules| rules.iter())
                .find(|rule| rule.production_id == production_id);
            
            if let Some(rule) = grammar_rule {
                assert_eq!(
                    parse_rule.rhs_len as usize, 
                    rule.rhs.len(),
                    "Rule length mismatch for production {}", i
                );
            }
        }
    }
}
```

### Performance Testing

```rust
use std::time::Instant;

fn benchmark_decoding() {
    let languages = vec![
        ("Python", rust_sitter_python::get_language()),
        ("JavaScript", rust_sitter_javascript::get_language()),
    ];
    
    for (name, lang) in languages {
        let start = Instant::now();
        
        let grammar = decoder::decode_grammar(lang);
        let grammar_time = start.elapsed();
        
        let table_start = Instant::now();
        let table = decoder::decode_parse_table(lang);
        let table_time = table_start.elapsed();
        
        println!("{} decoding:", name);
        println!("  Grammar: {:?} ({} rules)", grammar_time, grammar.rules.len());
        println!("  Table: {:?} ({} states)", table_time, table.lex_modes.len());
        
        // Test parsing performance
        let parser = rust_sitter::glr_parser::GLRParser::new(grammar, table);
        let parse_start = Instant::now();
        let _result = parser.parse("# Simple test comment");
        let parse_time = parse_start.elapsed();
        println!("  Parse: {:?}", parse_time);
        println!();
    }
}
```

## Error Handling

### Common Issues and Solutions

```rust
use rust_sitter::decoder;

fn robust_decoding(lang: &rust_sitter::TSLanguage) -> Result<(), Box<dyn std::error::Error>> {
    // Grammar decoding is generally safe, but validation is good practice
    let grammar = decoder::decode_grammar(lang);
    
    // Validate grammar structure
    if grammar.rules.is_empty() {
        return Err("Grammar has no rules".into());
    }
    
    if grammar.tokens.is_empty() {
        return Err("Grammar has no tokens".into());
    }
    
    // Parse table decoding
    let table = decoder::decode_parse_table(lang);
    
    // Validate table structure
    if table.lex_modes.is_empty() {
        return Err("Parse table has no lex modes".into());
    }
    
    if table.rules.is_empty() {
        return Err("Parse table has no rules".into());
    }
    
    // Validate consistency between grammar and table
    if table.rules.len() != grammar.rules.values().map(|r| r.len()).sum::<usize>() {
        eprintln!("Warning: Rule count mismatch between grammar and table");
    }
    
    println!("Grammar and table decoded successfully!");
    Ok(())
}
```

Grammar decoding opens up powerful possibilities for dynamic language processing, tooling development, and grammar analysis. Use these patterns to build flexible, multi-language applications with rust-sitter's GLR parser.