#![allow(clippy::needless_range_loop)]

//! Property-based tests for attribute parsing in adze-macro.
//!
//! Covers grammar, language, leaf, word, skip, extra, prec/prec_left/prec_right
//! attribute parsing and invalid attribute rejection using proptest and regular tests.

use adze_common::NameValueExpr;
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token, parse_quote};

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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn extract_leaf_text(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path == "text").unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        s.value()
    } else {
        panic!("Expected string literal for text param");
    }
}

fn extract_leaf_pattern(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path == "pattern").unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        s.value()
    } else {
        panic!("Expected string literal for pattern param");
    }
}

fn extract_grammar_name(m: &ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if !is_adze_attr(a, "grammar") {
            return None;
        }
        let expr: syn::Expr = a.parse_args().ok()?;
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = expr
        {
            Some(s.value())
        } else {
            None
        }
    })
}

fn skip_expr_str(attr: &Attribute) -> String {
    attr.parse_args::<syn::Expr>()
        .expect("skip attribute should contain an expression")
        .to_token_stream()
        .to_string()
}

fn prec_value(attr: &Attribute) -> i32 {
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        i.base10_parse().unwrap()
    } else {
        panic!("Expected integer literal for prec param");
    }
}

// ── 1. Grammar attribute with varying names ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_attr_name_preserved(idx in 0usize..=4) {
        let names = ["arith", "json_parser", "sql", "my_lang", "x"];
        let gname = names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
            }
        });
        prop_assert!(is_adze_attr(&m.attrs[0], "grammar"));
        prop_assert_eq!(extract_grammar_name(&m).unwrap(), gname);
    }
}

// ── 2. Grammar module preserves item count ──────────────────────────────────

proptest! {
    #[test]
    fn grammar_module_item_count(idx in 0usize..=2) {
        // 0 => just language, 1 => language + extra, 2 => language + 2 extras
        let m = match idx {
            0 => parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        tok: String,
                    }
                }
            }),
            1 => parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        tok: String,
                    }
                    #[adze::extra]
                    struct Ws {
                        #[adze::leaf(pattern = r"\s")]
                        _ws: (),
                    }
                }
            }),
            _ => parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        tok: String,
                    }
                    #[adze::extra]
                    struct Ws {
                        #[adze::leaf(pattern = r"\s")]
                        _ws: (),
                    }
                    #[adze::extra]
                    struct Comment {
                        #[adze::leaf(pattern = r"//[^\n]*")]
                        _c: (),
                    }
                }
            }),
        };
        let expected = idx + 1;
        prop_assert_eq!(module_items(&m).len(), expected);
    }
}

// ── 3. Language attribute on struct ──────────────────────────────────────────

#[test]
fn language_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(s.ident, "Program");
}

// ── 4. Language attribute on enum ───────────────────────────────────────────

#[test]
fn language_attr_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(e.ident, "Expr");
}

// ── 5. Leaf text attribute parsed correctly ──────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_roundtrip(idx in 0usize..=5) {
        let texts = ["+", "-", "**", "->", "::", "=="];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_text(attr), t);
    }
}

// ── 6. Leaf pattern attribute parsed correctly ──────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_roundtrip(idx in 0usize..=4) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"[0-9a-f]+", r"."];
        let p = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #p)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), p);
    }
}

// ── 7. Leaf with transform has three params ─────────────────────────────────

#[test]
fn leaf_with_transform_param_count() {
    let s: ItemStruct = parse_quote! {
        pub struct S {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            num: i32,
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
    assert!(params.iter().any(|p| p.path == "pattern"));
    assert!(params.iter().any(|p| p.path == "transform"));
}

// ── 8. Leaf on unit enum variant ────────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_unit_variant_text(idx in 0usize..=3) {
        let keywords = ["if", "else", "while", "fn"];
        let kw = keywords[idx];
        let ident = syn::Ident::new(&format!("Kw{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum K {
                #[adze::leaf(text = #kw)]
                #ident
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[0].fields, Fields::Unit));
        let attr = e.variants[0].attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_text(attr), kw);
    }
}

// ── 9. Word attribute detected on struct ────────────────────────────────────

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
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"word".to_string()));
}

// ── 10. Word combined with leaf pattern ─────────────────────────────────────

proptest! {
    #[test]
    fn word_with_leaf_pattern(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"[a-z]+", r"\w+"];
        let p = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #p)]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), p);
    }
}

