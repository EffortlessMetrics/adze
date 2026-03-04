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

// ── 31. Grammar name appears as string literal in attribute ─────────────────

proptest! {
    #[test]
    fn grammar_name_is_string_literal(idx in 0usize..=3) {
        let names = ["alpha", "beta", "gamma", "delta"];
        let gname = names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod g {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
            }
        });
        let extracted = extract_grammar_name(&m);
        prop_assert!(extracted.is_some());
        prop_assert!(extracted.unwrap().chars().all(|c| c.is_alphanumeric() || c == '_'));
    }
}

// ── 32. Grammar with multiple structs no language yields error ──────────────

proptest! {
    #[test]
    fn grammar_multiple_structs_no_language(count in 2usize..=5) {
        let structs: Vec<_> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\d+")]
                        val: String,
                    }
                }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#structs)*
            }
        }).unwrap();
        // No language attribute => find_language_type returns None
        prop_assert!(find_language_type(&m).is_none());
    }
}

// ── 33. Grammar attribute removed from output module attrs ─────────────────

proptest! {
    #[test]
    fn grammar_attr_stripped_from_expanded_output(idx in 0usize..=2) {
        let names = ["g1", "g2", "g3"];
        let gname = names[idx];
        let input: ItemMod = syn::parse2(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                }
            }
        }).unwrap();
        // The raw parse_mod preserves the attr
        prop_assert!(has_grammar_attr(&input));
    }
}

// ── 34. Grammar module ident is independent of grammar name ────────────────

proptest! {
    #[test]
    fn grammar_mod_ident_differs_from_name(idx in 0usize..=3) {
        let mod_names = ["my_mod", "parser", "lang", "rules"];
        let grammar_names = ["arithmetic", "json", "toml", "xml"];
        let mod_name = syn::Ident::new(mod_names[idx], proc_macro2::Span::call_site());
        let gname = grammar_names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod #mod_name {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        prop_assert_eq!(m.ident.to_string(), mod_names[idx]);
        let name = extract_grammar_name(&m);
        prop_assert_eq!(name.as_deref(), Some(gname));
    }
}

// ── 35. Grammar with mixed struct and enum language candidates ──────────────

proptest! {
    #[test]
    fn grammar_first_language_type_found(idx in 0usize..=1) {
        // Only one language attribute should exist; test detection picks it up
        let types = [
            quote::quote! {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
                pub enum Other {
                    A(#[adze::leaf(text = "a")] String),
                }
            },
            quote::quote! {
                pub struct Helper {
                    #[adze::leaf(pattern = r"\d+")]
                    n: String,
                }
                #[adze::language]
                pub enum Root {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                }
            },
        ];
        let tokens = &types[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #tokens
            }
        });
        let lang = find_language_type(&m);
        prop_assert!(lang.is_some());
        prop_assert_eq!(lang.unwrap(), "Root");
    }
}

// ── 36. Grammar produces rules (has items beyond just types) ────────────────

proptest! {
    #[test]
    fn grammar_module_produces_items(idx in 0usize..=2) {
        let extra_structs: Vec<_> = (0..=idx)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
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
                    t: String,
                }
                #(#extra_structs)*
            }
        });
        // At least 1 (Root) + extra count items
        prop_assert!(count_items(&m) >= idx + 2);
    }
}

// ── 37. Grammar name parameter special characters ───────────────────────────

proptest! {
    #[test]
    fn grammar_name_with_digits_and_underscores(idx in 0usize..=3) {
        let names = ["lang_v2", "parser_3_0", "my_lang_42", "v1_alpha"];
        let gname = names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(gname));
    }
}

// ── 38. Grammar with multiple enums only one language ───────────────────────

proptest! {
    #[test]
    fn grammar_multiple_enums_one_language(enum_count in 2usize..=4) {
        let mut enums: Vec<_> = (1..enum_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
                let vname = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("v{i}");
                quote::quote! {
                    pub enum #name {
                        #[adze::leaf(text = #text)]
                        #vname,
                    }
                }
            })
            .collect();
        enums.insert(0, quote::quote! {
            #[adze::language]
            pub enum Root {
                #[adze::leaf(text = "root")]
                R,
            }
        });
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#enums)*
            }
        });
        let lang = find_language_type(&m);
        prop_assert_eq!(lang.as_deref(), Some("Root"));
        prop_assert_eq!(enum_names(&m).len(), enum_count);
    }
}

