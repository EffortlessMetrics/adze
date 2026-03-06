#![allow(clippy::needless_range_loop)]

//! Property-based tests for visibility handling in adze-macro.
//!
//! Uses proptest to generate randomized type definitions and verify that
//! syn correctly parses and preserves visibility modifiers on structs,
//! enums, fields, and modules across different visibility combinations.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Visibility};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn is_inherited(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Inherited)
}

fn is_restricted(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Restricted(_))
}

fn vis_to_string(vis: &Visibility) -> String {
    vis.to_token_stream().to_string()
}

// ── 1. Public struct in grammar module ──────────────────────────────────────

proptest! {
    #[test]
    fn pub_struct_in_grammar_module(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar {
                pub struct MyStruct { #(#fields),* }
            }
        }).unwrap();
        let (_, items) = m.content.unwrap();
        if let syn::Item::Struct(s) = &items[0] {
            prop_assert!(is_pub(&s.vis));
            prop_assert_eq!(s.fields.len(), field_count);
        } else {
            prop_assert!(false, "Expected struct item");
        }
    }
}

// ── 2. Private struct in grammar module ─────────────────────────────────────

proptest! {
    #[test]
    fn private_struct_in_grammar_module(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: String }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar {
                struct PrivateStruct { #(#fields),* }
            }
        }).unwrap();
        let (_, items) = m.content.unwrap();
        if let syn::Item::Struct(s) = &items[0] {
            prop_assert!(is_inherited(&s.vis));
            prop_assert_eq!(s.fields.len(), field_count);
        } else {
            prop_assert!(false, "Expected struct item");
        }
    }
}

// ── 3. pub(crate) struct preserved ──────────────────────────────────────────

proptest! {
    #[test]
    fn pub_crate_struct_preserved(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub(crate) struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(is_restricted(&s.vis));
        let vis_str = vis_to_string(&s.vis);
        prop_assert!(vis_str.contains("crate"), "Expected pub(crate), got: {}", vis_str);
    }
}

// ── 4. Field visibility pub preserved ───────────────────────────────────────

proptest! {
    #[test]
    fn pub_field_visibility_preserved(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { pub #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_pub(&field.vis));
        }
    }
}

// ── 5. Private field visibility preserved ───────────────────────────────────

proptest! {
    #[test]
    fn private_field_visibility_preserved(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: String }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_inherited(&field.vis));
        }
    }
}

// ── 6. pub(crate) field visibility preserved ────────────────────────────────

proptest! {
    #[test]
    fn pub_crate_field_visibility_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { pub(crate) #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_restricted(&field.vis));
        }
    }
}

// ── 7. Enum visibility pub ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn pub_enum_visibility(variant_count in 1usize..=6) {
        let variants: Vec<proc_macro2::TokenStream> = (0..variant_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name(i32) }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variants),* }
        }).unwrap();
        prop_assert!(is_pub(&e.vis));
        prop_assert_eq!(e.variants.len(), variant_count);
    }
}

// ── 8. Private enum visibility ──────────────────────────────────────────────

proptest! {
    #[test]
    fn private_enum_visibility(variant_count in 1usize..=6) {
        let variants: Vec<proc_macro2::TokenStream> = (0..variant_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            enum E { #(#variants),* }
        }).unwrap();
        prop_assert!(is_inherited(&e.vis));
        prop_assert_eq!(e.variants.len(), variant_count);
    }
}

// ── 9. pub(crate) enum visibility ───────────────────────────────────────────

proptest! {
    #[test]
    fn pub_crate_enum_visibility(variant_count in 1usize..=5) {
        let variants: Vec<proc_macro2::TokenStream> = (0..variant_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name(String) }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub(crate) enum E { #(#variants),* }
        }).unwrap();
        prop_assert!(is_restricted(&e.vis));
    }
}

// ── 10. Module visibility pub ───────────────────────────────────────────────