// ── 11. Skip attribute with boolean default ─────────────────────────────────

#[test]
fn skip_attr_bool_false() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
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
    assert_eq!(skip_expr_str(attr), "false");
}

// ── 12. Skip attribute with various expressions ─────────────────────────────

proptest! {
    #[test]
    fn skip_attr_various_defaults(idx in 0usize..=3) {
        let exprs = ["false", "true", "0", "42"];
        let e = exprs[idx];
        let expr: syn::Expr = syn::parse_str(e).unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr)]
                field: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), e);
    }
}

// ── 13. Extra attribute on struct ───────────────────────────────────────────

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

// ── 14. Extra attribute recognized in module ────────────────────────────────

proptest! {
    #[test]
    fn extra_types_in_module(idx in 0usize..=2) {
        let extra_patterns = [r"\s", r"\s+", r"//[^\n]*"];
        let p = extra_patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
                #[adze::extra]
                struct Extra0 {
                    #[adze::leaf(pattern = #p)]
                    _e: (),
                }
            }
        });
        let extras: Vec<_> = module_items(&m).iter().filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else {
                false
            }
        }).collect();
        prop_assert_eq!(extras.len(), 1);
    }
}

// ── 15. Prec attribute with varying levels ──────────────────────────────────

proptest! {
    #[test]
    fn prec_attr_value_preserved(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec(#lit)]
                Cmp(Box<Expr>, Box<Expr>)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter().find(|a| is_adze_attr(a, "prec")).unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 16. Prec_left attribute with varying levels ─────────────────────────────

proptest! {
    #[test]
    fn prec_left_attr_value_preserved(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter().find(|a| is_adze_attr(a, "prec_left")).unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 17. Prec_right attribute with varying levels ────────────────────────────

proptest! {
    #[test]
    fn prec_right_attr_value_preserved(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                Cons(Box<Expr>, Box<Expr>)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter().find(|a| is_adze_attr(a, "prec_right")).unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 18. Multiple prec kinds coexist in one enum ─────────────────────────────

#[test]
fn multiple_prec_kinds_in_enum() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(3)]
            Cmp(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(1)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };
    let names: Vec<String> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert!(names.contains(&"prec".to_string()));
    assert!(names.contains(&"prec_left".to_string()));
    assert!(names.contains(&"prec_right".to_string()));
}

// ── 19. External attribute detected ─────────────────────────────────────────

#[test]
fn external_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken {
            #[adze::leaf(pattern = r"\t+")]
            _indent: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

// ── 20. Delimited attribute detected on field ───────────────────────────────

#[test]
fn delimited_attr_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
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

// ── 21. Repeat attribute with non_empty ─────────────────────────────────────

#[test]
fn repeat_attr_non_empty_parsing() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "non_empty");
}

// ── 22. Grammar attribute requires string literal ───────────────────────────

#[test]
fn grammar_attr_non_string_fails_parse() {
    // A numeric grammar name still parses as a module, but extract_grammar_name returns None
    let m = parse_mod(quote::quote! {
        #[adze::grammar(42)]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert!(extract_grammar_name(&m).is_none());
}

// ── 23. Leaf attribute missing on plain field ───────────────────────────────

#[test]
fn plain_field_has_no_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        pub struct S {
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(adze_attr_names(&field.attrs).is_empty());
}

// ── 24. Invalid attribute path not recognized ───────────────────────────────

#[test]
fn non_adze_attr_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[serde::rename("x")]
        pub struct S {
            value: i32,
        }
    };
    assert!(adze_attr_names(&s.attrs).is_empty());
}

// ── 25. Multiple adze attributes on one item ────────────────────────────────

#[test]
fn multiple_adze_attrs_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::extra]
        pub struct Token {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"word".to_string()));
    assert!(names.contains(&"extra".to_string()));
}

// ── 26. Grammar with language enum in module ────────────────────────────────

#[test]
fn grammar_module_has_language_enum() {
    let m = parse_mod(quote::quote! {
        #[adze::grammar("calc")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "0")]
                Zero,
            }
        }
    });
    let has_lang = module_items(&m).iter().any(|i| {
        if let Item::Enum(e) = i {
            e.attrs.iter().any(|a| is_adze_attr(a, "language"))
        } else {
            false
        }
    });
    assert!(has_lang);
}

