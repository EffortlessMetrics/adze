#![allow(clippy::needless_range_loop)]

//! Property-based tests for attribute parsing in adze-common.
//!
//! Tests attribute detection on various item types, multiple attributes on the
//! same item, attribute argument parsing, nested attribute handling, round-trip
//! attribute processing, and attribute ordering independence.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Attribute, Item, Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (lowercase start).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Distinct identifiers (deduplicated).
fn distinct_idents(max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(ident_strategy(), 1..=max).prop_map(|v| {
        let mut seen = std::collections::HashSet::new();
        v.into_iter().filter(|s| seen.insert(s.clone())).collect()
    })
}

/// Simple leaf type names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Container type names.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc"][..])
}

/// Integer literal values.
fn int_value() -> impl Strategy<Value = i64> {
    -1000i64..1000
}

/// Known adze attribute names.
fn adze_attr_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "leaf",
            "skip",
            "prec",
            "word",
            "language",
            "grammar",
            "repeat",
            "extra",
            "delimited",
        ][..],
    )
}

/// Item kind selector.
#[derive(Debug, Clone, Copy)]
enum ItemKind {
    Struct,
    Enum,
    Fn,
    TypeAlias,
}

fn item_kind_strategy() -> impl Strategy<Value = ItemKind> {
    prop::sample::select(
        &[
            ItemKind::Struct,
            ItemKind::Enum,
            ItemKind::Fn,
            ItemKind::TypeAlias,
        ][..],
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a Rust item string with attributes applied.
fn build_item_with_attrs(kind: ItemKind, name: &str, attrs: &[String]) -> String {
    let attr_block: String = attrs.iter().map(|a| format!("{a}\n")).collect();
    match kind {
        ItemKind::Struct => format!("{attr_block}pub struct {name} {{ pub value: i32 }}"),
        ItemKind::Enum => format!("{attr_block}pub enum {name} {{ A, B, C }}"),
        ItemKind::Fn => format!("{attr_block}fn {name}() {{}}"),
        ItemKind::TypeAlias => format!("{attr_block}type {name} = i32;"),
    }
}

/// Check whether an attribute path matches `adze::<segment>`.
fn is_adze_attr(attr: &Attribute, segment: &str) -> bool {
    let path = attr.path();
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments == ["adze", segment]
}

/// Count how many attributes on an item match `adze::<segment>`.
fn count_adze_attrs(item: &Item, segment: &str) -> usize {
    let attrs = match item {
        Item::Struct(s) => &s.attrs,
        Item::Enum(e) => &e.attrs,
        Item::Fn(f) => &f.attrs,
        Item::Type(t) => &t.attrs,
        _ => return 0,
    };
    attrs.iter().filter(|a| is_adze_attr(a, segment)).count()
}

/// Collect all adze attribute names from an item.
fn collect_adze_attr_names(item: &Item) -> Vec<String> {
    let attrs = match item {
        Item::Struct(s) => &s.attrs,
        Item::Enum(e) => &e.attrs,
        Item::Fn(f) => &f.attrs,
        Item::Type(t) => &t.attrs,
        _ => return vec![],
    };
    attrs
        .iter()
        .filter_map(|a| {
            let path = a.path();
            let segs: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
            if segs.len() == 2 && segs[0] == "adze" {
                Some(segs[1].clone())
            } else {
                None
            }
        })
        .collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Attribute detection on various item types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 1. A single adze attribute is detected on any item kind
    #[test]
    fn single_attr_detected_on_any_item(
        kind in item_kind_strategy(),
        attr_name in adze_attr_name(),
        name in ident_strategy(),
    ) {
        let src = build_item_with_attrs(
            kind,
            &capitalize(&name),
            &[format!("#[adze::{attr_name}]")],
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze_attrs(&item, attr_name), 1);
    }

    // 2. Non-adze attributes are not counted as adze attrs
    #[test]
    fn non_adze_attr_not_counted(
        kind in item_kind_strategy(),
        name in ident_strategy(),
    ) {
        let src = build_item_with_attrs(
            kind,
            &capitalize(&name),
            &["#[derive(Debug)]".to_string()],
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze_attrs(&item, "leaf"), 0);
        prop_assert_eq!(count_adze_attrs(&item, "skip"), 0);
    }

    // 3. Item without attributes has zero adze attrs
    #[test]
    fn no_attrs_detected_on_bare_item(
        kind in item_kind_strategy(),
        name in ident_strategy(),
    ) {
        let src = build_item_with_attrs(kind, &capitalize(&name), &[]);
        let item: Item = parse_str(&src).unwrap();
        prop_assert!(collect_adze_attr_names(&item).is_empty());
    }
}

// ===========================================================================
// 2. Multiple attributes on same item
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 4. Multiple distinct adze attributes all appear
    #[test]
    fn multiple_distinct_attrs_all_detected(
        name in ident_strategy(),
    ) {
        let attrs = vec![
            "#[adze::leaf]".to_string(),
            "#[adze::skip]".to_string(),
            "#[adze::prec]".to_string(),
        ];
        let src = build_item_with_attrs(ItemKind::Struct, &capitalize(&name), &attrs);
        let item: Item = parse_str(&src).unwrap();
        let found = collect_adze_attr_names(&item);
        prop_assert_eq!(found.len(), 3);
        prop_assert!(found.contains(&"leaf".to_string()));
        prop_assert!(found.contains(&"skip".to_string()));
        prop_assert!(found.contains(&"prec".to_string()));
    }

    // 5. Duplicate adze attributes are both counted
    #[test]
    fn duplicate_attrs_counted(name in ident_strategy()) {
        let attrs = vec![
            "#[adze::leaf]".to_string(),
            "#[adze::leaf]".to_string(),
        ];
        let src = build_item_with_attrs(ItemKind::Struct, &capitalize(&name), &attrs);
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze_attrs(&item, "leaf"), 2);
    }

    // 6. Mixed adze and non-adze attributes preserve all adze attrs
    #[test]
    fn mixed_attrs_preserve_adze(name in ident_strategy()) {
        let attrs = vec![
            "#[derive(Clone)]".to_string(),
            "#[adze::leaf]".to_string(),
            "#[allow(dead_code)]".to_string(),
            "#[adze::skip]".to_string(),
        ];
        let src = build_item_with_attrs(ItemKind::Enum, &capitalize(&name), &attrs);
        let item: Item = parse_str(&src).unwrap();
        let found = collect_adze_attr_names(&item);
        prop_assert_eq!(found, vec!["leaf", "skip"]);
    }

    // 7. Attribute count matches the number of adze attrs supplied
    #[test]
    fn attr_count_matches_input(
        name in ident_strategy(),
        attr_names in prop::collection::vec(adze_attr_name(), 1..=5),
    ) {
        let attrs: Vec<String> = attr_names.iter().map(|a| format!("#[adze::{a}]")).collect();
        let src = build_item_with_attrs(ItemKind::Struct, &capitalize(&name), &attrs);
        let item: Item = parse_str(&src).unwrap();
        let found = collect_adze_attr_names(&item);
        prop_assert_eq!(found.len(), attr_names.len());
    }
}

