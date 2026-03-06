//! Attribute parsing v6 tests for the adze-macro crate.
//!
//! 64 tests across 8 categories:
//! 1. attr_parse_*    — basic attribute parsing
//! 2. attr_struct_*   — struct annotation parsing
//! 3. attr_enum_*     — enum annotation parsing
//! 4. attr_field_*    — field attribute parsing
//! 5. attr_generic_*  — generic type handling
//! 6. attr_validate_* — attribute validation
//! 7. attr_error_*    — error handling for invalid attrs
//! 8. attr_complex_*  — complex attribute combinations

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, DeriveInput, Fields, ItemEnum, ItemStruct, Meta, Type, parse_quote, parse2};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_struct_str(code: &str) -> ItemStruct {
    syn::parse_str::<ItemStruct>(code).expect("failed to parse ItemStruct")
}

fn parse_enum_str(code: &str) -> ItemEnum {
    syn::parse_str::<ItemEnum>(code).expect("failed to parse ItemEnum")
}

fn parse_derive_str(code: &str) -> DeriveInput {
    syn::parse_str::<DeriveInput>(code).expect("failed to parse DeriveInput")
}

fn parse_type_str(code: &str) -> Type {
    syn::parse_str::<Type>(code).expect("failed to parse Type")
}

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
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

fn named_fields(s: &ItemStruct) -> Vec<&syn::Field> {
    match &s.fields {
        Fields::Named(f) => f.named.iter().collect(),
        _ => panic!("expected named fields"),
    }
}

fn unnamed_fields(s: &ItemStruct) -> Vec<&syn::Field> {
    match &s.fields {
        Fields::Unnamed(f) => f.unnamed.iter().collect(),
        _ => panic!("expected unnamed fields"),
    }
}

fn empty_skip_set() -> HashSet<&'static str> {
    HashSet::new()
}

fn box_skip_set() -> HashSet<&'static str> {
    HashSet::from(["Box", "Arc"])
}

fn container_skip_set() -> HashSet<&'static str> {
    HashSet::from(["Vec", "Option"])
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn meta_to_string(attr: &Attribute) -> String {
    attr.meta.to_token_stream().to_string()
}

// ============================================================================
// 1. attr_parse_* — basic attribute parsing (8 tests)
// ============================================================================

#[test]
fn attr_parse_single_path_attribute() {
    let s = parse_struct_str("#[inline] struct S { x: i32 }");
    assert_eq!(s.attrs.len(), 1);
    assert!(matches!(s.attrs[0].meta, Meta::Path(_)));
}

#[test]
fn attr_parse_name_value_attribute() {
    let s = parse_struct_str(r#"#[doc = "hello"] struct S { x: i32 }"#);
    assert_eq!(s.attrs.len(), 1);
    assert!(matches!(s.attrs[0].meta, Meta::NameValue(_)));
}

#[test]
fn attr_parse_list_attribute() {
    let s = parse_struct_str("#[derive(Debug)] struct S { x: i32 }");
    assert_eq!(s.attrs.len(), 1);
    assert!(matches!(s.attrs[0].meta, Meta::List(_)));
}

#[test]
fn attr_parse_meta_path_ident() {
    let attr: Attribute = parse_quote!(#[inline]);
    let path_str = attr.path().to_token_stream().to_string();
    assert_eq!(path_str, "inline");
}

#[test]
fn attr_parse_two_segment_path() {
    let attr: Attribute = parse_quote!(#[adze::language]);
    let segments: Vec<_> = attr
        .path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    assert_eq!(segments, ["adze", "language"]);
}

#[test]
fn attr_parse_name_value_expr_string() {
    let nve: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nve.path.to_string(), "text");
    let expr_str = nve.expr.to_token_stream().to_string();
    assert!(expr_str.contains("hello"));
}

#[test]
fn attr_parse_name_value_expr_integer() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
    let expr_str = nve.expr.to_token_stream().to_string();
    assert_eq!(expr_str, "42");
}

#[test]
fn attr_parse_field_then_params_no_params() {
    let ftp: FieldThenParams = parse_quote!(u32);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ============================================================================
// 2. attr_struct_* — struct annotation parsing (8 tests)
// ============================================================================

#[test]
fn attr_struct_derive_debug_clone() {
    let s = parse_struct_str("#[derive(Debug, Clone)] struct Node { id: u32 }");
    assert_eq!(s.attrs.len(), 1);
    let meta_str = meta_to_string(&s.attrs[0]);
    assert!(meta_str.contains("Debug"));
    assert!(meta_str.contains("Clone"));
}

#[test]
fn attr_struct_repr_c() {
    let s = parse_struct_str("#[repr(C)] struct Layout { a: u32, b: u64 }");
    let meta_str = meta_to_string(&s.attrs[0]);
    assert!(meta_str.contains("repr"));
    assert!(meta_str.contains("C"));
}

#[test]
fn attr_struct_adze_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct Root { value: i32 }
    };
    assert!(is_adze_attr(&s.attrs[0], "language"));
}

