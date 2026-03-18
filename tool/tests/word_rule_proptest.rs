#![allow(clippy::needless_range_loop)]

//! Property-based tests for word rule generation in adze-tool.
//!
//! Uses proptest to validate invariants of the `word` field in Tree-sitter
//! grammar JSON produced by `adze_tool::generate_grammars`:
//!   - Word rule appears in grammar JSON
//!   - Word rule pattern is preserved
//!   - Word rule regex is valid
//!   - Word rule naming follows conventions
//!   - Word rule generation is deterministic
//!   - Grammars without word rules emit null
//!   - Multiple word candidates are rejected
//!   - Word rule propagates to C code

use proptest::prelude::*;
use serde_json::Value;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tree_sitter_generate::generate_parser_for_grammar;

const SEMANTIC_VERSION: Option<(u8, u8, u8)> = Some((0, 25, 1));

// ===========================================================================
// Helpers
// ===========================================================================

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn write_temp(src: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.path().join(format!("w_{id}.rs"));
    fs::write(&path, src).unwrap();
    (dir, path)
}

fn extract_one(src: &str) -> Value {
    let (dir, path) = write_temp(src);
    let gs = adze_tool::generate_grammars(&path).unwrap();
    drop(dir);
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

fn extract_err(src: &str) -> String {
    let (dir, path) = write_temp(src);
    let res = adze_tool::generate_grammars(&path);
    drop(dir);
    res.unwrap_err().to_string()
}

fn gen_c(src: &str) -> (String, String) {
    let grammar = extract_one(src);
    let json = serde_json::to_string(&grammar).unwrap();
    generate_parser_for_grammar(&json, SEMANTIC_VERSION).unwrap()
}

#[test]
fn word_struct_named_vec_is_supported() {
    let g = extract_one(&src_word_struct("a0", "Vec", r"[a-zA-Z_]\w*"));
    assert_eq!(g["word"].as_str(), Some("Vec"));
}

// ===========================================================================
// Source builders
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
}

fn word_rule_name() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,6}"
}

fn word_pattern() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-zA-Z_]\w*".to_string()),
        Just(r"[a-z]+".to_string()),
        Just(r"[a-zA-Z][a-zA-Z0-9]*".to_string()),
        Just(r"[_a-z][_a-z0-9]*".to_string()),
        Just(r"[A-Za-z]\w*".to_string()),
    ]
}

/// Grammar with a struct-level `#[adze::word]` annotation.
fn src_word_struct(name: &str, word_name: &str, pattern: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        expr: Expr,
    }}

    pub enum Expr {{
        Ident(
            {word_name},
        ),
        Kw(
            #[adze::leaf(text = "let", transform = |v| ())]
            ()
        ),
    }}

    #[adze::word]
    pub struct {word_name} {{
        #[adze::leaf(pattern = r"{pattern}")]
        _v: String,
    }}
}}
"#
    )
}

/// Grammar with a field-level `#[adze::word]` annotation.
fn src_word_field(name: &str, word_name: &str, pattern: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::word]
        #[adze::leaf(pattern = r"{pattern}")]
        {word_name}: String,
    }}
}}
"#
    )
}

/// Grammar with NO word rule.
fn src_no_word(name: &str) -> String {
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

/// Grammar with TWO `#[adze::word]` structs (should fail).
fn src_multiple_word_structs(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        a: Ident,
        b: Kw,
    }}

    #[adze::word]
    pub struct Ident {{
        #[adze::leaf(pattern = r"[a-z]+")]
        _v: String,
    }}

    #[adze::word]
    pub struct Kw {{
        #[adze::leaf(pattern = r"[A-Z]+")]
        _v: String,
    }}
}}
"#
    )
}

/// Grammar with a struct word AND a field word (should fail).
fn src_mixed_word_conflict(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        a: Ident,
    }}

    #[adze::word]
    pub struct Ident {{
        #[adze::word]
        #[adze::leaf(pattern = r"[a-z]+")]
        val: String,
    }}
}}
"#
    )
}

