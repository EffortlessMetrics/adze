//! Comprehensive v4 tests for adze-common grammar expansion logic.
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr parsing, FieldThenParams parsing, multi-step expansion
//! pipelines, deep nesting, edge cases, and composition patterns.

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

/// Simulate a full grammar expansion pipeline for a single field type:
/// 1. Check optionality (extract Option)
/// 2. Check repetition (extract Vec)
/// 3. Strip wrappers (filter Box/Arc/Rc)
/// 4. Wrap leaf (adze::WithLeaf)
fn pipeline(ty: &Type) -> (bool, bool, String, String) {
    let wrappers = skip(&["Box", "Arc", "Rc"]);
    let containers = skip(&["Vec", "Option"]);

    let (after_opt, is_opt) = try_extract_inner_type(ty, "Option", &wrappers);
    let src = if is_opt { &after_opt } else { ty };

    let (after_vec, is_rep) = try_extract_inner_type(src, "Vec", &wrappers);
    let src2 = if is_rep { &after_vec } else { src };

    let leaf = filter_inner_type(src2, &wrappers);
    let wrapped = wrap_leaf_type(src2, &containers);

    (is_opt, is_rep, ty_str(&leaf), ty_str(&wrapped))
}

// ===========================================================================
// 1. try_extract_inner_type — basic extraction
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_mismatch_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_plain_type_returns_unchanged() {
    let ty: Type = parse_quote!(Identifier);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Identifier");
}

#[test]
fn extract_reference_type_not_path() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_tuple_type_not_path() {
    let ty: Type = parse_quote!((i32, u64));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u64)");
}

// ===========================================================================
// 2. try_extract_inner_type — with skip_over
// ===========================================================================

#[test]
fn extract_through_box() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_through_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<Vec<f64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_skip_present_but_target_absent() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_through_rc() {
    let ty: Type = parse_quote!(Rc<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// 3. filter_inner_type — basic filtering
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_u32() {
    let ty: Type = parse_quote!(Arc<u32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "u32");
}

#[test]
fn filter_not_in_skip_returns_original() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_plain_type_returns_unchanged() {
    let ty: Type = parse_quote!(MyStruct);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "MyStruct");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "i32"
    );
}

#[test]
fn filter_empty_skip_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < String >"
    );
}

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&mut u8);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& mut u8");
}

#[test]
fn filter_triple_nested_wrappers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Token>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "Token"
    );
}

// ===========================================================================
// 4. wrap_leaf_type — basic wrapping
// ===========================================================================

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(Expr);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < Expr >");
}

#[test]
fn wrap_vec_skipped_inner_wrapped() {
    let ty: Type = parse_quote!(Vec<Stmt>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&w), "Vec < adze :: WithLeaf < Stmt > >");
}

#[test]
fn wrap_option_skipped_inner_wrapped() {
    let ty: Type = parse_quote!(Option<Token>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&w), "Option < adze :: WithLeaf < Token > >");
}

#[test]
fn wrap_nested_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<Item>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(ty_str(&w), "Option < Vec < adze :: WithLeaf < Item > > >");
}

#[test]
fn wrap_vec_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Vec<i32>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 16]);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < [u8 ; 16] >");
}

#[test]
fn wrap_result_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<Foo, Bar>);
    let w = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&w),
        "Result < adze :: WithLeaf < Foo > , adze :: WithLeaf < Bar > >"
    );
}

#[test]
fn wrap_deeply_nested_skip_chain() {
    let ty: Type = parse_quote!(Option<Vec<Option<Leaf>>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&w),
        "Option < Vec < Option < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 5. NameValueExpr parsing
// ===========================================================================

#[test]
fn nve_string_literal() {
    let nv: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nv.path.to_string(), "name");
}

#[test]
fn nve_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    assert!(matches!(nv.expr, syn::Expr::Lit(_)));
}

#[test]
fn nve_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
}

#[test]
fn nve_path_expr() {
    let nv: NameValueExpr = parse_quote!(kind = SomeEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
    assert!(matches!(nv.expr, syn::Expr::Path(_)));
}

#[test]
fn nve_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
    assert!(matches!(nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn nve_tuple_expr() {
    let nv: NameValueExpr = parse_quote!(range = (0, 100));
    assert_eq!(nv.path.to_string(), "range");
    assert!(matches!(nv.expr, syn::Expr::Tuple(_)));
}

#[test]
fn nve_array_expr() {
    let nv: NameValueExpr = parse_quote!(items = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "items");
    assert!(matches!(nv.expr, syn::Expr::Array(_)));
}

// ===========================================================================
// 6. FieldThenParams parsing
// ===========================================================================

#[test]
fn ftp_bare_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(Expr, precedence = 5);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
}

#[test]
fn ftp_type_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(Token, name = "plus", assoc = "left");
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "assoc");
}

#[test]
fn ftp_generic_type_with_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<Stmt>, separator = ",");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "separator");
}

