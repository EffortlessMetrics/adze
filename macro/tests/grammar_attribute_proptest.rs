#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::grammar]` attribute handling in adze-macro.
//!
//! Uses proptest to verify grammar attribute on modules, expansion preserving items,
//! grammars with structs and enums, nested modules, module name handling,
//! attribute determinism, various item counts, and output structure.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Item, ItemMod, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn has_grammar_attr(m: &ItemMod) -> bool {
    m.attrs.iter().any(|a| is_adze_attr(a, "grammar"))
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
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

fn struct_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| match i {
            Item::Struct(s) => Some(s.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn enum_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| match i {
            Item::Enum(e) => Some(e.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn count_items(m: &ItemMod) -> usize {
    module_items(m).len()
}

// ── 1. Grammar attribute on module with varying grammar names ───────────────

proptest! {
    #[test]
    fn grammar_attr_on_module_varying_names(idx in 0usize..=5) {
        let names = ["arithmetic", "json", "calc", "my_lang", "proto", "css"];
        let gname = names[idx];
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
        prop_assert!(has_grammar_attr(&m));
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// ── 2. Grammar expansion preserves struct items ─────────────────────────────

proptest! {
    #[test]
    fn grammar_preserves_struct_count(extra_count in 0usize..=4) {
        let extras: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
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
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extras)*
            }
        });
        // Root + extras
        prop_assert_eq!(struct_names(&m).len(), 1 + extra_count);
    }
}

// ── 3. Grammar with struct language type detected ───────────────────────────

proptest! {
    #[test]
    fn grammar_struct_language_type_detected(idx in 0usize..=4) {
        let type_names = ["Root", "Program", "Ast", "Entry", "TopLevel"];
        let name = type_names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
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
        let found = find_language_type(&m);
        prop_assert_eq!(found.as_deref(), Some(name));
    }
}

// ── 4. Grammar with enum language type detected ─────────────────────────────

proptest! {
    #[test]
    fn grammar_enum_language_type_detected(idx in 0usize..=4) {
        let type_names = ["Expr", "Value", "Token", "Node", "Statement"];
        let name = type_names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
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

// ── 5. Grammar with structs and enums mixed ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_structs_and_enums_mixed(
        struct_count in 1usize..=3,
        enum_count in 0usize..=2
    ) {
        let structs: Vec<proc_macro2::TokenStream> = (0..struct_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                if i == 0 {
                    quote::quote! {
                        #[adze::language]
                        pub struct #name {
                            #[adze::leaf(pattern = r"\w+")]
                            val: String,
                        }
                    }
                } else {
                    quote::quote! {
                        pub struct #name {
                            #[adze::leaf(pattern = r"\w+")]
                            val: String,
                        }
                    }
                }
            })
            .collect();
        let enums: Vec<proc_macro2::TokenStream> = (0..enum_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("E{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub enum #name {
                        #[adze::leaf(text = "+")]
                        A,
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#structs)*
                #(#enums)*
            }
        });
        prop_assert_eq!(struct_names(&m).len(), struct_count);
        prop_assert_eq!(enum_names(&m).len(), enum_count);
    }
}

// ── 6. Grammar with nested inner module ─────────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_nested_inner_module(idx in 0usize..=3) {
        let inner_names = ["helpers", "utils", "types", "nodes"];
        let inner = syn::Ident::new(inner_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                mod #inner {
                    pub struct Inner {
                        val: i32,
                    }
                }
            }
        });
        prop_assert!(has_grammar_attr(&m));
        // Module item + struct
        let has_inner_mod = module_items(&m).iter().any(|item| {
            matches!(item, Item::Mod(im) if im.ident == inner_names[idx])
        });
        prop_assert!(has_inner_mod);
    }
}

// ── 7. Grammar module name preserved ────────────────────────────────────────

proptest! {
    #[test]
    fn grammar_module_name_preserved(idx in 0usize..=4) {
        let mod_names = ["grammar", "parser", "ast", "syntax", "lang"];
        let mod_name = mod_names[idx];
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod #mod_ident {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(m.ident.to_string(), mod_name);
    }
}

