#![allow(dead_code)]

use quote::ToTokens;
use std::collections::HashSet;
use syn::Type;
use syn::parse_quote;

use adze_common::filter_inner_type;
use adze_common::try_extract_inner_type;
use adze_common::wrap_leaf_type;

// Helper function to compare types as strings
#[allow(dead_code)]
fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// Helper function to normalize whitespace in type strings for comparison
#[allow(dead_code)]
fn normalize_type_string(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join("")
}

// ===== Category 1: filter_inner_type basics (8 tests) =====

#[test]
fn filter_option_single_wrapper() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_vec_single_wrapper() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "i32");
}

#[test]
fn filter_box_single_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<u64>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "u64");
}

#[test]
fn filter_bare_type_unchanged() {
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_nested_option_vec() {
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_with_nonempty_skip_set() {
    let skip: HashSet<&str> = ["Arc", "Mutex"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // Box is not in skip set, so it's returned unchanged
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Box<String>"
    );
}

#[test]
fn filter_empty_skip_set_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    // Nothing to filter, returns original
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Box<Option<String>>"
    );
}

#[test]
fn filter_unknown_wrapper_returns_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // Option is not in skip set
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Option<String>"
    );
}

// ===== Category 2: filter_inner_type with skip variations (8 tests) =====

#[test]
fn filter_skip_option_only() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Box<Vec<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    // Only Option is skipped, Box and Vec remain
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Box<Vec<String>>"
    );
}

#[test]
fn filter_skip_vec_only() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    // Vec is skipped, Box remains
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Box<String>"
    );
}

#[test]
fn filter_skip_box_only() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_skip_multiple_wrappers() {
    let skip: HashSet<&str> = ["Box", "Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_skip_all_known_wrappers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Vec", "Option", "Result"]
        .into_iter()
        .collect();
    let ty: Type = parse_quote!(Arc<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_skip_none_with_explicit_empty_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // Empty skip set means no filtering
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Vec<String>"
    );
}

#[test]
fn filter_skip_nonmatching_wrapper() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // Box is not in skip set
    assert_eq!(
        normalize_type_string(type_to_string(&filtered)),
        "Box<String>"
    );
}

#[test]
fn filter_skip_with_deeply_nested_types() {
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Box<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

// ===== Category 3: try_extract_inner_type basics (8 tests) =====

#[test]
fn extract_option_found() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_vec_found() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "i32");
}

#[test]
fn extract_box_found() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "u64");
}

#[test]
fn extract_not_found_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_wrong_target_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "Vec<String>");
}

#[test]
fn extract_with_skip_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_bare_type_not_found() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_target_wrapped_in_skip() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "i32");
}

// ===== Category 4: try_extract_inner_type with skip variations (8 tests) =====

#[test]
fn extract_skip_option_no_match() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(
        normalize_type_string(type_to_string(&inner)),
        "Option<String>"
    );
}

#[test]
fn extract_skip_vec_nested_option() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_skip_box_with_target() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_skip_multiple_looking_for_one() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "i32");
}

#[test]
fn extract_skip_all_but_target_not_present() {
    let skip: HashSet<&str> = ["Box", "Arc", "Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip);
    assert!(!extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "Arc<String>");
}

#[test]
fn extract_skip_doesnt_affect_matching() {
    let skip: HashSet<&str> = ["Mutex"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

#[test]
fn extract_skip_stops_on_non_skip_non_target() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(
        normalize_type_string(type_to_string(&inner)),
        "Box<Vec<String>>"
    );
}

#[test]
fn extract_complex_nested_with_skip() {
    let skip: HashSet<&str> = ["Box", "Arc", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<Option<String>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(normalize_type_string(type_to_string(&inner)), "String");
}

// ===== Category 5: wrap_leaf_type basics (8 tests) =====

#[test]
fn wrap_string_bare_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<String>"
    );
}

#[test]
fn wrap_u32_primitive() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<u32>"
    );
}

#[test]
fn wrap_bool_primitive() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<bool>"
    );
}

#[test]
fn wrap_custom_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<MyCustomType>"
    );
}

#[test]
fn wrap_path_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(normalize_type_string(type_to_string(&wrapped)).contains("adze::WithLeaf"));
}

#[test]
fn wrap_vec_skips_wrapper() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec is skipped, but inner String is wrapped
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Vec<adze::WithLeaf<String>>"
    );
}

#[test]
fn wrap_option_with_skip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Option is skipped, but inner i32 is wrapped
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Option<adze::WithLeaf<i32>>"
    );
}

#[test]
fn wrap_nested_with_skip() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec is skipped, Option and String should be wrapped
    let wrapped_str = normalize_type_string(type_to_string(&wrapped));
    assert!(wrapped_str.contains("Vec") && wrapped_str.contains("WithLeaf"));
}

// ===== Category 6: wrap_leaf_type with skip variations (8 tests) =====

#[test]
fn wrap_skip_vec_only() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<u64>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Vec<adze::WithLeaf<u64>>"
    );
}