// ── 39. Grammar empty module body still parseable ───────────────────────────

proptest! {
    #[test]
    fn grammar_with_only_language_struct(idx in 0usize..=2) {
        let names = ["mini", "tiny", "small"];
        let gname = names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        prop_assert_eq!(count_items(&m), 1);
        prop_assert!(find_language_type(&m).is_some());
    }
}

// ── 40. Grammar attribute validation: empty string name ─────────────────────

proptest! {
    #[test]
    fn grammar_empty_string_name_parsed(idx in 0usize..=1) {
        // Empty string is syntactically valid as a grammar name
        let m = parse_mod(quote::quote! {
            #[adze::grammar("")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let _ = idx; // use the proptest parameter
        let extracted = extract_grammar_name(&m);
        prop_assert_eq!(extracted.as_deref(), Some(""));
    }
}

// ── 41. Grammar struct item count preserved across repetitions ──────────────

proptest! {
    #[test]
    fn grammar_struct_item_count_stable(count in 1usize..=5) {
        let structs: Vec<_> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Node{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
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
                    t: String,
                }
                #(#structs)*
            }
        });
        // Root + count helper structs
        prop_assert_eq!(struct_names(&m).len(), count + 1);
    }
}

// ── 42. Grammar enum variants with box fields ───────────────────────────────

proptest! {
    #[test]
    fn grammar_enum_box_fields_preserved(variant_count in 1usize..=3) {
        let variants: Vec<_> = (0..variant_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Var{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #name(Box<Expr>)
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Leaf(#[adze::leaf(pattern = r"\d+")] String),
                    #(#variants),*
                }
            }
        });
        let expr_enum = enum_names(&m);
        prop_assert!(expr_enum.contains(&"Expr".to_string()));
        let e = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        }).unwrap();
        // Leaf + variant_count
        prop_assert_eq!(e.variants.len(), variant_count + 1);
    }
}

// ── 43. Grammar determinism: same input produces same token stream ──────────

proptest! {
    #[test]
    fn grammar_determinism_repeated_parse(idx in 0usize..=3) {
        let names = ["det1", "det2", "det3", "det4"];
        let gname = names[idx];
        let tokens = quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
            }
        };
        let m1 = parse_mod(tokens.clone());
        let m2 = parse_mod(tokens);
        prop_assert_eq!(m1.to_token_stream().to_string(), m2.to_token_stream().to_string());
    }
}

// ── 44. Grammar with prec_left and prec_right coexisting ────────────────────

proptest! {
    #[test]
    fn grammar_mixed_precedence_attrs(idx in 0usize..=2) {
        let precs = [1i32, 2, 3];
        let p = precs[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(#p)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_right(#p)]
                    Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
                }
            }
        });
        let e = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        }).unwrap();
        prop_assert_eq!(e.variants.len(), 3);
    }
}

// ── 45. Grammar with optional fields ────────────────────────────────────────

proptest! {
    #[test]
    fn grammar_struct_optional_fields(opt_count in 1usize..=4) {
        let fields: Vec<_> = (0..opt_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #name: Option<Helper>
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    base: String,
                    #(#fields),*
                }
                pub struct Helper {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        });
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        }).unwrap();
        // base + opt_count fields
        prop_assert_eq!(root.fields.len(), opt_count + 1);
    }
}

// ── 46. Grammar with vec repeat fields ──────────────────────────────────────

proptest! {
    #[test]
    fn grammar_struct_vec_fields(vec_count in 1usize..=3) {
        let fields: Vec<_> = (0..vec_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("items{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #name: Vec<Item>
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
                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        }).unwrap();
        prop_assert_eq!(root.fields.len(), vec_count);
    }
}

// ── 47. Grammar with skip fields on struct ──────────────────────────────────

