//! Comprehensive attribute validation and error handling tests for adze-macro.
//!
//! Covers:
//! - Valid attribute combinations
//! - Invalid attribute parameters (error cases via expand_grammar)
//! - Edge cases in attribute parsing
//! - Missing required fields
//! - Conflicting attributes
//! - Type manipulation in macro context
//! - syn parsing of macro inputs

use std::collections::HashSet;

use adze_common::{FieldThenParams, NameValueExpr, filter_inner_type, wrap_leaf_type};
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

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Valid attribute combinations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn valid_language_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(s.fields.iter().count(), 1);
}

#[test]
fn valid_language_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(e.variants.len(), 1);
}

#[test]
fn valid_extra_plus_leaf_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn valid_word_plus_leaf_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"word".to_string()));
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn valid_repeat_plus_delimited_on_field() {
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
fn valid_prec_left_with_leaf_text_in_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")] (),
                Box<Expr>,
            ),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
    // The middle field should have leaf
    if let Fields::Unnamed(ref fields) = e.variants[0].fields {
        assert!(
            fields.unnamed[1]
                .attrs
                .iter()
                .any(|a| is_adze_attr(a, "leaf"))
        );
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn valid_external_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn valid_all_three_prec_variants_coexist() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(1)]
            Cmp(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, Box<Expr>),
        }
    };
    assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec")));
    assert!(
        e.variants[1]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
    assert!(
        e.variants[2]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_right"))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Invalid attribute parameters (error cases)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn error_leaf_params_missing_equals_sign() {
    // Parsing `key value` (without `=`) should fail as NameValueExpr
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(pattern r"\d+")]
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
    let result = attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated);
    assert!(result.is_err(), "Parsing leaf without `=` should fail");
}

#[test]
fn error_repeat_unknown_parameter_name() {
    // `non_empty` is the known param; parsing succeeds but we can validate the name
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(unknown_param = true)]
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
    let known_params = ["non_empty"];
    let has_unknown = params
        .iter()
        .any(|p| !known_params.contains(&p.path.to_string().as_str()));
    assert!(has_unknown, "Should detect unknown parameter");
}

#[test]
fn error_leaf_no_text_or_pattern() {
    // A leaf with no text or pattern — parsing succeeds but has neither required param
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(transform = |v| v.to_string())]
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    let has_text = params.iter().any(|p| p.path == "text");
    let has_pattern = params.iter().any(|p| p.path == "pattern");
    assert!(
        !has_text && !has_pattern,
        "Leaf should have either text or pattern"
    );
}

#[test]
fn error_prec_with_non_integer_arg() {
    // prec with a string instead of integer
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec("high")]
            A(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    // Parsing succeeds syntactically, but the value is a string, not an integer
    let is_int = matches!(
        &expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(_),
            ..
        })
    );
    assert!(
        !is_int,
        "prec with string arg should not be an integer literal"
    );
}

#[test]
fn error_skip_with_no_args() {
    // skip without a default value expression
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip]
            visited: bool,
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
    let result = attr.parse_args::<syn::Expr>();
    assert!(
        result.is_err(),
        "skip without arguments should fail to parse args"
    );
}

#[test]
fn error_grammar_attr_with_empty_string_name() {
    // An empty string is a valid string literal but a questionable grammar name
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
        assert!(s.value().is_empty(), "Grammar name is empty string");
    } else {
        panic!("Expected string literal");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Edge cases in attribute parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn leaf_text_with_empty_string() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "")]
            Empty,
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert!(s.value().is_empty());
    } else {
        panic!("Expected empty string literal");
    }
}

#[test]
fn leaf_text_with_special_characters() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "=>")]
            Arrow,
            #[adze::leaf(text = "...")]
            Ellipsis,
            #[adze::leaf(text = "::")]
            DoubleColon,
        }
    };
    let texts: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let attr = v.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            let params = attr
                .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
                .unwrap();
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &params[0].expr
            {
                s.value()
            } else {
                panic!("Expected string literal");
            }
        })
        .collect();
    assert_eq!(texts, vec!["=>", "...", "::"]);
}

