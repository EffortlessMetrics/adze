//! Comprehensive tests for composing `filter_inner_type` with other type
//! operations (`try_extract_inner_type`, `wrap_leaf_type`) in adze-common.
//!
//! Covers: filter→extract, filter→wrap, wrap→filter, extract→wrap roundtrips,
//! wrap→extract roundtrips, idempotency, multiple filters, nested wrappers,
//! triple/quadruple compositions, and type validity preservation.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. filter then extract → composed result
// ===========================================================================

#[test]
fn filter_then_extract_option_vec_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    // filtered = Vec<i32>
    let (extracted, found) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "i32");
}

#[test]
fn filter_then_extract_box_option_string() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let (extracted, found) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "String");
}

#[test]
fn filter_then_extract_no_match() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // filtered = i32, extract Option won't find anything
    let (result, found) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_then_extract_strips_two_layers() {
    let ty: Type = parse_quote!(Box<Option<Vec<u64>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    let (extracted, found) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "u64");
}

// ===========================================================================
// 2. filter then wrap → composed result
// ===========================================================================

#[test]
fn filter_then_wrap_simple() {
    let ty: Type = parse_quote!(Option<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn filter_then_wrap_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_nested_with_skip() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u8 >");
}

#[test]
fn filter_then_wrap_preserves_vec_skip() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    // filtered = Vec<i32>, wrap with Vec in skip → Vec<adze::WithLeaf<i32>>
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

// ===========================================================================
// 3. wrap then filter → composed result
// ===========================================================================

#[test]
fn wrap_then_filter_leaf_unchanged() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // wrapped = adze::WithLeaf<i32>, filter with empty skip → unchanged
    let filtered = filter_inner_type(&wrapped, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&wrapped));
}

#[test]
fn wrap_then_filter_vec_strip_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // wrapped = Vec<adze::WithLeaf<i32>>
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec"]));
    // strips Vec → adze::WithLeaf<i32>
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_then_filter_option_strip_option() {
    let ty: Type = parse_quote!(Option<u64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_then_filter_non_skip_wrap_not_stripped() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // wrapped = adze::WithLeaf<i32>
    // filter with Vec skip won't match WithLeaf
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

// ===========================================================================
// 4. extract then wrap → roundtrip
// ===========================================================================

#[test]
fn extract_then_wrap_vec_roundtrip() {
    let ty: Type = parse_quote!(Vec<adze::WithLeaf<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    // extracted = adze::WithLeaf<i32>
    let wrapped = wrap_leaf_type(&extracted, &skip(&[]));
    // wrapping WithLeaf<i32> again → adze::WithLeaf<adze::WithLeaf<i32>>
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn extract_then_wrap_option_identity_leaf() {
    let ty: Type = parse_quote!(Option<String>);
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "String");
    let wrapped = wrap_leaf_type(&extracted, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_then_wrap_with_skip_preserves_container() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    let wrapped = wrap_leaf_type(&extracted, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn extract_skip_then_wrap() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "bool");
    let wrapped = wrap_leaf_type(&extracted, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

// ===========================================================================
// 5. wrap then extract → roundtrip
// ===========================================================================

#[test]
fn wrap_then_extract_leaf_type() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // wrapped = adze::WithLeaf<i32>
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    // "WithLeaf" is the last segment of adze::WithLeaf, won't match at top level
    // since path is adze::WithLeaf, last segment IS WithLeaf
    assert!(found);
    assert_eq!(ty_str(&extracted), "i32");
}

#[test]
fn wrap_vec_then_extract_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // wrapped = adze::WithLeaf<Vec<i32>>, extract WithLeaf
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "Vec < i32 >");
}

#[test]
fn wrap_option_in_skip_then_extract_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    // wrapped = Option<adze::WithLeaf<i32>>
    let (extracted, found) = try_extract_inner_type(&wrapped, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_then_extract_not_found() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let (result, found) = try_extract_inner_type(&wrapped, "Vec", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&wrapped));
}

// ===========================================================================
// 6. filter is idempotent (filter twice → same)
// ===========================================================================

#[test]
fn filter_idempotent_plain_type() {
    let ty: Type = parse_quote!(i32);
    let first = filter_inner_type(&ty, &skip(&["Vec"]));
    let second = filter_inner_type(&first, &skip(&["Vec"]));
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_idempotent_after_unwrap() {
    let ty: Type = parse_quote!(Vec<String>);
    let first = filter_inner_type(&ty, &skip(&["Vec"]));
    let second = filter_inner_type(&first, &skip(&["Vec"]));
    assert_eq!(ty_str(&first), "String");
    assert_eq!(ty_str(&second), "String");
}

#[test]
fn filter_idempotent_nested() {
    let ty: Type = parse_quote!(Option<Box<u8>>);
    let first = filter_inner_type(&ty, &skip(&["Option", "Box"]));
    let second = filter_inner_type(&first, &skip(&["Option", "Box"]));
    assert_eq!(ty_str(&first), ty_str(&second));
    assert_eq!(ty_str(&first), "u8");
}

#[test]
fn filter_idempotent_non_matching() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let first = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    let second = filter_inner_type(&first, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_idempotent_empty_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let first = filter_inner_type(&ty, &skip(&[]));
    let second = filter_inner_type(&first, &skip(&[]));
    assert_eq!(ty_str(&first), ty_str(&second));
    assert_eq!(ty_str(&first), "Vec < i32 >");
}

// ===========================================================================
// 7. Multiple filter calls → behavior
// ===========================================================================

#[test]
fn multiple_filters_strip_layers_progressively() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let after_vec = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&after_vec), "Option < i32 >");
    let after_option = filter_inner_type(&after_vec, &skip(&["Option"]));
    assert_eq!(ty_str(&after_option), "i32");
}

