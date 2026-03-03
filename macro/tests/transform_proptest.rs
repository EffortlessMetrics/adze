#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::leaf(transform = ...)]` in adze-macro.
//!
//! Uses proptest to generate randomized transform closures, path formats, and
//! annotation combinations, then verifies that syn correctly parses and
//! preserves the leaf transform attributes through the expansion pipeline.

use adze_common::NameValueExpr;
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Token, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn find_leaf_attr(attrs: &[Attribute]) -> &Attribute {
    attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap()
}

fn has_transform_param(attr: &Attribute) -> bool {
    let params = leaf_params(attr);
    params.iter().any(|p| p.path == "transform")
}

fn extract_transform_tokens(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params
        .iter()
        .find(|p| p.path == "transform")
        .unwrap();
    nv.expr.to_token_stream().to_string()
}

fn extract_param_value(attr: &Attribute, key: &str) -> Option<String> {
    let params = leaf_params(attr);
    params.iter().find(|p| p.path == key).map(|nv| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.expr
        {
            s.value()
        } else {
            nv.expr.to_token_stream().to_string()
        }
    })
}

// ── 1. Transform annotation detected on struct field ────────────────────────

proptest! {
    #[test]
    fn transform_detected_on_struct_field(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.parse::<i32>().unwrap()),
            quote::quote!(|v: &str| v.len()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                value: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 2. Transform function path extraction ───────────────────────────────────

proptest! {
    #[test]
    fn transform_path_extraction(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.to_uppercase()),
            quote::quote!(|v| v.trim().to_string()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+", transform = #closure)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let tokens = extract_transform_tokens(attr);
        prop_assert!(!tokens.is_empty());
    }
}

// ── 3. Transform with various path formats ──────────────────────────────────

proptest! {
    #[test]
    fn transform_various_path_formats(idx in 0usize..=4) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v: &str| v.parse::<u64>().unwrap()),
            quote::quote!(|v| String::from(v)),
            quote::quote!(|v| v.chars().count()),
            quote::quote!(|v| v.to_lowercase()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+", transform = #closure)]
                f: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
    }
}

// ── 4. Transform combined with text ─────────────────────────────────────────

proptest! {
    #[test]
    fn transform_combined_with_text(idx in 0usize..=3) {
        let texts = ["true", "false", "null", "undefined"];
        let text = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #text, transform = |v| v.to_string())]
                tok: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        let has_text = params.iter().any(|p| p.path == "text");
        let has_transform = params.iter().any(|p| p.path == "transform");
        prop_assert!(has_text);
        prop_assert!(has_transform);
    }
}

// ── 5. Transform combined with pattern ──────────────────────────────────────

proptest! {
    #[test]
    fn transform_combined_with_pattern(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-f0-9]+", r"\d+\.\d+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse::<f64>().unwrap())]
                num: f64,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_param_value(attr, "pattern").unwrap(), pat);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 6. Multiple transformed fields ──────────────────────────────────────────

proptest! {
    #[test]
    fn multiple_transformed_fields(count in 2usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    #name: i32
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let transform_count = s.fields.iter()
            .filter(|f| {
                f.attrs.iter().any(|a| is_adze_attr(a, "leaf"))
                    && has_transform_param(find_leaf_attr(&f.attrs))
            })
            .count();
        prop_assert_eq!(transform_count, count);
    }
}

// ── 7. Transform on enum variant fields ─────────────────────────────────────

