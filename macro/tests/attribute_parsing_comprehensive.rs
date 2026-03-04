//! Comprehensive attribute parsing tests for the adze-macro crate.
//!
//! Tests cover:
//! 1. Token stream parsing helpers (syn roundtrips, attribute detection)
//! 2. Type extraction utilities (try_extract_inner_type, filter_inner_type, wrap_leaf_type)
//! 3. Attribute value parsing (NameValueExpr, FieldThenParams)
//! 4. Edge cases in attribute processing (closures, raw strings, nested generics)

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, ItemEnum, ItemStruct, Type, parse_quote, parse2};

// ── Helper Functions ─────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    segments.len() == 2 && segments[0].ident == "adze" && segments[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            let segs: Vec<_> = attr.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ============================================================================
// 1. NameValueExpr PARSING
// ============================================================================

/// NameValueExpr: parse simple string value
#[test]
fn name_value_expr_string_literal() {
    let nv: NameValueExpr = parse_quote!(pattern = "hello");
    assert_eq!(nv.path, "pattern");
}

/// NameValueExpr: parse integer value
#[test]
fn name_value_expr_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path, "precedence");
}

/// NameValueExpr: parse closure expression as value
#[test]
fn name_value_expr_closure_value() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nv.path, "transform");
    let expr_str = nv.expr.to_token_stream().to_string();
    assert!(expr_str.contains("parse"));
}

/// NameValueExpr: parse boolean-like identifier value
#[test]
fn name_value_expr_bool_value() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path, "enabled");
}

/// NameValueExpr: parse raw string literal value
#[test]
fn name_value_expr_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"[a-z]+");
    assert_eq!(nv.path, "pattern");
}

// ============================================================================
// 2. FieldThenParams PARSING
// ============================================================================

/// FieldThenParams: bare type with no params
#[test]
fn field_then_params_bare_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "String");
}

/// FieldThenParams: type followed by single key-value param
#[test]
fn field_then_params_single_param() {
    let ftp: FieldThenParams = parse_quote!(i32, transform = |v| v.parse().unwrap());
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "transform");
}

/// FieldThenParams: type with multiple params
#[test]
fn field_then_params_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(u64, name = "count", priority = 5);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path, "name");
    assert_eq!(ftp.params[1].path, "priority");
}

/// FieldThenParams: generic type with params
#[test]
fn field_then_params_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, name = "items");
    assert_eq!(ts(&ftp.field.ty), "Vec < String >");
    assert_eq!(ftp.params.len(), 1);
}

// ============================================================================
// 3. try_extract_inner_type
// ============================================================================

/// Extract inner type from Vec<T>
#[test]
fn extract_inner_vec_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

/// Extraction fails when target doesn't match
#[test]
fn extract_inner_mismatch_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ts(&inner), "Option < String >");
}

/// Skip through Box to find Vec inside
#[test]
fn extract_inner_skip_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

/// Skip through multiple layers (Box<Arc<Option<T>>>)
#[test]
fn extract_inner_skip_through_two_layers() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Option<i64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "i64");
}

/// Non-path type (reference) returns unchanged
#[test]
fn extract_inner_non_path_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

/// Skip type present but target not found inside returns original
#[test]
fn extract_inner_skip_present_target_absent() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < String >");
}

// ============================================================================
// 4. filter_inner_type
// ============================================================================

/// Unwrap single Box layer
#[test]
fn filter_inner_single_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "String");
}

/// Unwrap nested Box<Arc<T>>
#[test]
fn filter_inner_nested_box_arc() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<u8>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "u8");
}

/// Non-skip type left unchanged
#[test]
fn filter_inner_no_match() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "Vec < String >");
}

/// Empty skip set returns original
#[test]
fn filter_inner_empty_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<i32>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "Box < i32 >");
}

/// Tuple type (non-Path) passes through
#[test]
fn filter_inner_tuple_type() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "(i32 , u32)");
}

// ============================================================================
// 5. wrap_leaf_type
// ============================================================================

/// Plain type gets wrapped in adze::WithLeaf
#[test]
fn wrap_leaf_plain_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < String >"
    );
}

/// Skip-set container wraps only inner args
#[test]
fn wrap_leaf_vec_wraps_inner() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Vec < adze :: WithLeaf < String > >"
    );
}

/// Nested skip containers recursively wrap leaf
#[test]
fn wrap_leaf_option_vec_recursive() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Option < Vec < adze :: WithLeaf < u32 > > >"
    );
}

/// Multi-arg generic in skip set wraps each type arg
#[test]
fn wrap_leaf_result_wraps_both_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

