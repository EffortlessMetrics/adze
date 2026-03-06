#![allow(clippy::needless_range_loop)]

//! Property-based tests for enum variant handling in adze-macro.
//!
//! Uses proptest to generate randomized enum structures and verify that
//! syn correctly parses and preserves variant kinds, field counts,
//! field names, annotations, and other structural properties.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemEnum};

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

fn variant_is_unit(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Unit)
}

fn variant_is_unnamed(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Unnamed(_))
}

fn variant_is_named(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Named(_))
}

fn named_field_names(v: &syn::Variant) -> Vec<String> {
    match &v.fields {
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect(),
        _ => vec![],
    }
}

fn field_type_strings(v: &syn::Variant) -> Vec<String> {
    match &v.fields {
        Fields::Unnamed(u) => u
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unit => vec![],
    }
}

// ── 1. Named field variants detected for random field counts ────────────────

proptest! {
    #[test]
    fn named_field_variant_detected(field_count in 1usize..=5) {
        // Build a named-field variant with field_count fields
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: String }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V { #(#fields),* }
            }
        }).unwrap();
        prop_assert!(variant_is_named(&e.variants[0]));
    }
}

// ── 2. Unnamed (tuple) variants detected for random field counts ────────────

proptest! {
    #[test]
    fn unnamed_variant_detected(field_count in 1usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| quote::quote! { String })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V(#(#fields),*)
            }
        }).unwrap();
        prop_assert!(variant_is_unnamed(&e.variants[0]));
    }
}

// ── 3. Unit variants detected ───────────────────────────────────────────────

proptest! {
    #[test]
    fn unit_variant_detected(count in 1usize..=8) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #(#names),*
            }
        }).unwrap();
        for v in &e.variants {
            prop_assert!(variant_is_unit(v));
        }
    }
}

// ── 4. Mixed variant types preserved ────────────────────────────────────────

proptest! {
    #[test]
    fn mixed_variant_types_preserved(
        n_unit in 1usize..=3,
        n_tuple in 1usize..=3,
        n_named in 1usize..=3,
    ) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_unit {
            let name = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name });
        }
        for i in 0..n_tuple {
            let name = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(String) });
        }
        for i in 0..n_named {
            let name = syn::Ident::new(&format!("N{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name { x: i32 } });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        let total = n_unit + n_tuple + n_named;
        prop_assert_eq!(e.variants.len(), total);
        for i in 0..n_unit {
            prop_assert!(variant_is_unit(&e.variants[i]));
        }
        for i in 0..n_tuple {
            prop_assert!(variant_is_unnamed(&e.variants[n_unit + i]));
        }
        for i in 0..n_named {
            prop_assert!(variant_is_named(&e.variants[n_unit + n_tuple + i]));
        }
    }
}

// ── 5. Variant count preservation ───────────────────────────────────────────

proptest! {
    #[test]
    fn variant_count_preserved(count in 1usize..=10) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #(#names(String)),*
            }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 6. Field name extraction from named variants ────────────────────────────

proptest! {
    #[test]
    fn field_name_extraction(field_count in 1usize..=6) {
        let expected_names: Vec<String> = (0..field_count)
            .map(|i| format!("field_{i}"))
            .collect();
        let fields: Vec<proc_macro2::TokenStream> = expected_names.iter()
            .map(|name| {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                quote::quote! { #ident: i32 }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V { #(#fields),* }
            }
        }).unwrap();
        let actual_names = named_field_names(&e.variants[0]);
        prop_assert_eq!(actual_names, expected_names);
    }
}

// ── 7. Variant with single leaf annotation ──────────────────────────────────

proptest! {
    #[test]
    fn variant_with_leaf_annotation(idx in 0usize..=4) {
        let texts = ["if", "else", "while", "for", "return"];
        let text = texts[idx];
        let name = syn::Ident::new(&format!("K{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum K {
                #[adze::leaf(text = #text)]
                #name
            }
        }).unwrap();
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(variant_is_unit(&e.variants[0]));
    }
}

// ── 8. Multiple annotations per variant ─────────────────────────────────────

proptest! {
    #[test]
    fn multiple_annotations_per_variant(prec_val in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec_val);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                #[adze::leaf(text = "+")]
                Add
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert!(names.contains(&"prec_left".to_string()));
        prop_assert!(names.contains(&"leaf".to_string()));
        prop_assert_eq!(names.len(), 2);
    }
}

// ── 9. Unnamed field count matches ──────────────────────────────────────────

proptest! {
    #[test]
    fn unnamed_field_count_matches(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { i32 })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V(#(#fields),*)
            }
        }).unwrap();
        if let Fields::Unnamed(u) = &e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), count);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 10. Named field count matches ───────────────────────────────────────────

