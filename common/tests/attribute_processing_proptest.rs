#![allow(clippy::needless_range_loop)]

//! Property-based tests for attribute processing in adze-common.
//!
//! Tests focus on how adze attributes (`#[adze::leaf]`, `#[adze::language]`,
//! `#[adze::grammar]`, etc.) are parsed, processed, and interact with the
//! type-processing utilities provided by adze-common.

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

fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,10}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i32", "u32", "i64", "u64", "f32", "f64", "bool", "char", "String", "usize", "isize",
            "u8", "i8",
        ][..],
    )
}

fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc"][..])
}

fn adze_attr() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "leaf",
            "language",
            "grammar",
            "skip",
            "prec",
            "word",
            "repeat",
            "extra",
            "delimited",
        ][..],
    )
}

fn non_adze_attr() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "#[derive(Debug)]",
            "#[derive(Clone)]",
            "#[allow(dead_code)]",
            "#[cfg(test)]",
            "#[doc = \"hello\"]",
        ][..],
    )
}

fn int_value() -> impl Strategy<Value = i64> {
    -500i64..500
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(ch) => ch.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn is_adze_attr(attr: &Attribute, segment: &str) -> bool {
    let segs: Vec<_> = attr
        .path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    segs == ["adze", segment]
}

fn get_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Struct(s) => &s.attrs,
        Item::Enum(e) => &e.attrs,
        Item::Fn(f) => &f.attrs,
        Item::Type(t) => &t.attrs,
        Item::Mod(m) => &m.attrs,
        _ => &[],
    }
}

fn count_adze(item: &Item, segment: &str) -> usize {
    get_attrs(item)
        .iter()
        .filter(|a| is_adze_attr(a, segment))
        .count()
}

fn collect_adze_names(item: &Item) -> Vec<String> {
    get_attrs(item)
        .iter()
        .filter_map(|a| {
            let segs: Vec<_> = a
                .path()
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if segs.len() == 2 && segs[0] == "adze" {
                Some(segs[1].clone())
            } else {
                None
            }
        })
        .collect()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Parsing #[adze::leaf] attributes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 1. Bare `#[adze::leaf]` is recognised on a struct.
    #[test]
    fn leaf_attr_on_struct(name in ident_strategy()) {
        let src = format!("#[adze::leaf]\npub struct {} {{ pub v: i32 }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "leaf"), 1);
    }

    /// 2. `#[adze::leaf]` with a pattern parameter retains the attribute.
    #[test]
    fn leaf_attr_with_pattern_param(name in ident_strategy()) {
        let src = format!(
            "#[adze::leaf(pattern = \"[0-9]+\")]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "leaf"), 1);
        let tokens = get_attrs(&item)[0].meta.to_token_stream().to_string();
        prop_assert!(tokens.contains("pattern"));
    }

    /// 3. `#[adze::leaf]` on an enum is parsed.
    #[test]
    fn leaf_attr_on_enum(name in ident_strategy()) {
        let src = format!("#[adze::leaf]\npub enum {} {{ A, B }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "leaf"), 1);
    }

    /// 4. `#[adze::leaf]` with transform parameter preserves both key and value.
    #[test]
    fn leaf_attr_with_transform_param(name in ident_strategy(), val in 0i64..500) {
        let src = format!(
            "#[adze::leaf(transform = {val})]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let tokens = get_attrs(&item)[0].meta.to_token_stream().to_string();
        prop_assert!(tokens.contains(&val.to_string()));
    }
}

// ===========================================================================
// 2. Parsing #[adze::language] attributes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 5. `#[adze::language]` is recognised on an enum.
    #[test]
    fn language_attr_on_enum(name in ident_strategy()) {
        let src = format!("#[adze::language]\npub enum {} {{ A }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "language"), 1);
    }

    /// 6. `#[adze::language]` with a string parameter preserves the name.
    #[test]
    fn language_attr_with_name_param(name in ident_strategy()) {
        let src = format!(
            "#[adze::language(\"my_lang\")]\npub enum {} {{ X }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "language"), 1);
        let tokens = get_attrs(&item)[0].meta.to_token_stream().to_string();
        prop_assert!(tokens.contains("my_lang"));
    }

    /// 7. Re-serialising an item with `#[adze::language]` preserves it.
    #[test]
    fn language_attr_roundtrip(name in ident_strategy()) {
        let src = format!("#[adze::language]\npub enum {} {{ V }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        let re_src = item.to_token_stream().to_string();
        let reparsed: Item = parse_str(&re_src).unwrap();
        prop_assert_eq!(count_adze(&reparsed, "language"), 1);
    }
}

// ===========================================================================
// 3. Parsing #[adze::grammar] attributes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 8. `#[adze::grammar("name")]` on a module preserves the grammar name.
    #[test]
    fn grammar_attr_on_module(name in ident_strategy()) {
        let src = format!("#[adze::grammar(\"{name}\")]\nmod {name} {{}}");
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "grammar"), 1);
        let tokens = get_attrs(&item)[0].meta.to_token_stream().to_string();
        prop_assert!(tokens.contains(&name));
    }

    /// 9. `#[adze::grammar]` without params is still a valid attribute.
    #[test]
    fn grammar_attr_no_params(name in ident_strategy()) {
        let src = format!("#[adze::grammar]\nmod {name} {{}}");
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "grammar"), 1);
    }

    /// 10. `#[adze::grammar]` does not leak to inner items of the module.
    #[test]
    fn grammar_attr_stays_on_module(name in ident_strategy()) {
        let cap = capitalize(&name);
        let src = format!(
            "#[adze::grammar(\"{name}\")]\nmod {name} {{\n  pub struct {cap} {{ pub v: i32 }}\n}}"
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, "grammar"), 1);
        if let Item::Mod(m) = &item
            && let Some((_, items)) = &m.content
        {
            for inner in items {
                prop_assert_eq!(count_adze(inner, "grammar"), 0);
            }
        }
    }
}

