#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar output structure validation in adze-common.
//!
//! Covers: output token stream well-formedness, JSON-like structure properties,
//! determinism of expansion output, expansion invariants, and edge cases
//! (empty types, deeply nested, many fields, unusual compositions).

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Item, ItemEnum, ItemMod, ItemStruct, Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn container() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box"][..])
}

fn skip_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Arc", "Rc", "Cell"][..])
}

fn pascal_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .unwrap()
        .prop_filter("must be valid grammar name", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Generates type strings with 0-3 nesting depth.
fn nested_type_string() -> impl Strategy<Value = String> {
    let depth0 = leaf_type().prop_map(|s| s.to_string());
    let depth1 = (container(), leaf_type()).prop_map(|(c, l)| format!("{c}<{l}>"));
    let depth2 =
        (container(), container(), leaf_type()).prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"));
    let depth3 = (container(), container(), container(), leaf_type())
        .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>"));
    prop_oneof![depth0, depth1, depth2, depth3]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn build_grammar_module(grammar_name: &str, mod_name: &str, body: &str) -> String {
    format!(
        r#"#[adze::grammar("{grammar_name}")]
mod {mod_name} {{
{body}
}}"#
    )
}

/// Check that angle brackets are balanced in a token string.
fn brackets_balanced(s: &str) -> bool {
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

// ===========================================================================
// 1. Output token stream well-formedness — balanced brackets
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Wrapped output always has balanced angle brackets.
    #[test]
    fn wrap_output_balanced_brackets(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(brackets_balanced(&s), "unbalanced: {}", s);
    }

    // 2. Wrapped container output has balanced brackets.
    #[test]
    fn wrap_container_output_balanced(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[ctr]));
        let s = ty_str(&wrapped);
        prop_assert!(brackets_balanced(&s), "unbalanced: {}", s);
    }

    // 3. Deeply nested wrap output has balanced brackets.
    #[test]
    fn wrap_nested_output_balanced(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]));
        let s = ty_str(&wrapped);
        prop_assert!(brackets_balanced(&s), "unbalanced: {}", s);
    }

    // 4. Extracted output always has balanced brackets.
    #[test]
    fn extract_output_balanced(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (result, _) = try_extract_inner_type(&ty, ctr, &skip(&[]));
        let s = ty_str(&result);
        prop_assert!(brackets_balanced(&s), "unbalanced: {}", s);
    }

    // 5. Filtered output always has balanced brackets.
    #[test]
    fn filter_output_balanced(wrapper in skip_name(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        let s = ty_str(&filtered);
        prop_assert!(brackets_balanced(&s), "unbalanced: {}", s);
    }
}

// ===========================================================================
// 2. Output is always re-parseable as a valid Rust type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 6. Wrapped leaf output re-parses.
    #[test]
    fn wrap_output_reparseable(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[]));
        let s = ty_str(&wrapped);
        let reparsed: syn::Result<Type> = parse_str(&s);
        prop_assert!(reparsed.is_ok(), "failed to reparse: {}", s);
    }

    // 7. Wrapped container output re-parses.
    #[test]
    fn wrap_container_output_reparseable(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[ctr]));
        let s = ty_str(&wrapped);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }

    // 8. Extract output re-parses.
    #[test]
    fn extract_output_reparseable(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (result, _) = try_extract_inner_type(&ty, ctr, &skip(&[]));
        let s = ty_str(&result);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }

    // 9. Filter output re-parses.
    #[test]
    fn filter_output_reparseable(wrapper in skip_name(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        let s = ty_str(&filtered);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }

    // 10. Full pipeline output re-parses.
    #[test]
    fn pipeline_output_reparseable(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Box<{inner}>>")).unwrap();
        let (after, _) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        let filtered = filter_inner_type(&after, &skip(&["Box"]));
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }

    // 11. Nested type string output re-parses after all three operations.
    #[test]
    fn nested_all_ops_reparseable(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let skip_all = skip(&["Option", "Vec", "Box"]);
        let (extracted, _) = try_extract_inner_type(&ty, "Option", &skip_all);
        let filtered = filter_inner_type(&extracted, &skip_all);
        let wrapped = wrap_leaf_type(&filtered, &skip_all);
        let s = ty_str(&wrapped);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }
}

