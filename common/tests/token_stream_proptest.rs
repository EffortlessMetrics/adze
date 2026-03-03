#![allow(clippy::needless_range_loop)]

//! Property-based tests for TokenStream processing in adze-common.
//!
//! Exercises TokenStream parsing, roundtrip stringification, comparison,
//! empty/large streams, struct/enum definition generation, and manipulation
//! through the public API surface re-exported by adze-common.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (lowercase start, no keywords).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Simple leaf type names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&[
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
        "String", "usize", "isize",
    ][..])
}

/// Container type names.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc"][..])
}

/// Small unsigned values for repetition counts.
fn small_count() -> impl Strategy<Value = usize> {
    1usize..=20
}

/// Generate a valid Rust expression string.
fn expr_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (-1000i64..1000).prop_map(|v| v.to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
        Just("\"hello\"".to_string()),
        Just("42usize".to_string()),
    ]
}

/// Generate a field definition string like `name: Type`.
fn field_def_strategy() -> impl Strategy<Value = (String, String)> {
    (ident_strategy(), leaf_type_name()).prop_map(|(name, ty)| (name, ty.to_string()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 1. Parse valid Rust expression to TokenStream
    #[test]
    fn parse_valid_expr_to_token_stream(expr in expr_strategy()) {
        let ts: TokenStream = expr.parse().unwrap();
        prop_assert!(!ts.is_empty(), "parsed TokenStream should not be empty for: {expr}");
    }

    // 2. TokenStream to string roundtrip preserves parsability
    #[test]
    fn token_stream_string_roundtrip(expr in expr_strategy()) {
        let ts1: TokenStream = expr.parse().unwrap();
        let stringified = ts1.to_string();
        let ts2: TokenStream = stringified.parse().unwrap();
        // Both should re-stringify identically
        prop_assert_eq!(ts1.to_string(), ts2.to_string());
    }

    // 3. TokenStream comparison via string equality
    #[test]
    fn token_stream_string_equality(leaf in leaf_type_name()) {
        let ts1: TokenStream = leaf.parse().unwrap();
        let ts2: TokenStream = leaf.parse().unwrap();
        prop_assert_eq!(ts1.to_string(), ts2.to_string());
    }

    // 4. Empty TokenStream has no tokens
    #[test]
    fn empty_token_stream_is_empty(_dummy in 0u8..1) {
        let ts: TokenStream = TokenStream::new();
        prop_assert!(ts.is_empty());
        prop_assert_eq!(ts.to_string(), "");
    }

    // 5. Empty string parses to empty TokenStream
    #[test]
    fn empty_string_parses_to_empty_stream(_dummy in 0u8..1) {
        let ts: TokenStream = "".parse().unwrap();
        prop_assert!(ts.is_empty());
    }

    // 6. Large TokenStream from repeated additions
    #[test]
    fn large_token_stream_from_additions(count in 10usize..=100) {
        let mut ts = TokenStream::new();
        for i in 0..count {
            let lit: TokenStream = format!("{i}u64").parse().unwrap();
            ts.extend(lit);
        }
        let s = ts.to_string();
        // Every literal should appear in the output
        for i in 0..count {
            prop_assert!(s.contains(&format!("{i}")), "missing literal {i} in stream");
        }
    }

    // 7. Large TokenStream from struct with many fields
    #[test]
    fn large_struct_token_stream(n in 5usize..=30) {
        let fields: Vec<String> = (0..n)
            .map(|i| format!("field_{i}: u32"))
            .collect();
        let body = fields.join(", ");
        let src = format!("struct Big {{ {body} }}");
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        prop_assert!(!ts.is_empty());
        let s = ts.to_string();
        for i in 0..n {
            prop_assert!(s.contains(&format!("field_{i}")), "missing field_{i}");
        }
    }

    // 8. TokenStream from struct definition roundtrips through syn
    #[test]
    fn struct_def_roundtrip(name in ident_strategy(), fields in prop::collection::vec(field_def_strategy(), 1..=5)) {
        let field_strs: Vec<String> = fields.iter().enumerate()
            .map(|(i, (_, ty))| format!("f{i}: {ty}"))
            .collect();
        let src = format!("struct {name} {{ {} }}", field_strs.join(", "));
        // Must parse as a valid item
        let upper_name = {
            let mut c = name.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        let src = format!("struct {upper_name} {{ {} }}", field_strs.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 9. TokenStream from enum definition roundtrips through syn
    #[test]
    fn enum_def_roundtrip(
        name in ident_strategy(),
        variant_count in 1usize..=8,
    ) {
        let upper_name = {
            let mut c = name.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        let variants: Vec<String> = (0..variant_count)
            .map(|i| format!("V{i}"))
            .collect();
        let src = format!("enum {upper_name} {{ {} }}", variants.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 10. Enum with tuple variants roundtrips
    #[test]
    fn enum_tuple_variants_roundtrip(variant_count in 1usize..=6) {
        let variants: Vec<String> = (0..variant_count)
            .map(|i| format!("V{i}(i32)"))
            .collect();
        let src = format!("enum MyEnum {{ {} }}", variants.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 11. Enum with struct variants roundtrips
    #[test]
    fn enum_struct_variants_roundtrip(variant_count in 1usize..=4) {
        let variants: Vec<String> = (0..variant_count)
            .map(|i| format!("V{i} {{ x: u32, y: i64 }}"))
            .collect();
        let src = format!("enum Foo {{ {} }}", variants.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 12. TokenStream extend preserves order
    #[test]
    fn extend_preserves_order(count in small_count()) {
        let mut ts = TokenStream::new();
        let mut expected_parts = Vec::new();
        for i in 0..count {
            let part: TokenStream = format!("item_{i}").parse().unwrap();
            expected_parts.push(format!("item_{i}"));
            ts.extend(part);
        }
        let s = ts.to_string();
        // Verify ordering: each item_i appears before item_{i+1}
        for i in 0..count.saturating_sub(1) {
            let pos_a = s.find(&expected_parts[i]).unwrap();
            let pos_b = s.find(&expected_parts[i + 1]).unwrap();
            prop_assert!(pos_a < pos_b, "item_{i} should appear before item_{}", i + 1);
        }
    }

    // 13. quote! macro produces non-empty TokenStream
    #[test]
    fn quote_produces_nonempty(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let ts = quote! { let x: #ty = Default::default(); };
        prop_assert!(!ts.is_empty());
        prop_assert!(ts.to_string().contains(leaf));
    }

    // 14. quote! with interpolated identifiers roundtrips
    #[test]
    fn quote_ident_interpolation(name in ident_strategy()) {
        let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
        let ts = quote! { fn #ident() {} };
        let item: syn::Item = syn::parse2(ts).unwrap();
        let s = item.to_token_stream().to_string();
        prop_assert!(s.contains(&name));
    }

    // 15. Type TokenStream preserves leaf type name
    #[test]
    fn type_to_token_stream_preserves_name(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let ts = ty.to_token_stream();
        prop_assert_eq!(ts.to_string(), leaf);
    }

    // 16. Container type TokenStream contains inner type
    #[test]
    fn container_token_stream_contains_inner(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let src = format!("{container}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let s = ty.to_token_stream().to_string();
        prop_assert!(s.contains(container));
        prop_assert!(s.contains(inner));
    }

    // 17. wrap_leaf_type output TokenStream always contains "WithLeaf" for leaves
    #[test]
    fn wrap_leaf_token_stream_has_with_leaf(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ts = wrapped.to_token_stream();
        prop_assert!(ts.to_string().contains("WithLeaf"));
    }

    // 18. filter_inner_type TokenStream equals original for non-skip types
    #[test]
    fn filter_token_stream_identity_for_non_skip(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            ty.to_token_stream().to_string(),
            filtered.to_token_stream().to_string()
        );
    }

    // 19. try_extract produces TokenStream that parses as a valid Type
    #[test]
    fn extract_produces_valid_type_token_stream(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let src = format!("{container}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, _) = try_extract_inner_type(&ty, container, &skip);
        let ts = result.to_token_stream();
        // The resulting TokenStream should parse back as a valid Type
        let _reparsed: Type = syn::parse2(ts).unwrap();
    }

    // 20. TokenStream from nested containers preserves structure
    #[test]
    fn nested_container_token_stream(inner in leaf_type_name()) {
        let src = format!("Vec<Option<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let ts = ty.to_token_stream();
        let s = ts.to_string();
        prop_assert!(s.contains("Vec"));
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains(inner));
    }

    // 21. Multiple TokenStreams from different types are distinct
    #[test]
    fn distinct_types_produce_distinct_streams(
        a in leaf_type_name(),
        b in leaf_type_name(),
    ) {
        prop_assume!(a != b);
        let ts_a: TokenStream = a.parse().unwrap();
        let ts_b: TokenStream = b.parse().unwrap();
        prop_assert_ne!(ts_a.to_string(), ts_b.to_string());
    }

    // 22. TokenStream concatenation via quote splice
    #[test]
    fn quote_splice_concatenation(
        a in leaf_type_name(),
        b in leaf_type_name(),
    ) {
        let ty_a: Type = parse_str(a).unwrap();
        let ty_b: Type = parse_str(b).unwrap();
        let combined = quote! { (#ty_a, #ty_b) };
        let s = combined.to_string();
        prop_assert!(s.contains(a));
        prop_assert!(s.contains(b));
    }

    // 23. TokenStream from parse_quote matches manual construction
    #[test]
    fn parse_quote_matches_manual(leaf in leaf_type_name()) {
        let ident = syn::Ident::new(leaf, proc_macro2::Span::call_site());
        let from_parse_quote: Type = parse_quote!(#ident);
        let from_parse_str: Type = parse_str(leaf).unwrap();
        prop_assert_eq!(
            from_parse_quote.to_token_stream().to_string(),
            from_parse_str.to_token_stream().to_string()
        );
    }

    // 24. Large enum with many variants produces correct TokenStream
    #[test]
    fn large_enum_token_stream(n in 10usize..=50) {
        let variants: Vec<String> = (0..n).map(|i| format!("Variant{i}")).collect();
        let src = format!("enum BigEnum {{ {} }}", variants.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let s = ts.to_string();
        for i in 0..n {
            prop_assert!(s.contains(&format!("Variant{i}")), "missing Variant{i}");
        }
    }

    // 25. TokenStream iteration count matches token count
    #[test]
    fn token_stream_iteration(count in 1usize..=10) {
        let parts: Vec<String> = (0..count).map(|i| format!("{i}u32")).collect();
        let src = parts.join(" + ");
        let ts: TokenStream = src.parse().unwrap();
        // Should have at least `count` token trees (literals) plus operators
        let tree_count = ts.into_iter().count();
        prop_assert!(tree_count >= count, "expected at least {count} trees, got {tree_count}");
    }

    // 26. Struct with generic parameters roundtrips
    #[test]
    fn generic_struct_roundtrip(field_count in 1usize..=4) {
        let fields: Vec<String> = (0..field_count)
            .map(|i| format!("f{i}: T"))
            .collect();
        let src = format!("struct Gen<T> {{ {} }}", fields.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 27. TokenStream from function signature is parsable
    #[test]
    fn fn_signature_token_stream(
        name in ident_strategy(),
        ret in leaf_type_name(),
    ) {
        let src = format!("fn {name}() -> {ret} {{ todo!() }}");
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 28. TokenStream clone equality
    #[test]
    fn token_stream_clone_equality(leaf in leaf_type_name()) {
        let ts: TokenStream = leaf.parse().unwrap();
        let cloned = ts.clone();
        prop_assert_eq!(ts.to_string(), cloned.to_string());
    }

    // 29. Wrapped type TokenStream parses as valid Type
    #[test]
    fn wrapped_type_parses_back(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ts = wrapped.to_token_stream();
        let _reparsed: Type = syn::parse2(ts).unwrap();
    }

    // 30. TokenStream from impl block roundtrips
    #[test]
    fn impl_block_roundtrip(method_count in 1usize..=4) {
        let methods: Vec<String> = (0..method_count)
            .map(|i| format!("fn m{i}(&self) {{}}"))
            .collect();
        let src = format!("impl Foo {{ {} }}", methods.join(" "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    // 31. Filtered container type TokenStream parses as valid Type
    #[test]
    fn filtered_container_parses_back(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        let ts = filtered.to_token_stream();
        let _reparsed: Type = syn::parse2(ts).unwrap();
    }

    // 32. Repeated wrap+filter cycle stays consistent
    #[test]
    fn wrap_filter_cycle_consistency(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip_wrap: HashSet<&str> = HashSet::new();
        let skip_filter: HashSet<&str> = HashSet::new();
        // Wrap and then filter should produce a valid TokenStream each time
        let wrapped = wrap_leaf_type(&ty, &skip_wrap);
        let _ = filter_inner_type(&wrapped, &skip_filter);
        // Second pass should be identical
        let wrapped2 = wrap_leaf_type(&ty, &skip_wrap);
        prop_assert_eq!(
            wrapped.to_token_stream().to_string(),
            wrapped2.to_token_stream().to_string()
        );
    }

    // 33. TokenStream from type alias roundtrips
    #[test]
    fn type_alias_roundtrip(name in ident_strategy(), target in leaf_type_name()) {
        let upper_name = {
            let mut c = name.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        let src = format!("type {upper_name} = {target};");
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }
}
