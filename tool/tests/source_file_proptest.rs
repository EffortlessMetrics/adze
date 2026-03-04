#![allow(clippy::needless_range_loop)]

//! Property-based tests for source file processing in adze-tool.
//!
//! Exercises `generate_grammars` with dynamically generated Rust source files
//! to verify source file processing properties: single/multiple source files,
//! no/one/many grammars, path handling, determinism, and error handling.

use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// ===========================================================================
// Helpers
// ===========================================================================

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII wrapper for a temporary `.rs` file deleted on drop.
struct TempFile(PathBuf);

impl TempFile {
    fn new(source: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("adze_sfp_{}_{}.rs", std::process::id(), id));
        std::fs::write(&path, source).expect("failed to write temp file");
        TempFile(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn grammars(path: &Path) -> adze_tool::ToolResult<Vec<Value>> {
    adze_tool::generate_grammars(path)
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
}

fn grammar_count() -> impl Strategy<Value = usize> {
    2..=5usize
}

fn variant_count() -> impl Strategy<Value = usize> {
    1..=4usize
}

fn field_count() -> impl Strategy<Value = usize> {
    1..=4usize
}

// ===========================================================================
// Source generators
// ===========================================================================

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

fn src_enum_n_variants(name: &str, n: usize) -> String {
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

fn src_struct_n_fields(name: &str, n: usize) -> String {
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

fn src_with_extras(name: &str) -> String {
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

fn src_nested(name: &str, depth: usize) -> String {
    let open: String = (0..depth).map(|i| format!("mod n{i} {{\n")).collect();
    let close: String = (0..depth).map(|_| "}\n".to_string()).collect();
    format!(
        r#"
{open}
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
{close}
"#
    )
}

fn src_no_grammar() -> String {
    r#"
pub struct Foo {
    pub x: u32,
}

pub fn bar() -> bool {
    true
}
"#
    .to_string()
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

fn rule_names(g: &Value) -> Vec<String> {
    g["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

// ===========================================================================
// 1. Process single source file — enum grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn single_source_enum_extracts_one_grammar(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }
}

// ===========================================================================
// 2. Process single source file — struct grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn single_source_struct_extracts_one_grammar(name in grammar_name()) {
        let f = TempFile::new(&src_struct(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }
}

// ===========================================================================
// 3. Process single source file — name matches annotation
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn single_source_name_matches(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 4. Process single source file — rules contain source_file
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn single_source_has_source_file_rule(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("source_file"));
    }
}

// ===========================================================================
// 5. Process multiple source files independently
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_independent_source_files(
        name_a in grammar_name(),
        name_b in grammar_name(),
    ) {
        let fa = TempFile::new(&src_enum(&name_a));
        let fb = TempFile::new(&src_struct(&name_b));
        let oa = grammars(fa.path()).unwrap();
        let ob = grammars(fb.path()).unwrap();
        prop_assert_eq!(oa.len(), 1);
        prop_assert_eq!(ob.len(), 1);
        prop_assert_eq!(oa[0]["name"].as_str().unwrap(), name_a.as_str());
        prop_assert_eq!(ob[0]["name"].as_str().unwrap(), name_b.as_str());
    }
}

// ===========================================================================
// 6. Multiple source files produce independent results
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn multiple_source_files_no_interference(name in grammar_name()) {
        let f1 = TempFile::new(&src_enum(&name));
        let f2 = TempFile::new(&src_no_grammar());
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1.len(), 1);
        prop_assert!(o2.is_empty());
    }
}

// ===========================================================================
// 7. Source file with no grammars — empty file
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn no_grammar_empty_file(_x in 0..5u8) {
        let f = TempFile::new("");
        let out = grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }
}

// ===========================================================================
// 8. Source file with no grammars — plain Rust code
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn no_grammar_plain_rust(_x in 0..5u8) {
        let f = TempFile::new(&src_no_grammar());
        let out = grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }
}

// ===========================================================================
// 9. Source file with no grammars — comments only
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn no_grammar_comments_only(n in 1..6usize) {
        let src: String = (0..n).map(|i| format!("// comment line {i}\n")).collect();
        let f = TempFile::new(&src);
        let out = grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }
}

// ===========================================================================
// 10. Source file with no grammars — module without grammar attr
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn no_grammar_plain_module(name in grammar_name()) {
        let src = format!("mod {name} {{ pub struct S {{ pub x: u32 }} }}");
        let f = TempFile::new(&src);
        let out = grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }
}

// ===========================================================================
// 11. Source file with one grammar — extras present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn one_grammar_with_extras(name in grammar_name()) {
        let f = TempFile::new(&src_with_extras(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        prop_assert!(out[0].get("extras").is_some());
        prop_assert!(out[0]["extras"].is_array());
    }
}

// ===========================================================================
// 12. Source file with one grammar — recursive enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn one_grammar_recursive_enum(name in grammar_name()) {
        let f = TempFile::new(&src_recursive(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        let names = rule_names(&out[0]);
        prop_assert!(names.contains(&"Expr".to_string()));
    }
}

// ===========================================================================
// 13. Source file with one grammar — varying variant count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn one_grammar_varying_variants(name in grammar_name(), n in variant_count()) {
        let f = TempFile::new(&src_enum_n_variants(&name, n));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        let root = &out[0]["rules"]["Root"];
        let members = root["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), n);
    }
}

// ===========================================================================
// 14. Source file with one grammar — varying field count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn one_grammar_varying_fields(name in grammar_name(), n in field_count()) {
        let f = TempFile::new(&src_struct_n_fields(&name, n));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        let names = rule_names(&out[0]);
        prop_assert!(names.contains(&"Root".to_string()));
    }
}

