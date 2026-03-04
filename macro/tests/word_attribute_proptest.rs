#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::word]` attribute handling in adze-macro.
//!
//! Covers word attribute parsing, word on enum variants, word with regex pattern,
//! word combined with other attributes, multiple word variants, word attribute
//! preserved in expansion, word vs leaf distinction, and word attribute determinism.

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

// ── 1. Word attribute parsing: meta is always Path (no args) ────────────────

proptest! {
    #[test]
    fn word_attr_meta_is_path_style(idx in 0usize..=3) {
        let struct_names = ["Ident", "Token", "Word", "Name"];
        let name = syn::Ident::new(struct_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct #name {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }).unwrap();
        let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
        prop_assert!(matches!(word_attr.meta, syn::Meta::Path(_)),
            "word attribute must be path-style (no arguments)");
    }
}

// ── 2. Word on enum variants: variant count preserved ───────────────────────

proptest! {
    #[test]
    fn word_on_enum_preserves_variant_count(n in 2usize..=5) {
        let variants: Vec<proc_macro2::TokenStream> = (0..n)
            .map(|i| {
                let vname = syn::Ident::new(&format!("Kw{i}"), proc_macro2::Span::call_site());
                let txt = format!("keyword{i}");
                quote::quote! {
                    #[adze::leaf(text = #txt)]
                    #vname
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::word]
            pub enum Keywords {
                #(#variants),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert_eq!(e.variants.len(), n);
        for i in 0..n {
            let expected = format!("Kw{i}");
            prop_assert_eq!(e.variants[i].ident.to_string(), expected);
        }
    }
}

// ── 3. Word with regex pattern: value roundtrips exactly ────────────────────

proptest! {
    #[test]
    fn word_regex_pattern_roundtrips(idx in 0usize..=6) {
        let patterns = [
            r"[a-zA-Z_]\w*",
            r"[\p{L}_][\p{L}\p{N}_]*",
            r"[a-zA-Z$_][a-zA-Z0-9$_]*",
            r"[a-zA-Z_]{1,}[a-zA-Z0-9_]{0,255}",
            r"[a-zA-Z_][a-zA-Z0-9_\-]*",
            r"\w+",
            r"[a-zA-Z\u{00C0}-\u{024F}_]+",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 4. Word combined with language: both attrs present ──────────────────────

proptest! {
    #[test]
    fn word_combined_with_language_both_present(idx in 0usize..=1) {
        // idx 0 = word first, idx 1 = language first
        let s: ItemStruct = if idx == 0 {
            syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::language]
                #[adze::word]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }).unwrap()
        };
        let names = adze_attr_names(&s.attrs);
        prop_assert_eq!(names.len(), 2);
        prop_assert!(names.contains(&"word".to_string()));
        prop_assert!(names.contains(&"language".to_string()));
    }
}

// ── 5. Multiple word variants: each enum variant has leaf text ──────────────

proptest! {
    #[test]
    fn word_enum_variant_leaf_texts_preserved(n in 2usize..=4) {
        let keywords: Vec<&str> = vec!["if", "else", "while", "for"];
        let variants: Vec<proc_macro2::TokenStream> = (0..n)
            .map(|i| {
                let vname = syn::Ident::new(&format!("K{i}"), proc_macro2::Span::call_site());
                let kw = keywords[i];
                quote::quote! {
                    #[adze::leaf(text = #kw)]
                    #vname
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::word]
            pub enum KwEnum {
                #(#variants),*
            }
        }).unwrap();
        for i in 0..n {
            let v = &e.variants[i];
            let attr = v.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            let txt = extract_leaf_text(attr);
            prop_assert_eq!(txt, keywords[i]);
        }
    }
}

// ── 6. Word attribute preserved in expansion: survives parse_quote ───────────

proptest! {
    #[test]
    fn word_attr_survives_parse_roundtrip(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        // Re-serialize and re-parse
        let tokens = s.to_token_stream();
        let reparsed: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert!(reparsed.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = reparsed.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 7. Word vs leaf distinction: word is on type, leaf on field ─────────────

proptest! {
    #[test]
    fn word_on_type_leaf_on_field(idx in 0usize..=3) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        // word is a type-level attribute
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert!(!s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        // leaf is a field-level attribute
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 8. Word attribute determinism: same input always yields same output ──────

proptest! {
    #[test]
    fn word_parsing_is_deterministic(idx in 0usize..=4) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+", r"[A-Z]+", r"\w{1,32}"];
        let pat = patterns[idx];
        let make_struct = || -> ItemStruct {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }).unwrap()
        };
        let s1 = make_struct();
        let s2 = make_struct();
        prop_assert_eq!(s1.to_token_stream().to_string(), s2.to_token_stream().to_string());
    }
}

// ── 9. Word on enum with mixed variant styles ───────────────────────────────

proptest! {
    #[test]
    fn word_enum_mixed_variant_styles(idx in 0usize..=1) {
        // idx 0: unit + tuple, idx 1: unit + named
        let e: ItemEnum = if idx == 0 {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub enum Token {
                    #[adze::leaf(text = "if")]
                    If,
                    Ident(
                        #[adze::leaf(pattern = r"[a-z]+")]
                        String
                    ),
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub enum Token {
                    #[adze::leaf(text = "while")]
                    While,
                    Named {
                        #[adze::leaf(pattern = r"[a-z]+")]
                        value: String,
                    },
                }
            }).unwrap()
        };
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert_eq!(e.variants.len(), 2);
    }
}

// ── 10. Word struct field type preserved ─────────────────────────────────────

proptest! {
    #[test]
    fn word_field_type_preserved(idx in 0usize..=2) {
        // Test with different field types
        let s: ItemStruct = if idx == 0 {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub struct W {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap()
        } else if idx == 1 {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub struct W(
                    #[adze::leaf(pattern = r"\w+")]
                    String
                );
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::leaf(text = "kw")]
                pub struct W;
            }).unwrap()
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        match idx {
            0 => {
                let field = s.fields.iter().next().unwrap();
                prop_assert_eq!(field.ty.to_token_stream().to_string(), "String");
            }
            1 => {
                let field = s.fields.iter().next().unwrap();
                prop_assert_eq!(field.ty.to_token_stream().to_string(), "String");
            }
            _ => {
                prop_assert!(matches!(s.fields, Fields::Unit));
            }
        }
    }
}

// ── 11. Word does not appear on fields ──────────────────────────────────────

proptest! {
    #[test]
    fn word_never_on_fields(idx in 0usize..=2) {
        let patterns = [r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
                #[adze::skip(false)]
                visited: bool,
            }
        }).unwrap();
        for field in s.fields.iter() {
            prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "word")),
                "word must not appear as a field attribute");
        }
    }
}

// ── 12. Word in module: no interference with grammar expansion ──────────────

proptest! {
    #[test]
    fn word_module_grammar_name_preserved(idx in 0usize..=2) {
        let grammar_names = ["test_lang", "my_grammar", "word_grammar"];
        let gname = grammar_names[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#gname)]
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
        // Grammar name is on the module attr
        let grammar_attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar"));
        // After parse, the grammar attr should still be present on the raw module
        prop_assert!(grammar_attr.is_some());
        // word struct should exist
        let has_word = module_items(&m).iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        prop_assert!(has_word);
    }
}

// ── 13. Word with pattern containing quantifiers ────────────────────────────

proptest! {
    #[test]
    fn word_pattern_quantifier_variants(idx in 0usize..=4) {
        let patterns = [
            r"[a-z]+",
            r"[a-z]*",
            r"[a-z]?",
            r"[a-z]{2,10}",
            r"[a-z]{3}",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        prop_assert_eq!(extract_leaf_pattern(attr), pat);
    }
}

// ── 14. Word attr count is exactly one ──────────────────────────────────────

proptest! {
    #[test]
    fn word_attr_count_exactly_one(idx in 0usize..=2) {
        let struct_names = ["Id", "Tok", "Nm"];
        let name = syn::Ident::new(struct_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(Debug)]
            #[adze::word]
            pub struct #name {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }).unwrap();
        let word_count = s.attrs.iter().filter(|a| is_adze_attr(a, "word")).count();
        prop_assert_eq!(word_count, 1);
        // total attrs: derive + word
        prop_assert_eq!(s.attrs.len(), 2);
    }
}

// ── 15. Word with transform closure: all params detected ────────────────────

proptest! {
    #[test]
    fn word_with_transform_all_params(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat, transform = |v| v.to_lowercase())]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let params = leaf_params(attr);
        let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
        prop_assert!(names.contains(&"pattern".to_string()));
        prop_assert!(names.contains(&"transform".to_string()));
        prop_assert_eq!(names.len(), 2);
    }
}

// ── 16. Word enum: word attr on enum not on variants ────────────────────────

proptest! {
    #[test]
    fn word_on_enum_not_on_variants(n in 2usize..=4) {
        let variants: Vec<proc_macro2::TokenStream> = (0..n)
            .map(|i| {
                let vname = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let txt = format!("v{i}");
                quote::quote! {
                    #[adze::leaf(text = #txt)]
                    #vname
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::word]
            pub enum Tokens {
                #(#variants),*
            }
        }).unwrap();
        // word is on the enum itself
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
        // word is NOT on any variant
        for v in &e.variants {
            prop_assert!(!v.attrs.iter().any(|a| is_adze_attr(a, "word")));
        }
    }
}

// ── 17. Word struct ident preserved across names ────────────────────────────

proptest! {
    #[test]
    fn word_struct_ident_matches(idx in 0usize..=5) {
        let names = ["Identifier", "WordToken", "Ident", "Name", "Symbol", "Lexeme"];
        let name_str = names[idx];
        let ident = syn::Ident::new(name_str, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), name_str);
    }
}

