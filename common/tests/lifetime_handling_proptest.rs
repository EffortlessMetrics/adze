#![allow(clippy::needless_range_loop)]

//! Property-based tests for lifetime parameter handling in adze-common.
//!
//! Covers: lifetime in struct definition, lifetime in enum definition,
//! lifetime parameter extraction, lifetime bounds handling, multiple
//! lifetime parameters, lifetime with generic types, lifetime in
//! generated code, and lifetime determinism.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{GenericParam, ItemEnum, ItemStruct, Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn lifetime_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{1,4}")
        .unwrap()
        .prop_filter("must be valid lifetime", |s| {
            !s.is_empty() && syn::parse_str::<syn::Lifetime>(&format!("'{s}")).is_ok()
        })
}

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn pascal_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn generic_param_name() -> impl Strategy<Value = String> {
    prop::sample::select(&["T", "U", "V", "W", "A", "B"][..]).prop_map(String::from)
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

/// Returns two distinct lifetime names from a pair of generated strings.
fn distinct_lifetime_pair() -> impl Strategy<Value = (String, String)> {
    (lifetime_name(), lifetime_name()).prop_filter("must be distinct", |(a, b)| a != b)
}

// ===========================================================================
// 1. Lifetime in struct definition
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with a single lifetime parameter parses with one generic param.
    #[test]
    fn struct_single_lifetime_param_count(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub struct {name}<'{lt}> {{ pub val: &'{lt} str, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
    }

    /// Struct lifetime parameter is a LifetimeParam variant.
    #[test]
    fn struct_lifetime_is_lifetime_param(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub struct {name}<'{lt}> {{ pub val: &'{lt} str, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let param = parsed.generics.params.first().unwrap();
        match param {
            GenericParam::Lifetime(lp) => {
                prop_assert_eq!(lp.lifetime.ident.to_string(), lt);
            }
            _ => prop_assert!(false, "expected lifetime param"),
        }
    }

    /// Struct field referencing the lifetime contains the lifetime name in tokens.
    #[test]
    fn struct_field_contains_lifetime(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub struct {name}<'{lt}> {{ pub val: &'{lt} str, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_tokens = ty_str(&parsed.fields.iter().next().unwrap().ty);
        prop_assert!(field_tokens.contains(&lt));
    }

    /// Struct with lifetime and multiple fields preserves field count.
    #[test]
    fn struct_lifetime_multi_field_count(
        name in pascal_ident(),
        lt in lifetime_name(),
        count in 2usize..=6,
    ) {
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: &'{lt} str,"))
            .collect();
        let src = format!("pub struct {name}<'{lt}> {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
    }
}

// ===========================================================================
// 2. Lifetime in enum definition
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Enum with a lifetime parameter parses with one generic param.
    #[test]
    fn enum_single_lifetime_param_count(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub enum {name}<'{lt}> {{ Borrowed(&'{lt} str), Owned(String), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
    }

    /// Enum lifetime param is extractable as LifetimeParam.
    #[test]
    fn enum_lifetime_is_lifetime_param(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub enum {name}<'{lt}> {{ A(&'{lt} str), B, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let param = parsed.generics.params.first().unwrap();
        match param {
            GenericParam::Lifetime(lp) => {
                prop_assert_eq!(lp.lifetime.ident.to_string(), lt);
            }
            _ => prop_assert!(false, "expected lifetime param"),
        }
    }

    /// Enum variant count is preserved with lifetime parameter.
    #[test]
    fn enum_lifetime_variant_count(
        name in pascal_ident(),
        lt in lifetime_name(),
        extra_variants in 0usize..=4,
    ) {
        let mut variants = vec![format!("Borrowed(&'{lt} str)")];
        for i in 0..extra_variants {
            variants.push(format!("V{i}(i32)"));
        }
        let src = format!("pub enum {name}<'{lt}> {{ {} }}", variants.join(", "));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), 1 + extra_variants);
    }
}

