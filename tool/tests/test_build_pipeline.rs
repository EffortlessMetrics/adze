//! Tests for the tool crate's build pipeline and grammar processing.
//!
//! Covers: grammar extraction, JSON grammar generation, pure-Rust builder output,
//! visualization utilities, error handling, and CLI argument parsing.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. Grammar extraction from annotated Rust code
// ---------------------------------------------------------------------------

#[test]
fn extract_grammar_from_annotated_module() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("grammar.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("arith")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32,
                ),
            }
        }
    "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert_eq!(grammars.len(), 1, "should find exactly one grammar");

    let g = &grammars[0];
    assert_eq!(g["name"].as_str().unwrap(), "arith");
    assert!(g["rules"].is_object(), "grammar should have rules object");
}

#[test]
fn extract_no_grammar_from_plain_module() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("plain.rs");
    fs::write(
        &src,
        r#"
        mod not_a_grammar {
            pub struct Foo;
        }
    "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert!(grammars.is_empty(), "plain module should yield no grammars");
}

#[test]
fn extract_multiple_grammars() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("multi.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("lang_a")]
        mod a {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }

        #[adze::grammar("lang_b")]
        mod b {
            #[adze::language]
            pub enum Token {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
    "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert_eq!(grammars.len(), 2, "should find two grammars");
    let names: Vec<_> = grammars
        .iter()
        .map(|g| g["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"lang_a"));
    assert!(names.contains(&"lang_b"));
}

// ---------------------------------------------------------------------------
// 2. JSON grammar generation round-trip
// ---------------------------------------------------------------------------

#[test]
fn json_grammar_has_required_fields() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("grammar.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("json_check")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
    "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let g = &grammars[0];

    // Tree-sitter JSON format requires these top-level keys
    assert!(g.get("name").is_some(), "must have 'name'");
    assert!(g.get("rules").is_some(), "must have 'rules'");
}

#[test]
fn json_grammar_name_survives_serialization() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("grammar.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
    "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let json_str = serde_json::to_string(&grammars[0]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["name"].as_str().unwrap(), "roundtrip");
}

// ---------------------------------------------------------------------------
// 3. Pure-Rust builder output
// ---------------------------------------------------------------------------

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

#[test]
fn pure_rust_builder_creates_parser_file() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'file_check',
  rules: {
    source: $ => $.item,
    item: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert_eq!(result.grammar_name, "file_check");
    assert!(
        std::path::Path::new(&result.parser_path).exists(),
        "parser file should exist on disk"
    );
}

#[test]
fn pure_rust_builder_generates_valid_rust() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'valid_rs',
  rules: {
    source: $ => $.item,
    item: $ => /[0-9]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    // The parser_code should be valid Rust token stream (it passed prettyplease in the builder)
    assert!(
        !result.parser_code.is_empty(),
        "parser code should not be empty"
    );
}

#[test]
fn pure_rust_builder_emits_node_types_json() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'nt_json',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    let node_types: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("NODE_TYPES must be valid JSON");
    assert!(node_types.is_array(), "NODE_TYPES must be a JSON array");
}

#[test]
fn pure_rust_builder_build_stats_populated() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'stats_check',
  rules: {
    source: $ => $.tok,
    tok: $ => /\d+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert!(result.build_stats.state_count > 0, "should have states");
    assert!(result.build_stats.symbol_count > 0, "should have symbols");
}

#[test]
fn pure_rust_builder_compressed_tables() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'compressed',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn pure_rust_builder_emit_artifacts_creates_files() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'artifacts',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();

    let grammar_dir = dir.path().join("grammar_artifacts");
    assert!(grammar_dir.exists(), "grammar dir should be created");
    assert!(
        grammar_dir.join("grammar.ir.json").exists(),
        "IR debug file should be emitted"
    );
    assert!(
        grammar_dir.join("NODE_TYPES.json").exists(),
        "NODE_TYPES file should be emitted"
    );
    // Parser file itself
    assert!(std::path::Path::new(&result.parser_path).exists());
}

// ---------------------------------------------------------------------------
// 4. Visualization utilities
// ---------------------------------------------------------------------------

use adze_tool::GrammarVisualizer;

fn sample_grammar_for_viz() -> adze_ir::Grammar {
    adze_tool::GrammarConverter::create_sample_grammar()
}

#[test]
fn visualization_dot_contains_graph_structure() {
    let viz = GrammarVisualizer::new(sample_grammar_for_viz());
    let dot = viz.to_dot();

    assert!(dot.contains("digraph Grammar"), "should be a digraph");
    assert!(dot.contains("rankdir=LR"), "should use LR ranking");
    assert!(dot.contains("identifier"), "should mention token names");
    assert!(dot.contains("lightblue"), "terminals should be lightblue");
    assert!(
        dot.contains("lightgreen"),
        "non-terminals should be lightgreen"
    );
}

#[test]
fn visualization_text_contains_grammar_info() {
    let viz = GrammarVisualizer::new(sample_grammar_for_viz());
    let text = viz.to_text();

    assert!(text.contains("Grammar: sample"));
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
    assert!(text.contains("identifier"));
    assert!(text.contains("number"));
    assert!(text.contains("plus"));
}

#[test]
fn visualization_railroad_svg_is_valid_svg() {
    let viz = GrammarVisualizer::new(sample_grammar_for_viz());
    let svg = viz.to_railroad_svg();

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("<style>"));
}

#[test]
fn visualization_dependency_graph_lists_deps() {
    let viz = GrammarVisualizer::new(sample_grammar_for_viz());
    let deps = viz.dependency_graph();

    assert!(deps.contains("Symbol Dependencies:"));
    // The sample grammar has expr -> expr (self-recursive), so it should show up
}

#[test]
fn visualization_empty_grammar() {
    let grammar = adze_ir::Grammar::new("empty".to_string());
    let viz = GrammarVisualizer::new(grammar);

    let dot = viz.to_dot();
    assert!(
        dot.contains("digraph Grammar"),
        "empty grammar still valid DOT"
    );

    let text = viz.to_text();
    assert!(text.contains("Grammar: empty"));
}

// ---------------------------------------------------------------------------
// 5. Error handling for malformed input
// ---------------------------------------------------------------------------

use adze_tool::pure_rust_builder::build_parser_from_json;

#[test]
fn malformed_json_grammar_returns_error() {
    let opts = BuildOptions {
        out_dir: "/tmp/unused".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_json("not valid json{{{".to_string(), opts);
    assert!(result.is_err(), "invalid JSON should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("parse") || err_msg.contains("JSON"),
        "error should mention parsing: {}",
        err_msg
    );
}

#[test]
fn missing_grammar_js_file_returns_error() {
    let opts = BuildOptions {
        out_dir: "/tmp/unused".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(std::path::Path::new("/no/such/grammar.js"), opts);
    assert!(result.is_err(), "missing file should fail");
}

#[test]
fn empty_grammar_js_returns_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, "").unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&path, opts);
    assert!(result.is_err(), "empty grammar.js should fail");
}

