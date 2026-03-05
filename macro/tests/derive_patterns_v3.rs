//! Tests for derive and attribute patterns in adze-macro.
//!
//! Covers:
//!   - NameValueExpr parsing (text, pattern, transform, various literal types)
//!   - FieldThenParams parsing (field + param combinations)
//!   - Derive attribute combinations with macro attrs
//!   - Type annotation patterns with attributes
//!   - Token stream roundtrip (parse to AST, back to tokens)
//!   - Attribute validation (valid and invalid forms)
//!   - Edge cases (empty attrs, special chars, complex expressions)

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Fields, GenericArgument, ItemEnum, ItemStruct, Lit,
    PathArguments, Token, Type, TypePath, parse_quote, parse2, punctuated::Punctuated,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_derive(tokens: TokenStream) -> DeriveInput {
    parse2::<DeriveInput>(tokens).expect("failed to parse DeriveInput")
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn field_types(fields: &Fields) -> Vec<String> {
    fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn extract_inner_type<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(seg) = path.segments.last()
        && seg.ident == wrapper
        && let PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

fn nv_path(nv: &NameValueExpr) -> String {
    nv.path.to_string()
}

fn nv_str_value(nv: &NameValueExpr) -> Option<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = &nv.expr
    {
        Some(s.value())
    } else {
        None
    }
}

fn nv_int_value(nv: &NameValueExpr) -> Option<i64> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = &nv.expr
    {
        i.base10_parse::<i64>().ok()
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 1: NameValueExpr parsing (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn nve_text_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nv_path(&nv), "text");
    assert_eq!(nv_str_value(&nv).unwrap(), "hello");
}

#[test]
fn nve_pattern_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+(\.\d+)?");
    assert_eq!(nv_path(&nv), "pattern");
    assert_eq!(nv_str_value(&nv).unwrap(), r"\d+(\.\d+)?");
}

#[test]
fn nve_transform_closure() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nv_path(&nv), "transform");
    assert!(matches!(nv.expr, Expr::Closure(_)));
}

#[test]
fn nve_integer_precedence() {
    let nv: NameValueExpr = parse_quote!(precedence = 99);
    assert_eq!(nv_path(&nv), "precedence");
    assert_eq!(nv_int_value(&nv).unwrap(), 99);
}

#[test]
fn nve_bool_value() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv_path(&nv), "non_empty");
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(b), ..
    }) = &nv.expr
    {
        assert!(b.value);
    } else {
        panic!("expected bool literal");
    }
}

#[test]
fn nve_punctuated_text_and_transform() {
    let params: Punctuated<NameValueExpr, Token![,]> =
        parse_quote!(text = "+", transform = |v| v.to_string());
    assert_eq!(params.len(), 2);
    assert_eq!(nv_path(&params[0]), "text");
    assert_eq!(nv_path(&params[1]), "transform");
    assert_eq!(nv_str_value(&params[0]).unwrap(), "+");
}

#[test]
fn nve_punctuated_pattern_transform() {
    let params: Punctuated<NameValueExpr, Token![,]> =
        parse_quote!(pattern = r"[a-zA-Z_]\w*", transform = |v| v.to_uppercase());
    assert_eq!(params.len(), 2);
    assert_eq!(nv_str_value(&params[0]).unwrap(), r"[a-zA-Z_]\w*");
    assert!(matches!(params[1].expr, Expr::Closure(_)));
}

#[test]
fn nve_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv_path(&nv), "offset");
    // Negative literals parse as Expr::Unary, not Expr::Lit
    assert!(matches!(nv.expr, Expr::Unary(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 2: FieldThenParams parsing (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn ftp_bare_type_only() {
    let ftp: FieldThenParams = parse_quote!(MyToken);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "MyToken");
}

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(Separator, text = ",");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(nv_path(&ftp.params[0]), "text");
    assert_eq!(nv_str_value(&ftp.params[0]).unwrap(), ",");
}

#[test]
fn ftp_type_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(
        Number,
        pattern = r"\d+",
        transform = |v| v.parse::<i32>().unwrap()
    );
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(nv_path(&ftp.params[0]), "pattern");
    assert_eq!(nv_path(&ftp.params[1]), "transform");
}

#[test]
fn ftp_with_leaf_attr_on_field() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ";")]
        ()
    );
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(is_adze_attr(&ftp.field.attrs[0], "leaf"));
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_generic_type_field() {
    let ftp: FieldThenParams = parse_quote!(Vec<Item>);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Vec < Item >");
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_option_type_with_params() {
    let ftp: FieldThenParams = parse_quote!(Option<Comma>, text = ",");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(
        ftp.field.ty.to_token_stream().to_string(),
        "Option < Comma >"
    );
}

#[test]
fn ftp_box_type_field() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Box < Expr >");
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 3: Derive attribute combinations (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn derive_debug_with_language_attr() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        #[adze::language]
        pub struct Root {
            items: Vec<Item>,
        }
    });
    assert_eq!(di.ident, "Root");
    assert_eq!(di.attrs.len(), 2);
    assert!(di.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn derive_debug_clone_with_extra_attr() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone)]
        #[adze::extra]
        struct Whitespace {
            _ws: (),
        }
    });
    assert_eq!(di.attrs.len(), 2);
    assert!(di.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn derive_partialeq_eq_struct() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Token {
            value: String,
        }
    });
    assert_eq!(di.ident, "Token");
    if let syn::Data::Struct(ref s) = di.data {
        assert_eq!(s.fields.len(), 1);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn enum_variant_with_prec_left() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            Num(i32),
        }
    };
    let add_variant = &e.variants[0];
    assert!(
        add_variant
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
}

