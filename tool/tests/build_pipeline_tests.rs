//! Comprehensive build-pipeline tests for the adze-tool crate.
//!
//! Validates: grammar JSON generation from annotated types, rule definitions,
//! extras/whitespace, precedence declarations, external tokens, C parser output
//! structure, multi-grammar crates, error cases, schema validation, build
//! artifacts, node-types metadata, and unicode rule names.

use std::fs;

use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a Rust source file and extract grammars via `generate_grammars`.
fn grammars_from_rust(code: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, code).unwrap();
    adze_tool::generate_grammars(&src).unwrap()
}

/// Write a grammar.js, build with pure-Rust builder, return result.
fn build_from_grammar_js(js: &str) -> adze_tool::pure_rust_builder::BuildResult {
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

/// Build default options pointing at a temp dir.
fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// =========================================================================
// 1. Grammar JSON generation from annotated types
// =========================================================================

#[test]
fn json_generated_from_annotated_enum() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("from_enum")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Ident(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0]["name"].as_str().unwrap(), "from_enum");
    assert!(gs[0]["rules"].is_object());
}

// =========================================================================
// 2. Grammar JSON includes correct rule definitions
// =========================================================================

#[test]
fn json_contains_source_file_rule() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("rules_check")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = gs[0]["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("source_file"),
        "generated grammar must contain 'source_file' rule; keys: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
}

#[test]
fn json_rule_for_variant_is_present() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("variant_rule")]
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
    // The top-level enum becomes a CHOICE; variant rules are generated
    // with names like Expression_Num, Expression_Id (or Expr_Num, Expr_Id).
    // At minimum, the rules object should have more than just source_file.
    assert!(
        rules.len() > 1,
        "should have variant-level rules; got keys: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
}

// =========================================================================
// 3. Grammar JSON includes correct extras (whitespace)
// =========================================================================

#[test]
fn json_extras_contains_whitespace_symbol() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("ws_extras")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let extras = gs[0]["extras"].as_array().expect("must have extras array");
    let extra_names: Vec<&str> = extras
        .iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        extra_names
            .iter()
            .any(|n| n.contains("Whitespace") || n.contains("whitespace")),
        "extras should reference a Whitespace symbol; got: {:?}",
        extra_names
    );
}

#[test]
fn json_without_extras_has_empty_or_missing_extras() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("no_extras")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    // Either "extras" is missing or it's an empty array
    match gs[0].get("extras") {
        None => {} // ok
        Some(v) => {
            let arr = v.as_array().unwrap();
            assert!(
                arr.is_empty(),
                "grammar with no #[adze::extra] should have empty extras; got: {:?}",
                arr
            );
        }
    }
}

// =========================================================================
// 4. Grammar JSON includes correct precedence declarations
// =========================================================================

#[test]
fn json_prec_left_variant_wraps_in_prec_left() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("prec_check")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string_pretty(&gs[0]).unwrap();
    // The generated JSON should contain a PREC_LEFT node somewhere
    assert!(
        json_str.contains("PREC_LEFT"),
        "generated grammar JSON must contain PREC_LEFT; snippet:\n{}",
        &json_str[..json_str.len().min(2000)]
    );
}

// =========================================================================
// 5. Grammar JSON includes external tokens when present
// =========================================================================

#[test]
fn json_externals_present_when_declared() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("ext_tok")]
        mod grammar {
            #[adze::language]
            pub enum Lang {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }

            #[adze::external]
            struct Indent {
                #[adze::leaf(pattern = r"\t+")]
                _indent: (),
            }
        }
        "#,
    );
    assert!(
        gs[0].get("externals").is_some(),
        "grammar with #[adze::external] must have 'externals' key"
    );
    let externals = gs[0]["externals"].as_array().unwrap();
    assert!(!externals.is_empty(), "externals array must not be empty");
}

#[test]
fn json_no_externals_key_when_none_declared() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("no_ext")]
        mod grammar {
            #[adze::language]
            pub enum Lang {
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    assert!(
        gs[0].get("externals").is_none(),
        "grammar without externals should not contain 'externals' key"
    );
}

// =========================================================================
// 6. C parser code generation produces valid C (basic structure check)
//    NOTE: We test the pure-Rust path since the C path requires tree-sitter CLI.
//    The generated Rust code is formatted by prettyplease, so basic checks suffice.
// =========================================================================

#[test]
fn generated_parser_code_contains_language_struct() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'c_struct_check',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    // The generated Rust code should reference the static LANGUAGE struct.
    assert!(
        result.parser_code.contains("LANGUAGE")
            || result.parser_code.contains("Language")
            || result.parser_code.contains("language"),
        "generated code must reference the language struct"
    );
}

#[test]
fn generated_parser_code_is_nonempty_and_has_rust_keywords() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'rust_kw',
  rules: {
    source: $ => $.item,
    item: $ => /[0-9]+/
  }
});
"#,
    );
    assert!(!result.parser_code.is_empty());
    // prettyplease-formatted Rust should contain typical keywords
    assert!(
        result.parser_code.contains("static") || result.parser_code.contains("const"),
        "generated Rust code should contain static/const declarations"
    );
}

