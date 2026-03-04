#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::word]` annotation in adze-macro.
//!
//! Uses proptest to generate randomized patterns, struct names, field types,
//! and annotation combinations, then verifies that syn correctly parses and
//! preserves the word attribute and its interaction with other grammar
//! annotations.

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

fn count_words_in_module(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|i| match i {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "word")),
            Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "word")),
            _ => false,
        })
        .count()
}

fn word_struct_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            {
                return Some(s.ident.to_string());
            }
            None
        })
        .collect()
}

// ── 1. Word annotation detected on struct with random pattern ───────────────

proptest! {
    #[test]
    fn word_annotation_detected_on_struct(idx in 0usize..=4) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+", r"[A-Za-z_][A-Za-z0-9_]*", r"[a-zA-Z$_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"word".to_string()));
        prop_assert_eq!(names.len(), 1);
    }
}

// ── 2. Word with regex pattern preserved exactly ────────────────────────────

proptest! {
    #[test]
    fn word_regex_pattern_preserved(idx in 0usize..=5) {
        let patterns = [
            r"[a-zA-Z_]\w*",
            r"\w+",
            r"[a-z][a-z0-9]*",
            r"[\p{L}_][\p{L}\p{N}_]*",
            r"[a-zA-Z$_][a-zA-Z0-9$_]*",
            r"[a-zA-Z_]{1,}[a-zA-Z0-9_]{0,255}",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Token {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 3. Word with text pattern (keyword literal) ─────────────────────────────

proptest! {
    #[test]
    fn word_with_text_literal(idx in 0usize..=4) {
        let texts = ["let", "fn", "if", "else", "return"];
        let txt = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Keyword {
                #[adze::leaf(text = #txt)]
                _kw: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let value = extract_leaf_text(attr);
        prop_assert_eq!(value, txt);
    }
}

// ── 4. Only one word per grammar module ─────────────────────────────────────

proptest! {
    #[test]
    fn only_one_word_per_grammar(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Code {
                    ident: Identifier,
                    kw: Keyword,
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }

                pub struct Keyword {
                    #[adze::leaf(text = "let")]
                    _kw: (),
                }
            }
        });
        prop_assert_eq!(count_words_in_module(&m), 1);
        let names = word_struct_names(&m);
        prop_assert_eq!(&names[0], "Identifier");
    }
}

// ── 5. Word in module context with language struct ──────────────────────────

proptest! {
    #[test]
    fn word_in_module_with_language(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z_]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    name: Identifier,
                }

                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let has_language = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "language"))
            } else { false }
        });
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        });
        prop_assert!(has_language);
        prop_assert!(has_word);
    }
}

// ── 6. Word type detection among mixed items ────────────────────────────────

proptest! {
    #[test]
    fn word_type_detection_in_module(n_regular in 1usize..=3) {
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
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let words = word_struct_names(&m);
        prop_assert_eq!(words.len(), 1);
        prop_assert_eq!(&words[0], "Ident");
    }
}

// ── 7. Word combined with leaf on struct ────────────────────────────────────

proptest! {
    #[test]
    fn word_combined_with_leaf(idx in 0usize..=3) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+", r"[A-Z_]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"word".to_string()));
        let field = s.fields.iter().next().unwrap();
        let has_leaf = field.attrs.iter().any(|a| is_adze_attr(a, "leaf"));
        prop_assert!(has_leaf);
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 8. Word struct name preserved with random names ─────────────────────────

