#![allow(clippy::needless_range_loop)]

//! Comprehensive error-handling and edge-case tests for the adze-macro crate.
//!
//! Tests cover:
//! - Malformed attribute parsing and graceful parse failures
//! - Type extraction edge cases (nested, recursive, non-path types)
//! - Grammar structure boundary conditions
//! - NameValueExpr / FieldThenParams error paths
//! - Attribute recognition edge cases (misspellings, wrong paths)
//! - Leaf, prec, skip, extra, word, delimited, repeat attribute edge cases
//! - Grammar module structural validation
//! - wrap_leaf_type / filter_inner_type corner cases

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|a| {
            let segs: Vec<_> = a.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn expansion_skip_set() -> HashSet<&'static str> {
    ["Spanned", "Box", "Option", "Vec"].into_iter().collect()
}

// =====================================================================
// §1  Malformed token stream parsing – graceful failures
// =====================================================================

#[test]
fn err_01_empty_token_stream_not_parseable_as_struct() {
    let tokens = TokenStream::new();
    let result: Result<ItemStruct, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn err_02_empty_token_stream_not_parseable_as_enum() {
    let tokens = TokenStream::new();
    let result: Result<ItemEnum, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn err_03_empty_token_stream_not_parseable_as_mod() {
    let tokens = TokenStream::new();
    let result: Result<ItemMod, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn err_04_bare_identifier_not_parseable_as_struct() {
    let tokens = quote! { foo };
    let result: Result<ItemStruct, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn err_05_integer_literal_not_parseable_as_struct() {
    let tokens = quote! { 42 };
    let result: Result<ItemStruct, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn err_06_string_literal_not_parseable_as_mod() {
    let tokens = quote! { "hello" };
    let result: Result<ItemMod, _> = syn::parse2(tokens);
    assert!(result.is_err());
}

// =====================================================================
// §2  NameValueExpr parsing edge cases
// =====================================================================

#[test]
fn err_07_name_value_expr_missing_value_fails() {
    let result: Result<NameValueExpr, _> = syn::parse_str("key =");
    assert!(result.is_err());
}

#[test]
fn err_08_name_value_expr_missing_eq_fails() {
    let result: Result<NameValueExpr, _> = syn::parse_str("key \"value\"");
    assert!(result.is_err());
}

#[test]
fn err_09_name_value_expr_empty_input_fails() {
    let result: Result<NameValueExpr, _> = syn::parse_str("");
    assert!(result.is_err());
}

#[test]
fn err_10_name_value_expr_valid_string_value() {
    let nv: NameValueExpr = syn::parse_str("text = \"hello\"").unwrap();
    assert_eq!(nv.path.to_string(), "text");
}

#[test]
fn err_11_name_value_expr_valid_int_value() {
    let nv: NameValueExpr = syn::parse_str("level = 5").unwrap();
    assert_eq!(nv.path.to_string(), "level");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &nv.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("Expected int literal");
    }
}

#[test]
fn err_12_name_value_expr_negative_int_value() {
    let nv: NameValueExpr = syn::parse_str("level = -3").unwrap();
    assert_eq!(nv.path.to_string(), "level");
}

#[test]
fn err_13_name_value_expr_closure_value() {
    let nv: NameValueExpr = syn::parse_str("transform = |v| v.parse().unwrap()").unwrap();
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

// =====================================================================
// §3  FieldThenParams parsing edge cases
// =====================================================================

#[test]
fn err_14_field_then_params_type_only() {
    let ftp: FieldThenParams = syn::parse_str("String").unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn err_15_field_then_params_with_single_param() {
    let ftp: FieldThenParams = syn::parse_str("i32, name = \"test\"").unwrap();
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn err_16_field_then_params_with_multiple_params() {
    let ftp: FieldThenParams = syn::parse_str("u64, a = 1, b = 2, c = 3").unwrap();
    assert_eq!(ftp.params.len(), 3);
}

#[test]
fn err_17_field_then_params_empty_input_fails() {
    let result: Result<FieldThenParams, _> = syn::parse_str("");
    assert!(result.is_err());
}

// =====================================================================
// §4  try_extract_inner_type edge cases
// =====================================================================

#[test]
fn err_18_extract_plain_type_no_generics() {
    let (inner, ok) = try_extract_inner_type(&ty("String"), "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_19_extract_reference_type_returns_unchanged() {
    let t: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn err_20_extract_tuple_type_returns_unchanged() {
    let t: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "(i32 , u32)");
}

#[test]
fn err_21_extract_array_type_returns_unchanged() {
    let t: Type = parse_quote!([u8; 4]);
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "[u8 ; 4]");
}

#[test]
fn err_22_extract_slice_ref_type_returns_unchanged() {
    let t: Type = parse_quote!(&[u8]);
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& [u8]");
}

#[test]
fn err_23_extract_deeply_nested_through_skip() {
    let t = ty("Spanned<Box<Option<Vec<String>>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &expansion_skip_set());
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_24_extract_skip_chain_no_target_returns_original() {
    let t = ty("Box<Spanned<String>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &expansion_skip_set());
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < Spanned < String > >");
}

#[test]
fn err_25_extract_option_of_option() {
    let t = ty("Option<Option<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Option < i32 >");
}

#[test]
fn err_26_extract_vec_of_vec() {
    let t = ty("Vec<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Vec < u8 >");
}

#[test]
fn err_27_extract_with_empty_skip_set() {
    let t = ty("Box<Vec<String>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < Vec < String > >");
}

#[test]
fn err_28_extract_wrong_target_name() {
    let t = ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Vec < String >");
}

// =====================================================================
// §5  filter_inner_type edge cases
// =====================================================================

#[test]
fn err_29_filter_no_wrapper_returns_same() {
    let t = ty("String");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn err_30_filter_empty_skip_set_returns_same() {
    let t = ty("Box<String>");
    let filtered = filter_inner_type(&t, &skip(&[]));
    assert_eq!(ts(&filtered), "Box < String >");
}

#[test]
fn err_31_filter_double_wrapper() {
    let t = ty("Box<Box<i32>>");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "i32");
}

#[test]
fn err_32_filter_mixed_wrappers() {
    let t = ty("Spanned<Box<Option<u64>>>");
    let filtered = filter_inner_type(&t, &expansion_skip_set());
    assert_eq!(ts(&filtered), "u64");
}

#[test]
fn err_33_filter_reference_type_unchanged() {
    let t: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "& str");
}

#[test]
fn err_34_filter_tuple_type_unchanged() {
    let t: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "(i32 , u32)");
}

#[test]
fn err_35_filter_non_skip_generic_unchanged() {
    let t = ty("HashMap<String, i32>");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "HashMap < String , i32 >");
}

// =====================================================================
// §6  wrap_leaf_type edge cases
// =====================================================================

#[test]
fn err_36_wrap_plain_type() {
    let t = ty("String");
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn err_37_wrap_reference_type() {
    let t: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn err_38_wrap_array_type() {
    let t: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn err_39_wrap_tuple_type() {
    let t: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn err_40_wrap_vec_skipped_inner_wrapped() {
    let t = ty("Vec<String>");
    let wrapped = wrap_leaf_type(&t, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn err_41_wrap_option_skipped_inner_wrapped() {
    let t = ty("Option<i32>");
    let wrapped = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(ts(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn err_42_wrap_nested_skip_types() {
    let t = ty("Vec<Option<String>>");
    let wrapped = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(
        ts(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn err_43_wrap_non_skip_generic_wraps_whole() {
    let t = ty("HashMap<String, i32>");
    let wrapped = wrap_leaf_type(&t, &skip(&["Vec"]));
    assert_eq!(
        ts(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// =====================================================================
// §7  Attribute recognition edge cases
// =====================================================================

#[test]
fn err_44_non_adze_attr_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[serde(rename = "test")]
        pub struct Foo {
            value: i32,
        }
    };
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn err_45_single_segment_attr_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        pub struct Foo {
            value: i32,
        }
    };
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn err_46_three_segment_attr_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[a::b::c]
        pub struct Foo {
            value: i32,
        }
    };
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn err_47_wrong_prefix_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[not_adze::language]
        pub struct Foo {
            value: i32,
        }
    };
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn err_48_multiple_adze_attrs_all_collected() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        #[adze::word]
        pub struct Root {}
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["language", "extra", "word"]);
}

#[test]
fn err_49_mixed_adze_and_non_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        #[serde(rename = "test")]
        pub struct Root {}
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["language"]);
}

#[test]
fn err_50_is_adze_attr_exact_match() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Foo {}
    };
    let attr = &s.attrs[0];
    assert!(is_adze_attr(attr, "language"));
    assert!(!is_adze_attr(attr, "leaf"));
    assert!(!is_adze_attr(attr, "grammar"));
    assert!(!is_adze_attr(attr, "Language"));
}

// =====================================================================
// §8  Grammar module structural edge cases
// =====================================================================

#[test]
fn err_51_grammar_module_with_no_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("empty")]
        mod grammar {}
    });
    assert!(module_items(&m).is_empty());
}

#[test]
fn err_52_grammar_module_no_language_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("nolang")]
        mod grammar {
            pub struct Helper {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let has_lang = module_items(&m).iter().any(|item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
        Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
        _ => false,
    });
    assert!(!has_lang);
}

#[test]
fn err_53_grammar_name_empty_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert!(s.value().is_empty());
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_54_grammar_name_with_special_chars() {
    let m = parse_mod(quote! {
        #[adze::grammar("my-grammar_v2.0")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my-grammar_v2.0");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_55_grammar_name_with_unicode() {
    let m = parse_mod(quote! {
        #[adze::grammar("日本語")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "日本語");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_56_module_with_only_use_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("uses")]
        mod grammar {
            use std::fmt;
            use std::collections::HashMap;
        }
    });
    let structs: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .collect();
    assert!(structs.is_empty());
}

#[test]
fn err_57_module_with_function_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("funcs")]
        mod grammar {
            fn helper() -> bool { true }
        }
    });
    let fns: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Fn(_)))
        .collect();
    assert_eq!(fns.len(), 1);
}

#[test]
fn err_58_multiple_language_annotations() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root1 {}

            #[adze::language]
            pub struct Root2 {}
        }
    });
    let lang_count = module_items(&m)
        .iter()
        .filter(|item| match item {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
            _ => false,
        })
        .count();
    assert_eq!(lang_count, 2);
}