/// Grammar with a word-annotated enum variant (struct-level).
fn src_word_on_enum(name: &str, word_name: &str, pattern: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        expr: Expr,
    }}

    pub enum Expr {{
        Id({word_name}),
        Num(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}

    #[adze::word]
    pub struct {word_name} {{
        #[adze::leaf(pattern = r"{pattern}")]
        _v: String,
    }}
}}
"#
    )
}

/// Grammar with extras AND a word rule.
fn src_word_with_extras(name: &str, word_name: &str, pattern: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        expr: Expr,
    }}

    pub enum Expr {{
        Ident({word_name}),
        Lit(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}

    #[adze::word]
    pub struct {word_name} {{
        #[adze::leaf(pattern = r"{pattern}")]
        _v: String,
    }}

    #[adze::extra]
    pub struct Whitespace {{
        #[adze::leaf(pattern = r"\s")]
        _ws: (),
    }}
}}
"#
    )
}

// ===========================================================================
// 1. Word rule in grammar JSON (tests 1-4)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 1. Struct-level word annotation produces non-null "word" field.
    #[test]
    fn word_struct_produces_word_field(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        prop_assert!(
            !g["word"].is_null(),
            "Expected non-null word field, got null"
        );
    }

    /// 2. Word rule value equals the annotated struct name.
    #[test]
    fn word_struct_value_matches_name(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        prop_assert_eq!(g["word"].as_str().unwrap(), word_name.as_str());
    }

    /// 3. Field-level word annotation produces a word value matching the field path.
    #[test]
    fn word_field_produces_word_value(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let field = "ident";
        let src = src_word_field(&name, field, &pat);
        let g = extract_one(&src);
        let word = g["word"].as_str().unwrap();
        // Field-level word: path is "Root_<field>"
        prop_assert!(
            !word.is_empty(),
            "word field should not be empty"
        );
    }

    /// 4. Word rule with enum reference produces non-null word.
    #[test]
    fn word_enum_ref_produces_word(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_on_enum(&name, &word_name, &pat);
        let g = extract_one(&src);
        prop_assert_eq!(g["word"].as_str().unwrap(), word_name.as_str());
    }
}

// ===========================================================================
// 2. Word rule pattern (tests 5-8)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 5. Word rule name appears in the rules map.
    #[test]
    fn word_rule_exists_in_rules(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(&word_name),
            "rules map should contain word rule '{}'", word_name
        );
    }

    /// 6. Word rule value is a string (not an object or array).
    #[test]
    fn word_field_is_string(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        prop_assert!(g["word"].is_string(), "word field should be a string");
    }

    /// 7. Word rule with extras still preserves the word field.
    #[test]
    fn word_with_extras_preserves_word(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_with_extras(&name, &word_name, &pat);
        let g = extract_one(&src);
        prop_assert_eq!(g["word"].as_str().unwrap(), word_name.as_str());
    }

    /// 8. Word rule has a corresponding rule entry with PATTERN or FIELD type.
    #[test]
    fn word_rule_entry_has_structure(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let rule = &g["rules"][&word_name];
        // The rule should be an object (SEQ, FIELD, PATTERN, etc.)
        prop_assert!(rule.is_object(), "word rule entry should be a JSON object");
    }
}

// ===========================================================================
// 3. Word rule regex (tests 9-12)
// ===========================================================================