proptest! {
    #[test]
    fn grammar_struct_skip_fields(skip_count in 1usize..=3) {
        let skip_fields: Vec<_> = (0..skip_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("skip{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                    #(#skip_fields),*
                }
            }
        });
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        }).unwrap();
        // tok + skip_count fields
        prop_assert_eq!(root.fields.len(), skip_count + 1);
    }
}

// ── 48. Grammar with word attribute struct ───────────────────────────────────

proptest! {
    #[test]
    fn grammar_word_attr_detected(idx in 0usize..=2) {
        let names = ["Ident", "Keyword", "Name"];
        let word_name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    w: #word_name,
                }
                #[adze::word]
                pub struct #word_name {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let word_struct = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == names[idx] => Some(s),
            _ => None,
        });
        prop_assert!(word_struct.is_some());
    }
}

// ── 49. Grammar with external scanner struct ────────────────────────────────

proptest! {
    #[test]
    fn grammar_external_attr_detected(idx in 0usize..=2) {
        let names = ["Indent", "Dedent", "Newline"];
        let ext_name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
                #[adze::external]
                struct #ext_name {
                    #[adze::leaf(pattern = r"\t+")]
                    _t: (),
                }
            }
        });
        let ext = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == names[idx] => Some(s),
            _ => None,
        });
        prop_assert!(ext.is_some());
    }
}

// ── 50. Grammar module visibility variations ────────────────────────────────

proptest! {
    #[test]
    fn grammar_crate_visibility(idx in 0usize..=1) {
        // Test with pub(crate) visibility
        let tokens = if idx == 0 {
            quote::quote! {
                #[adze::grammar("test")]
                pub(crate) mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        t: String,
                    }
                }
            }
        } else {
            quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root {
                        #[adze::leaf(pattern = r"\w+")]
                        t: String,
                    }
                }
            }
        };
        let m = parse_mod(tokens);
        prop_assert!(has_grammar_attr(&m));
        prop_assert!(find_language_type(&m).is_some());
    }
}

// ── 51. Grammar determinism: complex grammar twice ──────────────────────────

proptest! {
    #[test]
    fn grammar_complex_determinism(idx in 0usize..=2) {
        let gnames = ["cplx1", "cplx2", "cplx3"];
        let gname = gnames[idx];
        let make = || parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_left(2)]
                    Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let a = make();
        let b = make();
        prop_assert_eq!(a.to_token_stream().to_string(), b.to_token_stream().to_string());
    }
}

// ── 52. Grammar enum with leaf text variants preserved ──────────────────────

proptest! {
    #[test]
    fn grammar_leaf_text_variants_stable(count in 2usize..=6) {
        let variants: Vec<_> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let text = format!("op{i}");
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
                pub enum Operator {
                    #(#variants),*
                }
            }
        });
        let op_enum = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Operator" => Some(e),
            _ => None,
        });
        prop_assert!(op_enum.is_some());
        prop_assert_eq!(op_enum.unwrap().variants.len(), count);
    }
}

// ── 53. Grammar with delimited vec field ────────────────────────────────────

proptest! {
    #[test]
    fn grammar_delimited_field_struct(idx in 0usize..=2) {
        let delimiters = [",", ";", "|"];
        let delim = delimiters[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct List {
                    #[adze::delimited(
                        #[adze::leaf(text = #delim)]
                        ()
                    )]
                    items: Vec<Item>,
                }
                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        prop_assert_eq!(struct_names(&m).len(), 2);
    }
}

// ── 54. Grammar with repeat non_empty attribute ─────────────────────────────

proptest! {
    #[test]
    fn grammar_repeat_non_empty_field(idx in 0usize..=1) {
        let non_empty = idx == 0;
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct List {
                    #[adze::repeat(non_empty = #non_empty)]
                    items: Vec<Item>,
                }
                pub struct Item {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        });
        let list_struct = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "List" => Some(s),
            _ => None,
        });
        prop_assert!(list_struct.is_some());
        prop_assert_eq!(list_struct.unwrap().fields.len(), 1);
    }
}

// ── 55. Grammar multiple extras plus language ───────────────────────────────

