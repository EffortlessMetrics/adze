#![allow(clippy::needless_range_loop)]

//! Property-based tests for C code generation in adze-tool.
//!
//! Uses proptest to validate invariants of the C code produced by the
//! Tree-sitter code generation pipeline:
//!   - C output is deterministic
//!   - C output contains parser function
//!   - C output contains tree-sitter API calls
//!   - C output compiles (syntax checks)
//!   - C generation with various grammars
//!   - C output includes symbol tables
//!   - C output with external scanner stubs

use adze_tool::generate_grammars;
use proptest::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tree_sitter_generate::generate_parser_for_grammar;

const SEMANTIC_VERSION: Option<(u8, u8, u8)> = Some((0, 25, 1));

// ===========================================================================
// Helpers
// ===========================================================================

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write Rust source to a temp file and return (dir, path) so the dir lives
/// long enough for parsing.
fn write_temp(src: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.path().join(format!("g_{id}.rs"));
    std::fs::write(&path, src).unwrap();
    (dir, path)
}

/// Extract one grammar JSON from source.
fn extract_one(src: &str) -> serde_json::Value {
    let (dir, path) = write_temp(src);
    let gs = generate_grammars(&path).unwrap();
    drop(dir);
    assert_eq!(gs.len(), 1);
    gs.into_iter().next().unwrap()
}

/// Generate C code from grammar source. Returns (grammar_name, c_code).
fn gen_c(src: &str) -> (String, String) {
    let grammar = extract_one(src);
    let json = serde_json::to_string(&grammar).unwrap();
    generate_parser_for_grammar(&json, SEMANTIC_VERSION).unwrap()
}

// ===========================================================================
// Source generators
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
}

fn src_enum(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}
}}
"#
    )
}

fn src_enum_n(name: &str, n: usize) -> String {
    let variants: String = (0..n)
        .map(|i| {
            format!(
                r#"        V{i}(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        )"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Root {{
{variants}
    }}
}}
"#
    )
}

fn src_struct(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        value: u32,
    }}
}}
"#
    )
}

fn src_struct_n(name: &str, n: usize) -> String {
    let fields: String = (0..n)
        .map(|i| {
            format!(
                r#"        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        f{i}: u32,"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
{fields}
    }}
}}
"#
    )
}

fn src_extras(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}

    #[adze::extra]
    struct Whitespace {{
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }}
}}
"#
    )
}

fn src_recursive(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
        Neg(
            #[adze::leaf(text = "-", transform = |v| ())]
            (),
            Box<Expr>,
        ),
    }}
}}
"#
    )
}

fn src_prec_left(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
        #[adze::prec_left(1)]
        Add(
            Box<Expr>,
            #[adze::leaf(text = "+", transform = |v| ())]
            (),
            Box<Expr>,
        ),
    }}
}}
"#
    )
}

fn src_optional(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        required: u32,
        #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
        opt: Option<String>,
    }}
}}
"#
    )
}

fn src_vec(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::repeat(non_empty = true)]
        items: Vec<Item>,
    }}

    pub struct Item {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: u32,
    }}

    #[adze::extra]
    struct Whitespace {{
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }}
}}
"#
    )
}

fn src_text_leaves(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Op {{
        Plus(
            #[adze::leaf(text = "+", transform = |v| ())]
            ()
        ),
        Minus(
            #[adze::leaf(text = "-", transform = |v| ())]
            ()
        ),
    }}
}}
"#
    )
}

fn src_unboxed(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Language {{
        e: Expression,
    }}

    pub enum Expression {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }}
}}
"#
    )
}

fn src_multi(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|i| {
            let gname = format!("{prefix}_{i}");
            format!(
                r#"
#[adze::grammar("{gname}")]
mod m{i} {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}
}}
"#
            )
        })
        .collect()
}

