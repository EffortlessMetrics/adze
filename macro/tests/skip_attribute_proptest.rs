#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::skip]` attribute handling in adze-macro.
//!
//! Uses proptest to generate randomized default expressions, field counts,
//! type combinations, and annotation orderings, then verifies that syn
//! correctly parses and preserves the skip attribute and its interactions
//! with other adze annotations.

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

fn skip_expr_str(attr: &Attribute) -> String {
    attr.parse_args::<syn::Expr>()
        .expect("skip attribute should contain an expression")
        .to_token_stream()
        .to_string()
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn count_skip_fields(s: &ItemStruct) -> usize {
    s.fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count()
}

fn skip_field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect()
}

// ── 1. Skip attribute detected on struct field with random default expr ─────

proptest! {
    #[test]
    fn skip_detected_on_struct_field(idx in 0usize..=4) {
        let defaults = ["false", "true", "0", "42", "None"];
        let def = defaults[idx];
        let def_tokens: proc_macro2::TokenStream = def.parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Node {
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 2. Skip default expression preserved exactly ────────────────────────────

proptest! {
    #[test]
    fn skip_default_expr_preserved(idx in 0usize..=5) {
        let exprs = ["false", "true", "0", "42", "0u32", "None"];
        let expected_strs = ["false", "true", "0", "42", "0u32", "None"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                f: bool,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected_strs[idx]);
    }
}

// ── 3. Skip with constructor call expressions ───────────────────────────────

proptest! {
    #[test]
    fn skip_constructor_call_preserved(idx in 0usize..=3) {
        let call_strs = ["String::new()", "Vec::new()", "Default::default()", "Vec::with_capacity(10)"];
        let expected = ["String :: new ()", "Vec :: new ()", "Default :: default ()", "Vec :: with_capacity (10)"];
        let call_tokens: proc_macro2::TokenStream = call_strs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#call_tokens)]
                data: String,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 4. Skip on multiple struct fields with random count ─────────────────────

proptest! {
    #[test]
    fn skip_multiple_fields_random_count(count in 1usize..=5) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("skip_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool,
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Multi {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
                #(#field_tokens)*
            }
        }).unwrap();
        prop_assert_eq!(count_skip_fields(&s), count);
    }
}

// ── 5. Skip combined with leaf fields preserves field order ─────────────────

proptest! {
    #[test]
    fn skip_leaf_interleaved_order(idx in 0usize..=2) {
        // Three layouts: skip-first, skip-middle, skip-last
        let s: ItemStruct = match idx {
            0 => parse_quote! {
                pub struct N {
                    #[adze::skip(0)]
                    meta: i32,
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }
            },
            1 => parse_quote! {
                pub struct N {
                    #[adze::leaf(pattern = r"\w+")]
                    a: String,
                    #[adze::skip(false)]
                    b: bool,
                    #[adze::leaf(pattern = r"\d+")]
                    c: String,
                }
            },
            _ => parse_quote! {
                pub struct N {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                    #[adze::skip(false)]
                    meta: bool,
                }
            },
        };
        let annotations: Vec<Vec<String>> = s.fields.iter()
            .map(|f| adze_attr_names(&f.attrs))
            .collect();
        match idx {
            0 => {
                prop_assert_eq!(&annotations[0], &vec!["skip".to_string()]);
                prop_assert_eq!(&annotations[1], &vec!["leaf".to_string()]);
            }
            1 => {
                prop_assert_eq!(&annotations[0], &vec!["leaf".to_string()]);
                prop_assert_eq!(&annotations[1], &vec!["skip".to_string()]);
                prop_assert_eq!(&annotations[2], &vec!["leaf".to_string()]);
            }
            _ => {
                prop_assert_eq!(&annotations[0], &vec!["leaf".to_string()]);
                prop_assert_eq!(&annotations[1], &vec!["skip".to_string()]);
            }
        }
    }
}

// ── 6. Skip on enum variant named field ─────────────────────────────────────

proptest! {
    #[test]
    fn skip_on_enum_named_field(idx in 0usize..=3) {
        let defaults = ["false", "true", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Literal {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                    #[adze::skip(#def_tokens)]
                    cached: bool,
                },
            }
        }).unwrap();
        let variant = &e.variants[0];
        if let Fields::Named(ref named) = variant.fields {
            let skip_field = named.named.iter().find(|f| {
                f.ident.as_ref().is_some_and(|i| i == "cached")
            }).unwrap();
            prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 7. Skip on enum variant unnamed field ───────────────────────────────────

proptest! {
    #[test]
    fn skip_on_enum_unnamed_field(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                WithMeta(
                    #[adze::leaf(pattern = r"\d+")]
                    String,
                    #[adze::skip(#def_tokens)]
                    bool,
                ),
            }
        }).unwrap();
        if let Fields::Unnamed(ref unnamed) = e.variants[0].fields {
            prop_assert!(unnamed.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 8. Multiple skip fields in enum variant ─────────────────────────────────

proptest! {
    #[test]
    fn skip_multiple_enum_variant_fields(n_skip in 1usize..=4) {
        let skip_tokens: Vec<proc_macro2::TokenStream> = (0..n_skip)
            .map(|i| {
                let name = syn::Ident::new(&format!("m{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(0)]
                    #name: i32,
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Annotated {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #(#skip_tokens)*
                },
            }
        }).unwrap();
        if let Fields::Named(ref named) = e.variants[0].fields {
            let skip_count = named.named.iter()
                .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
                .count();
            prop_assert_eq!(skip_count, n_skip);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 9. Skip with pattern parameter in grammar module ────────────────────────

proptest! {
    #[test]
    fn skip_in_grammar_module_struct(idx in 0usize..=3) {
        let defaults = ["false", "0", "String::new()", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                    #[adze::skip(#def_tokens)]
                    visited: bool,
                }
            }
        });
        let root = find_struct_in_mod(&m, "Root").unwrap();
        let skip_field = root.fields.iter().find(|f| {
            f.ident.as_ref().is_some_and(|i| i == "visited")
        }).unwrap();
        prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 10. Skip in grammar module non-root struct ──────────────────────────────

proptest! {
    #[test]
    fn skip_in_non_root_struct(idx in 0usize..=2) {
        let defaults = ["0", "false", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    child: Child,
                }
                pub struct Child {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #[adze::skip(#def_tokens)]
                    depth: i32,
                }
            }
        });
        let child = find_struct_in_mod(&m, "Child").unwrap();
        prop_assert!(child.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        ));
    }
}

// ── 11. Skip in grammar module enum variant ─────────────────────────────────

proptest! {
    #[test]
    fn skip_in_grammar_enum_variant(idx in 0usize..=2) {
        let defaults = ["false", "true", "0"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num {
                        #[adze::leaf(pattern = r"\d+")]
                        value: String,
                        #[adze::skip(#def_tokens)]
                        negated: bool,
                    },
                }
            }
        });
        let expr = find_enum_in_mod(&m, "Expr").unwrap();
        if let Fields::Named(ref named) = expr.variants[0].fields {
            let skip_field = named.named.iter().find(|f| {
                f.ident.as_ref().is_some_and(|i| i == "negated")
            }).unwrap();
            prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 12. Skip field type variations ──────────────────────────────────────────

proptest! {
    #[test]
    fn skip_field_type_variations(idx in 0usize..=5) {
        let types = ["bool", "i32", "u64", "String", "f64", "char"];
        let defaults = ["false", "0", "0", "String::new()", "0.0", "'x'"];
        let ty_tokens: proc_macro2::TokenStream = types[idx].parse().unwrap();
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                field: #ty_tokens,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, types[idx]);
    }
}

// ── 13. Skip field visibility preserved ─────────────────────────────────────

proptest! {
    #[test]
    fn skip_field_visibility_preserved(vis_idx in 0usize..=2) {
        let s: ItemStruct = match vis_idx {
            0 => syn::parse2(quote::quote! {
                pub struct N {
                    #[adze::skip(false)]
                    flag: bool,
                }
            }).unwrap(),
            1 => syn::parse2(quote::quote! {
                pub struct N {
                    #[adze::skip(false)]
                    pub flag: bool,
                }
            }).unwrap(),
            _ => syn::parse2(quote::quote! {
                pub struct N {
                    #[adze::skip(false)]
                    pub(crate) flag: bool,
                }
            }).unwrap(),
        };
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        match vis_idx {
            0 => prop_assert!(matches!(field.vis, syn::Visibility::Inherited)),
            1 => prop_assert!(matches!(field.vis, syn::Visibility::Public(_))),
            _ => prop_assert!(matches!(field.vis, syn::Visibility::Restricted(_))),
        }
    }
}

// ── 14. Skip attr is list-style (has parenthesized args) ────────────────────

proptest! {
    #[test]
    fn skip_attr_is_list_style(idx in 0usize..=3) {
        let defaults = ["false", "0", "None", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert!(matches!(attr.meta, syn::Meta::List(_)));
    }
}

// ── 15. Skip does not carry leaf annotation ─────────────────────────────────

proptest! {
    #[test]
    fn skip_field_has_no_leaf(idx in 0usize..=3) {
        let defaults = ["false", "0", "true", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 16. Skip field name preserved with random names ─────────────────────────

proptest! {
    #[test]
    fn skip_field_name_preserved(idx in 0usize..=4) {
        let names = ["visited", "cached", "line_no", "my_metadata", "is_valid"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(false)]
                #ident: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), names[idx]);
    }
}

// ── 17. Skip with derive on struct does not interfere ───────────────────────

proptest! {
    #[test]
    fn skip_with_derive_attrs(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[derive(Debug, Clone)]
            pub struct N {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(#def_tokens)]
                checked: bool,
            }
        }).unwrap();
        let derive_count = s.attrs.iter().filter(|a| {
            a.path().segments.iter().next().is_some_and(|s| s.ident == "derive")
        }).count();
        prop_assert_eq!(derive_count, 1);
        prop_assert_eq!(count_skip_fields(&s), 1);
    }
}

// ── 18. Skip coexists with extra annotation in grammar module ───────────────

proptest! {
    #[test]
    fn skip_coexists_with_extra(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                    #[adze::skip(#def_tokens)]
                    meta: bool,
                }
                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }
            }
        });
        let root = find_struct_in_mod(&m, "Root").unwrap();
        prop_assert!(root.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        ));
        let has_extra = module_items(&m).iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else { false }
        });
        prop_assert!(has_extra);
    }
}

