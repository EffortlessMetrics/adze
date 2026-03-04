//! Comprehensive tests for grammar analysis and extraction in adze-common.
//!
//! Covers: type extraction, container filtering, leaf wrapping, attribute parsing
//! (NameValueExpr / FieldThenParams), edge cases, error handling, and composition.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
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
// 1. NameValueExpr parsing
// ===========================================================================

#[test]
fn nve_string_literal_value() {
    let nve: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn nve_integer_literal_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
}

#[test]
fn nve_bool_literal_value() {
    let nve: NameValueExpr = parse_quote!(flag = true);
    assert_eq!(nve.path.to_string(), "flag");
}

#[test]
fn nve_negative_integer_value() {
    let nve: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_path_expression_value() {
    let nve: NameValueExpr = parse_quote!(kind = SomeEnum::Variant);
    assert_eq!(nve.path.to_string(), "kind");
}

#[test]
fn nve_clone_and_eq() {
    let nve: NameValueExpr = parse_quote!(x = 1);
    let cloned = nve.clone();
    assert_eq!(nve, cloned);
}

#[test]
fn nve_debug_impl() {
    let nve: NameValueExpr = parse_quote!(key = "val");
    let dbg = format!("{:?}", nve);
    assert!(dbg.contains("NameValueExpr"));
}

// ===========================================================================
// 2. FieldThenParams parsing
// ===========================================================================

#[test]
fn ftp_type_only_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_single_param() {
    let ftp: FieldThenParams = parse_quote!(i32, name = "count");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn ftp_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(bool, a = 1, b = "two", c = true);
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[0].path.to_string(), "a");
    assert_eq!(ftp.params[1].path.to_string(), "b");
    assert_eq!(ftp.params[2].path.to_string(), "c");
}

#[test]
fn ftp_generic_type_with_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, separator = ",");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "separator");
}

#[test]
fn ftp_option_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_clone_and_eq() {
    let ftp: FieldThenParams = parse_quote!(u8);
    let cloned = ftp.clone();
    assert_eq!(ftp, cloned);
}

#[test]
fn ftp_debug_impl() {
    let ftp: FieldThenParams = parse_quote!(u64);
    let dbg = format!("{:?}", ftp);
    assert!(dbg.contains("FieldThenParams"));
}

// ===========================================================================
// 3. try_extract_inner_type — basic extraction
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_target_mismatch_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_nested_with_skip() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_double_skip_layers() {
    let ty: Type = parse_quote!(Arc<Box<Vec<bool>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_skip_but_no_target_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_non_path_type_reference() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_non_path_type_tuple() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn extract_non_path_type_array() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

#[test]
fn extract_plain_type_no_generics_no_match() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_target_directly_found_no_skip_needed() {
    let ty: Type = parse_quote!(Option<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u16 >");
}

// ===========================================================================
// 4. filter_inner_type — container unwrapping
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<bool>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "bool"
    );
}

#[test]
fn filter_not_in_skip_set_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < String >"
    );
}

#[test]
fn filter_non_path_type_passthrough() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_tuple_type_passthrough() {
    let ty: Type = parse_quote!((u8, u16));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(u8 , u16)"
    );
}

#[test]
fn filter_plain_type_no_generics() {
    let ty: Type = parse_quote!(usize);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "usize");
}

#[test]
fn filter_three_layers_deep() {
    let ty: Type = parse_quote!(Rc<Box<Arc<f64>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Rc", "Box", "Arc"]))),
        "f64"
    );
}

// ===========================================================================
// 5. wrap_leaf_type — WithLeaf wrapping
// ===========================================================================

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_vec_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Option<bool>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < bool > >"
    );
}

#[test]
fn wrap_nested_skip_types() {
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < u32 > > >"
    );
}

#[test]
fn wrap_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_non_path_reference() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 16]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 16] >"
    );
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (i32 , u32) >"
    );
}

#[test]
fn wrap_result_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

// ===========================================================================
// 6. Composition — extract then filter, extract then wrap, etc.
// ===========================================================================