// ===========================================================================
// 3. Lifetime parameter extraction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Reference type with lifetime is not matched by container extraction.
    #[test]
    fn lifetime_ref_not_extracted_as_container(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("&'{lt} str")).unwrap();
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(!extracted, "reference type should not match container");
    }

    /// Option<&'a T> extracts to &'a T.
    #[test]
    fn option_of_lifetime_ref_extracts(
        lt in lifetime_name(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Option<&'{lt} {inner}>")).unwrap();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(extracted);
        let result_str = ty_str(&result);
        prop_assert!(result_str.contains(&lt), "extracted type should contain lifetime");
    }

    /// Vec<&'a T> extracts to &'a T.
    #[test]
    fn vec_of_lifetime_ref_extracts(
        lt in lifetime_name(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<&'{lt} {inner}>")).unwrap();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
        prop_assert!(extracted);
        let result_str = ty_str(&result);
        prop_assert!(result_str.contains(&lt));
    }

    /// Box<&'a str> is filterable through Box skip set, preserving lifetime.
    #[test]
    fn filter_box_preserves_lifetime_ref(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Box<&'{lt} str>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
        let s = ty_str(&filtered);
        prop_assert!(s.contains(&lt), "filtered result should preserve lifetime");
        prop_assert!(s.contains("str"));
    }
}

// ===========================================================================
// 4. Lifetime bounds handling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with lifetime bound 'a: 'b parses and preserves both lifetimes.
    #[test]
    fn struct_lifetime_bound_preserves_both(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt_a}: '{lt_b}, '{lt_b}> {{ pub val: &'{lt_a} str, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
    }

    /// Lifetime bound on first param has non-empty bounds list.
    #[test]
    fn struct_lifetime_bound_has_bounds(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt_a}: '{lt_b}, '{lt_b}> {{ pub val: &'{lt_a} str, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let param = parsed.generics.params.first().unwrap();
        if let GenericParam::Lifetime(lp) = param {
            prop_assert!(!lp.bounds.is_empty(), "lifetime should have bounds");
        } else {
            prop_assert!(false, "expected lifetime param");
        }
    }

    /// Enum with where clause lifetime bound parses correctly.
    #[test]
    fn enum_where_clause_lifetime_bound(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub enum {name}<'{lt_a}, '{lt_b}> where '{lt_a}: '{lt_b} {{ A(&'{lt_a} str), B(&'{lt_b} str), }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert!(parsed.generics.where_clause.is_some());
        prop_assert_eq!(parsed.variants.len(), 2);
    }
}

// ===========================================================================
// 5. Multiple lifetime parameters
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with two distinct lifetimes preserves both.
    #[test]
    fn struct_two_lifetimes(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt_a}, '{lt_b}> {{ pub a: &'{lt_a} str, pub b: &'{lt_b} str, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
        prop_assert_eq!(parsed.fields.len(), 2);
    }

    /// Each lifetime in a two-lifetime struct is a LifetimeParam with the correct name.
    #[test]
    fn struct_two_lifetimes_names(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt_a}, '{lt_b}> {{ pub a: &'{lt_a} str, pub b: &'{lt_b} str, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let params: Vec<String> = parsed
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Lifetime(lp) => Some(lp.lifetime.ident.to_string()),
                _ => None,
            })
            .collect();
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(&params[0], &lt_a);
        prop_assert_eq!(&params[1], &lt_b);
    }

    /// Enum with two lifetimes parses both as lifetime params.
    #[test]
    fn enum_two_lifetimes(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub enum {name}<'{lt_a}, '{lt_b}> {{ A(&'{lt_a} str), B(&'{lt_b} str), }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
    }

    /// Fields referencing different lifetimes contain their respective lifetime tokens.
    #[test]
    fn struct_fields_reference_distinct_lifetimes(
        name in pascal_ident(),
        (lt_a, lt_b) in distinct_lifetime_pair(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt_a}, '{lt_b}> {{ pub a: &'{lt_a} str, pub b: &'{lt_b} [u8], }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let fields: Vec<_> = parsed.fields.iter().collect();
        let a_tokens = ty_str(&fields[0].ty);
        let b_tokens = ty_str(&fields[1].ty);
        prop_assert!(a_tokens.contains(&lt_a));
        prop_assert!(b_tokens.contains(&lt_b));
    }
}