// ── 19. Skip coexists with word annotation in grammar module ────────────────

proptest! {
    #[test]
    fn skip_coexists_with_word(idx in 0usize..=2) {
        let defaults = ["false", "0", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    ident: Identifier,
                    #[adze::skip(#def_tokens)]
                    checked: bool,
                }
                #[adze::word]
                pub struct Identifier {
                    #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                    name: String,
                }
            }
        });
        let root = find_struct_in_mod(&m, "Root").unwrap();
        prop_assert!(root.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        ));
        let has_word = module_items(&m).iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else { false }
        });
        prop_assert!(has_word);
    }
}

// ── 20. Skip coexists with precedence annotations ───────────────────────────

proptest! {
    #[test]
    fn skip_with_precedence(prec_level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec_level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("arith")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    Meta {
                        #[adze::leaf(pattern = r"\d+")]
                        value: String,
                        #[adze::skip(false)]
                        annotated: bool,
                    },
                }
            }
        });
        let expr = find_enum_in_mod(&m, "Expr").unwrap();
        let has_prec = expr.variants.iter().any(|v|
            v.attrs.iter().any(|a| is_adze_attr(a, "prec_left"))
        );
        prop_assert!(has_prec);
        let meta = expr.variants.iter().find(|v| v.ident == "Meta").unwrap();
        if let Fields::Named(ref named) = meta.fields {
            prop_assert!(named.named.iter().any(|f|
                f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
            ));
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 21. All fields skipped in struct ────────────────────────────────────────

proptest! {
    #[test]
    fn all_fields_skipped(count in 1usize..=5) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool,
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct AllSkip {
                #(#field_tokens)*
            }
        }).unwrap();
        prop_assert_eq!(count_skip_fields(&s), count);
        let non_skip = s.fields.iter()
            .filter(|f| !f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .count();
        prop_assert_eq!(non_skip, 0);
    }
}