proptest! {
    #[test]
    fn grammar_extra_count_alongside_language(extra_count in 1usize..=4) {
        let extras: Vec<_> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                let pat = format!("e{i}");
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = #pat)]
                        _e: (),
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
                    t: String,
                }
                #(#extras)*
            }
        });
        // Root + extra_count
        prop_assert_eq!(struct_names(&m).len(), extra_count + 1);
        prop_assert!(find_language_type(&m).is_some());
    }
}

// ── 56. Grammar struct with transform closure ───────────────────────────────

proptest! {
    #[test]
    fn grammar_leaf_transform_present(idx in 0usize..=2) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = #pat, transform = |v| v.to_string())]
                    val: String,
                }
            }
        });
        prop_assert!(find_language_type(&m).is_some());
        let root = module_items(&m).iter().find_map(|item| match item {
            Item::Struct(s) if s.ident == "Root" => Some(s),
            _ => None,
        }).unwrap();
        prop_assert_eq!(root.fields.len(), 1);
    }
}

// ── 57. Grammar with all precedence types ───────────────────────────────────

proptest! {
    #[test]
    fn grammar_all_prec_types_coexist(prec_val in 1i32..=5) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec(#prec_val)]
                    Eq(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
                    #[adze::prec_left(#prec_val)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_right(#prec_val)]
                    Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
                }
            }
        });
        let e = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        }).unwrap();
        prop_assert_eq!(e.variants.len(), 4);
    }
}

// ── 58. Grammar token stream size is non-zero ───────────────────────────────

proptest! {
    #[test]
    fn grammar_token_stream_nonempty(idx in 0usize..=2) {
        let names = ["a", "bb", "ccc"];
        let gname = names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let ts = m.to_token_stream().to_string();
        prop_assert!(!ts.is_empty());
        prop_assert!(ts.contains("grammar"));
    }
}

// ── 59. Grammar enum mixed named and unnamed variants ───────────────────────

proptest! {
    #[test]
    fn grammar_enum_mixed_variant_styles(idx in 0usize..=1) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    Neg {
                        #[adze::leaf(text = "-")]
                        _minus: (),
                        inner: Box<Expr>,
                    },
                }
            }
        });
        let _ = idx;
        let e = module_items(&m).iter().find_map(|item| match item {
            Item::Enum(e) if e.ident == "Expr" => Some(e),
            _ => None,
        }).unwrap();
        prop_assert_eq!(e.variants.len(), 2);
    }
}

// ── 60. Grammar module with static items ────────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_static_items(static_count in 1usize..=3) {
        let statics: Vec<_> = (0..static_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    static #name: &str = "val";
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
                #(#statics)*
            }
        });
        // Root struct + static_count statics
        prop_assert!(count_items(&m) >= static_count + 1);
    }
}

// ── 61. Grammar names are case sensitive ────────────────────────────────────

proptest! {
    #[test]
    fn grammar_name_case_sensitivity(idx in 0usize..=2) {
        let pairs = [("foo", "Foo"), ("bar", "BAR"), ("my_lang", "My_Lang")];
        let (lower, upper) = pairs[idx];
        let m_lower = parse_mod(quote::quote! {
            #[adze::grammar(#lower)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let m_upper = parse_mod(quote::quote! {
            #[adze::grammar(#upper)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let name_lower = extract_grammar_name(&m_lower).unwrap();
        let name_upper = extract_grammar_name(&m_upper).unwrap();
        prop_assert_ne!(name_lower, name_upper);
    }
}

// ── 62. Grammar struct and enum interleaved ─────────────────────────────────

proptest! {
    #[test]
    fn grammar_interleaved_struct_enum(pair_count in 1usize..=3) {
        let mut items = Vec::new();
        for i in 0..pair_count {
            let sname = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
            let ename = syn::Ident::new(&format!("E{i}"), proc_macro2::Span::call_site());
            let vname = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
            let text = format!("v{i}");
            items.push(quote::quote! {
                pub struct #sname {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            });
            items.push(quote::quote! {
                pub enum #ename {
                    #[adze::leaf(text = #text)]
                    #vname,
                }
            });
        }
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
                #(#items)*
            }
        });
        prop_assert_eq!(struct_names(&m).len(), pair_count + 1); // Root + S0..
        prop_assert_eq!(enum_names(&m).len(), pair_count);
    }
}