// ===========================================================================
// 3. Wrap output structure — always contains "WithLeaf" at correct position
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 12. Wrapping a leaf always produces output starting with "adze :: WithLeaf".
    #[test]
    fn wrap_leaf_starts_with_withleaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(s.starts_with("adze :: WithLeaf"), "unexpected: {}", s);
    }

    // 13. Wrapping container not in skip set wraps the whole thing.
    #[test]
    fn wrap_non_skip_container_wrapped_entirely(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        // Skip set does NOT contain the container
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(s.starts_with("adze :: WithLeaf"), "unexpected: {}", s);
    }

    // 14. Wrapping container in skip set preserves outer container name.
    #[test]
    fn wrap_skip_container_preserves_outer(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[ctr])));
        prop_assert!(s.starts_with(ctr), "expected to start with {ctr}, got: {}", s);
    }

    // 15. Wrapping with skip set produces exactly one "WithLeaf" for single-nested types.
    #[test]
    fn wrap_single_nested_one_withleaf(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[ctr])));
        let count = s.matches("WithLeaf").count();
        prop_assert_eq!(count, 1, "expected 1 WithLeaf in: {}", s);
    }

    // 16. Wrapping double-nested with both in skip still produces exactly one WithLeaf.
    #[test]
    fn wrap_double_nested_one_withleaf(
        c1 in container(),
        c2 in container(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{c1}<{c2}<{inner}>>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[c1, c2])));
        let count = s.matches("WithLeaf").count();
        prop_assert_eq!(count, 1, "expected 1 WithLeaf in: {}", s);
    }
}

// ===========================================================================
// 4. Extract output — extracted type is substring of original
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 17. Extracted inner type string appears in the original type string.
    #[test]
    fn extract_inner_is_substring_of_original(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, ctr, &skip(&[]));
        prop_assert!(ok);
        let orig = ty_str(&ty);
        let extracted_s = ty_str(&result);
        prop_assert!(orig.contains(&extracted_s), "{} not in {}", extracted_s, orig);
    }

    // 18. Extracted type is strictly shorter than original when extraction succeeds.
    #[test]
    fn extract_result_shorter_than_original(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, ctr, &skip(&[]));
        prop_assert!(ok);
        prop_assert!(ty_str(&result).len() < ty_str(&ty).len());
    }

    // 19. Failed extraction returns exactly the same string as original.
    #[test]
    fn extract_failure_returns_original_string(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }
}

// ===========================================================================
// 5. Filter output — filtered type is shorter or equal
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 20. Filtering a skip-set wrapper produces shorter output.
    #[test]
    fn filter_produces_shorter_output(wrapper in skip_name(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        prop_assert!(ty_str(&filtered).len() < ty_str(&ty).len());
    }

    // 21. Filtering a non-skip type preserves length exactly.
    #[test]
    fn filter_non_skip_preserves_length(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[]));
        prop_assert_eq!(ty_str(&filtered).len(), ty_str(&ty).len());
    }

    // 22. Filtered output never contains the stripped wrapper name at the start.
    #[test]
    fn filter_output_does_not_start_with_stripped_wrapper(
        wrapper in skip_name(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        let s = ty_str(&filtered);
        prop_assert!(!s.starts_with(wrapper), "still starts with {wrapper}: {}", s);
    }
}