#[test]
fn leaf_pattern_with_complex_regex() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r#"[a-zA-Z_][a-zA-Z0-9_]*"#)]
            ident: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params.len(), 1);
}

#[test]
fn prec_with_zero_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(0)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn prec_with_large_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(999)]
            Op(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 999);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn grammar_name_with_hyphens() {
    let m = parse_mod(quote! {
        #[adze::grammar("my-grammar-name")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my-grammar-name");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn grammar_name_with_underscores() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_grammar_name")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my_grammar_name");
    } else {
        panic!("Expected string literal");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Missing required fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_module_without_grammar_attr_has_no_grammar_name() {
    let m = parse_mod(quote! {
        mod grammar {
            pub struct Root {}
        }
    });
    let has_grammar = m.attrs.iter().any(|a| is_adze_attr(a, "grammar"));
    assert!(
        !has_grammar,
        "Module without grammar attr should not have grammar"
    );
}

#[test]
fn module_without_language_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Foo {
                value: i32,
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let has_language = items.iter().any(|item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
        Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
        _ => false,
    });
    assert!(!has_language, "Module has no language type");
}

#[test]
fn leaf_field_with_only_transform_missing_pattern_or_text() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    let param_names: Vec<String> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(!param_names.contains(&"text".to_string()));
    assert!(!param_names.contains(&"pattern".to_string()));
    assert!(param_names.contains(&"transform".to_string()));
}

#[test]
fn variant_without_any_attrs() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(i32),
            Name(String),
        }
    };
    for variant in &e.variants {
        assert!(adze_attr_names(&variant.attrs).is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Conflicting attributes detection
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn detect_both_text_and_pattern_on_leaf() {
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(text = "+", pattern = r"\+")]
            op: String,
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    let has_text = params.iter().any(|p| p.path == "text");
    let has_pattern = params.iter().any(|p| p.path == "pattern");
    assert!(
        has_text && has_pattern,
        "Both text and pattern are present — potential conflict"
    );
}

#[test]
fn detect_multiple_prec_on_same_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            #[adze::prec_right(2)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let names = adze_attr_names(&e.variants[0].attrs);
    assert_eq!(names.len(), 2, "Multiple precedence attrs on same variant");
    assert!(names.contains(&"prec_left".to_string()));
    assert!(names.contains(&"prec_right".to_string()));
}

#[test]
fn detect_language_and_extra_on_same_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        pub struct Root {
            value: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"language".to_string()));
    assert!(names.contains(&"extra".to_string()));
    assert_eq!(
        names.len(),
        2,
        "Both language and extra on same struct — potential conflict"
    );
}

#[test]
fn detect_language_and_external_on_same_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::external]
        pub struct Root;
    };
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"language".to_string()));
    assert!(names.contains(&"external".to_string()));
    assert_eq!(
        names.len(),
        2,
        "Both language and external on same struct — potential conflict"
    );
}

