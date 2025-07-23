// Pure-Rust parser builder that uses the new IR and GLR infrastructure
// This module replaces the old Tree-sitter C generation with pure Rust code

use std::path::Path;
use std::fs;
use std::io::Write;
use anyhow::{Result, Context};
use rust_sitter_ir::{Grammar, optimize_grammar};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_tablegen::{StaticLanguageGenerator, NodeTypesGenerator};
use crate::grammar_js::{parse_grammar_js_v2, GrammarJsConverter};
use serde_json::Value;

/// Options for building a parser
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Output directory for generated files
    pub out_dir: String,
    /// Whether to emit debug artifacts
    pub emit_artifacts: bool,
    /// Whether to generate compressed tables
    pub compress_tables: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        BuildOptions {
            out_dir: std::env::var("OUT_DIR").unwrap_or_else(|_| "target/debug".to_string()),
            emit_artifacts: std::env::var("RUST_SITTER_EMIT_ARTIFACTS")
                .map(|s| s.parse().unwrap_or(false))
                .unwrap_or(false),
            compress_tables: true,
        }
    }
}

/// Result of building a parser
#[derive(Debug)]
pub struct BuildResult {
    /// Name of the grammar
    pub grammar_name: String,
    /// Path to generated parser module
    pub parser_path: String,
    /// Generated NODE_TYPES.json content
    pub node_types: String,
}

/// Build a parser from a grammar.js file
pub fn build_parser_from_grammar_js(grammar_js_path: &Path, options: BuildOptions) -> Result<BuildResult> {
    // Read and parse grammar.js
    let grammar_js_content = fs::read_to_string(grammar_js_path)
        .context("Failed to read grammar.js file")?;
    
    let grammar_js = parse_grammar_js_v2(&grammar_js_content)
        .context("Failed to parse grammar.js")?;
    
    // Convert to IR
    let converter = GrammarJsConverter::new(grammar_js);
    let mut grammar = converter.convert()
        .context("Failed to convert grammar.js to IR")?;
    
    // Optimize the grammar
    grammar = optimize_grammar(grammar)
        .context("Failed to optimize grammar")?;
    
    // Build the parser
    build_parser(grammar, options)
}

/// Build a parser for all grammars in a crate
pub fn build_parser_for_crate(root_file: &Path, options: BuildOptions) -> Result<Vec<BuildResult>> {
    let mut results = Vec::new();
    
    // Find all grammar definitions
    let grammars = crate::generate_grammars(root_file);
    
    for grammar_json in grammars {
        let result = build_parser_from_json(grammar_json, options.clone())?;
        results.push(result);
    }
    
    Ok(results)
}

/// Build a parser from a JSON grammar (Tree-sitter format)
pub fn build_parser_from_json(grammar_json: Value, options: BuildOptions) -> Result<BuildResult> {
    // Parse grammar name
    let grammar_name = grammar_json["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Grammar missing name field"))?
        .to_string();
    
    // Convert JSON to grammar.js structure, then to IR
    let grammar_js_str = serde_json::to_string(&grammar_json)?;
    let grammar_js = parse_grammar_js_v2(&format!("module.exports = {}", grammar_js_str))
        .context("Failed to parse grammar JSON")?;
    
    let converter = GrammarJsConverter::new(grammar_js);
    let mut grammar = converter.convert()
        .context("Failed to convert grammar JSON to IR")?;
    
    // Optimize the grammar
    grammar = optimize_grammar(grammar)
        .context("Failed to optimize grammar")?;
    
    // Build the parser
    build_parser(grammar, options)
}

/// Build a parser from an IR Grammar
pub fn build_parser(grammar: Grammar, options: BuildOptions) -> Result<BuildResult> {
    let grammar_name = grammar.name.clone();
    
    // Step 1: Compute FIRST/FOLLOW sets
    let first_follow = FirstFollowSets::compute(&grammar);
    
    // Step 2: Build LR(1) automaton
    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .context("Failed to build LR(1) automaton")?;
    
    // Step 3: Generate static language code
    let mut generator = StaticLanguageGenerator::new(grammar.clone(), parse_table);
    
    // Compress tables if requested
    if options.compress_tables {
        let compressed = rust_sitter_tablegen::compress::CompressedParseTable::from_parse_table(&generator.parse_table);
        generator.compressed_tables = Some(compressed.into());
    }
    
    // Generate the Rust code
    let language_code = generator.generate_language_code();
    
    // Step 4: Generate NODE_TYPES.json
    let node_types_gen = NodeTypesGenerator::new(&grammar);
    let node_types = node_types_gen.generate();
    let node_types_json = serde_json::to_string_pretty(&node_types)?;
    
    // Step 5: Write output files
    let grammar_dir = Path::new(&options.out_dir).join(format!("grammar_{}", grammar_name));
    
    if options.emit_artifacts {
        // Create output directory
        if grammar_dir.exists() {
            fs::remove_dir_all(&grammar_dir)
                .context("Failed to remove old grammar directory")?;
        }
        fs::create_dir_all(&grammar_dir)
            .context("Failed to create grammar directory")?;
        
        // Write grammar IR for debugging
        let grammar_ir_path = grammar_dir.join("grammar.ir.json");
        let mut grammar_ir_file = fs::File::create(&grammar_ir_path)?;
        grammar_ir_file.write_all(serde_json::to_string_pretty(&grammar)?.as_bytes())?;
        
        // Write NODE_TYPES.json
        let node_types_path = grammar_dir.join("NODE_TYPES.json");
        let mut node_types_file = fs::File::create(&node_types_path)?;
        node_types_file.write_all(node_types_json.as_bytes())?;
    }
    
    // Write the parser module
    let parser_module_name = format!("parser_{}.rs", grammar_name.to_lowercase().replace('-', "_"));
    let parser_path = grammar_dir.join(&parser_module_name);
    let mut parser_file = fs::File::create(&parser_path)?;
    
    // Write module header
    writeln!(parser_file, "// Auto-generated parser for {}", grammar_name)?;
    writeln!(parser_file, "// Generated by rust-sitter pure-Rust builder")?;
    writeln!(parser_file)?;
    writeln!(parser_file, "#![allow(dead_code)]")?;
    writeln!(parser_file, "#![allow(non_snake_case)]")?;
    writeln!(parser_file, "#![allow(non_camel_case_types)]")?;
    writeln!(parser_file)?;
    
    // Write the generated code
    writeln!(parser_file, "{}", language_code)?;
    
    Ok(BuildResult {
        grammar_name,
        parser_path: parser_path.to_string_lossy().to_string(),
        node_types: node_types_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_build_simple_parser() {
        let grammar_js = r#"
module.exports = grammar({
  name: 'test',
  
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
        "#;
        
        let temp_dir = TempDir::new().unwrap();
        let grammar_path = temp_dir.path().join("grammar.js");
        fs::write(&grammar_path, grammar_js).unwrap();
        
        let options = BuildOptions {
            out_dir: temp_dir.path().to_string_lossy().to_string(),
            emit_artifacts: true,
            compress_tables: false,
        };
        
        let result = build_parser_from_grammar_js(&grammar_path, options).unwrap();
        assert_eq!(result.grammar_name, "test");
        
        // Check that files were created
        let parser_path = Path::new(&result.parser_path);
        assert!(parser_path.exists());
        
        // Check NODE_TYPES content
        let node_types: Value = serde_json::from_str(&result.node_types).unwrap();
        assert!(node_types.is_array());
    }
}