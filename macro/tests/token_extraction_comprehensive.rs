//! Comprehensive tests for token extraction and processing in the adze-macro crate.
//!
//! Covers token stream parsing, attribute extraction, NameValueExpr / FieldThenParams
//! parsing, type-level helpers (`try_extract_inner_type`, `filter_inner_type`,
//! `wrap_leaf_type`), pattern matching on adze-style attributes, error cases, and
//! edge cases including empty tokens, special characters, and unicode identifiers.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Fields, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

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

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    syn::parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    syn::parse2(tokens).expect("failed to parse enum")
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .expect("failed to parse leaf params")
}

fn default_skip_set() -> HashSet<&'static str> {
    HashSet::from(["Box", "Option", "Vec", "Spanned"])
}

// =============================================================================
// Section 1: NameValueExpr parsing (tests 1–10)
// =============================================================================

#[test]
fn nve_parse_text_key() {
    let nve: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nve.path, "text");
}

#[test]
fn nve_parse_pattern_key() {
    let nve: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nve.path, "pattern");
}

#[test]
fn nve_parse_transform_closure() {
    let nve: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nve.path, "transform");
    // The expression should round-trip without panic
    let _ = nve.expr.to_token_stream().to_string();
}

#[test]
fn nve_parse_non_empty_flag() {
    let nve: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nve.path, "non_empty");
}

#[test]
fn nve_parse_integer_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path, "precedence");
    assert!(nve.expr.to_token_stream().to_string().contains("42"));
}

#[test]
fn nve_parse_negative_integer() {
    let nve: NameValueExpr = parse_quote!(level = -1);
    assert_eq!(nve.path, "level");
    assert!(nve.expr.to_token_stream().to_string().contains("1"));
}

#[test]
fn nve_parse_float_value() {
    let nve: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nve.path, "weight");
}

#[test]
fn nve_parse_bool_false_value() {
    let nve: NameValueExpr = parse_quote!(enabled = false);
    assert_eq!(nve.path, "enabled");
}

#[test]
fn nve_parse_empty_string_value() {
    let nve: NameValueExpr = parse_quote!(text = "");
    assert_eq!(nve.path, "text");
}

#[test]
fn nve_parse_string_with_special_chars() {
    let nve: NameValueExpr = parse_quote!(text = "->>");
    assert_eq!(nve.path, "text");
    assert!(nve.expr.to_token_stream().to_string().contains("->>"));
}

// =============================================================================
// Section 2: FieldThenParams parsing (tests 11–18)
// =============================================================================

#[test]
fn ftp_type_only() {
    let ftp: FieldThenParams = parse_quote!(u32);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
}

#[test]
fn ftp_type_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(String, text = "hello");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "text");
}

#[test]
fn ftp_type_with_multiple_params() {
    let ftp: FieldThenParams =
        parse_quote!(i32, pattern = r"\d+", transform = |v| v.parse().unwrap());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path, "pattern");
    assert_eq!(ftp.params[1].path, "transform");
}

#[test]
fn ftp_boxed_type() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    assert!(ftp.params.is_empty());
    assert!(ftp.field.ty.to_token_stream().to_string().contains("Box"));
}

#[test]
fn ftp_option_type() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>, text = "?");
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn ftp_vec_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Token>);
    assert!(ftp.field.ty.to_token_stream().to_string().contains("Vec"));
}

#[test]
fn ftp_preserves_field_attrs() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = "+")]
        ()
    );
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(is_adze_attr(&ftp.field.attrs[0], "leaf"));
}

// =============================================================================
// Section 3: try_extract_inner_type (tests 19–27)
// =============================================================================

#[test]
fn extract_vec_string() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_option_i32() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn extract_box_vec_string() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_no_match_returns_original() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(
        inner.to_token_stream().to_string(),
        ty.to_token_stream().to_string()
    );
}

#[test]
fn extract_plain_type_not_extracted() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(u64);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
}

#[test]
fn extract_reference_type_not_extracted() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

#[test]
fn extract_tuple_type_not_extracted() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, u32));
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
}

#[test]
fn extract_nested_skip_over_types() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_spanned_vec() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Spanned<Vec<Number>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// =============================================================================
// Section 4: filter_inner_type (tests 28–34)
// =============================================================================

#[test]
fn filter_box_string() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_box_arc_string() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc"]);
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_no_match_unchanged() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        ty.to_token_stream().to_string()
    );
}

#[test]
fn filter_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        ty.to_token_stream().to_string()
    );
}

#[test]
fn filter_plain_type_unchanged() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(u32);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "u32");
}

#[test]
fn filter_reference_type_unchanged() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "& str");
}

#[test]
fn filter_tuple_type_unchanged() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!((u32, String));
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(u32 , String)");
}

// =============================================================================
// Section 5: wrap_leaf_type (tests 35–44)
// =============================================================================

#[test]
fn wrap_plain_type() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_i32() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_vec_wraps_inner() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_wraps_inner() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_box_wraps_inner() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Box<Expr>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Box < adze :: WithLeaf < Expr > >"
    );
}