#[test]
fn attr_struct_adze_extra() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace { _ws: () }
    };
    assert!(is_adze_attr(&s.attrs[0], "extra"));
}

#[test]
fn attr_struct_adze_word() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        struct Identifier { name: String }
    };
    assert!(is_adze_attr(&s.attrs[0], "word"));
}

#[test]
fn attr_struct_adze_external() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(is_adze_attr(&s.attrs[0], "external"));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn attr_struct_multiple_attrs_preserved_order() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        #[allow(dead_code)]
        struct Multi { x: i32 }
    };
    assert_eq!(s.attrs.len(), 3);
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, ["language"]);
}

#[test]
fn attr_struct_tuple_struct_with_attr() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "fn")]
        struct Keyword(());
    };
    assert!(is_adze_attr(&s.attrs[0], "leaf"));
    let fields = unnamed_fields(&s);
    assert_eq!(fields.len(), 1);
}

// ============================================================================
// 3. attr_enum_* — enum annotation parsing (8 tests)
// ============================================================================

#[test]
fn attr_enum_variant_prec_left() {
    let e = parse_enum_str("enum Expr { #[adze::prec_left(1)] Add(Box<Expr>, Box<Expr>) }");
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_left"));
}

#[test]
fn attr_enum_variant_prec_right() {
    let e = parse_enum_str("enum Expr { #[adze::prec_right(2)] Pow(Box<Expr>, Box<Expr>) }");
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_right"));
}

#[test]
fn attr_enum_variant_prec_no_assoc() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(3)]
            Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
        }
    };
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec"));
}

#[test]
fn attr_enum_variant_leaf_text_unit() {
    let e: ItemEnum = parse_quote! {
        enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    };
    assert!(is_adze_attr(&e.variants[0].attrs[0], "leaf"));
    assert!(is_adze_attr(&e.variants[1].attrs[0], "leaf"));
    assert!(matches!(e.variants[0].fields, Fields::Unit));
}

#[test]
fn attr_enum_variant_names_extraction() {
    let e = parse_enum_str("enum Token { Ident, Number, Whitespace }");
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, ["Ident", "Number", "Whitespace"]);
}

#[test]
fn attr_enum_variant_tuple_field_types() {
    let e = parse_enum_str("enum Expr { Binary(Box<Expr>, Op, Box<Expr>) }");
    match &e.variants[0].fields {
        Fields::Unnamed(f) => {
            assert_eq!(f.unnamed.len(), 3);
            assert_eq!(type_str(&f.unnamed[1].ty), "Op");
        }
        _ => panic!("expected unnamed fields"),
    }
}

#[test]
fn attr_enum_variant_named_fields() {
    let e = parse_enum_str("enum Stmt { Assign { target: Ident, value: Expr } }");
    match &e.variants[0].fields {
        Fields::Named(f) => {
            let names: Vec<_> = f
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            assert_eq!(names, ["target", "value"]);
        }
        _ => panic!("expected named fields"),
    }
}

#[test]
fn attr_enum_top_level_adze_grammar() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug)]
        #[adze::grammar("test")]
        enum Expr { Lit(i32) }
    };
    assert_eq!(e.attrs.len(), 2);
    let names = adze_attr_names(&e.attrs);
    assert_eq!(names, ["grammar"]);
}

// ============================================================================
// 4. attr_field_* — field attribute parsing (8 tests)
// ============================================================================

#[test]
fn attr_field_leaf_text() {
    let s: ItemStruct = parse_quote! {
        struct Plus {
            #[adze::leaf(text = "+")]
            symbol: (),
        }
    };
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "leaf"));
}

#[test]
fn attr_field_leaf_pattern() {
    let s: ItemStruct = parse_quote! {
        struct Number {
            #[adze::leaf(pattern = r"\d+")]
            raw: String,
        }
    };
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "leaf"));
    assert!(matches!(field.attrs[0].meta, Meta::List(_)));
}

