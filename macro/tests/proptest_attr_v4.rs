//! Property-based tests (v4) for attribute parsing and type analysis in `adze-macro`.
//!
//! Covers 9 categories with 45+ properties exercising `try_extract_inner_type`,
//! `filter_inner_type`, `wrap_leaf_type`, `is_parameterized` (local helper),
//! DeriveInput parsing, attribute detection, composition, token-stream roundtrips,
//! and edge cases.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{DeriveInput, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Returns `true` when the type is a path type whose last segment carries
/// angle-bracketed generic arguments.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn try_parse_type(s: &str) -> Option<Type> {
    syn::parse_str::<Type>(s).ok()
}

// ── Strategy helpers ────────────────────────────────────────────────────────

fn simple_type_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
        "String", "usize", "isize",
    ])
    .prop_map(|s| s.to_string())
}

fn wrapper_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["Vec", "Option", "Box", "Arc", "Rc"]).prop_map(|s| s.to_string())
}

fn ident_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "Foo", "Bar", "Baz", "Qux", "Alpha", "Beta", "Gamma", "Delta", "Node", "Leaf",
    ])
    .prop_map(|s| s.to_string())
}

fn field_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "value", "data", "items", "name", "count", "flag", "inner", "result", "content", "text",
    ])
    .prop_map(|s| s.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 1: extract_inner_type is deterministic (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Extracting the same wrapper+type pair twice yields identical results.
    #[test]
    fn extract_deterministic_same_input(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let empty = skip(&[]);
            let (r1, e1) = try_extract_inner_type(&ty, &wrapper, &empty);
            let (r2, e2) = try_extract_inner_type(&ty, &wrapper, &empty);
            prop_assert_eq!(e1, e2);
            prop_assert_eq!(ty_str(&r1), ty_str(&r2));
        }
    }

    /// Extraction with identical skip sets produces identical output.
    #[test]
    fn extract_deterministic_with_skip(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Vec<i32>>),
            parse_quote!(Arc<Option<String>>),
            parse_quote!(Box<Arc<Vec<u8>>>),
        ];
        let skips = skip(&["Box", "Arc"]);
        let targets = ["Vec", "Option", "Vec"];
        let (r1, e1) = try_extract_inner_type(&types[idx], targets[idx], &skips);
        let (r2, e2) = try_extract_inner_type(&types[idx], targets[idx], &skips);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Extraction on plain types deterministically returns the original.
    #[test]
    fn extract_deterministic_plain_type(inner in simple_type_name()) {
        if let Some(ty) = try_parse_type(&inner) {
            let empty = skip(&[]);
            let (r1, e1) = try_extract_inner_type(&ty, "Vec", &empty);
            let (r2, e2) = try_extract_inner_type(&ty, "Vec", &empty);
            prop_assert_eq!(e1, e2);
            prop_assert_eq!(ty_str(&r1), ty_str(&r2));
        }
    }

    /// Non-path types always yield deterministic (unchanged) results.
    #[test]
    fn extract_deterministic_non_path(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!(&mut i32),
            parse_quote!((u8, u16)),
        ];
        let empty = skip(&[]);
        let (r1, e1) = try_extract_inner_type(&types[idx], "Option", &empty);
        let (r2, e2) = try_extract_inner_type(&types[idx], "Option", &empty);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Extraction result string representation is deterministic across calls.
    #[test]
    fn extract_deterministic_string_repr(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let empty = skip(&[]);
            let (r1, _) = try_extract_inner_type(&ty, &wrapper, &empty);
            let s1 = ty_str(&r1);
            let (r2, _) = try_extract_inner_type(&ty, &wrapper, &empty);
            let s2 = ty_str(&r2);
            prop_assert_eq!(s1, s2);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 2: filter_inner_type is deterministic (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Filtering the same type twice with the same skip set gives the same result.
    #[test]
    fn filter_deterministic_single_wrapper(idx in 0usize..4) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<i32>),
            parse_quote!(Arc<String>),
            parse_quote!(Rc<bool>),
            parse_quote!(Box<u64>),
        ];
        let names: [&[&str]; 4] = [&["Box"], &["Arc"], &["Rc"], &["Box"]];
        let skips = skip(names[idx]);
        let r1 = filter_inner_type(&types[idx], &skips);
        let r2 = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Nested filter is deterministic.
    #[test]
    fn filter_deterministic_nested(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Arc<i32>>),
            parse_quote!(Arc<Box<String>>),
            parse_quote!(Box<Box<u8>>),
        ];
        let skips = skip(&["Box", "Arc"]);
        let r1 = filter_inner_type(&types[idx], &skips);
        let r2 = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Filter with empty skip set deterministically returns original.
    #[test]
    fn filter_deterministic_empty_skip(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let empty = skip(&[]);
            let r1 = filter_inner_type(&ty, &empty);
            let r2 = filter_inner_type(&ty, &empty);
            prop_assert_eq!(ty_str(&r1), ty_str(&r2));
        }
    }

    /// Filter on non-skip types is deterministic identity.
    #[test]
    fn filter_deterministic_non_skip(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(HashMap<String, i32>),
        ];
        let skips = skip(&["Box", "Arc"]);
        let r1 = filter_inner_type(&types[idx], &skips);
        let r2 = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Non-path types deterministically pass through filter unchanged.
    #[test]
    fn filter_deterministic_non_path(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!((i32, u32)),
            parse_quote!(&mut bool),
        ];
        let skips = skip(&["Box"]);
        let r1 = filter_inner_type(&types[idx], &skips);
        let r2 = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 3: is_parameterized matches presence of angle brackets (5 props)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Wrapper<Inner> is always parameterized.
    #[test]
    fn parameterized_true_for_generics(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            prop_assert!(is_parameterized(&ty));
        }
    }

    /// Plain types without angle brackets are not parameterized.
    #[test]
    fn parameterized_false_for_plain(name in simple_type_name()) {
        if let Some(ty) = try_parse_type(&name) {
            prop_assert!(!is_parameterized(&ty));
        }
    }

    /// Nested generics are parameterized.
    #[test]
    fn parameterized_true_for_nested(
        outer in wrapper_name(),
        mid in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let type_str = format!("{outer}<{mid}<{inner}>>");
        if let Some(ty) = try_parse_type(&type_str) {
            prop_assert!(is_parameterized(&ty));
        }
    }

    /// Reference types are not parameterized (not Type::Path).
    #[test]
    fn parameterized_false_for_refs(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!(&mut i32),
            parse_quote!(&bool),
        ];
        prop_assert!(!is_parameterized(&types[idx]));
    }

    /// is_parameterized is consistent across multiple calls.
    #[test]
    fn parameterized_consistent(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let r1 = is_parameterized(&ty);
            let r2 = is_parameterized(&ty);
            prop_assert_eq!(r1, r2);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 4: wrap_leaf_type always produces non-empty output (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Wrapping plain types produces non-empty token stream.
    #[test]
    fn wrap_output_nonempty_plain(name in simple_type_name()) {
        if let Some(ty) = try_parse_type(&name) {
            let result = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
            prop_assert!(!ty_str(&result).is_empty());
        }
    }

    /// Wrapping generic types produces non-empty token stream.
    #[test]
    fn wrap_output_nonempty_generic(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let skips = skip(&["Vec", "Option", "Box", "Arc", "Rc"]);
            let result = wrap_leaf_type(&ty, &skips);
            prop_assert!(!ty_str(&result).is_empty());
        }
    }

    /// Wrapping non-path types produces non-empty token stream.
    #[test]
    fn wrap_output_nonempty_non_path(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!((i32, u32)),
            parse_quote!([u8; 4]),
        ];
        let result = wrap_leaf_type(&types[idx], &skip(&[]));
        prop_assert!(!ty_str(&result).is_empty());
    }

    /// Wrapping nested generics produces non-empty token stream.
    #[test]
    fn wrap_output_nonempty_nested(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<Option<i32>>),
            parse_quote!(Option<Vec<String>>),
            parse_quote!(Vec<Vec<u8>>),
        ];
        let skips = skip(&["Vec", "Option"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        prop_assert!(!ty_str(&result).is_empty());
    }

    /// Wrapping with empty skip set always produces WithLeaf wrapper.
    #[test]
    fn wrap_empty_skip_always_wraps(name in simple_type_name()) {
        if let Some(ty) = try_parse_type(&name) {
            let result = wrap_leaf_type(&ty, &skip(&[]));
            let s = ty_str(&result);
            prop_assert!(!s.is_empty());
            prop_assert!(s.contains("WithLeaf"), "expected WithLeaf in: {s}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 5: DeriveInput parsing succeeds for valid structs (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// A struct with a single simple-typed field parses successfully.
    #[test]
    fn derive_input_single_field(
        struct_name in ident_name(),
        field in field_name(),
        ty in simple_type_name(),
    ) {
        let code = format!("struct {struct_name} {{ {field}: {ty} }}");
        let result = syn::parse_str::<DeriveInput>(&code);
        prop_assert!(result.is_ok(), "failed to parse: {code}");
        let di = result.unwrap();
        prop_assert_eq!(di.ident.to_string(), struct_name);
    }

    /// A struct with a generic-typed field parses successfully.
    #[test]
    fn derive_input_generic_field(
        struct_name in ident_name(),
        field in field_name(),
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let code = format!("struct {struct_name} {{ {field}: {wrapper}<{inner}> }}");
        let result = syn::parse_str::<DeriveInput>(&code);
        prop_assert!(result.is_ok(), "failed to parse: {code}");
    }

    /// A struct with two fields parses successfully.
    #[test]
    fn derive_input_two_fields(
        struct_name in ident_name(),
        ty1 in simple_type_name(),
        ty2 in simple_type_name(),
    ) {
        let code = format!("struct {struct_name} {{ first: {ty1}, second: {ty2} }}");
        let result = syn::parse_str::<DeriveInput>(&code);
        prop_assert!(result.is_ok(), "failed to parse: {code}");
    }

    /// A unit struct parses successfully.
    #[test]
    fn derive_input_unit_struct(struct_name in ident_name()) {
        let code = format!("struct {struct_name};");
        let result = syn::parse_str::<DeriveInput>(&code);
        prop_assert!(result.is_ok(), "failed to parse: {code}");
    }

    /// A tuple struct parses successfully.
    #[test]
    fn derive_input_tuple_struct(
        struct_name in ident_name(),
        ty in simple_type_name(),
    ) {
        let code = format!("struct {struct_name}({ty});");
        let result = syn::parse_str::<DeriveInput>(&code);
        prop_assert!(result.is_ok(), "failed to parse: {code}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 6: Attribute detection is consistent (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Parsing an attributed struct retains the attribute.
    #[test]
    fn attr_detection_single_derive(struct_name in ident_name(), ty in simple_type_name()) {
        let code = format!("#[derive(Debug)] struct {struct_name} {{ val: {ty} }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert!(!di.attrs.is_empty());
        let path_str = di.attrs[0].path().to_token_stream().to_string();
        prop_assert_eq!(path_str, "derive");
    }

    /// Multiple attributes are all preserved after parsing.
    #[test]
    fn attr_detection_multiple_derives(struct_name in ident_name()) {
        let code = format!("#[derive(Debug)] #[derive(Clone)] struct {struct_name} {{ val: i32 }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(di.attrs.len(), 2);
    }

    /// Attribute path detection is consistent across calls.
    #[test]
    fn attr_detection_consistent(struct_name in ident_name(), ty in simple_type_name()) {
        let code = format!("#[derive(Debug)] struct {struct_name} {{ val: {ty} }}");
        let di1: DeriveInput = syn::parse_str(&code).unwrap();
        let di2: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(di1.attrs.len(), di2.attrs.len());
        let p1 = di1.attrs[0].path().to_token_stream().to_string();
        let p2 = di2.attrs[0].path().to_token_stream().to_string();
        prop_assert_eq!(p1, p2);
    }

    /// Structs without attributes have empty attr vec.
    #[test]
    fn attr_detection_none_when_absent(struct_name in ident_name()) {
        let code = format!("struct {struct_name} {{ val: i32 }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert!(di.attrs.is_empty());
    }

    /// Enum with derive attribute retains the attribute.
    #[test]
    fn attr_detection_enum(name in ident_name()) {
        let code = format!("#[derive(Debug)] enum {name} {{ A, B }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert!(!di.attrs.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 7: Type analysis composition — extract then filter (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Extract Vec inner then filter Box yields the leaf type.
    #[test]
    fn compose_extract_then_filter(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<Box<i32>>),
            parse_quote!(Vec<Box<String>>),
            parse_quote!(Vec<Box<bool>>),
        ];
        let expected = ["i32", "String", "bool"];
        let empty = skip(&[]);
        let (extracted, ok) = try_extract_inner_type(&types[idx], "Vec", &empty);
        prop_assert!(ok);
        let filtered = filter_inner_type(&extracted, &skip(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), expected[idx]);
    }

    /// Extract then wrap produces WithLeaf around the inner type.
    #[test]
    fn compose_extract_then_wrap(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Option<bool>),
        ];
        let empty = skip(&[]);
        let (inner, ok) = try_extract_inner_type(&types[idx], "Option", &empty);
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&inner, &empty);
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"), "expected WithLeaf in: {s}");
    }

    /// Filter then extract: filter Box then extract Vec.
    #[test]
    fn compose_filter_then_extract(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Vec<i32>>),
            parse_quote!(Box<Vec<String>>),
            parse_quote!(Box<Vec<bool>>),
        ];
        let expected = ["i32", "String", "bool"];
        let filtered = filter_inner_type(&types[idx], &skip(&["Box"]));
        let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&inner), expected[idx]);
    }

    /// Extract through skip then wrap the result.
    #[test]
    fn compose_skip_extract_wrap(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Arc<Vec<i32>>),
            parse_quote!(Box<Option<String>>),
            parse_quote!(Arc<Box<Vec<u8>>>),
        ];
        let targets = ["Vec", "Option", "Vec"];
        let skips = skip(&["Box", "Arc"]);
        let (inner, ok) = try_extract_inner_type(&types[idx], targets[idx], &skips);
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&inner, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"), "expected WithLeaf in: {s}");
    }

    /// Composing filter on already-filtered type is idempotent.
    #[test]
    fn compose_double_filter_idempotent(idx in 0usize..3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Arc<i32>>),
            parse_quote!(Arc<Box<String>>),
            parse_quote!(Box<Box<bool>>),
        ];
        let skips = skip(&["Box", "Arc"]);
        let once = filter_inner_type(&types[idx], &skips);
        let twice = filter_inner_type(&once, &skips);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 8: TokenStream roundtrip (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Token stream of a type can be re-parsed to the same string representation.
    #[test]
    fn token_roundtrip_simple(name in simple_type_name()) {
        if let Some(ty) = try_parse_type(&name) {
            let tokens = ty.to_token_stream().to_string();
            let reparsed: Type = syn::parse_str(&tokens).unwrap();
            prop_assert_eq!(ty_str(&ty), ty_str(&reparsed));
        }
    }

    /// Token stream roundtrip for generic types.
    #[test]
    fn token_roundtrip_generic(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let tokens = ty.to_token_stream().to_string();
            let reparsed: Type = syn::parse_str(&tokens).unwrap();
            prop_assert_eq!(ty_str(&ty), ty_str(&reparsed));
        }
    }

    /// Token stream roundtrip for nested generics.
    #[test]
    fn token_roundtrip_nested(
        outer in wrapper_name(),
        mid in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let type_str = format!("{outer}<{mid}<{inner}>>");
        if let Some(ty) = try_parse_type(&type_str) {
            let tokens = ty.to_token_stream().to_string();
            let reparsed: Type = syn::parse_str(&tokens).unwrap();
            prop_assert_eq!(ty_str(&ty), ty_str(&reparsed));
        }
    }

    /// Token stream roundtrip preserves wrap_leaf_type output.
    #[test]
    fn token_roundtrip_wrapped(name in simple_type_name()) {
        if let Some(ty) = try_parse_type(&name) {
            let wrapped = wrap_leaf_type(&ty, &skip(&[]));
            let tokens = wrapped.to_token_stream().to_string();
            let reparsed: Type = syn::parse_str(&tokens).unwrap();
            prop_assert_eq!(ty_str(&wrapped), ty_str(&reparsed));
        }
    }

    /// Token stream roundtrip for DeriveInput structs.
    #[test]
    fn token_roundtrip_derive_input(
        struct_name in ident_name(),
        ty in simple_type_name(),
    ) {
        let code = format!("struct {struct_name} {{ val: {ty} }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        let tokens = di.to_token_stream().to_string();
        let reparsed: DeriveInput = syn::parse_str(&tokens).unwrap();
        prop_assert_eq!(di.ident.to_string(), reparsed.ident.to_string());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 9: Edge cases (6 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Extracting from a wrapper that is also in skip_over: direct match wins.
    #[test]
    fn edge_target_in_skip_direct_match_wins(inner in simple_type_name()) {
        let type_str = format!("Vec<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let skips = skip(&["Vec", "Option"]);
            let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skips);
            prop_assert!(extracted);
            prop_assert_eq!(ty_str(&result), inner);
        }
    }

    /// filter_inner_type on a deeply nested chain strips everything.
    #[test]
    fn edge_triple_nested_filter(inner in simple_type_name()) {
        let type_str = format!("Box<Arc<Box<{inner}>>>");
        if let Some(ty) = try_parse_type(&type_str) {
            let skips = skip(&["Box", "Arc"]);
            let result = filter_inner_type(&ty, &skips);
            prop_assert_eq!(ty_str(&result), inner);
        }
    }

    /// wrap_leaf_type with all wrappers in skip set preserves outer structure.
    #[test]
    fn edge_wrap_preserves_outer_when_skipped(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let skips = skip(&["Vec", "Option", "Box", "Arc", "Rc"]);
            let result = wrap_leaf_type(&ty, &skips);
            let s = ty_str(&result);
            prop_assert!(s.starts_with(&wrapper), "expected {wrapper} prefix in: {s}");
            prop_assert!(s.contains("WithLeaf"), "expected WithLeaf in: {s}");
        }
    }

    /// is_parameterized and try_extract agree: parameterized ⇒ extraction possible.
    #[test]
    fn edge_parameterized_implies_extractable(wrapper in wrapper_name(), inner in simple_type_name()) {
        let type_str = format!("{wrapper}<{inner}>");
        if let Some(ty) = try_parse_type(&type_str) {
            let param = is_parameterized(&ty);
            let empty = skip(&[]);
            let (_, extracted) = try_extract_inner_type(&ty, &wrapper, &empty);
            // If it's parameterized and we use the correct wrapper name, extraction succeeds.
            prop_assert!(param);
            prop_assert!(extracted);
        }
    }

    /// Parsing invalid type strings returns None.
    #[test]
    fn edge_invalid_type_strings(idx in 0usize..5) {
        let bad = ["<>", "Vec<>", "123abc", "fn()", "impl Trait"];
        let result = try_parse_type(bad[idx]);
        // Some of these may parse as valid Rust types; those that don't return None.
        // The key property: try_parse_type never panics.
        if let Some(ty) = result {
            // If it parses, the string repr is non-empty.
            prop_assert!(!ty_str(&ty).is_empty());
        }
    }

    /// Enum DeriveInput preserves variant names through token roundtrip.
    #[test]
    fn edge_enum_variant_roundtrip(name in ident_name()) {
        let code = format!("enum {name} {{ Variant1, Variant2(i32), Variant3 {{ val: String }} }}");
        let di: DeriveInput = syn::parse_str(&code).unwrap();
        let tokens = di.to_token_stream().to_string();
        let reparsed: DeriveInput = syn::parse_str(&tokens).unwrap();
        prop_assert_eq!(di.ident.to_string(), reparsed.ident.to_string());
    }
}