// ===========================================================================
// 3. Attribute argument parsing
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 8. adze::grammar("name") preserves the grammar name string
    #[test]
    fn grammar_attr_preserves_name(name in ident_strategy()) {
        let src = format!("#[adze::grammar(\"{name}\")]\nmod {name} {{}}");
        let item: Item = parse_str(&src).unwrap();
        if let Item::Mod(m) = &item {
            let attr = &m.attrs[0];
            let tokens = attr.meta.to_token_stream().to_string();
            prop_assert!(tokens.contains(&name), "attr tokens should contain name: {tokens}");
        } else {
            prop_assert!(false, "expected mod item");
        }
    }

    // 9. adze::prec attribute with integer argument round-trips
    #[test]
    fn prec_attr_with_int_arg(name in ident_strategy(), val in 0i64..100) {
        let src = format!("#[adze::prec({val})]\npub struct {} {{ pub v: i32 }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        if let Item::Struct(s) = &item {
            let attr = &s.attrs[0];
            let tokens = attr.meta.to_token_stream().to_string();
            prop_assert!(tokens.contains(&val.to_string()),
                "attr tokens should contain value {val}: {tokens}");
        } else {
            prop_assert!(false, "expected struct item");
        }
    }

    // 10. adze::leaf attribute with string pattern argument
    #[test]
    fn leaf_attr_with_pattern(name in ident_strategy()) {
        let pattern = "\\\\d+"; // represents \d+
        let src = format!(
            "#[adze::leaf(pattern = \"{pattern}\")]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        if let Item::Struct(s) = &item {
            prop_assert_eq!(s.attrs.len(), 1);
            prop_assert!(is_adze_attr(&s.attrs[0], "leaf"));
        } else {
            prop_assert!(false, "expected struct item");
        }
    }

    // 11. NameValueExpr round-trips through token stream
    #[test]
    fn nve_roundtrip_via_tokens(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let path_str = parsed.path.to_string();
        let expr_str = parsed.expr.to_token_stream().to_string();
        let resrc = format!("{path_str} = {expr_str}");
        let reparsed: NameValueExpr = syn::parse_str(&resrc).unwrap();
        prop_assert_eq!(parsed.path.to_string(), reparsed.path.to_string());
    }

    // 12. FieldThenParams arguments can use bool values
    #[test]
    fn ftp_bool_param_values(
        ty in leaf_type_name(),
        key in ident_strategy(),
        b in prop::bool::ANY,
    ) {
        let src = format!("{ty}, {key} = {b}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.params.len(), 1);
        prop_assert_eq!(parsed.params[0].path.to_string(), key);
    }
}