// ===========================================================================
// 15. Source file with multiple grammars — correct count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_grammars_correct_count(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), n);
    }
}

// ===========================================================================
// 16. Source file with multiple grammars — distinct names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_grammars_distinct_names(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = grammars(f.path()).unwrap();
        let names: HashSet<&str> = out.iter()
            .map(|g| g["name"].as_str().unwrap())
            .collect();
        prop_assert_eq!(names.len(), n);
    }
}

// ===========================================================================
// 17. Source file with multiple grammars — all have rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_grammars_all_have_rules(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = grammars(f.path()).unwrap();
        for (i, g) in out.iter().enumerate() {
            prop_assert!(
                g["rules"].is_object(),
                "grammar {} missing rules object", i
            );
        }
    }
}

// ===========================================================================
// 18. Source file with multiple grammars — all have source_file rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_grammars_all_have_source_file(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = grammars(f.path()).unwrap();
        for (i, g) in out.iter().enumerate() {
            let rules = g["rules"].as_object().unwrap();
            prop_assert!(
                rules.contains_key("source_file"),
                "grammar {} missing source_file rule", i
            );
        }
    }
}

// ===========================================================================
// 19. Source file path handling — absolute path works
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn path_absolute_works(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let abs = f.path().canonicalize().unwrap();
        let out = grammars(&abs).unwrap();
        prop_assert_eq!(out.len(), 1);
    }
}

// ===========================================================================
// 20. Source file path handling — different filenames yield same grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn path_different_filenames_same_grammar(name in grammar_name()) {
        let src = src_enum(&name);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let p1 = dir.join(format!("adze_sfp_fn1_{}_{}.rs", std::process::id(), id));
        let p2 = dir.join(format!("adze_sfp_fn2_{}_{}.rs", std::process::id(), id));
        std::fs::write(&p1, &src).unwrap();
        std::fs::write(&p2, &src).unwrap();
        let o1 = grammars(&p1).unwrap();
        let o2 = grammars(&p2).unwrap();
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 21. Source file path handling — nested grammar inside module path
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn path_nested_module_grammar(name in grammar_name()) {
        let f = TempFile::new(&src_nested(&name, 2));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 22. Source file determinism — enum grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn determinism_enum(name in grammar_name()) {
        let src = src_enum(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 23. Source file determinism — struct grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn determinism_struct(name in grammar_name()) {
        let src = src_struct(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 24. Source file determinism — multiple grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn determinism_multiple_grammars(prefix in grammar_name(), n in grammar_count()) {
        let src = src_multi(&prefix, n);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 25. Source file determinism — extras grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn determinism_extras(name in grammar_name()) {
        let src = src_with_extras(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 26. Source file determinism — JSON roundtrip stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn determinism_json_roundtrip(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        let json_str = serde_json::to_string(&out[0]).unwrap();
        let back: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&out[0], &back);
    }
}

// ===========================================================================
// 27. Source file error handling — nonexistent path
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn error_nonexistent_path(name in grammar_name()) {
        let path = std::env::temp_dir().join(format!("adze_sfp_noexist_{name}.rs"));
        let _ = std::fs::remove_file(&path); // ensure absent
        let result = std::panic::catch_unwind(|| grammars(&path));
        // syn_inline_mod::parse_and_inline_modules panics or returns error
        prop_assert!(result.is_err() || result.unwrap().is_err());
    }
}

// ===========================================================================
// 28. Source file error handling — directory instead of file
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn error_directory_path(_x in 0..3u8) {
        let dir = tempfile::TempDir::new().unwrap();
        let result = std::panic::catch_unwind(|| grammars(dir.path()));
        prop_assert!(result.is_err() || result.unwrap().is_err());
    }
}

// ===========================================================================
// 29. Source file error handling — non-Rust content gracefully handled
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn error_non_rust_content(_x in 0..5u8) {
        let f = TempFile::new("This is not valid Rust code at all {{{{");
        let result = std::panic::catch_unwind(|| grammars(f.path()));
        // Should either panic (syn parse failure) or return error
        prop_assert!(result.is_err() || result.unwrap().is_err());
    }
}

// ===========================================================================
// 30. Source file with mixed grammar and non-grammar modules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn mixed_grammar_and_plain_modules(name in grammar_name()) {
        let src = format!(
            r#"
mod plain {{
    pub struct Foo {{ pub x: u32 }}
}}

{grammar}

mod another_plain {{
    pub fn bar() -> bool {{ true }}
}}
"#,
            grammar = src_enum(&name)
        );
        let f = TempFile::new(&src);
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 31. Source file output structure — word key present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn output_has_word_key(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        prop_assert!(out[0].get("word").is_some(), "'word' key must be present");
    }
}

// ===========================================================================
// 32. Source file output structure — source_file is first rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn output_source_file_is_first_rule(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        let first_key = out[0]["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first_key, "source_file");
    }
}

// ===========================================================================
// 33. Source file output structure — at least 2 rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn output_at_least_two_rules(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = grammars(f.path()).unwrap();
        let count = out[0]["rules"].as_object().unwrap().len();
        prop_assert!(count >= 2, "expected >= 2 rules, got {}", count);
    }
}

// ===========================================================================
// 34. Source file determinism — recursive grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn determinism_recursive(name in grammar_name()) {
        let src = src_recursive(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = grammars(f1.path()).unwrap();
        let o2 = grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// 35. Source file with deeply nested module — grammar found
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn deeply_nested_module_grammar(name in grammar_name(), depth in 1..=4usize) {
        let f = TempFile::new(&src_nested(&name, depth));
        let out = grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }
}
