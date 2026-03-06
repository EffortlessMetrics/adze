//! Comprehensive v4 tests for adze-common-syntax-core syntax helpers.

use adze_common_syntax_core::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ── Helper constructors ──────────────────────────────────────────────

fn empty() -> HashSet<&'static str> {
    HashSet::new()
}

fn set_of(items: &[&'static str]) -> HashSet<&'static str> {
    items.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// 1. try_extract_inner_type — 12 tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn extract_option_string_direct() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_i32_direct() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_wrong_target_returns_false() {
    let ty: Type = parse_quote!(Option<u8>);
    let (ret, ok) = try_extract_inner_type(&ty, "Vec", &empty());
    assert!(!ok);
    assert_eq!(ty_str(&ret), "Option < u8 >");
}

#[test]
fn extract_plain_type_no_generics() {
    let ty: Type = parse_quote!(bool);
    let (ret, ok) = try_extract_inner_type(&ty, "Option", &empty());
    assert!(!ok);
    assert_eq!(ty_str(&ret), "bool");
}

#[test]
fn extract_skip_over_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<f64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &set_of(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_skip_over_arc_to_option() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &set_of(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_over_miss_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (ret, ok) = try_extract_inner_type(&ty, "Vec", &set_of(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&ret), "Box < String >");
}

#[test]
fn extract_double_skip_to_target() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u16>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &set_of(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn extract_reference_type_not_path() {
    let ty: Type = parse_quote!(&str);
    let (ret, ok) = try_extract_inner_type(&ty, "Option", &empty());
    assert!(!ok);
    assert_eq!(ty_str(&ret), "& str");
}

#[test]
fn extract_tuple_type_not_path() {
    let ty: Type = parse_quote!((i32, u64));
    let (ret, ok) = try_extract_inner_type(&ty, "Vec", &empty());
    assert!(!ok);
    assert_eq!(ty_str(&ret), "(i32 , u64)");
}

#[test]
fn extract_nested_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn extract_target_inside_non_skip_wrapper_fails() {
    // Rc is NOT in skip set so we can't see through it
    let ty: Type = parse_quote!(Rc<Vec<u8>>);
    let (ret, ok) = try_extract_inner_type(&ty, "Vec", &set_of(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&ret), "Rc < Vec < u8 > >");
}

// ═══════════════════════════════════════════════════════════════════════
// 2. filter_inner_type — 10 tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn filter_box_removes_wrapper() {
    let ty: Type = parse_quote!(Box<String>);
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "String");
}

#[test]
fn filter_arc_removes_wrapper() {
    let ty: Type = parse_quote!(Arc<u32>);
    let r = filter_inner_type(&ty, &set_of(&["Arc"]));
    assert_eq!(ty_str(&r), "u32");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<i64>>);
    let r = filter_inner_type(&ty, &set_of(&["Box", "Arc"]));
    assert_eq!(ty_str(&r), "i64");
}

#[test]
fn filter_non_skip_type_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "Vec < String >");
}

#[test]
fn filter_plain_primitive_unchanged() {
    let ty: Type = parse_quote!(u8);
    let r = filter_inner_type(&ty, &set_of(&["Box", "Arc"]));
    assert_eq!(ty_str(&r), "u8");
}

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "& [u8]");
}

#[test]
fn filter_empty_skip_set_no_change() {
    let ty: Type = parse_quote!(Box<String>);
    let r = filter_inner_type(&ty, &empty());
    assert_eq!(ty_str(&r), "Box < String >");
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<bool>>>);
    let r = filter_inner_type(&ty, &set_of(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&r), "bool");
}

#[test]
fn filter_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "Vec < String >");
}

#[test]
fn filter_tuple_type_unchanged() {
    let ty: Type = parse_quote!((u8, u16));
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "(u8 , u16)");
}