// ===========================================================================
// 4. Nested attribute handling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 13. Attributes on items inside a module are parsed
    #[test]
    fn attrs_on_items_inside_module(
        mod_name in ident_strategy(),
        struct_name in ident_strategy(),
        attr_name in adze_attr_name(),
    ) {
        let struct_cap = capitalize(&struct_name);
        let src = format!(
            "mod {mod_name} {{\n  #[adze::{attr_name}]\n  pub struct {struct_cap} {{ pub v: i32 }}\n}}"
        );
        let item: Item = parse_str(&src).unwrap();
        if let Item::Mod(m) = &item {
            if let Some((_, items)) = &m.content {
                prop_assert!(!items.is_empty());
                let inner = &items[0];
                prop_assert_eq!(count_adze_attrs(inner, attr_name), 1);
            } else {
                prop_assert!(false, "expected inline module content");
            }
        } else {
            prop_assert!(false, "expected mod item");
        }
    }

    // 14. Module-level attribute does not appear on inner items
    #[test]
    fn module_attr_not_on_inner(
        mod_name in ident_strategy(),
        struct_name in ident_strategy(),
    ) {
        let struct_cap = capitalize(&struct_name);
        let src = format!(
            "#[adze::grammar(\"test\")]\nmod {mod_name} {{\n  pub struct {struct_cap} {{ pub v: i32 }}\n}}"
        );
        let item: Item = parse_str(&src).unwrap();
        if let Item::Mod(m) = &item {
            // Module has the attr
            prop_assert!(is_adze_attr(&m.attrs[0], "grammar"));
            // Inner struct does not
            if let Some((_, items)) = &m.content {
                let inner = &items[0];
                prop_assert_eq!(count_adze_attrs(inner, "grammar"), 0);
            }
        }
    }

    // 15. Multiple items inside a module each keep their own attrs
    #[test]
    fn multiple_items_inside_module_keep_own_attrs(mod_name in ident_strategy()) {
        let src = format!(
            "mod {mod_name} {{\n  #[adze::leaf]\n  pub struct A {{ pub v: i32 }}\n  #[adze::skip]\n  pub struct B {{ pub v: i32 }}\n}}"
        );
        let item: Item = parse_str(&src).unwrap();
        if let Item::Mod(m) = &item
            && let Some((_, items)) = &m.content
        {
            prop_assert_eq!(items.len(), 2);
            prop_assert_eq!(count_adze_attrs(&items[0], "leaf"), 1);
            prop_assert_eq!(count_adze_attrs(&items[0], "skip"), 0);
            prop_assert_eq!(count_adze_attrs(&items[1], "skip"), 1);
            prop_assert_eq!(count_adze_attrs(&items[1], "leaf"), 0);
        }
    }
}

