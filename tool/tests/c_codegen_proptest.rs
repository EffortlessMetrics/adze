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
use std::path::Path;
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