// ===========================================================================
// 1. C output is deterministic (tests 1-4)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 1. Same enum grammar always produces identical C code.
    #[test]
    fn c_output_deterministic_enum(name in grammar_name()) {
        let (_, c1) = gen_c(&src_enum(&name));
        let (_, c2) = gen_c(&src_enum(&name));
        prop_assert_eq!(&c1, &c2);
    }

    /// 2. Same struct grammar always produces identical C code.
    #[test]
    fn c_output_deterministic_struct(name in grammar_name()) {
        let (_, c1) = gen_c(&src_struct(&name));
        let (_, c2) = gen_c(&src_struct(&name));
        prop_assert_eq!(&c1, &c2);
    }

    /// 3. Same recursive grammar always produces identical C code.
    #[test]
    fn c_output_deterministic_recursive(name in grammar_name()) {
        let (_, c1) = gen_c(&src_recursive(&name));
        let (_, c2) = gen_c(&src_recursive(&name));
        prop_assert_eq!(&c1, &c2);
    }

    /// 4. Same prec_left grammar always produces identical C code.
    #[test]
    fn c_output_deterministic_prec(name in grammar_name()) {
        let (_, c1) = gen_c(&src_prec_left(&name));
        let (_, c2) = gen_c(&src_prec_left(&name));
        prop_assert_eq!(&c1, &c2);
    }
}

// ===========================================================================
// 2. C code contains parser function (tests 5-9)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 5. Enum grammar C code contains tree_sitter_ parser function.
    #[test]
    fn c_contains_parser_fn_enum(name in grammar_name()) {
        let (gname, c) = gen_c(&src_enum(&name));
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(
            c.contains(&expected),
            "C code missing parser function `{expected}`"
        );
    }

    /// 6. Struct grammar C code contains tree_sitter_ parser function.
    #[test]
    fn c_contains_parser_fn_struct(name in grammar_name()) {
        let (gname, c) = gen_c(&src_struct(&name));
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(
            c.contains(&expected),
            "C code missing parser function `{expected}`"
        );
    }

    /// 7. Grammar with extras produces C code with parser function.
    #[test]
    fn c_contains_parser_fn_extras(name in grammar_name()) {
        let (gname, c) = gen_c(&src_extras(&name));
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(c.contains(&expected));
    }

    /// 8. Recursive grammar produces C code with parser function.
    #[test]
    fn c_contains_parser_fn_recursive(name in grammar_name()) {
        let (gname, c) = gen_c(&src_recursive(&name));
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(c.contains(&expected));
    }

    /// 9. Prec-left grammar produces C code with parser function.
    #[test]
    fn c_contains_parser_fn_prec(name in grammar_name()) {
        let (gname, c) = gen_c(&src_prec_left(&name));
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(c.contains(&expected));
    }
}

// ===========================================================================
// 3. C code contains tree-sitter API calls (tests 10-14)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 10. C code references TSLanguage type.
    #[test]
    fn c_contains_tslanguage(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(c.contains("TSLanguage"), "C code missing TSLanguage");
    }

    /// 11. C code contains ts_builtin_sym references.
    #[test]
    fn c_contains_builtin_sym(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("ts_builtin_sym"),
            "C code missing ts_builtin_sym"
        );
    }

    /// 12. Struct grammar C code references TSLanguage.
    #[test]
    fn c_struct_contains_tslanguage(name in grammar_name()) {
        let (_, c) = gen_c(&src_struct(&name));
        prop_assert!(c.contains("TSLanguage"));
    }

    /// 13. C code includes LANGUAGE_VERSION constant.
    #[test]
    fn c_contains_language_version(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("LANGUAGE_VERSION"),
            "C code missing LANGUAGE_VERSION"
        );
    }

    /// 14. C code includes STATE_COUNT constant.
    #[test]
    fn c_contains_state_count(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("STATE_COUNT"),
            "C code missing STATE_COUNT"
        );
    }
}

// ===========================================================================
// 4. C code compiles – syntax checks (tests 15-19)
// ===========================================================================