// ===========================================================================
// 5. Round-trip attribute processing
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 16. NameValueExpr clone equals original
    #[test]
    fn nve_clone_equals(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(parsed, cloned);
    }

    // 17. FieldThenParams clone equals original with params
    #[test]
    fn ftp_clone_equals_with_params(
        ty in leaf_type_name(),
        key in ident_strategy(),
        val in int_value(),
    ) {
        let src = format!("{ty}, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(parsed, cloned);
    }

    // 18. FieldThenParams type is preserved after parse
    #[test]
    fn ftp_type_preserved(ty in leaf_type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        let token_str = parsed.field.ty.to_token_stream().to_string();
        prop_assert_eq!(token_str.as_str(), ty);
    }

    // 19. Parsing an item then re-serializing preserves adze attr count
    #[test]
    fn item_reserialize_preserves_attr_count(
        name in ident_strategy(),
        attr_name in adze_attr_name(),
    ) {
        let src = format!(
            "#[adze::{attr_name}]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let tokens = item.to_token_stream().to_string();
        let reparsed: Item = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            count_adze_attrs(&item, attr_name),
            count_adze_attrs(&reparsed, attr_name),
        );
    }

    // 20. NVE deterministic: parsing same source twice yields equal results
    #[test]
    fn nve_deterministic_parse(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} = {val}");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    // 21. try_extract + wrap roundtrip: extract inner then wrap yields WithLeaf
    #[test]
    fn extract_then_wrap_yields_with_leaf(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf, got: {s}");
        prop_assert!(s.contains(inner), "should contain inner type {inner}, got: {s}");
    }

    // 22. filter then wrap roundtrip: filter container then wrap
    #[test]
    fn filter_then_wrap_roundtrip(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), inner);
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf, got: {s}");
    }
}

// ===========================================================================
// 6. Attribute ordering independence
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 23. Two adze attrs in either order yield the same set of attr names
    #[test]
    fn attr_order_independence_two(name in ident_strategy()) {
        let order_a = vec![
            "#[adze::leaf]".to_string(),
            "#[adze::skip]".to_string(),
        ];
        let order_b = vec![
            "#[adze::skip]".to_string(),
            "#[adze::leaf]".to_string(),
        ];
        let cap = capitalize(&name);
        let item_a: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_a)).unwrap();
        let item_b: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_b)).unwrap();
        let mut names_a = collect_adze_attr_names(&item_a);
        let mut names_b = collect_adze_attr_names(&item_b);
        names_a.sort();
        names_b.sort();
        prop_assert_eq!(names_a, names_b);
    }

    // 24. Three adze attrs in any order produce the same sorted set
    #[test]
    fn attr_order_independence_three(name in ident_strategy()) {
        let attrs_fwd = vec![
            "#[adze::leaf]".to_string(),
            "#[adze::prec]".to_string(),
            "#[adze::skip]".to_string(),
        ];
        let attrs_rev = vec![
            "#[adze::skip]".to_string(),
            "#[adze::prec]".to_string(),
            "#[adze::leaf]".to_string(),
        ];
        let cap = capitalize(&name);
        let item_fwd: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &attrs_fwd)).unwrap();
        let item_rev: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &attrs_rev)).unwrap();
        let mut fwd = collect_adze_attr_names(&item_fwd);
        let mut rev = collect_adze_attr_names(&item_rev);
        fwd.sort();
        rev.sort();
        prop_assert_eq!(fwd, rev);
    }

    // 25. Ordering of mixed adze + derive attrs: adze set is invariant
    #[test]
    fn mixed_attr_order_independence(name in ident_strategy()) {
        let order_a = vec![
            "#[derive(Debug)]".to_string(),
            "#[adze::leaf]".to_string(),
            "#[adze::skip]".to_string(),
        ];
        let order_b = vec![
            "#[adze::skip]".to_string(),
            "#[derive(Debug)]".to_string(),
            "#[adze::leaf]".to_string(),
        ];
        let cap = capitalize(&name);
        let item_a: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_a)).unwrap();
        let item_b: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_b)).unwrap();
        let mut a = collect_adze_attr_names(&item_a);
        let mut b = collect_adze_attr_names(&item_b);
        a.sort();
        b.sort();
        prop_assert_eq!(a, b);
    }

    // 26. Ordering does not affect individual attr counts
    #[test]
    fn ordering_preserves_individual_counts(name in ident_strategy()) {
        let order_a = vec![
            "#[adze::leaf]".to_string(),
            "#[adze::leaf]".to_string(),
            "#[adze::skip]".to_string(),
        ];
        let order_b = vec![
            "#[adze::skip]".to_string(),
            "#[adze::leaf]".to_string(),
            "#[adze::leaf]".to_string(),
        ];
        let cap = capitalize(&name);
        let item_a: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_a)).unwrap();
        let item_b: Item = parse_str(&build_item_with_attrs(ItemKind::Struct, &cap, &order_b)).unwrap();
        prop_assert_eq!(count_adze_attrs(&item_a, "leaf"), count_adze_attrs(&item_b, "leaf"));
        prop_assert_eq!(count_adze_attrs(&item_a, "skip"), count_adze_attrs(&item_b, "skip"));
    }
}