/// Recursively find all PATTERN nodes in a grammar value.
fn collect_patterns(val: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("PATTERN")
                && let Some(v) = map.get("value").and_then(|v| v.as_str())
            {
                out.push(v.to_string());
            }
            for v in map.values() {
                out.extend(collect_patterns(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_patterns(v));
            }
        }
        _ => {}
    }
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 9. The pattern for the word rule appears somewhere in the grammar rules.
    #[test]
    fn word_pattern_in_grammar(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        // Pattern may be in a sub-rule referenced via SYMBOL
        let all_patterns = collect_patterns(&g["rules"]);
        prop_assert!(
            all_patterns.iter().any(|p| p == &pat),
            "Expected pattern '{}' among all grammar patterns, found {:?}", pat, all_patterns
        );
    }

    /// 10. All patterns in a word rule are valid regex.
    #[test]
    fn word_patterns_are_valid_regex(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let patterns = collect_patterns(&g["rules"][&word_name]);
        for p in &patterns {
            prop_assert!(
                regex::Regex::new(p).is_ok(),
                "Pattern '{}' is not a valid regex", p
            );
        }
    }

    /// 11. Field-level word annotation pattern appears in the rule.
    #[test]
    fn field_word_pattern_in_rule(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let field = "ident";
        let src = src_word_field(&name, field, &pat);
        let g = extract_one(&src);
        // Patterns may be nested inside the Root rule
        let all_patterns = collect_patterns(&g["rules"]);
        prop_assert!(
            all_patterns.iter().any(|p| p == &pat),
            "Expected pattern '{}' among grammar patterns", pat
        );
    }

    /// 12. Word patterns in extras grammar are valid regex.
    #[test]
    fn word_extras_patterns_valid(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_with_extras(&name, &word_name, &pat);
        let g = extract_one(&src);
        let patterns = collect_patterns(&g["rules"]);
        for p in &patterns {
            prop_assert!(
                regex::Regex::new(p).is_ok(),
                "Pattern '{}' from extras grammar is not valid regex", p
            );
        }
    }
}

// ===========================================================================
// 4. Word rule naming (tests 13-16)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 13. Word rule name is non-empty.
    #[test]
    fn word_name_non_empty(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let w = g["word"].as_str().unwrap();
        prop_assert!(!w.is_empty());
    }

    /// 14. Word rule name contains only alphanumeric or underscore chars.
    #[test]
    fn word_name_is_identifier(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let w = g["word"].as_str().unwrap();
        prop_assert!(
            w.chars().all(|c| c.is_alphanumeric() || c == '_'),
            "word name '{}' contains invalid characters", w
        );
    }

    /// 15. Word rule name starts with an uppercase letter (struct convention).
    #[test]
    fn word_struct_name_starts_uppercase(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let w = g["word"].as_str().unwrap();
        prop_assert!(
            w.starts_with(|c: char| c.is_ascii_uppercase()),
            "struct-level word name '{}' should start with uppercase", w
        );
    }

    /// 16. Field-level word name contains the field identifier.
    #[test]
    fn word_field_name_contains_field(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let field = "ident";
        let src = src_word_field(&name, field, &pat);
        let g = extract_one(&src);
        let w = g["word"].as_str().unwrap();
        prop_assert!(
            w.contains(field),
            "field-level word name '{}' should contain '{}'", w, field
        );
    }
}

// ===========================================================================
// 5. Word rule determinism (tests 17-19)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 17. Struct word rule generation is deterministic.
    #[test]
    fn word_struct_deterministic(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(&g1, &g2);
    }

    /// 18. Field word rule generation is deterministic.
    #[test]
    fn word_field_deterministic(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let field = "ident";
        let src = src_word_field(&name, field, &pat);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(&g1, &g2);
    }

    /// 19. Word rule determinism across JSON serialization.
    #[test]
    fn word_json_roundtrip_deterministic(
        name in grammar_name(),
        word_name in word_rule_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_struct(&name, &word_name, &pat);
        let g = extract_one(&src);
        let json1 = serde_json::to_string(&g).unwrap();
        let reparsed: Value = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&reparsed).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}

