#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for grammar extraction from annotated Rust types.
//!
//! Tests the full grammar extraction pipeline:
//! 1. Grammar extraction from annotated Rust types (modules parsed via syn)
//! 2. Attribute parsing variations (leaf, prec, skip, extra, word, external, etc.)
//! 3. Property tests for grammar output determinism
//! 4. Edge cases (empty enums, unit structs, multiple variants, nested containers)
//!
//! Since `adze-macro` is a proc-macro crate, we test through:
//! - Parsing annotated module structures with `syn`
//! - Testing `adze_common` helper functions used by expansion.rs
//! - Verifying structural invariants of grammar modules

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

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

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn find_struct<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn struct_field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect()
}

fn struct_field_type_strings(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn variant_names(e: &ItemEnum) -> Vec<String> {
    e.variants.iter().map(|v| v.ident.to_string()).collect()
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

fn find_language_type(m: &ItemMod) -> Option<String> {
    module_items(m).iter().find_map(|item| match item {
        Item::Enum(e) if e.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(e.ident.to_string())
        }
        Item::Struct(s) if s.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(s.ident.to_string())
        }
        _ => None,
    })
}

fn count_extras(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|item| match item {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
            _ => false,
        })
        .count()
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn extract_text_value(attr: &Attribute) -> String {
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

fn extract_pattern_value(attr: &Attribute) -> String {
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

// =====================================================================
// 1. Grammar extraction: struct language type with varying field counts
// =====================================================================

proptest! {
    #[test]
    fn grammar_struct_field_count_preserved(count in 1usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #name: String,
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #(#fields)*
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }
}

// =====================================================================
// 2. Grammar extraction: enum language type with varying variant counts
// =====================================================================

proptest! {
    #[test]
    fn grammar_enum_variant_count_preserved(count in 1usize..=6) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("v{i}");
                quote! {
                    #[adze::leaf(text = #text)]
                    #name,
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Tok {
                    #(#variants)*
                }
            }
        });
        let e = find_enum(&m, "Tok").unwrap();
        prop_assert_eq!(e.variants.len(), count);
    }
}

// =====================================================================
// 3. Grammar name extraction determinism
// =====================================================================

proptest! {
    #[test]
    fn grammar_name_extraction_deterministic(idx in 0usize..=5) {
        let names = ["alpha", "beta_lang", "calc", "my_parser", "json", "css"];
        let gname = names[idx];
        let build = || {
            parse_mod(quote! {
                #[adze::grammar(#gname)]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        v: String,
                    }
                }
            })
        };
        let a = extract_grammar_name(&build());
        let b = extract_grammar_name(&build());
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(a.as_deref(), Some(gname));
    }
}

// =====================================================================
// 4. Attribute parsing: leaf text variants
// =====================================================================

proptest! {
    #[test]
    fn leaf_text_roundtrip(idx in 0usize..=5) {
        let texts = ["+", "-", "**", "==", "!=", "::"];
        let text = texts[idx];
        let s: ItemStruct = parse_quote! {
            pub struct Tok {
                #[adze::leaf(text = #text)]
                _tok: (),
            }
        };
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_text_value(attr);
        prop_assert_eq!(extracted, text);
    }
}

// =====================================================================
// 5. Attribute parsing: leaf pattern variants
// =====================================================================

proptest! {
    #[test]
    fn leaf_pattern_roundtrip(idx in 0usize..=4) {
        let patterns = [r"\d+", r"[a-zA-Z_]\w*", r"\s+", r"0[xX][0-9a-fA-F]+", r"//[^\n]*"];
        let pat = patterns[idx];
        let s: ItemStruct = parse_quote! {
            pub struct Tok {
                #[adze::leaf(pattern = #pat)]
                tok: String,
            }
        };
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_pattern_value(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// =====================================================================
// 6. Attribute parsing: leaf with transform
// =====================================================================

#[test]
fn leaf_with_transform_has_three_params() {
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
    assert!(params.iter().any(|p| p.path == "pattern"));
    assert!(params.iter().any(|p| p.path == "transform"));
}

// =====================================================================
// 7. Grammar output determinism: same input produces same token stream
// =====================================================================

proptest! {
    #[test]
    fn grammar_module_output_determinism(idx in 0usize..=3) {
        let bodies: Vec<proc_macro2::TokenStream> = vec![
            quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            },
            quote! {
                #[adze::language]
                pub enum Expr {
                    #[adze::leaf(text = "+")]
                    Plus,
                    #[adze::leaf(text = "-")]
                    Minus,
                }
            },
            quote! {
                #[adze::language]
                pub struct Code {
                    items: Vec<Item>,
                }
                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            },
            quote! {
                #[adze::language]
                pub enum Expr {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        i32
                    ),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            },
        ];
        let body = &bodies[idx];
        let build = || {
            parse_mod(quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #body
                }
            })
            .to_token_stream()
            .to_string()
        };
        let a = build();
        let b = build();
        prop_assert_eq!(a, b);
    }
}