// ===========================================================================
// 6. Lifetime with generic types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with lifetime + type param preserves both in generics.
    #[test]
    fn struct_lifetime_plus_type_param(
        name in pascal_ident(),
        lt in lifetime_name(),
        param in generic_param_name(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt}, {param}> {{ pub r: &'{lt} str, pub v: {param}, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
        // First param should be lifetime
        match parsed.generics.params.first().unwrap() {
            GenericParam::Lifetime(lp) => prop_assert_eq!(lp.lifetime.ident.to_string(), lt),
            _ => prop_assert!(false, "first param should be lifetime"),
        }
        // Second param should be type
        match parsed.generics.params.iter().nth(1).unwrap() {
            GenericParam::Type(tp) => prop_assert_eq!(tp.ident.to_string(), param),
            _ => prop_assert!(false, "second param should be type"),
        }
    }

    /// Option<&'a T> extraction through container yields reference with lifetime.
    #[test]
    fn option_lifetime_ref_with_generic(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Option<&'{lt} str>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        let s = ty_str(&result);
        let expected_lt = format!("'{lt}");
        prop_assert!(s.contains(&expected_lt));
    }

    /// Box<Vec<&'a str>> extraction through Box skip finds Vec.
    #[test]
    fn extract_vec_through_box_with_lifetime(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Box<Vec<&'{lt} str>>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
        prop_assert!(ok);
        let s = ty_str(&result);
        prop_assert!(s.contains(&lt), "extracted inner should contain lifetime");
    }

    /// Enum with lifetime + type param preserves both.
    #[test]
    fn enum_lifetime_plus_type_param(
        name in pascal_ident(),
        lt in lifetime_name(),
        param in generic_param_name(),
    ) {
        let src = format!(
            "pub enum {name}<'{lt}, {param}> {{ Ref(&'{lt} str), Val({param}), }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
        prop_assert_eq!(parsed.variants.len(), 2);
    }
}

// ===========================================================================
// 7. Lifetime in generated code (wrap/filter)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// wrap_leaf_type on a reference with lifetime wraps the whole reference.
    #[test]
    fn wrap_lifetime_ref_wraps_entire_ref(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("&'{lt} str")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"), "should be wrapped: {s}");
    }

    /// filter_inner_type on a reference type without containers returns unchanged.
    #[test]
    fn filter_lifetime_ref_unchanged(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("&'{lt} str")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&["Box", "Arc"]));
        let s = ty_str(&filtered);
        prop_assert!(s.contains(&lt));
        prop_assert!(s.contains("str"));
    }

    /// wrap_leaf_type on Option<&'a str> with Option in skip wraps inner ref.
    #[test]
    fn wrap_option_lifetime_ref_skip(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Option<&'{lt} str>")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&["Option"]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains("WithLeaf"));
    }

    /// Wrapped lifetime reference type roundtrips through parse.
    #[test]
    fn wrap_lifetime_ref_roundtrip(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("&'{lt} str")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        let s = ty_str(&wrapped);
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }
}

// ===========================================================================
// 8. Lifetime determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing a struct with lifetime twice produces identical tokens.
    #[test]
    fn determinism_struct_lifetime_parse(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub struct {name}<'{lt}> {{ pub val: &'{lt} str, }}");
        let a: ItemStruct = parse_str(&src).unwrap();
        let b: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(
            a.to_token_stream().to_string(),
            b.to_token_stream().to_string()
        );
    }

    /// Extraction from Option<&'a str> is deterministic across runs.
    #[test]
    fn determinism_extraction_lifetime(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Option<&'{lt} str>")).unwrap();
        let run = || {
            let (result, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
            (ty_str(&result), ok)
        };
        prop_assert_eq!(run(), run());
    }

    /// wrap_leaf_type on lifetime reference is deterministic.
    #[test]
    fn determinism_wrap_lifetime(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("&'{lt} str")).unwrap();
        let run = || ty_str(&wrap_leaf_type(&ty, &skip_set(&[])));
        prop_assert_eq!(run(), run());
    }

    /// filter_inner_type on Box<&'a str> is deterministic.
    #[test]
    fn determinism_filter_lifetime(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Box<&'{lt} str>")).unwrap();
        let run = || ty_str(&filter_inner_type(&ty, &skip_set(&["Box"])));
        prop_assert_eq!(run(), run());
    }

    /// Full pipeline: extract + filter + wrap on lifetime type is deterministic.
    #[test]
    fn determinism_full_pipeline_lifetime(
        lt in lifetime_name(),
    ) {
        let ty: Type = parse_str(&format!("Option<Box<&'{lt} str>>")).unwrap();
        let run = || {
            let (extracted, _) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
            let filtered = filter_inner_type(&extracted, &skip_set(&["Box"]));
            ty_str(&wrap_leaf_type(&filtered, &skip_set(&[])))
        };
        prop_assert_eq!(run(), run());
    }
}