#[test]
fn attr_field_skip_with_value() {
    let s: ItemStruct = parse_quote! {
        struct WithMeta {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "skip"));
}

#[test]
fn attr_field_repeat_non_empty() {
    let s: ItemStruct = parse_quote! {
        struct Items {
            #[adze::repeat(non_empty = true)]
            list: Vec<Item>,
        }
    };
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "repeat"));
    let meta_str = meta_to_string(&field.attrs[0]);
    assert!(meta_str.contains("non_empty"));
}

#[test]
fn attr_field_type_extraction_option() {
    let s: ItemStruct = parse_quote! {
        struct Opt { value: Option<String> }
    };
    let field = &named_fields(&s)[0];
    let (inner, extracted) = try_extract_inner_type(&field.ty, "Option", &empty_skip_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn attr_field_type_extraction_vec() {
    let s: ItemStruct = parse_quote! {
        struct List { items: Vec<Token> }
    };
    let field = &named_fields(&s)[0];
    let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &empty_skip_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "Token");
}

#[test]
fn attr_field_doc_comment_present() {
    let d = parse_derive_str("struct S { /// A field\n val: i32 }");
    match &d.data {
        syn::Data::Struct(data) => {
            let fields: Vec<_> = data.fields.iter().collect();
            assert!(!fields[0].attrs.is_empty());
            let meta_str = meta_to_string(&fields[0].attrs[0]);
            assert!(meta_str.contains("doc"));
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn attr_field_multiple_attrs_on_one_field() {
    let s: ItemStruct = parse_quote! {
        struct Delimited {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = &named_fields(&s)[0];
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names, ["repeat", "delimited"]);
}

// ============================================================================
// 5. attr_generic_* — generic type handling (8 tests)
// ============================================================================

#[test]
fn attr_generic_vec_is_parameterized() {
    let ty = parse_type_str("Vec<i32>");
    assert!(is_parameterized(&ty));
}

#[test]
fn attr_generic_option_is_parameterized() {
    let ty = parse_type_str("Option<String>");
    assert!(is_parameterized(&ty));
}

#[test]
fn attr_generic_plain_not_parameterized() {
    let ty = parse_type_str("i32");
    assert!(!is_parameterized(&ty));
}

#[test]
fn attr_generic_extract_through_box() {
    let ty = parse_type_str("Box<Vec<Token>>");
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &box_skip_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "Token");
}

#[test]
fn attr_generic_extract_through_arc_box() {
    let ty = parse_type_str("Arc<Box<Vec<Expr>>>");
    let skip = HashSet::from(["Arc", "Box"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&inner), "Expr");
}

#[test]
fn attr_generic_filter_strips_box() {
    let ty = parse_type_str("Box<String>");
    let filtered = filter_inner_type(&ty, &box_skip_set());
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn attr_generic_filter_strips_nested_wrappers() {
    let ty = parse_type_str("Box<Arc<u64>>");
    let filtered = filter_inner_type(&ty, &box_skip_set());
    assert_eq!(type_str(&filtered), "u64");
}

#[test]
fn attr_generic_wrap_leaf_simple() {
    let ty = parse_type_str("i32");
    let wrapped = wrap_leaf_type(&ty, &empty_skip_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < i32 >");
}

// ============================================================================
// 6. attr_validate_* — attribute validation (8 tests)
// ============================================================================

#[test]
fn attr_validate_adze_language_detected() {
    let attr: Attribute = parse_quote!(#[adze::language]);
    assert!(is_adze_attr(&attr, "language"));
}

#[test]
fn attr_validate_adze_grammar_detected() {
    let attr: Attribute = parse_quote!(#[adze::grammar("test")]);
    assert!(is_adze_attr(&attr, "grammar"));
    assert!(!is_adze_attr(&attr, "language"));
}

#[test]
fn attr_validate_adze_leaf_detected() {
    let attr: Attribute = parse_quote!(#[adze::leaf(text = "fn")]);
    assert!(is_adze_attr(&attr, "leaf"));
}

#[test]
fn attr_validate_non_adze_derive_rejected() {
    let attr: Attribute = parse_quote!(#[derive(Clone)]);
    assert!(!is_adze_attr(&attr, "grammar"));
    assert!(!is_adze_attr(&attr, "leaf"));
    assert!(!is_adze_attr(&attr, "language"));
}

#[test]
fn attr_validate_single_segment_rejected() {
    let attr: Attribute = parse_quote!(#[inline]);
    assert!(!is_adze_attr(&attr, "inline"));
}

#[test]
fn attr_validate_wrong_namespace_rejected() {
    let attr: Attribute = parse_quote!(#[serde::rename]);
    assert!(!is_adze_attr(&attr, "rename"));
}

#[test]
fn attr_validate_field_then_params_with_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn attr_validate_field_then_params_multiple() {
    let ftp: FieldThenParams = parse_quote!(u32, min = 0, max = 100);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

// ============================================================================
// 7. attr_error_* — error handling for invalid attrs (8 tests)
// ============================================================================

#[test]
fn attr_error_parse_str_invalid_struct_fails() {
    let result = syn::parse_str::<ItemStruct>("not a struct at all!!!");
    assert!(result.is_err());
}

#[test]
fn attr_error_parse_str_invalid_enum_fails() {
    let result = syn::parse_str::<ItemEnum>("123 bogus");
    assert!(result.is_err());
}

#[test]
fn attr_error_parse_str_incomplete_struct() {
    let result = syn::parse_str::<ItemStruct>("struct { x: }");
    assert!(result.is_err());
}

#[test]
fn attr_error_parse_str_invalid_type() {
    let result = syn::parse_str::<Type>("+++");
    assert!(result.is_err());
}

#[test]
fn attr_error_parse2_empty_token_stream() {
    let empty = TokenStream::new();
    let result = parse2::<ItemStruct>(empty);
    assert!(result.is_err());
}

#[test]
fn attr_error_parse_str_missing_field_type() {
    let result = syn::parse_str::<ItemStruct>("struct S { x: , y: i32 }");
    assert!(result.is_err());
}

#[test]
fn attr_error_extract_no_match_returns_original() {
    let ty = parse_type_str("HashMap<String, i32>");
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn attr_error_extract_plain_type_returns_original() {
    let ty = parse_type_str("u64");
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "u64");
}

// ============================================================================
// 8. attr_complex_* — complex attribute combinations (8 tests)
// ============================================================================

#[test]
fn attr_complex_struct_with_all_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        #[adze::extra]
        struct Triple { x: i32 }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, ["language", "word", "extra"]);
}

#[test]
fn attr_complex_enum_variants_mixed_prec() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, Box<Expr>),
        }
    };
    assert_eq!(e.variants.len(), 4);
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_left"));
    assert!(is_adze_attr(&e.variants[3].attrs[0], "prec_right"));
}

#[test]
fn attr_complex_field_repeat_delimited_combo() {
    let s: ItemStruct = parse_quote! {
        struct FnCall {
            #[adze::leaf(text = "(")]
            open_paren: (),
            #[adze::repeat(non_empty = false)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            args: Vec<Expr>,
            #[adze::leaf(text = ")")]
            close_paren: (),
        }
    };
    let fields = named_fields(&s);
    assert_eq!(fields.len(), 3);
    assert!(is_adze_attr(&fields[0].attrs[0], "leaf"));
    let middle_names = adze_attr_names(&fields[1].attrs);
    assert_eq!(middle_names, ["repeat", "delimited"]);
    assert!(is_adze_attr(&fields[2].attrs[0], "leaf"));
}

#[test]
fn attr_complex_wrap_leaf_skips_containers() {
    let ty = parse_type_str("Vec<String>");
    let wrapped = wrap_leaf_type(&ty, &container_skip_set());
    assert_eq!(type_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn attr_complex_wrap_leaf_nested_option_vec() {
    let ty = parse_type_str("Option<Vec<Expr>>");
    let wrapped = wrap_leaf_type(&ty, &container_skip_set());
    assert_eq!(
        type_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < Expr > > >"
    );
}

#[test]
fn attr_complex_roundtrip_attrs_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        struct Roundtrip { x: i32 }
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    let names = adze_attr_names(&reparsed.attrs);
    assert_eq!(names, ["language", "extra"]);
}

#[test]
fn attr_complex_struct_lifetime_generic() {
    let s = parse_struct_str("struct Borrowed<'a> { data: &'a str }");
    assert_eq!(s.generics.params.len(), 1);
    assert!(s.generics.where_clause.is_none());
}

#[test]
fn attr_complex_struct_generic_with_where_clause() {
    let s = parse_struct_str("struct Bounded<T> where T: Send + Sync { value: T }");
    assert!(s.generics.where_clause.is_some());
    assert_eq!(s.generics.params.len(), 1);
}
