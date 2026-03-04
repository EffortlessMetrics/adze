//! Comprehensive tests for token attribute parsing in the adze-macro crate.
//!
//! Exercises `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! `NameValueExpr`, and `FieldThenParams` across a wide range of Rust types,
//! nesting depths, skip-over sets, and edge cases.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn tok(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn box_skip() -> HashSet<&'static str> {
    HashSet::from(["Box"])
}

fn box_arc_skip() -> HashSet<&'static str> {
    HashSet::from(["Box", "Arc"])
}

fn vec_option_skip() -> HashSet<&'static str> {
    HashSet::from(["Vec", "Option"])
}

fn full_skip() -> HashSet<&'static str> {
    HashSet::from(["Box", "Arc", "Vec", "Option", "Spanned"])
}

// =============================================================================
// 1. try_extract_inner_type — Option<T>
// =============================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "i32");
}

#[test]
fn extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "bool");
}

#[test]
fn extract_option_unit_tuple() {
    let ty: Type = parse_quote!(Option<()>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "()");
}

// =============================================================================
// 2. try_extract_inner_type — Vec<T>
// =============================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u8");
}

#[test]
fn extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "f64");
}

// =============================================================================
// 3. try_extract_inner_type — Box<T>
// =============================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_box_custom_type() {
    let ty: Type = parse_quote!(Box<MyExpr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "MyExpr");
}

// =============================================================================
// 4. try_extract_inner_type — not found / mismatch
// =============================================================================

#[test]
fn extract_no_match_plain_type() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_no_match_wrong_container() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "Vec < String >");
}

#[test]
fn extract_non_path_reference() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "& str");
}

#[test]
fn extract_non_path_tuple() {
    let ty: Type = parse_quote!((i32, u64));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "(i32 , u64)");
}

// =============================================================================
// 5. try_extract_inner_type — with skip_over
// =============================================================================

#[test]
fn extract_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Option<u32>>);
    let skip = HashSet::from(["Arc"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(tok(&inner), "u32");
}

#[test]
fn extract_through_box_arc_skip() {
    let ty: Type = parse_quote!(Box<Arc<Vec<i32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_arc_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "i32");
}

#[test]
fn skip_box_but_inner_not_target() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "Box < String >");
}

#[test]
fn skip_does_not_match_outer() {
    let ty: Type = parse_quote!(Rc<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "Rc < Vec < String > >");
}

// =============================================================================
// 6. try_extract_inner_type — nested generics
// =============================================================================

#[test]
fn extract_option_vec_nested() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "Vec < String >");
}

#[test]
fn extract_vec_option_nested() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "Option < i32 >");
}

#[test]
fn extract_nested_option_skip_vec() {
    // Skip over Vec, look for Option inside: Vec<Option<T>> → T
    let skip = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(tok(&inner), "bool");
}

// =============================================================================
// 7. try_extract_inner_type — multiple skip-over types
// =============================================================================

#[test]
fn extract_through_two_skips() {
    let ty: Type = parse_quote!(Box<Arc<Option<f32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &box_arc_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "f32");
}

#[test]
fn extract_full_skip_chain() {
    let ty: Type = parse_quote!(Box<Option<u16>>);
    let skip = HashSet::from(["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(tok(&inner), "u16");
}

// =============================================================================
// 8. filter_inner_type
// =============================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "String");
}

#[test]
fn filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    assert_eq!(tok(&filter_inner_type(&ty, &box_arc_skip())), "String");
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(String);
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "String");
}

#[test]
fn filter_non_skip_container_unchanged() {
    let ty: Type = parse_quote!(Rc<String>);
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "Rc < String >");
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        tok(&filter_inner_type(&ty, &empty_skip())),
        "Box < String >"
    );
}

#[test]
fn filter_nested_three_deep() {
    let ty: Type = parse_quote!(Box<Arc<Box<i32>>>);
    let skip = HashSet::from(["Box", "Arc"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_non_path_tuple_unchanged() {
    let ty: Type = parse_quote!((u8, u16));
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "(u8 , u16)");
}

#[test]
fn filter_non_path_reference_unchanged() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "& str");
}