// ═══════════════════════════════════════════════════════════════════════
// 3. wrap_leaf_type — 10 tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    let r = wrap_leaf_type(&ty, &empty());
    assert_eq!(ty_str(&r), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_primitive_u64() {
    let ty: Type = parse_quote!(u64);
    let r = wrap_leaf_type(&ty, &empty());
    assert_eq!(ty_str(&r), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_skip_vec_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let r = wrap_leaf_type(&ty, &set_of(&["Vec"]));
    assert_eq!(ty_str(&r), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skip_option_wraps_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let r = wrap_leaf_type(&ty, &set_of(&["Option"]));
    assert_eq!(ty_str(&r), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_nested_skip_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let r = wrap_leaf_type(&ty, &set_of(&["Vec", "Option"]));
    assert_eq!(ty_str(&r), "Vec < Option < adze :: WithLeaf < bool > > >");
}

#[test]
fn wrap_non_skip_container_wraps_whole() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let r = wrap_leaf_type(&ty, &set_of(&["Vec"]));
    assert_eq!(ty_str(&r), "adze :: WithLeaf < HashMap < String , i32 > >");
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let r = wrap_leaf_type(&ty, &set_of(&["Result"]));
    assert_eq!(
        ty_str(&r),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let r = wrap_leaf_type(&ty, &empty());
    assert_eq!(ty_str(&r), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let r = wrap_leaf_type(&ty, &empty());
    assert_eq!(ty_str(&r), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_empty_skip_wraps_container() {
    let ty: Type = parse_quote!(Vec<u8>);
    let r = wrap_leaf_type(&ty, &empty());
    assert_eq!(ty_str(&r), "adze :: WithLeaf < Vec < u8 > >");
}

// ═══════════════════════════════════════════════════════════════════════
// 4. NameValueExpr parsing — 8 tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nve_string_literal() {
    let nve: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn nve_integer_literal() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
}

#[test]
fn nve_bool_literal() {
    let nve: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nve.path.to_string(), "enabled");
}

#[test]
fn nve_path_expr() {
    let nve: NameValueExpr = parse_quote!(mode = SomeEnum::Variant);
    assert_eq!(nve.path.to_string(), "mode");
}

#[test]
fn nve_negative_number() {
    let nve: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_has_eq_token() {
    let nve: NameValueExpr = parse_quote!(key = "val");
    // eq_token exists (unit struct, always present after successful parse)
    let _ = nve.eq_token;
}

#[test]
fn nve_underscore_ident() {
    let nve: NameValueExpr = parse_quote!(my_key = 99);
    assert_eq!(nve.path.to_string(), "my_key");
}

#[test]
fn nve_clone_eq() {
    let nve: NameValueExpr = parse_quote!(x = 1);
    let nve2 = nve.clone();
    assert_eq!(nve, nve2);
}

// ═══════════════════════════════════════════════════════════════════════
// 5. FieldThenParams parsing — 5 tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn ftp_type_only() {
    let ftp: FieldThenParams = parse_quote!(MyType);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(MyType, name = "test");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn ftp_type_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(String, key = "val", priority = 5);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "key");
    assert_eq!(ftp.params[1].path.to_string(), "priority");
}

#[test]
fn ftp_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<u8>, limit = 100);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "limit");
}

#[test]
fn ftp_clone_eq() {
    let ftp: FieldThenParams = parse_quote!(bool);
    let ftp2 = ftp.clone();
    assert_eq!(ftp, ftp2);
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Edge cases — 5+ tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn extract_qualified_path_type() {
    // std::vec::Vec<u8> — last segment is Vec so it should match
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_qualified_path_type() {
    let ty: Type = parse_quote!(std::boxed::Box<f32>);
    let r = filter_inner_type(&ty, &set_of(&["Box"]));
    assert_eq!(ty_str(&r), "f32");
}

#[test]
fn wrap_qualified_skip_wraps_inner() {
    let ty: Type = parse_quote!(std::vec::Vec<char>);
    let r = wrap_leaf_type(&ty, &set_of(&["Vec"]));
    assert_eq!(
        ty_str(&r),
        "std :: vec :: Vec < adze :: WithLeaf < char > >"
    );
}

#[test]
fn extract_deeply_nested_generics() {
    // Box<Arc<Box<Option<u8>>>> — skip Box and Arc, target Option
    let ty: Type = parse_quote!(Box<Arc<Box<Option<u8>>>>);
    let skip = set_of(&["Box", "Arc"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn wrap_nested_three_levels_skip() {
    let ty: Type = parse_quote!(Vec<Option<Vec<i32>>>);
    let skip = set_of(&["Vec", "Option"]);
    let r = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&r),
        "Vec < Option < Vec < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn nve_debug_impl() {
    let nve: NameValueExpr = parse_quote!(key = "v");
    let dbg = format!("{:?}", nve);
    assert!(dbg.contains("NameValueExpr"));
}

#[test]
fn ftp_debug_impl() {
    let ftp: FieldThenParams = parse_quote!(u32);
    let dbg = format!("{:?}", ftp);
    assert!(dbg.contains("FieldThenParams"));
}