// ── 18. Word in module with extra: both types independently addressable ─────

proptest! {
    #[test]
    fn word_and_extra_independently_found(idx in 0usize..=2) {
        let ws_patterns = [r"\s", r"\s+", r"[ \t\n]+"];
        let ws_pat = ws_patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    id: Ident,
                }
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #ws_pat)]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let word_names: Vec<_> = items.iter().filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            {
                return Some(s.ident.to_string());
            }
            None
        }).collect();
        let extra_names: Vec<_> = items.iter().filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            {
                return Some(s.ident.to_string());
            }
            None
        }).collect();
        prop_assert_eq!(word_names, vec!["Ident"]);
        prop_assert_eq!(extra_names, vec!["Ws"]);
    }
}

// ── 19. Word vs leaf on unit struct: both attrs on struct ───────────────────

proptest! {
    #[test]
    fn word_with_leaf_on_unit_struct(idx in 0usize..=3) {
        let texts = ["identifier", "keyword", "token", "symbol"];
        let txt = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            #[adze::leaf(text = #txt)]
            pub struct W;
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"word".to_string()));
        prop_assert!(names.contains(&"leaf".to_string()));
        prop_assert!(matches!(s.fields, Fields::Unit));
    }
}

// ── 20. Word determinism: token stream identical across runs ────────────────

