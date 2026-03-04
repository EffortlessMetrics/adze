//! Comprehensive tests for adze attribute parsing (v2).
//!
//! Covers all 12 adze attribute types: grammar, language, leaf, extra, prec_left,
//! prec_right, prec, skip, repeat, delimited, word, external.
//! Tests attribute argument parsing, validation rules, combining multiple
//! attributes, error cases, and proc_macro2/syn infrastructure.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn prec_value(attr: &Attribute) -> i32 {
    let expr: syn::Expr = attr.parse_args().unwrap();
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) => i.base10_parse().unwrap(),
        _ => panic!("Expected integer literal"),
    }
}

// ============================================================================
// 1. grammar attribute
// ============================================================================

#[test]
fn grammar_attr_recognized_on_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod my_grammar {}
    });
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
}

#[test]
fn grammar_attr_string_name_extracted() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang_v3")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my_lang_v3");
    } else {
        panic!("Expected string literal grammar name");
    }
}

#[test]
fn grammar_attr_preserves_module_ident() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        pub mod calculator {}
    });
    assert_eq!(m.ident.to_string(), "calculator");
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}

#[test]
fn grammar_module_items_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr { Lit(i32) }
            #[adze::extra]
            struct Ws {}
        }
    });
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

// ============================================================================
// 2. language attribute
// ============================================================================

#[test]
fn language_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root { expr: Box<Expr> }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn language_attr_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr { Num(i32) }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn language_attr_takes_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {}
    };
    let attr = s
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "language"))
        .unwrap();
    // language attribute has no arguments; parse_args should fail
    assert!(attr.parse_args::<syn::Expr>().is_err());
}

// ============================================================================
// 3. leaf attribute
// ============================================================================

#[test]
fn leaf_attr_with_text_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Sep {
            #[adze::leaf(text = "+")]
            _plus: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn leaf_attr_with_pattern_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "pattern");
}

#[test]
fn leaf_attr_with_pattern_and_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    let names: Vec<String> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn leaf_attr_text_value_extraction() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "::")]
            PathSep,
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "::");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn leaf_attr_on_unit_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    for variant in &e.variants {
        assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(matches!(variant.fields, Fields::Unit));
    }
}

#[test]
fn leaf_attr_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "9")]
        struct BigDigit;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn leaf_attr_on_tuple_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<f64>().unwrap())]
                f64
            ),
        }
    };
    let field = match &e.variants[0].fields {
        Fields::Unnamed(u) => &u.unnamed[0],
        _ => panic!("Expected unnamed fields"),
    };
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn leaf_pattern_with_complex_regex() {
    let s: ItemStruct = parse_quote! {
        pub struct FloatLit {
            #[adze::leaf(pattern = r"-?\d+(\.\d+)?([eE][+-]?\d+)?")]
            value: String,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"-?\d+(\.\d+)?([eE][+-]?\d+)?");
    } else {
        panic!("Expected string literal");
    }
}

// ============================================================================
// 4. extra attribute
// ============================================================================

#[test]
fn extra_attr_on_struct() {
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
fn extra_attr_takes_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Comment {}
    };
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
    assert!(attr.parse_args::<syn::Expr>().is_err());
}

#[test]
fn multiple_extra_structs_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
            #[adze::extra]
            struct Whitespace {}
            #[adze::extra]
            struct LineComment {}
        }
    });
    let (_, items) = m.content.unwrap();
    let extra_count = items
        .iter()
        .filter(|item| {
            if let syn::Item::Struct(s) = item {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else {
                false
            }
        })
        .count();
    assert_eq!(extra_count, 2);
}

// ============================================================================
// 5. prec_left attribute
// ============================================================================

#[test]
fn prec_left_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
}

#[test]
fn prec_left_value_extracted() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(42)]
            Sub(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(attr), 42);
}

// ============================================================================
// 6. prec_right attribute
// ============================================================================

#[test]
fn prec_right_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_right"))
    );
}

#[test]
fn prec_right_value_extracted() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(99)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    assert_eq!(prec_value(attr), 99);
}

// ============================================================================
// 7. prec attribute (no associativity)
// ============================================================================

#[test]
fn prec_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(3)]
            Cmp(Box<Expr>, Box<Expr>),
        }
    };
    assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec")));
}

#[test]
fn prec_value_extracted() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(7)]
            Eq(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    assert_eq!(prec_value(attr), 7);
}

#[test]
fn all_three_prec_kinds_coexist() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(5)]
            Eq(Box<Expr>, Box<Expr>),
            #[adze::prec_left(10)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(15)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let attrs: Vec<_> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert_eq!(attrs, vec!["prec", "prec_left", "prec_right"]);

    let values: Vec<i32> = e.variants.iter().map(|v| prec_value(&v.attrs[0])).collect();
    assert_eq!(values, vec![5, 10, 15]);
}

// ============================================================================
// 8. skip attribute
// ============================================================================