// ── 22. Skip attr count on mixed struct ─────────────────────────────────────

proptest! {
    #[test]
    fn skip_attr_count_in_mixed_struct(n_leaf in 1usize..=3, n_skip in 1usize..=3) {
        let leaf_tokens: Vec<proc_macro2::TokenStream> = (0..n_leaf)
            .map(|i| {
                let name = syn::Ident::new(&format!("l{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #name: String,
                }
            })
            .collect();
        let skip_tokens: Vec<proc_macro2::TokenStream> = (0..n_skip)
            .map(|i| {
                let name = syn::Ident::new(&format!("s{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool,
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Mixed {
                #(#leaf_tokens)*
                #(#skip_tokens)*
            }
        }).unwrap();
        let skip_count = count_skip_fields(&s);
        let leaf_count = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        prop_assert_eq!(skip_count, n_skip);
        prop_assert_eq!(leaf_count, n_leaf);
        prop_assert_eq!(s.fields.iter().count(), n_leaf + n_skip);
    }
}

// ── 23. Skip appears exactly once per field ─────────────────────────────────

proptest! {
    #[test]
    fn skip_appears_exactly_once_per_field(idx in 0usize..=3) {
        let defaults = ["false", "0", "true", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let skip_count = field.attrs.iter().filter(|a| is_adze_attr(a, "skip")).count();
        prop_assert_eq!(skip_count, 1);
    }
}

// ── 24. Skip ordering — before language in module ───────────────────────────

proptest! {
    #[test]
    fn skip_field_in_first_struct_of_module(idx in 0usize..=1) {
        let defaults = ["false", "0"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        // Skip in non-language struct placed before language
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                pub struct Helper {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #[adze::skip(#def_tokens)]
                    meta: bool,
                }
                #[adze::language]
                pub struct Root {
                    child: Helper,
                }
            }
        });
        let helper = find_struct_in_mod(&m, "Helper").unwrap();
        prop_assert!(helper.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        ));
    }
}

// ── 25. Skip with negative integer expressions ──────────────────────────────

proptest! {
    #[test]
    fn skip_negative_integer(val in 1i32..=100) {
        let neg_str = format!("-{val}");
        let neg_tokens: proc_macro2::TokenStream = neg_str.parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#neg_tokens)]
                sentinel: i32,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        let expr_str = skip_expr_str(attr);
        prop_assert!(expr_str.contains(&val.to_string()));
    }
}

// ── 26. Skip with float expressions ─────────────────────────────────────────

proptest! {
    #[test]
    fn skip_float_expression(idx in 0usize..=3) {
        let floats = ["0.0", "1.5", "3.14", "0.001"];
        let float_tokens: proc_macro2::TokenStream = floats[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#float_tokens)]
                score: f64,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), floats[idx]);
    }
}