proptest! {
    #[test]
    fn named_field_count_matches(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: u32 }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V { #(#fields),* }
            }
        }).unwrap();
        if let Fields::Named(n) = &e.variants[0].fields {
            prop_assert_eq!(n.named.len(), count);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 11. Variant ident names preserved ───────────────────────────────────────

proptest! {
    #[test]
    fn variant_ident_names_preserved(count in 1usize..=8) {
        let expected: Vec<String> = (0..count).map(|i| format!("Var{i}")).collect();
        let idents: Vec<syn::Ident> = expected.iter()
            .map(|n| syn::Ident::new(n, proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#idents),* }
        }).unwrap();
        let actual: Vec<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        prop_assert_eq!(actual, expected);
    }
}

// ── 12. Precedence values parsed correctly ──────────────────────────────────

proptest! {
    #[test]
    fn precedence_values_parsed(prec in 0i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec_left"))
            .unwrap();
        let expr: syn::Expr = attr.parse_args().unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) = expr {
            prop_assert_eq!(i.base10_parse::<i32>().unwrap(), prec);
        } else {
            prop_assert!(false, "Expected int literal");
        }
    }
}

// ── 13. Leaf text attribute round-trips ─────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_roundtrips(idx in 0usize..=5) {
        let keywords = ["+", "-", "*", "/", "==", "!="];
        let kw = keywords[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #kw)]
                V
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
            attr.parse_args_with(
                syn::punctuated::Punctuated::parse_terminated,
            ).unwrap();
        prop_assert_eq!(params[0].path.to_string(), "text");
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &params[0].expr {
            prop_assert_eq!(s.value(), kw);
        } else {
            prop_assert!(false, "Expected string literal");
        }
    }
}

// ── 14. Unit variants have zero fields ──────────────────────────────────────

proptest! {
    #[test]
    fn unit_variants_have_zero_fields(count in 1usize..=6) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#names),* }
        }).unwrap();
        for v in &e.variants {
            prop_assert_eq!(field_type_strings(v).len(), 0);
        }
    }
}

// ── 15. Field types preserved for unnamed variants ──────────────────────────

proptest! {
    #[test]
    fn field_types_preserved_unnamed(n_string in 0usize..=3, n_i32 in 0usize..=3) {
        prop_assume!(n_string + n_i32 >= 1);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for _ in 0..n_string {
            fields.push(quote::quote! { String });
        }
        for _ in 0..n_i32 {
            fields.push(quote::quote! { i32 });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V(#(#fields),*) }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), n_string + n_i32);
        for i in 0..n_string {
            prop_assert_eq!(&types[i], "String");
        }
        for i in 0..n_i32 {
            prop_assert_eq!(&types[n_string + i], "i32");
        }
    }
}

// ── 16. prec_right annotation detected ──────────────────────────────────────

proptest! {
    #[test]
    fn prec_right_annotation_detected(prec in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_right(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert!(names.contains(&"prec_right".to_string()));
    }
}

// ── 17. prec annotation (no associativity) detected ─────────────────────────

proptest! {
    #[test]
    fn prec_annotation_detected(prec in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec(#lit)]
                V(i32)
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert!(names.contains(&"prec".to_string()));
    }
}

// ── 18. Leaf with pattern attribute round-trips ─────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_roundtrips(idx in 0usize..=3) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"\s+"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V(#[adze::leaf(pattern = #pat)] String)
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = u.unnamed[0].attrs.iter()
                .find(|a| is_adze_attr(a, "leaf"))
                .unwrap();
            let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
                attr.parse_args_with(
                    syn::punctuated::Punctuated::parse_terminated,
                ).unwrap();
            prop_assert_eq!(params[0].path.to_string(), "pattern");
        } else {
            prop_assert!(false, "Expected unnamed");
        }
    }
}

// ── 19. Annotations not on variant don't appear ─────────────────────────────

proptest! {
    #[test]
    fn unannotated_variants_have_no_adze_attrs(count in 1usize..=5) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#names(i32)),* }
        }).unwrap();
        for v in &e.variants {
            prop_assert!(adze_attr_names(&v.attrs).is_empty());
        }
    }
}

// ── 20. Multiple leaf-annotated unit variants ───────────────────────────────

