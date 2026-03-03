#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::extra]` annotation in adze-macro.
//!
//! Uses proptest to generate randomized patterns, struct names, field counts,
//! and annotation combinations, then verifies that syn correctly parses and
//! preserves the extra attribute and its interaction with other grammar
//! annotations.

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

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
}

fn extract_leaf_pattern(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path.to_string() == "pattern").unwrap();
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
    let nv = params.iter().find(|p| p.path.to_string() == "text").unwrap();
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
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                    return Some(s.ident.to_string());
                }
            }
            None
        })
        .collect()
}

// ── 1. Extra annotation detected on struct with random pattern ──────────────

proptest! {
    #[test]
    fn extra_annotation_detected_on_struct(idx in 0usize..=4) {
        let patterns = [r"\s", r"\s+", r"\n", r"\r?\n", r"[ \t]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"extra".to_string()));
        prop_assert_eq!(names.len(), 1);
    }
}

// ── 2. Extra with text pattern (literal match) ─────────────────────────────

proptest! {
    #[test]
    fn extra_with_text_literal(idx in 0usize..=3) {
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

// ── 3. Extra with regex pattern preserved exactly ───────────────────────────

proptest! {
    #[test]
    fn extra_regex_pattern_preserved(idx in 0usize..=5) {
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

// ── 4. Multiple extras in grammar with random count ─────────────────────────

proptest! {
    #[test]
    fn multiple_extras_random_count(count in 1usize..=4) {
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

// ── 5. Extra coexists with language struct ──────────────────────────────────

proptest! {
    #[test]
    fn extra_coexists_with_language(idx in 0usize..=2) {
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
        let has_extra = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else { false }
        });
        prop_assert!(has_language);
        prop_assert!(has_extra);
    }
}

// ── 6. Extra coexists with language enum ────────────────────────────────────

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

// ── 7. Extra type detection in module among mixed items ─────────────────────

proptest! {
    #[test]
    fn extra_type_detection_in_module(n_regular in 1usize..=3) {
        let regular_tokens: Vec<proc_macro2::TokenStream> = (0..n_regular)
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
                    token: String,
                }
                #(#regular_tokens)*
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let extras = extra_struct_names(&m);
        prop_assert_eq!(extras.len(), 1);
        prop_assert_eq!(&extras[0], "Ws");
    }
}

// ── 8. Extra ordering independence — extras before language ──────────────────

proptest! {
    #[test]
    fn extra_before_language(idx in 0usize..=2) {
        let names = ["Whitespace", "Newline", "Comment"];
        let pats = [r"\s", r"\n", r"//[^\n]*"];
        let ename = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let pat = pats[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::extra]
                struct #ename {
                    #[adze::leaf(pattern = #pat)]
                    _f: (),
                }
                #[adze::language]
                pub struct Code {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                }
            }
        });
        prop_assert_eq!(count_extras_in_module(&m), 1);
        let items = module_items(&m);
        let has_lang = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "language")) }
            else { false }
        });
        prop_assert!(has_lang);
    }
}

// ── 9. Extra ordering — extras between other types ──────────────────────────

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

// ── 10. Extra struct name varies ────────────────────────────────────────────

proptest! {
    #[test]
    fn extra_struct_name_preserved(idx in 0usize..=4) {
        let names = ["Whitespace", "Ws", "Skip", "Blank", "Ignore"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct #ident {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), names[idx]);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    }
}

// ── 11. Extra on enum parses correctly ──────────────────────────────────────

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

// ── 12. Extra attr is path-style (no arguments) ─────────────────────────────

proptest! {
    #[test]
    fn extra_attr_is_path_style(idx in 0usize..=2) {
        let patterns = [r"\s", r"\n", r"[ \t]"];
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

// ── 13. Extra field type is always unit ──────────────────────────────────────

proptest! {
    #[test]
    fn extra_field_type_is_unit(idx in 0usize..=3) {
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

// ── 14. Extra with comment patterns ─────────────────────────────────────────

proptest! {
    #[test]
    fn extra_with_comment_patterns(idx in 0usize..=3) {
        let patterns = [r"//[^\n]*", r"#[^\n]*", r";[^\n]*", r"--[^\n]*"];
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

// ── 15. Extra preserves non-adze attributes ─────────────────────────────────

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

// ── 16. Extra coexists with word annotation ─────────────────────────────────

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

// ── 17. Extra coexists with external annotation ─────────────────────────────

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

// ── 18. Extra on unit struct ────────────────────────────────────────────────

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

// ── 19. Extra on tuple struct ───────────────────────────────────────────────

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
    }
}

// ── 20. Extra visibility variations ─────────────────────────────────────────

proptest! {
    #[test]
    fn extra_visibility_variants(vis_idx in 0usize..=2) {
        // Test three visibility forms: private, pub, pub(crate)
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

// ── 22. Extra in grammar with precedence operators ──────────────────────────

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
        let items = module_items(&m);
        let has_prec = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.variants.iter().any(|v| v.attrs.iter().any(|a| is_adze_attr(a, "prec_left")))
            } else { false }
        });
        prop_assert!(has_prec);
        prop_assert_eq!(count_extras_in_module(&m), 1);
    }
}

// ── 23. Multiple extras have distinct names ─────────────────────────────────

proptest! {
    #[test]
    fn multiple_extras_distinct_names(count in 2usize..=5) {
        let extra_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Extra{i}"), proc_macro2::Span::call_site());
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
        // All names are unique
        let unique: std::collections::HashSet<_> = names.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 24. Extra appears exactly once per struct ───────────────────────────────

proptest! {
    #[test]
    fn extra_appears_exactly_once(idx in 0usize..=2) {
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

// ── 25. Extra ordering independence — reversed module order ─────────────────

proptest! {
    #[test]
    fn extra_ordering_reversed(idx in 0usize..=2) {
        let extra_names_list = [
            vec!["Comment", "Ws"],
            vec!["Ws"],
            vec!["Newline", "Ws", "Comment"],
        ];
        let chosen = &extra_names_list[idx];
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
        // Extras placed after language
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
        // Extras placed before language
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
        let names_after = extra_struct_names(&m_after);
        let names_before = extra_struct_names(&m_before);
        // Same set of extra names regardless of position
        let set_a: std::collections::HashSet<_> = names_after.iter().collect();
        let set_b: std::collections::HashSet<_> = names_before.iter().collect();
        prop_assert_eq!(set_a, set_b);
    }
}

// ── 26. Extra with all annotation types coexisting ──────────────────────────

proptest! {
    #[test]
    fn extra_with_all_annotations(extra_count in 1usize..=3) {
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

// ── 27. Extra struct single field count ─────────────────────────────────────

proptest! {
    #[test]
    fn extra_struct_has_one_field(idx in 0usize..=3) {
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
    }
}

// ── 28. Extra with Unicode whitespace patterns ──────────────────────────────

proptest! {
    #[test]
    fn extra_unicode_patterns(idx in 0usize..=2) {
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

// ── 31. Extra attr path segments correct ────────────────────────────────────

proptest! {
    #[test]
    fn extra_attr_path_segments(idx in 0usize..=2) {
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

// ── 32. Extra distinct from language in mixed module ────────────────────────

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
        let language_names: Vec<_> = items.iter().filter_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                    return Some(s.ident.to_string());
                }
            }
            None
        }).collect();
        let extras = extra_struct_names(&m);
        prop_assert_eq!(&language_names, &vec!["Program".to_string()]);
        prop_assert_eq!(&extras, &vec![extra_names[idx].to_string()]);
        // No overlap
        for name in &extras {
            prop_assert!(!language_names.contains(name));
        }
    }
}