// =====================================================================
// 8. Edge case: single unit variant enum
// =====================================================================

#[test]
fn single_unit_variant_enum_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    let e = find_enum(&m, "Token").unwrap();
    assert_eq!(e.variants.len(), 1);
    assert!(matches!(e.variants[0].fields, Fields::Unit));
}

// =====================================================================
// 9. Edge case: struct with only skip fields (and one real field)
// =====================================================================

#[test]
fn struct_with_skip_and_leaf_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Node {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
                #[adze::skip(0)]
                counter: usize,
                #[adze::skip(false)]
                visited: bool,
            }
        }
    });
    let s = find_struct(&m, "Node").unwrap();
    assert_eq!(s.fields.len(), 3);
    let skip_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count();
    assert_eq!(skip_count, 2);
}

// =====================================================================
// 10. Edge case: multiple extra types in a grammar
// =====================================================================

#[test]
fn multiple_extras_counted_correctly() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
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
                _comment: (),
            }
        }
    });
    assert_eq!(count_extras(&m), 2);
}

// =====================================================================
// 11. Edge case: enum with mixed variant kinds
// =====================================================================

#[test]
fn enum_mixed_unit_tuple_struct_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "nil")]
                Nil,
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                Binary {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    _op: (),
                    right: Box<Expr>,
                },
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 3);
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// =====================================================================
// 12. Attribute: prec_left detected on variant
// =====================================================================

#[test]
fn prec_left_attr_on_variant_detected() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let add = &e.variants[1];
    assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

// =====================================================================
// 13. Attribute: prec_right detected on variant
// =====================================================================

#[test]
fn prec_right_attr_on_variant_detected() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_right(2)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
        }
    };
    let cons = &e.variants[1];
    assert!(cons.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
}

// =====================================================================
// 14. Attribute: prec (no assoc) detected on variant
// =====================================================================

#[test]
fn prec_attr_on_variant_detected() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec(3)]
            Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
        }
    };
    let cmp = &e.variants[1];
    assert!(cmp.attrs.iter().any(|a| is_adze_attr(a, "prec")));
}

// =====================================================================
// 15. Grammar extraction: word attribute detected
// =====================================================================

