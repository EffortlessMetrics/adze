#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar generation functionality in the tool crate.
//!
//! Covers: grammar extraction from annotated Rust, JSON structure validation,
//! rule expansion (struct, enum, leaf, extras, externals, precedence, Vec, Option),
//! error handling, visualization, and the pure-Rust builder pipeline.

use std::fs;
use tempfile::TempDir;

/// Helper: write Rust source to a temp file and extract grammars.
fn extract(src: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Helper: extract exactly one grammar and return it.
fn extract_one(src: &str) -> serde_json::Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

// ---------------------------------------------------------------------------
// 1. Basic grammar extraction
// ---------------------------------------------------------------------------

#[test]
fn single_struct_language_generates_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("struct_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub name: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "struct_lang");
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("source_file"));
    assert!(rules.contains_key("Root"));
}

#[test]
fn single_enum_language_generates_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_lang")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "enum_lang");
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Token"));
}

#[test]
fn no_grammar_module_yields_empty() {
    let gs = extract("pub fn nothing() {}");
    assert!(gs.is_empty());
}

#[test]
fn nested_module_grammar_extracted() {
    let g = extract_one(
        r#"
        mod outer {
            #[adze::grammar("nested")]
            mod inner {
                #[adze::language]
                pub enum Tok {
                    Lit(#[adze::leaf(pattern = r"\d+")] i32),
                }
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "nested");
}

// ---------------------------------------------------------------------------
// 2. JSON structure validation
// ---------------------------------------------------------------------------

#[test]
fn grammar_json_has_required_top_level_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_test")]
        mod grammar {
            #[adze::language]
            pub enum E {
                A(#[adze::leaf(pattern = r"a")] String),
            }
        }
        "#,
    );
    assert!(g.get("name").is_some());
    assert!(g.get("rules").is_some());
    assert!(g.get("extras").is_some());
}

#[test]
fn grammar_json_roundtrips_through_serde() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub enum E {
                X(#[adze::leaf(pattern = r"x")] String),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(g, reparsed);
}

#[test]
fn source_file_rule_references_language_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("srcfile")]
        mod grammar {
            #[adze::language]
            pub enum MyRoot {
                Tok(#[adze::leaf(pattern = r"\w+")] String),
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "MyRoot");
}

// ---------------------------------------------------------------------------
// 3. Leaf fields — pattern and text
// ---------------------------------------------------------------------------

#[test]
fn leaf_pattern_generates_pattern_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("pat")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub num: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // The generated pattern rule should exist
    let pattern_rule = rules.get("Root_num").unwrap();
    assert_eq!(pattern_rule["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(pattern_rule["value"].as_str().unwrap(), r"\d+");
}

#[test]
fn leaf_text_generates_string_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("txt")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "+")]
                pub op: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let string_rule = rules.get("Root_op").unwrap();
    assert_eq!(string_rule["type"].as_str().unwrap(), "STRING");
    assert_eq!(string_rule["value"].as_str().unwrap(), "+");
}

// ---------------------------------------------------------------------------
// 4. Enum variants (CHOICE)
// ---------------------------------------------------------------------------

#[test]
fn enum_generates_choice_with_variants() {
    let g = extract_one(
        r#"
        #[adze::grammar("choice_test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let expr_rule = &g["rules"]["Expr"];
    assert_eq!(expr_rule["type"].as_str().unwrap(), "CHOICE");
    let members = expr_rule["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[test]
fn enum_with_three_variants_has_three_members() {
    let g = extract_one(
        r#"
        #[adze::grammar("three_var")]
        mod grammar {
            #[adze::language]
            pub enum Kind {
                A(#[adze::leaf(text = "a")] String),
                B(#[adze::leaf(text = "b")] String),
                C(#[adze::leaf(text = "c")] String),
            }
        }
        "#,
    );
    let members = g["rules"]["Kind"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
}

// ---------------------------------------------------------------------------
// 5. Optional fields
// ---------------------------------------------------------------------------

#[test]
fn option_field_generates_choice_with_blank() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub required: String,
                #[adze::leaf(pattern = r"[a-z]+")]
                pub maybe: Option<String>,
            }
        }
        "#,
    );
    let root_rule = &g["rules"]["Root"];
    // Root is a SEQ of two members (required + optional)
    assert_eq!(root_rule["type"].as_str().unwrap(), "SEQ");
    let members = root_rule["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);

    // The optional member should be a CHOICE with BLANK
    let opt_member = &members[1];
    assert_eq!(opt_member["type"].as_str().unwrap(), "CHOICE");
    let choices = opt_member["members"].as_array().unwrap();
    assert!(choices.iter().any(|c| c["type"].as_str() == Some("BLANK")));
}

// ---------------------------------------------------------------------------
// 6. Vec fields (REPEAT)
// ---------------------------------------------------------------------------

#[test]
fn vec_field_generates_repeat_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub items: Vec<String>,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // Vec generates a _vec_contents named rule
    assert!(
        rules.contains_key("Root_items_vec_contents"),
        "should generate vec_contents rule, got keys: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
    let contents = &rules["Root_items_vec_contents"];
    assert_eq!(contents["type"].as_str().unwrap(), "REPEAT1");
}

#[test]
fn vec_non_empty_does_not_wrap_in_choice() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_ne")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                #[adze::repeat(non_empty = true)]
                pub nums: Vec<i32>,
            }
        }
        "#,
    );
    // With non_empty, the reference should be a direct SYMBOL (not wrapped in CHOICE)
    let root_rule = &g["rules"]["Root"];
    // root has a FIELD wrapping a SYMBOL
    let field_content = &root_rule["content"];
    assert_eq!(
        field_content["type"].as_str().unwrap(),
        "SYMBOL",
        "non_empty Vec should reference symbol directly, got: {}",
        serde_json::to_string_pretty(&root_rule).unwrap()
    );
}

// ---------------------------------------------------------------------------
// 7. Extras (whitespace)
// ---------------------------------------------------------------------------

#[test]
fn extra_struct_appears_in_extras_list() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s+")]
                pub ws: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(
        extras
            .iter()
            .any(|e| e["name"].as_str() == Some("Whitespace")),
        "extras should contain Whitespace, got: {:?}",
        extras
    );
}