proptest! {
    #[test]
    fn word_struct_name_preserved(idx in 0usize..=4) {
        let names = ["Identifier", "Word", "Token", "Ident", "Name"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct #ident {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), names[idx]);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 9. Word attr is path-style (no arguments) ──────────────────────────────

proptest! {
    #[test]
    fn word_attr_is_path_style(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
        prop_assert!(matches!(word_attr.meta, syn::Meta::Path(_)));
    }
}

// ── 10. Word field type is String ───────────────────────────────────────────

proptest! {
    #[test]
    fn word_field_type_is_string(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ty.to_token_stream().to_string(), "String");
    }
}

// ── 11. Word attr path has two segments ─────────────────────────────────────

proptest! {
    #[test]
    fn word_attr_path_segments(idx in 0usize..=2) {
        let names = ["Ident", "Token", "Word"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct #ident {
                name: String,
            }
        }).unwrap();
        let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
        let segs: Vec<_> = word_attr.path().segments.iter().collect();
        prop_assert_eq!(segs.len(), 2);
        prop_assert_eq!(segs[0].ident.to_string(), "adze");
        prop_assert_eq!(segs[1].ident.to_string(), "word");
    }
}

// ── 12. Word on unit struct ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn word_on_unit_struct(idx in 0usize..=3) {
        let texts = ["identifier", "word", "token", "name"];
        let txt = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            #[adze::leaf(text = #txt)]
            pub struct WordToken;
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert!(matches!(s.fields, Fields::Unit));
    }
}

// ── 13. Word on tuple struct ────────────────────────────────────────────────

proptest! {
    #[test]
    fn word_on_tuple_struct(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident(
                #[adze::leaf(pattern = #pat)]
                String
            );
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert!(matches!(s.fields, Fields::Unnamed(_)));
        if let Fields::Unnamed(ref u) = s.fields {
            prop_assert_eq!(u.unnamed.len(), 1);
        }
    }
}

// ── 14. Word preserves non-adze attributes ──────────────────────────────────

proptest! {
    #[test]
    fn word_preserves_derive_attrs(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(Debug, Clone)]
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert_eq!(s.attrs.len(), 2);
        let adze_names = adze_attr_names(&s.attrs);
        prop_assert_eq!(adze_names, vec!["word".to_string()]);
        let has_derive = s.attrs.iter().any(|a| {
            a.path().segments.iter().next().map(|s| s.ident == "derive").unwrap_or(false)
        });
        prop_assert!(has_derive);
    }
}

// ── 15. Word coexists with extra annotation ─────────────────────────────────

proptest! {
    #[test]
    fn word_coexists_with_extra(idx in 0usize..=2) {
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
        let has_extra = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "extra")) }
            else { false }
        });
        prop_assert!(has_word);
        prop_assert!(has_extra);
    }
}

// ── 16. Word coexists with external annotation ──────────────────────────────

proptest! {
    #[test]
    fn word_coexists_with_external(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
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
                #[adze::external]
                struct IndentToken;
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        let has_external = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "external")) }
            else { false }
        });
        prop_assert!(has_word);
        prop_assert!(has_external);
    }
}

// ── 17. Word combined with language annotation ──────────────────────────────

proptest! {
    #[test]
    fn word_combined_with_language(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert_eq!(names.len(), 2);
        prop_assert!(names.contains(&"word".to_string()));
        prop_assert!(names.contains(&"language".to_string()));
    }
}

// ── 18. Word struct distinct from language struct in module ──────────────────

proptest! {
    #[test]
    fn word_struct_distinct_from_language(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Program {
                    name: Identifier,
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let language_names: Vec<_> = items.iter().filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "language"))
            {
                return Some(s.ident.to_string());
            }
            None
        }).collect();
        let word_names = word_struct_names(&m);
        prop_assert_eq!(language_names, vec!["Program"]);
        prop_assert_eq!(word_names, vec!["Identifier"]);
    }
}

// ── 19. Word pattern with special regex characters ──────────────────────────

