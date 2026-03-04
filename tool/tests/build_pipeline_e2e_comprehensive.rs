#![allow(clippy::needless_range_loop)]

//! End-to-end comprehensive tests for the full build pipeline in adze-tool.
//!
//! Tests the complete flow: Rust source → grammar extraction → JSON generation →
//! pure-Rust builder → parser code + NODE_TYPES.json + BuildResult artifacts.

use std::fs;
use std::path::Path;

use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write Rust source to a temp file, extract grammars, then build each one
/// through the full pipeline via `build_parser_from_json`.
fn full_pipeline_from_rust(code: &str) -> Vec<BuildResult> {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, code).unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let mut results = Vec::new();
    for g in &grammars {
        let json_str = serde_json::to_string(g).unwrap();
        let out = TempDir::new().unwrap();
        let opts = BuildOptions {
            out_dir: out.path().to_string_lossy().into(),
            emit_artifacts: false,
            compress_tables: false,
        };
        results.push(build_parser_from_json(json_str, opts).unwrap());
    }
    results
}

/// Build a single grammar from Rust source through the full pipeline.
fn single_pipeline(code: &str) -> BuildResult {
    let results = full_pipeline_from_rust(code);
    assert_eq!(results.len(), 1, "expected exactly one grammar");
    results.into_iter().next().unwrap()
}

/// Build from grammar.js content.
fn build_js(js: &str) -> BuildResult {
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
fn try_build_js(js: &str) -> anyhow::Result<BuildResult> {
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

/// Default build options targeting a temp dir.
fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// =========================================================================
// 1–5  Simple struct grammar → JSON + parser code
// =========================================================================

#[test]
fn e2e_struct_grammar_produces_parser_code() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("struct_e2e")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    assert_eq!(r.grammar_name, "struct_e2e");
    assert!(!r.parser_code.is_empty(), "parser code must not be empty");
}

#[test]
fn e2e_struct_grammar_produces_node_types_json() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("struct_nt")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                value: i32,
            }
        }
        "#,
    );
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(val.is_array(), "NODE_TYPES must be a JSON array");
    let arr = val.as_array().unwrap();
    assert!(!arr.is_empty(), "NODE_TYPES must have entries");
}

#[test]
fn e2e_struct_grammar_parser_file_written() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("struct_file")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    )
    .unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let json_str = serde_json::to_string(&grammars[0]).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser_from_json(json_str, opts).unwrap();
    assert!(
        Path::new(&r.parser_path).exists(),
        "parser file must exist on disk"
    );
}

#[test]
fn e2e_struct_grammar_json_has_expected_keys() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("struct_keys")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    )
    .unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let g = &grammars[0];
    assert!(g.get("name").is_some());
    assert!(g.get("rules").is_some());
    assert_eq!(g["name"].as_str().unwrap(), "struct_keys");
}

#[test]
fn e2e_struct_grammar_build_stats_populated() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("struct_stats")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                tok: String,
            }
        }
        "#,
    );
    assert!(r.build_stats.state_count > 0, "must have states");
    assert!(r.build_stats.symbol_count > 0, "must have symbols");
}

// =========================================================================
// 6–10  Enum grammar → JSON + parser code
// =========================================================================

#[test]
fn e2e_enum_grammar_produces_parser_code() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("enum_e2e")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    assert_eq!(r.grammar_name, "enum_e2e");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn e2e_enum_grammar_has_multiple_rules() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("enum_rules")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    )
    .unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let rules = grammars[0]["rules"].as_object().unwrap();
    assert!(
        rules.len() >= 2,
        "enum grammar should produce multiple rules"
    );
}

#[test]
fn e2e_enum_grammar_node_types_contain_type_field() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("enum_nt_type")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let entries: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in &entries {
        assert!(
            entry.get("type").is_some(),
            "every NODE_TYPES entry must have 'type'"
        );
    }
}

#[test]
fn e2e_enum_grammar_parser_code_references_language() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("enum_lang")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Val(#[adze::leaf(pattern = r"[0-9]+")] i32),
            }
        }
        "#,
    );
    assert!(
        r.parser_code.contains("TSLanguage") || r.parser_code.contains("Language"),
        "generated code should reference the language type"
    );
}