proptest! {
    #[test]
    fn word_tokenstream_deterministic(idx in 0usize..=3) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+", r"[A-Z]+"];
        let pat = patterns[idx];
        let results: Vec<String> = (0..3).map(|_| {
            let s: ItemStruct = syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }).unwrap();
            s.to_token_stream().to_string()
        }).collect();
        prop_assert_eq!(&results[0], &results[1]);
        prop_assert_eq!(&results[1], &results[2]);
    }
}

// ── 21. Word module: word struct not confused with language struct ───────────

proptest! {
    #[test]
    fn word_not_confused_with_language(idx in 0usize..=2) {
        let word_names_pool = ["Ident", "WordTok", "Identifier"];
        let lang_names_pool = ["Program", "Code", "Root"];
        let wn = syn::Ident::new(word_names_pool[idx], proc_macro2::Span::call_site());
        let ln = syn::Ident::new(lang_names_pool[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #ln {
                    ident: #wn,
                }
                #[adze::word]
                pub struct #wn {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        for item in items {
            if let Item::Struct(s) = item {
                let is_word = s.attrs.iter().any(|a| is_adze_attr(a, "word"));
                let is_lang = s.attrs.iter().any(|a| is_adze_attr(a, "language"));
                // A struct should not be both word and language (in this test)
                prop_assert!(!(is_word && is_lang),
                    "struct {} should not be both word and language", s.ident);
            }
        }
    }
}

// ── 22. Word struct fields: non-leaf fields not flagged as leaf ──────────────