#[test]
fn word_attr_detected_on_struct() {
    let m = parse_mod(quote! {
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
    let id = find_struct(&m, "Identifier").unwrap();
    assert!(id.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// =====================================================================
// 16. Grammar extraction: external attribute detected
// =====================================================================

#[test]
fn external_attr_detected_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
            #[adze::external]
            struct IndentToken {
                #[adze::leaf(pattern = r"\t+")]
                _indent: (),
            }
        }
    });
    let indent = find_struct(&m, "IndentToken").unwrap();
    assert!(indent.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

// =====================================================================
// 17. wrap_leaf_type: expansion non-leaf set from gen_field
// =====================================================================

#[test]
fn wrap_leaf_with_full_expansion_non_leaf_set() {
    let non_leaf = skip(&["Spanned", "Box", "Option", "Vec"]);
    let cases = [
        ("i32", "adze :: WithLeaf < i32 >"),
        ("String", "adze :: WithLeaf < String >"),
        ("Option<i32>", "Option < adze :: WithLeaf < i32 > >"),
        ("Vec<String>", "Vec < adze :: WithLeaf < String > >"),
        ("Box<u64>", "Box < adze :: WithLeaf < u64 > >"),
        (
            "Option<Vec<i32>>",
            "Option < Vec < adze :: WithLeaf < i32 > > >",
        ),
        ("Spanned<i32>", "Spanned < adze :: WithLeaf < i32 > >"),
    ];
    for (input, expected) in &cases {
        let wrapped = wrap_leaf_type(&ty(input), &non_leaf);
        assert_eq!(ts(&wrapped), *expected, "wrap_leaf_type({input})");
    }
}

// =====================================================================
// 18. try_extract_inner_type: extraction through containers
// =====================================================================

#[test]
fn extract_inner_type_through_box() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<i32>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_inner_type_miss() {
    let (_, ok) = try_extract_inner_type(&ty("HashMap<String, i32>"), "Vec", &skip(&[]));
    assert!(!ok);
}

// =====================================================================
// 19. filter_inner_type: double unwrap
// =====================================================================

#[test]
fn filter_inner_double_unwrap() {
    let filtered = filter_inner_type(&ty("Box<Arc<String>>"), &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "String");
}

// =====================================================================
// 20. NameValueExpr parsing with various key names
// =====================================================================

proptest! {
    #[test]
    fn name_value_expr_key_preserved(idx in 0usize..=5) {
        let keys = ["text", "pattern", "transform", "non_empty", "precedence", "value"];
        let key = keys[idx];
        let ident = syn::Ident::new(key, proc_macro2::Span::call_site());
        let nv: NameValueExpr = parse_quote!(#ident = 42);
        prop_assert_eq!(nv.path.to_string(), key);
    }
}

// =====================================================================
// 21. FieldThenParams parsing: unit type
// =====================================================================

#[test]
fn field_then_params_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// =====================================================================
// 22. FieldThenParams parsing: with params
// =====================================================================

#[test]
fn field_then_params_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
}

// =====================================================================
// 23. Grammar module: inline content required
// =====================================================================

#[test]
fn grammar_module_must_have_inline_content() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(m.content.is_some());
}

// =====================================================================
// 24. Grammar module: module ident preserved
// =====================================================================

proptest! {
    #[test]
    fn module_ident_preserved(idx in 0usize..=4) {
        let mod_names = ["grammar", "parser", "ast", "syntax", "lang"];
        let mod_name = mod_names[idx];
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod #mod_ident {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        prop_assert_eq!(m.ident.to_string(), mod_name);
    }
}

// =====================================================================
// 25. Grammar extraction: language attr on enum detected
// =====================================================================

proptest! {
    #[test]
    fn language_attr_on_enum_detected(idx in 0usize..=4) {
        let type_names = ["Expr", "Value", "Token", "Node", "Statement"];
        let name = type_names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum #ident {
                    #[adze::leaf(text = "+")]
                    Plus,
                }
            }
        });
        let found = find_language_type(&m);
        prop_assert_eq!(found.as_deref(), Some(name));
    }
}

// =====================================================================
// 26. Grammar extraction: language attr on struct detected
// =====================================================================

proptest! {
    #[test]
    fn language_attr_on_struct_detected(idx in 0usize..=4) {
        let type_names = ["Root", "Program", "Ast", "Entry", "TopLevel"];
        let name = type_names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #ident {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
            }
        });
        let found = find_language_type(&m);
        prop_assert_eq!(found.as_deref(), Some(name));
    }
}

// =====================================================================
// 27. Grammar extraction: no language attr means none found
// =====================================================================

#[test]
fn no_language_attr_returns_none() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Root {
                v: String,
            }
        }
    });
    assert!(find_language_type(&m).is_none());
}

// =====================================================================
// 28. Struct: field ordering preserved
// =====================================================================

#[test]
fn struct_field_ordering_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Rec {
                #[adze::leaf(pattern = r"\w+")]
                alpha: String,
                #[adze::leaf(pattern = r"\d+")]
                beta: String,
                #[adze::leaf(pattern = r"\s+")]
                gamma: String,
            }
        }
    });
    let s = find_struct(&m, "Rec").unwrap();
    assert_eq!(struct_field_names(s), vec!["alpha", "beta", "gamma"]);
}

// =====================================================================
// 29. Enum: variant ordering preserved
// =====================================================================

#[test]
fn enum_variant_ordering_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Op {
                #[adze::leaf(text = "+")]
                Add,
                #[adze::leaf(text = "-")]
                Sub,
                #[adze::leaf(text = "*")]
                Mul,
                #[adze::leaf(text = "/")]
                Div,
            }
        }
    });
    let e = find_enum(&m, "Op").unwrap();
    assert_eq!(variant_names(e), vec!["Add", "Sub", "Mul", "Div"]);
}

// =====================================================================
// 30. Struct: Vec field type detected
// =====================================================================

#[test]
fn struct_vec_field_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct List {
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "List").unwrap();
    assert!(struct_field_type_strings(s)[0].contains("Vec"));
}

// =====================================================================
// 31. Struct: Option field type detected
// =====================================================================

#[test]
fn struct_option_field_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: Option<String>,
            }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert!(struct_field_type_strings(s)[0].contains("Option"));
}

// =====================================================================
// 32. Struct: Box<Self> recursive type
// =====================================================================