#[test]
fn wrap_skip_option_only() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Option<adze::WithLeaf<bool>>"
    );
}

#[test]
fn wrap_skip_box_only() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Box<adze::WithLeaf<u8>>"
    );
}

#[test]
fn wrap_skip_multiple_wrappers() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Both Vec and Option are skipped, only i32 is wrapped
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "Vec<Option<adze::WithLeaf<i32>>>"
    );
}

#[test]
fn wrap_skip_empty_set_wraps_all() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // No skip, so entire Vec<String> is wrapped
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<Vec<String>>"
    );
}

#[test]
fn wrap_skip_nonmatching_doesnt_affect() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec not in skip set, so entire Vec<String> is wrapped
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<Vec<String>>"
    );
}

#[test]
fn wrap_skip_result_both_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Result is skipped, both type args wrapped
    let wrapped_str = normalize_type_string(type_to_string(&wrapped));
    assert!(wrapped_str.contains("Result"));
    assert!(wrapped_str.contains("WithLeaf"));
}

#[test]
fn wrap_skip_deeply_nested() {
    let skip: HashSet<&str> = ["Vec", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Box<Option<String>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec and Box are skipped, Option and String wrapped
    let wrapped_str = normalize_type_string(type_to_string(&wrapped));
    assert!(wrapped_str.contains("Vec") && wrapped_str.contains("Box"));
}

// ===== Category 7: Combined operations (8 tests) =====

#[test]
fn extract_then_filter() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (extracted, was_extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(was_extracted);
    let filtered = filter_inner_type(&extracted, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn filter_then_wrap() {
    let skip: HashSet<&str> = ["Box", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    let wrapped = wrap_leaf_type(&filtered, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<String>"
    );
}

#[test]
fn extract_then_wrap() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let (extracted, _) = try_extract_inner_type(&ty, "Option", &skip);
    let wrapped = wrap_leaf_type(&extracted, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<i32>"
    );
}

#[test]
fn wrap_with_skip_then_filter() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let filtered = filter_inner_type(&wrapped, &skip);
    // wrapped is Vec<WithLeaf<String>>, filtering out Vec gives WithLeaf<String>
    assert!(normalize_type_string(type_to_string(&filtered)).contains("WithLeaf"));
}

#[test]
fn filter_all_layers_chain() {
    let skip: HashSet<&str> = ["Box", "Option", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn round_trip_extract_and_wrap() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (extracted, was_extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    // try_extract_inner_type returns the INNER type of Vec, which is String
    assert!(was_extracted);
    assert_eq!(normalize_type_string(type_to_string(&extracted)), "String");
    let wrapped = wrap_leaf_type(&extracted, &skip);
    // String gets wrapped in WithLeaf
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<String>"
    );
}

#[test]
fn identity_wrap_then_filter_no_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // wrapped is adze::WithLeaf<String>
    let filtered = filter_inner_type(&wrapped, &skip);
    // No skip set, returns unchanged
    assert!(normalize_type_string(type_to_string(&filtered)).contains("WithLeaf"));
}

#[test]
fn complex_chain_extract_filter_wrap() {
    let skip: HashSet<&str> = ["Box", "Arc", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Box<Vec<String>>>);
    let (extracted, _) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted.to_token_stream().to_string().contains("String"));
    assert_eq!(normalize_type_string(type_to_string(&extracted)), "String");
    let filtered = filter_inner_type(&extracted, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip);
    assert_eq!(
        normalize_type_string(type_to_string(&wrapped)),
        "adze::WithLeaf<String>"
    );
}

// ===== Category 8: Type edge cases and complex scenarios (8 tests) =====

#[test]
fn filter_deeply_nested_generics() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Box<Box<String>>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "String");
}

#[test]
fn extract_from_deeply_nested_generics() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Box<Vec<String>>>>);
    let (extracted, was_extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(was_extracted);
    assert_eq!(normalize_type_string(type_to_string(&extracted)), "String");
}

#[test]
fn wrap_non_path_type_array() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(normalize_type_string(type_to_string(&wrapped)).contains("WithLeaf"));
}

#[test]
fn wrap_non_path_type_tuple() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((String, i32));
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(normalize_type_string(type_to_string(&wrapped)).contains("WithLeaf"));
}

#[test]
fn filter_with_reference_type() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(normalize_type_string(type_to_string(&filtered)), "&str");
}

#[test]
fn wrap_reference_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(normalize_type_string(type_to_string(&wrapped)).contains("WithLeaf"));
}

#[test]
fn extract_option_result_with_skip() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<Option<String>, i32>);
    let (extracted, was_extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(was_extracted);
    assert_eq!(normalize_type_string(type_to_string(&extracted)), "String");
}

#[test]
fn wrap_result_with_both_generic_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, Box<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let wrapped_str = normalize_type_string(type_to_string(&wrapped));
    // Both String and Box<i32> should have inner types wrapped
    assert!(wrapped_str.contains("Result"));
    assert!(wrapped_str.contains("WithLeaf"));
}