// ===========================================================================
// 4. Attribute with parameters
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 11. NameValueExpr parses any valid identifier as key with an int value.
    #[test]
    fn nve_arbitrary_key_int_value(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} = {val}");
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(nve.path.to_string(), key);
    }

    /// 12. NameValueExpr parses string literal values.
    #[test]
    fn nve_string_value(key in ident_strategy()) {
        let src = format!("{key} = \"hello_world\"");
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(nve.path.to_string(), key);
        let expr_s = nve.expr.to_token_stream().to_string();
        prop_assert!(expr_s.contains("hello_world"));
    }

    /// 13. FieldThenParams with multiple NVE params preserves param count.
    #[test]
    fn ftp_multiple_params_count(
        ty in type_name(),
        k1 in ident_strategy(),
        k2 in ident_strategy(),
        v1 in int_value(),
        v2 in int_value(),
    ) {
        // Guard against duplicate keys generating parse confusion
        prop_assume!(k1 != k2);
        let src = format!("{ty}, {k1} = {v1}, {k2} = {v2}");
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ftp.params.len(), 2);
    }

    /// 14. FieldThenParams param keys match input order.
    #[test]
    fn ftp_param_keys_in_order(
        ty in type_name(),
        k1 in ident_strategy(),
        k2 in ident_strategy(),
    ) {
        prop_assume!(k1 != k2);
        let src = format!("{ty}, {k1} = 1, {k2} = 2");
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ftp.params[0].path.to_string(), k1);
        prop_assert_eq!(ftp.params[1].path.to_string(), k2);
    }

    /// 15. Attribute with parenthesised args tokenises correctly.
    #[test]
    fn attr_with_paren_args_tokenises(
        attr in adze_attr(),
        name in ident_strategy(),
        val in 0i64..500,
    ) {
        let src = format!(
            "#[adze::{attr}({val})]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let tokens = get_attrs(&item)[0].meta.to_token_stream().to_string();
        prop_assert!(tokens.contains(&val.to_string()));
    }
}