// ---------------------------------------------------------------------------
// 8. External tokens
// ---------------------------------------------------------------------------

#[test]
fn external_struct_appears_in_externals() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_test")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }

            #[adze::external]
            pub struct Indent {
                #[adze::leaf(pattern = r"\t+")]
                pub tab: String,
            }
        }
        "#,
    );
    let externals = g.get("externals").unwrap().as_array().unwrap();
    assert!(
        externals
            .iter()
            .any(|e| e["name"].as_str() == Some("Indent")),
        "externals should contain Indent, got: {:?}",
        externals
    );
}

// ---------------------------------------------------------------------------
// 9. Precedence attributes
// ---------------------------------------------------------------------------

#[test]
fn prec_left_attribute_generates_prec_left_wrapper() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_l")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_left(1)]
                Add {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    op: String,
                    right: Box<Expr>,
                },
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // Add variant should have been created with PREC_LEFT since it has prec + fields
    let add_rule = rules.get("Expr_Add").unwrap();
    assert_eq!(
        add_rule["type"].as_str().unwrap(),
        "PREC_LEFT",
        "Add rule should be PREC_LEFT, got: {}",
        serde_json::to_string_pretty(add_rule).unwrap()
    );
    assert_eq!(add_rule["value"].as_u64().unwrap(), 1);
}

#[test]
fn prec_right_attribute_generates_prec_right_wrapper() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_r")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_right(2)]
                Assign {
                    left: Box<Expr>,
                    #[adze::leaf(text = "=")]
                    op: String,
                    right: Box<Expr>,
                },
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let assign_rule = rules.get("Expr_Assign").unwrap();
    assert_eq!(assign_rule["type"].as_str().unwrap(), "PREC_RIGHT");
    assert_eq!(assign_rule["value"].as_u64().unwrap(), 2);
}