/// Array type (non-Path) gets wrapped entirely
#[test]
fn wrap_leaf_array_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

// ============================================================================
// 6. ATTRIBUTE DETECTION ON STRUCTS / ENUMS
// ============================================================================

/// Detect adze::grammar on struct
#[test]
fn detect_grammar_attr_on_struct() {
    let s = parse_struct(quote! {
        #[adze::grammar]
        struct G { field: String }
    });
    assert!(is_adze_attr(&s.attrs[0], "grammar"));
}

/// Mixed adze and derive attributes; only adze names extracted
#[test]
fn mixed_adze_and_derive_attrs() {
    let s = parse_struct(quote! {
        #[derive(Debug)]
        #[adze::grammar]
        #[adze::language]
        struct S { f: u8 }
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["grammar", "language"]);
}

/// Attribute on enum variant field
#[test]
fn attr_on_enum_variant_field() {
    let e = parse_enum(quote! {
        enum E {
            V(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32
            ),
        }
    });
    let field = &e.variants[0].fields.iter().next().unwrap();
    assert!(is_adze_attr(&field.attrs[0], "leaf"));
}

/// Attribute order preserved across multiple items
#[test]
fn preserve_attr_order_on_struct() {
    let s = parse_struct(quote! {
        #[adze::grammar]
        #[adze::language]
        #[adze::word]
        struct O { f: u8 }
    });
    assert_eq!(
        adze_attr_names(&s.attrs),
        vec!["grammar", "language", "word"]
    );
}

// ============================================================================
// 7. EDGE CASES & ROUNDTRIPS
// ============================================================================

/// Doc comments are separate from adze attrs
#[test]
fn doc_comment_not_counted_as_adze() {
    let s = parse_struct(quote! {
        /// doc
        #[adze::grammar]
        struct D { f: u8 }
    });
    // doc comment becomes an attribute in syn; only 1 should be adze
    assert_eq!(adze_attr_names(&s.attrs).len(), 1);
}

/// Quote-reparse roundtrip preserves struct identity and attrs
#[test]
fn roundtrip_struct_preserves_attrs() {
    let s = parse_struct(quote! {
        #[adze::grammar]
        pub struct RT { x: String }
    });
    let reparsed = parse_struct(quote! { #s });
    assert_eq!(reparsed.ident, "RT");
    assert!(is_adze_attr(&reparsed.attrs[0], "grammar"));
}

/// Quote-reparse roundtrip for enum with variant attrs
#[test]
fn roundtrip_enum_preserves_variant_attrs() {
    let e = parse_enum(quote! {
        enum E {
            #[adze::word]
            A,
            B,
        }
    });
    let reparsed = parse_enum(quote! { #e });
    assert!(is_adze_attr(&reparsed.variants[0].attrs[0], "word"));
    assert!(reparsed.variants[1].attrs.is_empty());
}

/// Leaf attribute with text= argument parses correctly
#[test]
fn leaf_attr_text_argument() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(text = "+")]
            op: (),
        }
    });
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("text"));
    assert!(attr_str.contains("+"));
}

/// prec_left with numeric argument
#[test]
fn prec_left_numeric_arg() {
    let e = parse_enum(quote! {
        enum E {
            #[adze::prec_left(1)]
            Add(Box<E>, (), Box<E>),
        }
    });
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_left"));
}

/// Multiple fields with heterogeneous adze attrs
#[test]
fn heterogeneous_field_attrs() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(pattern = "id")]
            name: String,
            #[adze::skip(Default::default())]
            ignored: u32,
        }
    });
    if let Fields::Named(ref fields) = s.fields {
        assert!(is_adze_attr(&fields.named[0].attrs[0], "leaf"));
        assert!(is_adze_attr(&fields.named[1].attrs[0], "skip"));
    } else {
        panic!("expected named fields");
    }
}

/// Non-adze two-segment path is not misidentified
#[test]
fn non_adze_two_segment_path_ignored() {
    let s = parse_struct(quote! {
        #[serde::rename]
        #[adze::grammar]
        struct S { f: u8 }
    });
    assert_eq!(adze_attr_names(&s.attrs), vec!["grammar"]);
}

/// Struct with where clause still has attrs
#[test]
fn where_clause_does_not_affect_attrs() {
    let s = parse_struct(quote! {
        #[adze::grammar]
        struct W<T> where T: Clone { f: T }
    });
    assert!(is_adze_attr(&s.attrs[0], "grammar"));
    assert!(s.generics.where_clause.is_some());
}