// ===========================================================================
// 5. Attribute without parameters
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 16. Every known adze attr name works without parameters.
    #[test]
    fn bare_adze_attr_parses(attr in adze_attr(), name in ident_strategy()) {
        let src = format!("#[adze::{attr}]\npub struct {} {{ pub v: i32 }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, attr), 1);
    }

    /// 17. Bare attribute re-serialises and re-parses identically.
    #[test]
    fn bare_attr_roundtrip(attr in adze_attr(), name in ident_strategy()) {
        let src = format!("#[adze::{attr}]\npub struct {} {{ pub v: i32 }}", capitalize(&name));
        let item: Item = parse_str(&src).unwrap();
        let re_src = item.to_token_stream().to_string();
        let reparsed: Item = parse_str(&re_src).unwrap();
        prop_assert_eq!(count_adze(&item, attr), count_adze(&reparsed, attr));
    }

    /// 18. FieldThenParams without params has empty params list.
    #[test]
    fn ftp_no_params_empty(ty in type_name()) {
        let ftp: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert!(ftp.params.is_empty());
        prop_assert!(ftp.comma.is_none());
    }
}

// ===========================================================================
// 6. Multiple attributes on one item
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 19. Two distinct adze attrs both appear in collected names.
    #[test]
    fn two_distinct_attrs_collected(name in ident_strategy()) {
        let src = format!(
            "#[adze::leaf]\n#[adze::skip]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert!(names.contains(&"leaf".to_string()));
        prop_assert!(names.contains(&"skip".to_string()));
    }

    /// 20. Adze attrs mixed with non-adze attrs: only adze ones collected.
    #[test]
    fn mixed_attrs_only_adze_collected(
        name in ident_strategy(),
        extra in non_adze_attr(),
    ) {
        let src = format!(
            "{extra}\n#[adze::leaf]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert_eq!(names.len(), 1);
        prop_assert_eq!(&names[0], "leaf");
    }

    /// 21. N adze attrs → collected names has length N.
    #[test]
    fn n_attrs_yields_n_names(
        name in ident_strategy(),
        attrs in prop::collection::vec(adze_attr(), 1..=6),
    ) {
        let attr_lines: Vec<String> = attrs.iter().map(|a| format!("#[adze::{a}]")).collect();
        let src = format!(
            "{}\npub struct {} {{ pub v: i32 }}",
            attr_lines.join("\n"),
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert_eq!(names.len(), attrs.len());
    }

    /// 22. Duplicate adze attrs are both counted.
    #[test]
    fn duplicate_attrs_both_counted(name in ident_strategy(), attr in adze_attr()) {
        let src = format!(
            "#[adze::{attr}]\n#[adze::{attr}]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        prop_assert_eq!(count_adze(&item, attr), 2);
    }
}

// ===========================================================================
// 7. Unknown attribute handling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 23. An unknown `#[adze::something]` still parses as an attribute.
    #[test]
    fn unknown_adze_attr_still_parses(name in ident_strategy(), unknown in ident_strategy()) {
        let src = format!(
            "#[adze::{unknown}]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert_eq!(names.len(), 1);
        prop_assert_eq!(&names[0], &unknown);
    }

    /// 24. Unknown attr mixed with known attr: both collected.
    #[test]
    fn unknown_with_known_both_collected(name in ident_strategy(), unknown in ident_strategy()) {
        prop_assume!(unknown != "leaf");
        let src = format!(
            "#[adze::{unknown}]\n#[adze::leaf]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert_eq!(names.len(), 2);
        prop_assert_eq!(&names[0], &unknown);
        prop_assert_eq!(&names[1], "leaf");
    }

    /// 25. Non-adze path attributes (e.g. `#[serde::rename]`) are not collected.
    #[test]
    fn foreign_path_attr_not_collected(name in ident_strategy()) {
        let src = format!(
            "#[serde::rename]\npub struct {} {{ pub v: i32 }}",
            capitalize(&name)
        );
        let item: Item = parse_str(&src).unwrap();
        let names = collect_adze_names(&item);
        prop_assert!(names.is_empty());
    }
}

// ===========================================================================
// 8. Attribute ordering
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 26. Reordering two adze attrs produces same sorted name set.
    #[test]
    fn two_attrs_order_independent(name in ident_strategy()) {
        let cap = capitalize(&name);
        let fwd = format!("#[adze::leaf]\n#[adze::skip]\npub struct {cap} {{ pub v: i32 }}");
        let rev = format!("#[adze::skip]\n#[adze::leaf]\npub struct {cap} {{ pub v: i32 }}");
        let item_f: Item = parse_str(&fwd).unwrap();
        let item_r: Item = parse_str(&rev).unwrap();
        let mut nf = collect_adze_names(&item_f);
        let mut nr = collect_adze_names(&item_r);
        nf.sort();
        nr.sort();
        prop_assert_eq!(nf, nr);
    }

    /// 27. Three adze attrs in any permutation yield same sorted set.
    #[test]
    fn three_attrs_permutation_invariant(name in ident_strategy()) {
        let cap = capitalize(&name);
        let a = format!("#[adze::leaf]\n#[adze::prec]\n#[adze::word]\npub struct {cap} {{ pub v: i32 }}");
        let b = format!("#[adze::word]\n#[adze::leaf]\n#[adze::prec]\npub struct {cap} {{ pub v: i32 }}");
        let item_a: Item = parse_str(&a).unwrap();
        let item_b: Item = parse_str(&b).unwrap();
        let mut na = collect_adze_names(&item_a);
        let mut nb = collect_adze_names(&item_b);
        na.sort();
        nb.sort();
        prop_assert_eq!(na, nb);
    }

    /// 28. Interleaving non-adze attrs doesn't affect adze name set.
    #[test]
    fn interleaved_non_adze_no_effect(name in ident_strategy()) {
        let cap = capitalize(&name);
        let plain = format!("#[adze::leaf]\n#[adze::skip]\npub struct {cap} {{ pub v: i32 }}");
        let inter = format!(
            "#[derive(Debug)]\n#[adze::leaf]\n#[allow(unused)]\n#[adze::skip]\npub struct {cap} {{ pub v: i32 }}"
        );
        let item_p: Item = parse_str(&plain).unwrap();
        let item_i: Item = parse_str(&inter).unwrap();
        let mut np = collect_adze_names(&item_p);
        let mut ni = collect_adze_names(&item_i);
        np.sort();
        ni.sort();
        prop_assert_eq!(np, ni);
    }

    /// 29. Ordering preserves per-attr count for duplicates.
    #[test]
    fn ordering_preserves_per_attr_count(name in ident_strategy()) {
        let cap = capitalize(&name);
        let a = format!("#[adze::leaf]\n#[adze::leaf]\n#[adze::skip]\npub struct {cap} {{ pub v: i32 }}");
        let b = format!("#[adze::skip]\n#[adze::leaf]\n#[adze::leaf]\npub struct {cap} {{ pub v: i32 }}");
        let ia: Item = parse_str(&a).unwrap();
        let ib: Item = parse_str(&b).unwrap();
        prop_assert_eq!(count_adze(&ia, "leaf"), count_adze(&ib, "leaf"));
        prop_assert_eq!(count_adze(&ia, "skip"), count_adze(&ib, "skip"));
    }
}

// ===========================================================================
// 9. Attribute processing combined with type utilities
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// 30. Extracting inner type from a container paired with leaf attr processing.
    #[test]
    fn extract_inner_with_leaf_context(
        inner in type_name(),
        container in container_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, container, &skip(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted), inner);
    }

    /// 31. filter_inner_type strips container consistently for attr-processed types.
    #[test]
    fn filter_strips_container(inner in type_name(), container in container_name()) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let arr = [container];
        let filtered = filter_inner_type(&ty, &skip(&arr));
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    /// 32. wrap_leaf_type through a skip container wraps only the inner leaf.
    #[test]
    fn wrap_through_skip_container(inner in type_name(), container in container_name()) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let arr = [container];
        let wrapped = wrap_leaf_type(&ty, &skip(&arr));
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with(container), "expected {container}, got: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf, got: {s}");
    }

    /// 33. extract + wrap round-trip: extract inner, then wrap yields WithLeaf.
    #[test]
    fn extract_wrap_roundtrip(inner in type_name()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(inner));
    }

    /// 34. filter is idempotent: filtering twice gives same result.
    #[test]
    fn filter_idempotent(inner in type_name(), container in container_name()) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let arr = [container];
        let once = filter_inner_type(&ty, &skip(&arr));
        let twice = filter_inner_type(&once, &skip(&arr));
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    /// 35. NameValueExpr clone is equal to original (Eq contract).
    #[test]
    fn nve_clone_eq_contract(key in ident_strategy(), val in int_value()) {
        let src = format!("{key} = {val}");
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = nve.clone();
        prop_assert_eq!(nve, cloned);
    }
}
