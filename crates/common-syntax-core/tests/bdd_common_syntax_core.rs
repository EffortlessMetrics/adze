use std::collections::HashSet;

use adze_common_syntax_core::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
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
fn given_field_then_params_with_multiple_pairs_when_parsing_then_key_names_round_trip() {
    let input: FieldThenParams = parse_quote!(String, pattern = r"\\d+", max = 4);

    assert_eq!(input.params[0].path.to_string(), "pattern");
    assert_eq!(input.params[1].path.to_string(), "max");
}

#[test]
fn given_name_value_expr_with_transform_when_try_extract_inner_type_runs_then_nonmatching_type_preserved()
 {
    let skip_over: HashSet<&str> = HashSet::new();
    let typed: Type = parse_quote!(Option<String>);
    let expr: NameValueExpr = parse_quote!(transform = |x| x + 1);

    let (_, extracted) = try_extract_inner_type(&typed, "Vec", &skip_over);
    assert_eq!(expr.path.to_string(), "transform");
    assert!(!extracted);
}