#[test]
fn multiple_filters_single_skip_each() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&step1), "Arc < String >");
    let step2 = filter_inner_type(&step1, &skip(&["Arc"]));
    assert_eq!(ty_str(&step2), "String");
}

#[test]
fn multiple_filters_same_skip_strips_recursively() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    // filter_inner_type recurses: strips outer Vec, then inner Vec → i32
    let step1 = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&step1), "i32");
    // Second call is idempotent
    let step2 = filter_inner_type(&step1, &skip(&["Vec"]));
    assert_eq!(ty_str(&step2), "i32");
}

#[test]
fn multiple_filters_stop_at_non_skip() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let step1 = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&step1), "HashMap < String , i32 >");
    // HashMap not in skip, so second call is no-op
    let step2 = filter_inner_type(&step1, &skip(&["Vec"]));
    assert_eq!(ty_str(&step2), "HashMap < String , i32 >");
}

// ===========================================================================
// 8. filter then filter with different params
// ===========================================================================

#[test]
fn filter_vec_then_filter_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let step1 = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&step1), "Option < bool >");
    let step2 = filter_inner_type(&step1, &skip(&["Option"]));
    assert_eq!(ty_str(&step2), "bool");
}

#[test]
fn filter_option_then_filter_vec() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    let step1 = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&step1), "Vec < f64 >");
    let step2 = filter_inner_type(&step1, &skip(&["Vec"]));
    assert_eq!(ty_str(&step2), "f64");
}

#[test]
fn filter_box_then_filter_option_vec() {
    let ty: Type = parse_quote!(Box<Option<Vec<u32>>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&step1), "Option < Vec < u32 > >");
    let step2 = filter_inner_type(&step1, &skip(&["Option", "Vec"]));
    assert_eq!(ty_str(&step2), "u32");
}

#[test]
fn filter_different_params_versus_combined() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    // Sequential single-skip filters
    let seq = filter_inner_type(&filter_inner_type(&ty, &skip(&["Vec"])), &skip(&["Option"]));
    // Combined multi-skip filter
    let combined = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&seq), ty_str(&combined));
}

// ===========================================================================
// 9. wrap then wrap then filter → behavior
// ===========================================================================

#[test]
fn wrap_wrap_then_filter_leaf() {
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    // twice = adze::WithLeaf<adze::WithLeaf<i32>>
    // filter with empty skip → unchanged
    let filtered = filter_inner_type(&twice, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&twice));
}