// =====================================================================
// §9  Leaf attribute parameter edge cases
// =====================================================================

#[test]
fn err_59_leaf_with_empty_text() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(text = "")]
            _empty: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert!(s.value().is_empty());
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_60_leaf_with_complex_regex_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
            ident: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"[a-zA-Z_][a-zA-Z0-9_]*");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_61_leaf_with_escaped_chars_in_text() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(text = "\n")]
            _nl: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "\n");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_62_leaf_text_single_char() {
    let nv: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
}

#[test]
fn err_63_leaf_pattern_dot_star() {
    let nv: NameValueExpr = parse_quote!(pattern = ".*");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), ".*");
    } else {
        panic!("Expected string literal");
    }
}

// =====================================================================
// §10  Precedence attribute edge cases
// =====================================================================

#[test]
fn err_64_prec_zero_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(0)]
            Zero(Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("Expected int literal");
    }
}

#[test]
fn err_65_prec_large_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(9999)]
            High(Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 9999);
    } else {
        panic!("Expected int literal");
    }
}

#[test]
fn err_66_prec_left_and_prec_right_on_different_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>),
            #[adze::prec_right(2)]
            Pow(Box<Expr>),
        }
    };
    let first_attrs = adze_attr_names(&e.variants[0].attrs);
    let second_attrs = adze_attr_names(&e.variants[1].attrs);
    assert_eq!(first_attrs, vec!["prec_left"]);
    assert_eq!(second_attrs, vec!["prec_right"]);
}

