//! Comprehensive tests for syntax manipulation and parsing utilities
//! in adze-common (re-exported from adze-common-syntax-core).
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr parsing, FieldThenParams parsing, and their composition.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ===========================================================================
// try_extract_inner_type — basic extraction
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn extract_no_match_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
    assert_eq!(
        inner.to_token_stream().to_string(),
        "HashMap < String , i32 >"
    );
}

#[test]
fn extract_bare_type_no_match() {
    let ty: Type = parse_quote!(u64);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "u64");
}

// ===========================================================================
// try_extract_inner_type — skip_over behaviour
// ===========================================================================

#[test]
fn extract_skips_box_to_find_option() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "bool");
}

#[test]
fn extract_skips_arc_and_box_to_find_vec() {
    let skip: HashSet<&str> = ["Arc", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Box<Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "u8");
}

#[test]
fn extract_skip_chain_target_not_found() {
    let skip: HashSet<&str> = ["Box", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Rc<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "Box < Rc < String > >");
}

#[test]
fn extract_skip_stops_at_non_skip_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Result<Vec<u8>>>);
    // Result is not in skip set, so we cannot reach Vec
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(
        inner.to_token_stream().to_string(),
        "Box < Result < Vec < u8 > > >"
    );
}

// ===========================================================================
// try_extract_inner_type — non-path types
// ===========================================================================

#[test]
fn extract_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

#[test]
fn extract_tuple_type_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "(i32 , u32)");
}

#[test]
fn extract_slice_type_unchanged() {
    let ty: Type = parse_quote!([u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "[u8]");
}

// ===========================================================================
// filter_inner_type — basic filtering
// ===========================================================================

#[test]
fn filter_box_strips_to_inner() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Token>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Token");
}

#[test]
fn filter_arc_strips_to_inner() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Mutex>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Mutex");
}

#[test]
fn filter_not_in_skip_set_returns_original() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < String >");
}

#[test]
fn filter_empty_skip_set_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Box < i32 >");
}

#[test]
fn filter_nested_skip_types_strips_all() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn filter_stops_at_non_skip_boundary() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Box<Inner>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        "Vec < Box < Inner > >"
    );
}

// ===========================================================================
// filter_inner_type — non-path types
// ===========================================================================

#[test]
fn filter_tuple_type_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!((u8, u16));
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(u8 , u16)");
}

#[test]
fn filter_reference_type_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(&mut String);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "& mut String");
}

// ===========================================================================
// wrap_leaf_type — basic wrapping
// ===========================================================================

#[test]
fn wrap_simple_type_wraps_in_with_leaf() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Identifier);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Identifier >"
    );
}

#[test]
fn wrap_preserves_vec_container() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Statement>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < Statement > >"
    );
}

#[test]
fn wrap_preserves_option_container() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Expr>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < Expr > >"
    );
}

#[test]
fn wrap_nested_skip_containers() {
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<Tok>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < Tok > > >"
    );
}

#[test]
fn wrap_non_skip_generic_wraps_entire_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Vec < u8 > >"
    );
}

// ===========================================================================
// wrap_leaf_type — non-path types
// ===========================================================================

#[test]
fn wrap_array_type_wraps_entirely() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_unit_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn wrap_reference_type_wraps_entirely() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

// ===========================================================================
// wrap_leaf_type — multiple generic args
// ===========================================================================

#[test]
fn wrap_result_type_in_skip_set_wraps_both_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<Token, Error>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Result < adze :: WithLeaf < Token > , adze :: WithLeaf < Error > >"
    );
}

// ===========================================================================
// NameValueExpr parsing
// ===========================================================================

#[test]
fn name_value_string_literal() {
    let nv: NameValueExpr = parse_quote!(pattern = "hello");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), "hello");
        } else {
            panic!("expected string literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn name_value_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
        } else {
            panic!("expected int literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Bool(b) = &lit.lit {
            assert!(b.value);
        } else {
            panic!("expected bool literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn name_value_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
    // Negative integer parses as Expr::Unary(Neg, Lit)
    assert!(matches!(&nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn name_value_path_expression() {
    let nv: NameValueExpr = parse_quote!(kind = MyEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
    assert!(matches!(&nv.expr, syn::Expr::Path(_)));
}

// ===========================================================================
// FieldThenParams parsing
// ===========================================================================

#[test]
fn field_then_params_bare_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(MyNode);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "MyNode");
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_params_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(String, pattern = "[a-z]+");
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "String");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(u32, min = 0, max = 100, label = "count");
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "u32");
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
    assert_eq!(ftp.params[2].path.to_string(), "label");
}

#[test]
fn field_then_params_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Token>);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Vec < Token >");
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_params_option_type_with_param() {
    let ftp: FieldThenParams = parse_quote!(Option<String>, default = "none");
    assert_eq!(
        ftp.field.ty.to_token_stream().to_string(),
        "Option < String >"
    );
    assert_eq!(ftp.params.len(), 1);
}

// ===========================================================================
// Composition — extract then filter
// ===========================================================================

#[test]
fn compose_extract_then_filter() {
    let skip_extract: HashSet<&str> = ["Box"].into_iter().collect();
    let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Box<Leaf>>);

    let (after_extract, ok) = try_extract_inner_type(&ty, "Option", &skip_extract);
    assert!(ok);
    assert_eq!(after_extract.to_token_stream().to_string(), "Box < Leaf >");

    let filtered = filter_inner_type(&after_extract, &skip_filter);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn compose_filter_then_wrap() {
    let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
    let skip_wrap: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Item>>);

    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < Item >");

    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < Item > >"
    );
}