#[test]
fn extract_then_wrap() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_then_filter() {
    let ty: Type = parse_quote!(Vec<Box<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < i32 >");
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_skip_then_filter_then_wrap() {
    let ty: Type = parse_quote!(Arc<Vec<Box<u64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < u64 >");
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "u64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
}

// ===========================================================================
// 7. Edge cases — unusual but valid types
// ===========================================================================

#[test]
fn extract_qualified_path_type() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    // Last segment is Vec, so extraction should work
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_qualified_path_type() {
    let ty: Type = parse_quote!(std::boxed::Box<f32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "f32");
}

#[test]
fn wrap_qualified_path_type_in_skip() {
    let ty: Type = parse_quote!(std::vec::Vec<i64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < i64 > >"
    );
}

#[test]
fn extract_with_lifetime_reference_no_match() {
    let ty: Type = parse_quote!(&'a str);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}

#[test]
fn extract_from_custom_generic() {
    let ty: Type = parse_quote!(MyContainer<Data>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyContainer", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Data");
}

#[test]
fn filter_custom_wrapper() {
    let ty: Type = parse_quote!(Wrapper<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Wrapper"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

#[test]
fn wrap_custom_skip_type() {
    let ty: Type = parse_quote!(Container<Leaf>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Container"]));
    assert_eq!(ty_str(&wrapped), "Container < adze :: WithLeaf < Leaf > >");
}

// ===========================================================================
// 8. Idempotency and identity properties
// ===========================================================================

#[test]
fn filter_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(String);
    let once = filter_inner_type(&ty, &skip(&["Box"]));
    let twice = filter_inner_type(&once, &skip(&["Box"]));
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn extract_no_match_returns_same_type() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_no_match_returns_same_type() {
    let ty: Type = parse_quote!(Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

// ===========================================================================
// 9. FieldThenParams — more attribute patterns
// ===========================================================================

#[test]
fn ftp_qualified_type() {
    let ftp: FieldThenParams = parse_quote!(std::string::String);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_reference_type() {
    let ftp: FieldThenParams = parse_quote!(&str);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_param_with_string_value() {
    let ftp: FieldThenParams = parse_quote!(Token, pattern = "[a-z]+");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn ftp_param_with_negative_int() {
    let ftp: FieldThenParams = parse_quote!(Expr, precedence = -5);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
}

// ===========================================================================
// 10. Multiple generic arguments in extraction
// ===========================================================================

#[test]
fn extract_first_arg_from_hashmap() {
    // try_extract_inner_type extracts the *first* generic argument
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_first_arg_from_result() {
    let ty: Type = parse_quote!(Result<u8, String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 11. Deeply nested types
// ===========================================================================

#[test]
fn extract_deeply_nested_three_skips() {
    let ty: Type = parse_quote!(A<B<C<Vec<u8>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["A", "B", "C"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_deeply_nested() {
    let ty: Type = parse_quote!(A<B<C<u8>>>);
    let filtered = filter_inner_type(&ty, &skip(&["A", "B", "C"]));
    assert_eq!(ty_str(&filtered), "u8");
}

#[test]
fn wrap_deeply_nested_skip() {
    let ty: Type = parse_quote!(Vec<Option<Vec<f32>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Vec < adze :: WithLeaf < f32 > > > >"
    );
}

// ===========================================================================
// 12. NameValueExpr — expression diversity
// ===========================================================================

#[test]
fn nve_float_literal() {
    let nve: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nve.path.to_string(), "weight");
}

#[test]
fn nve_char_literal() {
    let nve: NameValueExpr = parse_quote!(delim = ',');
    assert_eq!(nve.path.to_string(), "delim");
}

#[test]
fn nve_array_expression() {
    let nve: NameValueExpr = parse_quote!(items = [1, 2, 3]);
    assert_eq!(nve.path.to_string(), "items");
}

#[test]
fn nve_closure_expression() {
    let nve: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(nve.path.to_string(), "transform");
}

// ===========================================================================
// 13. Numeric and primitive types
// ===========================================================================

#[test]
fn wrap_all_integer_primitives() {
    for prim in [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "usize", "isize",
    ] {
        let ty: Type = syn::parse_str(prim).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[]));
        assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {} >", prim));
    }
}

#[test]
fn filter_box_of_each_float() {
    for float_ty in ["f32", "f64"] {
        let ty: Type = syn::parse_str(&format!("Box<{}>", float_ty)).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        assert_eq!(ty_str(&filtered), float_ty);
    }
}

// ===========================================================================
// 14. Empty and singleton skip sets
// ===========================================================================

#[test]
fn extract_empty_skip_set_direct_match() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_singleton_skip_with_match() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 15. Type identity preservation
// ===========================================================================

#[test]
fn filter_preserves_non_matching_generics() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , Vec < u8 > >");
}

#[test]
fn wrap_preserves_non_skip_structure() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    // HashMap not in skip set, so the whole thing gets wrapped
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn extract_preserves_inner_structure() {
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Box < String > >");
}