proptest! {
    #[test]
    fn transform_on_enum_variant_fields(idx in 0usize..=2) {
        let variant_names = ["Number", "Float", "Hex"];
        let vname = syn::Ident::new(variant_names[idx], proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #vname(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }
        }).unwrap();
        let variant = &e.variants[0];
        if let Fields::Unnamed(ref u) = variant.fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert!(has_transform_param(attr));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 8. Transform preservation in expansion ──────────────────────────────────

proptest! {
    #[test]
    fn transform_preserved_in_expansion(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.parse::<i32>().unwrap()),
            quote::quote!(|v: &str| v.len()),
        ];
        let closure = &closures[idx];
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = #closure)]
                        i32
                    ),
                }
            }
        };
        let (_, items) = m.content.as_ref().unwrap();
        let enm = items.iter().find_map(|item| {
            if let syn::Item::Enum(e) = item { Some(e) } else { None }
        }).unwrap();
        let variant = &enm.variants[0];
        if let Fields::Unnamed(ref u) = variant.fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert!(has_transform_param(attr));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 9. Transform closure with typed parameter ───────────────────────────────

proptest! {
    #[test]
    fn transform_closure_with_typed_param(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v: &str| v.parse::<i32>().unwrap()),
            quote::quote!(|v: &str| v.len()),
            quote::quote!(|v: &str| v.to_uppercase()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+", transform = #closure)]
                field: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let tokens = extract_transform_tokens(find_leaf_attr(&field.attrs));
        prop_assert!(tokens.contains("&str") || tokens.contains("& str"));
    }
}

// ── 10. Transform with turbofish syntax ─────────────────────────────────────

proptest! {
    #[test]
    fn transform_with_turbofish(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse::<i32>().unwrap()),
            quote::quote!(|v| v.parse::<u64>().unwrap()),
            quote::quote!(|v| v.parse::<f64>().unwrap()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                num: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let tokens = extract_transform_tokens(attr);
        prop_assert!(tokens.contains("parse"));
    }
}

// ── 11. Transform param order independence ──────────────────────────────────

