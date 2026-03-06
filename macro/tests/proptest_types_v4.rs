#![allow(clippy::needless_range_loop)]

//! Property-based tests (v4) for `adze-common` type utilities used by `adze-macro`.
//!
//! Covers: `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! and a local `is_parameterized` helper. Exercises type extraction properties,
//! wrapping invariants, parameterized detection, and token-stream roundtrips.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Type, parse_quote};

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

/// Parse a type string, returning None on failure.
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

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: try_extract_inner_type properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. Extraction succeeds for matching wrapper ─────────────────────────────

proptest! {
    #[test]
    fn extract_succeeds_for_matching_wrapper(idx in 0usize..=4) {
        let wrappers = ["Vec", "Option", "Box", "Arc", "Rc"];
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Box<bool>),
            parse_quote!(Arc<u64>),
            parse_quote!(Rc<f32>),
        ];
        let empty = skip(&[]);
        let (_, extracted) = try_extract_inner_type(&types[idx], wrappers[idx], &empty);
        prop_assert!(extracted);
    }
}

// ── 2. Extraction returns correct inner type ────────────────────────────────

proptest! {
    #[test]
    fn extract_returns_correct_inner(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Box<u8>),
            parse_quote!(Arc<char>),
        ];
        let wrappers = ["Vec", "Option", "Box", "Arc"];
        let expected = ["i32", "String", "u8", "char"];
        let empty = skip(&[]);
        let (inner, _) = try_extract_inner_type(&types[idx], wrappers[idx], &empty);
        prop_assert_eq!(ty_str(&inner), expected[idx]);
    }
}

// ── 3. Extraction fails for non-matching wrapper ────────────────────────────

proptest! {
    #[test]
    fn extract_fails_for_non_matching_wrapper(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(String),
            parse_quote!(i32),
        ];
        let empty = skip(&[]);
        let (_, extracted) = try_extract_inner_type(&types[idx], "HashMap", &empty);
        prop_assert!(!extracted);
    }
}

// ── 4. Extraction through skip-over wrappers ────────────────────────────────

proptest! {
    #[test]
    fn extract_through_skip_over(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Vec<i32>>),
            parse_quote!(Arc<Option<String>>),
            parse_quote!(Box<Arc<Vec<u8>>>),
        ];
        let targets = ["Vec", "Option", "Vec"];
        let expected = ["i32", "String", "u8"];
        let skips = skip(&["Box", "Arc"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], targets[idx], &skips);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), expected[idx]);
    }
}

// ── 5. Non-path types return unchanged ──────────────────────────────────────

proptest! {
    #[test]
    fn extract_non_path_returns_unchanged(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!(&mut i32),
            parse_quote!((i32, u32)),
        ];
        let empty = skip(&[]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], "Vec", &empty);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&types[idx]));
    }
}

// ── 6. Extraction idempotent on plain types ─────────────────────────────────

proptest! {
    #[test]
    fn extract_idempotent_on_plain_types(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(f64),
            parse_quote!(usize),
        ];
        let empty = skip(&[]);
        let (result, extracted) = try_extract_inner_type(&types[idx], "Option", &empty);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&types[idx]));
    }
}

// ── 7. Double extraction strips both layers ─────────────────────────────────

proptest! {
    #[test]
    fn double_extraction_strips_both(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<Option<i32>>),
            parse_quote!(Vec<Option<String>>),
            parse_quote!(Vec<Option<bool>>),
        ];
        let expected = ["i32", "String", "bool"];
        let empty = skip(&[]);
        let (mid, ok1) = try_extract_inner_type(&types[idx], "Vec", &empty);
        let (inner, ok2) = try_extract_inner_type(&mid, "Option", &empty);
        prop_assert!(ok1);
        prop_assert!(ok2);
        prop_assert_eq!(ty_str(&inner), expected[idx]);
    }
}

// ── 8. Extraction with skip_over containing the target returns extracted ────

proptest! {
    #[test]
    fn extract_skip_over_does_not_interfere_with_target(idx in 0usize..=2) {
        // skip_over only affects wrapper resolution, not the target itself
        let types: Vec<Type> = vec![
            parse_quote!(Vec<u32>),
            parse_quote!(Option<i64>),
            parse_quote!(Box<f64>),
        ];
        let wrappers = ["Vec", "Option", "Box"];
        let expected = ["u32", "i64", "f64"];
        // target itself is also in skip, but direct match takes precedence
        let skips = skip(&["Vec", "Option", "Box"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], wrappers[idx], &skips);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), expected[idx]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: filter_inner_type properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. filter_inner_type strips single wrapper ──────────────────────────────

proptest! {
    #[test]
    fn filter_strips_single_wrapper(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<i32>),
            parse_quote!(Arc<String>),
            parse_quote!(Rc<bool>),
            parse_quote!(Box<u64>),
        ];
        let expected = ["i32", "String", "bool", "u64"];
        let names = [&["Box"][..], &["Arc"], &["Rc"], &["Box"]];
        let result = filter_inner_type(&types[idx], &skip(names[idx]));
        prop_assert_eq!(ty_str(&result), expected[idx]);
    }
}