#[test]
fn err_67_prec_negative_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(-1)]
            Neg(Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    // Negative literal parses as a unary expression, not a direct int literal
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert!(matches!(expr, syn::Expr::Unary(_)));
}

// =====================================================================
// §11  Skip attribute edge cases
// =====================================================================

#[test]
fn err_68_skip_with_bool_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = expr
    {
        assert!(!b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn err_69_skip_with_int_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(0)]
            count: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("Expected int literal");
    }
}

#[test]
fn err_70_skip_with_string_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(String::new())]
            label: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    // Should parse as a method call expression
    assert!(matches!(expr, syn::Expr::Call(_)));
}

#[test]
fn err_71_skip_with_vec_new_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(Vec::new())]
            items: Vec<i32>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert!(matches!(expr, syn::Expr::Call(_)));
}

// =====================================================================
// §12  Extra attribute edge cases
// =====================================================================

#[test]
fn err_72_extra_is_path_style_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {}
    };
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
    assert!(matches!(attr.meta, syn::Meta::Path(_)));
}

#[test]
fn err_73_extra_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::extra]
        enum Comment {
            #[adze::leaf(pattern = r"//[^\n]*")]
            Line,
            #[adze::leaf(pattern = r"/\*[\s\S]*?\*/")]
            Block,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

// =====================================================================
// §13  Word attribute edge cases
// =====================================================================

#[test]
fn err_74_word_is_path_style_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
    assert!(matches!(attr.meta, syn::Meta::Path(_)));
}