#[test]
fn wrap_vec_wrap_then_filter_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // wrapped = Vec<adze::WithLeaf<i32>>
    let double_wrapped = wrap_leaf_type(&wrapped, &skip(&["Vec"]));
    // double_wrapped = Vec<adze::WithLeaf<adze::WithLeaf<i32>>>
    let filtered = filter_inner_type(&double_wrapped, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&filtered),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_option_twice_then_filter_option() {
    let ty: Type = parse_quote!(Option<u8>);
    let w1 = wrap_leaf_type(&ty, &skip(&["Option"]));
    // w1 = Option<adze::WithLeaf<u8>>
    let w2 = wrap_leaf_type(&w1, &skip(&["Option"]));
    // w2 = Option<adze::WithLeaf<adze::WithLeaf<u8>>>
    let filtered = filter_inner_type(&w2, &skip(&["Option"]));
    assert_eq!(
        ty_str(&filtered),
        "adze :: WithLeaf < adze :: WithLeaf < u8 > >"
    );
}

// ===========================================================================
// 10. Chain: wrap → filter → identity exploration
// ===========================================================================

#[test]
fn wrap_no_skip_filter_no_skip_identity() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // adze::WithLeaf<String>
    // filter with no skip → unchanged (WithLeaf not in skip)
    let filtered = filter_inner_type(&wrapped, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&wrapped));
}

#[test]
fn wrap_with_skip_filter_same_skip_strips_container() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // Vec<adze::WithLeaf<i32>>
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec"]));
    // Strips Vec → adze::WithLeaf<i32>
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_option_filter_option_yields_wrapped_leaf() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_nested_filter_all_yields_double_wrapped() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    // Vec<Option<adze::WithLeaf<i32>>>
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

// ===========================================================================
// 11. Chain: extract → wrap → identity
// ===========================================================================

#[test]
fn extract_vec_wrap_vec_roundtrip() {
    let original: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&original, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
    // Wrap does not reconstruct Vec — it wraps in WithLeaf
    let rewrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&rewrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_option_wrap_preserves_leaf() {
    let ty: Type = parse_quote!(Option<f32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "f32");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f32 >");
}

#[test]
fn extract_skip_wrap_skip_composition() {
    let ty: Type = parse_quote!(Option<Vec<u16>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u16");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Option"]));
    // u16 is not Option, so wraps in WithLeaf
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u16 >");
}

#[test]
fn extract_not_found_wrap_preserves() {
    let ty: Type = parse_quote!(String);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!found);
    let wrapped = wrap_leaf_type(&result, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 12. filter with 1 skip → behavior
// ===========================================================================

#[test]
fn filter_1skip_vec_strips_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_1skip_option_strips_option() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_1skip_box_strips_box() {
    let ty: Type = parse_quote!(Box<u64>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn filter_1skip_non_matching_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

// ===========================================================================
// 13. filter with 2 skips → behavior
// ===========================================================================

#[test]
fn filter_2skip_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_2skip_option_box() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let result = filter_inner_type(&ty, &skip(&["Option", "Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_2skip_only_outer_matches() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "HashMap < String , i32 >");
}

#[test]
fn filter_2skip_only_inner_matches_stops() {
    let ty: Type = parse_quote!(HashMap<Option<i32>, String>);
    // HashMap is not in skip, so returns unchanged
    let result = filter_inner_type(&ty, &skip(&["Option", "Box"]));
    assert_eq!(ty_str(&result), "HashMap < Option < i32 > , String >");
}

// ===========================================================================
// 14. filter with 3 skips → behavior
// ===========================================================================

#[test]
fn filter_3skip_vec_option_box() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_3skip_box_arc_option() {
    let ty: Type = parse_quote!(Box<Arc<Option<f64>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Option"]));
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn filter_3skip_partial_match() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_3skip_none_match() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "HashMap < String , i32 >");
}

// ===========================================================================
// 15. Nested: Vec<Option<Box<i32>>> with various filter params
// ===========================================================================

#[test]
fn deeply_nested_filter_none() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Vec < Option < Box < i32 > > >");
}

#[test]
fn deeply_nested_filter_vec_only() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Option < Box < i32 > >");
}

#[test]
fn deeply_nested_filter_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "Box < i32 >");
}

#[test]
fn deeply_nested_filter_all_three() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn deeply_nested_filter_only_middle() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    // Only Option in skip: Vec is not skipped, so returns unchanged
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Vec < Option < Box < i32 > > >");
}