proptest! {
    #[test]
    fn word_struct_skip_field_not_leaf(idx in 0usize..=2) {
        let patterns = [r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
                #[adze::skip(0u32)]
                counter: u32,
            }
        }).unwrap();
        let name_field = s.fields.iter().find(|f| {
            f.ident.as_ref().map(|i| i == "name").unwrap_or(false)
        }).unwrap();
        let counter_field = s.fields.iter().find(|f| {
            f.ident.as_ref().map(|i| i == "counter").unwrap_or(false)
        }).unwrap();
        prop_assert!(name_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(!counter_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(counter_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 23. Word with alternation pattern: pipe preserved ───────────────────────

proptest! {
    #[test]
    fn word_alternation_pattern_preserved(idx in 0usize..=2) {
        let patterns = [
            r"[a-z]+|[A-Z]+",
            r"\w+|@\w+",
            r"[a-zA-Z_]\w*|\$[a-zA-Z_]\w*",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert!(extracted.contains('|'));
        prop_assert_eq!(extracted, pat);
    }
}

// ── 24. Word coexists with derive attrs ─────────────────────────────────────

proptest! {
    #[test]
    fn word_coexists_with_derive(idx in 0usize..=2) {
        let patterns = [r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(Debug, Clone, PartialEq)]
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        // derive is not an adze attr
        let adze_names = adze_attr_names(&s.attrs);
        prop_assert_eq!(adze_names, vec!["word".to_string()]);
        // but total attr count is 2
        prop_assert_eq!(s.attrs.len(), 2);
        // word is present
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 25. Word enum ident preserved ───────────────────────────────────────────

proptest! {
    #[test]
    fn word_enum_ident_preserved(idx in 0usize..=3) {
        let names = ["Keywords", "Tokens", "Literals", "Reserved"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::word]
            pub enum #ident {
                #[adze::leaf(text = "a")]
                A,
                #[adze::leaf(text = "b")]
                B,
            }
        }).unwrap();
        prop_assert_eq!(e.ident.to_string(), names[idx]);
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 26. Word in module with prec_left: word not on enum variant ─────────────

proptest! {
    #[test]
    fn word_separate_from_prec_variants(prec in 1i32..=5) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Ident(Identifier),
                    #[adze::prec_left(#prec)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        // word is on the struct, not on any enum variant
        let expr_enum = items.iter().find_map(|i| {
            if let Item::Enum(e) = i { if e.ident == "Expr" { Some(e) } else { None } }
            else { None }
        }).unwrap();
        for v in &expr_enum.variants {
            prop_assert!(!v.attrs.iter().any(|a| is_adze_attr(a, "word")));
        }
    }
}

// ── 27. Word on tuple struct preserves tuple field count ────────────────────

proptest! {
    #[test]
    fn word_tuple_struct_field_count(idx in 0usize..=2) {
        let patterns = [r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident(
                #[adze::leaf(pattern = #pat)]
                String
            );
        }).unwrap();
        prop_assert!(matches!(s.fields, Fields::Unnamed(_)));
        prop_assert_eq!(s.fields.iter().count(), 1);
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 28. Word struct: field name is preserved ────────────────────────────────

proptest! {
    #[test]
    fn word_field_name_preserved(idx in 0usize..=3) {
        let field_names = ["name", "value", "text", "ident"];
        let fname = syn::Ident::new(field_names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = r"\w+")]
                #fname: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), field_names[idx]);
    }
}

// ── 29. Word determinism in module context ──────────────────────────────────

proptest! {
    #[test]
    fn word_module_deterministic(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let make_mod = || -> ItemMod {
            parse_mod(quote::quote! {
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
            })
        };
        let m1 = make_mod();
        let m2 = make_mod();
        prop_assert_eq!(m1.to_token_stream().to_string(), m2.to_token_stream().to_string());
    }
}

// ── 30. Word visibility: pub(crate) struct ──────────────────────────────────

proptest! {
    #[test]
    fn word_visibility_variants(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                struct Ident { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub(crate) struct Ident { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        match idx {
            0 => prop_assert!(matches!(s.vis, syn::Visibility::Public(_))),
            1 => prop_assert!(matches!(s.vis, syn::Visibility::Inherited)),
            _ => prop_assert!(matches!(s.vis, syn::Visibility::Restricted(_))),
        }
    }
}

// ── 31. Word attr is always an outer attribute ──────────────────────────────

proptest! {
    #[test]
    fn word_attr_is_outer(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct A { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::language]
                pub struct B { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[derive(Debug)]
                #[adze::word]
                pub struct C { #[adze::leaf(pattern = r"\w+")] name: String }
            }).unwrap(),
        };
        let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
        prop_assert!(matches!(word_attr.style, syn::AttrStyle::Outer));
    }
}

// ── 32. Word with doc comments preserved ────────────────────────────────────

proptest! {
    #[test]
    fn word_with_doc_comments(idx in 0usize..=1) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                /// This is the word token.
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                /// Doc after word attr.
                pub struct Ident {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }).unwrap(),
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let has_doc = s.attrs.iter().any(|a| a.path().is_ident("doc"));
        prop_assert!(has_doc);
    }
}

// ── 33. Word struct alongside multiple keyword leaf variants ────────────────

proptest! {
    #[test]
    fn word_with_multiple_keyword_variants(n in 2usize..=5) {
        let _keywords = ["if", "else", "while", "for", "return"];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("kw_grammar")]
            mod grammar {
                #[adze::language]
                pub enum Token {
                    Ident(Identifier),
                    #[adze::leaf(text = "if")]
                    If,
                    #[adze::leaf(text = "else")]
                    Else,
                    #[adze::leaf(text = "while")]
                    While,
                    #[adze::leaf(text = "for")]
                    For,
                    #[adze::leaf(text = "return")]
                    Return,
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let word_count = items.iter().filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        }).count();
        prop_assert_eq!(word_count, 1);

        let token_enum = items.iter().find_map(|i| {
            if let Item::Enum(e) = i { Some(e) } else { None }
        }).unwrap();
        // At least n keyword variants should be present
        let leaf_variants = token_enum.variants.iter()
            .filter(|v| v.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        prop_assert!(leaf_variants >= n);
    }
}

// ── 34. Word struct referenced as field type from language struct ────────────

proptest! {
    #[test]
    fn word_referenced_from_language_field(idx in 0usize..=1) {
        let m = match idx {
            0 => parse_mod(quote::quote! {
                #[adze::grammar("ref_test")]
                mod grammar {
                    #[adze::language]
                    pub struct Program {
                        name: Identifier,
                    }
                    #[adze::word]
                    pub struct Identifier {
                        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                        name: String,
                    }
                }
            }),
            _ => parse_mod(quote::quote! {
                #[adze::grammar("ref_test2")]
                mod grammar {
                    #[adze::language]
                    pub struct Program {
                        first: Identifier,
                        second: Identifier,
                    }
                    #[adze::word]
                    pub struct Identifier {
                        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                        name: String,
                    }
                }
            }),
        };
        let items = module_items(&m);
        let word_struct = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "word")) { Some(s) } else { None }
            } else { None }
        });
        prop_assert!(word_struct.is_some());
        prop_assert_eq!(word_struct.unwrap().ident.to_string(), "Identifier");
    }
}