// ===========================================================================
// 6. Determinism of all output functions
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 23. wrap_leaf_type output is identical across 3 calls.
    #[test]
    fn wrap_deterministic_triple(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let sk = skip(&[]);
        let a = ty_str(&wrap_leaf_type(&ty, &sk));
        let b = ty_str(&wrap_leaf_type(&ty, &sk));
        let c = ty_str(&wrap_leaf_type(&ty, &sk));
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
    }

    // 24. try_extract_inner_type output is identical across 3 calls.
    #[test]
    fn extract_deterministic_triple(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let sk = skip(&[]);
        let results: Vec<_> = (0..3)
            .map(|_| {
                let (r, e) = try_extract_inner_type(&ty, ctr, &sk);
                (ty_str(&r), e)
            })
            .collect();
        prop_assert_eq!(&results[0], &results[1]);
        prop_assert_eq!(&results[1], &results[2]);
    }

    // 25. filter_inner_type output is identical across 3 calls.
    #[test]
    fn filter_deterministic_triple(wrapper in skip_name(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let wrapper_arr = [wrapper];
        let sk = skip(&wrapper_arr);
        let a = ty_str(&filter_inner_type(&ty, &sk));
        let b = ty_str(&filter_inner_type(&ty, &sk));
        let c = ty_str(&filter_inner_type(&ty, &sk));
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
    }

    // 26. Full pipeline determinism: extract → filter → wrap identical across 2 runs.
    #[test]
    fn pipeline_deterministic(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Box<{inner}>>")).unwrap();
        let run = || {
            let (after, _) = try_extract_inner_type(&ty, "Option", &skip(&[]));
            let filtered = filter_inner_type(&after, &skip(&["Box"]));
            ty_str(&wrap_leaf_type(&filtered, &skip(&[])))
        };
        let a = run();
        let b = run();
        prop_assert_eq!(&a, &b);
    }

    // 27. FieldThenParams parse output is deterministic for field type string.
    #[test]
    fn field_then_params_deterministic(inner in leaf_type()) {
        let src = format!("{inner}, key = 1");
        let a: FieldThenParams = syn::parse_str(&src).unwrap();
        let b: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ty_str(&a.field.ty), ty_str(&b.field.ty));
        prop_assert_eq!(a.params.len(), b.params.len());
    }

    // 28. NameValueExpr parse output is deterministic.
    #[test]
    fn name_value_deterministic(val in 0i32..500) {
        let src = format!("param = {val}");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a.path.to_string(), b.path.to_string());
    }
}