#[test]
fn filter_vec_in_skip() {
    let ty: Type = parse_quote!(Vec<u32>);
    let skip = HashSet::from(["Vec"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "u32");
}

#[test]
fn filter_option_in_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip = HashSet::from(["Option"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "bool");
}

// =============================================================================
// 9. wrap_leaf_type
// =============================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_vec_skipped_inner_wrapped() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_skipped_inner_wrapped() {
    let ty: Type = parse_quote!(Option<u64>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Option < adze :: WithLeaf < u64 > >"
    );
}

#[test]
fn wrap_nested_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_option_vec_nested() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_non_skip_container_wraps_whole() {
    let ty: Type = parse_quote!(Rc<String>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "adze :: WithLeaf < Rc < String > >"
    );
}

#[test]
fn wrap_non_path_reference() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_non_path_tuple() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < (i32 , u32) >"
    );
}

#[test]
fn wrap_non_path_array() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let skip = HashSet::from(["Result"]);
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &skip)),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// =============================================================================
// 10. Primitive type handling
// =============================================================================

#[test]
fn extract_option_u8() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u8");
}

#[test]
fn extract_option_u16() {
    let ty: Type = parse_quote!(Option<u16>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u16");
}

#[test]
fn extract_option_u32() {
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u32");
}

#[test]
fn extract_option_u64() {
    let ty: Type = parse_quote!(Option<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u64");
}

#[test]
fn extract_option_usize() {
    let ty: Type = parse_quote!(Option<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "usize");
}

#[test]
fn extract_option_isize() {
    let ty: Type = parse_quote!(Option<isize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "isize");
}

#[test]
fn extract_option_f32() {
    let ty: Type = parse_quote!(Option<f32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "f32");
}

#[test]
fn extract_option_char() {
    let ty: Type = parse_quote!(Option<char>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "char");
}

// =============================================================================
// 11. Complex path types
// =============================================================================

#[test]
fn extract_qualified_path() {
    let ty: Type = parse_quote!(std::vec::Vec<String>);
    // The last segment is Vec, so extraction should work.
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_custom_module_type() {
    let ty: Type = parse_quote!(my_mod::MyType);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "my_mod :: MyType");
}

#[test]
fn filter_qualified_box() {
    let ty: Type = parse_quote!(std::boxed::Box<u32>);
    // Last segment is Box, which is in the skip set.
    assert_eq!(tok(&filter_inner_type(&ty, &box_skip())), "u32");
}

#[test]
fn wrap_qualified_vec_skipped() {
    let ty: Type = parse_quote!(std::vec::Vec<String>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "std :: vec :: Vec < adze :: WithLeaf < String > >"
    );
}

// =============================================================================
// 12. NameValueExpr parsing
// =============================================================================

#[test]
fn name_value_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nv.path.to_string(), "text");
}

#[test]
fn name_value_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
}

#[test]
fn name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
}

#[test]
fn name_value_path_expr() {
    let nv: NameValueExpr = parse_quote!(kind = SomeEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
}

// =============================================================================
// 13. FieldThenParams parsing
// =============================================================================

#[test]
fn field_only_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(String, text = "hello");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "text");
}

#[test]
fn field_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(i32, min = 0, max = 100);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn field_with_three_params() {
    let ftp: FieldThenParams = parse_quote!(bool, x = 1, y = 2, z = 3);
    assert_eq!(ftp.params.len(), 3);
}

#[test]
fn field_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    assert!(ftp.params.is_empty());
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Vec"));
}

#[test]
fn field_generic_type_with_param() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>, default = 0);
    assert_eq!(ftp.params.len(), 1);
}

// =============================================================================
// 14. Edge cases — empty and identity
// =============================================================================

#[test]
fn extract_from_plain_string_no_match() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &full_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn filter_plain_i32_unchanged() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(tok(&filter_inner_type(&ty, &full_skip())), "i32");
}

#[test]
fn wrap_bool_leaf() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < bool >"
    );
}