#[test]
fn struct_box_self_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Expr {
                child: Option<Box<Expr>>,
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    });
    let s = find_struct(&m, "Expr").unwrap();
    let types = struct_field_type_strings(s);
    assert!(types[0].contains("Box"));
}

// =====================================================================
// 33. Enum: tuple variant with multiple fields
// =====================================================================

#[test]
fn enum_tuple_variant_multiple_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    if let Fields::Unnamed(u) = &e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 3);
    } else {
        panic!("Expected unnamed fields");
    }
}

// =====================================================================
// 34. Enum: struct variant named fields extracted
// =====================================================================

#[test]
fn enum_struct_variant_field_names() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Binary {
                left: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                right: Box<Expr>,
            },
        }
    };
    if let Fields::Named(n) = &e.variants[0].fields {
        let names: Vec<_> = n
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        assert_eq!(names, vec!["left", "_op", "right"]);
    } else {
        panic!("Expected named fields");
    }
}

// =====================================================================
// 35. Attribute collection: all adze attrs on enum variant
// =====================================================================

#[test]
fn all_adze_attrs_on_variant_collected() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let attrs = adze_attr_names(&e.variants[0].attrs);
    assert_eq!(attrs, vec!["prec_left"]);
}

// =====================================================================
// 36. Grammar: multiple types coexist
// =====================================================================

proptest! {
    #[test]
    fn grammar_mixed_struct_enum_count(
        struct_extra in 0usize..=3,
        enum_extra in 0usize..=2
    ) {
        let structs: Vec<proc_macro2::TokenStream> = (0..struct_extra)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
                quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
                        v: String,
                    }
                }
            })
            .collect();
        let enums: Vec<proc_macro2::TokenStream> = (0..enum_extra)
            .map(|i| {
                let name = syn::Ident::new(&format!("Enum{i}"), proc_macro2::Span::call_site());
                quote! {
                    pub enum #name {
                        #[adze::leaf(text = "+")]
                        A,
                    }
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
                #(#structs)*
                #(#enums)*
            }
        });
        let struct_count = module_items(&m).iter().filter(|i| matches!(i, Item::Struct(_))).count();
        let enum_count = module_items(&m).iter().filter(|i| matches!(i, Item::Enum(_))).count();
        prop_assert_eq!(struct_count, 1 + struct_extra);
        prop_assert_eq!(enum_count, enum_extra);
    }
}

// =====================================================================
// 37. wrap_leaf_type: property — always produces WithLeaf for plain types
// =====================================================================

proptest! {
    #[test]
    fn wrap_leaf_plain_always_has_with_leaf(idx in 0usize..=4) {
        let types = ["i32", "u64", "String", "f32", "bool"];
        let wrapped = wrap_leaf_type(&ty(types[idx]), &skip(&[]));
        prop_assert!(ts(&wrapped).contains("WithLeaf"));
    }
}

// =====================================================================
// 38. wrap_leaf_type: property — skip container preserves outer name
// =====================================================================

proptest! {
    #[test]
    fn wrap_leaf_skip_preserves_outer(idx in 0usize..=3) {
        let containers = ["Vec<i32>", "Option<String>", "Box<u64>", "Vec<bool>"];
        let outers = ["Vec", "Option", "Box", "Vec"];
        let wrapped = wrap_leaf_type(&ty(containers[idx]), &skip(&[outers[idx]]));
        let s = ts(&wrapped);
        prop_assert!(s.starts_with(outers[idx]));
        prop_assert!(s.contains("WithLeaf"));
    }
}

// =====================================================================
// 39. try_extract_inner_type: property — extracted flag is correct
// =====================================================================

proptest! {
    #[test]
    fn extract_inner_type_flag_matches(idx in 0usize..=3) {
        let types = ["Vec<i32>", "Option<String>", "Box<u8>", "HashMap<String, i32>"];
        let targets = ["Vec", "Option", "Box", "Vec"];
        let expected_flags = [true, true, true, false];
        let (_, extracted) = try_extract_inner_type(&ty(types[idx]), targets[idx], &skip(&[]));
        prop_assert_eq!(extracted, expected_flags[idx]);
    }
}

// =====================================================================
// 40. Grammar extraction: extra attr counted with varying extra counts
// =====================================================================

proptest! {
    #[test]
    fn extra_count_matches_annotations(extra_count in 0usize..=3) {
        let extras: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                let pat = format!(r"\s{}", i);
                quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = #pat)]
                        _v: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
                #(#extras)*
            }
        });
        prop_assert_eq!(count_extras(&m), extra_count);
    }
}