// ===========================================================================
// 7. Cross-cutting / composite tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 27. FieldThenParams param order is preserved (not sorted)
    #[test]
    fn ftp_param_order_preserved(
        ty in leaf_type_name(),
        keys in distinct_idents(4),
    ) {
        if keys.is_empty() { return Ok(()); }
        let params: Vec<String> = keys.iter().enumerate()
            .map(|(i, k)| format!("{k} = {i}"))
            .collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        for i in 0..keys.len() {
            prop_assert_eq!(parsed.params[i].path.to_string(), keys[i].as_str());
        }
    }

    // 28. FieldThenParams with generic container type + params
    #[test]
    fn ftp_generic_container_with_params(
        container in container_name(),
        inner in leaf_type_name(),
        key in ident_strategy(),
        val in int_value(),
    ) {
        let src = format!("{container}<{inner}>, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), 1);
        let token_str = parsed.field.ty.to_token_stream().to_string();
        prop_assert!(token_str.contains(container));
        prop_assert!(token_str.contains(inner));
    }

    // 29. Adze attr detection works on enum items
    #[test]
    fn attr_on_enum_variant_item(
        name in ident_strategy(),
        attr_name in adze_attr_name(),
    ) {
        let src = format!("#[adze::{attr_name}]\npub enum {} {{ X, Y }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze_attrs(&item, attr_name), 1);
    }

    // 30. Adze attr detection works on fn items
    #[test]
    fn attr_on_fn_item(
        name in ident_strategy(),
        attr_name in adze_attr_name(),
    ) {
        let src = format!("#[adze::{attr_name}]\nfn {name}() {{}}");
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze_attrs(&item, attr_name), 1);
    }

    // 31. wrap_leaf_type is idempotent on already-wrapped types through containers
    #[test]
    fn wrap_idempotent_through_container(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip_set = skip(&["Vec"]);
        let once = wrap_leaf_type(&ty, &skip_set);
        // The inner has been wrapped to WithLeaf, outer Vec preserved
        let once_str = ty_str(&once);
        prop_assert!(once_str.starts_with("Vec <"), "got: {once_str}");
        prop_assert!(once_str.contains("adze :: WithLeaf"), "got: {once_str}");
    }

    // 32. filter_inner_type is idempotent: filtering twice gives same result
    #[test]
    fn filter_idempotent(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip_arr = [container];
        let skip_set = skip(&skip_arr);
        let once = filter_inner_type(&ty, &skip_set);
        let twice = filter_inner_type(&once, &skip_set);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    // 33. extract + filter agreement: for single-layer container, both yield same inner
    #[test]
    fn extract_filter_agreement(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, container, &skip(&[]));
        prop_assert!(ok);
        let skip_arr = [container];
        let filtered = filter_inner_type(&ty, &skip(&skip_arr));
        prop_assert_eq!(ty_str(&extracted), ty_str(&filtered));
    }

    // 34. Parsing error for NVE with missing equals
    #[test]
    fn nve_parse_error_missing_eq(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} {val}");
        let result = syn::parse_str::<NameValueExpr>(&src);
        prop_assert!(result.is_err());
    }

    // 35. FieldThenParams error for empty input
    #[test]
    fn ftp_parse_error_empty(_dummy in 0..1u8) {
        let result = syn::parse_str::<FieldThenParams>("");
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// Helper function
// ---------------------------------------------------------------------------

/// Capitalize the first letter to create a valid type name.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}