// =========================================================================
// 7. Build pipeline handles multiple grammars in one crate
// =========================================================================

#[test]
fn multiple_grammars_in_one_file() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("multi_a")]
        mod grammar_a {
            #[adze::language]
            pub enum A {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }

        #[adze::grammar("multi_b")]
        mod grammar_b {
            #[adze::language]
            pub enum B {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }

        #[adze::grammar("multi_c")]
        mod grammar_c {
            #[adze::language]
            pub enum C {
                Hex(#[adze::leaf(pattern = r"[0-9a-f]+")] String),
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 3, "should extract exactly three grammars");
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"multi_a"));
    assert!(names.contains(&"multi_b"));
    assert!(names.contains(&"multi_c"));
}

// =========================================================================
// 8. Build pipeline handles grammar with no rules (error case)
// =========================================================================

#[test]
fn grammar_with_no_rules_in_grammar_js_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'empty_rules',
  rules: {}
});
"#,
    )
    .unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_grammar_js(&path, opts);
    assert!(
        result.is_err(),
        "grammar with no rules should fail to build"
    );
}

// =========================================================================
// 9. Build pipeline handles grammar with single rule
// =========================================================================

#[test]
fn single_rule_grammar_builds() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'single_rule',
  rules: {
    source: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "single_rule");
    assert!(result.build_stats.state_count > 0);
    assert!(!result.parser_code.is_empty());
}

// =========================================================================
// 10. Build pipeline handles grammar with 50+ rules
// =========================================================================

#[test]
fn large_grammar_with_many_rules_builds() {
    // Generate a grammar.js with 50+ rules programmatically
    let mut rules = String::from("source: $ => choice(\n");
    for i in 0..55 {
        if i > 0 {
            rules.push_str(",\n");
        }
        rules.push_str(&format!("      $.rule_{i}"));
    }
    rules.push_str("\n    )");
    for i in 0..55 {
        rules.push_str(&format!(",\n    rule_{i}: $ => /token_{i}/"));
    }
    let js = format!(
        r#"
module.exports = grammar({{
  name: 'large_grammar',
  rules: {{
    {rules}
  }}
}});
"#
    );

    let result = build_from_grammar_js(&js);
    assert_eq!(result.grammar_name, "large_grammar");
    assert!(
        result.build_stats.symbol_count >= 55,
        "symbol_count ({}) should be >= 55 for a 55-rule grammar",
        result.build_stats.symbol_count
    );
}

// =========================================================================
// 11. Generated grammar validates against JSON schema (basic structure)
// =========================================================================

#[test]
fn generated_json_has_required_top_level_keys() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("schema_check")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let g = &gs[0];
    // Tree-sitter grammar JSON requires at minimum: name, rules
    assert!(
        g.get("name").and_then(|v| v.as_str()).is_some(),
        "must have string 'name'"
    );
    assert!(
        g.get("rules").and_then(|v| v.as_object()).is_some(),
        "must have object 'rules'"
    );
    // word is optional but should be present (may be null)
    assert!(
        g.get("word").is_some(),
        "should have 'word' key (may be null)"
    );
}

#[test]
fn generated_json_rule_values_are_objects_with_type() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("rule_types")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = gs[0]["rules"].as_object().unwrap();
    for (name, rule_value) in rules {
        assert!(
            rule_value.is_object(),
            "rule '{}' value should be an object; got: {:?}",
            name,
            rule_value
        );
        assert!(
            rule_value.get("type").is_some(),
            "rule '{}' object should have a 'type' field",
            name
        );
    }
}

// =========================================================================
// 12. Build artifacts are placed in correct output directory
// =========================================================================

#[test]
fn parser_file_placed_in_out_dir() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'outdir_check',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_grammar_js(&grammar_path, opts).unwrap();

    let parser_path = std::path::Path::new(&result.parser_path);
    assert!(parser_path.exists(), "parser file must exist");
    assert!(
        parser_path.starts_with(dir.path()),
        "parser file ({}) must be under out_dir ({})",
        parser_path.display(),
        dir.path().display()
    );
}

#[test]
fn emit_artifacts_creates_debug_files_in_out_dir() {
    let dir = TempDir::new().unwrap();
    let grammar_path = dir.path().join("grammar.js");
    fs::write(
        &grammar_path,
        r#"
module.exports = grammar({
  name: 'artifact_dir',
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

    let grammar_dir = dir.path().join("grammar_artifact_dir");
    assert!(grammar_dir.exists(), "artifact directory must be created");
    assert!(
        grammar_dir.join("grammar.ir.json").exists(),
        "IR JSON artifact must exist"
    );
    assert!(
        grammar_dir.join("NODE_TYPES.json").exists(),
        "NODE_TYPES artifact must exist"
    );
}

// =========================================================================
// 13. Grammar name appears in generated code
// =========================================================================

#[test]
fn grammar_name_in_generated_rust_code() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'my_unique_lang',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "my_unique_lang");
    // The name typically appears in comments, doc-strings, or constant names
    let code_lower = result.parser_code.to_lowercase();
    assert!(
        code_lower.contains("my_unique_lang"),
        "grammar name 'my_unique_lang' should appear somewhere in generated code"
    );
}

#[test]
fn grammar_name_from_rust_annotation_propagates() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("fancy_name_42")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                X(#[adze::leaf(pattern = r"x")] String),
            }
        }
        "#,
    );
    assert_eq!(gs[0]["name"].as_str().unwrap(), "fancy_name_42");
}