/// Verify C code is non-empty and has balanced braces (lightweight syntax
/// check without invoking an actual C compiler).
fn check_c_syntax(c: &str) {
    assert!(!c.is_empty(), "C code is empty");
    // Balanced braces
    let open = c.chars().filter(|&ch| ch == '{').count();
    let close = c.chars().filter(|&ch| ch == '}').count();
    assert_eq!(open, close, "Unbalanced braces in C code");
    // Balanced parens
    let open_p = c.chars().filter(|&ch| ch == '(').count();
    let close_p = c.chars().filter(|&ch| ch == ')').count();
    assert_eq!(open_p, close_p, "Unbalanced parens in C code");
    // Balanced brackets
    let open_b = c.chars().filter(|&ch| ch == '[').count();
    let close_b = c.chars().filter(|&ch| ch == ']').count();
    assert_eq!(open_b, close_b, "Unbalanced brackets in C code");
    // Contains at least one semicolon (statements exist)
    assert!(c.contains(';'), "C code has no semicolons");
    // Contains #include or #define (preprocessor present)
    assert!(
        c.contains("#include") || c.contains("#define"),
        "C code has no preprocessor directives"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 15. Enum grammar C code passes syntax checks.
    #[test]
    fn c_syntax_enum(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        check_c_syntax(&c);
    }

    /// 16. Struct grammar C code passes syntax checks.
    #[test]
    fn c_syntax_struct(name in grammar_name()) {
        let (_, c) = gen_c(&src_struct(&name));
        check_c_syntax(&c);
    }

    /// 17. Recursive grammar C code passes syntax checks.
    #[test]
    fn c_syntax_recursive(name in grammar_name()) {
        let (_, c) = gen_c(&src_recursive(&name));
        check_c_syntax(&c);
    }

    /// 18. Prec-left grammar C code passes syntax checks.
    #[test]
    fn c_syntax_prec(name in grammar_name()) {
        let (_, c) = gen_c(&src_prec_left(&name));
        check_c_syntax(&c);
    }

    /// 19. Optional-field grammar C code passes syntax checks.
    #[test]
    fn c_syntax_optional(name in grammar_name()) {
        let (_, c) = gen_c(&src_optional(&name));
        check_c_syntax(&c);
    }
}

// ===========================================================================
// 5. C code generation with various grammars (tests 20-25)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 20. Varying variant counts produce valid C code.
    #[test]
    fn c_gen_varying_variants(name in grammar_name(), n in 1..=5usize) {
        let (_, c) = gen_c(&src_enum_n(&name, n));
        check_c_syntax(&c);
        prop_assert!(c.contains("TSLanguage"));
    }

    /// 21. Varying struct fields produce valid C code.
    #[test]
    fn c_gen_varying_fields(name in grammar_name(), n in 1..=5usize) {
        let (_, c) = gen_c(&src_struct_n(&name, n));
        check_c_syntax(&c);
        prop_assert!(c.contains("TSLanguage"));
    }

    /// 22. Text-literal grammar produces valid C code.
    #[test]
    fn c_gen_text_leaves(name in grammar_name()) {
        let (_, c) = gen_c(&src_text_leaves(&name));
        check_c_syntax(&c);
    }

    /// 23. Unboxed field grammar produces valid C code.
    #[test]
    fn c_gen_unboxed(name in grammar_name()) {
        let (_, c) = gen_c(&src_unboxed(&name));
        check_c_syntax(&c);
    }

    /// 24. Vec/repeat grammar produces valid C code.
    #[test]
    fn c_gen_vec(name in grammar_name()) {
        let (_, c) = gen_c(&src_vec(&name));
        check_c_syntax(&c);
    }

    /// 25. Multiple grammars each produce separate valid C.
    #[test]
    fn c_gen_multi(prefix in grammar_name(), count in 2..=3usize) {
        let src = src_multi(&prefix, count);
        let (dir, path) = write_temp(&src);
        let grammars = generate_grammars(&path).unwrap();
        drop(dir);
        prop_assert_eq!(grammars.len(), count);
        for g in &grammars {
            let json = serde_json::to_string(g).unwrap();
            let (_, c) = generate_parser_for_grammar(&json, SEMANTIC_VERSION).unwrap();
            check_c_syntax(&c);
        }
    }
}

// ===========================================================================
// 6. C code includes symbol tables (tests 26-30)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 26. C code contains SYMBOL_COUNT.
    #[test]
    fn c_has_symbol_count(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("SYMBOL_COUNT"),
            "C code missing SYMBOL_COUNT"
        );
    }

    /// 27. C code contains ts_symbol_names array.
    #[test]
    fn c_has_symbol_names(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("ts_symbol_names"),
            "C code missing ts_symbol_names"
        );
    }

    /// 28. C code contains ts_symbol_map array.
    #[test]
    fn c_has_symbol_map(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("ts_symbol_map"),
            "C code missing ts_symbol_map"
        );
    }

    /// 29. Struct grammar C code has symbol tables.
    #[test]
    fn c_struct_has_symbol_tables(name in grammar_name()) {
        let (_, c) = gen_c(&src_struct(&name));
        prop_assert!(c.contains("ts_symbol_names"));
        prop_assert!(c.contains("SYMBOL_COUNT"));
    }

    /// 30. Extras grammar C code has symbol tables.
    #[test]
    fn c_extras_has_symbol_tables(name in grammar_name()) {
        let (_, c) = gen_c(&src_extras(&name));
        prop_assert!(c.contains("ts_symbol_names"));
        prop_assert!(c.contains("SYMBOL_COUNT"));
    }
}

