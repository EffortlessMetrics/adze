//! Comprehensive tests for grammar extraction functionality in adze-common.
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr parsing, FieldThenParams parsing, trait implementations,
//! determinism, edge cases, and composition patterns.

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

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type — direct match (no skip-over)
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_hashmap_first_arg() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
    // Extracts the first generic argument
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_result_first_arg() {
    let ty: Type = parse_quote!(Result<Token, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Token");
}

#[test]
fn extract_custom_container() {
    let ty: Type = parse_quote!(Repeated<Statement>);
    let (inner, ok) = try_extract_inner_type(&ty, "Repeated", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Statement");
}

// ===========================================================================
// 2. try_extract_inner_type — no match
// ===========================================================================

#[test]
fn extract_no_match_different_outer() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Vec < i32 >");
}

#[test]
fn extract_no_match_plain_type() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_no_match_primitive() {
    let ty: Type = parse_quote!(bool);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn extract_no_match_reference_type() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_no_match_tuple_type() {
    let ty: Type = parse_quote!((i32, u64));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "(i32 , u64)");
}

#[test]
fn extract_no_match_array_type() {
    let ty: Type = parse_quote!([u8; 16]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "[u8 ; 16]");
}

#[test]
fn extract_no_match_slice_ref() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& [u8]");
}

// ===========================================================================
// 3. try_extract_inner_type — with skip-over
// ===========================================================================

#[test]
fn extract_through_box() {
    let ty: Type = parse_quote!(Box<Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Leaf");
}

#[test]
fn extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Vec<Token>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Token");
}

#[test]
fn extract_through_two_layers() {
    let ty: Type = parse_quote!(Box<Arc<Option<Inner>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Inner");
}

#[test]
fn extract_through_three_layers() {
    let ty: Type = parse_quote!(Rc<Box<Arc<Vec<Item>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc", "Box", "Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Item");
}

#[test]
fn extract_skip_but_target_not_found() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < String >");
}

#[test]
fn extract_skip_nested_target_not_found() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < Arc < String > >");
}

// ===========================================================================
// 4. filter_inner_type — basic filtering
// ===========================================================================

#[test]
fn filter_box_strips_wrapper() {
    let ty: Type = parse_quote!(Box<Expr>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Expr");
}

#[test]
fn filter_arc_strips_wrapper() {
    let ty: Type = parse_quote!(Arc<Token>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Arc"]))), "Token");
}

#[test]
fn filter_rc_strips_wrapper() {
    let ty: Type = parse_quote!(Rc<Node>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Rc"]))), "Node");
}

#[test]
fn filter_no_match_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Vec < i32 >");
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&[]))), "Box < String >");
}

#[test]
fn filter_plain_type_passthrough() {
    let ty: Type = parse_quote!(i64);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "i64");
}

// ===========================================================================
// 5. filter_inner_type — nested filtering
// ===========================================================================

#[test]
fn filter_double_box() {
    let ty: Type = parse_quote!(Box<Box<Core>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Core");
}

#[test]
fn filter_box_arc_to_leaf() {
    let ty: Type = parse_quote!(Box<Arc<Leaf>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))), "Leaf");
}

#[test]
fn filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_three_nested_all_skipped() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Final>>>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "Final"
    );
}

// ===========================================================================
// 6. filter_inner_type — non-path types
// ===========================================================================

#[test]
fn filter_reference_type_passthrough() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_tuple_type_passthrough() {
    let ty: Type = parse_quote!((u32, bool));
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "(u32 , bool)");
}

#[test]
fn filter_unit_type_passthrough() {
    let ty: Type = parse_quote!(());
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "()");
}

#[test]
fn filter_array_type_passthrough() {
    let ty: Type = parse_quote!([u8; 32]);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "[u8 ; 32]");
}

// ===========================================================================
// 7. wrap_leaf_type — basic wrapping
// ===========================================================================

#[test]
fn wrap_plain_ident() {
    let ty: Type = parse_quote!(Identifier);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Identifier >"
    );
}

#[test]
fn wrap_primitive_type() {
    let ty: Type = parse_quote!(u32);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < u32 >"
    );
}

#[test]
fn wrap_bool_type() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < bool >"
    );
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

// ===========================================================================
// 8. wrap_leaf_type — skip-over containers
// ===========================================================================