// =====================================================================
// 41. Enum: many unit leaf variants
// =====================================================================

proptest! {
    #[test]
    fn many_unit_leaf_variants(count in 2usize..=8) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Tok{i}"), proc_macro2::Span::call_site());
                let text = format!("t{i}");
                quote! {
                    #[adze::leaf(text = #text)]
                    #name,
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Tokens {
                    #(#variants)*
                }
            }
        });
        let e = find_enum(&m, "Tokens").unwrap();
        prop_assert_eq!(e.variants.len(), count);
        for v in &e.variants {
            prop_assert!(matches!(v.fields, Fields::Unit));
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// =====================================================================
// 42. Struct: delimited repeat attr detected
// =====================================================================

#[test]
fn delimited_attr_detected_on_field() {
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

// =====================================================================
// 43. Struct: repeat non_empty attr detected
// =====================================================================

#[test]
fn repeat_non_empty_attr_detected_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// =====================================================================
// 44. Enum: precedence attrs with various levels
// =====================================================================

proptest! {
    #[test]
    fn prec_left_levels_vary(level in 0i32..=10) {
        let e: ItemEnum = syn::parse2(quote! {
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(#level)]
                Op(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }).unwrap();
        let op = &e.variants[1];
        prop_assert!(op.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// =====================================================================
// 45. Grammar extraction: grammar name with underscores
// =====================================================================

proptest! {
    #[test]
    fn grammar_name_with_underscores(idx in 0usize..=3) {
        let names = ["my_lang", "proto_buf", "json_v2", "html_parser"];
        let gname = names[idx];
        let m = parse_mod(quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// =====================================================================
// 46. Struct: use item in module preserved
// =====================================================================

#[test]
fn use_item_in_module_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Root {
                items: Vec<Spanned<Item>>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let has_use = module_items(&m).iter().any(|i| matches!(i, Item::Use(_)));
    assert!(has_use);
}

// =====================================================================
// 47. filter_inner_type: empty skip set is noop
// =====================================================================

#[test]
fn filter_inner_empty_skip_noop() {
    let orig = ty("Box<String>");
    let filtered = filter_inner_type(&orig, &skip(&[]));
    assert_eq!(ts(&filtered), ts(&orig));
}

// =====================================================================
// 48. filter_inner_type: non-path type unchanged
// =====================================================================

#[test]
fn filter_inner_non_path_unchanged() {
    let filtered = filter_inner_type(&ty("(i32, u32)"), &skip(&["Box"]));
    assert_eq!(ts(&filtered), "(i32 , u32)");
}

// =====================================================================
// 49. wrap_leaf_type: nested option + vec
// =====================================================================

#[test]
fn wrap_leaf_nested_option_vec() {
    let wrapped = wrap_leaf_type(&ty("Option<Vec<i32>>"), &skip(&["Option", "Vec"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < i32 > > >");
}

// =====================================================================
// 50. Enum: leaf text values on unit variants roundtrip
// =====================================================================

#[test]
fn enum_unit_leaf_text_roundtrip() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
            #[adze::leaf(text = "*")]
            Mul,
        }
    };
    let texts: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let attr = v.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            extract_text_value(attr)
        })
        .collect();
    assert_eq!(texts, vec!["+", "-", "*"]);
}

// =====================================================================
// 51. Struct: leaf pattern values on fields roundtrip
// =====================================================================

#[test]
fn struct_leaf_pattern_roundtrip() {
    let s: ItemStruct = parse_quote! {
        pub struct Tok {
            #[adze::leaf(pattern = r"\d+")]
            num: String,
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            ident: String,
        }
    };
    let patterns: Vec<String> = s
        .fields
        .iter()
        .map(|f| {
            let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            extract_pattern_value(attr)
        })
        .collect();
    assert_eq!(patterns, vec![r"\d+", r"[a-zA-Z_]\w*"]);
}

// =====================================================================
// 52. Grammar: nested inner module preserved
// =====================================================================

#[test]
fn nested_inner_module_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
            mod helpers {
                pub fn util() {}
            }
        }
    });
    let has_inner = module_items(&m)
        .iter()
        .any(|i| matches!(i, Item::Mod(im) if im.ident == "helpers"));
    assert!(has_inner);
}

// =====================================================================
// 53. try_extract_inner_type: non-path type returns unchanged
// =====================================================================

#[test]
fn extract_inner_non_path_type_unchanged() {
    let (_, ok) = try_extract_inner_type(&ty("&str"), "Option", &skip(&[]));
    assert!(!ok);
}

// =====================================================================
// 54. wrap_leaf_type: array type gets wrapped
// =====================================================================

#[test]
fn wrap_leaf_array_type() {
    let wrapped = wrap_leaf_type(&ty("[u8; 4]"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

// =====================================================================
// 55. Struct: visibility preserved on language struct
// =====================================================================

#[test]
fn struct_visibility_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

// =====================================================================
// 56. Enum: visibility preserved on language enum
// =====================================================================

#[test]
fn enum_visibility_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert!(matches!(e.vis, syn::Visibility::Public(_)));
}

// =====================================================================
// 57. Enum: multiple prec attrs on different variants
// =====================================================================

#[test]
fn enum_multiple_prec_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
        }
    };
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
            .any(|a| is_adze_attr(a, "prec_left"))
    );
    assert!(
        e.variants[3]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_right"))
    );
}

// =====================================================================
// 58. Grammar extraction: struct with many field types
// =====================================================================

#[test]
fn struct_with_many_field_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Node {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                num: i32,
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                child: Option<Box<Node>>,
                items: Vec<Helper>,
                #[adze::skip(true)]
                flag: bool,
            }
            pub struct Helper {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "Node").unwrap();
    assert_eq!(s.fields.len(), 5);
}

// =====================================================================
// 59. Grammar: complete arithmetic grammar structure
// =====================================================================

#[test]
fn complete_arithmetic_grammar_structure() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(1)]
                Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Div(Box<Expr>, #[adze::leaf(text = "/")] (), Box<Expr>),
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m).as_deref(), Some("arithmetic"));
    assert_eq!(find_language_type(&m).as_deref(), Some("Expr"));
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 5);
    assert_eq!(count_extras(&m), 1);
}