// ===========================================================================
// 7. C code with external scanner stubs (tests 31-35)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 31. C code contains EXTERNAL_TOKEN_COUNT (even when 0).
    #[test]
    fn c_has_external_token_count(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("EXTERNAL_TOKEN_COUNT"),
            "C code missing EXTERNAL_TOKEN_COUNT"
        );
    }

    /// 32. Grammar without externals has EXTERNAL_TOKEN_COUNT of 0.
    #[test]
    fn c_external_token_count_zero(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        // The define should be 0 for grammars without external tokens
        prop_assert!(
            c.contains("#define EXTERNAL_TOKEN_COUNT 0"),
            "Expected EXTERNAL_TOKEN_COUNT 0 for grammar without externals"
        );
    }

    /// 33. Recursive grammar also has zero external tokens.
    #[test]
    fn c_recursive_external_zero(name in grammar_name()) {
        let (_, c) = gen_c(&src_recursive(&name));
        prop_assert!(c.contains("#define EXTERNAL_TOKEN_COUNT 0"));
    }

    /// 34. C code contains ts_lex function (scanner entry point).
    #[test]
    fn c_has_lex_function(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        prop_assert!(
            c.contains("ts_lex"),
            "C code missing ts_lex function"
        );
    }

    /// 35. Grammar name in C matches the grammar annotation name.
    #[test]
    fn c_grammar_name_matches(name in grammar_name()) {
        let (gname, _) = gen_c(&src_enum(&name));
        prop_assert_eq!(gname, name);
    }
}

// ===========================================================================
// 8. Grammar JSON generation from module data (tests 36-40)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 36. Grammar JSON has a "name" field matching the annotation.
    #[test]
    fn json_has_name_field(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }

    /// 37. Grammar JSON has a "rules" object.
    #[test]
    fn json_has_rules_object(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        prop_assert!(g["rules"].is_object(), "Expected rules to be an object");
    }

    /// 38. Grammar JSON has "extras" array.
    #[test]
    fn json_has_extras_array(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        prop_assert!(g["extras"].is_array(), "Expected extras to be an array");
    }

    /// 39. Struct grammar JSON has "name" and "rules".
    #[test]
    fn json_struct_has_name_and_rules(name in grammar_name()) {
        let g = extract_one(&src_struct(&name));
        prop_assert!(g["name"].is_string());
        prop_assert!(g["rules"].is_object());
    }

    /// 40. Recursive grammar JSON has "name" and "rules".
    #[test]
    fn json_recursive_has_name_and_rules(name in grammar_name()) {
        let g = extract_one(&src_recursive(&name));
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
        prop_assert!(g["rules"].is_object());
    }
}

// ===========================================================================
// 9. Generated JSON is valid JSON (tests 41-45)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 41. Enum grammar JSON round-trips through serde.
    #[test]
    fn json_roundtrip_enum(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let serialized = serde_json::to_string(&g).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(g, parsed);
    }

    /// 42. Struct grammar JSON round-trips through serde.
    #[test]
    fn json_roundtrip_struct(name in grammar_name()) {
        let g = extract_one(&src_struct(&name));
        let serialized = serde_json::to_string(&g).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(g, parsed);
    }

    /// 43. Recursive grammar JSON round-trips through serde.
    #[test]
    fn json_roundtrip_recursive(name in grammar_name()) {
        let g = extract_one(&src_recursive(&name));
        let serialized = serde_json::to_string(&g).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(g, parsed);
    }

    /// 44. Pretty-printed JSON is also valid.
    #[test]
    fn json_pretty_valid(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let pretty = serde_json::to_string_pretty(&g).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&pretty).unwrap();
        prop_assert_eq!(g, parsed);
    }

    /// 45. Varying-variant grammar JSON is valid.
    #[test]
    fn json_valid_varying_variants(name in grammar_name(), n in 1..=5usize) {
        let g = extract_one(&src_enum_n(&name, n));
        let serialized = serde_json::to_string(&g).unwrap();
        let _: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    }
}

