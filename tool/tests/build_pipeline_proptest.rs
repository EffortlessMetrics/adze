//! Property-based tests for the adze-tool build pipeline.
//!
//! Tests exercise `generate_grammars` with dynamically generated Rust source
//! files to verify pipeline properties like correctness, determinism, and
//! error handling.

#![allow(clippy::needless_range_loop)]

use adze_tool::generate_grammars;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII wrapper for a temporary `.rs` file deleted on drop.
struct TempFile(PathBuf);

impl TempFile {
    fn new(source: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("adze_bpp_{}_{}.rs", std::process::id(), id));
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

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
}

fn variant_count() -> impl Strategy<Value = usize> {
    1..=5usize
}

fn field_count() -> impl Strategy<Value = usize> {
    1..=5usize
}

fn nesting_depth() -> impl Strategy<Value = usize> {
    1..=4usize
}

fn grammar_count() -> impl Strategy<Value = usize> {
    2..=4usize
}

// ---------------------------------------------------------------------------
// Source generators
// ---------------------------------------------------------------------------

/// Simple enum grammar with one Number variant.
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

/// Enum grammar with N variants.
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

/// Simple struct grammar with one field.
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

/// Struct grammar with N leaf fields.
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

/// Grammar with whitespace extra.
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

/// Grammar inside N levels of nested modules.
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

/// Multiple grammar modules in one file.
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

/// Recursive enum with Box.
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

/// Enum with prec_left.
fn src_prec(name: &str) -> String {
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

/// Struct with Optional field.
fn _src_optional(name: &str) -> String {
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

/// Struct with Vec field.
fn _src_vec(name: &str) -> String {
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

/// One top-level grammar + one nested grammar.
fn src_mixed(prefix: &str) -> String {
    let n1 = format!("{prefix}_top");
    let n2 = format!("{prefix}_inner");
    format!(
        r#"
#[adze::grammar("{n1}")]
mod top_grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}
}}

mod wrapper {{
    #[adze::grammar("{n2}")]
    mod inner_grammar {{
        #[adze::language]
        pub enum Token {{
            Word(
                #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
                String
            ),
        }}
    }}
}}
"#
    )
}

/// Enum with text-literal leaves.
fn _src_text_leaves(name: &str) -> String {
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

// ===========================================================================
// Tests: Pipeline processes input files (1–5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// 1. Simple enum grammar produces exactly one output.
    #[test]
    fn pipeline_processes_simple_enum(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }

    /// 2. Simple struct grammar produces exactly one output.
    #[test]
    fn pipeline_processes_simple_struct(name in grammar_name()) {
        let f = TempFile::new(&src_struct(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }

    /// 3. Enum grammar works with varying variant counts.
    #[test]
    fn pipeline_processes_varying_variants(name in grammar_name(), n in variant_count()) {
        let f = TempFile::new(&src_enum_n(&name, n));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }

    /// 4. Struct grammar works with varying field counts.
    #[test]
    fn pipeline_processes_varying_fields(name in grammar_name(), n in field_count()) {
        let f = TempFile::new(&src_struct_n(&name, n));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }

    /// 5. Recursive enum grammar (Box) processes successfully.
    #[test]
    fn pipeline_processes_recursive_enum(name in grammar_name()) {
        let f = TempFile::new(&src_recursive(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }
}

// ===========================================================================
// Tests: Output contains grammar JSON (6–12)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// 6. Output JSON has a "name" field.
    #[test]
    fn output_has_name_field(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out[0].get("name").is_some());
    }

    /// 7. Output JSON has a "rules" field.
    #[test]
    fn output_has_rules_field(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out[0].get("rules").is_some());
    }

    /// 8. Output JSON has an "extras" key when extras defined.
    #[test]
    fn output_has_extras_key(name in grammar_name()) {
        let f = TempFile::new(&src_extras(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out[0].get("extras").is_some());
    }

    /// 9. "name" in the output matches the annotation string.
    #[test]
    fn output_name_matches_annotation(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }

    /// 10. Rules map always contains "source_file".
    #[test]
    fn output_rules_contain_source_file(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("source_file"));
    }

    /// 11. Rules map contains the root type name.
    #[test]
    fn output_rules_contain_root_type(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key("Expr"),
            "rules: {:?}",
            rules.keys().collect::<Vec<_>>()
        );
    }

    /// 12. Rules map has at least 2 entries (source_file + root type).
    #[test]
    fn output_rules_min_count(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        prop_assert!(rules.len() >= 2, "expected >= 2 rules, got {}", rules.len());
    }
}