#[test]
fn wrap_vec_skips_container_wraps_inner() {
    let ty: Type = parse_quote!(Vec<Token>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Token > >"
    );
}

#[test]
fn wrap_option_skips_container_wraps_inner() {
    let ty: Type = parse_quote!(Option<Expr>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < Expr > >"
    );
}

#[test]
fn wrap_vec_option_nested_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Leaf>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < Leaf > > >"
    );
}

#[test]
fn wrap_option_vec_nested_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < Node > > >"
    );
}

#[test]
fn wrap_no_skip_wraps_container_itself() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_result_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<Good, Bad>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < Good > , adze :: WithLeaf < Bad > >"
    );
}

// ===========================================================================
// 9. wrap_leaf_type — non-path types
// ===========================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (i32 , bool) >"
    );
}

// ===========================================================================
// 10. NameValueExpr parsing — various expression kinds
// ===========================================================================

#[test]
fn nve_string_value() {
    let nve: NameValueExpr = parse_quote!(pattern = "\\d+");
    assert_eq!(nve.path.to_string(), "pattern");
}

#[test]
fn nve_integer_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 10);
    assert_eq!(nve.path.to_string(), "precedence");
}

#[test]
fn nve_bool_value() {
    let nve: NameValueExpr = parse_quote!(inline = true);
    assert_eq!(nve.path.to_string(), "inline");
}

#[test]
fn nve_negative_value() {
    let nve: NameValueExpr = parse_quote!(offset = -5);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_path_value() {
    let nve: NameValueExpr = parse_quote!(kind = Assoc::Left);
    assert_eq!(nve.path.to_string(), "kind");
}

#[test]
fn nve_closure_value() {
    let nve: NameValueExpr = parse_quote!(transform = |s: String| s.len());
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn nve_block_value() {
    let nve: NameValueExpr = parse_quote!(init = { Vec::new() });
    assert_eq!(nve.path.to_string(), "init");
}

#[test]
fn nve_tuple_value() {
    let nve: NameValueExpr = parse_quote!(range = (0, 100));
    assert_eq!(nve.path.to_string(), "range");
}

// ===========================================================================
// 11. NameValueExpr trait implementations
// ===========================================================================

#[test]
fn nve_clone_produces_equal() {
    let nve: NameValueExpr = parse_quote!(key = 42);
    let cloned = nve.clone();
    assert_eq!(nve, cloned);
}

#[test]
fn nve_eq_same_content() {
    let a: NameValueExpr = parse_quote!(x = 1);
    let b: NameValueExpr = parse_quote!(x = 1);
    assert_eq!(a, b);
}

#[test]
fn nve_debug_contains_struct_name() {
    let nve: NameValueExpr = parse_quote!(key = "val");
    let dbg = format!("{:?}", nve);
    assert!(dbg.contains("NameValueExpr"));
}

#[test]
fn nve_debug_contains_path() {
    let nve: NameValueExpr = parse_quote!(my_key = 99);
    let dbg = format!("{:?}", nve);
    assert!(dbg.contains("path"));
}

// ===========================================================================
// 12. FieldThenParams parsing — basic
// ===========================================================================

#[test]
fn ftp_bare_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(u32);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_bare_string_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_generic_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<Token>);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(String, pattern = "abc");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn ftp_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(i32, min = 0, max = 100);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn ftp_with_three_params() {
    let ftp: FieldThenParams = parse_quote!(Token, name = "id", prec = 5, assoc = true);
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "prec");
    assert_eq!(ftp.params[2].path.to_string(), "assoc");
}

// ===========================================================================
// 13. FieldThenParams trait implementations
// ===========================================================================

#[test]
fn ftp_clone_produces_equal() {
    let ftp: FieldThenParams = parse_quote!(u32, key = 1);
    let cloned = ftp.clone();
    assert_eq!(ftp, cloned);
}

#[test]
fn ftp_debug_contains_struct_name() {
    let ftp: FieldThenParams = parse_quote!(bool);
    let dbg = format!("{:?}", ftp);
    assert!(dbg.contains("FieldThenParams"));
}

#[test]
fn ftp_eq_same_content() {
    let a: FieldThenParams = parse_quote!(String);
    let b: FieldThenParams = parse_quote!(String);
    assert_eq!(a, b);
}

