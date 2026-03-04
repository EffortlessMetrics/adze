#![allow(clippy::needless_range_loop)]

//! Property-based tests for `source_file` rule generation in adze-tool grammar JSON.
//!
//! Validates that `generate_grammars` always produces a well-formed `source_file`
//! entry: present, first key, SYMBOL type, references the `#[adze::language]` root,
//! deterministic, and correctly named.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use proptest::prelude::*;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

static CTR: AtomicU64 = AtomicU64::new(0);

/// RAII temp `.rs` file — deleted on drop.
struct Tmp(PathBuf);

impl Tmp {
    fn new(src: &str) -> Self {
        let id = CTR.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("adze_sfgen_{}_{}.rs", std::process::id(), id));
        std::fs::write(&p, src).expect("write tmp");
        Tmp(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn extract(path: &Path) -> Vec<Value> {
    adze_tool::generate_grammars(path).expect("generate_grammars failed")
}

fn sf(g: &Value) -> &Value {
    &g["rules"]["source_file"]
}

// ---------------------------------------------------------------------------
// Source generators
// ---------------------------------------------------------------------------

fn src_struct_root(gname: &str, root: &str) -> String {
    format!(
        r#"
#[adze::grammar("{gname}")]
mod grammar {{
    #[adze::language]
    pub struct {root} {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        value: u32,
    }}
}}
"#
    )
}

fn src_enum_root(gname: &str, root: &str) -> String {
    format!(
        r#"
#[adze::grammar("{gname}")]
mod grammar {{
    #[adze::language]
    pub enum {root} {{
        Lit(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}
}}
"#
    )
}

fn src_struct_with_child(gname: &str, root: &str, child: &str) -> String {
    format!(
        r#"
#[adze::grammar("{gname}")]
mod grammar {{
    #[adze::language]
    pub struct {root} {{
        pub child: {child},
    }}
    pub struct {child} {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: u32,
    }}
}}
"#
    )
}

fn src_enum_multi_variant(gname: &str, root: &str, n: usize) -> String {
    let variants: String = (0..n)
        .map(|i| {
            format!(
                "        V{i}(#[adze::leaf(pattern = r\"\\d+\", transform = |v| v.parse().unwrap())] u32)"
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"
#[adze::grammar("{gname}")]
mod grammar {{
    #[adze::language]
    pub enum {root} {{
{variants}
    }}
}}
"#
    )
}

fn src_with_extras(gname: &str, root: &str) -> String {
    format!(
        r#"
#[adze::grammar("{gname}")]
mod grammar {{
    #[adze::language]
    pub struct {root} {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: u32,
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

fn src_multi_grammar(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|i| {
            format!(
                r#"
#[adze::grammar("{prefix}_{i}")]
mod m{i} {{
    #[adze::language]
    pub struct Root {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: u32,
    }}
}}
"#
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn gname() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,8}"
}

/// PascalCase identifiers for root type names.
fn pascal() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-z]{2,8}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

// ===========================================================================
// 1. source_file rule always present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 1a — struct root always has source_file
    #[test]
    fn sf_always_present_struct(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        prop_assert!(gs[0]["rules"].as_object().unwrap().contains_key("source_file"));
    }

    /// 1b — enum root always has source_file
    #[test]
    fn sf_always_present_enum(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        prop_assert!(gs[0]["rules"].as_object().unwrap().contains_key("source_file"));
    }

    /// 1c — grammar with extras still has source_file
    #[test]
    fn sf_always_present_with_extras(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_with_extras(&name, &root));
        let gs = extract(t.path());
        prop_assert!(gs[0]["rules"].as_object().unwrap().contains_key("source_file"));
    }
}

// ===========================================================================
// 2. source_file references root type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 2a — struct root: source_file name == root ident
    #[test]
    fn sf_references_struct_root(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["name"].as_str().unwrap(), root.as_str());
    }

    /// 2b — enum root: source_file name == root ident
    #[test]
    fn sf_references_enum_root(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["name"].as_str().unwrap(), root.as_str());
    }

    /// 2c — struct with child: source_file still points to root, not child
    #[test]
    fn sf_references_root_not_child(name in gname()) {
        let t = Tmp::new(&src_struct_with_child(&name, "Parent", "Child"));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["name"].as_str().unwrap(), "Parent");
    }
}

// ===========================================================================
// 3. source_file is SYMBOL type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 3a — struct root → SYMBOL type
    #[test]
    fn sf_type_is_symbol_struct(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["type"].as_str().unwrap(), "SYMBOL");
    }

    /// 3b — enum root → SYMBOL type
    #[test]
    fn sf_type_is_symbol_enum(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["type"].as_str().unwrap(), "SYMBOL");
    }

    /// 3c — with extras → still SYMBOL (not FIELD, SEQ, etc.)
    #[test]
    fn sf_type_is_symbol_with_extras(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_with_extras(&name, &root));
        let gs = extract(t.path());
        let ty = sf(&gs[0])["type"].as_str().unwrap();
        prop_assert_eq!(ty, "SYMBOL");
    }

    /// 3d — source_file object has exactly two keys: "type" and "name"
    #[test]
    fn sf_has_exactly_type_and_name(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let obj = sf(&gs[0]).as_object().unwrap();
        prop_assert_eq!(obj.len(), 2);
        prop_assert!(obj.contains_key("type"));
        prop_assert!(obj.contains_key("name"));
    }
}