// ===========================================================================
// Tests: Output determinism (13–16)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 13. Same enum input produces identical output.
    #[test]
    fn deterministic_enum(name in grammar_name()) {
        let src = src_enum(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = generate_grammars(f1.path()).unwrap();
        let o2 = generate_grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }

    /// 14. Same struct input produces identical output.
    #[test]
    fn deterministic_struct(name in grammar_name()) {
        let src = src_struct(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = generate_grammars(f1.path()).unwrap();
        let o2 = generate_grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }

    /// 15. Same extras input produces identical output.
    #[test]
    fn deterministic_extras(name in grammar_name()) {
        let src = src_extras(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = generate_grammars(f1.path()).unwrap();
        let o2 = generate_grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }

    /// 16. Same prec_left input produces identical output.
    #[test]
    fn deterministic_prec(name in grammar_name()) {
        let src = src_prec(&name);
        let f1 = TempFile::new(&src);
        let f2 = TempFile::new(&src);
        let o1 = generate_grammars(f1.path()).unwrap();
        let o2 = generate_grammars(f2.path()).unwrap();
        prop_assert_eq!(o1, o2);
    }
}

// ===========================================================================
// Tests: Multiple grammars (17–19)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 17. N grammar modules produce N output values.
    #[test]
    fn multiple_grammars_correct_count(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), n);
    }

    /// 18. Multiple grammars have distinct names.
    #[test]
    fn multiple_grammars_distinct_names(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = generate_grammars(f.path()).unwrap();
        let names: Vec<&str> = out.iter().map(|g| g["name"].as_str().unwrap()).collect();
        let unique: HashSet<&str> = names.iter().copied().collect();
        prop_assert_eq!(names.len(), unique.len());
    }

    /// 19. Every grammar in a multi-grammar file has rules.
    #[test]
    fn multiple_grammars_all_have_rules(prefix in grammar_name(), n in grammar_count()) {
        let f = TempFile::new(&src_multi(&prefix, n));
        let out = generate_grammars(f.path()).unwrap();
        for (i, g) in out.iter().enumerate() {
            prop_assert!(
                g.get("rules").and_then(Value::as_object).is_some(),
                "grammar {} missing rules",
                i
            );
        }
    }
}

// ===========================================================================
// Tests: Error handling / no annotations (20–23)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 20. File with no adze annotations returns empty vec.
    #[test]
    fn no_annotations_returns_empty(name in grammar_name()) {
        let src = format!("mod {name} {{ pub struct Foo {{ x: u32 }} }}");
        let f = TempFile::new(&src);
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }

    /// 21. Empty file returns empty vec.
    #[test]
    fn empty_file_returns_empty(_x in 0..5u8) {
        let f = TempFile::new("");
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }

    /// 22. Comment-only file returns empty vec.
    #[test]
    fn comment_only_returns_empty(n in 1..5usize) {
        let src: String = (0..n).map(|i| format!("// comment {i}\n")).collect();
        let f = TempFile::new(&src);
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }

    /// 23. Plain module without grammar attribute returns empty.
    #[test]
    fn plain_module_returns_empty(name in grammar_name()) {
        let src = format!("mod {name} {{ pub enum E {{ A, B }} }}");
        let f = TempFile::new(&src);
        let out = generate_grammars(f.path()).unwrap();
        prop_assert!(out.is_empty());
    }
}

// ===========================================================================
// Tests: Nested modules (24–27)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 24. Grammar inside a nested module is found.
    #[test]
    fn nested_module_finds_grammar(name in grammar_name()) {
        let f = TempFile::new(&src_nested(&name, 1));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }

    /// 25. Grammar inside deeply nested modules is found.
    #[test]
    fn deeply_nested_finds_grammar(name in grammar_name(), d in nesting_depth()) {
        let f = TempFile::new(&src_nested(&name, d));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 1);
    }

    /// 26. Deeply nested grammar's name matches annotation.
    #[test]
    fn nested_name_matches(name in grammar_name(), d in nesting_depth()) {
        let f = TempFile::new(&src_nested(&name, d));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out[0]["name"].as_str().unwrap(), name.as_str());
    }

    /// 27. Top-level and nested grammars both found.
    #[test]
    fn nested_and_toplevel_both_found(prefix in grammar_name()) {
        let f = TempFile::new(&src_mixed(&prefix));
        let out = generate_grammars(f.path()).unwrap();
        prop_assert_eq!(out.len(), 2);
    }
}

// ===========================================================================
// Tests: Grammar features (28–30)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 28. Enum root type generates a CHOICE rule.
    #[test]
    fn enum_choice_type(name in grammar_name(), n in 2..=5usize) {
        let f = TempFile::new(&src_enum_n(&name, n));
        let out = generate_grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        let root_rule = &rules["Root"];
        prop_assert_eq!(
            root_rule["type"].as_str().unwrap(),
            "CHOICE",
            "root rule type: {:?}",
            root_rule
        );
    }

    /// 29. CHOICE members count equals variant count.
    #[test]
    fn enum_choice_members_match_variant_count(name in grammar_name(), n in 1..=5usize) {
        let f = TempFile::new(&src_enum_n(&name, n));
        let out = generate_grammars(f.path()).unwrap();
        let rules = out[0]["rules"].as_object().unwrap();
        let root = &rules["Root"];
        let members = root["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), n);
    }

    /// 30. Output is serializable to a JSON string and back.
    #[test]
    fn output_json_roundtrip(name in grammar_name()) {
        let f = TempFile::new(&src_enum(&name));
        let out = generate_grammars(f.path()).unwrap();
        let json_str = serde_json::to_string(&out[0]).unwrap();
        let back: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&out[0], &back);
    }
}