// ===========================================================================
// 7. Expansion invariants — algebraic properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 29. filter_inner_type is idempotent: filter(filter(x)) == filter(x).
    #[test]
    fn filter_idempotent(wrapper in skip_name(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let wrapper_arr = [wrapper];
        let sk = skip(&wrapper_arr);
        let once = filter_inner_type(&ty, &sk);
        let twice = filter_inner_type(&once, &sk);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    // 30. Extract then re-extract with different target yields no double extraction.
    #[test]
    fn extract_different_target_no_double(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (after_opt, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(ok1);
        // Extracted type is a leaf, so extracting Vec should fail.
        let (_, ok2) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
        prop_assert!(!ok2);
    }

    // 31. Extraction preserves the inner type exactly.
    #[test]
    fn extract_preserves_inner_exactly(ctr in container(), inner in leaf_type()) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, ctr, &skip(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 32. Wrapping then extracting "adze" segment: wrapped output contains leaf string.
    #[test]
    fn wrap_output_contains_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(s.contains(leaf), "{} not in {}", leaf, s);
    }

    // 33. Extracting from container<container<T>> with inner container as target
    //     (and outer as skip) yields T — only when outer != inner_ctr.
    #[test]
    fn extract_inner_container_through_skip(
        outer in prop::sample::select(&["Arc", "Rc", "Cell"][..]),
        inner_ctr in container(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{outer}<{inner_ctr}<{leaf}>>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, inner_ctr, &skip(&[outer]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    // 34. Filter with empty skip set is always identity.
    #[test]
    fn filter_empty_skip_is_identity(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[]));
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 35. Extraction with non-matching target is always identity on the string.
    #[test]
    fn extract_nonmatch_is_identity(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "NONEXISTENT", &skip(&[]));
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }
}

// ===========================================================================
// 8. Struct output — field count and name preservation
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 36. Struct with N fields always has exactly N fields after parsing.
    #[test]
    fn struct_field_count(name in pascal_ident(), count in 1usize..=10) {
        let fields: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: i32,"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
    }

    // 37. Struct field names are always in the order declared.
    #[test]
    fn struct_field_order(name in pascal_ident(), count in 1usize..=8) {
        let expected: Vec<String> = (0..count).map(|i| format!("field{i}")).collect();
        let fields: Vec<String> = expected
            .iter()
            .map(|n| format!("    pub {n}: u32,"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let actual: Vec<String> = parsed
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        prop_assert_eq!(actual, expected);
    }

    // 38. Struct field types match the declared types exactly.
    #[test]
    fn struct_field_types_match(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 1..=6),
    ) {
        let fields: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for (i, field) in parsed.fields.iter().enumerate() {
            prop_assert_eq!(field.ty.to_token_stream().to_string(), types[i]);
        }
    }

    // 39. Struct ident matches the declared name.
    #[test]
    fn struct_ident_matches(name in pascal_ident(), ty in leaf_type()) {
        let src = format!("pub struct {name} {{ pub v: {ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
    }
}

// ===========================================================================
// 9. Enum output — variant count, names, and types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 40. Enum variant count matches.
    #[test]
    fn enum_variant_count(name in pascal_ident(), count in 1usize..=10) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
    }

    // 41. Enum variant names preserved in order.
    #[test]
    fn enum_variant_order(name in pascal_ident(), count in 1usize..=8) {
        let expected: Vec<String> = (0..count).map(|i| format!("Var{i}")).collect();
        let variants: Vec<String> = expected.iter().map(|n| format!("    {n},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for i in 0..count {
            prop_assert_eq!(parsed.variants[i].ident.to_string(), expected[i].as_str());
        }
    }

    // 42. Enum with tuple variants preserves inner type.
    #[test]
    fn enum_tuple_variant_type(name in pascal_ident(), ty in leaf_type()) {
        let src = format!("pub enum {name} {{ A({ty}), B({ty}), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for v in &parsed.variants {
            let f = v.fields.iter().next().unwrap();
            prop_assert_eq!(f.ty.to_token_stream().to_string(), ty);
        }
    }

    // 43. Enum ident matches.
    #[test]
    fn enum_ident_matches(name in pascal_ident()) {
        let src = format!("pub enum {name} {{ A, B, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
    }
}

// ===========================================================================
// 10. Grammar module output — items preserved inside module
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 44. Grammar module preserves struct item count.
    #[test]
    fn grammar_module_struct_count(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
        count in 1usize..=4,
        ty in leaf_type(),
    ) {
        let structs: Vec<String> = (0..count)
            .map(|i| format!("    pub struct S{i} {{ pub v: {ty}, }}"))
            .collect();
        let body = structs.join("\n");
        let src = build_grammar_module(&gname, &mname, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), count);
    }

    // 45. Grammar module struct names match.
    #[test]
    fn grammar_module_struct_names(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
        sname in pascal_ident(),
        ty in leaf_type(),
    ) {
        let body = format!("    pub struct {sname} {{ pub v: {ty}, }}");
        let src = build_grammar_module(&gname, &mname, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            prop_assert_eq!(s.ident.to_string(), sname);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    // 46. Grammar module enum inside module is Item::Enum.
    #[test]
    fn grammar_module_enum_detected(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
        ename in pascal_ident(),
    ) {
        let body = format!("    pub enum {ename} {{ A, B, }}");
        let src = build_grammar_module(&gname, &mname, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert!(matches!(&items[0], Item::Enum(_)));
    }

    // 47. Grammar module name matches declared mod name.
    #[test]
    fn grammar_module_name_matches(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
    ) {
        let body = "    pub struct S { pub v: i32, }";
        let src = build_grammar_module(&gname, &mname, body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), mname);
    }
}

// ===========================================================================
// 11. FieldThenParams output — param counts and names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 48. FieldThenParams with N params has exactly N params.
    #[test]
    fn ftp_param_count(ty in leaf_type(), count in 1usize..=5) {
        let params: Vec<String> = (0..count).map(|i| format!("p{i} = {i}")).collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.params.len(), count);
    }

    // 49. FieldThenParams param names preserved in order.
    #[test]
    fn ftp_param_names_ordered(ty in leaf_type(), count in 1usize..=4) {
        let expected: Vec<String> = (0..count).map(|i| format!("key{i}")).collect();
        let params: Vec<String> = expected.iter().map(|k| format!("{k} = 0")).collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        for i in 0..count {
            prop_assert_eq!(parsed.params[i].path.to_string(), expected[i].as_str());
        }
    }

    // 50. FieldThenParams with no params has empty params list.
    #[test]
    fn ftp_no_params(ty in leaf_type()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert!(parsed.params.is_empty());
        prop_assert!(parsed.comma.is_none());
    }

    // 51. FieldThenParams field type string matches input.
    #[test]
    fn ftp_field_type_matches(ty in leaf_type()) {
        let src = format!("{ty}, x = 1");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.field.ty.to_token_stream().to_string(), ty);
    }
}

// ===========================================================================
// 12. Edge cases — empty / degenerate / deeply nested
// ===========================================================================

// 52. Unit type passes through all functions without panic.
#[test]
fn edge_unit_type_no_panic() {
    let ty: Type = parse_quote!(());
    let sk = skip(&["Box", "Option", "Vec"]);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &sk);
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &sk);
    assert_eq!(ty_str(&filtered), "()");
    let wrapped = wrap_leaf_type(&ty, &sk);
    assert!(ty_str(&wrapped).contains("WithLeaf"));
}