proptest! {
    #[test]
    fn transform_param_order_pattern_first(idx in 0usize..=1) {
        let patterns = [r"\d+", r"\w+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params[0].path.to_string(), "pattern");
        prop_assert_eq!(params[1].path.to_string(), "transform");
    }
}

// ── 12. Transform param order: transform first ──────────────────────────────

proptest! {
    #[test]
    fn transform_param_order_transform_first(idx in 0usize..=1) {
        let patterns = [r"\d+", r"\w+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(transform = |v| v.parse().unwrap(), pattern = #pat)]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params[0].path.to_string(), "transform");
        prop_assert_eq!(params[1].path.to_string(), "pattern");
    }
}

// ── 13. Enum with mixed transform and non-transform variants ────────────────

proptest! {
    #[test]
    fn enum_mixed_transform_and_plain(n_transform in 1usize..=3, n_plain in 1usize..=3) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_transform {
            let name = syn::Ident::new(&format!("Tr{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! {
                #name(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                )
            });
        }
        for i in 0..n_plain {
            let name = syn::Ident::new(&format!("Pl{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! {
                #name(
                    #[adze::leaf(pattern = r"\w+")]
                    String
                )
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_transform + n_plain);

        let mut transform_count = 0;
        for v in &e.variants {
            if let Fields::Unnamed(ref u) = v.fields {
                if has_transform_param(find_leaf_attr(&u.unnamed[0].attrs)) {
                    transform_count += 1;
                }
            }
        }
        prop_assert_eq!(transform_count, n_transform);
    }
}

// ── 14. Transform on named enum variant fields ──────────────────────────────

proptest! {
    #[test]
    fn transform_on_named_enum_fields(idx in 0usize..=2) {
        let field_names = ["value", "count", "amount"];
        let fname = syn::Ident::new(field_names[idx], proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Parsed {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    #fname: i32,
                }
            }
        }).unwrap();
        let variant = &e.variants[0];
        if let Fields::Named(ref named) = variant.fields {
            let field = &named.named[0];
            let attr = find_leaf_attr(&field.attrs);
            prop_assert!(has_transform_param(attr));
            prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), field_names[idx]);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 15. Transform closure returning different types ─────────────────────────

proptest! {
    #[test]
    fn transform_various_return_types(idx in 0usize..=4) {
        let closures_and_types: Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = vec![
            (quote::quote!(|v| v.parse::<i32>().unwrap()), quote::quote!(i32)),
            (quote::quote!(|v| v.parse::<u64>().unwrap()), quote::quote!(u64)),
            (quote::quote!(|v| v.parse::<f64>().unwrap()), quote::quote!(f64)),
            (quote::quote!(|v| v.len()), quote::quote!(usize)),
            (quote::quote!(|v| v.to_string()), quote::quote!(String)),
        ];
        let (closure, ty) = &closures_and_types[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+", transform = #closure)]
                field: #ty,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 16. Transform on optional field ─────────────────────────────────────────

proptest! {
    #[test]
    fn transform_on_optional_field(idx in 0usize..=1) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.parse::<i32>().unwrap()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                val: Option<i32>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert!(ty_str.contains("Option"));
    }
}

// ── 17. Transform param count ───────────────────────────────────────────────

proptest! {
    #[test]
    fn transform_leaf_has_two_params(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"[0-9]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let params = leaf_params(find_leaf_attr(&field.attrs));
        prop_assert_eq!(params.len(), 2);
    }
}

// ── 18. Transform only (no pattern or text) ─────────────────────────────────

proptest! {
    #[test]
    fn transform_only_leaf(idx in 0usize..=1) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.to_string()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(transform = #closure)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 1);
        prop_assert_eq!(params[0].path.to_string(), "transform");
    }
}

// ── 19. Transform token stream is non-trivial ───────────────────────────────

proptest! {
    #[test]
    fn transform_tokens_contain_closure_marker(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v: &str| v.len()),
            quote::quote!(|v| { let x = v.trim(); x.to_string() }),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let tokens = extract_transform_tokens(find_leaf_attr(&field.attrs));
        // Closure tokens contain the pipe character for parameter binding
        prop_assert!(tokens.contains("|"));
    }
}

// ── 20. Multiple enum variants each with transform ──────────────────────────

proptest! {
    #[test]
    fn multiple_enum_variants_with_transform(count in 2usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #name(
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        i32
                    )
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        for i in 0..count {
            if let Fields::Unnamed(ref u) = e.variants[i].fields {
                prop_assert!(has_transform_param(find_leaf_attr(&u.unnamed[0].attrs)));
            }
        }
    }
}

// ── 21. Transform with block body closure ───────────────────────────────────

proptest! {
    #[test]
    fn transform_block_body_closure(idx in 0usize..=1) {
        let closures = [
            quote::quote!(|v| { v.parse::<i32>().unwrap() }),
            quote::quote!(|v| { let trimmed = v.trim(); trimmed.parse::<i32>().unwrap() }),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                num: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 22. Transform preserves text param value ────────────────────────────────

proptest! {
    #[test]
    fn transform_preserves_text_value(idx in 0usize..=3) {
        let texts = ["+", "-", "*", "/"];
        let text = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #text, transform = |v| v.to_string())]
                op: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_param_value(attr, "text").unwrap(), text);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 23. Transform preserves pattern param value ─────────────────────────────

proptest! {
    #[test]
    fn transform_preserves_pattern_value(idx in 0usize..=3) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"[0-9]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_param_value(attr, "pattern").unwrap(), pat);
    }
}

// ── 24. Transform on unit struct via attribute ──────────────────────────────

#[test]
fn transform_not_on_unit_leaf_struct() {
    // Unit leaf structs use text, not transform — verify parsing still works
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::leaf(text = "hello")]
        pub struct Kw;
    })
    .unwrap();
    let attr = find_leaf_attr(&s.attrs);
    assert!(!has_transform_param(attr));
}

// ── 25. Enum with transform and prec_left ───────────────────────────────────

proptest! {
    #[test]
    fn transform_with_prec_left(idx in 0usize..=1) {
        let ops = ["+", "-"];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = #op)]
                    (),
                    Box<Expr>,
                ),
            }
        }).unwrap();

        // Check Number variant has transform
        let number = &e.variants[0];
        if let Fields::Unnamed(ref u) = number.fields {
            prop_assert!(has_transform_param(find_leaf_attr(&u.unnamed[0].attrs)));
        }

        // Check Add variant has prec_left
        let add = &e.variants[1];
        prop_assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// ── 26. Transform in grammar module ─────────────────────────────────────────

