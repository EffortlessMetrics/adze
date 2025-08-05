// Pure-Rust parser builder that uses the new IR and GLR infrastructure
// This module replaces the old Tree-sitter C generation with pure Rust code

use crate::grammar_js::{GrammarJsConverter, parse_grammar_js_v2};
use anyhow::{Context, Result};
use rust_sitter_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::Grammar;
use rust_sitter_tablegen::{AbiLanguageBuilder, NodeTypesGenerator};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

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
    /// Generated parser code
    pub parser_code: String,
    /// Generated NODE_TYPES.json content
    pub node_types_json: String,
}

/// Build a parser from a grammar.js file
pub fn build_parser_from_grammar_js(
    grammar_js_path: &Path,
    options: BuildOptions,
) -> Result<BuildResult> {
    // Read and parse grammar.js
    let grammar_js_content = fs::read_to_string(grammar_js_path)
        .with_context(|| format!("Failed to read grammar.js file at {:?}", grammar_js_path))?;

    // Try v3 parser first, fall back to v2 if needed
    let grammar_js = {
        let mut parser_v3 = crate::grammar_js::GrammarJsParserV3::new(grammar_js_content.clone());
        match parser_v3.parse() {
            Ok(g) => g,
            Err(_) => {
                // Fall back to v2 parser
                parse_grammar_js_v2(&grammar_js_content).context("Failed to parse grammar.js")?
            }
        }
    };

    // Parse grammar.js successfully

    // Convert to IR
    let converter = GrammarJsConverter::new(grammar_js);
    let grammar = converter
        .convert()
        .context("Failed to convert grammar.js to IR")?;

    // Grammar converted successfully

    // Optimize the grammar
    #[cfg(feature = "optimize")]
    {
        use rust_sitter_ir::optimizer::optimize_grammar;
        grammar = optimize_grammar(grammar).context("Failed to optimize grammar")?;
    }

    // Grammar optimized successfully

    // Build the parser
    build_parser(grammar, options)
}

/// Build a parser for all grammars in a crate
pub fn build_parser_for_crate(root_file: &Path, options: BuildOptions) -> Result<Vec<BuildResult>> {
    let mut results = Vec::new();

    // Find all grammar definitions
    let grammars = crate::generate_grammars(root_file);

    for grammar_json in grammars {
        // Convert serde_json::Value to string
        let grammar_json_str = grammar_json.to_string();
        let result = build_parser_from_json(grammar_json_str, options.clone())?;
        results.push(result);
    }

    Ok(results)
}

/// Build a parser from a JSON grammar (Tree-sitter format)
pub fn build_parser_from_json(grammar_json: String, options: BuildOptions) -> Result<BuildResult> {
    // Parse the JSON string
    let grammar_value: Value =
        serde_json::from_str(&grammar_json).context("Failed to parse grammar JSON")?;

    // Extract grammar name from JSON
    let grammar_name = grammar_value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Debug: Print the grammar JSON to understand the extras
    if grammar_name.contains("arithmetic") {
        eprintln!("DEBUG: Arithmetic grammar JSON:");
        eprintln!("{}", serde_json::to_string_pretty(&grammar_value).unwrap());
    }

    // Convert directly from JSON to GrammarJs structure
    let grammar_js = crate::grammar_js::from_json(&grammar_value)
        .context("Failed to convert JSON to GrammarJs")?;

    let converter = GrammarJsConverter::new(grammar_js);
    let grammar = converter
        .convert()
        .context("Failed to convert grammar JSON to IR")?;

    // Grammar converted from JSON

    // Optimize the grammar
    // TODO: Re-enable optimization after fixing unit rule elimination
    // #[cfg(not(feature = "no_opt"))]
    // {
    //     grammar = optimize_grammar(grammar).context("Failed to optimize grammar")?;
    // }

    // Grammar optimized successfully

    // Build the parser
    build_parser(grammar, options)
}

