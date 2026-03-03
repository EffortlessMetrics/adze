#![allow(clippy::needless_range_loop)]

//! Comprehensive build-pipeline tests for the adze-tool crate.
//!
//! Covers: grammar extraction from Rust source, JSON grammar generation,
//! build configuration options, error handling for malformed grammars,
//! and edge cases in build processing.

use std::fs;
use std::path::Path;

use adze_tool::GrammarConverter;
use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write Rust source to a temp file and extract grammars via `generate_grammars`.
fn grammars_from_rust(code: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, code).unwrap();
    adze_tool::generate_grammars(&src).unwrap()
}

/// Write a grammar.js and build with the pure-Rust builder.
fn build_js(js: &str) -> adze_tool::pure_rust_builder::BuildResult {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    build_parser_from_grammar_js(&path, opts).unwrap()
}

/// Try building from grammar.js, returning the Result.
fn try_build_js(js: &str) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    build_parser_from_grammar_js(&path, opts)
}

/// Default build options pointing at a temp dir.
fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// =========================================================================
// 1–4  Grammar extraction from Rust source
// =========================================================================

#[test]
fn extract_single_grammar_from_enum() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("single_enum")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Ident(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0]["name"].as_str().unwrap(), "single_enum");
}

#[test]
fn extract_grammar_rules_are_present() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("has_rules")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let rules = gs[0]["rules"].as_object().unwrap();
    // Must have at least source_file + variant rules.
    assert!(rules.len() >= 2, "expected >=2 rules, got {}", rules.len());
}

#[test]
fn extract_multiple_grammars_from_one_file() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("alpha")]
        mod g1 {
            #[adze::language]
            pub enum A { V(#[adze::leaf(pattern = r"a")] String) }
        }
        #[adze::grammar("beta")]
        mod g2 {
            #[adze::language]
            pub enum B { V(#[adze::leaf(pattern = r"b")] String) }
        }
        "#,
    );
    assert_eq!(gs.len(), 2);
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn no_grammar_attribute_yields_empty() {
    let gs = grammars_from_rust(
        r#"
        mod not_a_grammar {
            pub struct Foo;
        }
        "#,
    );
    assert!(gs.is_empty());
}

// =========================================================================
// 5–9  JSON grammar generation
// =========================================================================

#[test]
fn json_has_name_and_rules_keys() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("keys_test")]
        mod grammar {
            #[adze::language]
            pub enum T { X(#[adze::leaf(pattern = r"x")] String) }
        }
        "#,
    );
    assert!(gs[0].get("name").is_some());
    assert!(gs[0].get("rules").is_some());
}

#[test]
fn json_has_word_key() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("word_key")]
        mod grammar {
            #[adze::language]
            pub enum T { X(#[adze::leaf(pattern = r"x")] String) }
        }
        "#,
    );
    // word may be null but should be present
    assert!(gs[0].get("word").is_some(), "'word' key should be present");
}

#[test]
fn json_rule_values_have_type_field() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("typed_rules")]
        mod grammar {
            #[adze::language]
            pub enum T { N(#[adze::leaf(pattern = r"\d+")] i32) }
        }
        "#,
    );
    for (_name, rule) in gs[0]["rules"].as_object().unwrap() {
        assert!(rule.get("type").is_some(), "rule must have 'type' field");
    }
}

#[test]
fn json_prec_left_appears_in_output() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("prec_json")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
            #[adze::extra]
            struct Ws { #[adze::leaf(pattern = r"\s")] _w: () }
        }
        "#,
    );
    let json_str = serde_json::to_string(&gs[0]).unwrap();
    assert!(json_str.contains("PREC_LEFT"), "should contain PREC_LEFT");
}

#[test]
fn json_grammar_name_propagates() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("unique_name_99")]
        mod grammar {
            #[adze::language]
            pub enum T { V(#[adze::leaf(pattern = r"v")] String) }
        }
        "#,
    );
    assert_eq!(gs[0]["name"].as_str().unwrap(), "unique_name_99");
}

// =========================================================================
// 10–16  Build configuration options
// =========================================================================

#[test]
fn build_options_default_compress_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn build_options_default_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn build_with_compress_produces_code() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'compress_test',
  rules: { source: $ => $.t, t: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_without_compress_produces_code() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'no_compress',
  rules: { source: $ => $.t, t: $ => /[a-z]+/ }
});
"#,
    );
    assert!(!r.parser_code.is_empty());
}

#[test]
fn emit_artifacts_creates_ir_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'emit_test',
  rules: { source: $ => $.t, t: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    build_parser_from_grammar_js(&path, opts).unwrap();
    let artifact_dir = dir.path().join("grammar_emit_test");
    assert!(artifact_dir.join("grammar.ir.json").exists());
}

#[test]
fn emit_artifacts_creates_node_types_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'nt_emit',
  rules: { source: $ => $.t, t: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(
        dir.path()
            .join("grammar_nt_emit")
            .join("NODE_TYPES.json")
            .exists()
    );
}