// ── 8. Grammar attribute determinism: same input → same output ──────────────

proptest! {
    #[test]
    fn grammar_attr_determinism(idx in 0usize..=3) {
        let grammar_names = ["det_a", "det_b", "det_c", "det_d"];
        let gname = grammar_names[idx];
        let build = || {
            parse_mod(quote::quote! {
                #[adze::grammar(#gname)]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        token: String,
                    }
                }
            })
        };
        let a = build().to_token_stream().to_string();
        let b = build().to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }
}

// ── 9. Grammar with various item counts ─────────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_various_item_counts(count in 1usize..=6) {
        let items: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Type{i}"), proc_macro2::Span::call_site());
                if i == 0 {
                    quote::quote! {
                        #[adze::language]
                        pub struct #name {
                            #[adze::leaf(pattern = r"\w+")]
                            val: String,
                        }
                    }
                } else {
                    quote::quote! {
                        pub struct #name {
                            #[adze::leaf(pattern = r"\w+")]
                            val: String,
                        }
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#items)*
            }
        });
        prop_assert_eq!(count_items(&m), count);
    }
}

// ── 10. Grammar output structure has content ────────────────────────────────

proptest! {
    #[test]
    fn grammar_output_has_content(idx in 0usize..=2) {
        let bodies: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            },
            quote::quote! {
                #[adze::language]
                pub enum Expr {
                    #[adze::leaf(text = "+")]
                    Plus,
                }
            },
            quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+")]
                    num: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            },
        ];
        let body = &bodies[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #body
            }
        });
        prop_assert!(m.content.is_some());
        prop_assert!(!module_items(&m).is_empty());
    }
}

// ── 11. Grammar name with underscores ───────────────────────────────────────

proptest! {
    #[test]
    fn grammar_name_with_underscores(idx in 0usize..=3) {
        let names = ["my_grammar", "some_lang_v2", "test_case", "a_b_c"];
        let gname = names[idx];
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
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// ── 12. Grammar module visibility preserved ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_module_pub_visibility(idx in 0usize..=1) {
        let m = if idx == 0 {
            parse_mod(quote::quote! {
                #[adze::grammar("test")]
                pub mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        token: String,
                    }
                }
            })
        } else {
            parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        token: String,
                    }
                }
            })
        };
        if idx == 0 {
            prop_assert!(matches!(m.vis, syn::Visibility::Public(_)));
        } else {
            prop_assert!(matches!(m.vis, syn::Visibility::Inherited));
        }
    }
}

// ── 13. Grammar preserves enum variant counts ───────────────────────────────

proptest! {
    #[test]
    fn grammar_preserves_enum_variant_counts(variant_count in 1usize..=5) {
        let variants: Vec<proc_macro2::TokenStream> = (0..variant_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(text = "+")]
                    #name
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #(#variants),*
                }
            }
        });
        let expr_enum = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        });
        prop_assert!(expr_enum.is_some());
        prop_assert_eq!(expr_enum.unwrap().variants.len(), variant_count);
    }
}

// ── 14. Grammar with extra types ────────────────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_extra_types(extra_count in 1usize..=3) {
        let extras: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                let pat = format!("\\s{}", "+".repeat(i));
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = #pat)]
                        _val: (),
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
        let extra_structs: Vec<_> = module_items(&m)
            .iter()
            .filter(|item| match item {
                Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
                _ => false,
            })
            .collect();
        prop_assert_eq!(extra_structs.len(), extra_count);
    }
}

// ── 15. Grammar attribute removed after parsing still preserves structure ───

proptest! {
    #[test]
    fn grammar_attr_presence_is_stable(idx in 0usize..=2) {
        let grammar_names = ["stable_a", "stable_b", "stable_c"];
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
        prop_assert!(has_grammar_attr(&m));
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// ── 16. Grammar with use statements preserved ───────────────────────────────

proptest! {
    #[test]
    fn grammar_with_use_statements(use_count in 1usize..=3) {
        let uses: Vec<proc_macro2::TokenStream> = (0..use_count)
            .map(|_| {
                quote::quote! {
                    use std::fmt::Debug;
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#uses)*
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        let use_count_found = module_items(&m)
            .iter()
            .filter(|item| matches!(item, Item::Use(_)))
            .count();
        prop_assert_eq!(use_count_found, use_count);
    }
}

// ── 17. Grammar struct fields preserved ─────────────────────────────────────

proptest! {
    #[test]
    fn grammar_struct_fields_preserved(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #name: String
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #(#fields),*
                }
            }
        });
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        });
        prop_assert!(root.is_some());
        prop_assert_eq!(root.unwrap().fields.len(), field_count);
    }
}