// =============================================================================
// 15. Spanned skip type
// =============================================================================

#[test]
fn extract_through_spanned() {
    let skip = HashSet::from(["Spanned"]);
    let ty: Type = parse_quote!(Spanned<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(tok(&inner), "u32");
}

#[test]
fn filter_spanned_unwrap() {
    let skip = HashSet::from(["Spanned"]);
    let ty: Type = parse_quote!(Spanned<MyNode>);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "MyNode");
}

#[test]
fn wrap_spanned_skipped() {
    let skip = HashSet::from(["Spanned"]);
    let ty: Type = parse_quote!(Spanned<MyLeaf>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &skip)),
        "Spanned < adze :: WithLeaf < MyLeaf > >"
    );
}

// =============================================================================
// 16. Additional wrap_leaf_type cases
// =============================================================================

#[test]
fn wrap_deeply_nested_vec_option_vec() {
    let ty: Type = parse_quote!(Vec<Option<Vec<String>>>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Vec < Option < Vec < adze :: WithLeaf < String > > > >"
    );
}

#[test]
fn wrap_option_option_nested() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Option < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_custom_type_no_skip() {
    let ty: Type = parse_quote!(MyExpr);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "adze :: WithLeaf < MyExpr >"
    );
}

// =============================================================================
// 17. syn::parse_str type construction
// =============================================================================

#[test]
fn parse_str_simple_type() {
    let ty: Type = syn::parse_str("u32").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(tok(&inner), "u32");
}

#[test]
fn parse_str_option_type() {
    let ty: Type = syn::parse_str("Option<String>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn parse_str_vec_type() {
    let ty: Type = syn::parse_str("Vec<u8>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "u8");
}

#[test]
fn parse_str_nested_type() {
    let ty: Type = syn::parse_str("Box<Vec<String>>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

// =============================================================================
// 18. Additional filter_inner_type edge cases
// =============================================================================

#[test]
fn filter_arc_in_skip() {
    let ty: Type = parse_quote!(Arc<u64>);
    let skip = HashSet::from(["Arc"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "u64");
}

#[test]
fn filter_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip = HashSet::from(["Option"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "Vec < i32 >");
}

#[test]
fn filter_option_vec_both_in_skip() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    assert_eq!(tok(&filter_inner_type(&ty, &vec_option_skip())), "String");
}

// =============================================================================
// 19. Reserved keyword `gen` as a type name
// =============================================================================

#[test]
fn extract_vec_of_raw_ident() {
    let ty: Type = parse_quote!(Vec<r#gen>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "r#gen");
}

#[test]
fn wrap_raw_ident_type() {
    let ty: Type = parse_quote!(r#gen);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < r#gen >"
    );
}

// =============================================================================
// 20. Remaining tests to reach 60+
// =============================================================================

#[test]
fn extract_vec_of_custom_struct() {
    let ty: Type = parse_quote!(Vec<MyStruct>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "MyStruct");
}

#[test]
fn extract_option_of_vec_inner() {
    // Extract Option itself, getting Vec<i32> as the inner type.
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(tok(&inner), "Vec < i32 >");
}

#[test]
fn filter_box_of_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip = HashSet::from(["Box"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "Option < String >");
}

#[test]
fn filter_box_option_both_skipped() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip = HashSet::from(["Box", "Option"]);
    assert_eq!(tok(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn wrap_vec_vec_nested() {
    let ty: Type = parse_quote!(Vec<Vec<u8>>);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &vec_option_skip())),
        "Vec < Vec < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wrap_f64_leaf() {
    let ty: Type = parse_quote!(f64);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < f64 >"
    );
}

#[test]
fn wrap_usize_leaf() {
    let ty: Type = parse_quote!(usize);
    assert_eq!(
        tok(&wrap_leaf_type(&ty, &empty_skip())),
        "adze :: WithLeaf < usize >"
    );
}

#[test]
fn name_value_negative_int() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
}

#[test]
fn field_then_params_option_field() {
    let ftp: FieldThenParams = parse_quote!(Option<String>, nullable = true);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "nullable");
}