// ── 35. Word meta is not List or NameValue ──────────────────────────────────

proptest! {
    #[test]
    fn word_meta_is_never_list_or_namevalue(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct A { name: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                struct B { name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub(crate) struct C { name: String }
            }).unwrap(),
        };
        let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
        prop_assert!(!matches!(word_attr.meta, syn::Meta::List(_)));
        prop_assert!(!matches!(word_attr.meta, syn::Meta::NameValue(_)));
    }
}

// ── 36. Word with various identifier naming conventions ─────────────────────

proptest! {
    #[test]
    fn word_struct_naming_conventions(idx in 0usize..=3) {
        let names = ["Ident", "MyIdentifier", "WordToken", "IdentifierNode"];
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word] pub struct Ident { #[adze::leaf(pattern = r"\w+")] v: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word] pub struct MyIdentifier { #[adze::leaf(pattern = r"\w+")] v: String }
            }).unwrap(),
            2 => syn::parse2(quote::quote! {
                #[adze::word] pub struct WordToken { #[adze::leaf(pattern = r"\w+")] v: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word] pub struct IdentifierNode { #[adze::leaf(pattern = r"\w+")] v: String }
            }).unwrap(),
        };
        prop_assert_eq!(s.ident.to_string(), names[idx]);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 37. Word with underscore-only pattern ───────────────────────────────────

proptest! {
    #[test]
    fn word_underscore_only_pattern(idx in 0usize..=1) {
        let patterns = [r"_+", r"_{1,32}"];
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Under { #[adze::leaf(pattern = r"_+")] name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Under { #[adze::leaf(pattern = r"_{1,32}")] name: String }
            }).unwrap(),
        };
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let pat = extract_leaf_pattern(attr);
        prop_assert_eq!(pat, patterns[idx]);
    }
}

// ── 38. Word struct without any leaf attr (bare struct) ─────────────────────

proptest! {
    #[test]
    fn word_without_leaf_attr(idx in 0usize..=1) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct BareWord { name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct BareWord { value: u32 }
            }).unwrap(),
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let has_leaf = s.fields.iter().any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(!has_leaf);
    }
}

// ── 39. Word struct with leaf text instead of pattern ────────────────────────

proptest! {
    #[test]
    fn word_with_leaf_text_only(idx in 0usize..=2) {
        let texts = ["identifier", "ident", "ID"];
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::leaf(text = "identifier")]
                pub struct W;
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::leaf(text = "ident")]
                pub struct W;
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::leaf(text = "ID")]
                pub struct W;
            }).unwrap(),
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let leaf = s.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let text = extract_leaf_text(leaf);
        prop_assert_eq!(text, texts[idx]);
    }
}

// ── 40. Word attr position in attr list (first, middle, last) ───────────────