// 53. Reference type passes through without panic.
#[test]
fn edge_reference_type_no_panic() {
    let ty: Type = parse_quote!(&str);
    let sk = skip(&["Box"]);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &sk);
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &sk);
    assert_eq!(ty_str(&filtered), "& str");
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&wrapped).starts_with("adze :: WithLeaf"));
}

// 54. Tuple type passes through without panic.
#[test]
fn edge_tuple_type_no_panic() {
    let ty: Type = parse_quote!((i32, String, bool));
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert!(ty_str(&filtered).contains("i32"));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&wrapped).starts_with("adze :: WithLeaf"));
}

// 55. Array type passes through without panic.
#[test]
fn edge_array_type_no_panic() {
    let ty: Type = parse_quote!([u8; 32]);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert!(ty_str(&filtered).contains("u8"));
}

// 56. Deeply nested (4 layers) filter strips all layers.
#[test]
fn edge_deep_four_layer_filter() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<String>>>>);
    let sk = skip(&["Box", "Arc", "Rc", "Cell"]);
    let filtered = filter_inner_type(&ty, &sk);
    assert_eq!(ty_str(&filtered), "String");
}

// 57. Deeply nested (4 layers) wrap with all in skip produces one WithLeaf.
#[test]
fn edge_deep_four_layer_wrap() {
    let ty: Type = parse_quote!(Vec<Option<Vec<Option<bool>>>>);
    let sk = skip(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &sk);
    let s = ty_str(&wrapped);
    assert_eq!(s.matches("WithLeaf").count(), 1);
    assert!(s.contains("bool"));
}

// 58. Struct with zero fields (unit struct) parses.
#[test]
fn edge_unit_struct() {
    let src = "pub struct Empty;";
    let parsed: ItemStruct = parse_str(src).unwrap();
    assert_eq!(parsed.fields.len(), 0);
    assert_eq!(parsed.ident.to_string(), "Empty");
}

// 59. Struct with many fields (16) parses correctly.
#[test]
fn edge_many_fields_struct() {
    let fields: Vec<String> = (0..16).map(|i| format!("    pub f{i}: u32,")).collect();
    let src = format!("pub struct Big {{\n{}\n}}", fields.join("\n"));
    let parsed: ItemStruct = parse_str(&src).unwrap();
    assert_eq!(parsed.fields.len(), 16);
}

// 60. Enum with many variants (20) parses correctly.
#[test]
fn edge_many_variants_enum() {
    let variants: Vec<String> = (0..20).map(|i| format!("    V{i},")).collect();
    let src = format!("pub enum Big {{\n{}\n}}", variants.join("\n"));
    let parsed: ItemEnum = parse_str(&src).unwrap();
    assert_eq!(parsed.variants.len(), 20);
}

