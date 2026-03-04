#![allow(clippy::needless_range_loop)]

//! Property-based tests for module processing in adze-common.
//!
//! Covers: module item extraction (structs, enums), module attribute processing,
//! grammar annotation handling, visibility handling, nested items,
//! module name preservation, empty modules, and processing determinism.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Item, ItemMod, Type, Visibility, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,10}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn struct_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][A-Za-z0-9]{0,10}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["i32", "u32", "f64", "bool", "String", "usize", "u8", "i64"][..])
}

#[derive(Debug, Clone, Copy)]
enum VisKind {
    Inherited,
    Pub,
    PubCrate,
}

fn vis_strategy() -> impl Strategy<Value = VisKind> {
    prop::sample::select(&[VisKind::Inherited, VisKind::Pub, VisKind::PubCrate][..])
}

fn vis_prefix(v: VisKind) -> &'static str {
    match v {
        VisKind::Inherited => "",
        VisKind::Pub => "pub ",
        VisKind::PubCrate => "pub(crate) ",
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_mod(src: &str) -> ItemMod {
    parse_str(src).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Module item extraction (structs, enums) — property-based
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Struct count inside a module matches the number we insert.
    #[test]
    fn struct_count_matches_inserted(count in 1usize..=8) {
        let body: String = (0..count)
            .map(|i| format!("    struct S{i};"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("mod m {{\n{body}\n}}");
        let m = parse_mod(&src);
        let structs = module_items(&m).iter().filter(|i| matches!(i, Item::Struct(_))).count();
        prop_assert_eq!(structs, count);
    }

    /// Enum count inside a module matches the number we insert.
    #[test]
    fn enum_count_matches_inserted(count in 1usize..=8) {
        let body: String = (0..count)
            .map(|i| format!("    enum E{i} {{ A, B }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("mod m {{\n{body}\n}}");
        let m = parse_mod(&src);
        let enums = module_items(&m).iter().filter(|i| matches!(i, Item::Enum(_))).count();
        prop_assert_eq!(enums, count);
    }

    /// Struct ident matches what we put in.
    #[test]
    fn struct_ident_extracted_correctly(name in struct_name_strategy()) {
        let src = format!("mod m {{ struct {name}; }}");
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            prop_assert_eq!(s.ident.to_string(), name);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    /// Enum ident matches what we put in.
    #[test]
    fn enum_ident_extracted_correctly(name in struct_name_strategy()) {
        let src = format!("mod m {{ enum {name} {{ X }} }}");
        let m = parse_mod(&src);
        if let Item::Enum(e) = &module_items(&m)[0] {
            prop_assert_eq!(e.ident.to_string(), name);
        } else {
            prop_assert!(false, "expected enum");
        }
    }

    /// Enum variant count matches inserted count.
    #[test]
    fn enum_variant_count_matches(n_variants in 1usize..=10) {
        let variants: String = (0..n_variants)
            .map(|i| format!("V{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let src = format!("mod m {{ enum E {{ {variants} }} }}");
        let m = parse_mod(&src);
        if let Item::Enum(e) = &module_items(&m)[0] {
            prop_assert_eq!(e.variants.len(), n_variants);
        } else {
            prop_assert!(false, "expected enum");
        }
    }
}

// ===========================================================================
// 2. Module attribute processing
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// A module with N outer attributes has exactly N attrs.
    #[test]
    fn module_attr_count_matches(n_attrs in 1usize..=4) {
        let attrs: String = (0..n_attrs)
            .map(|_| "#[allow(dead_code)]".to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("{attrs}\nmod m {{ }}");
        let m = parse_mod(&src);
        prop_assert_eq!(m.attrs.len(), n_attrs);
    }

    /// Struct attributes inside module are preserved.
    #[test]
    fn struct_attr_inside_module_preserved(n_attrs in 1usize..=3) {
        let attrs: String = (0..n_attrs)
            .map(|_| "    #[derive(Debug)]".to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("mod m {{\n{attrs}\n    struct Foo;\n}}");
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            prop_assert_eq!(s.attrs.len(), n_attrs);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    /// cfg attribute path is preserved on module.
    #[test]
    fn cfg_attr_path_preserved(mod_name in ident_strategy()) {
        let src = format!("#[cfg(test)] mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        let path_str = m.attrs[0].path().to_token_stream().to_string();
        prop_assert_eq!(path_str, "cfg");
    }
}

// ===========================================================================
// 3. Module with grammar annotation
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Grammar-annotated module preserves the grammar attribute.
    #[test]
    fn grammar_attr_present(
        grammar_name in ident_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = format!(
            "#[adze::grammar(\"{grammar_name}\")]\nmod {mod_name} {{}}"
        );
        let m = parse_mod(&src);
        let has_grammar = m.attrs.iter().any(|a| {
            let segs: Vec<_> = a.path().segments.iter().map(|s| s.ident.to_string()).collect();
            segs == ["adze", "grammar"]
        });
        prop_assert!(has_grammar);
    }

    /// Grammar module body items are accessible after parsing.
    #[test]
    fn grammar_module_body_items_accessible(
        mod_name in ident_strategy(),
        ty in leaf_type(),
    ) {
        let src = format!(
            "#[adze::grammar(\"g\")]\nmod {mod_name} {{ pub struct Root {{ pub v: {ty}, }} }}"
        );
        let m = parse_mod(&src);
        let items = module_items(&m);
        prop_assert_eq!(items.len(), 1);
        prop_assert!(matches!(&items[0], Item::Struct(_)));
    }

    /// Type extraction works on fields inside grammar modules.
    #[test]
    fn type_extraction_in_grammar_module(
        mod_name in ident_strategy(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "#[adze::grammar(\"g\")]\nmod {mod_name} {{ pub struct Node {{ pub xs: Vec<{inner}>, }} }}"
        );
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            let field = s.fields.iter().next().unwrap();
            let (result, ok) = try_extract_inner_type(&field.ty, "Vec", &skip(&[]));
            prop_assert!(ok);
            prop_assert_eq!(type_str(&result), inner);
        }
    }
}

// ===========================================================================
// 4. Module visibility handling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Module visibility matches the prefix we use.
    #[test]
    fn module_visibility_matches(
        vis in vis_strategy(),
        mod_name in ident_strategy(),
    ) {
        let prefix = vis_prefix(vis);
        let src = format!("{prefix}mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        match vis {
            VisKind::Inherited => prop_assert!(matches!(m.vis, Visibility::Inherited)),
            VisKind::Pub => prop_assert!(matches!(m.vis, Visibility::Public(_))),
            VisKind::PubCrate => prop_assert!(matches!(m.vis, Visibility::Restricted(_))),
        }
    }

    /// Struct visibility inside module matches applied prefix.
    #[test]
    fn struct_visibility_inside_module(
        mod_vis in vis_strategy(),
        struct_vis in vis_strategy(),
    ) {
        let mp = vis_prefix(mod_vis);
        let sp = vis_prefix(struct_vis);
        let src = format!("{mp}mod m {{ {sp}struct Foo; }}");
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            match struct_vis {
                VisKind::Inherited => prop_assert!(matches!(s.vis, Visibility::Inherited)),
                VisKind::Pub => prop_assert!(matches!(s.vis, Visibility::Public(_))),
                VisKind::PubCrate => prop_assert!(matches!(s.vis, Visibility::Restricted(_))),
            }
        }
    }

    /// Field visibility is preserved inside module struct.
    #[test]
    fn field_visibility_in_module_struct(field_vis in vis_strategy()) {
        let fp = vis_prefix(field_vis);
        let src = format!("mod m {{ struct Foo {{ {fp}x: i32 }} }}");
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            let field = s.fields.iter().next().unwrap();
            match field_vis {
                VisKind::Inherited => prop_assert!(matches!(field.vis, Visibility::Inherited)),
                VisKind::Pub => prop_assert!(matches!(field.vis, Visibility::Public(_))),
                VisKind::PubCrate => prop_assert!(matches!(field.vis, Visibility::Restricted(_))),
            }
        }
    }
}

// ===========================================================================
// 5. Module with nested items
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Nested module is parsed as Item::Mod.
    #[test]
    fn nested_module_detected(
        outer in ident_strategy(),
        inner in ident_strategy(),
    ) {
        let src = format!("mod {outer} {{ mod {inner} {{ }} }}");
        let m = parse_mod(&src);
        let items = module_items(&m);
        prop_assert_eq!(items.len(), 1);
        prop_assert!(matches!(&items[0], Item::Mod(_)));
    }

    /// Nested module name is preserved.
    #[test]
    fn nested_module_name_preserved(
        outer in ident_strategy(),
        inner in ident_strategy(),
    ) {
        let src = format!("mod {outer} {{ mod {inner} {{ struct A; }} }}");
        let m = parse_mod(&src);
        if let Item::Mod(nested) = &module_items(&m)[0] {
            prop_assert_eq!(nested.ident.to_string(), inner);
        } else {
            prop_assert!(false, "expected nested module");
        }
    }

    /// Items inside nested module are accessible.
    #[test]
    fn nested_module_items_accessible(outer in ident_strategy()) {
        let src = format!("mod {outer} {{ mod inner {{ struct Foo; enum Bar {{ X }} }} }}");
        let m = parse_mod(&src);
        if let Item::Mod(nested) = &module_items(&m)[0] {
            let nested_items = &nested.content.as_ref().unwrap().1;
            prop_assert_eq!(nested_items.len(), 2);
        } else {
            prop_assert!(false, "expected nested module");
        }
    }

    /// Sibling and nested items coexist correctly.
    #[test]
    fn sibling_and_nested_items(outer in ident_strategy()) {
        let src = format!(
            "mod {outer} {{ struct Top; mod child {{ struct Inner; }} enum Side {{ A }} }}"
        );
        let m = parse_mod(&src);
        let items = module_items(&m);
        prop_assert_eq!(items.len(), 3);
        prop_assert!(matches!(&items[0], Item::Struct(s) if s.ident == "Top"));
        prop_assert!(matches!(&items[1], Item::Mod(m) if m.ident == "child"));
        prop_assert!(matches!(&items[2], Item::Enum(e) if e.ident == "Side"));
    }
}

// ===========================================================================
// 6. Module name preservation
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Module name round-trips through parse -> tokenize -> reparse.
    #[test]
    fn module_name_roundtrips(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{ struct A; }}");
        let m = parse_mod(&src);
        let tokens = m.to_token_stream().to_string();
        let reparsed: ItemMod = parse_str(&tokens).unwrap();
        prop_assert_eq!(m.ident.to_string(), reparsed.ident.to_string());
    }

    /// Module name survives grammar annotation round-trip.
    #[test]
    fn grammar_module_name_roundtrips(mod_name in ident_strategy()) {
        let src = format!("#[adze::grammar(\"g\")]\nmod {mod_name} {{ }}");
        let m = parse_mod(&src);
        let tokens = m.to_token_stream().to_string();
        let reparsed: ItemMod = parse_str(&tokens).unwrap();
        prop_assert_eq!(mod_name, reparsed.ident.to_string());
    }

    /// Module name is exactly the identifier we supplied.
    #[test]
    fn module_name_is_exact(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        prop_assert_eq!(m.ident.to_string(), mod_name);
    }
}

// ===========================================================================
// 7. Module processing with empty modules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Empty module always has zero items.
    #[test]
    fn empty_module_zero_items(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        prop_assert!(module_items(&m).is_empty());
    }

    /// Empty grammar module has zero items.
    #[test]
    fn empty_grammar_module_zero_items(mod_name in ident_strategy()) {
        let src = format!("#[adze::grammar(\"g\")]\nmod {mod_name} {{ }}");
        let m = parse_mod(&src);
        prop_assert!(module_items(&m).is_empty());
    }

    /// Empty module has content (braces present), not None.
    #[test]
    fn empty_module_has_content_some(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        prop_assert!(m.content.is_some());
    }

    /// Empty pub module visibility is public and items empty.
    #[test]
    fn empty_pub_module_vis_and_items(mod_name in ident_strategy()) {
        let src = format!("pub mod {mod_name} {{ }}");
        let m = parse_mod(&src);
        prop_assert!(matches!(m.vis, Visibility::Public(_)));
        prop_assert!(module_items(&m).is_empty());
    }
}

// ===========================================================================
// 8. Module processing determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Parsing the same module source twice yields identical ident.
    #[test]
    fn parsing_deterministic_ident(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{ struct A; enum B {{ X }} }}");
        let a = parse_mod(&src);
        let b = parse_mod(&src);
        prop_assert_eq!(a.ident.to_string(), b.ident.to_string());
    }

    /// Parsing the same module source twice yields identical item count.
    #[test]
    fn parsing_deterministic_item_count(
        mod_name in ident_strategy(),
        n in 1usize..=6,
    ) {
        let body: String = (0..n)
            .map(|i| format!("    struct S{i};"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("mod {mod_name} {{\n{body}\n}}");
        let a = parse_mod(&src);
        let b = parse_mod(&src);
        prop_assert_eq!(module_items(&a).len(), module_items(&b).len());
    }

    /// Token output is deterministic: tokenize twice gives same string.
    #[test]
    fn tokenize_deterministic(mod_name in ident_strategy(), ty in leaf_type()) {
        let src = format!("mod {mod_name} {{ struct Foo {{ x: {ty} }} }}");
        let m = parse_mod(&src);
        let t1 = m.to_token_stream().to_string();
        let t2 = m.to_token_stream().to_string();
        prop_assert_eq!(t1, t2);
    }

    /// Type utility functions are deterministic on module field types.
    #[test]
    fn type_utilities_deterministic(inner in leaf_type()) {
        let src = format!("mod m {{ struct N {{ v: Vec<Box<{inner}>> }} }}");
        let m = parse_mod(&src);
        if let Item::Struct(s) = &module_items(&m)[0] {
            let field = s.fields.iter().next().unwrap();

            let (a1, ok1) = try_extract_inner_type(&field.ty, "Vec", &skip(&["Box"]));
            let (a2, ok2) = try_extract_inner_type(&field.ty, "Vec", &skip(&["Box"]));
            prop_assert_eq!(ok1, ok2);
            prop_assert_eq!(type_str(&a1), type_str(&a2));

            let f1 = filter_inner_type(&field.ty, &skip(&["Vec", "Box"]));
            let f2 = filter_inner_type(&field.ty, &skip(&["Vec", "Box"]));
            prop_assert_eq!(type_str(&f1), type_str(&f2));

            let w1 = wrap_leaf_type(&field.ty, &skip(&["Vec"]));
            let w2 = wrap_leaf_type(&field.ty, &skip(&["Vec"]));
            prop_assert_eq!(type_str(&w1), type_str(&w2));
        }
    }
}