#[test]
fn deeply_nested_filter_skip_inner_only() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    // Only Box in skip: Vec not matched, returns unchanged
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < Option < Box < i32 > > >");
}

#[test]
fn deeply_nested_extract_after_filter() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    let (extracted, found) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "Box < i32 >");
}

#[test]
fn deeply_nested_wrap_after_full_filter() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

// ===========================================================================
// 16. filter preserves non-matching types
// ===========================================================================

#[test]
fn filter_preserves_plain_i32() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_preserves_string() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_preserves_custom_type() {
    let ty: Type = parse_quote!(MyStruct);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "MyStruct");
}

#[test]
fn filter_preserves_path_type() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&result),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn filter_preserves_tuple_struct() {
    let ty: Type = parse_quote!(MyWrapper<u8>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "MyWrapper < u8 >");
}

// ===========================================================================
// 17. filter + extract agree on matching types
// ===========================================================================

#[test]
fn filter_extract_agree_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn filter_extract_agree_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn filter_extract_agree_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let (extracted, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn filter_extract_agree_nested_two_layers() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    // extract Vec skipping Option
    let (_extracted, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    // extracted = Option<i32>, filter yields i32
    // These differ: filtered = i32, extracted = Option<i32>
    // But if we extract with skip, they should agree
    let (deep_extracted, deep_found) = try_extract_inner_type(&ty, "Option", &skip(&["Vec"]));
    assert!(deep_found);
    assert_eq!(ty_str(&filtered), ty_str(&deep_extracted));
}

#[test]
fn filter_extract_agree_single_wrapper_no_skip() {
    let ty: Type = parse_quote!(Arc<f64>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    let (extracted, found) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
    assert_eq!(ty_str(&filtered), "f64");
}

// ===========================================================================
// 18. Triple composition
// ===========================================================================

#[test]
fn triple_filter_filter_wrap() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let step1 = filter_inner_type(&ty, &skip(&["Vec"]));
    let step2 = filter_inner_type(&step1, &skip(&["Option"]));
    let step3 = wrap_leaf_type(&step2, &skip(&[]));
    assert_eq!(ty_str(&step3), "adze :: WithLeaf < i32 >");
}

#[test]
fn triple_extract_filter_wrap() {
    let ty: Type = parse_quote!(Option<Vec<Box<u32>>>);
    let (step1, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    let step2 = filter_inner_type(&step1, &skip(&["Vec", "Box"]));
    let step3 = wrap_leaf_type(&step2, &skip(&[]));
    assert_eq!(ty_str(&step3), "adze :: WithLeaf < u32 >");
}

#[test]
fn triple_wrap_filter_extract() {
    let ty: Type = parse_quote!(Vec<i32>);
    let step1 = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // Vec<adze::WithLeaf<i32>>
    let step2 = filter_inner_type(&step1, &skip(&["Vec"]));
    // adze::WithLeaf<i32>
    let (step3, found) = try_extract_inner_type(&step2, "WithLeaf", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&step3), "i32");
}

#[test]
fn triple_filter_extract_wrap_deep() {
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    let (step2, found) = try_extract_inner_type(&step1, "Option", &skip(&[]));
    assert!(found);
    let step3 = wrap_leaf_type(&step2, &skip(&["Vec"]));
    assert_eq!(ty_str(&step3), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn triple_wrap_wrap_filter() {
    let ty: Type = parse_quote!(i32);
    let w1 = wrap_leaf_type(&ty, &skip(&[]));
    let w2 = wrap_leaf_type(&w1, &skip(&[]));
    let filtered = filter_inner_type(&w2, &skip(&[]));
    // WithLeaf not in skip, so unchanged
    assert_eq!(
        ty_str(&filtered),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

// ===========================================================================
// 19. Quadruple composition
// ===========================================================================

#[test]
fn quad_filter_filter_filter_wrap() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let s1 = filter_inner_type(&ty, &skip(&["Vec"]));
    let s2 = filter_inner_type(&s1, &skip(&["Option"]));
    let s3 = filter_inner_type(&s2, &skip(&["Box"]));
    let s4 = wrap_leaf_type(&s3, &skip(&[]));
    assert_eq!(ty_str(&s4), "adze :: WithLeaf < i32 >");
}

#[test]
fn quad_wrap_filter_wrap_filter() {
    let ty: Type = parse_quote!(Vec<i32>);
    let s1 = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // Vec<adze::WithLeaf<i32>>
    let s2 = filter_inner_type(&s1, &skip(&["Vec"]));
    // adze::WithLeaf<i32>
    let s3 = wrap_leaf_type(&s2, &skip(&[]));
    // adze::WithLeaf<adze::WithLeaf<i32>>
    let s4 = filter_inner_type(&s3, &skip(&[]));
    // not in skip → unchanged
    assert_eq!(ty_str(&s4), "adze :: WithLeaf < adze :: WithLeaf < i32 > >");
}

#[test]
fn quad_extract_wrap_filter_extract() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (s1, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(found);
    assert_eq!(ty_str(&s1), "u8");
    let s2 = wrap_leaf_type(&s1, &skip(&[]));
    // adze::WithLeaf<u8>
    let s3 = filter_inner_type(&s2, &skip(&[]));
    // unchanged
    let (s4, found2) = try_extract_inner_type(&s3, "WithLeaf", &skip(&[]));
    assert!(found2);
    assert_eq!(ty_str(&s4), "u8");
}

#[test]
fn quad_filter_extract_filter_wrap() {
    let ty: Type = parse_quote!(Box<Option<Vec<Arc<i32>>>>);
    let s1 = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&s1), "Option < Vec < Arc < i32 > > >");
    let (s2, found) = try_extract_inner_type(&s1, "Vec", &skip(&["Option"]));
    assert!(found);
    assert_eq!(ty_str(&s2), "Arc < i32 >");
    let s3 = filter_inner_type(&s2, &skip(&["Arc"]));
    assert_eq!(ty_str(&s3), "i32");
    let s4 = wrap_leaf_type(&s3, &skip(&[]));
    assert_eq!(ty_str(&s4), "adze :: WithLeaf < i32 >");
}

#[test]
fn quad_wrap_extract_wrap_extract_roundtrip() {
    let ty: Type = parse_quote!(i32);
    let s1 = wrap_leaf_type(&ty, &skip(&[]));
    let (s2, found1) = try_extract_inner_type(&s1, "WithLeaf", &skip(&[]));
    assert!(found1);
    assert_eq!(ty_str(&s2), "i32");
    let s3 = wrap_leaf_type(&s2, &skip(&[]));
    let (s4, found2) = try_extract_inner_type(&s3, "WithLeaf", &skip(&[]));
    assert!(found2);
    assert_eq!(ty_str(&s4), "i32");
}

// ===========================================================================
// 20. All operations preserve type validity
// ===========================================================================

#[test]
fn validity_filter_produces_parseable_type() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    let reparsed: Type = syn::parse_str(&ty_str(&filtered)).expect("valid type");
    assert_eq!(ty_str(&reparsed), ty_str(&filtered));
}

#[test]
fn validity_wrap_produces_parseable_type() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let reparsed: Type = syn::parse_str(&ty_str(&wrapped)).expect("valid type");
    assert_eq!(ty_str(&reparsed), ty_str(&wrapped));
}