// ── 27. Leaf text on tuple variant field ────────────────────────────────────

#[test]
fn leaf_text_on_tuple_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>
            )
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        let attr = u.unnamed[1]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        assert_eq!(extract_leaf_text(attr), "+");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 28. Prec_left combined with leaf on variant ─────────────────────────────

proptest! {
    #[test]
    fn prec_left_with_leaf_field(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                Sub(
                    Box<Expr>,
                    #[adze::leaf(text = "-")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        let v = &e.variants[0];
        let attr = v.attrs.iter().find(|a| is_adze_attr(a, "prec_left")).unwrap();
        prop_assert_eq!(prec_value(attr), prec);
        if let Fields::Unnamed(ref u) = v.fields {
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 29. Skip and leaf coexist on different fields ───────────────────────────

#[test]
fn skip_and_leaf_on_different_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 30. Grammar attribute determinism ───────────────────────────────────────

#[test]
fn grammar_attr_parsing_deterministic() {
    let tokens = quote::quote! {
        #[adze::grammar("stable")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    };
    let m1 = parse_mod(tokens.clone());
    let m2 = parse_mod(tokens);
    assert_eq!(
        m1.to_token_stream().to_string(),
        m2.to_token_stream().to_string()
    );
}

// ── 31. Leaf pattern with special regex chars ───────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_special_regex(idx in 0usize..=4) {
        let patterns = [r"\d+\.\d+", r"[^\n]*", r"\t+", r"\r?\n", r"\b\w+\b"];
        let p = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #p)]
                tok: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), p);
    }
}

// ── 32. Extra with comment pattern in module ────────────────────────────────

#[test]
fn extra_comment_pattern_in_module() {
    let m = parse_mod(quote::quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _c: (),
            }
        }
    });
    let extra_count = module_items(&m)
        .iter()
        .filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else {
                false
            }
        })
        .count();
    assert_eq!(extra_count, 2);
}

// ── 33. Prec_right with leaf on variant ─────────────────────────────────────

proptest! {
    #[test]
    fn prec_right_with_leaf_field(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                Cons(
                    Box<Expr>,
                    #[adze::leaf(text = "::")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        let v = &e.variants[0];
        let attr = v.attrs.iter().find(|a| is_adze_attr(a, "prec_right")).unwrap();
        prop_assert_eq!(prec_value(attr), prec);
        if let Fields::Unnamed(ref u) = v.fields {
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 34. Word attribute inside grammar module ────────────────────────────────

#[test]
fn word_attr_inside_grammar_module() {
    let m = parse_mod(quote::quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                ident: Identifier,
            }
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let has_word = module_items(&m).iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    assert!(has_word);
}

// ── 35. All standard attributes recognized in a full grammar ────────────────

#[test]
fn full_grammar_all_attrs_recognized() {
    let m = parse_mod(quote::quote! {
        #[adze::grammar("full")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>
                ),
                #[adze::prec_right(2)]
                Cons(
                    Box<Expr>,
                    #[adze::leaf(text = "::")]
                    (),
                    Box<Expr>
                ),
                #[adze::prec(3)]
                Cmp(
                    Box<Expr>,
                    #[adze::leaf(text = "==")]
                    (),
                    Box<Expr>
                ),
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    // Grammar attr on module
    assert!(is_adze_attr(&m.attrs[0], "grammar"));
    assert_eq!(extract_grammar_name(&m).unwrap(), "full");

    // Language on enum
    let lang_enum = module_items(&m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                Some(e)
            } else {
                None
            }
        } else {
            None
        }
    });
    assert!(lang_enum.is_some());

    // prec_left, prec_right, prec on variants
    let e = lang_enum.unwrap();
    let variant_attrs: Vec<String> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert!(variant_attrs.contains(&"prec_left".to_string()));
    assert!(variant_attrs.contains(&"prec_right".to_string()));
    assert!(variant_attrs.contains(&"prec".to_string()));

    // extra and word on structs
    let struct_attrs: Vec<String> = module_items(&m)
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                Some(adze_attr_names(&s.attrs))
            } else {
                None
            }
        })
        .flatten()
        .collect();
    assert!(struct_attrs.contains(&"extra".to_string()));
    assert!(struct_attrs.contains(&"word".to_string()));
}