// ===========================================================================
// 10. Grammar rules appear in generated JSON (tests 46-50)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 46. source_file rule exists in JSON rules.
    #[test]
    fn json_has_source_file_rule(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("source_file"), "Missing source_file rule");
    }

    /// 47. Enum root type appears in JSON rules.
    #[test]
    fn json_enum_root_in_rules(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("Expr"), "Missing Expr rule");
    }

    /// 48. Struct root type appears in JSON rules.
    #[test]
    fn json_struct_root_in_rules(name in grammar_name()) {
        let g = extract_one(&src_struct(&name));
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("Root"), "Missing Root rule");
    }

    /// 49. Enum with N variants has CHOICE type for root rule.
    #[test]
    fn json_enum_has_choice(name in grammar_name(), n in 2..=4usize) {
        let g = extract_one(&src_enum_n(&name, n));
        let root_rule = &g["rules"]["Root"];
        prop_assert_eq!(root_rule["type"].as_str().unwrap(), "CHOICE");
        let members = root_rule["members"].as_array().unwrap();
        prop_assert!(members.len() >= n, "Expected at least {} CHOICE members", n);
    }

    /// 50. Text leaf grammar has string literal in rules.
    #[test]
    fn json_text_leaf_has_string(name in grammar_name()) {
        let g = extract_one(&src_text_leaves(&name));
        let json_str = serde_json::to_string(&g).unwrap();
        // The "+" and "-" text tokens should appear somewhere in the JSON
        prop_assert!(json_str.contains("+") || json_str.contains("-"),
            "Expected text literals in generated JSON");
    }
}

// ===========================================================================
// 11. Extras/whitespace handling in generated JSON (tests 51-55)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 51. Grammar with extras has non-empty extras array.
    #[test]
    fn json_extras_non_empty(name in grammar_name()) {
        let g = extract_one(&src_extras(&name));
        let extras = g["extras"].as_array().unwrap();
        prop_assert!(!extras.is_empty(), "Expected non-empty extras array");
    }

    /// 52. Grammar without extras has empty extras array.
    #[test]
    fn json_no_extras_empty(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let extras = g["extras"].as_array().unwrap();
        prop_assert!(extras.is_empty(), "Expected empty extras when no extra declared");
    }

    /// 53. Extras reference points to Whitespace symbol.
    #[test]
    fn json_extras_reference_whitespace(name in grammar_name()) {
        let g = extract_one(&src_extras(&name));
        let extras = g["extras"].as_array().unwrap();
        let has_ws = extras.iter().any(|e| {
            e["type"].as_str() == Some("SYMBOL")
                && e["name"].as_str() == Some("Whitespace")
        });
        prop_assert!(has_ws, "Expected Whitespace SYMBOL in extras");
    }

    /// 54. Vec grammar with extras produces non-empty extras.
    #[test]
    fn json_vec_extras_present(name in grammar_name()) {
        let g = extract_one(&src_vec(&name));
        let extras = g["extras"].as_array().unwrap();
        prop_assert!(!extras.is_empty(), "Expected extras in vec grammar");
    }

    /// 55. Extras entries have SYMBOL type.
    #[test]
    fn json_extras_are_symbols(name in grammar_name()) {
        let g = extract_one(&src_extras(&name));
        let extras = g["extras"].as_array().unwrap();
        for e in extras {
            prop_assert_eq!(e["type"].as_str().unwrap(), "SYMBOL",
                "Extra entries should be SYMBOL references");
        }
    }
}