#[test]
fn wrap_nested_vec_option() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_reference_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((u32, String));
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < (u32 , String) >"
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
fn wrap_non_skip_generic_wraps_entire() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// =============================================================================
// Section 6: Attribute extraction from struct annotations (tests 45–52)
// =============================================================================

#[test]
fn extract_language_attr_from_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            statements: Vec<Statement>,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn extract_extra_attr_from_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn extract_word_attr_from_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn extract_external_attr_from_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

#[test]
fn extract_leaf_text_params() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper {
            #[adze::leaf(text = "+")]
            _plus: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path, "text");
}

#[test]
fn extract_leaf_pattern_params() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper {
            #[adze::leaf(pattern = r"\d+")]
            _num: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path, "pattern");
}

#[test]
fn extract_leaf_pattern_and_transform() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn extract_skip_attr_value() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let val: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(val.to_token_stream().to_string(), "false");
}

// =============================================================================
// Section 7: Attribute extraction from enum annotations (tests 53–60)
// =============================================================================

#[test]
fn extract_language_attr_from_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn extract_prec_left_from_variant() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    let attr = variant
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    let val: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(val.to_token_stream().to_string(), "1");
}

#[test]
fn extract_prec_right_from_variant() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
        }
    };
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
}

#[test]
fn extract_prec_from_variant() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(3)]
            Compare(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
        }
    };
    let variant = &e.variants[0];
    let attr = variant
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let val: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(val.to_token_stream().to_string(), "3");
}

#[test]
fn extract_leaf_from_unit_variant() {
    let e: ItemEnum = parse_quote! {
        enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    assert_eq!(e.variants.len(), 2);
    for v in &e.variants {
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn extract_leaf_text_values_from_unit_variants() {
    let e: ItemEnum = parse_quote! {
        enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
            #[adze::leaf(text = "*")]
            Star,
        }
    };
    let texts: Vec<_> = e
        .variants
        .iter()
        .map(|v| {
            let attr = v.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            let params = leaf_params(attr);
            params[0].expr.to_token_stream().to_string()
        })
        .collect();
    assert_eq!(texts, vec!["\"+\"", "\"-\"", "\"*\""]);
}

#[test]
fn extract_repeat_non_empty_from_field() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            Numbers(
                #[adze::repeat(non_empty = true)]
                Vec<Number>
            ),
        }
    };
    let field = e.variants[0].fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path, "non_empty");
}

#[test]
fn extract_delimited_from_field() {
    let s: ItemStruct = parse_quote! {
        struct NumberList {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// =============================================================================
// Section 8: Grammar module attribute extraction (tests 61–65)
// =============================================================================

#[test]
fn grammar_module_has_grammar_attr() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    };
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
}

#[test]
fn grammar_attr_name_is_string_literal() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("my_lang")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    };
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let name_expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = name_expr
    {
        assert_eq!(s.value(), "my_lang");
    } else {
        panic!("Expected string literal grammar name");
    }
}

#[test]
fn find_language_annotated_type_in_module() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Root {
                A(#[adze::leaf(text = "a")] ()),
            }
            pub struct Helper {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    };
    let (_, items) = m.content.unwrap();
    let lang_type = items.iter().find_map(|item| match item {
        syn::Item::Enum(e) if e.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(e.ident.to_string())
        }
        syn::Item::Struct(s) if s.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(s.ident.to_string())
        }
        _ => None,
    });
    assert_eq!(lang_type.unwrap(), "Root");
}

#[test]
fn find_extra_types_in_module() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
    };
    let (_, items) = m.content.unwrap();
    let extras: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(s) if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) => {
                Some(s.ident.to_string())
            }
            _ => None,
        })
        .collect();
    assert_eq!(extras.len(), 2);
    assert!(extras.contains(&"Whitespace".to_string()));
    assert!(extras.contains(&"Comment".to_string()));
}

#[test]
fn collect_all_adze_attrs_from_module() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    };
    let top_attrs = adze_attr_names(&m.attrs);
    assert_eq!(top_attrs, vec!["grammar"]);
}

// =============================================================================
// Section 9: Token pattern matching (tests 66–72)
// =============================================================================

#[test]
fn is_sitter_attr_matches_adze_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct S;
    };
    assert!(is_adze_attr(&s.attrs[0], "language"));
    assert!(!is_adze_attr(&s.attrs[0], "extra"));
}

#[test]
fn is_sitter_attr_does_not_match_derive() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        struct S;
    };
    assert!(!is_adze_attr(&s.attrs[0], "language"));
}

#[test]
fn is_sitter_attr_does_not_match_other_crate() {
    let s: ItemStruct = parse_quote! {
        #[serde::rename("x")]
        struct S;
    };
    assert!(!is_adze_attr(&s.attrs[0], "rename"));
}

#[test]
fn attr_matching_all_adze_variants() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        #[adze::word]
        #[adze::external]
        struct S;
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["language", "extra", "word", "external"]);
}