#[test]
fn compose_extract_filter_wrap_pipeline() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Box<Node>>);

    let (after_vec, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);

    let filtered = filter_inner_type(&after_vec, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Node");

    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Node >"
    );
}

// ===========================================================================
// Idempotence and stability
// ===========================================================================

#[test]
fn filter_idempotent_on_non_skip_type() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    let first = filter_inner_type(&ty, &skip);
    let second = filter_inner_type(&first, &skip);
    assert_eq!(
        first.to_token_stream().to_string(),
        second.to_token_stream().to_string()
    );
}

#[test]
fn extract_idempotent_when_no_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (first, ok1) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok1);
    let (second, ok2) = try_extract_inner_type(&first, "Option", &skip);
    assert!(!ok2);
    assert_eq!(
        first.to_token_stream().to_string(),
        second.to_token_stream().to_string()
    );
}

// ===========================================================================
// NameValueExpr — equality / clone
// ===========================================================================

#[test]
fn name_value_expr_clone_eq() {
    let nv: NameValueExpr = parse_quote!(key = "val");
    let cloned = nv.clone();
    assert_eq!(nv, cloned);
}

#[test]
fn name_value_expr_debug_impl() {
    let nv: NameValueExpr = parse_quote!(key = 1);
    let debug_str = format!("{:?}", nv);
    assert!(debug_str.contains("NameValueExpr"));
}

// ===========================================================================
// FieldThenParams — equality / clone
// ===========================================================================

#[test]
fn field_then_params_clone_eq() {
    let ftp: FieldThenParams = parse_quote!(Ty, a = 1);
    let cloned = ftp.clone();
    assert_eq!(ftp, cloned);
}

#[test]
fn field_then_params_debug_impl() {
    let ftp: FieldThenParams = parse_quote!(Ty);
    let debug_str = format!("{:?}", ftp);
    assert!(debug_str.contains("FieldThenParams"));
}

// ===========================================================================
// Qualified / complex path types
// ===========================================================================

#[test]
fn extract_from_qualified_path() {
    let ty: Type = parse_quote!(std::option::Option<u32>);
    // The last segment is "Option", so extraction should work
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "u32");
}

#[test]
fn filter_qualified_box_path() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(std::boxed::Box<Leaf>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn wrap_qualified_vec_preserved() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(std::vec::Vec<Item>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "std :: vec :: Vec < adze :: WithLeaf < Item > >"
    );
}

// ===========================================================================
// Batch processing of FieldThenParams
// ===========================================================================

#[test]
fn batch_field_types_with_mixed_containers() {
    let fields: Vec<FieldThenParams> = vec![
        parse_quote!(String),
        parse_quote!(Option<u32>),
        parse_quote!(Vec<Token>),
        parse_quote!(Box<Expr>),
    ];
    let type_strs: Vec<String> = fields
        .iter()
        .map(|f| f.field.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(type_strs.len(), 4);
    assert_eq!(type_strs[0], "String");
    assert_eq!(type_strs[1], "Option < u32 >");
    assert_eq!(type_strs[2], "Vec < Token >");
    assert_eq!(type_strs[3], "Box < Expr >");
}

#[test]
fn batch_extract_option_fields() {
    let types: Vec<Type> = vec![
        parse_quote!(Option<A>),
        parse_quote!(String),
        parse_quote!(Option<B>),
        parse_quote!(Vec<C>),
    ];
    let extracted: Vec<(String, bool)> = types
        .iter()
        .map(|ty| {
            let (inner, ok) = try_extract_inner_type(ty, "Option", &HashSet::new());
            (inner.to_token_stream().to_string(), ok)
        })
        .collect();
    assert_eq!(extracted[0], ("A".to_string(), true));
    assert_eq!(extracted[1], ("String".to_string(), false));
    assert_eq!(extracted[2], ("B".to_string(), true));
    assert_eq!(extracted[3], ("Vec < C >".to_string(), false));
}

// ===========================================================================
// NameValueExpr with complex expressions
// ===========================================================================

#[test]
fn name_value_closure_expression() {
    let nv: NameValueExpr = parse_quote!(transform = |x: i32| x + 1);
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(&nv.expr, syn::Expr::Closure(_)));
}

#[test]
fn name_value_method_call_expression() {
    let nv: NameValueExpr = parse_quote!(default = String::new());
    assert_eq!(nv.path.to_string(), "default");
    assert!(matches!(&nv.expr, syn::Expr::Call(_)));
}