#[test]
fn prec_attribute_generates_prec_wrapper() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_plain")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec(5)]
                Grouped {
                    #[adze::leaf(text = "(")]
                    open: String,
                    inner: Box<Expr>,
                    #[adze::leaf(text = ")")]
                    close: String,
                },
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let grouped_rule = rules.get("Expr_Grouped").unwrap();
    assert_eq!(grouped_rule["type"].as_str().unwrap(), "PREC");
    assert_eq!(grouped_rule["value"].as_u64().unwrap(), 5);
}

// ---------------------------------------------------------------------------
// 10. Word rule
// ---------------------------------------------------------------------------

#[test]
fn word_annotation_sets_word_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("word_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                #[adze::word]
                pub name: String,
            }
        }
        "#,
    );
    // The "word" key in the grammar should be set
    let word = g.get("word").unwrap();
    assert!(
        !word.is_null(),
        "word should be set when #[adze::word] is used"
    );
}

// ---------------------------------------------------------------------------
// 11. Multi-field struct (SEQ)
// ---------------------------------------------------------------------------

#[test]
fn struct_with_multiple_fields_generates_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("seq_test")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub key: String,
                #[adze::leaf(text = ":")]
                pub sep: String,
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
        "#,
    );
    let pair = &g["rules"]["Pair"];
    assert_eq!(
        pair["type"].as_str().unwrap(),
        "SEQ",
        "multi-field struct should produce SEQ"
    );
    let members = pair["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
}

// ---------------------------------------------------------------------------
// 12. Skip attribute
// ---------------------------------------------------------------------------

#[test]
fn skip_attribute_excludes_field_from_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("skip_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub name: String,
                #[adze::skip]
                pub internal: (),
            }
        }
        "#,
    );
    let root = &g["rules"]["Root"];
    // With one field skipped and one remaining, Root should be a single FIELD
    assert_eq!(root["type"].as_str().unwrap(), "FIELD");
}

// ---------------------------------------------------------------------------
// 13. Error handling
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn missing_file_panics() {
    let _ = adze_tool::generate_grammars(std::path::Path::new("/nonexistent/path.rs"));
}

#[test]
fn tool_error_display_variants() {
    use adze_tool::ToolError;

    let errs: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
        ToolError::StructHasNoFields {
            name: "Empty".into(),
        },
        ToolError::ExpectedStringLiteral {
            context: "test".into(),
            actual: "42".into(),
        },
        ToolError::ExpectedIntegerLiteral {
            actual: "abc".into(),
        },
        ToolError::ExpectedPathType {
            actual: "tuple".into(),
        },
        ToolError::ExpectedSingleSegmentPath { actual: "3".into() },
        ToolError::grammar_validation("bad grammar"),
        ToolError::string_too_long("extract", 999),
        ToolError::complex_symbols_not_normalized("FIRST"),
        ToolError::expected_symbol_type("Terminal"),
        ToolError::expected_action_type("Shift"),
        ToolError::expected_error_type("ParseError"),
        ToolError::Other("custom msg".into()),
    ];

    for e in &errs {
        let msg = format!("{e}");
        assert!(!msg.is_empty(), "display must produce non-empty string");
    }
}

#[test]
fn tool_error_from_str() {
    let e: adze_tool::ToolError = "hello".into();
    assert!(format!("{e}").contains("hello"));
}

#[test]
fn tool_error_from_string() {
    let e: adze_tool::ToolError = String::from("world").into();
    assert!(format!("{e}").contains("world"));
}

// ---------------------------------------------------------------------------
// 14. Visualization
// ---------------------------------------------------------------------------

#[test]
fn visualizer_dot_output_is_valid() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn visualizer_text_contains_grammar_info() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: sample"));
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
}

#[test]
fn visualizer_svg_well_formed() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn visualizer_empty_grammar_does_not_panic() {
    let grammar = adze_ir::Grammar::new("empty".into());
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let _ = viz.to_dot();
    let _ = viz.to_text();
    let _ = viz.to_railroad_svg();
    let _ = viz.dependency_graph();
}

// ---------------------------------------------------------------------------
// 15. Pure-Rust builder pipeline
// ---------------------------------------------------------------------------