#[test]
fn enum_variant_with_prec_right() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
}

#[test]
fn enum_variant_with_plain_prec() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(3)]
            Special(Box<Expr>),
        }
    };
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec")));
}

#[test]
fn struct_with_leaf_and_skip_fields() {
    let s: ItemStruct = parse_quote! {
        struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn derive_with_serde_and_macro_attrs() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        pub struct Program {
            statements: Vec<Statement>,
        }
    });
    assert_eq!(di.attrs.len(), 2);
    let derive_attr = &di.attrs[0];
    assert_eq!(
        derive_attr
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        "derive"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 4: Type annotation patterns (7 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_vec_of_box() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    assert!(extract_inner_type(&ty, "Vec").is_some());
    let inner = extract_inner_type(&ty, "Vec").unwrap();
    assert!(extract_inner_type(inner, "Box").is_some());
}

#[test]
fn type_option_of_string() {
    let ty: Type = parse_quote!(Option<String>);
    let inner = extract_inner_type(&ty, "Option").unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn type_plain_no_generics() {
    let ty: Type = parse_quote!(i32);
    assert!(extract_inner_type(&ty, "Vec").is_none());
    assert!(extract_inner_type(&ty, "Option").is_none());
}

#[test]
fn type_reference() {
    let ty: Type = parse_quote!(&str);
    assert!(extract_inner_type(&ty, "Vec").is_none());
}

#[test]
fn type_tuple() {
    let ty: Type = parse_quote!((i32, String));
    assert!(extract_inner_type(&ty, "Vec").is_none());
}

#[test]
fn type_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let inner_vec = extract_inner_type(&ty, "Option").unwrap();
    assert_eq!(inner_vec.to_token_stream().to_string(), "Vec < u8 >");
    let inner_u8 = extract_inner_type(inner_vec, "Vec").unwrap();
    assert_eq!(inner_u8.to_token_stream().to_string(), "u8");
}

#[test]
fn type_result_with_two_params() {
    let ty: Type = parse_quote!(Result<String, Error>);
    // Result is not Vec/Option, so extraction should fail for those
    assert!(extract_inner_type(&ty, "Vec").is_none());
    assert!(extract_inner_type(&ty, "Option").is_none());
    // But extracting Result itself should work
    let inner = extract_inner_type(&ty, "Result").unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 5: Token stream roundtrip (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_named_struct() {
    let s: ItemStruct = parse_quote! {
        struct Foo { x: i32, y: String }
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(reparsed.ident, "Foo");
    assert_eq!(reparsed.fields.len(), 2);
}

#[test]
fn roundtrip_tuple_struct() {
    let s: ItemStruct = parse_quote! {
        struct Pair(i32, i32);
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(reparsed.ident, "Pair");
    assert!(matches!(reparsed.fields, Fields::Unnamed(_)));
}

#[test]
fn roundtrip_unit_struct() {
    let s: ItemStruct = parse_quote! {
        struct Marker;
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert!(matches!(reparsed.fields, Fields::Unit));
}

#[test]
fn roundtrip_enum() {
    let e: ItemEnum = parse_quote! {
        enum Dir { Up, Down, Left, Right }
    };
    let tokens = e.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(reparsed.variants.len(), 4);
}

#[test]
fn roundtrip_derive_input_attrs_preserved() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone)]
        struct W(u8);
    });
    let tokens = di.to_token_stream();
    let reparsed = parse_derive(tokens);
    assert_eq!(reparsed.attrs.len(), 1);
    assert_eq!(reparsed.ident, "W");
}

#[test]
fn roundtrip_struct_with_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            items: Vec<Item>,
        }
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert!(reparsed.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn roundtrip_name_value_expr() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+");
    let path_str = nv.path.to_string();
    let expr_str = nv.expr.to_token_stream().to_string();
    // Re-compose from components and verify
    assert_eq!(path_str, "pattern");
    assert!(expr_str.contains("\\d+"));
}