#[test]
fn ftp_option_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(Option<Ident>);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_nested_generic_with_param() {
    let ftp: FieldThenParams = parse_quote!(Option<Vec<Expr>>, min = 0);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "min");
}

#[test]
fn ftp_three_params() {
    let ftp: FieldThenParams = parse_quote!(Rule, name = "expr", prec = 1, assoc = "right");
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "assoc");
}

// ===========================================================================
// 7. Pipeline — full grammar field expansion
// ===========================================================================

#[test]
fn pipeline_plain_identifier() {
    let ty: Type = parse_quote!(Identifier);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(!opt);
    assert!(!rep);
    assert_eq!(leaf, "Identifier");
}

#[test]
fn pipeline_option_return_type() {
    let ty: Type = parse_quote!(Option<ReturnType>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(opt);
    assert!(!rep);
    assert_eq!(leaf, "ReturnType");
}

#[test]
fn pipeline_vec_statement() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(!opt);
    assert!(rep);
    assert_eq!(leaf, "Statement");
}

#[test]
fn pipeline_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Arg>>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(opt);
    assert!(rep);
    assert_eq!(leaf, "Arg");
}

#[test]
fn pipeline_box_wrapped_option() {
    let ty: Type = parse_quote!(Box<Option<Tail>>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(opt);
    assert!(!rep);
    assert_eq!(leaf, "Tail");
}

#[test]
fn pipeline_arc_vec() {
    let ty: Type = parse_quote!(Arc<Vec<Item>>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(!opt);
    assert!(rep);
    assert_eq!(leaf, "Item");
}

#[test]
fn pipeline_box_arc_option_vec() {
    let ty: Type = parse_quote!(Box<Arc<Option<Vec<Token>>>>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(opt);
    assert!(rep);
    assert_eq!(leaf, "Token");
}

#[test]
fn pipeline_rc_plain() {
    let ty: Type = parse_quote!(Rc<Literal>);
    let (opt, rep, leaf, _) = pipeline(&ty);
    assert!(!opt);
    assert!(!rep);
    assert_eq!(leaf, "Literal");
}

// ===========================================================================
// 8. wrap_leaf_type — composition with filter_inner_type
// ===========================================================================

#[test]
fn filter_then_wrap_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_arc_vec() {
    let ty: Type = parse_quote!(Arc<Vec<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u8 > >");
}

#[test]
fn extract_then_wrap_option() {
    let ty: Type = parse_quote!(Option<Ident>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Ident >");
}

#[test]
fn extract_then_wrap_vec() {
    let ty: Type = parse_quote!(Vec<Tok>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Tok >");
}

// ===========================================================================
// 9. Edge cases — qualified paths, generics with lifetimes, primitives
// ===========================================================================

#[test]
fn extract_qualified_path_not_matched() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    // Last segment is still "Vec"
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_qualified_box() {
    let ty: Type = parse_quote!(std::boxed::Box<Foo>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Foo");
}

#[test]
fn wrap_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < bool >");
}

#[test]
fn extract_empty_skip_set_no_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    // Box is NOT in the skip set, so it won't be skipped through
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Vec < String > >");
}

#[test]
fn filter_slice_type_unchanged() {
    let ty: Type = parse_quote!([u8]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "[u8]");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((A, B));
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < (A , B) >");
}

// ===========================================================================
// 10. NameValueExpr — identity and expression variety
// ===========================================================================

#[test]
fn nve_closure_expr() {
    let nv: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

#[test]
fn nve_method_call() {
    let nv: NameValueExpr = parse_quote!(value = foo.bar());
    assert_eq!(nv.path.to_string(), "value");
    assert!(matches!(nv.expr, syn::Expr::MethodCall(_)));
}

#[test]
fn nve_block_expr() {
    let nv: NameValueExpr = parse_quote!(
        init = {
            let x = 1;
            x
        }
    );
    assert_eq!(nv.path.to_string(), "init");
    assert!(matches!(nv.expr, syn::Expr::Block(_)));
}

// ===========================================================================
// 11. Idempotency and symmetry
// ===========================================================================

#[test]
fn filter_idempotent() {
    let ty: Type = parse_quote!(Box<String>);
    let s = skip(&["Box"]);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn wrap_not_in_skip_is_idempotent_structurally() {
    // Wrapping a non-skip type always adds WithLeaf, so wrapping again adds another layer.
    // This test verifies the first wrap is consistent.
    let ty: Type = parse_quote!(Foo);
    let s = skip(&[]);
    let w1 = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&w1), "adze :: WithLeaf < Foo >");
}

#[test]
fn extract_same_target_twice_is_idempotent() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
    // Extracting again from the inner type (i32) should find nothing.
    let (inner2, ok2) = try_extract_inner_type(&inner, "Option", &skip(&[]));
    assert!(!ok2);
    assert_eq!(ty_str(&inner2), "i32");
}

// ===========================================================================
// 12. Multiple grammar fields — simulating a struct definition
// ===========================================================================

#[test]
fn multi_field_struct_expansion() {
    let fields: Vec<Type> = vec![
        parse_quote!(Identifier),
        parse_quote!(Vec<Param>),
        parse_quote!(Option<ReturnType>),
        parse_quote!(Vec<Statement>),
    ];

    let results: Vec<_> = fields.iter().map(pipeline).collect();

    // Identifier: not optional, not repeated
    assert!(!results[0].0 && !results[0].1);
    assert_eq!(results[0].2, "Identifier");

    // Vec<Param>: repeated
    assert!(!results[1].0 && results[1].1);
    assert_eq!(results[1].2, "Param");

    // Option<ReturnType>: optional
    assert!(results[2].0 && !results[2].1);
    assert_eq!(results[2].2, "ReturnType");

    // Vec<Statement>: repeated
    assert!(!results[3].0 && results[3].1);
    assert_eq!(results[3].2, "Statement");
}

// ===========================================================================
// 13. FieldThenParams — field type extraction
// ===========================================================================

#[test]
fn ftp_field_type_is_preserved() {
    let ftp: FieldThenParams = parse_quote!(Vec<Expr>, sep = ",");
    let field_ty = &ftp.field.ty;
    assert_eq!(ty_str(field_ty), "Vec < Expr >");
}

#[test]
fn ftp_params_iterator() {
    let ftp: FieldThenParams = parse_quote!(Node, a = 1, b = 2, c = 3);
    let names: Vec<String> = ftp.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn ftp_bare_type_field_accessible() {
    let ftp: FieldThenParams = parse_quote!(bool);
    assert_eq!(ty_str(&ftp.field.ty), "bool");
}

// ===========================================================================
// 14. Complex nested generics
// ===========================================================================

#[test]
fn extract_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Leaf >");
    // Extract again from inner
    let (inner2, ok2) = try_extract_inner_type(&inner, "Option", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner2), "Leaf");
}

#[test]
fn extract_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<Item>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Item >");
}

#[test]
fn wrap_hashmap_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<K, V>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < HashMap < K , V > >");
}

#[test]
fn wrap_hashmap_in_skip_wraps_both_params() {
    let ty: Type = parse_quote!(HashMap<Key, Val>);
    let w = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ty_str(&w),
        "HashMap < adze :: WithLeaf < Key > , adze :: WithLeaf < Val > >"
    );
}