// ── 10. filter_inner_type strips nested wrappers ────────────────────────────

proptest! {
    #[test]
    fn filter_strips_nested_wrappers(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Arc<i32>>),
            parse_quote!(Arc<Box<String>>),
            parse_quote!(Box<Box<u8>>),
        ];
        let expected = ["i32", "String", "u8"];
        let skips = skip(&["Box", "Arc"]);
        let result = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), expected[idx]);
    }
}

// ── 11. filter_inner_type no-op on non-skip types ───────────────────────────

proptest! {
    #[test]
    fn filter_noop_on_non_skip_types(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(HashMap<String, i32>),
            parse_quote!(i32),
            parse_quote!(String),
        ];
        let skips = skip(&["Box", "Arc"]);
        let result = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), ty_str(&types[idx]));
    }
}

// ── 12. filter with empty skip set is identity ──────────────────────────────

proptest! {
    #[test]
    fn filter_empty_skip_is_identity(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Option<bool>),
            parse_quote!(i32),
            parse_quote!(String),
        ];
        let empty = skip(&[]);
        let result = filter_inner_type(&types[idx], &empty);
        prop_assert_eq!(ty_str(&result), ty_str(&types[idx]));
    }
}

// ── 13. filter on non-path types returns unchanged ──────────────────────────

proptest! {
    #[test]
    fn filter_non_path_returns_unchanged(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!((i32, u32)),
            parse_quote!(&mut bool),
        ];
        let skips = skip(&["Box", "Arc"]);
        let result = filter_inner_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), ty_str(&types[idx]));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: wrap_leaf_type properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 14. wrap_leaf_type wraps plain types ─────────────────────────────────────

proptest! {
    #[test]
    fn wrap_wraps_plain_types(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(u64),
            parse_quote!(char),
        ];
        let expected = [
            "adze :: WithLeaf < i32 >",
            "adze :: WithLeaf < String >",
            "adze :: WithLeaf < bool >",
            "adze :: WithLeaf < u64 >",
            "adze :: WithLeaf < char >",
        ];
        let skips = skip(&["Vec", "Option"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), expected[idx]);
    }
}

// ── 15. wrap_leaf_type preserves skip wrappers, wraps inner ─────────────────

proptest! {
    #[test]
    fn wrap_preserves_skip_wraps_inner(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Vec<bool>),
            parse_quote!(Option<u8>),
        ];
        let expected = [
            "Vec < adze :: WithLeaf < i32 > >",
            "Option < adze :: WithLeaf < String > >",
            "Vec < adze :: WithLeaf < bool > >",
            "Option < adze :: WithLeaf < u8 > >",
        ];
        let skips = skip(&["Vec", "Option"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), expected[idx]);
    }
}

// ── 16. wrap_leaf_type handles nested skip wrappers ─────────────────────────

proptest! {
    #[test]
    fn wrap_handles_nested_skips(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<Option<i32>>),
            parse_quote!(Option<Vec<String>>),
            parse_quote!(Vec<Vec<u8>>),
        ];
        let expected = [
            "Vec < Option < adze :: WithLeaf < i32 > > >",
            "Option < Vec < adze :: WithLeaf < String > > >",
            "Vec < Vec < adze :: WithLeaf < u8 > > >",
        ];
        let skips = skip(&["Vec", "Option"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        prop_assert_eq!(ty_str(&result), expected[idx]);
    }
}

// ── 17. wrap_leaf_type wraps non-path types ─────────────────────────────────

proptest! {
    #[test]
    fn wrap_wraps_non_path_types(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!((i32, u32)),
            parse_quote!([u8; 4]),
        ];
        let skips = skip(&["Vec"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        let s = ty_str(&result);
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf wrapper in: {}", s);
    }
}

// ── 18. wrap with empty skip wraps everything ───────────────────────────────

proptest! {
    #[test]
    fn wrap_empty_skip_wraps_everything(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Box<bool>),
            parse_quote!(i32),
        ];
        let empty = skip(&[]);
        let result = wrap_leaf_type(&types[idx], &empty);
        let s = ty_str(&result);
        prop_assert!(s.starts_with("adze :: WithLeaf"), "expected top-level wrap: {}", s);
    }
}

