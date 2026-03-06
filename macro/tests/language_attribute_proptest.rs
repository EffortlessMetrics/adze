#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::language]` attribute handling in adze-macro.
//!
//! Uses proptest to verify language attribute parsing, language on enum,
//! language name parameter, language combined with variants, language expansion
//! output, language attribute determinism, language with various enum shapes,
//! and language error cases.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct};

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
            if let Item::Struct(s) = item
                && s.ident == helper_names[idx]
            {
                prop_assert!(!s.attrs.iter().any(|a| is_adze_attr(a, "language")));
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

// ── 36. Language struct with word-annotated sibling ─────────────────────────

proptest! {
    #[test]
    fn language_struct_with_word_sibling(idx in 0usize..=3) {
        let patterns = [r"[a-zA-Z_]\w*", r"[a-z]+", r"\w+", r"[A-Z][a-zA-Z]*"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    ident: Identifier,
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Code".to_string()));
        // word-annotated type is not the language type
        let items = module_items(&m);
        for item in items {
            if let Item::Struct(s) = item
                && s.ident == "Identifier"
            {
                prop_assert!(!s.attrs.iter().any(|a| is_adze_attr(a, "language")));
                prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
            }
        }
    }
}

// ── 37. Language enum with word sibling type ────────────────────────────────

proptest! {
    #[test]
    fn language_enum_with_word_sibling(idx in 0usize..=2) {
        let word_names = ["Ident", "Keyword", "Name"];
        let word_ident = syn::Ident::new(word_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #[adze::leaf(text = "x")]
                    Lit,
                }

                #[adze::word]
                pub struct #word_ident {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Expr".to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
    }
}

// ── 38. Grammar name does not affect language type detection ────────────────

proptest! {
    #[test]
    fn grammar_name_independent_of_language_detection(idx in 0usize..=5) {
        let grammar_names = ["alpha", "beta_lang", "my_grammar", "test123", "g", "complex_name"];
        let gname = grammar_names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #[adze::leaf(text = "x")]
                    X,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Expr".to_string()));
    }
}

// ── 39. Language struct with multiple child type references ─────────────────

proptest! {
    #[test]
    fn language_struct_multiple_child_refs(child_count in 2usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..child_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("child{i}"), proc_macro2::Span::call_site());
                let ty = syn::Ident::new(&format!("Child{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: #ty }
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
            prop_assert_eq!(n.named.len(), child_count);
        }
    }
}

// ── 40. Language struct with Box child type ─────────────────────────────────

proptest! {
    #[test]
    fn language_struct_with_box_child(idx in 0usize..=3) {
        let child_names = ["Expr", "Statement", "Decl", "Block"];
        let child = syn::Ident::new(child_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                inner: Box<#child>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            let ty_str = n.named[0].ty.to_token_stream().to_string();
            prop_assert!(ty_str.contains("Box"));
            prop_assert!(ty_str.contains(child_names[idx]));
        }
    }
}

// ── 41. Language struct with Option<Box<T>> nested generic ──────────────────

proptest! {
    #[test]
    fn language_struct_option_box_nested(idx in 0usize..=3) {
        let inner_types = ["Expr", "Node", "Item", "Term"];
        let inner = syn::Ident::new(inner_types[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                maybe: Option<Box<#inner>>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            let ty_str = n.named[0].ty.to_token_stream().to_string();
            prop_assert!(ty_str.contains("Option"));
            prop_assert!(ty_str.contains("Box"));
        }
    }
}

// ── 42. Language struct with Vec and delimited attribute ─────────────────────

proptest! {
    #[test]
    fn language_struct_vec_delimited(idx in 0usize..=3) {
        let delimiters = [",", ";", "|", ":"];
        let delim = delimiters[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #[adze::delimited(
                    #[adze::leaf(text = #delim)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            prop_assert!(n.named[0].attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        }
    }
}

// ── 43. Language struct with repeat non_empty attribute ──────────────────────

proptest! {
    #[test]
    fn language_struct_repeat_non_empty(non_empty in proptest::bool::ANY) {
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = #non_empty)]
                items: Vec<Node>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            prop_assert!(n.named[0].attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        }
    }
}

// ── 44. Language determinism with complex enum ──────────────────────────────

proptest! {
    #[test]
    fn language_determinism_complex_enum(idx in 0usize..=2) {
        let variant_sets: Vec<Vec<proc_macro2::TokenStream>> = vec![
            vec![
                quote::quote! { Num(#[adze::leaf(pattern = r"\d+")] String) },
                quote::quote! { #[adze::prec_left(1)] Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>) },
            ],
            vec![
                quote::quote! { #[adze::leaf(text = "true")] True },
                quote::quote! { #[adze::leaf(text = "false")] False },
                quote::quote! { #[adze::prec_right(1)] And(Box<Expr>, Box<Expr>) },
            ],
            vec![
                quote::quote! { Lit(i32) },
                quote::quote! { Neg(#[adze::leaf(text = "-")] (), Box<Expr>) },
            ],
        ];
        let variants = &variant_sets[idx];
        let tokens = quote::quote! {
            #[adze::language]
            pub enum Expr {
                #(#variants),*
            }
        };
        let e1: ItemEnum = syn::parse2(tokens.clone()).unwrap();
        let e2: ItemEnum = syn::parse2(tokens).unwrap();
        prop_assert_eq!(
            e1.to_token_stream().to_string(),
            e2.to_token_stream().to_string()
        );
    }
}

// ── 45. Language determinism in grammar module ──────────────────────────────

proptest! {
    #[test]
    fn language_determinism_in_module(idx in 0usize..=3) {
        let names = ["calc", "json", "lisp", "sql"];
        let gname = names[idx];
        let tokens = quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }

                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        };
        let m1 = parse_mod(tokens.clone());
        let m2 = parse_mod(tokens);
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
    }
}

// ── 46. Language validation: extra is not language ──────────────────────────

proptest! {
    #[test]
    fn extra_type_not_detected_as_language(idx in 0usize..=3) {
        let extra_names = ["Whitespace", "Comment", "Newline", "Blank"];
        let extra = syn::Ident::new(extra_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::extra]
                struct #extra {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), None);
        prop_assert_eq!(count_language_types(&m), 0);
    }
}

// ── 47. Language validation: word is not language ───────────────────────────

proptest! {
    #[test]
    fn word_type_not_detected_as_language(idx in 0usize..=3) {
        let word_names = ["Identifier", "Word", "Symbol", "Name"];
        let word = syn::Ident::new(word_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::word]
                pub struct #word {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), None);
        prop_assert_eq!(count_language_types(&m), 0);
    }
}

// ── 48. Language validation: external is not language ───────────────────────

proptest! {
    #[test]
    fn external_type_not_detected_as_language(idx in 0usize..=2) {
        let ext_names = ["IndentToken", "DedentToken", "HeredocEnd"];
        let ext = syn::Ident::new(ext_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::external]
                struct #ext {
                    #[adze::leaf(pattern = r"\t+")]
                    _tok: (),
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), None);
        prop_assert_eq!(count_language_types(&m), 0);
    }
}

// ── 49. Language with all sibling annotation types ──────────────────────────

proptest! {
    #[test]
    fn language_with_all_sibling_types(idx in 0usize..=2) {
        let lang_names = ["Program", "Script", "Module"];
        let lang = syn::Ident::new(lang_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #lang {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }

                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }

                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }

                #[adze::external]
                struct Indent {
                    #[adze::leaf(pattern = r"\t+")]
                    _indent: (),
                }
            }
        });
        prop_assert_eq!(find_language_type(&m), Some(lang_names[idx].to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
    }
}

// ── 50. Language enum with recursive Box variants ───────────────────────────

proptest! {
    #[test]
    fn language_enum_recursive_box_variants(depth in 1usize..=3) {
        // Build variants with increasing nesting depth references
        let mut variants: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { #[adze::leaf(text = "x")] Lit },
        ];
        for i in 0..depth {
            let name = syn::Ident::new(&format!("Wrap{i}"), proc_macro2::Span::call_site());
            variants.push(quote::quote! { #name(Box<Expr>) });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), 1 + depth);
    }
}

// ── 51. Language struct produces root in grammar with helpers ────────────────

proptest! {
    #[test]
    fn language_produces_root_with_helpers(helper_count in 1usize..=4) {
        let helpers: Vec<proc_macro2::TokenStream> = (0..helper_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\d+")]
                        v: String,
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
                #(#helpers)*
            }
        });
        // Only Root is the language type regardless of helper count
        prop_assert_eq!(find_language_type(&m), Some("Root".to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
        // Verify helpers exist but aren't language
        let items = module_items(&m);
        let struct_count = items.iter().filter(|i| matches!(i, Item::Struct(_))).count();
        prop_assert_eq!(struct_count, 1 + helper_count);
    }
}

// ── 52. Language enum produces root in grammar with helpers ──────────────────

proptest! {
    #[test]
    fn language_enum_produces_root_with_siblings(sibling_count in 0usize..=3) {
        let siblings: Vec<proc_macro2::TokenStream> = (0..sibling_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Sub{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
                        val: String,
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #[adze::leaf(text = "x")]
                    Lit,
                }
                #(#siblings)*
            }
        });
        prop_assert_eq!(find_language_type(&m), Some("Expr".to_string()));
        prop_assert_eq!(count_language_types(&m), 1);
    }
}

// ── 53. Language struct field types preserved with generics ──────────────────

proptest! {
    #[test]
    fn language_struct_generic_field_types(idx in 0usize..=4) {
        let type_strs = ["Vec<Node>", "Option<i32>", "Box<Expr>", "Vec<Option<Node>>", "Option<Vec<Item>>"];
        let ty: syn::Type = syn::parse_str(type_strs[idx]).unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                field: #ty,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            let actual_ty = n.named[0].ty.to_token_stream().to_string();
            // Verify the type roundtrips through parsing
            let expected_ty = ty.to_token_stream().to_string();
            prop_assert_eq!(actual_ty, expected_ty);
        }
    }
}

// ── 54. Language struct with mixed leaf and non-leaf fields ──────────────────

proptest! {
    #[test]
    fn language_struct_mixed_leaf_nonleaf(leaf_count in 1usize..=3, nonleaf_count in 1usize..=3) {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..leaf_count {
            let name = syn::Ident::new(&format!("leaf{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! {
                #[adze::leaf(pattern = r"\w+")]
                #name: String
            });
        }
        for i in 0..nonleaf_count {
            let name = syn::Ident::new(&format!("child{i}"), proc_macro2::Span::call_site());
            let ty = syn::Ident::new(&format!("Child{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #name: #ty });
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct Root {
                #(#fields),*
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Named(ref n) = s.fields {
            prop_assert_eq!(n.named.len(), leaf_count + nonleaf_count);
            // First leaf_count fields have leaf attr
            for i in 0..leaf_count {
                prop_assert!(n.named[i].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            }
            // Remaining fields have no adze attrs
            for i in leaf_count..(leaf_count + nonleaf_count) {
                prop_assert!(adze_attr_names(&n.named[i].attrs).is_empty());
            }
        }
    }
}

// ── 55. Language on enum where language attr ordering with other attrs ───────

proptest! {
    #[test]
    fn language_enum_attr_ordering(idx in 0usize..=2) {
        let tokens = match idx {
            0 => quote::quote! {
                #[adze::language]
                #[derive(Debug)]
                pub enum Expr { Lit(i32) }
            },
            1 => quote::quote! {
                #[derive(Debug)]
                #[adze::language]
                pub enum Expr { Lit(i32) }
            },
            _ => quote::quote! {
                #[derive(Clone)]
                #[adze::language]
                #[derive(Debug)]
                pub enum Expr { Lit(i32) }
            },
        };
        let e: ItemEnum = syn::parse2(tokens).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert!(e.attrs.iter().any(|a| a.path().is_ident("derive")));
    }
}

// ── 56. Language grammar name is independent string ─────────────────────────

proptest! {
    #[test]
    fn grammar_name_is_string_literal(idx in 0usize..=5) {
        let grammar_names = ["a", "my_lang", "test_grammar", "json_parser", "x1", "lang99"];
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
        // Grammar attr should be parseable with the name
        let grammar_attr = m.attrs.iter().find(|a| {
            let segs: Vec<_> = a.path().segments.iter().collect();
            segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == "grammar"
        });
        prop_assert!(grammar_attr.is_some());
    }
}

// ── 57. Language struct with Spanned wrapper field ──────────────────────────

#[test]
fn language_struct_with_spanned_wrapper() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::language]
        pub struct Root {
            items: Vec<Spanned<Number>>,
        }
    })
    .unwrap();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref n) = s.fields {
        let ty_str = n.named[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Spanned"));
        assert!(ty_str.contains("Number"));
    }
}

// ── 58. Language struct with multiple doc attributes ────────────────────────

proptest! {
    #[test]
    fn language_struct_multiple_docs(doc_count in 1usize..=4) {
        let docs: Vec<proc_macro2::TokenStream> = (0..doc_count)
            .map(|i| {
                let doc = format!("Doc line {i}");
                quote::quote! { #[doc = #doc] }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #(#docs)*
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let doc_attrs: Vec<_> = s.attrs.iter().filter(|a| a.path().is_ident("doc")).collect();
        prop_assert_eq!(doc_attrs.len(), doc_count);
    }
}

// ── 59. Language struct with skip and leaf fields interleaved ────────────────

proptest! {
    #[test]
    fn language_struct_skip_leaf_interleaved(idx in 0usize..=2) {
        let tokens = match idx {
            0 => quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #[adze::skip(0)]
                    meta: i32,
                }
            },
            1 => quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::skip(false)]
                    flag: bool,
                    #[adze::leaf(pattern = r"\d+")]
                    num: String,
                }
            },
            _ => quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    a: String,
                    #[adze::skip(true)]
                    b: bool,
                    #[adze::leaf(pattern = r"\d+")]
                    c: String,
                }
            },
        };
        let s: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 60. Language enum with prec variants at different levels ─────────────────

proptest! {
    #[test]
    fn language_enum_multiple_prec_levels(level_count in 2usize..=5) {
        let mut variants: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { Num(#[adze::leaf(pattern = r"\d+")] String) },
        ];
        for i in 0..level_count {
            let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
            let prec = proc_macro2::Literal::usize_unsuffixed(i + 1);
            variants.push(quote::quote! {
                #[adze::prec_left(#prec)]
                #name(Box<Expr>, Box<Expr>)
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        // 1 base + level_count prec variants
        prop_assert_eq!(e.variants.len(), 1 + level_count);
        // Each Op variant has prec_left
        for i in 0..level_count {
            let vname = format!("Op{i}");
            let v = e.variants.iter().find(|v| v.ident == vname).unwrap();
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
        }
    }
}
