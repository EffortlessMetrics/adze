#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::language]` attribute handling in adze-macro.
//!
//! Uses proptest to verify language attribute parsing, language on enum,
//! language name parameter, language combined with variants, language expansion
//! output, language attribute determinism, language with various enum shapes,
//! and language error cases.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, parse_quote};

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

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
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

fn count_language_types(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|item| match item {
            Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
            _ => false,
        })
        .count()
}

// ── 1. Language attribute parsed on struct with varying names ────────────────

proptest! {
    #[test]
    fn language_parsed_on_struct_varying_names(idx in 0usize..=5) {
        let names = ["Root", "Program", "Ast", "Entry", "TopLevel", "Start"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(s.ident.to_string(), names[idx]);
    }
}

// ── 2. Language on enum with varying variant counts ─────────────────────────

proptest! {
    #[test]
    fn language_on_enum_varying_variant_count(count in 1usize..=6) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(text = "+")]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 3. Language name parameter detected in grammar module ───────────────────

proptest! {
    #[test]
    fn language_name_in_grammar_module(idx in 0usize..=4) {
        let grammar_names = ["arith", "json", "calc", "lang", "proto"];
        let gname = grammar_names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Root".to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
    }
}

// ── 4. Language combined with leaf variants on enum ──────────────────────────

proptest! {
    #[test]
    fn language_combined_with_leaf_variants(idx in 0usize..=4) {
        let texts = ["+", "-", "*", "/", "%"];
        let txt = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Op {
                #[adze::leaf(text = #txt)]
                This,
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let v = &e.variants[0];
        let names = adze_attr_names(&v.attrs);
        prop_assert!(names.contains(&"leaf".to_string()));
    }
}

// ── 5. Language expansion output preserves type ident ────────────────────────

proptest! {
    #[test]
    fn language_expansion_preserves_ident(idx in 0usize..=4) {
        let type_names = ["Expression", "Statement", "Value", "Token", "Node"];
        let ident = syn::Ident::new(type_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                f: String,
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), type_names[idx]);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 6. Language attribute determinism: parsing same input twice ──────────────

proptest! {
    #[test]
    fn language_attr_deterministic(idx in 0usize..=3) {
        let names = ["Foo", "Bar", "Baz", "Qux"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let tokens = quote::quote! {
            #[adze::language]
            pub struct #ident {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        };
        let s1: ItemStruct = syn::parse2(tokens.clone()).unwrap();
        let s2: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert_eq!(s1.to_token_stream().to_string(), s2.to_token_stream().to_string());
    }
}

// ── 7. Language with various enum shapes: unit variants ──────────────────────

proptest! {
    #[test]
    fn language_enum_unit_variants_shape(count in 1usize..=8) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("K{i}"), proc_macro2::Span::call_site());
                let text = format!("kw{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Keywords {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        for v in &e.variants {
            prop_assert!(matches!(v.fields, Fields::Unit));
        }
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 8. Language error: missing language attr in grammar module ───────────────

proptest! {
    #[test]
    fn language_error_missing_attr_in_module(idx in 0usize..=3) {
        let type_names = ["Foo", "Bar", "Baz", "Qux"];
        let ident = syn::Ident::new(type_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                pub struct #ident {
                    value: i32,
                }
            }
        });
        // No language type should be found since we omitted #[adze::language]
        prop_assert_eq!(find_language_type(&m), None);
        prop_assert_eq!(count_language_types(&m), 0);
    }
}

// ── 9. Language attr is always path-style (no args) ─────────────────────────

proptest! {
    #[test]
    fn language_attr_is_path_style_no_args(idx in 0usize..=3) {
        let names = ["A", "B", "C", "D"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                f: String,
            }
        }).unwrap();
        let lang = s.attrs.iter().find(|a| is_adze_attr(a, "language")).unwrap();
        prop_assert!(matches!(lang.meta, syn::Meta::Path(_)));
    }
}

// ── 10. Language on enum with named-field variants ──────────────────────────