// ===========================================================================
// 14. Determinism — same input produces identical output
// ===========================================================================

#[test]
fn determinism_extract_consistent() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    let (a, ok_a) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    let (b, ok_b) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert_eq!(ok_a, ok_b);
    assert_eq!(ts(&a), ts(&b));
}

#[test]
fn determinism_filter_consistent() {
    let ty: Type = parse_quote!(Box<Arc<Leaf>>);
    let s = &skip(&["Box", "Arc"]);
    let a = filter_inner_type(&ty, s);
    let b = filter_inner_type(&ty, s);
    assert_eq!(ts(&a), ts(&b));
}

#[test]
fn determinism_wrap_consistent() {
    let ty: Type = parse_quote!(Vec<Option<Tok>>);
    let s = &skip(&["Vec", "Option"]);
    let a = wrap_leaf_type(&ty, s);
    let b = wrap_leaf_type(&ty, s);
    assert_eq!(ts(&a), ts(&b));
}

// ===========================================================================
// 15. Composition — chaining extract, filter, and wrap
// ===========================================================================

#[test]
fn composition_extract_then_wrap() {
    let ty: Type = parse_quote!(Option<Ident>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Ident >");
}

#[test]
fn composition_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Token>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn composition_extract_then_filter() {
    let ty: Type = parse_quote!(Vec<Box<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Leaf");
}

#[test]
fn composition_filter_then_extract() {
    let ty: Type = parse_quote!(Box<Option<Item>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let (inner, ok) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Item");
}

#[test]
fn composition_full_pipeline() {
    let ty: Type = parse_quote!(Arc<Vec<Box<Leaf>>>);
    // 1. Filter off Arc
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ts(&filtered), "Vec < Box < Leaf > >");
    // 2. Extract from Vec
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Box < Leaf >");
    // 3. Filter off Box
    let final_ty = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ts(&final_ty), "Leaf");
    // 4. Wrap the leaf
    let wrapped = wrap_leaf_type(&final_ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Leaf >");
}

// ===========================================================================
// 16. Edge cases — qualified paths and complex type expressions
// ===========================================================================

#[test]
fn extract_qualified_path_type() {
    let ty: Type = parse_quote!(std::vec::Vec<Tok>);
    // The last segment is Vec so extraction should match
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Tok");
}

#[test]
fn filter_qualified_path_type() {
    let ty: Type = parse_quote!(std::boxed::Box<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Inner");
}

#[test]
fn wrap_qualified_path_type_not_skipped() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < std :: string :: String >");
}

#[test]
fn extract_with_lifetime_in_ref() {
    let ty: Type = parse_quote!(&'a str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& 'a str");
}

// ===========================================================================
// 17. Edge cases — idempotency
// ===========================================================================

#[test]
fn filter_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(Leaf);
    let once = filter_inner_type(&ty, &skip(&["Box"]));
    let twice = filter_inner_type(&once, &skip(&["Box"]));
    assert_eq!(ts(&once), ts(&twice));
}

#[test]
fn wrap_not_idempotent_wraps_again() {
    let ty: Type = parse_quote!(Token);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    // Second wrap adds another WithLeaf layer
    assert!(ts(&twice).contains("WithLeaf < adze :: WithLeaf"));
}

// ===========================================================================
// 18. Numeric primitive types through extraction pipeline
// ===========================================================================

#[test]
fn extract_vec_of_i8() {
    let ty: Type = parse_quote!(Vec<i8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i8");
}

#[test]
fn extract_option_of_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "f64");
}

#[test]
fn extract_vec_of_usize() {
    let ty: Type = parse_quote!(Vec<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "usize");
}

// ===========================================================================
// 19. wrap_leaf_type with multiple generic arguments
// ===========================================================================

#[test]
fn wrap_hashmap_both_args_wrapped_when_skipped() {
    let ty: Type = parse_quote!(HashMap<Key, Value>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ts(&wrapped),
        "HashMap < adze :: WithLeaf < Key > , adze :: WithLeaf < Value > >"
    );
}

#[test]
fn wrap_option_of_vec_three_skip_levels() {
    let ty: Type = parse_quote!(Option<Vec<Box<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(
        ts(&wrapped),
        "Option < Vec < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 20. FieldThenParams — field type inspection
// ===========================================================================

#[test]
fn ftp_field_type_is_accessible() {
    let ftp: FieldThenParams = parse_quote!(Vec<Node>);
    let field_ty = &ftp.field.ty;
    assert_eq!(ts(field_ty), "Vec < Node >");
}

#[test]
fn ftp_field_type_can_be_extracted() {
    let ftp: FieldThenParams = parse_quote!(Option<Expr>, name = "expr");
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Expr");
}

#[test]
fn ftp_param_values_are_expressions() {
    let ftp: FieldThenParams = parse_quote!(u32, min = 0, max = 255);
    assert!(matches!(ftp.params[0].expr, syn::Expr::Lit(_)));
    assert!(matches!(ftp.params[1].expr, syn::Expr::Lit(_)));
}

// ===========================================================================
// 21. NameValueExpr — expression variant coverage
// ===========================================================================

#[test]
fn nve_method_call_value() {
    let nve: NameValueExpr = parse_quote!(default = String::from("hello"));
    assert_eq!(nve.path.to_string(), "default");
}

#[test]
fn nve_array_value() {
    let nve: NameValueExpr = parse_quote!(items = [1, 2, 3]);
    assert_eq!(nve.path.to_string(), "items");
}

#[test]
fn nve_reference_value() {
    let nve: NameValueExpr = parse_quote!(target = &GLOBAL);
    assert_eq!(nve.path.to_string(), "target");
}

#[test]
fn nve_if_expr_value() {
    let nve: NameValueExpr = parse_quote!(val = if true { 1 } else { 0 });
    assert_eq!(nve.path.to_string(), "val");
}

// ===========================================================================
// 22. Cross-function consistency
// ===========================================================================

#[test]
fn filter_and_extract_agree_on_plain_type() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&filtered), ts(&extracted));
}

#[test]
fn filter_box_matches_extract_through_box() {
    let ty: Type = parse_quote!(Box<Leaf>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    // Extracting Leaf as target through empty skip (Box IS the target-like thing)
    // But that's different — just verify filter strips correctly
    assert_eq!(ts(&filtered), "Leaf");
}

#[test]
fn wrap_after_extract_and_filter() {
    let ty: Type = parse_quote!(Box<Option<Vec<Tok>>>);
    // Filter Box
    let no_box = filter_inner_type(&ty, &skip(&["Box"]));
    // Extract from Option
    let (vec_tok, ok) = try_extract_inner_type(&no_box, "Option", &skip(&[]));
    assert!(ok);
    // Wrap with Vec in skip set
    let wrapped = wrap_leaf_type(&vec_tok, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Tok > >");
}

// ===========================================================================
// 23. Large skip sets
// ===========================================================================

#[test]
fn large_skip_set_filters_deeply() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Mutex<Cell<Inner>>>>>);
    let s = skip(&["Box", "Arc", "Rc", "Mutex", "Cell"]);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ts(&filtered), "Inner");
}