// ===========================================================================
// 4. source_file is first rule key
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 4a — struct root: first key is "source_file"
    #[test]
    fn sf_is_first_key_struct(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let first = gs[0]["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first, "source_file");
    }

    /// 4b — enum root: first key is "source_file"
    #[test]
    fn sf_is_first_key_enum(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        let first = gs[0]["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first, "source_file");
    }

    /// 4c — with extras: first key is still "source_file"
    #[test]
    fn sf_is_first_key_with_extras(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_with_extras(&name, &root));
        let gs = extract(t.path());
        let first = gs[0]["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first, "source_file");
    }
}

// ===========================================================================
// 5. Multiple structs — source_file picks root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 5a — root annotated struct selected over non-annotated
    #[test]
    fn sf_picks_annotated_root_over_child(name in gname()) {
        let t = Tmp::new(&src_struct_with_child(&name, "Top", "Bottom"));
        let gs = extract(t.path());
        prop_assert_eq!(sf(&gs[0])["name"].as_str().unwrap(), "Top");
    }

    /// 5b — source_file name never equals the child struct name
    #[test]
    fn sf_never_child(name in gname()) {
        let t = Tmp::new(&src_struct_with_child(&name, "Root", "Inner"));
        let gs = extract(t.path());
        prop_assert_ne!(sf(&gs[0])["name"].as_str().unwrap(), "Inner");
    }

    /// 5c — each grammar in multi-grammar file has source_file pointing to its own root
    #[test]
    fn sf_per_grammar_in_multi_file(prefix in gname(), n in 2..=4usize) {
        let t = Tmp::new(&src_multi_grammar(&prefix, n));
        let gs = extract(t.path());
        prop_assert_eq!(gs.len(), n);
        for g in &gs {
            prop_assert_eq!(sf(g)["name"].as_str().unwrap(), "Root");
        }
    }
}

// ===========================================================================
// 6. source_file with enum root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 6a — enum root: referenced rule is CHOICE
    #[test]
    fn sf_enum_root_is_choice(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        let root_rule = &gs[0]["rules"][sf(&gs[0])["name"].as_str().unwrap()];
        prop_assert_eq!(root_rule["type"].as_str().unwrap(), "CHOICE");
    }

    /// 6b — enum with N variants: CHOICE has N members
    #[test]
    fn sf_enum_choice_member_count(name in gname(), n in 1..=4usize) {
        let t = Tmp::new(&src_enum_multi_variant(&name, "Tok", n));
        let gs = extract(t.path());
        let members = gs[0]["rules"]["Tok"]["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), n);
    }

    /// 6c — enum root: source_file is distinct from root rule body
    #[test]
    fn sf_distinct_from_enum_body(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        let sf_val = sf(&gs[0]);
        let root_val = &gs[0]["rules"][root.as_str()];
        prop_assert_ne!(sf_val, root_val);
    }
}

// ===========================================================================
// 7. source_file generation determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 7a — same struct source → identical source_file across runs
    #[test]
    fn sf_deterministic_struct(name in gname(), root in pascal()) {
        let src = src_struct_root(&name, &root);
        let t1 = Tmp::new(&src);
        let t2 = Tmp::new(&src);
        let g1 = extract(t1.path());
        let g2 = extract(t2.path());
        prop_assert_eq!(sf(&g1[0]), sf(&g2[0]));
    }

    /// 7b — same enum source → identical source_file across runs
    #[test]
    fn sf_deterministic_enum(name in gname(), root in pascal()) {
        let src = src_enum_root(&name, &root);
        let t1 = Tmp::new(&src);
        let t2 = Tmp::new(&src);
        let g1 = extract(t1.path());
        let g2 = extract(t2.path());
        prop_assert_eq!(sf(&g1[0]), sf(&g2[0]));
    }

    /// 7c — JSON round-trip preserves source_file
    #[test]
    fn sf_json_roundtrip(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let json = serde_json::to_string(&gs[0]).unwrap();
        let back: Value = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sf(&gs[0]), sf(&back));
    }

    /// 7d — grammar name change does not alter source_file value
    #[test]
    fn sf_independent_of_grammar_name(
        name_a in gname(),
        name_b in gname(),
        root in pascal(),
    ) {
        let ta = Tmp::new(&src_struct_root(&name_a, &root));
        let tb = Tmp::new(&src_struct_root(&name_b, &root));
        let ga = extract(ta.path());
        let gb = extract(tb.path());
        prop_assert_eq!(sf(&ga[0]), sf(&gb[0]));
    }
}

// ===========================================================================
// 8. source_file name format
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 8a — source_file key is literally "source_file"
    #[test]
    fn sf_key_is_source_file(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let keys: Vec<&String> = gs[0]["rules"].as_object().unwrap().keys().collect();
        prop_assert!(keys.contains(&&"source_file".to_string()));
    }

    /// 8b — source_file name value is a valid Rust identifier
    #[test]
    fn sf_name_is_valid_ident(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let sym_name = sf(&gs[0])["name"].as_str().unwrap();
        prop_assert!(syn::parse_str::<syn::Ident>(sym_name).is_ok(),
            "source_file name '{}' is not a valid Rust ident", sym_name);
    }

    /// 8c — source_file name starts with uppercase (PascalCase root)
    #[test]
    fn sf_name_starts_uppercase(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_struct_root(&name, &root));
        let gs = extract(t.path());
        let sym_name = sf(&gs[0])["name"].as_str().unwrap();
        prop_assert!(sym_name.starts_with(|c: char| c.is_ascii_uppercase()),
            "source_file name '{}' should start uppercase", sym_name);
    }

    /// 8d — source_file "name" field is a JSON string (never null/number)
    #[test]
    fn sf_name_is_json_string(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        prop_assert!(sf(&gs[0])["name"].is_string());
    }

    /// 8e — source_file "type" field is a JSON string (never null/number)
    #[test]
    fn sf_type_is_json_string(name in gname(), root in pascal()) {
        let t = Tmp::new(&src_enum_root(&name, &root));
        let gs = extract(t.path());
        prop_assert!(sf(&gs[0])["type"].is_string());
    }
}