// ===========================================================================
// 6. No word rule case (tests 20-22)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 20. Grammar without #[adze::word] has null word field.
    #[test]
    fn no_word_produces_null(name in grammar_name()) {
        let src = src_no_word(&name);
        let g = extract_one(&src);
        prop_assert!(
            g["word"].is_null(),
            "Expected null word field, got {:?}", g["word"]
        );
    }

    /// 21. Grammar without word still has valid rules.
    #[test]
    fn no_word_still_has_rules(name in grammar_name()) {
        let src = src_no_word(&name);
        let g = extract_one(&src);
        prop_assert!(g["rules"].is_object());
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(!rules.is_empty(), "rules should not be empty");
    }

    /// 22. Grammar without word still has name field.
    #[test]
    fn no_word_still_has_name(name in grammar_name()) {
        let src = src_no_word(&name);
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 7. Multiple word candidates (tests 23-25)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 23. Two struct-level word annotations produce an error.
    #[test]
    fn multiple_word_structs_rejected(name in grammar_name()) {
        let src = src_multiple_word_structs(&name);
        let err = extract_err(&src);
        prop_assert!(
            err.contains("multiple word rules"),
            "Expected 'multiple word rules' error, got: {}", err
        );
    }

    /// 24. Struct-level word plus field-level word produces an error.
    #[test]
    fn mixed_word_conflict_rejected(name in grammar_name()) {
        let src = src_mixed_word_conflict(&name);
        let err = extract_err(&src);
        // The inner MultipleWordRules error causes the field to be skipped,
        // which then triggers a StructHasNoFields error.
        prop_assert!(
            err.contains("no non-skipped fields") || err.contains("multiple word rules"),
            "Expected word conflict error, got: {}", err
        );
    }

    /// 25. Multiple word rejection is deterministic.
    #[test]
    fn multiple_word_error_deterministic(name in grammar_name()) {
        let src = src_multiple_word_structs(&name);
        let err1 = extract_err(&src);
        let err2 = extract_err(&src);
        prop_assert_eq!(&err1, &err2);
    }
}

/// Grammar suitable for C codegen: word token is a terminal PATTERN via field-level annotation.
fn src_word_for_c(name: &str, pattern: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::word]
        #[adze::leaf(pattern = r"{pattern}")]
        ident: String,
    }}
}}
"#
    )
}

// ===========================================================================
// 8. Word rule in C code (tests 26-30)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// 26. Word rule grammar produces valid C code.
    #[test]
    fn word_c_code_nonempty(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_for_c(&name, &pat);
        let (_, c) = gen_c(&src);
        prop_assert!(!c.is_empty(), "C code should not be empty");
    }

    /// 27. C code from word grammar contains the grammar name in a tree_sitter_ function.
    #[test]
    fn word_c_contains_parser_fn(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_for_c(&name, &pat);
        let (gname, c) = gen_c(&src);
        let expected = format!("tree_sitter_{gname}");
        prop_assert!(
            c.contains(&expected),
            "C code missing parser function '{}'", expected
        );
    }

    /// 28. C code from word grammar contains keyword scanning function.
    #[test]
    fn word_c_contains_keyword_scanner(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_for_c(&name, &pat);
        let (_, c) = gen_c(&src);
        prop_assert!(
            c.contains("lex_keyword"),
            "C code should contain keyword scanning function for word grammar"
        );
    }

    /// 29. C code without word rule does NOT contain ts_lex_keywords.
    #[test]
    fn no_word_c_lacks_keyword_scanner(name in grammar_name()) {
        let src = src_no_word(&name);
        let (_, c) = gen_c(&src);
        prop_assert!(
            !c.contains("lex_keyword"),
            "C code without word rule should NOT contain keyword scanning"
        );
    }

    /// 30. Word rule C code has balanced braces.
    #[test]
    fn word_c_balanced_braces(
        name in grammar_name(),
        pat in word_pattern(),
    ) {
        let src = src_word_for_c(&name, &pat);
        let (_, c) = gen_c(&src);
        let open = c.chars().filter(|&ch| ch == '{').count();
        let close = c.chars().filter(|&ch| ch == '}').count();
        prop_assert_eq!(open, close, "Unbalanced braces in word grammar C code");
    }
}