proptest! {
    #[test]
    fn transform_in_grammar_module(idx in 0usize..=1) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.parse::<i32>().unwrap()),
        ];
        let closure = &closures[idx];
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = #closure)]
                    v: i32,
                }
            }
        };
        let (_, items) = m.content.as_ref().unwrap();
        let st = items.iter().find_map(|item| {
            if let syn::Item::Struct(s) = item { Some(s) } else { None }
        }).unwrap();
        let field = st.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_transform_param(attr));
    }
}

// ── 27. Transform expr identity through clone ───────────────────────────────

proptest! {
    #[test]
    fn transform_nve_clone_identity(idx in 0usize..=2) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.len()),
            quote::quote!(|v: &str| v.to_string()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                val: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let params = leaf_params(find_leaf_attr(&field.attrs));
        let nv = params.iter().find(|p| p.path == "transform").unwrap();
        let cloned = nv.clone();
        prop_assert_eq!(nv.path.to_string(), cloned.path.to_string());
        prop_assert_eq!(
            nv.expr.to_token_stream().to_string(),
            cloned.expr.to_token_stream().to_string()
        );
    }
}

// ── 28. Transform combined with text and pattern in same enum ───────────────

proptest! {
    #[test]
    fn transform_text_and_pattern_enum(n_text in 1usize..=2, n_transform in 1usize..=2) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_text {
            let name = syn::Ident::new(&format!("Kw{i}"), proc_macro2::Span::call_site());
            let tv = format!("kw{i}");
            variant_tokens.push(quote::quote! {
                #[adze::leaf(text = #tv)]
                #name
            });
        }
        for i in 0..n_transform {
            let name = syn::Ident::new(&format!("Num{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! {
                #name(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                )
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_text + n_transform);

        // Text variants are unit
        for i in 0..n_text {
            prop_assert!(matches!(e.variants[i].fields, Fields::Unit));
        }
        // Transform variants have the transform param
        for i in 0..n_transform {
            let v = &e.variants[n_text + i];
            if let Fields::Unnamed(ref u) = v.fields {
                prop_assert!(has_transform_param(find_leaf_attr(&u.unnamed[0].attrs)));
            }
        }
    }
}

// ── 29. Transform field alongside skip field ────────────────────────────────

proptest! {
    #[test]
    fn transform_with_skip_field(idx in 0usize..=1) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.parse::<i32>().unwrap()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #closure)]
                value: i32,
                #[adze::skip(false)]
                visited: bool,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert_eq!(fields.len(), 2);
        let attr = find_leaf_attr(&fields[0].attrs);
        prop_assert!(has_transform_param(attr));
        prop_assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 30. Transform param name is always "transform" ──────────────────────────

proptest! {
    #[test]
    fn transform_param_name_is_transform(idx in 0usize..=4) {
        let closures = [
            quote::quote!(|v| v.parse().unwrap()),
            quote::quote!(|v| v.to_string()),
            quote::quote!(|v: &str| v.len()),
            quote::quote!(|v| v.parse::<u32>().unwrap()),
            quote::quote!(|v| v.trim().to_string()),
        ];
        let closure = &closures[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+", transform = #closure)]
                f: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let params = leaf_params(find_leaf_attr(&field.attrs));
        let transform_nv = params.iter().find(|p| p.path == "transform").unwrap();
        prop_assert_eq!(transform_nv.path.to_string(), "transform");
    }
}