proptest! {
    #[test]
    fn language_enum_with_named_field_variants(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: String }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Named { #(#fields),* },
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let v = &e.variants[0];
        prop_assert!(matches!(v.fields, Fields::Named(_)));
        if let Fields::Named(ref n) = v.fields {
            prop_assert_eq!(n.named.len(), field_count);
        }
    }
}

// ── 11. Language on enum with tuple variants ────────────────────────────────

proptest! {
    #[test]
    fn language_enum_with_tuple_variants(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| quote::quote! { String })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Tuple(#(#fields),*),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let v = &e.variants[0];
        prop_assert!(matches!(v.fields, Fields::Unnamed(_)));
        if let Fields::Unnamed(ref u) = v.fields {
            prop_assert_eq!(u.unnamed.len(), field_count);
        }
    }
}

// ── 12. Language with prec_left variants ────────────────────────────────────

proptest! {
    #[test]
    fn language_with_prec_left_variants(prec in 1i32..=10) {
        let prec_lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#prec_lit)]
                Add(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let add = e.variants.iter().find(|v| v.ident == "Add").unwrap();
        prop_assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// ── 13. Language with prec_right variants ───────────────────────────────────

proptest! {
    #[test]
    fn language_with_prec_right_variants(prec in 1i32..=10) {
        let prec_lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Num(i32),
                #[adze::prec_right(#prec_lit)]
                Cons(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let cons = e.variants.iter().find(|v| v.ident == "Cons").unwrap();
        prop_assert!(cons.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
    }
}

// ── 14. Language with prec (no assoc) variants ──────────────────────────────

proptest! {
    #[test]
    fn language_with_prec_no_assoc_variants(prec in 1i32..=10) {
        let prec_lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Num(i32),
                #[adze::prec(#prec_lit)]
                Cmp(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let cmp = e.variants.iter().find(|v| v.ident == "Cmp").unwrap();
        prop_assert!(cmp.attrs.iter().any(|a| is_adze_attr(a, "prec")));
    }
}

// ── 15. Language struct with varying field counts ────────────────────────────

proptest! {
    #[test]
    fn language_struct_varying_field_count(count in 1usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #name: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #(#fields),*
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            prop_assert_eq!(n.named.len(), count);
        } else {
            prop_assert!(false, "expected named fields");
        }
    }
}

// ── 16. Language on enum with mixed variant kinds ───────────────────────────

proptest! {
    #[test]
    fn language_enum_mixed_kinds(idx in 0usize..=2) {
        // Different orderings of unit/tuple/named variants
        let token_sets: Vec<proc_macro2::TokenStream> = match idx {
            0 => vec![
                quote::quote! { #[adze::leaf(text = "x")] Unit },
                quote::quote! { Tuple(String) },
            ],
            1 => vec![
                quote::quote! { Tuple(i32) },
                quote::quote! { Named { f: String } },
            ],
            _ => vec![
                quote::quote! { #[adze::leaf(text = "y")] Unit },
                quote::quote! { Named { a: i32 } },
                quote::quote! { Tuple(String) },
            ],
        };
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                #(#token_sets),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), token_sets.len());
    }
}

// ── 17. Language struct visibility varies ────────────────────────────────────

proptest! {
    #[test]
    fn language_struct_visibility_varies(idx in 0usize..=2) {
        let tokens = match idx {
            0 => quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    f: String,
                }
            },
            1 => quote::quote! {
                #[adze::language]
                pub(crate) struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    f: String,
                }
            },
            _ => quote::quote! {
                #[adze::language]
                struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    f: String,
                }
            },
        };
        let s: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        match idx {
            0 => prop_assert!(matches!(s.vis, syn::Visibility::Public(_))),
            1 => prop_assert!(matches!(s.vis, syn::Visibility::Restricted(_))),
            _ => prop_assert!(matches!(s.vis, syn::Visibility::Inherited)),
        }
    }
}

// ── 18. Language with derive attributes preserved ───────────────────────────

proptest! {
    #[test]
    fn language_with_derive_preserved(idx in 0usize..=3) {
        let derives = ["Debug", "Clone", "PartialEq", "Default"];
        let derive_ident = syn::Ident::new(derives[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(#derive_ident)]
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                f: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let derive_attr = s.attrs.iter().find(|a| a.path().is_ident("derive"));
        prop_assert!(derive_attr.is_some());
    }
}

// ── 19. Language is the only adze attr on a type ────────────────────────────

proptest! {
    #[test]
    fn language_only_adze_attr_on_type(idx in 0usize..=4) {
        let names = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                f: String,
            }
        }).unwrap();
        let type_adze_attrs = adze_attr_names(&s.attrs);
        prop_assert_eq!(type_adze_attrs, vec!["language".to_string()]);
    }
}

// ── 20. Language on enum with doc comments ──────────────────────────────────

proptest! {
    #[test]
    fn language_enum_with_doc_comment(idx in 0usize..=3) {
        let docs = ["A node.", "Main entry.", "Top-level.", "Root grammar."];
        let doc = docs[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[doc = #doc]
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let doc_attrs: Vec<_> = e.attrs.iter().filter(|a| a.path().is_ident("doc")).collect();
        prop_assert!(!doc_attrs.is_empty());
    }
}

// ── 21. Language enum coexists with extra types in module ───────────────────

proptest! {
    #[test]
    fn language_coexists_with_extras_in_module(extra_count in 1usize..=3) {
        let extras: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = r"\s")]
                        _ws: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extras)*
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Root".to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
    }
}

// ── 22. Language attr path always has two segments ──────────────────────────

proptest! {
    #[test]
    fn language_attr_two_path_segments(idx in 0usize..=3) {
        let names = ["X", "Y", "Z", "W"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum #ident {
                V(i32),
            }
        }).unwrap();
        let lang = e.attrs.iter().find(|a| is_adze_attr(a, "language")).unwrap();
        let segs: Vec<_> = lang.path().segments.iter().collect();
        prop_assert_eq!(segs.len(), 2);
        prop_assert_eq!(segs[0].ident.to_string(), "adze");
        prop_assert_eq!(segs[1].ident.to_string(), "language");
    }
}