#[test]
fn filter_box_of_box() {
    let ty: Type = parse_quote!(Box<Box<Inner>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

// ===========================================================================
// 15. Wrap with multi-level skip
// ===========================================================================

#[test]
fn wrap_option_vec_result_all_skipped() {
    let ty: Type = parse_quote!(Option<Vec<Result<A, B>>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Result"]));
    assert_eq!(
        ty_str(&w),
        "Option < Vec < Result < adze :: WithLeaf < A > , adze :: WithLeaf < B > > > >"
    );
}

#[test]
fn wrap_vec_option_partial_skip() {
    // Only Vec is in skip — Option is NOT, so Option<T> gets wrapped entirely
    let ty: Type = parse_quote!(Vec<Option<X>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&w), "Vec < adze :: WithLeaf < Option < X > > >");
}

// ===========================================================================
// 16. NameValueExpr and FieldThenParams — roundtrip patterns
// ===========================================================================

#[test]
fn nve_float_literal() {
    let nv: NameValueExpr = parse_quote!(weight = 0.5);
    assert_eq!(nv.path.to_string(), "weight");
}

#[test]
fn nve_char_literal() {
    let nv: NameValueExpr = parse_quote!(delim = ',');
    assert_eq!(nv.path.to_string(), "delim");
}

#[test]
fn ftp_with_bool_param() {
    let ftp: FieldThenParams = parse_quote!(Token, inline = true);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "inline");
}

// ===========================================================================
// 17. Pipeline wrapped output verification
// ===========================================================================

#[test]
fn pipeline_wrapped_plain_type() {
    let ty: Type = parse_quote!(Lit);
    let (_, _, _, wrapped) = pipeline(&ty);
    // Lit is not Vec/Option, so it should be wrapped in WithLeaf via container skip set
    assert_eq!(wrapped, "adze :: WithLeaf < Lit >");
}

#[test]
fn pipeline_wrapped_vec() {
    let ty: Type = parse_quote!(Vec<Tok>);
    let (_, _, _, wrapped) = pipeline(&ty);
    // After extracting Vec, inner is Tok which is not in container skip, so wrapped
    assert_eq!(wrapped, "adze :: WithLeaf < Tok >");
}

#[test]
fn pipeline_wrapped_option() {
    let ty: Type = parse_quote!(Option<Ret>);
    let (_, _, _, wrapped) = pipeline(&ty);
    assert_eq!(wrapped, "adze :: WithLeaf < Ret >");
}

#[test]
fn pipeline_wrapped_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Arg>>);
    let (_, _, _, wrapped) = pipeline(&ty);
    // After extracting Option then Vec, we wrap Arg
    assert_eq!(wrapped, "adze :: WithLeaf < Arg >");
}