#[test]
fn err_75_word_combined_with_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        pub struct Root {}
    };
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"language".to_string()));
    assert!(names.contains(&"word".to_string()));
}

// =====================================================================
// §14  Enum variant field type edge cases
// =====================================================================

#[test]
fn err_76_enum_unit_variant_no_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
}

#[test]
fn err_77_enum_tuple_variant_single_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
        }
    };
    if let Fields::Unnamed(u) = &e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 1);
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn err_78_enum_struct_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Binary { left: Box<Expr>, op: String, right: Box<Expr> },
        }
    };
    if let Fields::Named(n) = &e.variants[0].fields {
        assert_eq!(n.named.len(), 3);
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn err_79_enum_mixed_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::leaf(text = "nil")]
            Nil,
            Num(i32),
            Binary { left: Box<Expr>, right: Box<Expr> },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// =====================================================================
// §15  Grammar with complex type combinations
// =====================================================================

#[test]
fn err_80_option_field_detected() {
    let t: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn err_81_vec_field_detected() {
    let t: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_82_box_in_skip_set_peels_through() {
    let t = ty("Box<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn err_83_spanned_in_skip_set_peels_through() {
    let t = ty("Spanned<Option<String>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&["Spanned"]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_84_grammar_module_with_pub_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("vis")]
        pub mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(m.vis != syn::Visibility::Inherited);
}

#[test]
fn err_85_grammar_module_private_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("priv")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Inherited));
}

#[test]
fn err_86_struct_with_no_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Empty {}
    };
    assert_eq!(s.fields.len(), 0);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn err_87_struct_with_many_leaf_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct ManyLeaves {
            #[adze::leaf(text = "(")]
            _lp: (),
            #[adze::leaf(pattern = r"\d+")]
            num: String,
            #[adze::leaf(text = ",")]
            _comma: (),
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::leaf(text = ")")]
            _rp: (),
        }
    };
    let leaf_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .count();
    assert_eq!(leaf_count, 5);
}

#[test]
fn err_88_enum_with_many_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "+")]
            Add,
            #[adze::leaf(text = "-")]
            Sub,
            #[adze::leaf(text = "*")]
            Mul,
            #[adze::leaf(text = "/")]
            Div,
            #[adze::leaf(text = "%")]
            Mod,
            #[adze::leaf(text = "**")]
            Pow,
        }
    };
    assert_eq!(e.variants.len(), 6);
}

// =====================================================================
// §16  Delimited/repeat attribute recognition
// =====================================================================

#[test]
fn err_89_delimited_attr_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(Item, ",")]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

#[test]
fn err_90_repeat_attr_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::repeat]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// =====================================================================
// §17  Complex grammar structure validation
// =====================================================================

#[test]
fn err_91_grammar_with_enum_and_struct_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(Number),
                #[adze::leaf(text = "nil")]
                Nil,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    });
    assert!(find_enum_in_mod(&m, "Expr").is_some());
    assert!(find_struct_in_mod(&m, "Number").is_some());
}

