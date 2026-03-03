//! Command-line interface for the adze build tool.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod parse;
mod test;

/// Tree-sitter compatible CLI for adze
#[derive(Parser)]
#[command(name = "adze")]
#[command(about = "A pure-Rust implementation of Tree-sitter", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a parser from a grammar.js file
    Generate {
        /// Path to grammar.js file
        #[arg(short, long, default_value = "grammar.js")]
        grammar: PathBuf,

        /// Output directory for generated files
        #[arg(short, long, default_value = "src")]
        output: PathBuf,

        /// Emit debug artifacts
        #[arg(long)]
        debug: bool,

        /// Use pure-rust implementation
        #[arg(long, default_value_t = true)]
        pure_rust: bool,
    },

    /// Parse a file and output the syntax tree
    Parse {
        /// Path to file to parse
        file: PathBuf,

        /// Path to parser crate (MVP) or compiled library (future)
        /// Example: --parser ./my-parser-crate
        #[arg(short, long)]
        parser: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "tree")]
        format: OutputFormat,

        /// Show field names
        #[arg(long)]
        fields: bool,

        /// Show statistics
        #[arg(short, long)]
        stats: bool,
    },

    /// Test a parser against a corpus
    Test {
        /// Path to test directory
        #[arg(default_value = "test")]
        path: PathBuf,

        /// Filter tests by name
        #[arg(short, long)]
        filter: Option<String>,

        /// Update test expectations
        #[arg(short, long)]
        update: bool,
    },

    /// Initialize a new grammar project
    Init {
        /// Name of the language
        name: String,

        /// Create in current directory
        #[arg(long)]
        in_place: bool,
    },

    /// Show information about a grammar
    Info {
        /// Path to grammar.js or compiled parser
        #[arg(default_value = "grammar.js")]
        path: PathBuf,

        /// Show node types
        #[arg(long)]
        node_types: bool,

        /// Show grammar rules
        #[arg(long)]
        rules: bool,
    },
}

/// Output format for parse result display.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Tree,
    Sexp,
    Json,
    Dot,
}

pub fn run_generate(
    grammar: &PathBuf,
    output: &PathBuf,
    debug: bool,
    pure_rust: bool,
) -> Result<()> {
    use crate::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    println!("Generating parser from {:?}...", grammar);

    if !pure_rust {
        anyhow::bail!(
            "Only pure-rust implementation is currently supported. Remove --no-pure-rust flag."
        );
    }

    let options = BuildOptions {
        out_dir: output.to_str().unwrap().to_string(),
        emit_artifacts: debug,
        compress_tables: !debug,
    };

    let result = build_parser_from_grammar_js(grammar, options)?;

    println!("✓ Generated parser for '{}'", result.grammar_name);
    println!("  Parser: {}", result.parser_path);
    if debug {
        println!("  Debug artifacts emitted to: {:?}", output);
    }

    Ok(())
}

pub fn run_parse(
    file: &Path,
    parser: &Option<PathBuf>,
    format: &OutputFormat,
    fields: bool,
    stats: bool,
) -> Result<()> {
    use parse::OutputFormat as ParseOutputFormat;

    let parse_format = match format {
        OutputFormat::Tree | OutputFormat::Sexp => ParseOutputFormat::Sexp,
        OutputFormat::Json => ParseOutputFormat::Json,
        OutputFormat::Dot => ParseOutputFormat::Dot,
    };

    parse::parse_file(file, parser.as_deref(), parse_format, fields, stats)
}

pub fn run_test(path: &Path, filter: &Option<String>, update: bool) -> Result<()> {
    test::run_tests(Some(path), None, filter.as_deref(), update, true)
}

pub fn run_init(name: &str, in_place: bool) -> Result<()> {
    use std::fs;

    println!("Initializing grammar for '{}'", name);

    let dir = if in_place {
        PathBuf::from(".")
    } else {
        PathBuf::from(format!("tree-sitter-{}", name.to_lowercase()))
    };

    if !in_place {
        fs::create_dir_all(&dir)?;
    }

    // Create grammar.js
    let grammar_content = format!(
        r#"module.exports = grammar({{
  name: '{}',

  rules: {{
    // TODO: add the actual grammar rules
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      // TODO: add choices here
    ),

    // TODO: add other rules
  }}
}});
"#,
        name
    );

    fs::write(dir.join("grammar.js"), grammar_content)?;

    // Create basic Cargo.toml
    let cargo_content = format!(
        r#"[package]
name = "tree-sitter-{}"
version = "0.1.0"
edition = "2024"

[dependencies]
adze = "0.5.0-beta"

[build-dependencies]
adze-tool = "0.5.0-beta"
"#,
        name.to_lowercase()
    );

    fs::write(dir.join("Cargo.toml"), cargo_content)?;

    // Create build.rs
    let build_content = r#"fn main() {
    adze_tool::build_parsers();
}
"#;

    fs::write(dir.join("build.rs"), build_content)?;

    // Create src/lib.rs
    fs::create_dir_all(dir.join("src"))?;
    let lib_content = format!(
        r#"use adze::Grammar;

#[adze::grammar("{}")]
pub struct {};

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_can_load_grammar() {{
        let _ = {}::LANGUAGE;
    }}
}}
"#,
        name.to_lowercase(),
        name,
        name
    );

    fs::write(dir.join("src/lib.rs"), lib_content)?;

    println!("✓ Created grammar project in {:?}", dir);
    println!("  Next steps:");
    println!("    1. cd {:?}", dir);
    println!("    2. Edit grammar.js");
    println!("    3. cargo build");

    Ok(())
}

pub fn run_info(path: &PathBuf, node_types: bool, rules: bool) -> Result<()> {
    use crate::grammar_js::parse_grammar_js_v2;
    use std::fs;

    if path.extension().is_some_and(|ext| ext == "js") {
        // Parse grammar.js
        let content = fs::read_to_string(path)?;
        let grammar = parse_grammar_js_v2(&content)?;

        println!("Grammar: {}", grammar.name);
        println!("Rules: {}", grammar.rules.len());

        if rules {
            println!("\nRules:");
            for name in grammar.rules.keys() {
                println!("  - {}", name);
            }
        }

        if node_types {
            println!("\nNode types information requires a compiled parser.");
        }
    } else {
        anyhow::bail!("Info command for compiled parsers not yet implemented.");
    }

    Ok(())
}