#[test]
fn e2e_enum_with_three_variants_builds() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("enum_three")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                A(#[adze::leaf(pattern = r"a+")] String),
                B(#[adze::leaf(pattern = r"b+")] String),
                C(#[adze::leaf(pattern = r"c+")] String),
            }
        }
        "#,
    );
    assert_eq!(r.grammar_name, "enum_three");
    assert!(r.build_stats.symbol_count >= 3);
}

// =========================================================================
// 11–13  Grammar with extras → correct extras in output
// =========================================================================

#[test]
fn e2e_extras_appear_in_json() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("extras_json")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _w: (),
            }
        }
        "#,
    )
    .unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let extras = grammars[0]["extras"].as_array().expect("must have extras");
    assert!(!extras.is_empty(), "extras array must not be empty");
}

#[test]
fn e2e_extras_grammar_builds_to_parser_code() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("extras_build")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    assert_eq!(r.grammar_name, "extras_build");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn e2e_grammar_js_with_extras_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'extras_js',
  extras: $ => [/\s/, $.comment],
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/,
    comment: $ => seq('//', /[^\n]*/)
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "extras_js");
    assert!(!r.parser_code.is_empty());
}

// =========================================================================
// 14–16  Grammar with precedence → PREC rules in output
// =========================================================================

#[test]
fn e2e_prec_left_in_rust_grammar_json() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(
        &src,
        r#"
        #[adze::grammar("prec_e2e")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _w: (),
            }
        }
        "#,
    )
    .unwrap();
    let grammars = adze_tool::generate_grammars(&src).unwrap();
    let json_str = serde_json::to_string(&grammars[0]).unwrap();
    assert!(
        json_str.contains("PREC_LEFT"),
        "JSON must contain PREC_LEFT for prec_left annotation"
    );
}

#[test]
fn e2e_prec_left_grammar_full_pipeline() {
    let r = single_pipeline(
        r#"
        #[adze::grammar("prec_full")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _w: (),
            }
        }
        "#,
    );
    assert_eq!(r.grammar_name, "prec_full");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn e2e_grammar_js_prec_left_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'prec_js',
  rules: {
    expression: $ => choice(
      prec.left(1, seq($.expression, '+', $.expression)),
      $.number
    ),
    number: $ => /\d+/
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "prec_js");
    assert!(r.build_stats.state_count > 0);
}

// =========================================================================
// 17–19  Grammar with field names → fields in NODE_TYPES
// =========================================================================

#[test]
fn e2e_grammar_js_with_fields_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'fields_e2e',
  rules: {
    assignment: $ => seq(
      field('left', $.identifier),
      '=',
      field('right', $.value)
    ),
    identifier: $ => /[a-z]+/,
    value: $ => /\d+/
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "fields_e2e");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn e2e_fields_grammar_builds_with_node_types() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'fields_nt',
  rules: {
    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $.number)
    ),
    identifier: $ => /[a-z]+/,
    number: $ => /\d+/
  }
});
"#,
    );
    let entries: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(!entries.is_empty(), "NODE_TYPES must not be empty");
    // Verify at least one entry has named: true (the assignment rule)
    let has_named = entries
        .iter()
        .any(|e| e.get("named").and_then(|n| n.as_bool()).unwrap_or(false));
    assert!(
        has_named,
        "NODE_TYPES should have named entries for field-bearing rules"
    );
}

#[test]
fn e2e_field_grammar_parser_code_not_empty() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'field_names',
  rules: {
    pair: $ => seq(
      field('key', $.word),
      ':',
      field('val', $.word)
    ),
    word: $ => /[a-z]+/
  }
});
"#,
    );
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
    // NODE_TYPES should be parseable
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(val.is_array());
}

// =========================================================================
// 20–22  BuildResult contains all expected artifacts
// =========================================================================

#[test]
fn e2e_build_result_grammar_name_matches() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'result_name',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    assert_eq!(r.grammar_name, "result_name");
}

#[test]
fn e2e_build_result_parser_path_is_valid() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'result_path',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = opts_in(&dir);
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(
        Path::new(&r.parser_path).exists(),
        "parser_path must point to an existing file"
    );
}