// ── 27. Skip with char expressions ──────────────────────────────────────────

proptest! {
    #[test]
    fn skip_char_expression(idx in 0usize..=3) {
        let chars = ["'a'", "'x'", "'0'", "'_'"];
        let char_tokens: proc_macro2::TokenStream = chars[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#char_tokens)]
                tag: char,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), chars[idx]);
    }
}

// ── 28. Skip field names are distinct in multi-skip struct ──────────────────

proptest! {
    #[test]
    fn skip_field_names_distinct(count in 2usize..=5) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("skip_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool,
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #(#field_tokens)*
            }
        }).unwrap();
        let names = skip_field_names(&s);
        let unique: std::collections::HashSet<_> = names.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 29. Skip preserved in output token stream round-trip ────────────────────

proptest! {
    #[test]
    fn skip_preserved_in_token_stream(idx in 0usize..=3) {
        let defaults = ["false", "42", "None", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        // Round-trip through token stream
        let output = s.to_token_stream().to_string();
        prop_assert!(output.contains("adze :: skip"));
        prop_assert!(output.contains(defaults[idx]));
    }
}

// ── 30. Skip ordering — skip fields preserve insertion order ────────────────

proptest! {
    #[test]
    fn skip_fields_preserve_insertion_order(count in 2usize..=5) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("field_{i}"), proc_macro2::Span::call_site());
                let val = proc_macro2::Literal::usize_unsuffixed(i);
                quote::quote! {
                    #[adze::skip(#val)]
                    #name: usize,
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Ordered {
                #(#field_tokens)*
            }
        }).unwrap();
        let names = skip_field_names(&s);
        for i in 0..count {
            prop_assert_eq!(&names[i], &format!("field_{i}"));
        }
    }
}

// ── 31. Skip with Option<T> type variations ─────────────────────────────────

proptest! {
    #[test]
    fn skip_option_type_variations(idx in 0usize..=3) {
        let inner_types = ["i32", "String", "bool", "u64"];
        let inner_tokens: proc_macro2::TokenStream = inner_types[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(None)]
                maybe: Option<#inner_tokens>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert!(ty_str.contains("Option"));
        prop_assert!(ty_str.contains(inner_types[idx]));
    }
}

// ── 32. Skip with Vec<T> type variations ────────────────────────────────────