/// Build a parser from an IR Grammar
pub fn build_parser(mut grammar: Grammar, options: BuildOptions) -> Result<BuildResult> {
    let grammar_name = grammar.name.clone();

    // Ensure the grammar has a symbol registry
    let _ = grammar.get_or_build_registry();

    // Step 1: Compute FIRST/FOLLOW sets
    let first_follow = FirstFollowSets::compute(&grammar);

    // Write debug info to a file
    let debug_file_path =
        std::env::temp_dir().join(format!("rust_sitter_debug_{}.log", grammar_name));
    let mut debug_file = fs::File::create(&debug_file_path)?;

    writeln!(
        debug_file,
        "Debug: Grammar has {} tokens, {} rules",
        grammar.tokens.len(),
        grammar.rules.len()
    )?;
    writeln!(
        debug_file,
        "Debug: Token names: {:?}",
        grammar.tokens.values().map(|t| &t.name).collect::<Vec<_>>()
    )?;
    writeln!(
        debug_file,
        "Debug: Rule names: {:?}",
        grammar.rule_names.values().collect::<Vec<_>>()
    )?;

    // Debug symbol name to ID mapping
    writeln!(
        debug_file,
        "Debug: Symbol name to ID mapping in grammar.rule_names:"
    )?;
    for (symbol_id, name) in &grammar.rule_names {
        writeln!(debug_file, "  '{}' -> SymbolId({})", name, symbol_id.0)?;
    }

    // Debug: Print all rules in the grammar
    writeln!(debug_file, "Debug: All rules in grammar:")?;
    for (symbol_id, rules) in &grammar.rules {
        writeln!(
            debug_file,
            "  Symbol {:?} has {} rules:",
            symbol_id,
            rules.len()
        )?;
        for rule in rules {
            writeln!(debug_file, "    {:?} -> {:?}", rule.lhs, rule.rhs)?;
        }
    }

    // Step 2: Build LR(1) automaton
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).context("Failed to build LR(1) automaton")?;

    writeln!(
        debug_file,
        "Debug: Parse table has {} states, {} symbols",
        parse_table.state_count, parse_table.symbol_count
    )?;
    writeln!(
        debug_file,
        "Debug: Action table has {} entries",
        parse_table.action_table.len()
    )?;
    writeln!(
        debug_file,
        "Debug: Goto table has {} entries",
        parse_table.goto_table.len()
    )?;

    // Debug: Print detailed action table content
    writeln!(debug_file, "Debug: Action table contents:")?;
    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        writeln!(debug_file, "  State {}: {:?}", state_idx, state_actions)?;
    }

    // Debug: Print action table content
    for (state_idx, actions) in parse_table.action_table.iter().enumerate() {
        let non_error_actions: Vec<_> = actions
            .iter()
            .enumerate()
            .filter(|(_, a)| !matches!(a, Action::Error))
            .collect();
        if !non_error_actions.is_empty() {
            writeln!(
                debug_file,
                "Debug: State {} has {} non-error actions",
                state_idx,
                non_error_actions.len()
            )?;
        }
    }

    // Debug info written to temp file

    // Step 3: Generate static language code using ABI builder
    let language_code = if options.compress_tables {
        // TODO: Fix table compression - currently disabled due to bug
        // Compression results in nearly empty parse tables
        let abi_builder = AbiLanguageBuilder::new(&grammar, &parse_table);
        abi_builder.generate()
    } else {
        let abi_builder = AbiLanguageBuilder::new(&grammar, &parse_table);
        abi_builder.generate()
    };

    // Step 4: Generate NODE_TYPES.json
    let node_types_gen = NodeTypesGenerator::new(&grammar);
    let node_types_json = node_types_gen
        .generate()
        .map_err(|e| anyhow::anyhow!("Failed to generate NODE_TYPES: {}", e))?;

    // Step 5: Write output files
    let grammar_dir = Path::new(&options.out_dir).join(format!("grammar_{}", grammar_name));

    if options.emit_artifacts {
        // Create output directory
        if grammar_dir.exists() {
            fs::remove_dir_all(&grammar_dir).context("Failed to remove old grammar directory")?;
        }
        fs::create_dir_all(&grammar_dir).context("Failed to create grammar directory")?;

        // Write grammar IR for debugging
        let grammar_ir_path = grammar_dir.join("grammar.ir.json");
        let mut grammar_ir_file = fs::File::create(&grammar_ir_path)?;
        grammar_ir_file.write_all(serde_json::to_string_pretty(&grammar)?.as_bytes())?;

        // Write NODE_TYPES.json
        let node_types_path = grammar_dir.join("NODE_TYPES.json");
        let mut node_types_file = fs::File::create(&node_types_path)?;
        node_types_file.write_all(node_types_json.as_bytes())?;
    }

    // Ensure grammar dir exists for parser module
    if !grammar_dir.exists() {
        fs::create_dir_all(&grammar_dir)
            .with_context(|| format!("Failed to create grammar directory at {:?}", grammar_dir))?;
    }

    // Write the parser module
    let parser_module_name = format!(
        "parser_{}.rs",
        grammar_name.to_lowercase().replace('-', "_")
    );
    let parser_path = grammar_dir.join(&parser_module_name);
    let mut parser_file = fs::File::create(&parser_path)
        .with_context(|| format!("Failed to create parser file at {:?}", parser_path))?;

    // Write module header
    writeln!(parser_file, "// Auto-generated parser for {}", grammar_name)?;
    writeln!(parser_file, "// Generated by rust-sitter pure-Rust builder")?;
    writeln!(parser_file)?;
    writeln!(parser_file, "#[allow(dead_code)]")?;
    writeln!(parser_file, "#[allow(non_snake_case)]")?;
    writeln!(parser_file, "#[allow(non_camel_case_types)]")?;
    writeln!(parser_file, "#[allow(unused_unsafe)]")?;
    writeln!(parser_file)?;

    // Write the generated code
    writeln!(parser_file, "{}", language_code)?;

    Ok(BuildResult {
        grammar_name,
        parser_path: parser_path.to_string_lossy().to_string(),
        parser_code: language_code.to_string(),
        node_types_json,
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
        let node_types: Value = serde_json::from_str(&result.node_types_json).unwrap();
        assert!(node_types.is_array());
    }
}