proptest! {
    #[test]
    fn pub_module_visibility(item_count in 1usize..=4) {
        let items: Vec<proc_macro2::TokenStream> = (0..item_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! { pub struct #name { value: i32 } }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            pub mod grammar { #(#items)* }
        }).unwrap();
        prop_assert!(is_pub(&m.vis));
        let (_, content) = m.content.unwrap();
        prop_assert_eq!(content.len(), item_count);
    }
}

// ── 11. Private module visibility ───────────────────────────────────────────

proptest! {
    #[test]
    fn private_module_visibility(item_count in 1usize..=3) {
        let items: Vec<proc_macro2::TokenStream> = (0..item_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! { struct #name; }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar { #(#items)* }
        }).unwrap();
        prop_assert!(is_inherited(&m.vis));
    }
}

// ── 12. pub(crate) module visibility ────────────────────────────────────────

proptest! {
    #[test]
    fn pub_crate_module_visibility(item_count in 1usize..=3) {
        let items: Vec<proc_macro2::TokenStream> = (0..item_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! { struct #name; }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            pub(crate) mod grammar { #(#items)* }
        }).unwrap();
        prop_assert!(is_restricted(&m.vis));
    }
}

// ── 13. Mixed visibility fields in struct ───────────────────────────────────

proptest! {
    #[test]
    fn mixed_visibility_fields(n_pub in 1usize..=3, n_priv in 1usize..=3) {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_pub {
            let name = syn::Ident::new(&format!("pub_f{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { pub #name: i32 });
        }
        for i in 0..n_priv {
            let name = syn::Ident::new(&format!("priv_f{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #name: i32 });
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert_eq!(s.fields.len(), n_pub + n_priv);
        let field_vec: Vec<_> = s.fields.iter().collect();
        for i in 0..n_pub {
            prop_assert!(is_pub(&field_vec[i].vis));
        }
        for i in 0..n_priv {
            prop_assert!(is_inherited(&field_vec[n_pub + i].vis));
        }
    }
}

// ── 14. Grammar output preserves struct visibility ──────────────────────────

proptest! {
    #[test]
    fn grammar_module_preserves_struct_vis(use_pub in proptest::bool::ANY) {
        let m: ItemMod = if use_pub {
            syn::parse2(quote::quote! {
                mod grammar {
                    pub struct Language { value: i32 }
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                mod grammar {
                    struct Language { value: i32 }
                }
            }).unwrap()
        };
        let (_, items) = m.content.unwrap();
        if let syn::Item::Struct(s) = &items[0] {
            if use_pub {
                prop_assert!(is_pub(&s.vis));
            } else {
                prop_assert!(is_inherited(&s.vis));
            }
        } else {
            prop_assert!(false, "Expected struct");
        }
    }
}

// ── 15. Grammar output preserves enum visibility ────────────────────────────

proptest! {
    #[test]
    fn grammar_module_preserves_enum_vis(use_pub in proptest::bool::ANY) {
        let m: ItemMod = if use_pub {
            syn::parse2(quote::quote! {
                mod grammar {
                    pub enum Expr { A, B }
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                mod grammar {
                    enum Expr { A, B }
                }
            }).unwrap()
        };
        let (_, items) = m.content.unwrap();
        if let syn::Item::Enum(e) = &items[0] {
            if use_pub {
                prop_assert!(is_pub(&e.vis));
            } else {
                prop_assert!(is_inherited(&e.vis));
            }
        } else {
            prop_assert!(false, "Expected enum");
        }
    }
}

// ── 16. Mixed visibility items in module ────────────────────────────────────

proptest! {
    #[test]
    fn mixed_visibility_items_in_module(n_pub in 1usize..=3, n_priv in 1usize..=3) {
        let mut items: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_pub {
            let name = syn::Ident::new(&format!("Pub{i}"), proc_macro2::Span::call_site());
            items.push(quote::quote! { pub struct #name { value: i32 } });
        }
        for i in 0..n_priv {
            let name = syn::Ident::new(&format!("Priv{i}"), proc_macro2::Span::call_site());
            items.push(quote::quote! { struct #name { value: i32 } });
        }
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar { #(#items)* }
        }).unwrap();
        let (_, content) = m.content.unwrap();
        prop_assert_eq!(content.len(), n_pub + n_priv);
        for i in 0..n_pub {
            if let syn::Item::Struct(s) = &content[i] {
                prop_assert!(is_pub(&s.vis));
            }
        }
        for i in 0..n_priv {
            if let syn::Item::Struct(s) = &content[n_pub + i] {
                prop_assert!(is_inherited(&s.vis));
            }
        }
    }
}

// ── 17. pub(super) struct visibility ────────────────────────────────────────

proptest! {
    #[test]
    fn pub_super_struct_visibility(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub(super) struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(is_restricted(&s.vis));
        let vis_str = vis_to_string(&s.vis);
        prop_assert!(vis_str.contains("super"), "Expected pub(super), got: {}", vis_str);
    }
}

// ── 18. pub(super) field visibility ─────────────────────────────────────────

proptest! {
    #[test]
    fn pub_super_field_visibility(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { pub(super) #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_restricted(&field.vis));
        }
    }
}

// ── 19. Visibility unaffected by adze attributes on struct ──────────────────

proptest! {
    #[test]
    fn visibility_with_adze_attrs_on_struct(use_pub in proptest::bool::ANY) {
        let s: ItemStruct = if use_pub {
            syn::parse2(quote::quote! {
                #[adze::language]
                pub struct S {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::language]
                struct S {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }
            }).unwrap()
        };
        if use_pub {
            prop_assert!(is_pub(&s.vis));
        } else {
            prop_assert!(is_inherited(&s.vis));
        }
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 20. Visibility unaffected by adze attributes on enum ────────────────────

proptest! {
    #[test]
    fn visibility_with_adze_attrs_on_enum(use_pub in proptest::bool::ANY) {
        let e: ItemEnum = if use_pub {
            syn::parse2(quote::quote! {
                #[adze::language]
                pub enum E {
                    #[adze::leaf(text = "+")]
                    Plus,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::language]
                enum E {
                    #[adze::leaf(text = "+")]
                    Plus,
                }
            }).unwrap()
        };
        if use_pub {
            prop_assert!(is_pub(&e.vis));
        } else {
            prop_assert!(is_inherited(&e.vis));
        }
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 21. Three-way field visibility mix ──────────────────────────────────────

proptest! {
    #[test]
    fn three_way_field_visibility_mix(
        n_pub in 1usize..=2,
        n_priv in 1usize..=2,
        n_crate in 1usize..=2,
    ) {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_pub {
            let name = syn::Ident::new(&format!("pub_{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { pub #name: i32 });
        }
        for i in 0..n_priv {
            let name = syn::Ident::new(&format!("priv_{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #name: i32 });
        }
        for i in 0..n_crate {
            let name = syn::Ident::new(&format!("crate_{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { pub(crate) #name: i32 });
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let field_vec: Vec<_> = s.fields.iter().collect();
        let total = n_pub + n_priv + n_crate;
        prop_assert_eq!(field_vec.len(), total);
        for i in 0..n_pub {
            prop_assert!(is_pub(&field_vec[i].vis));
        }
        for i in 0..n_priv {
            prop_assert!(is_inherited(&field_vec[n_pub + i].vis));
        }
        for i in 0..n_crate {
            prop_assert!(is_restricted(&field_vec[n_pub + n_priv + i].vis));
        }
    }
}

// ── 22. Enum with pub(crate) and adze::language ─────────────────────────────

proptest! {
    #[test]
    fn pub_crate_enum_with_language(count in 1usize..=5) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name(i32) }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub(crate) enum E { #(#variants),* }
        }).unwrap();
        prop_assert!(is_restricted(&e.vis));
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 23. Module with mixed pub and private enums and structs ─────────────────

proptest! {
    #[test]
    fn module_mixed_struct_enum_visibility(n_structs in 1usize..=2, n_enums in 1usize..=2) {
        let mut items: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_structs {
            let name = syn::Ident::new(&format!("St{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                items.push(quote::quote! { pub struct #name { v: i32 } });
            } else {
                items.push(quote::quote! { struct #name { v: i32 } });
            }
        }
        for i in 0..n_enums {
            let name = syn::Ident::new(&format!("En{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                items.push(quote::quote! { enum #name { A } });
            } else {
                items.push(quote::quote! { pub enum #name { A } });
            }
        }
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar { #(#items)* }
        }).unwrap();
        let (_, content) = m.content.unwrap();
        prop_assert_eq!(content.len(), n_structs + n_enums);
        for i in 0..n_structs {
            if let syn::Item::Struct(s) = &content[i] {
                if i % 2 == 0 {
                    prop_assert!(is_pub(&s.vis));
                } else {
                    prop_assert!(is_inherited(&s.vis));
                }
            }
        }
        for i in 0..n_enums {
            if let syn::Item::Enum(e) = &content[n_structs + i] {
                if i % 2 == 0 {
                    prop_assert!(is_inherited(&e.vis));
                } else {
                    prop_assert!(is_pub(&e.vis));
                }
            }
        }
    }
}

// ── 24. Struct visibility independent of field annotations ──────────────────

proptest! {
    #[test]
    fn struct_vis_independent_of_field_attrs(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    pub #name: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            struct S { #(#fields),* }
        }).unwrap();
        // Struct is private
        prop_assert!(is_inherited(&s.vis));
        // But fields are public
        for field in &s.fields {
            prop_assert!(is_pub(&field.vis));
            prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 25. Visibility preserved on extra-annotated struct ──────────────────────

proptest! {
    #[test]
    fn visibility_on_extra_struct(use_pub in proptest::bool::ANY) {
        let s: ItemStruct = if use_pub {
            syn::parse2(quote::quote! {
                #[adze::extra]
                pub struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }).unwrap()
        };
        if use_pub {
            prop_assert!(is_pub(&s.vis));
        } else {
            prop_assert!(is_inherited(&s.vis));
        }
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    }
}

// ── 26. Visibility preserved on word-annotated struct ───────────────────────

proptest! {
    #[test]
    fn visibility_on_word_struct(use_pub in proptest::bool::ANY) {
        let s: ItemStruct = if use_pub {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::word]
                struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }).unwrap()
        };
        if use_pub {
            prop_assert!(is_pub(&s.vis));
        } else {
            prop_assert!(is_inherited(&s.vis));
        }
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 27. Visibility preserved on external-annotated struct ───────────────────

proptest! {
    #[test]
    fn visibility_on_external_struct(idx in 0usize..=2) {
        let vis_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { pub },
            quote::quote! {},
            quote::quote! { pub(crate) },
        ];
        let vis_tok = &vis_tokens[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::external]
            #vis_tok struct IndentToken;
        }).unwrap();
        match idx {
            0 => prop_assert!(is_pub(&s.vis)),
            1 => prop_assert!(is_inherited(&s.vis)),
            2 => prop_assert!(is_restricted(&s.vis)),
            _ => unreachable!(),
        }
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    }
}

// ── 28. Module visibility does not affect item visibility ───────────────────

proptest! {
    #[test]
    fn module_vis_does_not_affect_items(mod_pub in proptest::bool::ANY) {
        let m: ItemMod = if mod_pub {
            syn::parse2(quote::quote! {
                pub mod grammar {
                    struct Private { v: i32 }
                    pub struct Public { v: i32 }
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                mod grammar {
                    struct Private { v: i32 }
                    pub struct Public { v: i32 }
                }
            }).unwrap()
        };
        if mod_pub {
            prop_assert!(is_pub(&m.vis));
        } else {
            prop_assert!(is_inherited(&m.vis));
        }
        let (_, content) = m.content.unwrap();
        if let syn::Item::Struct(s) = &content[0] {
            prop_assert!(is_inherited(&s.vis));
        }
        if let syn::Item::Struct(s) = &content[1] {
            prop_assert!(is_pub(&s.vis));
        }
    }
}

// ── 29. Grammar module with pub(crate) items ────────────────────────────────

proptest! {
    #[test]
    fn grammar_module_pub_crate_items(count in 1usize..=4) {
        let items: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
                quote::quote! { pub(crate) struct #name { value: i32 } }
            })
            .collect();
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar { #(#items)* }
        }).unwrap();
        let (_, content) = m.content.unwrap();
        for item in &content {
            if let syn::Item::Struct(s) = item {
                prop_assert!(is_restricted(&s.vis));
            }
        }
    }
}

// ── 30. Pub field with leaf annotation ──────────────────────────────────────

proptest! {
    #[test]
    fn pub_field_with_leaf_annotation(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    pub #name: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_pub(&field.vis));
            prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 31. Pub(crate) field with skip annotation ───────────────────────────────

proptest! {
    #[test]
    fn pub_crate_field_with_skip_annotation(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("meta_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    pub(crate) #name: bool
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in &s.fields {
            prop_assert!(is_restricted(&field.vis));
            prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        }
    }
}

// ── 32. Enum variant fields inherit no extra visibility ─────────────────────

proptest! {
    #[test]
    fn enum_variant_fields_no_vis(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V { #(#fields),* }
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            for field in &n.named {
                prop_assert!(is_inherited(&field.vis));
            }
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 33. Visibility string round-trips correctly ─────────────────────────────

proptest! {
    #[test]
    fn visibility_string_roundtrip(idx in 0usize..=3) {
        let vis_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { pub },
            quote::quote! {},
            quote::quote! { pub(crate) },
            quote::quote! { pub(super) },
        ];
        let expected_contains = ["pub", "", "crate", "super"];
        let vis_tok = &vis_tokens[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #vis_tok struct S { value: i32 }
        }).unwrap();
        let vis_str = vis_to_string(&s.vis);
        if idx == 0 {
            prop_assert_eq!(vis_str.trim(), "pub");
        } else if idx == 1 {
            prop_assert!(vis_str.is_empty());
        } else {
            prop_assert!(vis_str.contains(expected_contains[idx]),
                "Expected '{}' in '{}'", expected_contains[idx], vis_str);
        }
    }
}

// ── 34. All items in module keep their original visibility ──────────────────

proptest! {
    #[test]
    fn all_items_keep_original_visibility(n_items in 2usize..=5) {
        let mut items: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut expected_pub: Vec<bool> = Vec::new();
        for i in 0..n_items {
            let name = syn::Ident::new(&format!("Item{i}"), proc_macro2::Span::call_site());
            let make_pub = i % 2 == 0;
            expected_pub.push(make_pub);
            if make_pub {
                items.push(quote::quote! { pub struct #name { val: i32 } });
            } else {
                items.push(quote::quote! { struct #name { val: i32 } });
            }
        }
        let m: ItemMod = syn::parse2(quote::quote! {
            mod grammar { #(#items)* }
        }).unwrap();
        let (_, content) = m.content.unwrap();
        prop_assert_eq!(content.len(), n_items);
        for i in 0..n_items {
            if let syn::Item::Struct(s) = &content[i] {
                if expected_pub[i] {
                    prop_assert!(is_pub(&s.vis),
                        "Item {} should be pub", i);
                } else {
                    prop_assert!(is_inherited(&s.vis),
                        "Item {} should be private", i);
                }
            }
        }
    }
}

// ── 35. pub(in path) struct visibility ──────────────────────────────────────

proptest! {
    #[test]
    fn pub_in_path_struct_visibility(field_count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub(in crate::outer) struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(is_restricted(&s.vis));
        let vis_str = vis_to_string(&s.vis);
        prop_assert!(vis_str.contains("crate"), "Expected path-restricted vis, got: {}", vis_str);
        prop_assert_eq!(s.fields.len(), field_count);
    }
}