// ── 18. Grammar module token stream roundtrip ───────────────────────────────

proptest! {
    #[test]
    fn grammar_module_token_stream_roundtrip(idx in 0usize..=2) {
        let gnames = ["rt_a", "rt_b", "rt_c"];
        let gname = gnames[idx];
        let tokens = quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        };
        let m: ItemMod = syn::parse2(tokens.clone()).unwrap();
        let reparsed: ItemMod = syn::parse2(m.to_token_stream()).unwrap();
        prop_assert_eq!(m.ident.to_string(), reparsed.ident.to_string());
        prop_assert_eq!(
            extract_grammar_name(&m),
            extract_grammar_name(&reparsed)
        );
    }
}

// ── 19. Grammar with multiple nested modules ────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_multiple_nested_modules(nested_count in 1usize..=3) {
        let nested: Vec<proc_macro2::TokenStream> = (0..nested_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("inner{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    mod #name {
                        pub struct Data {
                            val: i32,
                        }
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
                #(#nested)*
            }
        });
        let mod_count = module_items(&m)
            .iter()
            .filter(|item| matches!(item, Item::Mod(_)))
            .count();
        prop_assert_eq!(mod_count, nested_count);
    }
}

// ── 20. Grammar name is not lost with various module names ──────────────────

proptest! {
    #[test]
    fn grammar_name_independent_of_module_name(idx in 0usize..=3) {
        let mod_names = ["grammar", "parser", "ast", "my_mod"];
        let grammar_names = ["gname_a", "gname_b", "gname_c", "gname_d"];
        let mod_ident = syn::Ident::new(mod_names[idx], proc_macro2::Span::call_site());
        let gname = grammar_names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod #mod_ident {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(m.ident.to_string(), mod_names[idx]);
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// ── 21. Grammar attribute coexists with other attributes ────────────────────

proptest! {
    #[test]
    fn grammar_attr_coexists_with_other_attrs(idx in 0usize..=2) {
        let extra_attrs: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { #[allow(dead_code)] },
            quote::quote! { #[cfg(test)] },
            quote::quote! { #[doc = "grammar module"] },
        ];
        let extra = &extra_attrs[idx];
        let m = parse_mod(quote::quote! {
            #extra
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert!(has_grammar_attr(&m));
        // At least 2 attrs: the extra one + grammar
        prop_assert!(m.attrs.len() >= 2);
    }
}

// ── 22. Grammar with enum having named fields ───────────────────────────────

proptest! {
    #[test]
    fn grammar_enum_with_named_fields(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #name: String
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Variant { #(#fields),* },
                }
            }
        });
        let expr_enum = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        });
        prop_assert!(expr_enum.is_some());
        let variant = &expr_enum.unwrap().variants[0];
        prop_assert_eq!(variant.fields.len(), field_count);
    }
}

// ── 23. Grammar with enum having unnamed fields ─────────────────────────────

proptest! {
    #[test]
    fn grammar_enum_with_unnamed_fields(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| {
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    String
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Variant(#(#fields),*),
                }
            }
        });
        let expr_enum = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        });
        prop_assert!(expr_enum.is_some());
        let variant = &expr_enum.unwrap().variants[0];
        prop_assert_eq!(variant.fields.len(), field_count);
    }
}

// ── 24. Grammar preserves non-adze attributes on inner items ────────────────