// =====================================================================
// 60. Grammar: struct language with Vec and delimited
// =====================================================================

#[test]
fn struct_language_with_vec_and_delimited() {
    let m = parse_mod(quote! {
        #[adze::grammar("list_lang")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                #[adze::repeat(non_empty = true)]
                items: Vec<Number>,
            }
            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }
        }
    });
    assert_eq!(extract_grammar_name(&m).as_deref(), Some("list_lang"));
    assert_eq!(find_language_type(&m).as_deref(), Some("Program"));
    let s = find_struct(&m, "Program").unwrap();
    let field = s.fields.iter().next().unwrap();
    let attr_names = adze_attr_names(&field.attrs);
    assert!(attr_names.contains(&"delimited".to_string()));
    assert!(attr_names.contains(&"repeat".to_string()));
}

// =====================================================================
// 61. wrap_leaf_type: double-nested container
// =====================================================================

#[test]
fn wrap_leaf_double_nested() {
    let non_leaf = skip(&["Box", "Option"]);
    let wrapped = wrap_leaf_type(&ty("Box<Option<i32>>"), &non_leaf);
    assert_eq!(ts(&wrapped), "Box < Option < adze :: WithLeaf < i32 > > >");
}

// =====================================================================
// 62. try_extract_inner_type: skip chain through Box then Vec
// =====================================================================

#[test]
fn extract_inner_skip_chain_box_vec() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<String>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

// =====================================================================
// 63. Grammar: enum with only one tuple variant (leaf)
// =====================================================================

#[test]
fn enum_single_leaf_tuple_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Num {
                Value(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    u64
                ),
            }
        }
    });
    let e = find_enum(&m, "Num").unwrap();
    assert_eq!(e.variants.len(), 1);
    if let Fields::Unnamed(u) = &e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 1);
    } else {
        panic!("Expected unnamed fields");
    }
}

// =====================================================================
// 64. NameValueExpr: string literal value
// =====================================================================

#[test]
fn name_value_expr_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "+");
    } else {
        panic!("Expected string literal");
    }
}

// =====================================================================
// 65. FieldThenParams: complex field type
// =====================================================================

#[test]
fn field_then_params_complex_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<i32>);
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "Vec < i32 >");
}

// =====================================================================
// 66. Grammar module: token stream roundtrip is stable
// =====================================================================

#[test]
fn grammar_token_stream_stable_across_calls() {
    let build = || {
        parse_mod(quote! {
            #[adze::grammar("stable")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        i32
                    ),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        })
        .to_token_stream()
        .to_string()
    };
    let a = build();
    let b = build();
    let c = build();
    assert_eq!(a, b);
    assert_eq!(b, c);
}