#[test]
fn builder_from_json_produces_parser() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

    let dir = TempDir::new().unwrap();
    let g = extract_one(
        r#"
        #[adze::grammar("json_build")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json(json_str, opts).unwrap();
    assert_eq!(result.grammar_name, "json_build");
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn builder_invalid_json_returns_error() {
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

    let opts = BuildOptions {
        out_dir: "/tmp/unused".into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json("{not valid".into(), opts);
    assert!(result.is_err());
}

#[test]
fn builder_from_ir_grammar_succeeds() {
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
    use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

    let dir = TempDir::new().unwrap();
    let mut grammar = Grammar::new("ir_direct".into());

    let tok = SymbolId(1);
    let src = SymbolId(2);

    grammar.tokens.insert(
        tok,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(src, "source_file".into());
    grammar.rules.entry(src).or_default().push(Rule {
        lhs: src,
        rhs: vec![Symbol::Terminal(tok)],
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
    assert_eq!(result.grammar_name, "ir_direct");
    assert!(std::path::Path::new(&result.parser_path).exists());
}

// ---------------------------------------------------------------------------
// 16. Multiple grammars in one file
// ---------------------------------------------------------------------------

#[test]
fn multiple_grammar_modules_extracted() {
    let gs = extract(
        r#"
        #[adze::grammar("first")]
        mod gram_a {
            #[adze::language]
            pub enum A {
                X(#[adze::leaf(pattern = r"x")] String),
            }
        }

        #[adze::grammar("second")]
        mod gram_b {
            #[adze::language]
            pub enum B {
                Y(#[adze::leaf(pattern = r"y")] String),
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 2);
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"first"));
    assert!(names.contains(&"second"));
}

// ---------------------------------------------------------------------------
// 17. Delimited Vec
// ---------------------------------------------------------------------------

#[test]
fn delimited_vec_generates_seq_with_delimiter() {
    let g = extract_one(
        r#"
        #[adze::grammar("delim")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::leaf(pattern = r"\d+")]
                #[adze::delimited(#[adze::leaf(text = ",")] String)]
                pub items: Vec<i32>,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let contents = &rules["List_items_vec_contents"];
    // Delimited Vec generates a SEQ rule
    assert_eq!(
        contents["type"].as_str().unwrap(),
        "SEQ",
        "delimited vec should be SEQ, got: {}",
        serde_json::to_string_pretty(contents).unwrap()
    );
}

// ---------------------------------------------------------------------------
// 18. Grammar converter sample grammar
// ---------------------------------------------------------------------------

#[test]
fn sample_grammar_has_expected_structure() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.name, "sample");
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
    // Should have identifier, number, plus tokens
    let token_names: Vec<&str> = grammar.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(token_names.contains(&"identifier"));
    assert!(token_names.contains(&"number"));
    assert!(token_names.contains(&"plus"));
}

// ---------------------------------------------------------------------------
// 19. BuildOptions defaults
// ---------------------------------------------------------------------------

#[test]
fn build_options_default_values() {
    use adze_tool::pure_rust_builder::BuildOptions;

    let opts = BuildOptions {
        out_dir: "/tmp/test".into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "/tmp/test");
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
}

// ---------------------------------------------------------------------------
// 20. No-inline attribute
// ---------------------------------------------------------------------------

#[test]
fn no_inline_variant_creates_separate_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("noinline")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::no_inline]
                Add {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    op: String,
                    right: Box<Expr>,
                },
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // With no_inline, Expr_Add should be its own named rule
    assert!(
        rules.contains_key("Expr_Add"),
        "no_inline variant should create a separate rule"
    );
    // The Expr CHOICE should reference Expr_Add as a SYMBOL
    let expr = &g["rules"]["Expr"];
    let members = expr["members"].as_array().unwrap();
    let has_symbol_ref = members
        .iter()
        .any(|m| m["type"].as_str() == Some("SYMBOL") && m["name"].as_str() == Some("Expr_Add"));
    assert!(has_symbol_ref, "CHOICE should reference Expr_Add as SYMBOL");
}