proptest! {
    #[test]
    fn word_attr_position_in_list(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            // first
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                #[derive(Debug)]
                #[adze::language]
                pub struct A { name: String }
            }).unwrap(),
            // middle
            1 => syn::parse2(quote::quote! {
                #[derive(Debug)]
                #[adze::word]
                #[adze::language]
                pub struct A { name: String }
            }).unwrap(),
            // last
            _ => syn::parse2(quote::quote! {
                #[derive(Debug)]
                #[adze::language]
                #[adze::word]
                pub struct A { name: String }
            }).unwrap(),
        };
        let word_pos = s.attrs.iter().position(|a| is_adze_attr(a, "word")).unwrap();
        prop_assert_eq!(word_pos, idx);
    }
}

// ── 41. Grammar with word struct and many non-word structs ──────────────────

proptest! {
    #[test]
    fn word_among_many_non_word_structs(_idx in 0usize..=1) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("big_grammar")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    expr: Expr,
                }

                pub struct Expr {
                    ident: Identifier,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }

                pub struct Operator {
                    #[adze::leaf(text = "+")]
                    _op: (),
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let items = module_items(&m);
        let struct_count = items.iter().filter(|i| matches!(i, Item::Struct(_))).count();
        prop_assert!(struct_count >= 4);
        let word_count = items.iter().filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        }).count();
        prop_assert_eq!(word_count, 1);
    }
}

// ── 42. Word position in module does not affect struct content ───────────────

proptest! {
    #[test]
    fn word_position_in_module_irrelevant(idx in 0usize..=1) {
        // word at beginning vs end of module items
        let m = match idx {
            0 => parse_mod(quote::quote! {
                #[adze::grammar("pos_test")]
                mod grammar {
                    #[adze::word]
                    pub struct Identifier {
                        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                        name: String,
                    }
                    #[adze::language]
                    pub struct Root { id: Identifier }
                }
            }),
            _ => parse_mod(quote::quote! {
                #[adze::grammar("pos_test")]
                mod grammar {
                    #[adze::language]
                    pub struct Root { id: Identifier }
                    #[adze::word]
                    pub struct Identifier {
                        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                        name: String,
                    }
                }
            }),
        };
        let items = module_items(&m);
        let word_struct = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "word")) { Some(s) } else { None }
            } else { None }
        }).unwrap();
        prop_assert_eq!(word_struct.ident.to_string(), "Identifier");
        prop_assert_eq!(word_struct.fields.iter().count(), 1);
    }
}

// ── 43. Word alongside reserved keyword leaf text values ────────────────────

proptest! {
    #[test]
    fn word_with_reserved_keyword_leaves(idx in 0usize..=4) {
        let reserved = ["if", "else", "while", "for", "return"];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("reserved_test")]
            mod grammar {
                #[adze::language]
                pub enum Stmt {
                    Ident(Identifier),
                    #[adze::leaf(text = "if")]
                    If,
                    #[adze::leaf(text = "else")]
                    Else,
                    #[adze::leaf(text = "while")]
                    While,
                    #[adze::leaf(text = "for")]
                    For,
                    #[adze::leaf(text = "return")]
                    Return,
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let stmt_enum = items.iter().find_map(|i| {
            if let Item::Enum(e) = i { Some(e) } else { None }
        }).unwrap();
        // The keyword variant at `idx` should have the correct leaf text
        let leaf_variants: Vec<_> = stmt_enum.variants.iter()
            .filter(|v| v.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .collect();
        prop_assert!(idx < leaf_variants.len());
        let attr = leaf_variants[idx].attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let text = extract_leaf_text(attr);
        prop_assert_eq!(text, reserved[idx]);
    }
}

// ── 44. Word struct generics are preserved ──────────────────────────────────

#[test]
fn word_struct_with_no_generics_has_empty_generics() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    })
    .unwrap();
    assert!(s.generics.params.is_empty());
    assert!(s.generics.where_clause.is_none());
}

// ── 45. Word attr does not duplicate on re-parse ────────────────────────────

proptest! {
    #[test]
    fn word_no_duplication_on_reparse(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word] pub struct A { name: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word] #[adze::language] pub struct B { name: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[derive(Clone)] #[adze::word] pub struct C { name: String }
            }).unwrap(),
        };
        // Re-parse from token stream
        let tokens = s.to_token_stream();
        let reparsed: ItemStruct = syn::parse2(tokens).unwrap();
        let word_count = reparsed.attrs.iter().filter(|a| is_adze_attr(a, "word")).count();
        prop_assert_eq!(word_count, 1);
    }
}

// ── 46. Word expansion determinism across repeated parses ───────────────────