#[test]
fn parser_file_written_to_out_dir() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'outdir_write',
  rules: { source: $ => $.t, t: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = opts_in(&dir);
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    let p = Path::new(&r.parser_path);
    assert!(p.exists(), "parser file must exist at {}", p.display());
    assert!(p.starts_with(dir.path()));
}

// =========================================================================
// 17–23  Error handling for malformed grammars
// =========================================================================

#[test]
fn malformed_json_gives_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("{bad json".into(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn empty_rules_grammar_js_fails() {
    let result = try_build_js(
        r#"
module.exports = grammar({
  name: 'empty',
  rules: {}
});
"#,
    );
    assert!(result.is_err());
}

#[test]
fn nonexistent_grammar_js_file_errors() {
    let dir = TempDir::new().unwrap();
    let result =
        build_parser_from_grammar_js(Path::new("/nonexistent/path/grammar.js"), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn completely_invalid_grammar_js_content_errors() {
    let result = try_build_js("THIS IS NOT JAVASCRIPT AT ALL!!!");
    assert!(result.is_err());
}

#[test]
fn grammar_js_missing_name_errors() {
    let result = try_build_js(
        r#"
module.exports = grammar({
  rules: { source: $ => /x/ }
});
"#,
    );
    assert!(result.is_err());
}

#[test]
fn json_with_missing_rules_key_errors() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json(r#"{"name": "no_rules"}"#.into(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn json_with_empty_rules_errors() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json(
        r#"{"name": "empty_rules", "rules": {}}"#.into(),
        opts_in(&dir),
    );
    assert!(result.is_err());
}

// =========================================================================
// 24–30  Edge cases in build processing
// =========================================================================

#[test]
fn single_rule_grammar_builds_successfully() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'minimal',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    assert_eq!(r.grammar_name, "minimal");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn grammar_with_seq_and_choice_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'seq_choice',
  rules: {
    source: $ => choice($.a, $.b),
    a: $ => seq('let', /[a-z]+/),
    b: $ => /\d+/
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "seq_choice");
    assert!(r.build_stats.symbol_count >= 3);
}

#[test]
fn grammar_with_optional_and_repeat_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'opt_rep',
  rules: {
    source: $ => seq($.head, repeat($.tail)),
    head: $ => /[a-z]+/,
    tail: $ => seq(',', optional(/[a-z]+/))
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "opt_rep");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_stats_have_positive_counts() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'stats_check',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn node_types_json_is_valid_array() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'nt_arr',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_entries_have_type_field() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'nt_typed',
  rules: {
    source: $ => $.item,
    item: $ => /[0-9]+/
  }
});
"#,
    );
    let entries: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
    for e in &entries {
        assert!(e.get("type").is_some(), "entry missing 'type': {:?}", e);
    }
}

#[test]
fn grammar_name_appears_in_generated_code() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'name_in_code',
  rules: { source: $ => /x/ }
});
"#,
    );
    assert_eq!(r.grammar_name, "name_in_code");
    let lower = r.parser_code.to_lowercase();
    assert!(
        lower.contains("name_in_code"),
        "grammar name should appear in generated code"
    );
}

// =========================================================================
// 31–35  Additional edge cases and integration
// =========================================================================

#[test]
fn build_from_json_round_trip() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("round_trip")]
        mod grammar {
            #[adze::language]
            pub enum T { N(#[adze::leaf(pattern = r"\d+")] i32) }
        }
        "#,
    );
    let dir = TempDir::new().unwrap();
    let json_str = serde_json::to_string(&gs[0]).unwrap();
    let r = build_parser_from_json(json_str, opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "round_trip");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_from_ir_grammar_directly() {
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let dir = TempDir::new().unwrap();
    let mut grammar = Grammar::new("ir_direct".into());

    let tok_id = SymbolId(1);
    let src_id = SymbolId(2);

    grammar.tokens.insert(
        tok_id,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(src_id, "source_file".into());
    grammar.rules.entry(src_id).or_default().push(Rule {
        lhs: src_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let r = build_parser(grammar, opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "ir_direct");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn grammar_converter_creates_sample_grammar() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.name, "sample");
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

#[test]
fn empty_rust_file_yields_no_grammars() {
    let gs = grammars_from_rust("");
    assert!(gs.is_empty());
}

#[test]
fn large_grammar_with_many_rules_builds() {
    let mut rules = String::from("source: $ => choice(\n");
    for i in 0..30 {
        if i > 0 {
            rules.push_str(",\n");
        }
        rules.push_str(&format!("      $.r_{i}"));
    }
    rules.push_str("\n    )");
    for i in 0..30 {
        rules.push_str(&format!(",\n    r_{i}: $ => /tok_{i}/"));
    }
    let js = format!(
        r#"
module.exports = grammar({{
  name: 'large',
  rules: {{
    {rules}
  }}
}});
"#
    );
    let r = build_js(&js);
    assert_eq!(r.grammar_name, "large");
    assert!(r.build_stats.symbol_count >= 30);
}