proptest! {
    #[test]
    fn skip_vec_type_variations(idx in 0usize..=3) {
        let inner_types = ["i32", "String", "u8", "bool"];
        let inner_tokens: proc_macro2::TokenStream = inner_types[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(Vec::new())]
                items: Vec<#inner_tokens>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), "Vec :: new ()");
    }
}

// ── 33. Skip on struct with generic-like complex type ───────────────────────

proptest! {
    #[test]
    fn skip_complex_default_expressions(idx in 0usize..=3) {
        let exprs = [
            "Default::default()",
            "Vec::with_capacity(16)",
            "String::from(\"\")",
            "Some(0)",
        ];
        let expected = [
            "Default :: default ()",
            "Vec :: with_capacity (16)",
            "String :: from (\"\")",
            "Some (0)",
        ];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                data: String,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 34. Skip struct name variations ─────────────────────────────────────────

proptest! {
    #[test]
    fn skip_struct_name_preserved(idx in 0usize..=4) {
        let names = ["Node", "Token", "Statement", "MyAst", "ParseResult"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct #ident {
                #[adze::skip(false)]
                meta: bool,
            }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), names[idx]);
        prop_assert_eq!(count_skip_fields(&s), 1);
    }
}

// ── 35. Skip in multiple enum variants ──────────────────────────────────────

proptest! {
    #[test]
    fn skip_in_multiple_enum_variants(n_variants in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_variants)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let field_name = syn::Ident::new(&format!("val_{i}"), proc_macro2::Span::call_site());
                let skip_name = syn::Ident::new(&format!("skip_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #name {
                        #[adze::leaf(pattern = r"\w+")]
                        #field_name: String,
                        #[adze::skip(false)]
                        #skip_name: bool,
                    }
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Multi {
                #(#variant_tokens),*
            }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_variants);
        for i in 0..n_variants {
            if let Fields::Named(ref named) = e.variants[i].fields {
                let has_skip = named.named.iter().any(|f|
                    f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
                );
                prop_assert!(has_skip, "Variant {} should have skip field", i);
            } else {
                prop_assert!(false, "Expected named fields for variant {}", i);
            }
        }
    }
}

// ── 36. Skip expansion determinism — repeated parse yields identical output ─

proptest! {
    #[test]
    fn skip_expansion_deterministic(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let mk = || -> String {
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct N {
                    #[adze::skip(#def_tokens)]
                    f: bool,
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }).unwrap();
            s.to_token_stream().to_string()
        };
        prop_assert_eq!(mk(), mk());
        prop_assert_eq!(mk(), mk());
    }
}

// ── 37. Skip with doc comment on the field ──────────────────────────────────

proptest! {
    #[test]
    fn skip_with_doc_comment_preserved(idx in 0usize..=2) {
        let defaults = ["false", "0", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                /// This field is metadata
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let has_doc = field.attrs.iter().any(|a| a.path().is_ident("doc"));
        let has_skip = field.attrs.iter().any(|a| is_adze_attr(a, "skip"));
        prop_assert!(has_doc);
        prop_assert!(has_skip);
    }
}

// ── 38. Skip with cfg attribute on sibling field ────────────────────────────

proptest! {
    #[test]
    fn skip_coexists_with_cfg_attrs(idx in 0usize..=2) {
        let defaults = ["false", "0", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[cfg(test)]
                #[adze::leaf(pattern = r"\d+")]
                value: String,
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let skip_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "meta")
        ).unwrap();
        prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        let value_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "value")
        ).unwrap();
        let has_cfg = value_field.attrs.iter().any(|a| a.path().is_ident("cfg"));
        prop_assert!(has_cfg);
    }
}

// ── 39. Skip attr path is exactly adze::skip ────────────────────────────────

proptest! {
    #[test]
    fn skip_attr_path_is_adze_skip(idx in 0usize..=3) {
        let defaults = ["false", "0", "None", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        let path_str = attr.path().to_token_stream().to_string();
        prop_assert_eq!(path_str, "adze :: skip");
    }
}

// ── 40. Skip does not carry repeat annotation ───────────────────────────────

proptest! {
    #[test]
    fn skip_field_has_no_repeat(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    }
}

// ── 41. Skip with repeat annotation on sibling field ────────────────────────

proptest! {
    #[test]
    fn skip_coexists_with_repeat_sibling(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::repeat(non_empty = true)]
                items: Vec<String>,
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let repeat_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "items")
        ).unwrap();
        let skip_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "meta")
        ).unwrap();
        prop_assert!(repeat_field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 42. Skip with string literal default expression ─────────────────────────

proptest! {
    #[test]
    fn skip_string_literal_default(idx in 0usize..=3) {
        let strings = ["\"hello\"", "\"\"", "\"test data\"", "\"42\""];
        let str_tokens: proc_macro2::TokenStream = strings[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#str_tokens)]
                label: String,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), strings[idx]);
    }
}