#[test]
fn err_92_grammar_with_extra_whitespace_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("ws")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let ws = find_struct_in_mod(&m, "Whitespace").unwrap();
    assert!(ws.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn err_93_grammar_with_word_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("wordy")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let ident = find_struct_in_mod(&m, "Identifier").unwrap();
    assert!(ident.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// =====================================================================
// §18  Field type extraction in struct context
// =====================================================================

#[test]
fn err_94_struct_field_type_string() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "String");
}

#[test]
fn err_95_struct_field_type_option_string() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            name: Option<String>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let (inner, ok) = try_extract_inner_type(&field.ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_96_struct_field_type_vec_of_custom() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            children: Vec<Child>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let (inner, ok) = try_extract_inner_type(&field.ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Child");
}

#[test]
fn err_97_struct_field_type_box_of_self() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            child: Box<Node>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let filtered = filter_inner_type(&field.ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Node");
}

// =====================================================================
// §19  wrap_leaf_type with expansion_skip_set
// =====================================================================

#[test]
fn err_98_wrap_with_expansion_skip_set() {
    let t = ty("Vec<Option<String>>");
    let wrapped = wrap_leaf_type(&t, &expansion_skip_set());
    assert_eq!(
        ts(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn err_99_wrap_box_in_expansion_skip_set() {
    let t = ty("Box<i32>");
    let wrapped = wrap_leaf_type(&t, &expansion_skip_set());
    assert_eq!(ts(&wrapped), "Box < adze :: WithLeaf < i32 > >");
}

#[test]
fn err_100_wrap_spanned_in_expansion_skip_set() {
    let t = ty("Spanned<f64>");
    let wrapped = wrap_leaf_type(&t, &expansion_skip_set());
    assert_eq!(ts(&wrapped), "Spanned < adze :: WithLeaf < f64 > >");
}

// =====================================================================
// §20  Additional edge cases for completeness
// =====================================================================

#[test]
fn err_101_name_value_expr_with_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"hello\nworld");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), r"hello\nworld");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn err_102_name_value_expr_float_value() {
    let nv: NameValueExpr = parse_quote!(scale = 1.5);
    assert_eq!(nv.path.to_string(), "scale");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Float(_),
        ..
    }) = &nv.expr
    {
        // ok
    } else {
        panic!("Expected float literal");
    }
}

#[test]
fn err_103_extract_result_type_not_in_skip() {
    let t = ty("Result<String, Error>");
    let (inner, ok) = try_extract_inner_type(&t, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn err_104_filter_triple_nested_wrappers() {
    let t = ty("Box<Option<Vec<bool>>>");
    let filtered = filter_inner_type(&t, &expansion_skip_set());
    assert_eq!(ts(&filtered), "bool");
}

#[test]
fn err_105_grammar_module_struct_enum_count() {
    let m = parse_mod(quote! {
        #[adze::grammar("count")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                A(NodeA),
                B(NodeB),
            }
            pub struct NodeA {}
            pub struct NodeB {}
            pub struct NodeC {}
        }
    });
    let struct_count = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    let enum_count = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Enum(_)))
        .count();
    assert_eq!(struct_count, 3);
    assert_eq!(enum_count, 1);
}

#[test]
fn err_106_leaf_on_enum_variant_recognized() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    for v in &e.variants {
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn err_107_name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &nv.expr
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn err_108_field_then_params_trailing_comma_allowed() {
    // Punctuated::parse_terminated supports trailing commas
    let ftp: FieldThenParams = syn::parse_str("u32, a = 1,").unwrap();
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn err_109_extract_inner_path_type_no_generics() {
    let t = ty("MyCustomType");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "MyCustomType");
}

#[test]
fn err_110_wrap_leaf_preserves_nested_skip_types() {
    let t = ty("Option<Vec<Option<String>>>");
    let wrapped = wrap_leaf_type(&t, &skip(&["Option", "Vec"]));
    assert_eq!(
        ts(&wrapped),
        "Option < Vec < Option < adze :: WithLeaf < String > > > >"
    );
}