// ===========================================================================
// 12. Precedence rules in generated JSON (tests 56-60)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 56. Prec-left grammar JSON contains PREC_LEFT type.
    #[test]
    fn json_prec_left_type(name in grammar_name()) {
        let g = extract_one(&src_prec_left(&name));
        let json_str = serde_json::to_string(&g).unwrap();
        prop_assert!(json_str.contains("PREC_LEFT"),
            "Expected PREC_LEFT in grammar JSON");
    }

    /// 57. Prec-left grammar has precedence value in JSON.
    #[test]
    fn json_prec_left_value(name in grammar_name()) {
        let g = extract_one(&src_prec_left(&name));
        let json_str = serde_json::to_string(&g).unwrap();
        // prec_left(1) should include a value
        prop_assert!(json_str.contains("PREC_LEFT"), "Missing PREC_LEFT");
        // The value 1 should appear in context of the precedence
        prop_assert!(json_str.contains(r#""value":1"#) || json_str.contains(r#""value": 1"#),
            "Expected precedence value 1 in JSON");
    }

    /// 58. Non-precedence grammar JSON has no PREC_LEFT.
    #[test]
    fn json_no_prec_without_annotation(name in grammar_name()) {
        let g = extract_one(&src_enum(&name));
        let json_str = serde_json::to_string(&g).unwrap();
        prop_assert!(!json_str.contains("PREC_LEFT"),
            "Unexpected PREC_LEFT in simple enum grammar");
    }

    /// 59. Prec-left grammar root rule is still CHOICE type.
    #[test]
    fn json_prec_left_root_is_choice(name in grammar_name()) {
        let g = extract_one(&src_prec_left(&name));
        let root_rule = &g["rules"]["Expr"];
        prop_assert_eq!(root_rule["type"].as_str().unwrap(), "CHOICE",
            "Expected root of prec grammar to be CHOICE");
    }

    /// 60. Prec grammar produces more rules than simple grammar.
    #[test]
    fn json_prec_more_rules(name in grammar_name()) {
        let simple = extract_one(&src_enum(&name));
        let prec = extract_one(&src_prec_left(&name));
        let simple_count = simple["rules"].as_object().unwrap().len();
        let prec_count = prec["rules"].as_object().unwrap().len();
        prop_assert!(prec_count >= simple_count,
            "Prec grammar should have at least as many rules as simple grammar");
    }
}

// ===========================================================================
// 13. Source file generation (tests 61-64)
// ===========================================================================

/// 61. source_file rule references the root type.
#[test]
fn json_source_file_references_root_enum() {
    let g = extract_one(&src_enum("srcref"));
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Expr");
}

/// 62. Struct grammar source_file references struct root type.
#[test]
fn json_source_file_references_root_struct() {
    let g = extract_one(&src_struct("srcstruct"));
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Root");
}

/// 63. source_file is always the first key in rules.
#[test]
fn json_source_file_is_first_rule() {
    let g = extract_one(&src_enum("firstkey"));
    let rules = g["rules"].as_object().unwrap();
    let first_key = rules.keys().next().unwrap();
    assert_eq!(first_key, "source_file");
}

/// 64. Recursive grammar source_file also references root.
#[test]
fn json_source_file_references_root_recursive() {
    let g = extract_one(&src_recursive("recsrc"));
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Expr");
}

// ===========================================================================
// 14. Build pipeline determinism (tests 65-67)
// ===========================================================================

/// 65. JSON generation is deterministic for enum grammars.
#[test]
fn json_deterministic_enum() {
    let g1 = extract_one(&src_enum("det_enum"));
    let g2 = extract_one(&src_enum("det_enum"));
    assert_eq!(g1, g2);
}

/// 66. JSON generation is deterministic for struct grammars.
#[test]
fn json_deterministic_struct() {
    let g1 = extract_one(&src_struct("det_struct"));
    let g2 = extract_one(&src_struct("det_struct"));
    assert_eq!(g1, g2);
}

/// 67. Full pipeline (JSON → C) is deterministic end-to-end.
#[test]
fn full_pipeline_deterministic() {
    let src = src_prec_left("det_pipe");
    let (n1, c1) = gen_c(&src);
    let (n2, c2) = gen_c(&src);
    assert_eq!(n1, n2);
    assert_eq!(c1, c2);
}

// ===========================================================================
// 15. Multiple modules produce separate grammars (tests 68-70)
// ===========================================================================

/// 68. Two modules produce two grammar JSONs.
#[test]
fn multi_module_count() {
    let src = src_multi("multi", 2);
    let (dir, path) = write_temp(&src);
    let gs = generate_grammars(&path).unwrap();
    drop(dir);
    assert_eq!(gs.len(), 2);
}

/// 69. Three modules produce three grammars with distinct names.
#[test]
fn multi_module_distinct_names() {
    let src = src_multi("uniq", 3);
    let (dir, path) = write_temp(&src);
    let gs = generate_grammars(&path).unwrap();
    drop(dir);
    let names: Vec<_> = gs
        .iter()
        .map(|g| g["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names.len(), 3);
    let unique: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(unique.len(), 3, "Grammar names should be distinct");
}

/// 70. Each grammar from multi-module has its own rules object.
#[test]
fn multi_module_separate_rules() {
    let src = src_multi("sep", 2);
    let (dir, path) = write_temp(&src);
    let gs = generate_grammars(&path).unwrap();
    drop(dir);
    for g in &gs {
        assert!(g["rules"].is_object());
        assert!(g["rules"].as_object().unwrap().contains_key("source_file"));
    }
}