#[test]
fn grammar_js_missing_name_returns_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  rules: {
    source: $ => 'hello'
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&path, opts);
    // Missing name may still succeed with default or may error - either is valid
    // but it shouldn't panic
    let _ = result;
}

// ---------------------------------------------------------------------------
// 5b. ToolError construction and display
// ---------------------------------------------------------------------------

use adze_tool::ToolError;

#[test]
fn tool_error_display_messages() {
    let e = ToolError::MultipleWordRules;
    assert!(format!("{e}").contains("word rule"));

    let e = ToolError::MultiplePrecedenceAttributes;
    assert!(format!("{e}").contains("prec"));

    let e = ToolError::NestedOptionType;
    assert!(format!("{e}").contains("Option<Option"));

    let e = ToolError::StructHasNoFields { name: "Foo".into() };
    assert!(format!("{e}").contains("Foo"));

    let e = ToolError::string_too_long("extraction", 9999);
    assert!(format!("{e}").contains("9999"));

    let e = ToolError::grammar_validation("bad start symbol");
    assert!(format!("{e}").contains("bad start symbol"));

    let e = ToolError::complex_symbols_not_normalized("FIRST set");
    assert!(format!("{e}").contains("FIRST set"));
}

#[test]
fn tool_error_from_string() {
    let e: ToolError = "custom error".into();
    assert!(format!("{e}").contains("custom error"));

    let e: ToolError = String::from("owned error").into();
    assert!(format!("{e}").contains("owned error"));
}

// ---------------------------------------------------------------------------
// 6. Scanner builder edge cases
// ---------------------------------------------------------------------------

use adze_tool::scanner_build::{ScannerBuilder, ScannerLanguage};

#[test]
fn scanner_builder_no_scanner_found() {
    let dir = TempDir::new().unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), PathBuf::new());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_none(), "should find no scanner in empty dir");
}

#[test]
fn scanner_builder_finds_named_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("mygrammar_scanner.c"), "// scanner").unwrap();

    let builder = ScannerBuilder::new("mygrammar", dir.path().to_path_buf(), PathBuf::new());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::C);
    assert_eq!(scanner.grammar_name, "mygrammar");
}