#[test]
fn detect_skip_and_leaf_on_same_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(0)]
            #[adze::leaf(pattern = r"\d+")]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert!(names.contains(&"skip".to_string()));
    assert!(names.contains(&"leaf".to_string()));
    assert_eq!(
        names.len(),
        2,
        "Both skip and leaf on same field — conflicting"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Type manipulation in macro context
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_leaf_type_simple() {
    let t = ty("i32");
    let wrapped = wrap_leaf_type(&t, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_leaf_type_skips_vec() {
    let t = ty("Vec<String>");
    let wrapped = wrap_leaf_type(&t, &skip_set(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_leaf_type_skips_option_and_vec() {
    let t = ty("Option<Vec<i32>>");
    let wrapped = wrap_leaf_type(&t, &skip_set(&["Option", "Vec"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wrap_leaf_type_box_not_skipped() {
    let t = ty("Box<Expr>");
    let wrapped = wrap_leaf_type(&t, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Box < Expr > >");
}

#[test]
fn wrap_leaf_type_box_skipped() {
    let t = ty("Box<Expr>");
    let wrapped = wrap_leaf_type(&t, &skip_set(&["Box"]));
    assert_eq!(ts(&wrapped), "Box < adze :: WithLeaf < Expr > >");
}

#[test]
fn filter_inner_type_removes_box() {
    let t = ty("Box<String>");
    let filtered = filter_inner_type(&t, &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_inner_type_removes_nested_containers() {
    let t = ty("Box<Arc<String>>");
    let filtered = filter_inner_type(&t, &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_inner_type_preserves_non_skip_types() {
    let t = ty("Vec<String>");
    let filtered = filter_inner_type(&t, &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < String >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. syn parsing of macro inputs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_name_value_expr_simple() {
    let nve: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nve.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nve.expr
    {
        assert_eq!(s.value(), "hello");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn parse_name_value_expr_with_closure() {
    let nve: NameValueExpr = parse_quote!(transform = |v| v.parse::<i32>().unwrap());
    assert_eq!(nve.path.to_string(), "transform");
    assert!(matches!(&nve.expr, syn::Expr::Closure(_)));
}

#[test]
fn parse_name_value_expr_with_integer() {
    let nve: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(nve.path.to_string(), "precedence");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &nve.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn parse_name_value_expr_with_bool() {
    let nve: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nve.path.to_string(), "non_empty");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &nve.expr
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn parse_field_then_params_type_only() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "String");
}

#[test]
fn parse_field_then_params_with_params() {
    let ftp: FieldThenParams = parse_quote!(i32, min = 0, max = 100);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn parse_field_then_params_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
}

#[test]
fn parse_multiple_leaf_params_as_punctuated() {
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params[1].path.to_string(), "transform");
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Module structure validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn module_preserves_use_statements() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Root {
                items: Vec<Spanned<Item>>,
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let has_use = items.iter().any(|i| matches!(i, Item::Use(_)));
    assert!(has_use, "Module should preserve use statements");
}

#[test]
fn module_with_multiple_structs_and_enums() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Number {
                value: i32,
            }

            #[adze::extra]
            struct Whitespace {
                _ws: (),
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let struct_count = items
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    let enum_count = items.iter().filter(|i| matches!(i, Item::Enum(_))).count();
    assert_eq!(struct_count, 2);
    assert_eq!(enum_count, 1);
}

#[test]
fn module_visibility_is_preserved() {
    let m: ItemMod = parse_quote! {
        pub(crate) mod grammar {}
    };
    assert!(matches!(m.vis, syn::Visibility::Restricted(_)));
}

#[test]
fn empty_module_body_parses() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {}
    });
    let items = &m.content.as_ref().unwrap().1;
    assert!(items.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Enum variant field types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_variant_with_named_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<Expr>,
            }
        }
    };
    if let Fields::Named(ref fields) = e.variants[0].fields {
        assert_eq!(fields.named.len(), 2);
        let field_names: Vec<_> = fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(field_names, vec!["_bang", "value"]);
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn enum_variant_with_box_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg(Box<Expr>),
        }
    };
    if let Fields::Unnamed(ref fields) = e.variants[0].fields {
        let ty_str = fields.unnamed[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Box"), "Field type should be Box<Expr>");
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn enum_unit_variant_with_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
        assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Attribute counting and ordering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn non_adze_attrs_are_not_counted() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[cfg(test)]
        #[adze::language]
        #[allow(unused)]
        pub struct Root {}
    };
    let adze_count = adze_attr_names(&s.attrs).len();
    assert_eq!(adze_count, 1);
    assert_eq!(s.attrs.len(), 4);
}

#[test]
fn attr_order_preserved_in_parsing() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::language]
        #[adze::extra]
        pub struct Root {}
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["word", "language", "extra"]);
}

#[test]
fn field_attrs_mixed_with_standard() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[allow(unused)]
            #[adze::leaf(pattern = r"\w+")]
            #[cfg(test)]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.attrs.len(), 3);
    assert_eq!(adze_attr_names(&field.attrs), vec!["leaf"]);
}

#[test]
fn twelve_known_attrs_complete_set() {
    // Validate that all 12 known adze attributes can be recognized
    let known: Vec<&str> = vec![
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

    // Each can appear as a valid adze:: path
    for name in &known {
        let tokens: TokenStream = format!("#[adze::{name}] struct S;").parse().unwrap();
        let s: ItemStruct = syn::parse2(tokens).unwrap();
        assert!(
            s.attrs.iter().any(|a| is_adze_attr(a, name)),
            "Attribute adze::{name} should be recognized"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. try_extract_inner_type tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_inner_vec_of_string() {
    use adze_common::try_extract_inner_type;
    let t = ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_inner_option_of_i32() {
    use adze_common::try_extract_inner_type;
    let t = ty("Option<i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_inner_not_matching_returns_false() {
    use adze_common::try_extract_inner_type;
    let t = ty("HashMap<String, i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_inner_through_box_skip() {
    use adze_common::try_extract_inner_type;
    let t = ty("Box<Vec<u32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip_set(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_inner_non_path_type_unchanged() {
    use adze_common::try_extract_inner_type;
    let t: Type = parse_quote!(&str);
    let (_inner, ok) = try_extract_inner_type(&t, "Vec", &skip_set(&[]));
    assert!(!ok);
}

#[test]
fn extract_inner_plain_type_not_extracted() {
    use adze_common::try_extract_inner_type;
    let t = ty("String");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "String");
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Additional wrap_leaf_type edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_leaf_type_nested_option_vec() {
    let t = ty("Option<Vec<u64>>");
    let wrapped = wrap_leaf_type(&t, &skip_set(&["Option", "Vec"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < u64 > > >");
}

#[test]
fn wrap_leaf_type_plain_string() {
    let t = ty("String");
    let wrapped = wrap_leaf_type(&t, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_leaf_type_tuple_type() {
    let t: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&t, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn wrap_leaf_type_reference_type() {
    let t: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&t, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Additional filter_inner_type edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_inner_type_triple_nested() {
    let t = ty("Box<Arc<Rc<String>>>");
    let filtered = filter_inner_type(&t, &skip_set(&["Box", "Arc", "Rc"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_inner_type_empty_skip_preserves() {
    let t = ty("Box<String>");
    let filtered = filter_inner_type(&t, &skip_set(&[]));
    assert_eq!(ts(&filtered), "Box < String >");
}

#[test]
fn filter_inner_type_non_path_unchanged() {
    let t: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&t, &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "& str");
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Struct with multiple named fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_multiple_named_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct BinaryOp {
            left: Box<Expr>,
            #[adze::leaf(text = "+")]
            _op: (),
            right: Box<Expr>,
        }
    };
    assert_eq!(s.fields.iter().count(), 3);
    let field_names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(field_names, vec!["left", "_op", "right"]);
}

#[test]
fn struct_all_fields_have_leaf() {
    let s: ItemStruct = parse_quote! {
        pub struct Pair {
            #[adze::leaf(pattern = r"\d+")]
            a: String,
            #[adze::leaf(text = ",")]
            _sep: (),
            #[adze::leaf(pattern = r"\d+")]
            b: String,
        }
    };
    for field in &s.fields {
        assert!(
            field.attrs.iter().any(|a| is_adze_attr(a, "leaf")),
            "Each field should have leaf attr"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Enum variant type diversity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_mixed_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            #[adze::leaf(text = "nil")]
            Nil,
            Value(i32),
            Pair {
                left: Box<Node>,
                right: Box<Node>,
            },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn enum_single_variant() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert_eq!(e.variants.len(), 1);
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn enum_many_unit_variants() {
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
        }
    };
    assert_eq!(e.variants.len(), 5);
    for v in &e.variants {
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Optional and Vec field patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_optional_leaf_field() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeNum {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: Option<i32>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let ty_str = field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Option"));
}

#[test]
fn struct_vec_field_without_repeat() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(adze_attr_names(&field.attrs).is_empty());
    let ty_str = field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Vec"));
}

#[test]
fn enum_unnamed_optional_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            MaybeNeg(
                #[adze::leaf(text = "-")] Option<()>,
                Box<Expr>,
            ),
        }
    };
    if let Fields::Unnamed(ref fields) = e.variants[0].fields {
        let ty_str = fields.unnamed[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Option"));
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn enum_unnamed_vec_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Numbers(
                #[adze::repeat(non_empty = true)]
                Vec<Number>,
            ),
        }
    };
    if let Fields::Unnamed(ref fields) = e.variants[0].fields {
        let ty_str = fields.unnamed[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Vec"));
        assert!(
            fields.unnamed[0]
                .attrs
                .iter()
                .any(|a| is_adze_attr(a, "repeat"))
        );
    } else {
        panic!("Expected unnamed fields");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Grammar name edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_name_with_dots() {
    let m = parse_mod(quote! {
        #[adze::grammar("my.grammar.v2")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my.grammar.v2");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn grammar_name_with_numbers() {
    let m = parse_mod(quote! {
        #[adze::grammar("grammar123")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "grammar123");
    } else {
        panic!("Expected string literal");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Skip attribute value types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn skip_with_bool_value() {
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
    assert!(matches!(
        &expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(_),
            ..
        })
    ));
}

#[test]
fn skip_with_integer_value() {
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
    assert!(matches!(
        &expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(_),
            ..
        })
    ));
}

#[test]
fn skip_with_string_value() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip("default")]
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
    assert!(matches!(
        &expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. Repeat attribute variations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn repeat_with_non_empty_false() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = false)]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params[0].path.to_string(), "non_empty");
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

#[test]
fn repeat_without_params() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let result = attr.parse_args::<syn::Expr>();
    // repeat without params has no args to parse
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. Delimited attribute edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn delimited_parses_as_field_then_params() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = attr.parse_args().unwrap();
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert!(ftp.field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 21. Module items counting and types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn module_counts_language_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Helper {
                value: i32,
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let language_count = items
        .iter()
        .filter(|item| match item {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
            Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
            _ => false,
        })
        .count();
    assert_eq!(language_count, 1);
}

#[test]
fn module_with_private_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            struct PrivateHelper {
                value: i32,
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let private_count = items
        .iter()
        .filter(|item| {
            if let Item::Struct(s) = item {
                matches!(s.vis, syn::Visibility::Inherited)
            } else {
                false
            }
        })
        .count();
    assert_eq!(private_count, 1);
}

#[test]
fn module_with_const_item() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            const MAX: usize = 100;

            #[adze::language]
            pub struct Root {
                value: i32,
            }
        }
    });
    let items = &m.content.as_ref().unwrap().1;
    let const_count = items.iter().filter(|i| matches!(i, Item::Const(_))).count();
    assert_eq!(const_count, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 22. Leaf parameter ordering and combinations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn leaf_transform_before_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(transform = |v| v.parse().unwrap(), pattern = r"\d+")]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params[0].path.to_string(), "transform");
    assert_eq!(params[1].path.to_string(), "pattern");
}

#[test]
fn leaf_text_only_no_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn leaf_with_all_three_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap(), text = "42")]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 3);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
    assert!(names.contains(&"text".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// 23. FieldThenParams edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn field_then_params_with_attributed_field() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        ()
    );
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert!(!ftp.field.attrs.is_empty());
    assert!(ftp.field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn field_then_params_box_type() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Box < Expr >");
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_params_vec_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Number>);
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Vec < Number >");
    assert!(ftp.params.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 24. Visibility combinations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pub_crate_struct_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub(crate) struct Root {
            value: i32,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn private_struct_has_inherited_visibility() {
    let s: ItemStruct = parse_quote! {
        struct Helper {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

#[test]
fn pub_enum_visibility() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Lit(i32),
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Public(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 25. Precedence value edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn prec_left_with_negative_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(-1)]
            Sub(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    // Negative value parses as a unary neg expression
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert!(matches!(&expr, syn::Expr::Unary(_)));
}

#[test]
fn prec_values_differ_across_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
            #[adze::prec_left(3)]
            Pow(Box<Expr>, Box<Expr>),
        }
    };
    let values: Vec<i32> = e
        .variants
        .iter()
        .map(|v| {
            let attr = v
                .attrs
                .iter()
                .find(|a| is_adze_attr(a, "prec_left"))
                .unwrap();
            let expr: syn::Expr = attr.parse_args().unwrap();
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) = expr
            {
                i.base10_parse::<i32>().unwrap()
            } else {
                panic!("Expected int");
            }
        })
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}