// ── 19. wrap_leaf_type wraps multi-generic args in skip types ───────────────

proptest! {
    #[test]
    fn wrap_multi_generic_args(idx in 0usize..=1) {
        let types: Vec<Type> = vec![
            parse_quote!(Result<String, i32>),
            parse_quote!(Result<bool, u64>),
        ];
        let skips = skip(&["Result"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        let s = ty_str(&result);
        // Both type arguments should be wrapped
        let count = s.matches("WithLeaf").count();
        prop_assert_eq!(count, 2, "expected 2 WithLeaf wrappings in: {}", s);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: is_parameterized detection
// ═══════════════════════════════════════════════════════════════════════════

// ── 20. Parameterized types detected ────────────────────────────────────────

proptest! {
    #[test]
    fn parameterized_types_detected(idx in 0usize..=5) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Box<bool>),
            parse_quote!(HashMap<String, i32>),
            parse_quote!(Arc<u64>),
            parse_quote!(Result<i32, String>),
        ];
        prop_assert!(is_parameterized(&types[idx]));
    }
}

// ── 21. Non-parameterized types rejected ────────────────────────────────────

proptest! {
    #[test]
    fn non_parameterized_types_rejected(idx in 0usize..=5) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(usize),
            parse_quote!(f64),
            parse_quote!(char),
        ];
        prop_assert!(!is_parameterized(&types[idx]));
    }
}

// ── 22. Non-path types are not parameterized ────────────────────────────────

proptest! {
    #[test]
    fn non_path_not_parameterized(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(&str),
            parse_quote!(&mut i32),
            parse_quote!((i32, u32)),
            parse_quote!([u8; 4]),
        ];
        prop_assert!(!is_parameterized(&types[idx]));
    }
}

// ── 23. Nested parameterized types detected ─────────────────────────────────

proptest! {
    #[test]
    fn nested_parameterized_detected(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<Option<i32>>),
            parse_quote!(Box<Vec<String>>),
            parse_quote!(Option<Box<Arc<u8>>>),
        ];
        prop_assert!(is_parameterized(&types[idx]));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Token stream roundtrip properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 24. parse_str roundtrip for simple types ────────────────────────────────

proptest! {
    #[test]
    fn parse_str_roundtrip_simple(idx in 0usize..=6) {
        let type_strs = ["i32", "u64", "String", "bool", "f32", "char", "usize"];
        let ty = syn::parse_str::<Type>(type_strs[idx]).unwrap();
        let rendered = ty_str(&ty);
        let reparsed = syn::parse_str::<Type>(&rendered).unwrap();
        prop_assert_eq!(ty_str(&reparsed), rendered);
    }
}

// ── 25. parse_str roundtrip for generic types ───────────────────────────────

proptest! {
    #[test]
    fn parse_str_roundtrip_generic(idx in 0usize..=4) {
        let type_strs = [
            "Vec<i32>", "Option<String>", "Box<bool>",
            "HashMap<String, i32>", "Result<u8, String>",
        ];
        let ty = syn::parse_str::<Type>(type_strs[idx]).unwrap();
        let rendered = ty_str(&ty);
        let reparsed = syn::parse_str::<Type>(&rendered).unwrap();
        prop_assert_eq!(ty_str(&reparsed), rendered);
    }
}

// ── 26. quote roundtrip preserves type identity ─────────────────────────────

