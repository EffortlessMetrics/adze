use std::collections::HashSet;

use adze_common_type_ops_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

#[test]
fn given_leaf_annotation_without_transform_then_wrap_leaf_type_applies_outer_wrapper() {
    let skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let input: Type = parse_quote!(String);

    let wrapped = wrap_leaf_type(&input, &skip_over);

    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn given_nested_vec_when_filter_inner_type_is_called_then_option_and_box_are_stripped() {
    let mut skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    skip_over.insert("Arc");
    let input: Type = parse_quote!(Vec<Arc<Option<String>>>);

    let filtered = filter_inner_type(&input, &skip_over);

    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn given_nonmatching_target_when_try_extract_inner_type_runs_then_original_type_preserved() {
    let skip_over: HashSet<&str> = HashSet::new();
    let typed: Type = parse_quote!(Option<String>);

    let (_, extracted) = try_extract_inner_type(&typed, "Vec", &skip_over);
    assert!(!extracted);
}