proptest! {
    #[test]
    fn word_pattern_special_regex_chars(idx in 0usize..=4) {
        let patterns = [
            r"[a-zA-Z$_][a-zA-Z0-9$_]*",
            r"[a-zA-Z_][a-zA-Z0-9_\-]*",
            r"[a-zA-Z_]\w*|@[a-zA-Z_]\w*",
            r"[0-9a-zA-Z_]+",
            r"[a-zA-Z\u{00C0}-\u{024F}_]+",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Token {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 20. Word ordering before language ───────────────────────────────────────

proptest! {
    #[test]
    fn word_before_language_in_module(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
                #[adze::language]
                pub struct Code {
                    ident: Identifier,
                }
            }
        });
        prop_assert_eq!(count_words_in_module(&m), 1);
        let items = module_items(&m);
        let has_lang = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "language")) }
            else { false }
        });
        prop_assert!(has_lang);
    }
}

// ── 21. Word alongside keyword enum ─────────────────────────────────────────

proptest! {
    #[test]
    fn word_alongside_keyword_enum(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("kw_test")]
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
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        let has_enum = items.iter().any(|i| matches!(i, Item::Enum(_)));
        prop_assert!(has_word);
        prop_assert!(has_enum);
    }
}

// ── 22. Word with transform closure ─────────────────────────────────────────

proptest! {
    #[test]
    fn word_with_transform_param(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = #pat, transform = |v| v.to_uppercase())]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let params = leaf_params(attr);
        let param_names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
        prop_assert!(param_names.contains(&"pattern".to_owned()));
        prop_assert!(param_names.contains(&"transform".to_owned()));
    }
}

// ── 23. Word attr ordering among multiple attributes ────────────────────────