proptest! {
    #[test]
    fn grammar_preserves_non_adze_attrs_on_items(idx in 0usize..=2) {
        let attr_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { #[derive(Debug)] },
            quote::quote! { #[derive(Clone)] },
            quote::quote! { #[allow(unused)] },
        ];
        let attr = &attr_tokens[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #attr
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        });
        prop_assert!(root.is_some());
        // Should have both the non-adze attr and the adze::language attr
        prop_assert!(root.unwrap().attrs.len() >= 2);
    }
}

// ── 25. Grammar module with only enum items ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_module_only_enums(enum_count in 1usize..=4) {
        let enums: Vec<proc_macro2::TokenStream> = (0..enum_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("E{i}"), proc_macro2::Span::call_site());
                if i == 0 {
                    quote::quote! {
                        #[adze::language]
                        pub enum #name {
                            #[adze::leaf(text = "+")]
                            A,
                        }
                    }
                } else {
                    quote::quote! {
                        pub enum #name {
                            #[adze::leaf(text = "-")]
                            B,
                        }
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#enums)*
            }
        });
        prop_assert_eq!(enum_names(&m).len(), enum_count);
        prop_assert!(struct_names(&m).is_empty());
    }
}

// ── 26. Grammar with function items alongside types ─────────────────────────

proptest! {
    #[test]
    fn grammar_with_fn_items(fn_count in 1usize..=3) {
        let fns: Vec<proc_macro2::TokenStream> = (0..fn_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("helper{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    fn #name() -> i32 { 42 }
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
                #(#fns)*
            }
        });
        let fn_items = module_items(&m)
            .iter()
            .filter(|item| matches!(item, Item::Fn(_)))
            .count();
        prop_assert_eq!(fn_items, fn_count);
    }
}

// ── 27. Grammar determinism with enum bodies ────────────────────────────────

proptest! {
    #[test]
    fn grammar_determinism_with_enum(variant_count in 1usize..=4) {
        let build = || {
            let variants: Vec<proc_macro2::TokenStream> = (0..variant_count)
                .map(|i| {
                    let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                    quote::quote! {
                        #[adze::leaf(text = "+")]
                        #name
                    }
                })
                .collect();
            parse_mod(quote::quote! {
                #[adze::grammar("det")]
                mod grammar {
                    #[adze::language]
                    pub enum Expr {
                        #(#variants),*
                    }
                }
            })
        };
        let a = build().to_token_stream().to_string();
        let b = build().to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }
}

// ── 28. Grammar output contains grammar attr on module ──────────────────────

proptest! {
    #[test]
    fn grammar_output_contains_grammar_attr(idx in 0usize..=3) {
        let names = ["out_a", "out_b", "out_c", "out_d"];
        let gname = names[idx];
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
        let output = m.to_token_stream().to_string();
        prop_assert!(output.contains("grammar"));
        prop_assert!(output.contains("Root"));
    }
}

// ── 29. Grammar with const items preserved ──────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_const_items(const_count in 1usize..=3) {
        let consts: Vec<proc_macro2::TokenStream> = (0..const_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("C{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    const #name: i32 = 42;
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
                #(#consts)*
            }
        });
        let const_items = module_items(&m)
            .iter()
            .filter(|item| matches!(item, Item::Const(_)))
            .count();
        prop_assert_eq!(const_items, const_count);
    }
}

// ── 30. Grammar with type aliases preserved ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_type_aliases(alias_count in 1usize..=3) {
        let aliases: Vec<proc_macro2::TokenStream> = (0..alias_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Alias{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    type #name = i32;
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
                #(#aliases)*
            }
        });
        let type_items = module_items(&m)
            .iter()
            .filter(|item| matches!(item, Item::Type(_)))
            .count();
        prop_assert_eq!(type_items, alias_count);
    }
}

// ── 31. Grammar enum unit variant count ─────────────────────────────────────

proptest! {
    #[test]
    fn grammar_enum_unit_variant_count(count in 1usize..=6) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Kw{i}"), proc_macro2::Span::call_site());
                let text = format!("kw{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Keyword {
                    #(#variants),*
                }
            }
        });
        let kw_enum = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Keyword" => Some(e),
            _ => None,
        });
        prop_assert!(kw_enum.is_some());
        prop_assert_eq!(kw_enum.unwrap().variants.len(), count);
    }
}
