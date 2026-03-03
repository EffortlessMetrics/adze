//! Integration tests for the tool crate's build pipeline.
//!
//! Tests the end-to-end flow: Rust source → grammar extraction → JSON generation →
//! pure-Rust builder → parser output. Also covers error handling, visualization,
//! and the direct IR-to-parser path.

use std::fs;

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. Extract grammar from simple Rust source
// ---------------------------------------------------------------------------

#[test]
fn extract_single_grammar_from_rust_source() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("calc")]
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
    assert_eq!(grammars.len(), 1);
    assert_eq!(grammars[0]["name"].as_str().unwrap(), "calc");
}

#[test]
fn extract_zero_grammars_from_plain_rust() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, "pub fn hello() {}").unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert!(grammars.is_empty());
}

#[test]
fn extract_nested_grammar_module() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        mod outer {
            #[adze::grammar("inner_lang")]
            mod inner {
                #[adze::language]
                pub enum Token {
                    Word(#[adze::leaf(pattern = r"[a-z]+")] String),
                }
            }
        }
        "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert_eq!(grammars.len(), 1);
    assert_eq!(grammars[0]["name"].as_str().unwrap(), "inner_lang");
}

// ---------------------------------------------------------------------------
// 2. Generate Tree-sitter JSON from extracted grammar
// ---------------------------------------------------------------------------

#[test]
fn generated_json_has_name_and_rules() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("ts_json")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let g = &grammars[0];

    assert!(g.get("name").is_some(), "JSON must have 'name'");
    assert!(g.get("rules").is_some(), "JSON must have 'rules'");
    assert!(g["rules"].is_object());
}

#[test]
fn generated_json_roundtrips_through_serde() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("serde_rt")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    )
    .unwrap();

    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let json_str = serde_json::to_string_pretty(&grammars[0]).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(reparsed["name"].as_str().unwrap(), "serde_rt");
}

// ---------------------------------------------------------------------------
// 3. Verify generated JSON is valid
// ---------------------------------------------------------------------------

#[test]
fn json_grammar_is_valid_json_object() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("valid_json")]
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
    let g = &grammars[0];

    assert!(g.is_object(), "grammar must be a JSON object");
    // Validate the JSON can be serialized and re-parsed without loss
    let bytes = serde_json::to_vec(g).unwrap();
    let reparsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(g, &reparsed);
}

#[test]
fn json_rules_key_contains_entries() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("rule_entries")]
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
    let rules = grammars[0]["rules"].as_object().unwrap();
    assert!(!rules.is_empty(), "rules object must not be empty");
}

// ---------------------------------------------------------------------------
// 4. Test grammar extraction error handling
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn nonexistent_file_panics() {
    // syn-inline-mod panics on missing files; verify we propagate that
    let _ = adze_tool::generate_grammars(std::path::Path::new("/no/such/file.rs"));
}

#[test]
fn invalid_json_to_builder_returns_error() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

    let opts = BuildOptions {
        out_dir: "/tmp/unused".into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_json("{bad json".into(), opts);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("parse") || msg.contains("JSON") || msg.contains("json"),
        "error should reference JSON parsing: {msg}"
    );
}

#[test]
fn missing_grammar_js_returns_error() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let opts = BuildOptions {
        out_dir: "/tmp/unused".into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(std::path::Path::new("/missing/grammar.js"), opts);
    assert!(result.is_err());
}

#[test]
fn empty_grammar_js_returns_error() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, "").unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&path, opts);
    assert!(result.is_err(), "empty grammar.js must fail");
}

#[test]
fn tool_error_variants_display_correctly() {
    use adze_tool::ToolError;

    let e = ToolError::MultipleWordRules;
    assert!(format!("{e}").contains("word rule"));

    let e = ToolError::StructHasNoFields { name: "Bar".into() };
    assert!(format!("{e}").contains("Bar"));

    let e = ToolError::grammar_validation("missing start");
    assert!(format!("{e}").contains("missing start"));

    let e: ToolError = "arbitrary message".into();
    assert!(format!("{e}").contains("arbitrary message"));
}