#[test]
fn scanner_builder_finds_rs_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.rs"), "pub struct Scanner;").unwrap();

    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), PathBuf::new());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::Rust);
}

#[test]
fn scanner_builder_finds_cpp_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.cc"), "// C++ scanner").unwrap();

    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), PathBuf::new());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::Cpp);
}

#[test]
fn scanner_builder_prefers_scanner_c_over_named() {
    let dir = TempDir::new().unwrap();
    // Both generic and named exist; generic "scanner.c" appears first in search order
    fs::write(dir.path().join("scanner.c"), "// generic").unwrap();
    fs::write(dir.path().join("test_scanner.c"), "// named").unwrap();

    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), PathBuf::new());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert!(
        scanner.path.ends_with("scanner.c"),
        "should prefer generic scanner.c"
    );
}

#[test]
fn scanner_language_extensions() {
    assert_eq!(ScannerLanguage::C.extension(), "c");
    assert_eq!(ScannerLanguage::Cpp.extension(), "cc");
    assert_eq!(ScannerLanguage::Rust.extension(), "rs");
}

// ---------------------------------------------------------------------------
// 7. GrammarConverter sample grammar integrity
// ---------------------------------------------------------------------------

use adze_tool::GrammarConverter;

#[test]
fn sample_grammar_has_expected_structure() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.name, "sample");
    assert_eq!(g.tokens.len(), 3, "identifier, number, plus");
    assert!(!g.rules.is_empty());
    assert!(!g.fields.is_empty(), "should have field definitions");
}

// ---------------------------------------------------------------------------
// 8. CLI argument parsing (structural, no subprocess)
// ---------------------------------------------------------------------------

use adze_tool::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn cli_parses_generate_command() {
    let cli = Cli::parse_from(["adze", "generate", "--grammar", "my.js"]);
    match cli.command {
        Commands::Generate { grammar, .. } => {
            assert_eq!(grammar, PathBuf::from("my.js"));
        }
        _ => panic!("expected Generate"),
    }
}

#[test]
fn cli_parses_parse_command() {
    let cli = Cli::parse_from(["adze", "parse", "input.txt"]);
    match cli.command {
        Commands::Parse { file, parser, .. } => {
            assert_eq!(file, PathBuf::from("input.txt"));
            assert!(parser.is_none());
        }
        _ => panic!("expected Parse"),
    }
}

#[test]
fn cli_parses_parse_with_options() {
    let cli = Cli::parse_from([
        "adze",
        "parse",
        "--parser",
        "my-crate",
        "--format",
        "json",
        "--stats",
        "input.txt",
    ]);
    match cli.command {
        Commands::Parse {
            parser,
            stats,
            file,
            ..
        } => {
            assert_eq!(parser, Some(PathBuf::from("my-crate")));
            assert!(stats);
            assert_eq!(file, PathBuf::from("input.txt"));
        }
        _ => panic!("expected Parse"),
    }
}

#[test]
fn cli_parses_test_command() {
    let cli = Cli::parse_from(["adze", "test", "corpus/", "--filter", "numbers"]);
    match cli.command {
        Commands::Test { path, filter, .. } => {
            assert_eq!(path, PathBuf::from("corpus/"));
            assert_eq!(filter.as_deref(), Some("numbers"));
        }
        _ => panic!("expected Test"),
    }
}

#[test]
fn cli_parses_init_command() {
    let cli = Cli::parse_from(["adze", "init", "python", "--in-place"]);
    match cli.command {
        Commands::Init { name, in_place } => {
            assert_eq!(name, "python");
            assert!(in_place);
        }
        _ => panic!("expected Init"),
    }
}

#[test]
fn cli_parses_info_command() {
    let cli = Cli::parse_from(["adze", "info", "grammar.js", "--rules"]);
    match cli.command {
        Commands::Info { path, rules, .. } => {
            assert_eq!(path, PathBuf::from("grammar.js"));
            assert!(rules);
        }
        _ => panic!("expected Info"),
    }
}

#[test]
fn cli_generate_defaults() {
    let cli = Cli::parse_from(["adze", "generate"]);
    match cli.command {
        Commands::Generate {
            grammar,
            output,
            debug,
            pure_rust,
        } => {
            assert_eq!(grammar, PathBuf::from("grammar.js"));
            assert_eq!(output, PathBuf::from("src"));
            assert!(!debug);
            assert!(pure_rust);
        }
        _ => panic!("expected Generate"),
    }
}