#[test]
fn validity_extract_produces_parseable_type() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Box", &skip(&["Option"]));
    assert!(found);
    let reparsed: Type = syn::parse_str(&ty_str(&extracted)).expect("valid type");
    assert_eq!(ty_str(&reparsed), ty_str(&extracted));
}

#[test]
fn validity_triple_composition_parseable() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let s1 = filter_inner_type(&ty, &skip(&["Vec"]));
    let (s2, found) = try_extract_inner_type(&s1, "Option", &skip(&[]));
    assert!(found);
    let s3 = wrap_leaf_type(&s2, &skip(&["Box"]));
    let reparsed: Type = syn::parse_str(&ty_str(&s3)).expect("valid type");
    assert_eq!(ty_str(&reparsed), ty_str(&s3));
}

#[test]
fn validity_quad_composition_parseable() {
    let ty: Type = parse_quote!(Box<Arc<Option<Vec<u32>>>>);
    let s1 = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    let s2 = filter_inner_type(&s1, &skip(&["Option"]));
    let (s3, found) = try_extract_inner_type(&s2, "Vec", &skip(&[]));
    assert!(found);
    let s4 = wrap_leaf_type(&s3, &skip(&[]));
    let reparsed: Type = syn::parse_str(&ty_str(&s4)).expect("valid type");
    assert_eq!(ty_str(&reparsed), ty_str(&s4));
}