proptest! {
    #[test]
    fn word_expansion_deterministic_repeated(idx in 0usize..=3) {
        let patterns = [r"\w+", r"[a-zA-Z_]\w*", r"[a-z]+", r"\p{L}+"];
        let make = |p: &str| -> String {
            let pat = proc_macro2::Literal::string(p);
            let s: ItemStruct = syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }).unwrap();
            s.to_token_stream().to_string()
        };
        let first = make(patterns[idx]);
        let second = make(patterns[idx]);
        let third = make(patterns[idx]);
        prop_assert_eq!(&first, &second);
        prop_assert_eq!(&second, &third);
    }
}

// ── 47. Word struct field name does not collide with struct name ─────────────

proptest! {
    #[test]
    fn word_field_name_differs_from_struct_name(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident { #[adze::leaf(pattern = r"\w+")] value: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Token { #[adze::leaf(pattern = r"\w+")] text: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Word { #[adze::leaf(pattern = r"\w+")] content: String }
            }).unwrap(),
        };
        let struct_name = s.ident.to_string();
        for field in &s.fields {
            if let Some(ref name) = field.ident {
                prop_assert_ne!(name.to_string(), struct_name.to_lowercase());
            }
        }
    }
}

// ── 48. Word module round-trip preserves grammar name ────────────────────────

proptest! {
    #[test]
    fn word_module_grammar_name_roundtrip(idx in 0usize..=2) {
        let names = ["alpha", "beta_lang", "my_grammar"];
        let name = proc_macro2::Literal::string(names[idx]);
        let m = parse_mod(quote::quote! {
            #[adze::grammar(#name)]
            mod grammar {
                #[adze::language]
                pub struct Root { id: Identifier }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        // Re-parse from token stream
        let tokens = m.to_token_stream();
        let reparsed: ItemMod = syn::parse2(tokens).unwrap();
        let grammar_attr = reparsed.attrs.iter()
            .find(|a| is_adze_attr(a, "grammar"))
            .unwrap();
        let ts = grammar_attr.meta.to_token_stream().to_string();
        prop_assert!(ts.contains(names[idx]));
    }
}

// ── 49. Word on enum with named fields variant ──────────────────────────────

proptest! {
    #[test]
    fn word_on_enum_with_named_fields(idx in 0usize..=1) {
        let e: ItemEnum = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub enum TokenKind {
                    Ident { #[adze::leaf(pattern = r"\w+")] name: String },
                    #[adze::leaf(text = "+")] Plus,
                }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub enum TokenKind {
                    KeywordIf { #[adze::leaf(text = "if")] _kw: () },
                    KeywordElse { #[adze::leaf(text = "else")] _kw: () },
                }
            }).unwrap(),
        };
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert!(e.variants.len() >= 2);
    }
}

// ── 50. Word with both pattern and text fields in different fields ───────────

#[test]
fn word_struct_mixed_leaf_params() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::word]
        pub struct MixedToken {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = ":")]
            _colon: (),
        }
    })
    .unwrap();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(fields.len(), 2);
    let first_leaf = fields[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(first_leaf);
    assert_eq!(params[0].path.to_string(), "pattern");
    let second_leaf = fields[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params2 = leaf_params(second_leaf);
    assert_eq!(params2[0].path.to_string(), "text");
}

// ── 51. Word struct token stream equality is reflexive ──────────────────────

proptest! {
    #[test]
    fn word_token_stream_reflexive(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word] pub struct A { #[adze::leaf(pattern = r"\w+")] n: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word] pub struct B(#[adze::leaf(pattern = r"\w+")] String);
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word] #[adze::leaf(text = "x")] pub struct C;
            }).unwrap(),
        };
        let ts1 = s.to_token_stream().to_string();
        let ts2 = s.to_token_stream().to_string();
        prop_assert_eq!(ts1, ts2);
    }
}

// ── 52. Word module determinism with enum + keyword variants ────────────────

proptest! {
    #[test]
    fn word_module_with_keywords_deterministic(_idx in 0usize..=1) {
        let make = || {
            let m = parse_mod(quote::quote! {
                #[adze::grammar("det_kw")]
                mod grammar {
                    #[adze::language]
                    pub enum Expr {
                        Ident(Identifier),
                        #[adze::leaf(text = "true")]
                        True,
                        #[adze::leaf(text = "false")]
                        False,
                    }
                    #[adze::word]
                    pub struct Identifier {
                        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                        name: String,
                    }
                }
            });
            m.to_token_stream().to_string()
        };
        let a = make();
        let b = make();
        prop_assert_eq!(a, b);
    }
}

// ── 53. Word with Optional field type ───────────────────────────────────────