#[test]
fn e2e_build_result_node_types_is_valid_json_array() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'result_nt',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(val.is_array());
    let arr = val.as_array().unwrap();
    for entry in arr {
        assert!(entry.get("type").is_some(), "each entry needs 'type'");
    }
}

// =========================================================================
// 23–25  Pipeline determinism
// =========================================================================

#[test]
fn e2e_determinism_same_rust_source_same_json() {
    let code = r#"
        #[adze::grammar("det_json")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                X(#[adze::leaf(pattern = r"x+")] String),
            }
        }
    "#;
    let dir1 = TempDir::new().unwrap();
    let src1 = dir1.path().join("lib.rs");
    fs::write(&src1, code).unwrap();
    let g1 = adze_tool::generate_grammars(&src1).unwrap();

    let dir2 = TempDir::new().unwrap();
    let src2 = dir2.path().join("lib.rs");
    fs::write(&src2, code).unwrap();
    let g2 = adze_tool::generate_grammars(&src2).unwrap();

    assert_eq!(
        serde_json::to_string(&g1).unwrap(),
        serde_json::to_string(&g2).unwrap(),
        "same source must produce identical JSON"
    );
}

#[test]
fn e2e_determinism_same_grammar_js_same_parser_code() {
    let js = r#"
module.exports = grammar({
  name: 'det_code',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#;
    let r1 = build_js(js);
    let r2 = build_js(js);
    assert_eq!(
        r1.parser_code, r2.parser_code,
        "same grammar.js must produce identical parser code"
    );
}

#[test]
fn e2e_determinism_same_grammar_js_same_node_types() {
    let js = r#"
module.exports = grammar({
  name: 'det_nt',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#;
    let r1 = build_js(js);
    let r2 = build_js(js);
    assert_eq!(
        r1.node_types_json, r2.node_types_json,
        "same grammar.js must produce identical NODE_TYPES"
    );
}

// =========================================================================
// 26–30  Error cases end-to-end
// =========================================================================

#[test]
fn e2e_error_invalid_json_to_builder() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("{{{bad".into(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn e2e_error_missing_grammar_js_file() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_grammar_js(Path::new("/no/such/grammar.js"), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn e2e_error_empty_grammar_js() {
    let result = try_build_js("");
    assert!(result.is_err(), "empty grammar.js must fail");
}

#[test]
fn e2e_error_grammar_js_no_rules() {
    let result = try_build_js(
        r#"
module.exports = grammar({
  name: 'norules',
  rules: {}
});
"#,
    );
    assert!(result.is_err(), "grammar.js with empty rules must fail");
}

#[test]
fn e2e_error_garbage_content_in_grammar_js() {
    let result = try_build_js("NOT JAVASCRIPT AT ALL 123 !!!");
    assert!(result.is_err(), "garbage content must fail");
}

// =========================================================================
// 31–33  Additional pipeline integration tests
// =========================================================================

#[test]
fn e2e_build_from_ir_grammar_directly() {
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let dir = TempDir::new().unwrap();
    let mut grammar = Grammar::new("ir_e2e".into());

    let tok_id = SymbolId(1);
    let src_id = SymbolId(2);

    grammar.tokens.insert(
        tok_id,
        Token {
            name: "ident".into(),
            pattern: TokenPattern::Regex(r"[a-z]+".into()),
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
    assert_eq!(r.grammar_name, "ir_e2e");
    assert!(r.build_stats.state_count > 0);
    assert!(!r.parser_code.is_empty());
    assert!(Path::new(&r.parser_path).exists());
}

#[test]
fn e2e_compressed_tables_pipeline() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'comp_e2e',
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
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(
        r.parser_code.contains("PARSE_TABLE"),
        "compressed build should contain PARSE_TABLE"
    );
}

#[test]
fn e2e_emit_artifacts_creates_files() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'art_e2e',
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
    let _r = build_parser_from_grammar_js(&path, opts).unwrap();
    let grammar_dir = dir.path().join("grammar_art_e2e");
    assert!(grammar_dir.exists(), "artifact directory must be created");
    assert!(grammar_dir.join("grammar.ir.json").exists());
    assert!(grammar_dir.join("NODE_TYPES.json").exists());
}