// 61. Qualified path types: extraction works on fully qualified Option.
#[test]
fn edge_qualified_option_extraction() {
    let ty: Type = parse_quote!(std::option::Option<i64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i64");
}

// 62. Qualified path types: filter works on fully qualified Box.
#[test]
fn edge_qualified_box_filter() {
    let ty: Type = parse_quote!(std::boxed::Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

// 63. Qualified path types: wrap works on fully qualified Vec.
#[test]
fn edge_qualified_vec_wrap() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let s = ty_str(&wrapped);
    assert!(s.contains("WithLeaf"));
    assert!(s.contains("u8"));
    assert!(s.starts_with("std"));
}

// ===========================================================================
// 13. Cross-function composition edge cases
// ===========================================================================

// 64. Extract → wrap roundtrip for Result-like type (multi-generic).
#[test]
fn edge_result_extract_wrap() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// 65. Wrap on Result with Result in skip wraps both type args.
#[test]
fn edge_result_wrap_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    let s = ty_str(&wrapped);
    assert_eq!(s.matches("WithLeaf").count(), 2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 66. Composition: extract Option, then wrap with Vec skip preserves Vec.
    #[test]
    fn compose_extract_option_wrap_vec(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&after_opt, &skip(&["Vec"]));
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with("Vec"), "should start with Vec: {}", s);
        prop_assert_eq!(s.matches("WithLeaf").count(), 1);
    }

    // 67. Composition: filter strips wrapper, then extraction on the remainder works.
    //     Uses skip names that don't overlap with containers to avoid direct-match.
    #[test]
    fn compose_filter_then_extract(
        wrapper in prop::sample::select(&["Arc", "Rc", "Cell"][..]),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{ctr}<{inner}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        let (result, ok) = try_extract_inner_type(&filtered, ctr, &skip(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 68. Composition: wrap(filter(extract(type))) produces valid reparseable type.
    #[test]
    fn compose_full_pipeline_valid(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Option<{inner}>>")).unwrap();
        let (after_ext, _) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
        let filtered = filter_inner_type(&after_ext, &skip(&["Box"]));
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        let s = ty_str(&wrapped);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "failed to reparse: {}", s);
    }
}

// ===========================================================================
// 14. NameValueExpr output structure validation
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 69. NameValueExpr path is always a valid Rust identifier.
    #[test]
    fn nve_path_valid_ident(key in ident_strategy(), val in 0i32..100) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let path_str = parsed.path.to_string();
        prop_assert!(syn::parse_str::<syn::Ident>(&path_str).is_ok());
    }

    // 70. NameValueExpr with string value preserves the key.
    #[test]
    fn nve_string_value_key_preserved(key in ident_strategy()) {
        let src = format!("{key} = \"hello\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }
}

// ===========================================================================
// 15. Grammar module determinism — same source yields same parse
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 71. Grammar module with struct: two parses yield same item count.
    #[test]
    fn grammar_module_deterministic(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
        sname in pascal_ident(),
        ty in leaf_type(),
    ) {
        let body = format!("    pub struct {sname} {{ pub v: {ty}, }}");
        let src = build_grammar_module(&gname, &mname, &body);
        let a: ItemMod = parse_str(&src).unwrap();
        let b: ItemMod = parse_str(&src).unwrap();
        let items_a = &a.content.unwrap().1;
        let items_b = &b.content.unwrap().1;
        prop_assert_eq!(items_a.len(), items_b.len());
        prop_assert_eq!(a.ident.to_string(), b.ident.to_string());
    }

    // 72. Mixed module content is deterministic.
    #[test]
    fn grammar_module_mixed_deterministic(
        gname in grammar_name_strategy(),
        mname in ident_strategy(),
    ) {
        let body = "    pub struct S { pub v: i32, }\n    pub enum E { A, B, }";
        let src = build_grammar_module(&gname, &mname, body);
        let a: ItemMod = parse_str(&src).unwrap();
        let b: ItemMod = parse_str(&src).unwrap();
        let (_, items_a) = a.content.unwrap();
        let (_, items_b) = b.content.unwrap();
        prop_assert_eq!(items_a.len(), items_b.len());
        for i in 0..items_a.len() {
            let kind_a = std::mem::discriminant(&items_a[i]);
            let kind_b = std::mem::discriminant(&items_b[i]);
            prop_assert_eq!(kind_a, kind_b);
        }
    }
}