proptest! {
    #[test]
    fn quote_roundtrip_preserves_identity(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(Vec<String>),
            parse_quote!(Option<Box<u8>>),
            parse_quote!(HashMap<String, Vec<i32>>),
            parse_quote!(bool),
        ];
        let tokens = quote::quote!(#(#types),*);
        let rendered = tokens.to_string();
        prop_assert!(!rendered.is_empty());
        // Each type should survive re-parsing
        let ty = &types[idx];
        let ts = quote::quote!(#ty);
        let reparsed: Type = syn::parse2(ts).unwrap();
        prop_assert_eq!(ty_str(&reparsed), ty_str(ty));
    }
}

// ── 27. DeriveInput roundtrip for structs ───────────────────────────────────

proptest! {
    #[test]
    fn derive_input_roundtrip_struct(idx in 0usize..=2) {
        let sources = [
            "struct Foo { x: i32 }",
            "struct Bar { name: String, value: u64 }",
            "struct Baz { items: Vec<i32>, flag: bool }",
        ];
        let di = syn::parse_str::<syn::DeriveInput>(sources[idx]).unwrap();
        let tokens = quote::quote!(#di);
        let reparsed = syn::parse_str::<syn::DeriveInput>(&tokens.to_string()).unwrap();
        prop_assert_eq!(reparsed.ident.to_string(), di.ident.to_string());
    }
}

// ── 28. DeriveInput roundtrip for enums ─────────────────────────────────────

proptest! {
    #[test]
    fn derive_input_roundtrip_enum(idx in 0usize..=2) {
        let sources = [
            "enum Color { Red, Green, Blue }",
            "enum Expr { Lit(i32), Add(Box<Expr>, Box<Expr>) }",
            "enum Token { Ident(String), Num(u64) }",
        ];
        let di = syn::parse_str::<syn::DeriveInput>(sources[idx]).unwrap();
        let tokens = quote::quote!(#di);
        let reparsed = syn::parse_str::<syn::DeriveInput>(&tokens.to_string()).unwrap();
        prop_assert_eq!(reparsed.ident.to_string(), di.ident.to_string());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Cross-function composition properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 29. filter then extract yields same result as extract through skip ──────

proptest! {
    #[test]
    fn filter_then_extract_consistent(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Vec<i32>>),
            parse_quote!(Arc<Vec<String>>),
            parse_quote!(Box<Vec<bool>>),
        ];
        let expected_inner = ["i32", "String", "bool"];
        let skips = skip(&["Box", "Arc"]);
        let empty = skip(&[]);

        // Path 1: extract through skip
        let (inner1, ok1) = try_extract_inner_type(&types[idx], "Vec", &skips);
        // Path 2: filter, then extract
        let filtered = filter_inner_type(&types[idx], &skips);
        let (inner2, ok2) = try_extract_inner_type(&filtered, "Vec", &empty);

        prop_assert!(ok1);
        prop_assert!(ok2);
        prop_assert_eq!(ty_str(&inner1), expected_inner[idx]);
        prop_assert_eq!(ty_str(&inner2), expected_inner[idx]);
    }
}

// ── 30. wrap then filter restores wrapper structure ─────────────────────────

proptest! {
    #[test]
    fn wrap_output_contains_with_leaf(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(Vec<bool>),
            parse_quote!(Option<u8>),
        ];
        let skips = skip(&["Vec", "Option"]);
        let result = wrap_leaf_type(&types[idx], &skips);
        let s = ty_str(&result);
        prop_assert!(s.contains("WithLeaf"), "result should contain WithLeaf: {}", s);
    }
}

// ── 31. Extracted type from wrap result is WithLeaf for plain types ─────────

proptest! {
    #[test]
    fn extract_from_wrapped_vec_yields_with_leaf(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Vec<bool>),
        ];
        let skips = skip(&["Vec", "Option"]);
        let wrapped = wrap_leaf_type(&types[idx], &skips);
        let empty = skip(&[]);
        let (inner, ok) = try_extract_inner_type(&wrapped, "Vec", &empty);
        prop_assert!(ok);
        prop_assert!(ty_str(&inner).contains("WithLeaf"),
            "inner should be WithLeaf: {}", ty_str(&inner));
    }
}

// ── 32. Parameterized detection agrees with extraction success ──────────────

proptest! {
    #[test]
    fn parameterized_implies_extractable(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Box<bool>),
            parse_quote!(Arc<u64>),
            parse_quote!(Rc<f32>),
        ];
        let wrappers = ["Vec", "Option", "Box", "Arc", "Rc"];
        let empty = skip(&[]);
        // All these are parameterized, and extraction should succeed
        prop_assert!(is_parameterized(&types[idx]));
        let (_, ok) = try_extract_inner_type(&types[idx], wrappers[idx], &empty);
        prop_assert!(ok);
    }
}

// ── 33. Non-parameterized types are never extracted ─────────────────────────