#[test]
fn roundtrip_field_then_params() {
    let ftp: FieldThenParams = parse_quote!(MyType, name = "test");
    let field_str = ftp.field.ty.to_token_stream().to_string();
    assert_eq!(field_str, "MyType");
    assert_eq!(ftp.params.len(), 1);
    // Re-verify after accessing components
    assert_eq!(nv_path(&ftp.params[0]), "name");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 6: Attribute validation (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn valid_leaf_text_attr() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = &field.attrs[0];
    assert!(is_adze_attr(attr, "leaf"));
    let params: Punctuated<NameValueExpr, Token![,]> =
        attr.parse_args_with(Punctuated::parse_terminated).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(nv_path(&params[0]), "text");
}

#[test]
fn valid_leaf_pattern_attr() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(pattern = r"[0-9]+")]
            digits: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = field.attrs[0]
        .parse_args_with(Punctuated::parse_terminated)
        .unwrap();
    assert_eq!(nv_path(&params[0]), "pattern");
    assert_eq!(nv_str_value(&params[0]).unwrap(), "[0-9]+");
}

#[test]
fn valid_leaf_pattern_and_transform() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<u32>().unwrap())]
            count: u32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = field.attrs[0]
        .parse_args_with(Punctuated::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(nv_path(&params[0]), "pattern");
    assert!(matches!(params[1].expr, Expr::Closure(_)));
}

#[test]
fn valid_skip_attr_bool() {
    let s: ItemStruct = parse_quote! {
        struct N {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(is_adze_attr(&field.attrs[0], "skip"));
}

#[test]
fn valid_prec_left_attr_integer() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            V(i32),
        }
    };
    let attr = &e.variants[0].attrs[0];
    assert!(is_adze_attr(attr, "prec_left"));
}

#[test]
fn invalid_name_value_missing_eq_fails() {
    let result = syn::parse_str::<NameValueExpr>("key value");
    assert!(result.is_err());
}

#[test]
fn invalid_name_value_missing_value_fails() {
    let result = syn::parse_str::<NameValueExpr>("key =");
    assert!(result.is_err());
}

#[test]
fn invalid_name_value_missing_key_fails() {
    let result = syn::parse_str::<NameValueExpr>("= 42");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 7: Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_struct_fields() {
    let s: ItemStruct = parse_quote! {
        struct Empty {}
    };
    assert_eq!(s.fields.len(), 0);
    assert!(matches!(s.fields, Fields::Named(_)));
}

#[test]
fn edge_enum_all_unit_variants() {
    let e: ItemEnum = parse_quote! {
        enum Direction { North, South, East, West }
    };
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
    }
}

#[test]
fn edge_deeply_nested_type_extraction() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn edge_wrap_deeply_nested_skip() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Vec<i32>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < Vec < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn edge_filter_triple_nested_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Box<u8>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "u8");
}

#[test]
fn edge_nve_path_expression_value() {
    let nv: NameValueExpr = parse_quote!(module = std::io);
    assert_eq!(nv_path(&nv), "module");
    assert!(matches!(nv.expr, Expr::Path(_)));
}

#[test]
fn edge_ftp_attributed_field_with_params() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        (),
        text = ";"
    );
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(nv_str_value(&ftp.params[0]).unwrap(), ";");
}

#[test]
fn edge_format_ident_generation() {
    let base = "Expr";
    let generated = format_ident!("{}Node", base);
    assert_eq!(generated, "ExprNode");

    let variant_name = format_ident!("variant_{}", 0usize);
    assert_eq!(variant_name, "variant_0");
}

#[test]
fn edge_ident_span_preserved() {
    let ident = Ident::new("test_ident", Span::call_site());
    assert_eq!(ident.to_string(), "test_ident");
    // Ident equality is by name only, not span
    let ident2 = Ident::new("test_ident", Span::call_site());
    assert_eq!(ident, ident2);
}

#[test]
fn edge_empty_punctuated_list() {
    let params: Punctuated<NameValueExpr, Token![,]> = Punctuated::new();
    assert!(params.is_empty());
    assert_eq!(params.len(), 0);
}

#[test]
fn edge_struct_with_many_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[adze::language]
        #[allow(dead_code)]
        pub struct Multi {
            data: Vec<u8>,
        }
    };
    assert_eq!(s.attrs.len(), 3);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn edge_enum_mixed_field_kinds() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            Num(i32),
            BinOp { left: Box<Expr>, op: String, right: Box<Expr> },
            Nil,
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[1].fields, Fields::Named(_)));
    assert!(matches!(e.variants[2].fields, Fields::Unit));
}

#[test]
fn edge_try_extract_non_path_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "[u8 ; 4]");
}

#[test]
fn edge_wrap_array_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn edge_quote_generated_struct() {
    let name = format_ident!("Generated");
    let tokens = quote! {
        struct #name {
            value: i32,
        }
    };
    let s: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(s.ident, "Generated");
    assert_eq!(field_types(&s.fields), ["i32"]);
}