// ── 43. Skip field total attr count is exactly one adze attr ────────────────

proptest! {
    #[test]
    fn skip_field_exactly_one_adze_attr(idx in 0usize..=3) {
        let defaults = ["false", "0", "true", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let adze_count = field.attrs.iter()
            .filter(|a| {
                let segs: Vec<_> = a.path().segments.iter().collect();
                segs.len() == 2 && segs[0].ident == "adze"
            })
            .count();
        prop_assert_eq!(adze_count, 1);
    }
}

// ── 44. Skip on first field of struct ───────────────────────────────────────

proptest! {
    #[test]
    fn skip_on_first_field(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                first: bool,
                #[adze::leaf(pattern = r"\w+")]
                second: String,
            }
        }).unwrap();
        let first = s.fields.iter().next().unwrap();
        prop_assert!(first.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert_eq!(first.ident.as_ref().unwrap().to_string(), "first");
    }
}

// ── 45. Skip on last field of struct ────────────────────────────────────────

proptest! {
    #[test]
    fn skip_on_last_field(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::leaf(pattern = r"\w+")]
                first: String,
                #[adze::skip(#def_tokens)]
                last: bool,
            }
        }).unwrap();
        let last = s.fields.iter().last().unwrap();
        prop_assert!(last.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert_eq!(last.ident.as_ref().unwrap().to_string(), "last");
    }
}

// ── 46. Skip with HashMap default expression ────────────────────────────────

