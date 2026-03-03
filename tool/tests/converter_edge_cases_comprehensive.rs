#![allow(clippy::needless_range_loop)]

//! Edge case tests for the converter module in adze-tool.
//!
//! Tests Rust type definition → Tree-sitter grammar JSON conversion through
//! `generate_grammars`, covering empty structs, single-field structs, all
//! annotation types, single-variant enums, deep nesting, recursive types,
//! generics (Box, Option, Vec), conversion error handling, and Unicode
//! identifiers.

use std::fs;
use tempfile::TempDir;

use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule as IrRule, Symbol,
    SymbolId, Token, TokenPattern,
};
use adze_tool::GrammarConverter;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write Rust source to a temp file and extract grammars.
fn extract(src: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Extract exactly one grammar and return it.
fn extract_one(src: &str) -> serde_json::Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

/// Try to extract grammars, returning the ToolResult.
fn try_extract(src: &str) -> adze_tool::ToolResult<Vec<serde_json::Value>> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path)
}

/// Get the "rules" object from a grammar JSON value.
fn rules_of(g: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    g.get("rules").unwrap().as_object().unwrap()
}

// ===========================================================================
// 1. Empty struct conversion — must error (no non-skipped fields)
// ===========================================================================

#[test]
fn empty_struct_errors() {
    let result = try_extract(
        r#"
        #[adze::grammar("empty_struct")]
        mod grammar {
            #[adze::language]
            pub struct Empty {}
        }
        "#,
    );
    assert!(result.is_err(), "empty struct should fail conversion");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("no non-skipped fields") || err_msg.contains("no fields"),
        "error should mention missing fields: {err_msg}"
    );
}

// ===========================================================================
// 2. Single-field struct
// ===========================================================================

#[test]
fn single_field_struct_produces_field_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("single_field")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let root = &rules["Root"];
    // Single field struct should produce a FIELD node (not wrapped in SEQ)
    assert_eq!(
        root.get("type").and_then(|t| t.as_str()),
        Some("FIELD"),
        "single-field struct should produce a FIELD, not SEQ"
    );
}

#[test]
fn single_field_struct_field_name_matches() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_name")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "hello")]
                greeting: (),
            }
        }
        "#,
    );
    let root = &rules_of(&g)["Root"];
    assert_eq!(
        root.get("name").and_then(|n| n.as_str()),
        Some("greeting"),
        "field name should be 'greeting'"
    );
}

// ===========================================================================
// 3. Struct with all annotation types
// ===========================================================================

#[test]
fn struct_with_pattern_and_text_fields() {
    let g = extract_one(
        r#"
        #[adze::grammar("all_annot")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                num: i32,
                #[adze::leaf(text = "+")]
                op: (),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let root = &rules["Root"];
    assert_eq!(root.get("type").and_then(|t| t.as_str()), Some("SEQ"));
    let members = root.get("members").unwrap().as_array().unwrap();
    assert_eq!(members.len(), 2, "two fields → SEQ with 2 members");
}

#[test]
fn struct_with_optional_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_field")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                required: i32,
                #[adze::leaf(pattern = r"[a-z]+")]
                maybe: Option<String>,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let root = &rules["Root"];
    // Should be a SEQ; the optional field should include a CHOICE with BLANK
    let members = root.get("members").unwrap().as_array().unwrap();
    let opt_member = &members[1];
    assert_eq!(
        opt_member.get("type").and_then(|t| t.as_str()),
        Some("CHOICE"),
        "optional field should be wrapped in CHOICE"
    );
}

#[test]
fn struct_with_vec_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_field")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                items: Vec<String>,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    // Vec generates a _vec_contents auxiliary rule
    let has_vec_contents = rules.keys().any(|k| k.contains("vec_contents"));
    assert!(
        has_vec_contents,
        "Vec field should produce a _vec_contents rule"
    );
}