#[test]
fn validity_deeply_nested_all_ops() {
    let types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(Vec<String>),
        parse_quote!(Option<Box<u8>>),
        parse_quote!(Vec<Option<Box<Arc<f64>>>>),
    ];
    let skip_set = skip(&["Vec", "Option", "Box", "Arc"]);
    for ty in &types {
        let filtered = filter_inner_type(ty, &skip_set);
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        let reparsed: Type = syn::parse_str(&ty_str(&wrapped)).expect("valid type");
        assert_eq!(ty_str(&reparsed), ty_str(&wrapped));
    }
}

// ===========================================================================
// Additional composition edge cases
// ===========================================================================

#[test]
fn filter_empty_skip_then_extract_no_change() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
    let (extracted, found) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "i32");
}

#[test]
fn sequential_filter_equals_combined_filter() {
    let ty: Type = parse_quote!(Box<Arc<Vec<i32>>>);
    let seq_result = {
        let s1 = filter_inner_type(&ty, &skip(&["Box"]));
        let s2 = filter_inner_type(&s1, &skip(&["Arc"]));
        filter_inner_type(&s2, &skip(&["Vec"]))
    };
    let combined_result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Vec"]));
    assert_eq!(ty_str(&seq_result), ty_str(&combined_result));
    assert_eq!(ty_str(&combined_result), "i32");
}

#[test]
fn filter_preserves_non_path_reference_type() {
    let ty: Type = parse_quote!(&i32);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "& i32");
}

#[test]
fn wrap_then_extract_with_leaf_roundtrip_various_types() {
    let types: Vec<Type> = vec![
        parse_quote!(u8),
        parse_quote!(String),
        parse_quote!(bool),
        parse_quote!(f64),
    ];
    for ty in &types {
        let wrapped = wrap_leaf_type(ty, &skip(&[]));
        let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&[]));
        assert!(found);
        assert_eq!(ty_str(&extracted), ty_str(ty));
    }
}

#[test]
fn filter_then_wrap_with_matching_skip_container() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // Option<i32>
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Option"]));
    // Option<adze::WithLeaf<i32>>
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn filter_extract_disagree_when_target_differs() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // filtered = Option<i32>
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    // extracted = Option<i32>
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn wrap_nested_option_vec_then_filter_both() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    // Option<Vec<adze::WithLeaf<bool>>>
    let filtered = filter_inner_type(&wrapped, &skip(&["Option", "Vec"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < bool >");
}

#[test]
fn compose_on_already_wrapped_type() {
    let ty: Type = parse_quote!(adze::WithLeaf<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // WithLeaf not in skip → unchanged
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
    let rewrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(
        ty_str(&rewrapped),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn filter_arc_then_extract_option() {
    let ty: Type = parse_quote!(Arc<Option<Vec<u16>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Option < Vec < u16 > >");
    let (extracted, found) = try_extract_inner_type(&filtered, "Vec", &skip(&["Option"]));
    assert!(found);
    assert_eq!(ty_str(&extracted), "u16");
}

#[test]
fn filter_wrap_filter_is_not_identity() {
    let ty: Type = parse_quote!(Vec<i32>);
    let s1 = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&s1), "i32");
    let s2 = wrap_leaf_type(&s1, &skip(&[]));
    assert_eq!(ty_str(&s2), "adze :: WithLeaf < i32 >");
    let s3 = filter_inner_type(&s2, &skip(&["Vec"]));
    // WithLeaf not Vec, so unchanged
    assert_eq!(ty_str(&s3), "adze :: WithLeaf < i32 >");
}