#[test]
fn word_struct_with_optional_field() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: Option<String>,
        }
    })
    .unwrap();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let field = s.fields.iter().next().unwrap();
    let ty_str = field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Option"));
}

// ── 54. Word with Vec field type ────────────────────────────────────────────

#[test]
fn word_struct_with_vec_field() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::word]
        pub struct MultiToken {
            #[adze::leaf(pattern = r"\w+")]
            parts: Vec<String>,
        }
    })
    .unwrap();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let field = s.fields.iter().next().unwrap();
    let ty_str = field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Vec"));
}

// ── 55. Word attr count in module is independent of module attr count ────────

proptest! {
    #[test]
    fn word_count_independent_of_module_attrs(_idx in 0usize..=2) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("count_test")]
            mod grammar {
                #[adze::language]
                pub struct Root { ident: Identifier }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
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
        let items = module_items(&m);
        let word_count = items.iter().filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        }).count();
        let extra_count = items.iter().filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else { false }
        }).count();
        prop_assert_eq!(word_count, 1);
        prop_assert_eq!(extra_count, 2);
    }
}

// ── 56. Word struct attrs are non-empty ─────────────────────────────────────

proptest! {
    #[test]
    fn word_struct_always_has_attrs(idx in 0usize..=2) {
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word] pub struct A { n: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word] #[derive(Debug)] pub struct B { n: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word] #[adze::language] #[derive(Clone)] pub struct C { n: String }
            }).unwrap(),
        };
        prop_assert!(!s.attrs.is_empty());
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 57. Word in grammar with prec_right and prec operators ──────────────────

proptest! {
    #[test]
    fn word_with_mixed_precedence_operators(_idx in 0usize..=1) {
        let m = parse_mod(quote::quote! {
            #[adze::grammar("prec_mix")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Ident(Identifier),
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_right(2)]
                    Pow(Box<Expr>, #[adze::leaf(text = "**")] (), Box<Expr>),
                    #[adze::prec(3)]
                    Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        });
        prop_assert!(has_word);
        let enum_item = items.iter().find_map(|i| {
            if let Item::Enum(e) = i { Some(e) } else { None }
        }).unwrap();
        prop_assert_eq!(enum_item.variants.len(), 5);
    }
}

// ── 58. Word leaf pattern with anchors ──────────────────────────────────────

proptest! {
    #[test]
    fn word_pattern_with_anchors(idx in 0usize..=2) {
        let patterns = [r"^[a-zA-Z_]\w*$", r"[a-zA-Z]\w*\b", r"\b\w+\b"];
        let s: ItemStruct = match idx {
            0 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct A { #[adze::leaf(pattern = r"^[a-zA-Z_]\w*$")] n: String }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct A { #[adze::leaf(pattern = r"[a-zA-Z]\w*\b")] n: String }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub struct A { #[adze::leaf(pattern = r"\b\w+\b")] n: String }
            }).unwrap(),
        };
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let pat = extract_leaf_pattern(attr);
        prop_assert_eq!(pat, patterns[idx]);
    }
}

// ── 59. Word struct with multiple skip fields ───────────────────────────────

#[test]
fn word_struct_with_multiple_skip_fields() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::word]
        pub struct RichIdent {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::skip(false)]
            is_keyword: bool,
            #[adze::skip(0u32)]
            line_number: u32,
        }
    })
    .unwrap();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let skip_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count();
    assert_eq!(skip_count, 2);
}

// ── 60. Word enum variant count preserved after re-parse ────────────────────

proptest! {
    #[test]
    fn word_enum_variant_count_after_reparse(n in 2usize..=4) {
        let e: ItemEnum = match n {
            2 => syn::parse2(quote::quote! {
                #[adze::word]
                pub enum TK {
                    #[adze::leaf(text = "a")] A,
                    #[adze::leaf(text = "b")] B,
                }
            }).unwrap(),
            3 => syn::parse2(quote::quote! {
                #[adze::word]
                pub enum TK {
                    #[adze::leaf(text = "a")] A,
                    #[adze::leaf(text = "b")] B,
                    #[adze::leaf(text = "c")] C,
                }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                #[adze::word]
                pub enum TK {
                    #[adze::leaf(text = "a")] A,
                    #[adze::leaf(text = "b")] B,
                    #[adze::leaf(text = "c")] C,
                    #[adze::leaf(text = "d")] D,
                }
            }).unwrap(),
        };
        let tokens = e.to_token_stream();
        let reparsed: ItemEnum = syn::parse2(tokens).unwrap();
        prop_assert_eq!(reparsed.variants.len(), n);
        prop_assert!(reparsed.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}