#[test]
fn skip_attr_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn skip_attr_bool_value_extracted() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(true)]
            flag: bool,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
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
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn skip_attr_with_default_expr() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(Default::default())]
            counter: usize,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    // Should parse as a method call expression
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert!(matches!(expr, syn::Expr::Call(_)));
}

// ============================================================================
// 9. repeat attribute
// ============================================================================

#[test]
fn repeat_attr_on_vec_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

#[test]
fn repeat_attr_non_empty_param() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "non_empty");
}

#[test]
fn repeat_attr_non_empty_false() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = false)]
            items: Vec<Number>,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &params[0].expr
    {
        assert!(!b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ============================================================================
// 10. delimited attribute
// ============================================================================

#[test]
fn delimited_attr_on_vec_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Args {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Arg>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

#[test]
fn delimited_attr_inner_leaf_parsed() {
    let s: ItemStruct = parse_quote! {
        pub struct Stmts {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            stmts: Vec<Stmt>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), ";");
    } else {
        panic!("Expected string literal");
    }
}

// ============================================================================
// 11. word attribute
// ============================================================================

#[test]
fn word_attr_on_struct() {
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
fn word_attr_takes_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {}
    };
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
    assert!(attr.parse_args::<syn::Expr>().is_err());
}

// ============================================================================
// 12. external attribute
// ============================================================================

#[test]
fn external_attr_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn external_attr_takes_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct Heredoc;
    };
    let attr = s
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "external"))
        .unwrap();
    assert!(attr.parse_args::<syn::Expr>().is_err());
}

// ============================================================================
// 13. Combining multiple attributes
// ============================================================================

#[test]
fn word_and_language_combined_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::language]
        pub struct Identifier {
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"word".to_string()));
    assert!(names.contains(&"language".to_string()));
}

#[test]
fn repeat_and_delimited_on_same_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"repeat".to_string()));
    assert!(names.contains(&"delimited".to_string()));
}

#[test]
fn leaf_and_skip_on_different_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(0)]
            counter: usize,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(adze_attr_names(&fields[0].attrs), vec!["leaf"]);
    assert_eq!(adze_attr_names(&fields[1].attrs), vec!["skip"]);
}

#[test]
fn extra_and_external_in_same_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
            #[adze::extra]
            struct Whitespace {}
            #[adze::external]
            struct IndentToken;
        }
    });
    let (_, items) = m.content.unwrap();
    let mut found_extra = false;
    let mut found_external = false;
    for item in &items {
        if let syn::Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                found_extra = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "external")) {
                found_external = true;
            }
        }
    }
    assert!(found_extra);
    assert!(found_external);
}

#[test]
fn mixed_adze_and_derive_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        #[derive(PartialEq)]
        pub struct Root {}
    };
    let adze_count = s
        .attrs
        .iter()
        .filter(|a| {
            a.path()
                .segments
                .iter()
                .next()
                .map(|seg| seg.ident == "adze")
                .unwrap_or(false)
        })
        .count();
    let derive_count = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(adze_count, 1);
    assert_eq!(derive_count, 2);
}

// ============================================================================
// 14. Error / edge cases in attribute arguments
// ============================================================================

#[test]
fn unknown_adze_attr_parses_but_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[adze::nonexistent_attr]
        pub struct Foo {}
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["nonexistent_attr"]);
    let known = [
        "grammar",
        "language",
        "leaf",
        "skip",
        "prec",
        "prec_left",
        "prec_right",
        "delimited",
        "repeat",
        "extra",
        "external",
        "word",
    ];
    assert!(!known.contains(&names[0].as_str()));
}

#[test]
fn non_adze_two_segment_path_not_misidentified() {
    let s: ItemStruct = parse_quote! {
        #[serde::rename]
        #[adze::language]
        struct S {}
    };
    assert_eq!(adze_attr_names(&s.attrs), vec!["language"]);
}

#[test]
fn doc_comment_not_counted_as_adze() {
    let s: ItemStruct = parse_quote! {
        /// A documented struct
        #[adze::language]
        struct Documented {}
    };
    assert_eq!(adze_attr_names(&s.attrs).len(), 1);
}

#[test]
fn prec_arg_must_be_parseable_as_expr() {
    // Verify that prec with a valid integer literal parses
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(0)]
            ZeroPrec(Box<Expr>, Box<Expr>),
        }
    };
    let attr = &e.variants[0].attrs[0];
    assert_eq!(prec_value(attr), 0);
}

// ============================================================================
// 15. Attribute placement: wrong item type still parses at syn level
// ============================================================================

#[test]
fn language_on_enum_variant_parses_syntactically() {
    // syn allows any attribute on any item; validation happens in macro expansion
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::language]
            Num(i32),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "language"))
    );
}

#[test]
fn prec_left_on_struct_parses_syntactically() {
    let s: ItemStruct = parse_quote! {
        #[adze::prec_left(1)]
        pub struct ShouldBeVariant {}
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

// ============================================================================
// 16. NameValueExpr parsing patterns
// ============================================================================

#[test]
fn name_value_expr_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nv.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "hello");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn name_value_expr_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+\.\d+");
    assert_eq!(nv.path.to_string(), "pattern");
}