#[test]
fn struct_with_repeat_non_empty_vec() {
    let g = extract_one(
        r#"
        #[adze::grammar("nonempty_vec")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                #[adze::leaf(pattern = r"\d+")]
                nums: Vec<i32>,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let root = &rules["Root"];
    // non_empty = true → direct SYMBOL reference (no CHOICE with BLANK)
    let root_str = serde_json::to_string(root).unwrap();
    // The root should reference the vec_contents symbol directly
    assert!(
        root_str.contains("vec_contents"),
        "non-empty Vec should reference vec_contents: {root_str}"
    );
}

#[test]
fn struct_with_delimited_vec() {
    let g = extract_one(
        r#"
        #[adze::grammar("delim_vec")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                #[adze::leaf(pattern = r"\d+")]
                items: Vec<i32>,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    // Should produce both vec_contents and vec_delimiter rules
    let has_delimiter = rules.keys().any(|k| k.contains("delimiter"));
    assert!(
        has_delimiter,
        "delimited Vec should produce a delimiter rule"
    );
}

#[test]
fn struct_with_skip_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("skip_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                value: i32,
                #[adze::skip]
                _ignored: (),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let root = &rules["Root"];
    // With skip, only one field remains, so it should be a FIELD not SEQ
    assert_eq!(
        root.get("type").and_then(|t| t.as_str()),
        Some("FIELD"),
        "skip field should be excluded, leaving single FIELD"
    );
}

#[test]
fn struct_with_extra_whitespace() {
    let g = extract_one(
        r#"
        #[adze::grammar("extra_ws")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                value: i32,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let extras = g.get("extras").unwrap().as_array().unwrap();
    assert!(
        !extras.is_empty(),
        "extras should include Whitespace symbol"
    );
}

// ===========================================================================
// 4. Enum with single variant
// ===========================================================================

#[test]
fn enum_single_variant_produces_choice_with_one_member() {
    let g = extract_one(
        r#"
        #[adze::grammar("single_var")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let token = &rules["Token"];
    assert_eq!(
        token.get("type").and_then(|t| t.as_str()),
        Some("CHOICE"),
        "enum should always produce a CHOICE"
    );
    let members = token.get("members").unwrap().as_array().unwrap();
    assert_eq!(members.len(), 1, "single variant → one CHOICE member");
}

#[test]
fn enum_single_unit_variant() {
    let g = extract_one(
        r#"
        #[adze::grammar("unit_var")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "keyword")]
                Keyword(()),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    assert!(
        rules.contains_key("Token"),
        "Token enum should produce a rule"
    );
}

// ===========================================================================
// 5. Deeply nested types
// ===========================================================================

#[test]
fn deeply_nested_enum_three_levels() {
    let g = extract_one(
        r#"
        #[adze::grammar("deep")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                stmt: Statement,
            }

            pub enum Statement {
                Expr(Expression),
            }

            pub enum Expression {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    assert!(rules.contains_key("Program"));
    assert!(rules.contains_key("Statement"));
    assert!(rules.contains_key("Expression"));
}

#[test]
fn nested_option_is_rejected() {
    let result = try_extract(
        r#"
        #[adze::grammar("nested_opt")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                v: Option<Option<i32>>,
            }
        }
        "#,
    );
    assert!(
        result.is_err(),
        "Option<Option<_>> should be rejected"
    );
}

// ===========================================================================
// 6. Recursive type references (Box<Self>)
// ===========================================================================

#[test]
fn recursive_enum_with_box() {
    let g = extract_one(
        r#"
        #[adze::grammar("recursive")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Neg(
                    #[adze::leaf(text = "-")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let expr = &rules["Expr"];
    let members = expr.get("members").unwrap().as_array().unwrap();
    assert_eq!(members.len(), 2, "two variants in Expr");
    // The Neg variant should reference Expr recursively
    let grammar_str = serde_json::to_string(&g).unwrap();
    assert!(
        grammar_str.contains("\"name\":\"Expr\""),
        "should contain recursive reference to Expr"
    );
}

#[test]
fn recursive_binary_tree_enum() {
    let g = extract_one(
        r#"
        #[adze::grammar("bintree")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    assert!(
        rules.contains_key("Expr"),
        "should generate Expr rule for binary recursive type"
    );
}

// ===========================================================================
// 7. Types with generics — Box, Option, Vec, Spanned
// ===========================================================================

#[test]
fn box_type_is_transparent() {
    let g = extract_one(
        r#"
        #[adze::grammar("box_trans")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Box<Inner>,
            }

            pub struct Inner {
                #[adze::leaf(pattern = r"\d+")]
                v: i32,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    // Box should be skipped over; the field should reference Inner
    let root = &rules["Root"];
    let root_str = serde_json::to_string(root).unwrap();
    assert!(
        root_str.contains("\"name\":\"Inner\""),
        "Box<Inner> should resolve to Inner: {root_str}"
    );
}

#[test]
fn option_vec_combination() {
    // Vec<Option<T>> is not a nested option, so should work
    let g = extract_one(
        r#"
        #[adze::grammar("opt_vec")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                items: Vec<Option<i32>>,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let has_vec_contents = rules.keys().any(|k| k.contains("vec_contents"));
    assert!(has_vec_contents, "should generate vec_contents rule");
}

// ===========================================================================
// 8. Conversion error handling
// ===========================================================================

#[test]
fn empty_pattern_is_rejected() {
    let result = try_extract(
        r#"
        #[adze::grammar("empty_pat")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = "")]
                v: String,
            }
        }
        "#,
    );
    assert!(result.is_err(), "empty pattern should be rejected");
}

#[test]
fn multiple_word_rules_error() {
    let result = try_extract(
        r#"
        #[adze::grammar("multi_word")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                w: Word,
            }

            #[adze::word]
            pub struct Word {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }

            #[adze::word]
            pub struct Word2 {
                #[adze::leaf(pattern = r"[A-Z]+")]
                v: String,
            }
        }
        "#,
    );
    assert!(
        result.is_err(),
        "multiple word rules should be rejected"
    );
}

#[test]
fn no_grammar_attribute_yields_empty() {
    let grammars = extract(
        r#"
        mod grammar {
            pub struct Root {
                value: i32,
            }
        }
        "#,
    );
    assert!(grammars.is_empty(), "no #[adze::grammar] → no grammars");
}

#[test]
fn struct_with_all_fields_skipped_errors() {
    let result = try_extract(
        r#"
        #[adze::grammar("all_skip")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::skip]
                _a: (),
                #[adze::skip]
                _b: (),
            }
        }
        "#,
    );
    assert!(
        result.is_err(),
        "struct with all fields skipped should error"
    );
}

// ===========================================================================
// 9. Unicode identifiers
// ===========================================================================

#[test]
fn unicode_grammar_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("café")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    assert_eq!(
        g.get("name").and_then(|n| n.as_str()),
        Some("café"),
        "grammar name should preserve Unicode"
    );
}