#[test]
fn field_leaf_attr_pattern_detection() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            value: i32,
        }
    };
    let leaf_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .collect();
    assert_eq!(leaf_fields.len(), 2);
}

#[test]
fn discriminate_text_vs_pattern_leaf() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::leaf(text = "hello")]
            _text: (),
            #[adze::leaf(pattern = r"\w+")]
            _pat: String,
        }
    };
    for field in &s.fields {
        let attr = field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        let key = params[0].path.to_string();
        assert!(key == "text" || key == "pattern");
    }
}

#[test]
fn variant_has_both_prec_and_leaf_fields() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    let leaf_count = variant
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .count();
    assert_eq!(leaf_count, 1);
}

// =============================================================================
// Section 10: Error cases (tests 73–78)
// =============================================================================

#[test]
fn nve_parse_fails_on_missing_equals() {
    let result = syn::parse2::<NameValueExpr>(quote!(text));
    assert!(result.is_err());
}

#[test]
fn nve_parse_fails_on_missing_value() {
    let result = syn::parse2::<NameValueExpr>(quote!(text =));
    assert!(result.is_err());
}

#[test]
fn nve_parse_fails_on_empty_stream() {
    let result = syn::parse2::<NameValueExpr>(TokenStream::new());
    assert!(result.is_err());
}

#[test]
fn ftp_parse_fails_on_empty_stream() {
    let result = syn::parse2::<FieldThenParams>(TokenStream::new());
    assert!(result.is_err());
}

#[test]
fn struct_parse_fails_on_malformed_tokens() {
    let result = syn::parse2::<ItemStruct>(quote!(struct));
    assert!(result.is_err());
}

#[test]
fn mod_parse_fails_on_empty_stream() {
    let result = syn::parse2::<ItemMod>(TokenStream::new());
    assert!(result.is_err());
}

// =============================================================================
// Section 11: Edge cases – special characters and unicode (tests 79–86)
// =============================================================================

#[test]
fn leaf_text_with_multi_char_operator() {
    let nve: NameValueExpr = parse_quote!(text = ">>>=");
    assert_eq!(nve.path, "text");
    assert!(nve.expr.to_token_stream().to_string().contains(">>>="));
}

#[test]
fn leaf_text_with_backslash() {
    let nve: NameValueExpr = parse_quote!(text = "\\n");
    assert_eq!(nve.path, "text");
}

#[test]
fn leaf_pattern_complex_regex() {
    let nve: NameValueExpr = parse_quote!(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*");
    assert_eq!(nve.path, "pattern");
}

#[test]
fn leaf_pattern_with_unicode_class() {
    let nve: NameValueExpr = parse_quote!(pattern = r"[\p{L}\p{N}]+");
    assert_eq!(nve.path, "pattern");
}

#[test]
fn struct_with_many_fields() {
    let s: ItemStruct = parse_quote! {
        struct ManyFields {
            #[adze::leaf(text = "a")] _a: (),
            #[adze::leaf(text = "b")] _b: (),
            #[adze::leaf(text = "c")] _c: (),
            #[adze::leaf(text = "d")] _d: (),
            #[adze::leaf(text = "e")] _e: (),
            #[adze::leaf(text = "f")] _f: (),
            #[adze::leaf(text = "g")] _g: (),
            #[adze::leaf(text = "h")] _h: (),
            #[adze::leaf(text = "i")] _i: (),
            #[adze::leaf(text = "j")] _j: (),
        }
    };
    assert_eq!(s.fields.len(), 10);
    assert!(
        s.fields
            .iter()
            .all(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
    );
}

#[test]
fn enum_with_mixed_variant_styles() {
    let e: ItemEnum = parse_quote! {
        enum Mixed {
            #[adze::leaf(text = "x")]
            Unit,
            Tuple(#[adze::leaf(text = "y")] ()),
            Named {
                #[adze::leaf(text = "z")]
                _z: (),
            },
        }
    };
    assert_eq!(e.variants.len(), 3);
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn transform_closure_with_type_annotation() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    assert_eq!(params[1].path, "transform");
}

#[test]
fn wrap_deeply_nested_containers() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < Box < adze :: WithLeaf < String > > > >"
    );
}

// =============================================================================
// Section 12: Additional coverage (tests 87–90)
// =============================================================================

#[test]
fn multiple_nve_in_punctuated() {
    let params: Punctuated<NameValueExpr, Token![,]> =
        parse_quote!(text = "+", transform = |v| v.to_string());
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path, "text");
    assert_eq!(params[1].path, "transform");
}

#[test]
fn ftp_with_attributed_unit_type() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ";")]
        ()
    );
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(is_adze_attr(&ftp.field.attrs[0], "leaf"));
}

#[test]
fn extract_option_box_vec_deeply_nested() {
    let skip = default_skip_set();
    let ty: Type = parse_quote!(Option<Box<Vec<Number>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

#[test]
fn filter_triple_nested_containers() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc", "Rc"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}