#[test]
fn name_value_expr_bool() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
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
fn name_value_expr_closure() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse::<u64>().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

#[test]
fn name_value_expr_integer() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &nv.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
    } else {
        panic!("Expected integer literal");
    }
}

// ============================================================================
// 17. FieldThenParams parsing patterns
// ============================================================================

#[test]
fn field_then_params_bare_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "String");
}

#[test]
fn field_then_params_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(u32, non_empty = true);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "non_empty");
}

#[test]
fn field_then_params_unit_type_with_params() {
    let ftp: FieldThenParams = parse_quote!((), text = ",");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "text");
}

// ============================================================================
// 18. Type extraction and wrapping utilities
// ============================================================================

#[test]
fn extract_inner_vec_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_inner_skip_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_inner_mismatch_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
}

#[test]
fn filter_inner_unwraps_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_inner_no_match_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "Vec < String >");
}

#[test]
fn wrap_leaf_plain_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u64);
    assert_eq!(ts(&wrap_leaf_type(&ty, &skip)), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_leaf_vec_wraps_inner() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Vec < adze :: WithLeaf < String > >"
    );
}

// ============================================================================
// 19. Structural preservation
// ============================================================================

#[test]
fn attrs_preserve_struct_field_names() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            count: usize,
        }
    };
    let field_names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(field_names, vec!["name", "count"]);
}

#[test]
fn attrs_preserve_enum_variant_names() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(2)]
            Pow(Box<Expr>, Box<Expr>),
            Lit(i32),
        }
    };
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, vec!["Add", "Pow", "Lit"]);
}

#[test]
fn attr_order_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        #[adze::external]
        struct S {}
    };
    assert_eq!(
        adze_attr_names(&s.attrs),
        vec!["language", "word", "external"]
    );
}

// ============================================================================
// 20. TokenStream roundtrip
// ============================================================================

#[test]
fn quote_roundtrip_struct_preserves_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct RT { x: String }
    };
    let reparsed: ItemStruct = syn::parse2(quote! { #s }).unwrap();
    assert_eq!(reparsed.ident, "RT");
    assert!(is_adze_attr(&reparsed.attrs[0], "language"));
}

#[test]
fn quote_roundtrip_enum_preserves_variant_attrs() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            A(i32),
            B,
        }
    };
    let reparsed: ItemEnum = syn::parse2(quote! { #e }).unwrap();
    assert!(is_adze_attr(&reparsed.variants[0].attrs[0], "prec_left"));
    assert!(reparsed.variants[1].attrs.is_empty());
}

#[test]
fn quote_interpolation_preserves_attr() {
    let name = quote::format_ident!("MyStruct");
    let tokens = quote! { #[adze::language] pub struct #name {} };
    let s: ItemStruct = syn::parse2(tokens).unwrap();
    assert_eq!(s.ident, "MyStruct");
    assert!(is_adze_attr(&s.attrs[0], "language"));
}

// ============================================================================
// 21. Enum variant field kinds with attributes
// ============================================================================

#[test]
fn enum_mixed_variant_kinds_all_with_attrs() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            Number(
                #[adze::leaf(pattern = r"\d+")]
                String
            ),
            Complex {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// ============================================================================
// 22. All 12 known attribute names enumerated
// ============================================================================

#[test]
fn all_twelve_known_attribute_names() {
    let known = [
        "grammar",
        "language",
        "leaf",
        "skip",
        "prec",
        "prec_left",
        "prec_right",
        "delimited",
        "repeat",
        "extra",
        "external",
        "word",
    ];
    assert_eq!(known.len(), 12);
    let set: HashSet<_> = known.iter().collect();
    assert_eq!(set.len(), 12);
}

// ============================================================================
// 23. Complex grammar module structure
// ============================================================================

#[test]
fn full_grammar_module_with_all_attr_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("full_test")]
        mod grammar {
            use std::collections::HashMap;

            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_right(2)]
                Assign(Box<Expr>, #[adze::leaf(text = "=")] (), Box<Expr>),
                #[adze::prec(3)]
                Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    let (_, items) = m.content.unwrap();
    // use + enum + struct(word) + struct(extra) + struct(external) = 5
    assert_eq!(items.len(), 5);

    // Verify grammar name
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "full_test");
    } else {
        panic!("Expected string literal grammar name");
    }
}

#[test]
fn struct_with_multiple_leaf_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Assignment {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let leaf_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .count();
    assert_eq!(leaf_count, 3);
}

#[test]
fn grammar_with_use_items_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;
            use std::collections::HashMap;

            #[adze::language]
            pub struct Root {}
        }
    });
    let (_, items) = m.content.unwrap();
    let use_count = items
        .iter()
        .filter(|item| matches!(item, syn::Item::Use(_)))
        .count();
    assert_eq!(use_count, 2);
}
