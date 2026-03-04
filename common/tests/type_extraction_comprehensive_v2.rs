// Comprehensive tests for common crate type extraction and transformation functions.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ===== NameValueExpr parsing =====

#[test]
fn name_value_string_literal() {
    let nv: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nv.path.to_string(), "name");
}

#[test]
fn name_value_integer_literal() {
    let nv: NameValueExpr = parse_quote!(count = 42);
    assert_eq!(nv.path.to_string(), "count");
}

#[test]
fn name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
}

#[test]
fn name_value_path_expr() {
    let nv: NameValueExpr = parse_quote!(kind = MyKind::Variant);
    assert_eq!(nv.path.to_string(), "kind");
}

#[test]
fn name_value_clone() {
    let nv: NameValueExpr = parse_quote!(x = 1);
    let nv2 = nv.clone();
    assert_eq!(nv.path.to_string(), nv2.path.to_string());
}

#[test]
fn name_value_debug() {
    let nv: NameValueExpr = parse_quote!(x = 1);
    let debug = format!("{:?}", nv);
    assert!(debug.contains("NameValueExpr"));
}

#[test]
fn name_value_eq() {
    let a: NameValueExpr = parse_quote!(x = 1);
    let b: NameValueExpr = parse_quote!(x = 1);
    assert_eq!(a, b);
}

#[test]
fn name_value_ne_different_name() {
    let a: NameValueExpr = parse_quote!(x = 1);
    let b: NameValueExpr = parse_quote!(y = 1);
    assert_ne!(a, b);
}

// ===== FieldThenParams parsing =====

#[test]
fn field_only_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn field_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(i32, min = 0, max = 100);
    assert_eq!(ftp.params.len(), 2);
}

#[test]
fn field_then_params_clone() {
    let ftp: FieldThenParams = parse_quote!(u64);
    let ftp2 = ftp.clone();
    assert_eq!(ftp, ftp2);
}

#[test]
fn field_then_params_debug() {
    let ftp: FieldThenParams = parse_quote!(bool);
    let debug = format!("{:?}", ftp);
    assert!(debug.contains("FieldThenParams"));
}

#[test]
fn field_then_params_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Vec"));
}

#[test]
fn field_then_params_option_type() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>, default = 0);
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Option"));
    assert_eq!(ftp.params.len(), 1);
}

// ===== try_extract_inner_type =====

#[test]
fn extract_vec_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_option_i32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn extract_not_target_returns_false() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

#[test]
fn extract_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u8");
}

#[test]
fn extract_through_arc() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Option<f64>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "f64");
}

#[test]
fn extract_through_double_skip() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<bool>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "bool");
}

#[test]
fn extract_skip_without_target_returns_false() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

#[test]
fn extract_non_path_type_not_extracted() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

#[test]
fn extract_tuple_type_not_extracted() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, u32));
    let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn extract_vec_of_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Option < String >");
}

#[test]
fn extract_option_of_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < u8 >");
}

// ===== filter_inner_type =====

#[test]
fn filter_box_string() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_arc_i32() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_no_skip_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn filter_non_skip_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < String >");
}

#[test]
fn filter_non_path_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "& str");
}

#[test]
fn filter_triple_nested() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<u64>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "u64");
}

#[test]
fn filter_plain_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

// ===== wrap_leaf_type =====

#[test]
fn wrap_simple_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_i32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_vec_wraps_inner() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_wraps_inner() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn wrap_nested_skip_types() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_non_path_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_array_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_result_both_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_vec_not_in_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec itself gets wrapped since it's not in skip
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Vec < String > >"
    );
}

// ===== Combined pipeline tests =====

#[test]
fn extract_then_wrap() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn filter_then_wrap() {
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<i64>);
    let filtered = filter_inner_type(&ty, &filter_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i64 >"
    );
}

#[test]
fn extract_filter_wrap_pipeline() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Option<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    let filtered = filter_inner_type(&inner, &HashSet::new());
    let wrapped = wrap_leaf_type(&filtered, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < u8 >"
    );
}

// ===== Determinism =====

#[test]
fn extract_deterministic() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (a, _) = try_extract_inner_type(&ty, "Vec", &skip);
    let (b, _) = try_extract_inner_type(&ty, "Vec", &skip);
    assert_eq!(
        a.to_token_stream().to_string(),
        b.to_token_stream().to_string()
    );
}

#[test]
fn filter_deterministic() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let a = filter_inner_type(&ty, &skip);
    let b = filter_inner_type(&ty, &skip);
    assert_eq!(
        a.to_token_stream().to_string(),
        b.to_token_stream().to_string()
    );
}

#[test]
fn wrap_deterministic() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<u32>);
    let a = wrap_leaf_type(&ty, &skip);
    let b = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        a.to_token_stream().to_string(),
        b.to_token_stream().to_string()
    );
}