// ── 23. Language on unit struct ─────────────────────────────────────────────

proptest! {
    #[test]
    fn language_on_unit_struct_various(idx in 0usize..=3) {
        let names = ["Empty", "Marker", "Sentinel", "Noop"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct #ident;
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert!(matches!(s.fields, Fields::Unit));
    }
}

// ── 24. Language on tuple struct ────────────────────────────────────────────

proptest! {
    #[test]
    fn language_on_tuple_struct_various(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| quote::quote! { String })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root(#(#fields),*);
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert!(matches!(s.fields, Fields::Unnamed(_)));
        if let Fields::Unnamed(ref u) = s.fields {
            prop_assert_eq!(u.unnamed.len(), field_count);
        }
    }
}

// ── 25. Language struct with skip field ──────────────────────────────────────

proptest! {
    #[test]
    fn language_struct_with_skip_field(idx in 0usize..=3) {
        let defaults = ["false", "true", "0", "42"];
        let default_expr: syn::Expr = syn::parse_str(defaults[idx]).unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
                #[adze::skip(#default_expr)]
                meta: bool,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            prop_assert_eq!(n.named.len(), 2);
            let skip_field = &n.named[1];
            prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        }
    }
}

// ── 26. Language determinism on enum ────────────────────────────────────────

proptest! {
    #[test]
    fn language_enum_deterministic(idx in 0usize..=3) {
        let names = ["Expr", "Stmt", "Decl", "Item"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let tokens = quote::quote! {
            #[adze::language]
            pub enum #ident {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
            }
        };
        let e1: ItemEnum = syn::parse2(tokens.clone()).unwrap();
        let e2: ItemEnum = syn::parse2(tokens).unwrap();
        prop_assert_eq!(e1.to_token_stream().to_string(), e2.to_token_stream().to_string());
    }
}

// ── 27. Language struct with Vec field ───────────────────────────────────────