proptest! {
    #[test]
    fn word_attr_ordering_preserved(idx in 0usize..=1) {
        // Test both orderings: word-first and language-first
        if idx == 0 {
            let s: ItemStruct = syn::parse2(quote::quote! {
                #[adze::word]
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap();
            let names = adze_attr_names(&s.attrs);
            prop_assert_eq!(&names[0], "word");
            prop_assert_eq!(&names[1], "language");
        } else {
            let s: ItemStruct = syn::parse2(quote::quote! {
                #[adze::language]
                #[adze::word]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap();
            let names = adze_attr_names(&s.attrs);
            prop_assert_eq!(&names[0], "language");
            prop_assert_eq!(&names[1], "word");
        }
    }
}

// ── 24. Word struct with skip field ─────────────────────────────────────────

proptest! {
    #[test]
    fn word_struct_with_skip_field(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct RichIdent {
                #[adze::leaf(pattern = #pat)]
                name: String,
                #[adze::skip(false)]
                is_keyword: bool,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert_eq!(s.fields.iter().count(), 2);
        let has_leaf = s.fields.iter().any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        let has_skip = s.fields.iter().any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert!(has_leaf);
        prop_assert!(has_skip);
    }
}

// ── 25. Word in grammar with precedence operators ───────────────────────────

proptest! {
    #[test]
    fn word_in_grammar_with_precedence(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("expr_lang")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Ident(Identifier),
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        prop_assert!(has_word);
        let expr_enum = items.iter().find_map(|i| {
            if let Item::Enum(e) = i { if e.ident == "Expr" { Some(e) } else { None } }
            else { None }
        });
        prop_assert!(expr_enum.is_some());
        prop_assert_eq!(expr_enum.unwrap().variants.len(), 3);
    }
}

// ── 26. Word on enum parses correctly ───────────────────────────────────────

proptest! {
    #[test]
    fn word_on_enum_variant_count(n_variants in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_variants)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let txt = format!("kw{i}");
                quote::quote! {
                    #[adze::leaf(text = #txt)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::word]
            pub enum KeywordToken {
                #(#variant_tokens),*
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
        prop_assert_eq!(e.variants.len(), n_variants);
    }
}

// ── 27. Word visibility variants ────────────────────────────────────────────

proptest! {
    #[test]
    fn word_struct_visibility(idx in 0usize..=2) {
        // Test pub, private, and pub(crate)
        let s: ItemStruct = if idx == 0 {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub struct Ident {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap()
        } else if idx == 1 {
            syn::parse2(quote::quote! {
                #[adze::word]
                struct Ident {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                #[adze::word]
                pub(crate) struct Ident {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }).unwrap()
        };
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        match idx {
            0 => prop_assert!(matches!(s.vis, syn::Visibility::Public(_))),
            1 => prop_assert!(matches!(s.vis, syn::Visibility::Inherited)),
            _ => prop_assert!(matches!(s.vis, syn::Visibility::Restricted(_))),
        }
    }
}

// ── 28. Word appears exactly once in attrs ──────────────────────────────────

proptest! {
    #[test]
    fn word_appears_once_in_attrs(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let word_count = s.attrs.iter().filter(|a| is_adze_attr(a, "word")).count();
        prop_assert_eq!(word_count, 1);
    }
}

// ── 29. Word with empty pattern (edge case) ─────────────────────────────────

proptest! {
    #[test]
    fn word_with_empty_pattern(idx in 0usize..=1) {
        let patterns = ["", " "];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct EmptyWord {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
        let extracted = extract_leaf_pattern(attr);
        prop_assert_eq!(extracted, pat);
    }
}

// ── 30. Word in module with delimited list ──────────────────────────────────

proptest! {
    #[test]
    fn word_in_module_with_delimited(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("list_lang")]
            mod grammar {
                #[adze::language]
                pub struct IdentList {
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    idents: Vec<Identifier>,
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) }
            else { false }
        });
        prop_assert!(has_word);
        let lang = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        });
        prop_assert!(lang.is_some());
    }
}

// ── 31. Word with all three annotation kinds present ────────────────────────

proptest! {
    #[test]
    fn word_with_extra_and_external(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    ident: Identifier,
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
                #[adze::external]
                struct IndentToken;
            }
        });
        let items = module_items(&m);
        let mut found_word = false;
        let mut found_extra = false;
        let mut found_external = false;
        for item in items {
            if let Item::Struct(s) = item {
                if s.attrs.iter().any(|a| is_adze_attr(a, "word")) { found_word = true; }
                if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) { found_extra = true; }
                if s.attrs.iter().any(|a| is_adze_attr(a, "external")) { found_external = true; }
            }
        }
        prop_assert!(found_word);
        prop_assert!(found_extra);
        prop_assert!(found_external);
    }
}

// ── 32. Word single field count ─────────────────────────────────────────────

proptest! {
    #[test]
    fn word_struct_single_field(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert_eq!(s.fields.iter().count(), 1);
    }
}

// ── 33. Word pattern with alternation ───────────────────────────────────────

proptest! {
    #[test]
    fn word_pattern_with_alternation(idx in 0usize..=2) {
        let patterns = [
            r"[a-zA-Z_]\w*|@[a-zA-Z_]\w*",
            r"\w+|\$\w+",
            r"[a-z]+|_[a-z]+",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct MixedToken {
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

// ── 34. Word as language root in grammar module ─────────────────────────────

proptest! {
    #[test]
    fn word_as_language_root(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("word_root")]
            mod grammar {
                #[adze::word]
                #[adze::language]
                pub struct Token {
                    #[adze::leaf(pattern = #pat)]
                    value: String,
                }
            }
        });
        let items = module_items(&m);
        let root = items.iter().find_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) { Some(s) }
                else { None }
            } else { None }
        });
        prop_assert!(root.is_some());
        let root = root.unwrap();
        prop_assert!(root.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 35. Word with multiple regular structs in module ────────────────────────

proptest! {
    #[test]
    fn word_among_multiple_structs(n_regular in 1usize..=4) {
        let regular_tokens: Vec<proc_macro2::TokenStream> = (0..n_regular)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
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
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        prop_assert_eq!(count_words_in_module(&m), 1);
        let total_structs = module_items(&m).iter().filter(|i| matches!(i, Item::Struct(_))).count();
        // language + n_regular helpers + word = n_regular + 2
        prop_assert_eq!(total_structs, n_regular + 2);
    }
}