#[test]
fn large_skip_set_extracts_through_all() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<Data>>>>);
    let s = skip(&["Box", "Arc", "Rc"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &s);
    assert!(ok);
    assert_eq!(ts(&inner), "Data");
}

// ===========================================================================
// 24. FieldThenParams — edge cases
// ===========================================================================

#[test]
fn ftp_option_field_type() {
    let ftp: FieldThenParams = parse_quote!(Option<String>);
    assert!(ftp.comma.is_none());
    assert_eq!(ts(&ftp.field.ty), "Option < String >");
}

#[test]
fn ftp_nested_generic_field_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<Box<Token>>>);
    assert_eq!(ts(&ftp.field.ty), "Vec < Option < Box < Token > > >");
}

#[test]
fn ftp_with_string_param_value() {
    let ftp: FieldThenParams = parse_quote!(String, regex = "[a-z]+");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "regex");
}

// ===========================================================================
// 25. Same type, different operations
// ===========================================================================

#[test]
fn same_type_extract_filter_wrap_differ() {
    let ty: Type = parse_quote!(Vec<Leaf>);
    let s = skip(&["Vec"]);

    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(ok);
    assert_eq!(ts(&extracted), "Leaf");

    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ts(&filtered), "Leaf");

    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Leaf > >");
}