proptest! {
    #[test]
    fn skip_hashmap_default(idx in 0usize..=1) {
        let exprs = ["std::collections::HashMap::new()", "Default::default()"];
        let expected = ["std :: collections :: HashMap :: new ()", "Default :: default ()"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                map: std::collections::HashMap<String, i32>,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 47. Skip with tuple expression default ──────────────────────────────────

proptest! {
    #[test]
    fn skip_tuple_default(idx in 0usize..=2) {
        let exprs = ["(0, false)", "(1, 2, 3)", "(true, 0)"];
        let expected = ["(0 , false)", "(1 , 2 , 3)", "(true , 0)"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                pair: (i32, bool),
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 48. Skip with array expression default ──────────────────────────────────

proptest! {
    #[test]
    fn skip_array_default(idx in 0usize..=2) {
        let exprs = ["[0; 3]", "[1, 2, 3]", "[false; 2]"];
        let expected = ["[0 ; 3]", "[1 , 2 , 3]", "[false ; 2]"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                arr: [i32; 3],
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 49. Skip enum variant with only skip fields ─────────────────────────────

proptest! {
    #[test]
    fn skip_only_fields_in_enum_variant(n_fields in 1usize..=4) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #name: bool,
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                AllSkipped {
                    #(#field_tokens)*
                },
            }
        }).unwrap();
        if let Fields::Named(ref named) = e.variants[0].fields {
            let total = named.named.len();
            let skip_count = named.named.iter()
                .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
                .count();
            prop_assert_eq!(total, n_fields);
            prop_assert_eq!(skip_count, n_fields);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 50. Skip preserves non-adze attributes ──────────────────────────────────

proptest! {
    #[test]
    fn skip_preserves_non_adze_attrs(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[allow(dead_code)]
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let has_allow = field.attrs.iter().any(|a| a.path().is_ident("allow"));
        let has_skip = field.attrs.iter().any(|a| is_adze_attr(a, "skip"));
        prop_assert!(has_allow);
        prop_assert!(has_skip);
        prop_assert_eq!(field.attrs.len(), 2);
    }
}

// ── 51. Skip with delimited annotation on sibling ───────────────────────────

proptest! {
    #[test]
    fn skip_coexists_with_delimited_sibling(idx in 0usize..=2) {
        let defaults = ["false", "0", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    items: Vec<Item>,
                    #[adze::skip(#def_tokens)]
                    count: i32,
                }
                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        let root = find_struct_in_mod(&m, "Root").unwrap();
        let has_delimited = root.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "delimited"))
        );
        let has_skip = root.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        );
        prop_assert!(has_delimited);
        prop_assert!(has_skip);
    }
}

// ── 52. Skip with multiple different default exprs in same struct ───────────

proptest! {
    #[test]
    fn skip_different_defaults_same_struct(idx in 0usize..=2) {
        let pairs = [
            ("false", "0"),
            ("None", "true"),
            ("0", "String::new()"),
        ];
        let (d1, d2) = pairs[idx];
        let d1_tokens: proc_macro2::TokenStream = d1.parse().unwrap();
        let d2_tokens: proc_macro2::TokenStream = d2.parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#d1_tokens)]
                a: bool,
                #[adze::skip(#d2_tokens)]
                b: i32,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        let a_skip = fields[0].attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        let b_skip = fields[1].attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        let a_str = skip_expr_str(a_skip);
        let b_str = skip_expr_str(b_skip);
        prop_assert_ne!(a_str, b_str);
    }
}

// ── 53. Skip round-trip through quote/parse is stable ───────────────────────

proptest! {
    #[test]
    fn skip_quote_parse_roundtrip_stable(idx in 0usize..=3) {
        let defaults = ["false", "42", "None", "true"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let original: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let tokens = original.to_token_stream();
        let reparsed: ItemStruct = syn::parse2(tokens).unwrap();
        let orig_str = original.to_token_stream().to_string();
        let re_str = reparsed.to_token_stream().to_string();
        prop_assert_eq!(orig_str, re_str);
    }
}

// ── 54. Skip with prec_right on sibling enum variant ────────────────────────

proptest! {
    #[test]
    fn skip_with_prec_right(prec in 1i32..=5) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                Assign {
                    #[adze::leaf(pattern = r"\w+")]
                    lhs: String,
                    #[adze::leaf(text = "=")]
                    _eq: (),
                    #[adze::leaf(pattern = r"\w+")]
                    rhs: String,
                },
                Meta {
                    #[adze::skip(false)]
                    flag: bool,
                },
            }
        }).unwrap();
        let assign = &e.variants[0];
        prop_assert!(assign.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
        let meta = &e.variants[1];
        if let Fields::Named(ref named) = meta.fields {
            prop_assert!(named.named[0].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 55. Skip with boolean expression default ────────────────────────────────

proptest! {
    #[test]
    fn skip_boolean_expression(idx in 0usize..=2) {
        let exprs = ["1 > 0", "true && false", "!true"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                flag: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = field.attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        // Expression should parse successfully
        let _expr = skip_expr_str(attr);
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 56. Skip attr args are non-empty ────────────────────────────────────────

proptest! {
    #[test]
    fn skip_attr_args_nonempty(idx in 0usize..=4) {
        let defaults = ["false", "0", "None", "true", "42"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        if let syn::Meta::List(ref list) = attr.meta {
            prop_assert!(!list.tokens.is_empty());
        } else {
            prop_assert!(false, "Expected Meta::List");
        }
    }
}

// ── 57. Skip on Optional<T> sibling without skip ────────────────────────────

proptest! {
    #[test]
    fn skip_with_optional_sibling(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                maybe: Option<String>,
                #[adze::skip(#def_tokens)]
                meta: bool,
            }
        }).unwrap();
        let maybe_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "maybe")
        ).unwrap();
        let meta_field = s.fields.iter().find(|f|
            f.ident.as_ref().is_some_and(|i| i == "meta")
        ).unwrap();
        // Optional field should have no skip
        prop_assert!(!maybe_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        // Meta field should have skip
        prop_assert!(meta_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 58. Skip on Option<T> field with Some default ───────────────────────────

proptest! {
    #[test]
    fn skip_option_some_default(idx in 0usize..=2) {
        let exprs = ["Some(0)", "Some(false)", "Some(true)"];
        let expected = ["Some (0)", "Some (false)", "Some (true)"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                maybe: Option<i32>,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}

// ── 59. Skip in module with multiple structs ────────────────────────────────

proptest! {
    #[test]
    fn skip_in_multiple_structs_in_module(n_skip_structs in 1usize..=3) {
        let struct_tokens: Vec<proc_macro2::TokenStream> = (0..n_skip_structs)
            .map(|i| {
                let name = syn::Ident::new(&format!("Node{i}"), proc_macro2::Span::call_site());
                let field_name = syn::Ident::new(&format!("meta_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
                        value: String,
                        #[adze::skip(false)]
                        #field_name: bool,
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
                    name: String,
                }
                #(#struct_tokens)*
            }
        });
        for i in 0..n_skip_structs {
            let name = format!("Node{i}");
            let s = find_struct_in_mod(&m, &name).unwrap();
            prop_assert!(s.fields.iter().any(|f|
                f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
            ));
        }
    }
}

// ── 60. Skip field does not have delimited annotation ───────────────────────

proptest! {
    #[test]
    fn skip_field_has_no_delimited(idx in 0usize..=2) {
        let defaults = ["false", "0", "None"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#def_tokens)]
                f: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 61. Skip with closure expression default ────────────────────────────────

proptest! {
    #[test]
    fn skip_closure_default(idx in 0usize..=1) {
        let exprs = ["|| false", "|| 0"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                gen: fn() -> bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 62. Skip on enum with mixed variant types ───────────────────────────────

proptest! {
    #[test]
    fn skip_mixed_variant_types(idx in 0usize..=1) {
        let defaults = ["false", "0"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Mixed {
                Unit,
                Tuple(#[adze::leaf(pattern = r"\d+")] String),
                Named {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #[adze::skip(#def_tokens)]
                    meta: bool,
                },
            }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), 3);
        // Only Named variant should have skip
        let named = &e.variants[2];
        if let Fields::Named(ref fields) = named.fields {
            prop_assert!(fields.named.iter().any(|f|
                f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
            ));
        } else {
            prop_assert!(false, "Expected named fields");
        }
        // Unit and Tuple should have no skip
        if let Fields::Unit = e.variants[0].fields {
            // ok
        } else {
            prop_assert!(false, "Expected unit variant");
        }
    }
}

// ── 63. Skip field ordering with interleaved skip and non-skip ──────────────

proptest! {
    #[test]
    fn skip_interleaved_fields_ordering(n in 2usize..=4) {
        // Create alternating leaf/skip fields
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..n * 2)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                if i % 2 == 0 {
                    quote::quote! {
                        #[adze::leaf(pattern = r"\w+")]
                        #name: String,
                    }
                } else {
                    quote::quote! {
                        #[adze::skip(false)]
                        #name: bool,
                    }
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #(#field_tokens)*
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        for i in 0..(n * 2) {
            let has_skip = fields[i].attrs.iter().any(|a| is_adze_attr(a, "skip"));
            let has_leaf = fields[i].attrs.iter().any(|a| is_adze_attr(a, "leaf"));
            if i % 2 == 0 {
                prop_assert!(has_leaf, "Even field {} should be leaf", i);
                prop_assert!(!has_skip, "Even field {} should not be skip", i);
            } else {
                prop_assert!(has_skip, "Odd field {} should be skip", i);
                prop_assert!(!has_leaf, "Odd field {} should not be leaf", i);
            }
        }
    }
}

// ── 64. Skip with external annotation on sibling struct ─────────────────────

proptest! {
    #[test]
    fn skip_coexists_with_external(idx in 0usize..=1) {
        let defaults = ["false", "0"];
        let def_tokens: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                    #[adze::skip(#def_tokens)]
                    meta: bool,
                }
                #[adze::external]
                struct IndentToken;
            }
        });
        let root = find_struct_in_mod(&m, "Root").unwrap();
        prop_assert!(root.fields.iter().any(|f|
            f.attrs.iter().any(|a| is_adze_attr(a, "skip"))
        ));
        let has_external = module_items(&m).iter().any(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "external"))
            } else { false }
        });
        prop_assert!(has_external);
    }
}

// ── 65. Skip with Box type field ────────────────────────────────────────────

proptest! {
    #[test]
    fn skip_box_type_default(idx in 0usize..=1) {
        let exprs = ["Box::new(0)", "Box::new(false)"];
        let expected = ["Box :: new (0)", "Box :: new (false)"];
        let expr_tokens: proc_macro2::TokenStream = exprs[idx].parse().unwrap();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct N {
                #[adze::skip(#expr_tokens)]
                boxed: Box<i32>,
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap()
            .attrs.iter().find(|a| is_adze_attr(a, "skip")).unwrap();
        prop_assert_eq!(skip_expr_str(attr), expected[idx]);
    }
}