#[test]
fn unicode_text_literal_in_leaf() {
    let g = extract_one(
        r#"
        #[adze::grammar("unicode_text")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "→")]
                arrow: (),
                #[adze::leaf(pattern = r"\d+")]
                num: i32,
            }
        }
        "#,
    );
    let grammar_str = serde_json::to_string(&g).unwrap();
    assert!(
        grammar_str.contains("→"),
        "Unicode text literal should be preserved"
    );
}

// ===========================================================================
// 10. IR-level GrammarConverter edge cases
// ===========================================================================

#[test]
fn ir_grammar_with_no_rules_validates() {
    let g = Grammar::new("empty_ir".into());
    assert!(g.validate().is_ok(), "empty IR grammar should validate");
}

#[test]
fn ir_grammar_single_token_only() {
    let mut g = Grammar::new("tok_only".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    assert!(g.validate().is_ok());
    assert!(g.tokens.len() == 1);
    assert!(g.rules.is_empty());
}

#[test]
fn ir_grammar_fragile_token() {
    let mut g = Grammar::new("fragile".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "ws".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: true,
        },
    );
    let tok = g.tokens.get(&SymbolId(1)).unwrap();
    assert!(tok.fragile, "fragile flag should be preserved");
}

#[test]
fn ir_rule_with_empty_rhs() {
    let mut g = Grammar::new("empty_rhs".into());
    g.add_rule(IrRule {
        lhs: SymbolId(1),
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let rules = g.get_rules_for_symbol(SymbolId(1)).unwrap();
    assert_eq!(rules.len(), 1);
    assert!(rules[0].rhs.is_empty());
}

#[test]
fn ir_rule_precedence_and_associativity() {
    let mut g = Grammar::new("prec_test".into());
    g.add_rule(IrRule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(42)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });
    let rule = &g.get_rules_for_symbol(SymbolId(1)).unwrap()[0];
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(42)));
    assert_eq!(rule.associativity, Some(Associativity::Right));
    assert_eq!(rule.fields.len(), 1);
}

#[test]
fn ir_normalize_idempotent_on_simple_grammar() {
    let mut g = GrammarConverter::create_sample_grammar();
    let first = g.normalize();
    let first_len = first.len();
    let second = g.normalize();
    assert_eq!(
        first_len,
        second.len(),
        "normalizing twice should be idempotent"
    );
}

// ===========================================================================
// 11. Multi-variant enum with precedence
// ===========================================================================

#[test]
fn enum_with_prec_left_generates_prec_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let grammar_str = serde_json::to_string(&g).unwrap();
    assert!(
        grammar_str.contains("PREC_LEFT"),
        "should generate PREC_LEFT node: {grammar_str}"
    );
}

#[test]
fn enum_with_prec_right_generates_prec_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_right_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] i32),
                #[adze::prec_right(2)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let grammar_str = serde_json::to_string(&g).unwrap();
    assert!(
        grammar_str.contains("PREC_RIGHT"),
        "should generate PREC_RIGHT node"
    );
}

// ===========================================================================
// 12. Source file rule and root reference
// ===========================================================================

#[test]
fn source_file_rule_references_root_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("root_ref")]
        mod grammar {
            #[adze::language]
            pub struct MyRoot {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let source_file = &rules["source_file"];
    assert_eq!(
        source_file.get("type").and_then(|t| t.as_str()),
        Some("SYMBOL")
    );
    assert_eq!(
        source_file.get("name").and_then(|n| n.as_str()),
        Some("MyRoot"),
        "source_file should reference the #[adze::language] type"
    );
}

// ===========================================================================
// 13. External token annotation
// ===========================================================================

#[test]
fn external_token_appears_in_externals() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_tok")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                v: i32,
            }

            #[adze::external]
            struct Indent;
        }
        "#,
    );
    let externals = g
        .get("externals")
        .and_then(|e| e.as_array())
        .expect("externals should exist");
    let names: Vec<&str> = externals
        .iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"Indent"),
        "Indent should appear in externals"
    );
}