// ---------------------------------------------------------------------------
// 5. Test visualization output
// ---------------------------------------------------------------------------

#[test]
fn visualizer_dot_output_is_valid_digraph() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();

    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn visualizer_text_output_contains_tokens_and_rules() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let text = viz.to_text();

    assert!(text.contains("Grammar: sample"));
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
    assert!(text.contains("identifier"));
    assert!(text.contains("number"));
    assert!(text.contains("plus"));
}

#[test]
fn visualizer_svg_output_is_well_formed() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn visualizer_dependency_graph_output() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let deps = viz.dependency_graph();

    assert!(deps.contains("Symbol Dependencies:"));
}

#[test]
fn visualizer_handles_empty_grammar() {
    let grammar = adze_ir::Grammar::new("empty".into());
    let viz = adze_tool::GrammarVisualizer::new(grammar);

    // All formats should produce output without panicking
    assert!(viz.to_dot().contains("digraph"));
    assert!(viz.to_text().contains("Grammar: empty"));
    assert!(viz.to_railroad_svg().contains("<svg"));
    assert!(viz.dependency_graph().contains("Symbol Dependencies:"));
}

// ---------------------------------------------------------------------------
// 6. Test pure-Rust builder pipeline
// ---------------------------------------------------------------------------

#[test]
fn builder_from_grammar_js_produces_parser_file() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'bp_test',
  rules: {
    source: $ => $.item,
    item: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert_eq!(result.grammar_name, "bp_test");
    assert!(
        std::path::Path::new(&result.parser_path).exists(),
        "parser file must exist"
    );
    assert!(!result.parser_code.is_empty());
}

#[test]
fn builder_populates_build_stats() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'stats',
  rules: {
    source: $ => $.tok,
    tok: $ => /\d+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn builder_node_types_json_is_valid_array() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'nt_check',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    let node_types: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("must be valid JSON");
    assert!(node_types.is_array());
}

#[test]
fn builder_compressed_tables_succeeds() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

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
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };

    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn builder_from_ir_grammar_directly() {
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

    let dir = TempDir::new().unwrap();

    let mut grammar = Grammar::new("direct_ir".into());

    let num_sym = SymbolId(1);
    let source_sym = SymbolId(2);

    grammar.tokens.insert(
        num_sym,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(source_sym, "source_file".into());

    grammar.rules.entry(source_sym).or_default().push(Rule {
        lhs: source_sym,
        rhs: vec![Symbol::Terminal(num_sym)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "direct_ir");
    assert!(result.build_stats.state_count > 0);
    assert!(
        std::path::Path::new(&result.parser_path).exists(),
        "parser file must be written"
    );
}

#[test]
fn builder_emit_artifacts_creates_debug_files() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'artifact_test',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();

    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };

    let _result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();

    let grammar_dir = dir.path().join("grammar_artifact_test");
    assert!(grammar_dir.exists(), "grammar output dir must be created");
    assert!(
        grammar_dir.join("grammar.ir.json").exists(),
        "IR debug file must be emitted"
    );
    assert!(
        grammar_dir.join("NODE_TYPES.json").exists(),
        "NODE_TYPES must be emitted"
    );
}

// ---------------------------------------------------------------------------
// End-to-end: extraction → JSON → builder
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_extraction_to_builder() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("e2e")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    )
    .unwrap();

    // Step 1: extract
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    assert_eq!(grammars.len(), 1);

    // Step 2: serialize to JSON string
    let json_str = serde_json::to_string(&grammars[0]).unwrap();

    // Step 3: feed into the builder
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let result = build_parser_from_json(json_str, opts).unwrap();
    assert_eq!(result.grammar_name, "e2e");
    assert!(result.build_stats.state_count > 0);
    assert!(!result.parser_code.is_empty());
}
