// Pure-Rust parser builder that uses the new IR and GLR infrastructure
// This module replaces the old Tree-sitter C generation with pure Rust code

use crate::grammar_js::{GrammarJsConverter, parse_grammar_js_v2};
use anyhow::{Context, Result};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, TokenPattern};
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

/// Allocate a valid ProductionId safely
fn alloc_production_id(grammar: &Grammar) -> Result<ProductionId> {
    let max = grammar
        .rules
        .values()
        .flat_map(|rs| rs.iter().map(|r| r.production_id.0))
        .max()
        .unwrap_or(0);
    let next = max
        .checked_add(1)
        .context("too many productions (u16 overflow)")?;
    Ok(ProductionId(next))
}

/// Ensures every wrapper non-terminal that directly produces a pattern has an explicit unit rule N -> T.
/// This guarantees LR items expose terminal lookaheads, enabling token shifts from initial states.
///
/// A wrapper is any non-terminal N that:
/// 1. Has no rules at all (empty wrapper)
/// 2. Has unit rules (RHS length == 1) that need desugaring
fn desugar_pattern_wrappers(grammar: &mut Grammar) -> Result<()> {
    // Track non-terminals that need unit rules to tokens
    let mut wrappers_needing_rules = Vec::new();

    // First pass: Find non-terminals with no rules at all
    let all_nonterminals: Vec<SymbolId> = grammar
        .rule_names
        .keys()
        .filter(|id| !grammar.tokens.contains_key(*id))
        .copied()
        .collect();

    for nt_id in all_nonterminals {
        let has_rules = grammar
            .rules
            .get(&nt_id)
            .map(|rules| !rules.is_empty())
            .unwrap_or(false);

        if !has_rules {
            // This non-terminal has no rules - it's likely a wrapper for a pattern
            // For now, use a heuristic: if the name contains "Number", look for a number token
            // TODO: This should be improved to handle all pattern wrappers structurally
            if let Some(nt_name) = grammar.rule_names.get(&nt_id)
                && nt_name.to_lowercase().contains("number")
            {
                // Find a number token (one with \d pattern)
                for (tid, token) in &grammar.tokens {
                    if let TokenPattern::Regex(r) = &token.pattern
                        && (r.contains(r"\d") || r.contains("[0-9]"))
                    {
                        wrappers_needing_rules.push((nt_id, *tid));
                        break;
                    }
                }
            }
        }
    }

    // Second pass: Look for existing unit rules that might need desugaring
    // (This handles cases where the wrapper has a rule but it's to an inline pattern)
    let mut rules_to_add = Vec::new();
    for (_, rules) in &grammar.rules {
        for rule in rules {
            if rule.rhs.len() == 1 {
                // This is a unit rule
                match &rule.rhs[0] {
                    Symbol::Terminal(_) => {
                        // Already a terminal unit rule, good
                    }
                    Symbol::NonTerminal(_) => {
                        // Unit rule to another non-terminal, leave it alone
                    }
                    // Handle other symbol types that might represent inline patterns
                    _ => {
                        // For now, we don't handle these - would need to create tokens for patterns
                    }
                }
            }
        }
    }

    // Add unit rules for all wrappers that need them
    for (nt_id, token_id) in wrappers_needing_rules {
        // Check if this exact unit rule already exists to avoid duplicates
        let already_exists = grammar
            .rules
            .get(&nt_id)
            .map(|existing_rules| {
                existing_rules.iter().any(|r| {
                    r.rhs.len() == 1
                        && matches!(&r.rhs[0], Symbol::Terminal(tid) if *tid == token_id)
                })
            })
            .unwrap_or(false);

        if !already_exists {
            let production_id = alloc_production_id(grammar)?;
            let unit_rule = Rule {
                lhs: nt_id,
                rhs: vec![Symbol::Terminal(token_id)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id,
            };
            grammar.add_rule(unit_rule);
            rules_to_add.push((nt_id, token_id));
        }
    }

    // Rebuild symbol registry after changes
    let _ = grammar.get_or_build_registry();

    // Log what we did (only if debug logging is enabled)
    if std::env::var("RUST_LOG")
        .unwrap_or_default()
        .contains("debug")
        && !rules_to_add.is_empty()
    {
        eprintln!(
            "Desugaring: Added {} unit rules for pattern wrappers",
            rules_to_add.len()
        );
        for (nt, tok) in rules_to_add {
            eprintln!("  {} -> Terminal({})", nt.0, tok.0);
        }
    }

    Ok(())
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

    #[cfg(feature = "optimize")]
    let grammar = {
        use rust_sitter_ir::optimizer::optimize_grammar;
        optimize_grammar(grammar).context("Failed to optimize grammar")?
    };

    // Grammar optimized successfully

    // Build the parser
    build_parser(grammar, options)
}

/// Build a parser for all grammars in a crate
pub fn build_parser_for_crate(root_file: &Path, options: BuildOptions) -> Result<Vec<BuildResult>> {
    let mut results = Vec::new();

    // Find all grammar definitions
    let grammars = crate::generate_grammars(root_file)?;

    // Debug: write to file
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::File::create("/tmp/rust_sitter_grammars.txt") {
            writeln!(
                f,
                "Found {} grammars from {}",
                grammars.len(),
                root_file.display()
            )
            .ok();
        }
    }

    for grammar_json in grammars {
        // Convert serde_json::Value to string
        let grammar_json_str = serde_json::to_string(&grammar_json).unwrap();
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

    // Step 0: Desugar pattern wrappers into unit productions
    desugar_pattern_wrappers(&mut grammar)?;

    // Step 1: Compute FIRST/FOLLOW sets
    let first_follow = FirstFollowSets::compute(&grammar)
        .with_context(|| "Failed to compute FIRST/FOLLOW sets")?;

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

    // Debug: Print all rules in the grammar and verify desugaring
    writeln!(debug_file, "Debug: All rules in grammar:")?;
    let mut wrappers_with_rules = 0;
    let mut wrappers_without_rules = Vec::new();

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

        // Check if this is a wrapper that got desugared
        if let Some(name) = grammar.rule_names.get(symbol_id) {
            if rules.len() == 1 && rules[0].rhs.len() == 1 {
                if let Symbol::Terminal(_) = &rules[0].rhs[0] {
                    wrappers_with_rules += 1;
                    writeln!(debug_file, "    -> This appears to be a desugared wrapper")?;
                }
            } else if rules.is_empty() {
                wrappers_without_rules.push((symbol_id, name.clone()));
            }
        }
    }

    // Sanity check: Report any wrappers that didn't get rules
    if !wrappers_without_rules.is_empty() {
        writeln!(
            debug_file,
            "WARNING: {} non-terminals have no rules:",
            wrappers_without_rules.len()
        )?;
        for (id, name) in &wrappers_without_rules {
            writeln!(debug_file, "  - Symbol {:?}: {}", id, name)?;
        }
    }

    writeln!(
        debug_file,
        "Debug: Found {} desugared wrappers",
        wrappers_with_rules
    )?;

    // Step 2: Build LR(1) automaton
    let parse_table = match build_lr1_automaton(&grammar, &first_follow) {
        Ok(table) => {
            // Apply standard table normalization:
            // 1. Normalize EOF to SymbolId(0)
            // 2. Auto-detect and set appropriate GOTO indexing mode
            let normalized = table.normalize_eof_to_zero().with_detected_goto_indexing();

            // Ensure invariants
            debug_assert_eq!(normalized.eof_symbol, SymbolId(0));
            debug_assert!(
                normalized
                    .symbol_to_index
                    .contains_key(&normalized.eof_symbol)
            );

            normalized
        }
        Err(e) => {
            eprintln!("ERROR building LR(1) automaton for {}: {}", grammar_name, e);
            eprintln!(
                "Grammar stats: {} tokens, {} rules, {} externals",
                grammar.tokens.len(),
                grammar.rules.len(),
                grammar.externals.len()
            );
            return Err(anyhow::anyhow!("Failed to build LR(1) automaton: {}", e));
        }
    };

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

    // Sanity check: Verify all terminals are in symbol_to_index
    let mut unmapped_terminals = Vec::new();
    for token_id in grammar.tokens.keys() {
        if !parse_table.symbol_to_index.contains_key(token_id) {
            unmapped_terminals.push(token_id);
        }
    }

    if !unmapped_terminals.is_empty() {
        writeln!(
            debug_file,
            "ERROR: {} terminals not in symbol_to_index:",
            unmapped_terminals.len()
        )?;
        for tid in &unmapped_terminals {
            let name = grammar
                .tokens
                .get(*tid)
                .map(|t| t.name.as_str())
                .unwrap_or("<unknown>");
            writeln!(debug_file, "  - Token {:?}: {}", tid, name)?;
        }
        eprintln!(
            "ERROR: {} terminals not mapped in parse table",
            unmapped_terminals.len()
        );
    }

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
            .filter(|(_, a)| !a.is_empty())
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

    // Debug state 0 actions only in debug mode
    if std::env::var("RUST_LOG")
        .unwrap_or_default()
        .contains("debug")
        && let Some(state0_actions) = parse_table.action_table.first()
    {
        eprintln!(
            "State 0 debug: {} action cells, {} tokens",
            state0_actions.len(),
            grammar.tokens.len()
        );

        let mut token_actions = 0;
        for (symbol_idx, action_cell) in state0_actions.iter().enumerate() {
            if !action_cell.is_empty() {
                // Check if this is a token
                for (sym_id, idx) in &parse_table.symbol_to_index {
                    if *idx == symbol_idx && grammar.tokens.contains_key(sym_id) {
                        token_actions += 1;
                        break;
                    }
                }
            }
        }

        if token_actions > 0 {
            eprintln!(
                "State 0 has {} token actions - parser can accept input ✓",
                token_actions
            );
        } else {
            eprintln!("WARNING: State 0 has no token actions - parser cannot accept input!");
        }
    }

    // Debug info written to temp file

    // Step 3: Generate static language code using ABI builder
    let language_code = if options.compress_tables {
        // Compress the parse tables
        use rust_sitter_tablegen::compress::TableCompressor;
        let compressor = TableCompressor::new();

        // Collect token indices for validation
        let token_indices =
            rust_sitter_tablegen::helpers::collect_token_indices(&grammar, &parse_table);

        // Check if start symbol can be empty using FIRST/FOLLOW sets
        let start_can_be_empty = grammar
            .start_symbol()
            .map(|sym| first_follow.is_nullable(sym))
            .unwrap_or(false);

        let compressed_tables = compressor
            .compress(&parse_table, &token_indices, start_can_be_empty)
            .map_err(|e| anyhow::anyhow!("Failed to compress tables: {}", e))?;

        // Generate code with compressed tables
        let abi_builder = AbiLanguageBuilder::new(&grammar, &parse_table)
            .with_compressed_tables(&compressed_tables);
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

        // Write .parsetable file for GLR runtime
        #[cfg(feature = "serialization")]
        {
            use rust_sitter_tablegen::ParsetableWriter;

            // Extract version from Cargo.toml if available, otherwise use "0.1.0"
            let grammar_version = std::env::var("CARGO_PKG_VERSION")
                .unwrap_or_else(|_| "0.1.0".to_string());

            let writer = ParsetableWriter::new(
                &grammar,
                &parse_table,
                &grammar_name,
                &grammar_version,
            );

            let parsetable_path = grammar_dir.join(format!("{}.parsetable", grammar_name));
            writer
                .write_file(&parsetable_path)
                .with_context(|| format!("Failed to write .parsetable file to {:?}", parsetable_path))?;

            writeln!(
                debug_file,
                "Generated .parsetable file: {}",
                parsetable_path.display()
            )?;
        }
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
    writeln!(parser_file, "#[allow(unused_imports)]")?;
    writeln!(parser_file, "#[allow(non_snake_case)]")?;
    writeln!(parser_file, "#[allow(non_camel_case_types)]")?;
    writeln!(parser_file, "#[allow(unused_unsafe)]")?;
    writeln!(parser_file, "#[allow(unused_variables)]")?;
    writeln!(parser_file, "#[allow(unexpected_cfgs)]")?;
    writeln!(parser_file, "#[allow(unsafe_op_in_unsafe_fn)]")?;
    writeln!(parser_file, "#[allow(clippy::op_ref)]")?;
    writeln!(parser_file, "#[allow(clippy::char_lit_as_u8)]")?;
    writeln!(parser_file, "#[allow(clippy::unnecessary_cast)]")?;
    writeln!(parser_file)?;

    // Emit GRAMMAR_NAME constant for external scanner registration
    writeln!(parser_file, "/// Grammar name for external scanner registration")?;
    writeln!(parser_file, "pub const GRAMMAR_NAME: &str = {:?};", grammar_name)?;
    writeln!(parser_file)?;

    // Parse tokens to an AST and pretty-print to stable Rust source
    use prettyplease::unparse as pretty_unparse;
    use syn::{File, parse2};

    let file_ast: File =
        parse2(language_code.clone()).expect("generator must produce a parsable Rust file");
    let formatted = pretty_unparse(&file_ast);

    // Always end with a trailing newline (avoids include! edge cases)
    if formatted.ends_with('\n') {
        write!(parser_file, "{}", formatted)?;
    } else {
        writeln!(parser_file, "{}", formatted)?;
    }

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