proptest! {
    #[test]
    fn multiple_leaf_unit_variants(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
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
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), count);
        for v in &e.variants {
            prop_assert!(variant_is_unit(v));
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 21. Named variant field types preserved ─────────────────────────────────

proptest! {
    #[test]
    fn named_variant_field_types_preserved(n_fields in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                if i % 2 == 0 {
                    quote::quote! { #name: String }
                } else {
                    quote::quote! { #name: i32 }
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V { #(#fields),* } }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), n_fields);
        for i in 0..n_fields {
            if i % 2 == 0 {
                prop_assert_eq!(&types[i], "String");
            } else {
                prop_assert_eq!(&types[i], "i32");
            }
        }
    }
}

// ── 22. Box<T> field types preserved in unnamed variants ────────────────────

proptest! {
    #[test]
    fn box_field_types_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { Box<Expr> })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr { V(#(#fields),*) }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        for t in &types {
            prop_assert_eq!(t, "Box < Expr >");
        }
    }
}

// ── 23. Vec<T> field types preserved in named variants ──────────────────────

proptest! {
    #[test]
    fn vec_field_types_preserved(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("items{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: Vec<i32> }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V { #(#fields),* } }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        for t in &types {
            prop_assert_eq!(t, "Vec < i32 >");
        }
    }
}

// ── 24. Variant with prec_left + named fields ───────────────────────────────

proptest! {
    #[test]
    fn prec_left_with_named_fields(prec in 1i32..=10, n_fields in 1usize..=3) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let fields: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: i32 }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V { #(#fields),* }
            }
        }).unwrap();
        prop_assert!(variant_is_named(&e.variants[0]));
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert!(names.contains(&"prec_left".to_string()));
        if let Fields::Named(n) = &e.variants[0].fields {
            prop_assert_eq!(n.named.len(), n_fields);
        }
    }
}

// ── 25. Enum with language attribute preserved ──────────────────────────────

proptest! {
    #[test]
    fn language_attribute_preserved(count in 1usize..=5) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum E { #(#names(i32)),* }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(e.variants.len(), count);
    }
}

// ── 26. Mixed annotated and unannotated variants ────────────────────────────

proptest! {
    #[test]
    fn mixed_annotated_and_unannotated(n_annotated in 1usize..=3, n_plain in 1usize..=3) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_annotated {
            let name = syn::Ident::new(&format!("A{i}"), proc_macro2::Span::call_site());
            let text = format!("a{i}");
            variant_tokens.push(quote::quote! {
                #[adze::leaf(text = #text)]
                #name
            });
        }
        for i in 0..n_plain {
            let name = syn::Ident::new(&format!("P{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(i32) });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_annotated + n_plain);
        for i in 0..n_annotated {
            prop_assert!(!adze_attr_names(&e.variants[i].attrs).is_empty());
        }
        for i in 0..n_plain {
            prop_assert!(adze_attr_names(&e.variants[n_annotated + i].attrs).is_empty());
        }
    }
}

// ── 27. Option<T> field types preserved ─────────────────────────────────────

proptest! {
    #[test]
    fn option_field_types_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { Option<String> })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V(#(#fields),*) }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), count);
        for t in &types {
            prop_assert_eq!(t, "Option < String >");
        }
    }
}

// ── 28. Enum name preserved ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn enum_name_preserved(idx in 0usize..=4) {
        let names = ["Expr", "Token", "Stmt", "Type", "Decl"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum #name { A, B }
        }).unwrap();
        prop_assert_eq!(e.ident.to_string(), names[idx]);
    }
}

// ── 29. Triple-annotated variant ────────────────────────────────────────────

proptest! {
    #[test]
    fn triple_annotated_variant(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::language]
                #[adze::prec_left(#lit)]
                #[adze::leaf(text = "+")]
                V
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert_eq!(names.len(), 3);
        prop_assert!(names.contains(&"language".to_string()));
        prop_assert!(names.contains(&"prec_left".to_string()));
        prop_assert!(names.contains(&"leaf".to_string()));
    }
}

// ── 30. Interleaved unit and named variants ─────────────────────────────────

proptest! {
    #[test]
    fn interleaved_unit_and_named(pairs in 1usize..=4) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..pairs {
            let uname = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            let nname = syn::Ident::new(&format!("N{i}"), proc_macro2::Span::call_site());
            let fname = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #uname });
            variant_tokens.push(quote::quote! { #nname { #fname: i32 } });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), pairs * 2);
        for i in 0..pairs {
            prop_assert!(variant_is_unit(&e.variants[i * 2]));
            prop_assert!(variant_is_named(&e.variants[i * 2 + 1]));
        }
    }
}

// ── 31. Leaf annotation on tuple field is on the field, not variant ──────────

proptest! {
    #[test]
    fn leaf_on_field_not_variant(field_count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| quote::quote! { #[adze::leaf(pattern = r"\d+")] String })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V(#(#fields),*) }
        }).unwrap();
        // Variant itself has no adze attrs
        prop_assert!(adze_attr_names(&e.variants[0].attrs).is_empty());
        // But each field does
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            for f in &u.unnamed {
                prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            }
        }
    }
}

// ── 32. Variant discriminant not used (no = expr) ───────────────────────────

proptest! {
    #[test]
    fn variants_have_no_discriminant(count in 1usize..=6) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#names),* }
        }).unwrap();
        for v in &e.variants {
            prop_assert!(v.discriminant.is_none());
        }
    }
}