proptest! {
    #[test]
    fn language_struct_with_vec_field(idx in 0usize..=3) {
        let inner_types = ["Number", "Token", "Stmt", "Item"];
        let inner = syn::Ident::new(inner_types[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                items: Vec<#inner>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            let ty_str = n.named[0].ty.to_token_stream().to_string();
            prop_assert!(ty_str.contains("Vec"));
        }
    }
}

// ── 28. Language struct with Option field ────────────────────────────────────

proptest! {
    #[test]
    fn language_struct_with_option_field(idx in 0usize..=3) {
        let inner_types = ["i32", "String", "u64", "bool"];
        let inner: syn::Type = syn::parse_str(inner_types[idx]).unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                value: Option<#inner>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            let ty_str = n.named[0].ty.to_token_stream().to_string();
            prop_assert!(ty_str.contains("Option"));
        }
    }
}

// ── 29. Language non-language types in module have no language attr ──────────

proptest! {
    #[test]
    fn non_language_types_have_no_language_attr(idx in 0usize..=3) {
        let helper_names = ["Helper", "Util", "Support", "Aux"];
        let helper = syn::Ident::new(helper_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                pub struct #helper {
                    value: i32,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Root".to_string()));
        // The helper should NOT have language attr
        let items = module_items(&m);
        for item in items {
            if let Item::Struct(s) = item {
                if s.ident == helper.to_string() {
                    prop_assert!(!s.attrs.iter().any(|a| is_adze_attr(a, "language")));
                }
            }
        }
    }
}

// ── 30. Language attr order: before vs after derive ─────────────────────────

proptest! {
    #[test]
    fn language_attr_order_with_derive(before in proptest::bool::ANY) {
        let tokens = if before {
            quote::quote! {
                #[adze::language]
                #[derive(Debug)]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    f: String,
                }
            }
        } else {
            quote::quote! {
                #[derive(Debug)]
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    f: String,
                }
            }
        };
        let s: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert!(s.attrs.iter().any(|a| a.path().is_ident("derive")));
    }
}

// ── 31. Language enum with Box fields ───────────────────────────────────────

proptest! {
    #[test]
    fn language_enum_with_box_fields(count in 1usize..=4) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name(Box<Expr>) }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "x")]
                Base,
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        // 1 base + count recursive variants
        prop_assert_eq!(e.variants.len(), 1 + count);
    }
}

// ── 32. Language type name preserved in grammar module ──────────────────────

proptest! {
    #[test]
    fn language_type_name_preserved_in_module(idx in 0usize..=5) {
        let type_names = ["MyRoot", "LangEntry", "AstTop", "ParseRoot", "GrammarStart", "Main"];
        let ident = syn::Ident::new(type_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #ident {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some(type_names[idx].to_string()));
    }
}

// ── 33. Language enum variant count preserved ───────────────────────────────

proptest! {
    #[test]
    fn language_enum_variant_count_preserved(count in 2usize..=8) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Var{i}"), proc_macro2::Span::call_site());
                let text = format!("t{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum TokenKind {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 34. Language struct field names preserved ────────────────────────────────

proptest! {
    #[test]
    fn language_struct_field_names_preserved(count in 1usize..=5) {
        let expected_names: Vec<String> = (0..count).map(|i| format!("field{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected_names.iter()
            .map(|n| {
                let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #(#fields),*
            }
        }).unwrap();
        if let Fields::Named(ref n) = s.fields {
            let actual: Vec<String> = n.named.iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            prop_assert_eq!(actual, expected_names);
        }
    }
}

// ── 35. Language enum variant names preserved ───────────────────────────────

proptest! {
    #[test]
    fn language_enum_variant_names_preserved(count in 2usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("Variant{i}")).collect();
        let variants: Vec<proc_macro2::TokenStream> = expected.iter()
            .map(|n| {
                let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
                let text = n.to_lowercase();
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #ident
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Kind {
                #(#variants),*
            }
        }).unwrap();
        let actual: Vec<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        prop_assert_eq!(actual, expected);
    }
}