proptest! {
    #[test]
    fn non_parameterized_never_extracted(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(usize),
            parse_quote!(f64),
        ];
        let empty = skip(&[]);
        prop_assert!(!is_parameterized(&types[idx]));
        for wrapper in &["Vec", "Option", "Box"] {
            let (_, ok) = try_extract_inner_type(&types[idx], wrapper, &empty);
            prop_assert!(!ok);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Dynamic strategy-driven properties
// ═══════════════════════════════════════════════════════════════════════════

// ── 34. Generated simple types are never parameterized ──────────────────────

proptest! {
    #[test]
    fn generated_simple_types_not_parameterized(name in simple_type_name()) {
        let ty = syn::parse_str::<Type>(&name).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }
}

// ── 35. Generated wrapper<T> types are always parameterized ─────────────────

proptest! {
    #[test]
    fn generated_wrapped_types_parameterized(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        prop_assert!(is_parameterized(&ty));
    }
}

// ── 36. Generated wrapper extraction always succeeds ────────────────────────

proptest! {
    #[test]
    fn generated_wrapper_extraction_succeeds(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        let empty = skip(&[]);
        let (result, ok) = try_extract_inner_type(&ty, &wrapper, &empty);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }
}

// ── 37. Filter with matching skip extracts inner from generated types ───────

proptest! {
    #[test]
    fn generated_filter_strips_wrapper(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        let skips: HashSet<&str> = [wrapper.as_str()].into_iter().collect();
        let result = filter_inner_type(&ty, &skips);
        prop_assert_eq!(ty_str(&result), inner);
    }
}

// ── 38. Wrap of generated plain type yields WithLeaf<T> ─────────────────────

proptest! {
    #[test]
    fn generated_wrap_plain_yields_with_leaf(name in simple_type_name()) {
        let ty = syn::parse_str::<Type>(&name).unwrap();
        let empty = skip(&[]);
        let result = wrap_leaf_type(&ty, &empty);
        let s = ty_str(&result);
        let expected = format!("adze :: WithLeaf < {name} >");
        prop_assert_eq!(s, expected);
    }
}

// ── 39. Token roundtrip for generated wrapper types ─────────────────────────

proptest! {
    #[test]
    fn generated_wrapper_token_roundtrip(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        let tokens = quote::quote!(#ty);
        let reparsed: Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ty_str(&reparsed), ty_str(&ty));
    }
}

// ── 40. parse_str rejects Rust 2024 reserved keywords as types ──────────────

proptest! {
    #[test]
    fn reserved_keywords_rejected_as_types(idx in 0usize..=9) {
        let keywords = [
            "do", "abstract", "become", "final", "override",
            "priv", "typeof", "unsized", "virtual", "yield",
        ];
        let result = try_parse_type(keywords[idx]);
        prop_assert!(result.is_none(), "keyword '{}' should not parse as a type", keywords[idx]);
    }
}

// ── 41. Struct DeriveInput preserves field count ────────────────────────────

proptest! {
    #[test]
    fn derive_input_preserves_field_count(count in 1usize..=6) {
        let fields: Vec<String> = (0..count)
            .map(|i| format!("field_{i}: i32"))
            .collect();
        let body = fields.join(", ");
        let src = format!("struct TestStruct {{ {body} }}");
        let di = syn::parse_str::<syn::DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert_eq!(data.fields.len(), count);
        } else {
            prop_assert!(false, "expected struct");
        }
    }
}

// ── 42. Enum DeriveInput preserves variant count ────────────────────────────

proptest! {
    #[test]
    fn derive_input_preserves_variant_count(count in 1usize..=8) {
        let variants: Vec<String> = (0..count)
            .map(|i| format!("Variant{i}"))
            .collect();
        let body = variants.join(", ");
        let src = format!("enum TestEnum {{ {body} }}");
        let di = syn::parse_str::<syn::DeriveInput>(&src).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            prop_assert_eq!(data.variants.len(), count);
        } else {
            prop_assert!(false, "expected enum");
        }
    }
}

// ── 43. Wrap then extract roundtrip for Vec<T> ──────────────────────────────

proptest! {
    #[test]
    fn wrap_extract_roundtrip_vec(inner in simple_type_name()) {
        let src = format!("Vec<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        let skips = skip(&["Vec"]);
        let wrapped = wrap_leaf_type(&ty, &skips);
        // After wrapping, extracting Vec should give WithLeaf<inner>
        let empty = skip(&[]);
        let (extracted, ok) = try_extract_inner_type(&wrapped, "Vec", &empty);
        prop_assert!(ok);
        let expected = format!("adze :: WithLeaf < {inner} >");
        prop_assert_eq!(ty_str(&extracted), expected);
    }
}

// ── 44. Filter is idempotent ────────────────────────────────────────────────

proptest! {
    #[test]
    fn filter_is_idempotent(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty = syn::parse_str::<Type>(&src).unwrap();
        let skips: HashSet<&str> = [wrapper.as_str()].into_iter().collect();
        let once = filter_inner_type(&ty, &skips);
        let twice = filter_inner_type(&once, &skips);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}
