#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::extra]` attribute handling in adze-macro.
//!
//! Uses proptest to verify extra attribute parsing, struct/enum application,
//! pattern handling, whitespace/comment extras, multi-extra grammars,
//! combined annotations, and deterministic attribute output.

use adze_common::NameValueExpr;
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token};

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

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
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

fn count_extras_in_module(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|i| match i {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
            Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "extra")),
            _ => false,
        })
        .count()
}

fn extra_struct_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            {
                return Some(s.ident.to_string());
            }
            None
        })
        .collect()
}

// ── 1. Extra attribute parsed on struct with varying patterns ────────────────

proptest! {
    #[test]
    fn extra_attr_parsed_on_struct(idx in 0usize..=4) {
        let patterns = [r"\s", r"\s+", r"\n", r"\r?\n", r"[ \t]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let names = adze_attr_names(&s.attrs);
        prop_assert_eq!(names, vec!["extra".to_string()]);
    }
}

// ── 2. Extra attribute is always path-style meta (no args) ──────────────────

proptest! {
    #[test]
    fn extra_attr_always_path_style(idx in 0usize..=3) {
        let patterns = [r"\s", r"\n", r"\t", r"[ \t]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
        prop_assert!(matches!(extra_attr.meta, syn::Meta::Path(_)));
    }
}

// ── 3. Extra on struct preserves struct name ─────────────────────────────────

proptest! {
    #[test]
    fn extra_on_struct_preserves_name(idx in 0usize..=5) {
        let names = ["Whitespace", "Ws", "Skip", "Blank", "Newline", "Comment"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct #ident {
                #[adze::leaf(pattern = r"\s")]
                _f: (),
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), names[idx]);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    }
}

// ── 4. Extra with leaf pattern value preserved exactly ──────────────────────

proptest! {
    #[test]
    fn extra_leaf_pattern_preserved(idx in 0usize..=5) {
        let patterns = [r"\s", r"\s+", r"\n+", r"[ \t]+", r"\r?\n", r"\s|\n"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 5. Extra with leaf text literal preserved ───────────────────────────────

proptest! {
    #[test]
    fn extra_leaf_text_preserved(idx in 0usize..=3) {
        let texts = [" ", "\t", "\n", "  "];
        let txt = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct SpaceToken {
                #[adze::leaf(text = #txt)]
                _sp: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let value = extract_leaf_text(attr);
        prop_assert_eq!(value, txt);
    }
}

// ── 6. Extra on struct field type is unit ────────────────────────────────────

proptest! {
    #[test]
    fn extra_field_type_always_unit(idx in 0usize..=3) {
        let patterns = [r"\s", r"\n", r"\t", r"\r\n"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ty.to_token_stream().to_string(), "()");
    }
}

// ── 7. Extra on unit struct ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn extra_on_unit_struct(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            #[adze::leaf(pattern = #pat)]
            struct Ws;
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert!(matches!(s.fields, Fields::Unit));
    }
}

// ── 8. Extra on tuple struct ────────────────────────────────────────────────

proptest! {
    #[test]
    fn extra_on_tuple_struct(idx in 0usize..=2) {
        let patterns = [r"\s", r"\s+", r"\n"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws(
                #[adze::leaf(pattern = #pat)]
                ()
            );
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert!(matches!(s.fields, Fields::Unnamed(_)));
        if let Fields::Unnamed(ref u) = s.fields {
            prop_assert_eq!(u.unnamed.len(), 1);
        }
    }
}

// ── 9. Extra on enum with varying variant counts ────────────────────────────

proptest! {
    #[test]
    fn extra_on_enum_variant_count(n_variants in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_variants)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let txt = format!("v{i}");
                quote::quote! {
                    #[adze::leaf(text = #txt)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::extra]
            pub enum SkipToken {
                #(#variant_tokens),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert_eq!(e.variants.len(), n_variants);
    }
}

// ── 10. Extra for whitespace patterns ───────────────────────────────────────

proptest! {
    #[test]
    fn extra_whitespace_patterns(idx in 0usize..=4) {
        let patterns = [r"\s", r"\s+", r"[ \t]+", r"[ \t\r\n]+", r"\p{White_Space}+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 11. Extra for comment patterns ──────────────────────────────────────────

proptest! {
    #[test]
    fn extra_comment_patterns(idx in 0usize..=4) {
        let patterns = [r"//[^\n]*", r"#[^\n]*", r";[^\n]*", r"--[^\n]*", r"/\*[^*]*\*/"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = #pat)]
                _c: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 12. Extra combined with leaf — both attrs present ───────────────────────

proptest! {
    #[test]
    fn extra_combined_with_leaf_on_unit_struct(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"[ \t]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            #[adze::leaf(pattern = #pat)]
            struct Ws;
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"extra".to_string()));
        prop_assert!(names.contains(&"leaf".to_string()));
        prop_assert_eq!(names.len(), 2);
    }
}

// ── 13. Multiple extras in grammar ──────────────────────────────────────────

proptest! {
    #[test]
    fn multiple_extras_count(count in 1usize..=5) {
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
                let pat = format!(r"\s{{{}}}", i + 1);
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = #pat)]
                        _f: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), count);
    }
}

// ── 14. Multiple extras have distinct names ─────────────────────────────────

proptest! {
    #[test]
    fn multiple_extras_unique_names(count in 2usize..=5) {
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Ex{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = r"\s")]
                        _f: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        let names = extra_struct_names(&m);
        prop_assert_eq!(names.len(), count);
        let unique: std::collections::HashSet<_> = names.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 15. Extra coexists with language struct ──────────────────────────────────

proptest! {
    #[test]
    fn extra_coexists_with_language_struct(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"[ \t]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let has_language = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "language"))
            } else { false }
        });
        prop_assert!(has_language);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 16. Extra coexists with language enum ────────────────────────────────────

proptest! {
    #[test]
    fn extra_coexists_with_language_enum(idx in 0usize..=2) {
        let patterns = [r"\s", r"\s+", r"\n"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let has_lang_enum = items.iter().any(|i| {
            if let Item::Enum(e) = i { e.attrs.iter().any(|a| is_adze_attr(a, "language")) }
            else { false }
        });
        prop_assert!(has_lang_enum);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 17. Extra coexists with word annotation ─────────────────────────────────

proptest! {
    #[test]
    fn extra_coexists_with_word(idx in 0usize..=2) {
        let patterns = [r"\s", r"\s+", r"[ \t]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    ident: Identifier,
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        prop_assert!(has_word);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 18. Extra coexists with external annotation ─────────────────────────────

proptest! {
    #[test]
    fn extra_coexists_with_external(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
                #[adze::external]
                struct IndentToken;
            }
        });
        let items = module_items(&m);
        let has_external = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "external")) }
            else { false }
        });
        prop_assert!(has_external);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 19. Extra visibility variations ─────────────────────────────────────────

proptest! {
    #[test]
    fn extra_visibility_variations(vis_idx in 0usize..=2) {
        let s: ItemStruct = match vis_idx {
            0 => syn::parse2(quote::quote! {
                #[adze::extra]
                struct Ws { #[adze::leaf(pattern = r"\s")] _ws: (), }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::extra]
                pub struct Ws { #[adze::leaf(pattern = r"\s")] _ws: (), }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::extra]
                pub(crate) struct Ws { #[adze::leaf(pattern = r"\s")] _ws: (), }
            }).unwrap(),
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        match vis_idx {
            0 => prop_assert!(matches!(s.vis, syn::Visibility::Inherited)),
            1 => prop_assert!(matches!(s.vis, syn::Visibility::Public(_))),
            _ => prop_assert!(matches!(s.vis, syn::Visibility::Restricted(_))),
        }
    }
}

// ── 20. Extra preserves non-adze attributes ─────────────────────────────────

proptest! {
    #[test]
    fn extra_preserves_derive_attrs(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(Debug)]
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert_eq!(s.attrs.len(), 2);
        let adze_names = adze_attr_names(&s.attrs);
        prop_assert_eq!(adze_names, vec!["extra".to_string()]);
        let has_derive = s.attrs.iter().any(|a| {
            a.path().segments.iter().next().map(|s| s.ident == "derive").unwrap_or(false)
        });
        prop_assert!(has_derive);
    }
}

// ── 21. Extra with complex alternation regex ────────────────────────────────

proptest! {
    #[test]
    fn extra_complex_alternation_pattern(idx in 0usize..=2) {
        let patterns = [
            r"\s+|//[^\n]*",
            r"\s+|//[^\n]*|/\*[^*]*\*/",
            r"[ \t]+|\r?\n",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Skip {
                #[adze::leaf(pattern = #pat)]
                _s: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert!(extracted.contains('|'));
        prop_assert_eq!(extracted, pat);
    }
}

// ── 22. Extra with Unicode whitespace patterns ──────────────────────────────

proptest! {
    #[test]
    fn extra_unicode_whitespace_patterns(idx in 0usize..=2) {
        let patterns = [
            r"[\s\u{00A0}]+",
            r"[\s\u{2003}]+",
            r"[\s\u{00A0}\u{2003}]+",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct UniWs {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 23. Extra attr path has exactly two segments ────────────────────────────

proptest! {
    #[test]
    fn extra_attr_path_two_segments(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
        let segs: Vec<_> = extra_attr.path().segments.iter().collect();
        prop_assert_eq!(segs.len(), 2);
        prop_assert_eq!(segs[0].ident.to_string(), "adze");
        prop_assert_eq!(segs[1].ident.to_string(), "extra");
    }
}

// ── 24. Extra appears exactly once per struct ───────────────────────────────

proptest! {
    #[test]
    fn extra_appears_once_per_struct(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let extra_count = s.attrs.iter().filter(|a| is_adze_attr(a, "extra")).count();
        prop_assert_eq!(extra_count, 1);
    }
}

// ── 25. Extra ordering independence — extras before vs after language ────────

proptest! {
    #[test]
    fn extra_ordering_independence(idx in 0usize..=2) {
        let extra_name_sets = [
            vec!["Comment", "Ws"],
            vec!["Ws"],
            vec!["Newline", "Ws", "Comment"],
        ];
        let chosen = &extra_name_sets[idx];
        let extra_tokens: Vec<proc_macro2::TokenStream> = chosen.iter().map(|n| {
            let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::extra]
                struct #ident {
                    #[adze::leaf(pattern = r"\s")]
                    _f: (),
                }
            }
        }).collect();
        let m_after = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        let m_before = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#extra_tokens)*
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m_after), chosen.len());
        prop_assert_eq!(count_extras_in_module(&m_before), chosen.len());
        let set_a: std::collections::HashSet<_> = extra_struct_names(&m_after).into_iter().collect();
        let set_b: std::collections::HashSet<_> = extra_struct_names(&m_before).into_iter().collect();
        prop_assert_eq!(set_a, set_b);
    }
}

// ── 26. Extra determinism — same input always yields same output ────────────

proptest! {
    #[test]
    fn extra_attr_deterministic(idx in 0usize..=3) {
        let patterns = [r"\s", r"\s+", r"\n", r"[ \t]+"];
        let pat = patterns[idx];
        let tokens = quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        };
        let s1: ItemStruct = syn::parse2(tokens.clone()).unwrap();
        let s2: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert_eq!(s1.to_token_stream().to_string(), s2.to_token_stream().to_string());
    }
}

// ── 27. Extra in grammar with precedence operators ──────────────────────────

proptest! {
    #[test]
    fn extra_with_precedence_operators(prec_level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec_level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("arith")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let has_prec = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.variants.iter().any(|v| v.attrs.iter().any(|a| is_adze_attr(a, "prec_left")))
            } else { false }
        });
        prop_assert!(has_prec);
    }
}

// ── 28. Extra distinct from language struct names ────────────────────────────

proptest! {
    #[test]
    fn extra_distinct_from_language(idx in 0usize..=2) {
        let extra_names = ["Whitespace", "Blank", "Skip"];
        let ename = syn::Ident::new(extra_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #[adze::extra]
                struct #ename {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let lang_names: Vec<_> = items.iter().filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "language"))
            {
                return Some(s.ident.to_string());
            }
            None
        }).collect();
        let extras = extra_struct_names(&m);
        prop_assert_eq!(&lang_names, &vec!["Program".to_string()]);
        prop_assert_eq!(&extras, &vec![extra_names[idx].to_string()]);
        for name in &extras {
            prop_assert!(!lang_names.contains(name));
        }
    }
}

// ── 29. Extra in module with repeat fields ──────────────────────────────────

proptest! {
    #[test]
    fn extra_in_module_with_repeat(idx in 0usize..=1) {
        let patterns = [r"\s", r"\s+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct NumberList {
                    numbers: Vec<Number>,
                }
                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 30. Extra in module with delimited fields ───────────────────────────────

proptest! {
    #[test]
    fn extra_in_module_with_delimited(idx in 0usize..=1) {
        let patterns = [r"\s", r"\n"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct NumberList {
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    numbers: Vec<Number>,
                }
                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let lang = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        }).unwrap();
        let field = lang.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    }
}

// ── 31. Extra with all annotation types coexisting ──────────────────────────

proptest! {
    #[test]
    fn extra_with_all_annotation_types(extra_count in 1usize..=3) {
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Ex{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = r"\s")]
                        _f: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("full")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Ident(Identifier),
                    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
                #(#extra_tokens)*
                #[adze::external]
                struct IndentToken;
            }
        });
        let items = module_items(&m);
        let mut found = std::collections::HashMap::new();
        for item in items {
            match item {
                Item::Struct(s) => {
                    for attr_name in &["language", "word", "extra", "external"] {
                        if s.attrs.iter().any(|a| is_adze_attr(a, attr_name)) {
                            *found.entry(attr_name.to_string()).or_insert(0usize) += 1;
                        }
                    }
                }
                Item::Enum(e) => {
                    if e.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                        *found.entry("language".to_string()).or_insert(0usize) += 1;
                    }
                }
                _ => {}
            }
        }
        prop_assert_eq!(found.get("language"), Some(&1));
        prop_assert_eq!(found.get("word"), Some(&1));
        prop_assert_eq!(found.get("extra"), Some(&extra_count));
        prop_assert_eq!(found.get("external"), Some(&1));
    }
}

// ── 32. Extra struct has exactly one leaf field ─────────────────────────────

proptest! {
    #[test]
    fn extra_struct_single_leaf_field(idx in 0usize..=3) {
        let patterns = [r"\s", r"\n", r"[ \t]", r"\s+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert_eq!(s.fields.iter().count(), 1);
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 33. Extra module determinism — same grammar produces same AST ───────────

proptest! {
    #[test]
    fn extra_module_deterministic(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let make_mod = || {
            parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Code {
                        #[adze::leaf(pattern = r"\w+")]
                        token: String,
                    }
                    #[adze::extra]
                    struct Ws {
                        #[adze::leaf(pattern = #pat)]
                        _ws: (),
                    }
                }
            })
        };
        let m1 = make_mod();
        let m2 = make_mod();
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
    }
}

// ── 34. Extra between other types preserves detection ───────────────────────

proptest! {
    #[test]
    fn extra_between_other_types(idx in 0usize..=1) {
        let patterns = [r"\s", r"\s+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    child: Child,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
                pub struct Child {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let extras = extra_struct_names(&m);
        prop_assert_eq!(&extras[0], "Ws");
    }
}

// ── 35. Extra with empty pattern (edge case) ────────────────────────────────

proptest! {
    #[test]
    fn extra_with_empty_pattern_edge_case(idx in 0usize..=1) {
        let patterns = ["", " "];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct EmptyExtra {
                #[adze::leaf(pattern = #pat)]
                _empty: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 36. Extra struct with tab-only whitespace patterns ──────────────────────

proptest! {
    #[test]
    fn extra_tab_only_whitespace(idx in 0usize..=2) {
        let patterns = [r"\t", r"\t+", r"\t{1,4}"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct TabWs {
                #[adze::leaf(pattern = #pat)]
                _tab: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert_eq!(s.ident.to_string(), "TabWs");
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 37. Extra struct with carriage-return patterns ──────────────────────────

proptest! {
    #[test]
    fn extra_carriage_return_patterns(idx in 0usize..=2) {
        let patterns = [r"\r", r"\r\n", r"\r?\n"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct CrLf {
                #[adze::leaf(pattern = #pat)]
                _cr: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 38. Extra produces extras list — count in various module sizes ───────────

proptest! {
    #[test]
    fn extra_list_scales_with_count(count in 1usize..=6) {
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Skip{i}"), proc_macro2::Span::call_site());
                let pat = format!(r"\s{{{}}}", i + 1);
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = #pat)]
                        _f: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        let names = extra_struct_names(&m);
        prop_assert_eq!(names.len(), count);
        for i in 0..count {
            prop_assert_eq!(&names[i], &format!("Skip{i}"));
        }
    }
}

// ── 39. Extra list preserves insertion order ────────────────────────────────

proptest! {
    #[test]
    fn extra_list_preserves_order(idx in 0usize..=2) {
        let orderings: &[&[&str]] = &[
            &["Alpha", "Beta", "Gamma"],
            &["Gamma", "Alpha", "Beta"],
            &["Beta", "Gamma", "Alpha"],
        ];
        let chosen = orderings[idx];
        let extra_tokens: Vec<proc_macro2::TokenStream> = chosen.iter().map(|n| {
            let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::extra]
                struct #ident {
                    #[adze::leaf(pattern = r"\s")]
                    _f: (),
                }
            }
        }).collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        let names = extra_struct_names(&m);
        for i in 0..chosen.len() {
            prop_assert_eq!(&names[i], chosen[i]);
        }
    }
}

// ── 40. Multiple extras with mixed ws + comment types ───────────────────────

proptest! {
    #[test]
    fn extra_mixed_ws_and_comments(idx in 0usize..=2) {
        let ws_pats = [r"\s+", r"[ \t]+", r"\n+"];
        let comment_pats = [r"//[^\n]*", r"#[^\n]*", r"--[^\n]*"];
        let ws_pat = ws_pats[idx];
        let cmt_pat = comment_pats[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #ws_pat)]
                    _ws: (),
                }
                #[adze::extra]
                struct Comment {
                    #[adze::leaf(pattern = #cmt_pat)]
                    _c: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 2);
        let names = extra_struct_names(&m);
        prop_assert!(names.contains(&"Ws".to_string()));
        prop_assert!(names.contains(&"Comment".to_string()));
    }
}

// ── 41. Extra with doc-comment regex patterns ───────────────────────────────

proptest! {
    #[test]
    fn extra_docstring_patterns(idx in 0usize..=2) {
        let patterns = [
            r#"///[^\n]*"#,
            r#"//![^\n]*"#,
            r#"/\*\*[^*]*\*/"#,
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct DocComment {
                #[adze::leaf(pattern = #pat)]
                _doc: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 42. Extra with shell/script comment patterns ────────────────────────────

proptest! {
    #[test]
    fn extra_shell_comment_patterns(idx in 0usize..=2) {
        let patterns = [r"#[^\n]*", r"rem\s[^\n]*", r"%[^\n]*"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct ShellComment {
                #[adze::leaf(pattern = #pat)]
                _c: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 43. Extra interaction with optional fields in grammar ───────────────────

proptest! {
    #[test]
    fn extra_with_optional_fields_grammar(idx in 0usize..=1) {
        let patterns = [r"\s", r"\s+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: Option<i32>,
                    child: Option<Child>,
                }
                pub struct Child {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let lang = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        }).unwrap();
        prop_assert_eq!(lang.fields.iter().count(), 2);
    }
}

// ── 44. Extra interaction with boxed recursive grammar ──────────────────────

proptest! {
    #[test]
    fn extra_with_recursive_grammar(idx in 0usize..=1) {
        let patterns = [r"\s", r"\n"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                    Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let has_lang_enum = items.iter().any(|i| {
            if let Item::Enum(e) = i { e.attrs.iter().any(|a| is_adze_attr(a, "language")) }
            else { false }
        });
        prop_assert!(has_lang_enum);
    }
}

// ── 45. Extra on enum variant with named fields ─────────────────────────────

proptest! {
    #[test]
    fn extra_enum_variant_named_fields(idx in 0usize..=2) {
        let texts = ["ws0", "ws1", "ws2"];
        let txt = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::extra]
            pub enum SkipToken {
                Ws {
                    #[adze::leaf(text = #txt)]
                    _ws: (),
                },
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let variant = e.variants.first().unwrap();
        prop_assert!(matches!(variant.fields, Fields::Named(_)));
    }
}

// ── 46. Extra on enum variant with tuple fields ─────────────────────────────

proptest! {
    #[test]
    fn extra_enum_variant_tuple_fields(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::extra]
            pub enum SkipToken {
                Ws(#[adze::leaf(pattern = #pat)] ()),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let variant = e.variants.first().unwrap();
        prop_assert!(matches!(variant.fields, Fields::Unnamed(_)));
    }
}

// ── 47. Extra enum with mixed unit and tuple variants ───────────────────────

proptest! {
    #[test]
    fn extra_enum_mixed_variant_kinds(idx in 0usize..=1) {
        let patterns = [r"\s", r"\n"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::extra]
            pub enum SkipToken {
                #[adze::leaf(text = " ")]
                Space,
                Tab(#[adze::leaf(pattern = #pat)] ()),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert_eq!(e.variants.len(), 2);
        prop_assert!(matches!(e.variants[0].fields, Fields::Unit));
        prop_assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    }
}

// ── 48. Extra struct fields underscore naming convention ─────────────────────

proptest! {
    #[test]
    fn extra_field_underscore_naming(idx in 0usize..=3) {
        let field_names = ["_ws", "_skip", "_blank", "_space"];
        let fname = syn::Ident::new(field_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                #fname: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), field_names[idx]);
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 49. Extra validation — extra attr has no arguments ──────────────────────

proptest! {
    #[test]
    fn extra_attr_has_no_args(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
        // extra should be a simple path with no parenthesized or key-value args
        let ts = extra_attr.meta.to_token_stream().to_string();
        prop_assert!(!ts.contains('('));
        prop_assert!(!ts.contains('='));
    }
}

// ── 50. Extra validation — no duplicate extra attrs on same struct ───────────

proptest! {
    #[test]
    fn extra_no_duplicate_attr_on_struct(idx in 0usize..=2) {
        let names = ["Ws", "Comment", "Newline"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct #ident {
                #[adze::leaf(pattern = r"\s")]
                _f: (),
            }
        }).unwrap();
        let extra_count = s.attrs.iter().filter(|a| is_adze_attr(a, "extra")).count();
        prop_assert_eq!(extra_count, 1, "Extra should appear exactly once");
    }
}

// ── 51. Extra validation — extra attr style is outer ────────────────────────

proptest! {
    #[test]
    fn extra_attr_is_outer_style(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
        prop_assert!(matches!(extra_attr.style, syn::AttrStyle::Outer));
    }
}

// ── 52. Extra validation — extra on enum is also outer ──────────────────────

proptest! {
    #[test]
    fn extra_attr_on_enum_is_outer(idx in 0usize..=1) {
        let texts = ["space", "tab"];
        let txt = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::extra]
            pub enum SkipToken {
                #[adze::leaf(text = #txt)]
                Token,
            }
        }).unwrap();
        let extra_attr = e.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
        prop_assert!(matches!(extra_attr.style, syn::AttrStyle::Outer));
    }
}

// ── 53. Extra expansion determinism — multiple extras produce stable output ─

proptest! {
    #[test]
    fn extra_multi_deterministic(idx in 0usize..=2) {
        let counts = [2, 3, 4];
        let count = counts[idx];
        let make_mod = || {
            let extra_tokens: Vec<proc_macro2::TokenStream> = (0..count)
                .map(|i| {
                    let name = syn::Ident::new(&format!("Ex{i}"), proc_macro2::Span::call_site());
                    quote::quote! {
                        #[adze::extra]
                        struct #name {
                            #[adze::leaf(pattern = r"\s")]
                            _f: (),
                        }
                    }
                })
                .collect();
            parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub struct Code {
                        #[adze::leaf(pattern = r"\w+")]
                        token: String,
                    }
                    #(#extra_tokens)*
                }
            })
        };
        let m1 = make_mod();
        let m2 = make_mod();
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
        let n1 = extra_struct_names(&m1);
        let n2 = extra_struct_names(&m2);
        prop_assert_eq!(n1, n2);
    }
}

// ── 54. Extra expansion determinism — enum extras stable ────────────────────

proptest! {
    #[test]
    fn extra_enum_deterministic(idx in 0usize..=1) {
        let variant_counts = [2, 3];
        let vc = variant_counts[idx];
        let make_enum = || {
            let variants: Vec<proc_macro2::TokenStream> = (0..vc)
                .map(|i| {
                    let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                    let txt = format!("v{i}");
                    quote::quote! { #[adze::leaf(text = #txt)] #name }
                })
                .collect();
            let e: ItemEnum = syn::parse2(quote::quote! {
                #[adze::extra]
                pub enum SkipToken { #(#variants),* }
            }).unwrap();
            e
        };
        let e1 = make_enum();
        let e2 = make_enum();
        prop_assert_eq!(
            e1.to_token_stream().to_string(),
            e2.to_token_stream().to_string()
        );
    }
}

// ── 55. Extra with multiple annotation types on same module — stable ────────

proptest! {
    #[test]
    fn extra_full_grammar_deterministic(idx in 0usize..=1) {
        let patterns = [r"\s", r"\n"];
        let pat = patterns[idx];
        let make = || {
            parse_mod(quote::quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub enum Expr {
                        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                        #[adze::prec_left(1)]
                        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    }
                    #[adze::extra]
                    struct Ws {
                        #[adze::leaf(pattern = #pat)]
                        _ws: (),
                    }
                }
            })
        };
        let m1 = make();
        let m2 = make();
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
    }
}

// ── 56. Extra interaction — extra does not affect language field count ───────

proptest! {
    #[test]
    fn extra_does_not_affect_language_fields(idx in 0usize..=2) {
        let extra_counts = [0, 1, 3];
        let ec = extra_counts[idx];
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..ec)
            .map(|i| {
                let name = syn::Ident::new(&format!("Ex{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::extra]
                    struct #name {
                        #[adze::leaf(pattern = r"\s")]
                        _f: (),
                    }
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
                #(#extra_tokens)*
            }
        });
        let items = module_items(&m);
        let lang = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        }).unwrap();
        prop_assert_eq!(lang.fields.iter().count(), 1);
        prop_assert_eq!(count_extras_in_module(&m), ec);
    }
}

// ── 57. Extra interaction — extra does not affect enum variant count ─────────

proptest! {
    #[test]
    fn extra_does_not_affect_enum_variants(idx in 0usize..=1) {
        let variant_counts = [2, 3];
        let vc = variant_counts[idx];
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..vc)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let txt = format!("v{i}");
                quote::quote! { #[adze::leaf(text = #txt)] #name }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Token { #(#variant_tokens),* }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let lang_enum = items.iter().find_map(|i| {
            if let Item::Enum(e) = i {
                if e.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(e) }
                else { None }
            } else { None }
        }).unwrap();
        prop_assert_eq!(lang_enum.variants.len(), vc);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 58. Extra with skip field coexisting ────────────────────────────────────

proptest! {
    #[test]
    fn extra_with_skip_field_grammar(idx in 0usize..=1) {
        let patterns = [r"\s", r"\s+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct MyNode {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    value: i32,
                    #[adze::skip(false)]
                    visited: bool,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let lang = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        }).unwrap();
        let has_skip = lang.fields.iter().any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert!(has_skip);
    }
}

// ── 59. Extra struct field name is preserved ────────────────────────────────

proptest! {
    #[test]
    fn extra_struct_field_name_preserved(idx in 0usize..=3) {
        let field_names = ["_whitespace", "_ws", "_comment", "_newline"];
        let fname = syn::Ident::new(field_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                #fname: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let actual = field.ident.as_ref().unwrap().to_string();
        prop_assert_eq!(actual, field_names[idx]);
    }
}

// ── 60. Extra attr absent means not extra ───────────────────────────────────

proptest! {
    #[test]
    fn no_extra_attr_means_regular_struct(idx in 0usize..=2) {
        let names = ["Helper", "Aux", "NonExtra"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                _f: String,
            }
        }).unwrap();
        let extra_count = s.attrs.iter().filter(|a| is_adze_attr(a, "extra")).count();
        prop_assert_eq!(extra_count, 0);
    }
}