// =========================================================================
// 14. Node types metadata is generated correctly
// =========================================================================

#[test]
fn node_types_json_is_valid_array() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'nt_valid',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    let node_types: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("node_types_json must be valid JSON");
    assert!(node_types.is_array(), "node_types must be a JSON array");
}

#[test]
fn node_types_entries_have_type_field() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'nt_type',
  rules: {
    source: $ => $.item,
    item: $ => /[a-z]+/
  }
});
"#,
    );
    let node_types: Vec<serde_json::Value> = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in &node_types {
        assert!(
            entry.get("type").is_some(),
            "each node_type entry must have 'type'; got: {:?}",
            entry
        );
    }
}

#[test]
fn node_types_contains_named_node_for_source() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'nt_source',
  rules: {
    source: $ => $.tok,
    tok: $ => /abc/
  }
});
"#,
    );
    let node_types: Vec<serde_json::Value> = serde_json::from_str(&result.node_types_json).unwrap();
    let types: Vec<&str> = node_types
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(
        types.contains(&"source"),
        "node_types should contain a 'source' entry; got: {:?}",
        types
    );
}

// =========================================================================
// 15. Pipeline handles unicode rule names gracefully
// =========================================================================

#[test]
fn unicode_grammar_name_accepted() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    // Tree-sitter grammar names are typically ASCII, but the pipeline
    // should not panic on non-ASCII input.
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'lang_日本語',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    )
    .unwrap();
    let opts = opts_in(&dir);
    // May succeed or error, but must not panic.
    let result = build_parser_from_grammar_js(&path, opts);
    match result {
        Ok(r) => assert!(r.grammar_name.contains("日本語")),
        Err(e) => {
            // Controlled error is fine; just make sure we get a message, not a panic.
            let msg = format!("{e}");
            assert!(!msg.is_empty());
        }
    }
}

// =========================================================================
// Additional tests (16-25) to reach 20+ total
// =========================================================================

#[test]
fn build_stats_symbol_count_positive() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'sym_count',
  rules: {
    source: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#,
    );
    assert!(
        result.build_stats.symbol_count > 0,
        "symbol_count must be positive"
    );
    assert!(
        result.build_stats.state_count > 0,
        "state_count must be positive"
    );
}

#[test]
fn build_parser_from_json_round_trip() {
    // Extract grammar JSON from Rust source, then feed it into the JSON builder.
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("json_rt")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let dir = TempDir::new().unwrap();
    let json_str = serde_json::to_string(&gs[0]).unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_json(json_str, opts).unwrap();
    assert_eq!(result.grammar_name, "json_rt");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_parser_from_ir_grammar_directly() {
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let dir = TempDir::new().unwrap();
    let mut grammar = Grammar::new("direct_ir_test".into());

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

    let opts = opts_in(&dir);
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "direct_ir_test");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn compressed_tables_succeed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'compressed_tbl',
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
    let result = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn malformed_json_returns_descriptive_error() {
    let dir = TempDir::new().unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_json("{{{bad".into(), opts);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.to_lowercase().contains("json") || msg.to_lowercase().contains("parse"),
        "error should mention JSON/parse: {msg}"
    );
}

#[test]
fn grammar_with_string_literal_token_builds() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'string_tok',
  rules: {
    source: $ => seq($.kw, $.ident),
    kw: $ => 'let',
    ident: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "string_tok");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn grammar_with_choice_builds() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'choice_test',
  rules: {
    source: $ => choice($.num, $.word),
    num: $ => /[0-9]+/,
    word: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "choice_test");
    assert!(result.build_stats.symbol_count >= 3);
}

#[test]
fn grammar_with_optional_builds() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'opt_test',
  rules: {
    source: $ => seq($.a, optional($.b)),
    a: $ => /a/,
    b: $ => /b/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "opt_test");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_with_repeat_builds() {
    let result = build_from_grammar_js(
        r#"
module.exports = grammar({
  name: 'repeat_test',
  rules: {
    source: $ => repeat($.item),
    item: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(result.grammar_name, "repeat_test");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn nonexistent_grammar_js_errors() {
    let dir = TempDir::new().unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_grammar_js(std::path::Path::new("/no/such/file.js"), opts);
    assert!(result.is_err(), "missing grammar.js must fail");
}
