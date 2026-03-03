#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::prec]`, `#[adze::prec_left]`, `#[adze::prec_right]`
//! annotations in adze-macro.
//!
//! Uses proptest to generate randomized precedence levels, variant counts, and
//! annotation combinations, then verifies that syn correctly parses and preserves
//! the precedence attributes, their values, and their interaction with other
//! grammar annotations.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemEnum, parse_quote};

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

fn prec_value(attr: &Attribute) -> i32 {
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        i.base10_parse::<i32>().unwrap()
    } else {
        panic!("Expected int literal in precedence attribute");
    }
}

/// Returns (variant_name, attr_kind, prec_level) for all variants with prec attrs.
fn extract_prec_info(e: &ItemEnum) -> Vec<(String, String, i32)> {
    e.variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if let Some(attr) = v.attrs.iter().find(|a| is_adze_attr(a, kind)) {
                    return Some((v.ident.to_string(), kind.to_string(), prec_value(attr)));
                }
            }
            None
        })
        .collect()
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

// ── 1. prec annotation detected on variant with random level ────────────────

proptest! {
    #[test]
    fn prec_annotation_detected_on_variant(level in 0i32..=50) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec(#lit)]
                Compare(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[1].attrs);
        prop_assert!(names.contains(&"prec".to_string()));
        prop_assert!(!names.contains(&"prec_left".to_string()));
        prop_assert!(!names.contains(&"prec_right".to_string()));
    }
}

// ── 2. prec_left annotation detected on variant with random level ───────────

proptest! {
    #[test]
    fn prec_left_annotation_detected_on_variant(level in 0i32..=50) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[1].attrs);
        prop_assert!(names.contains(&"prec_left".to_string()));
        prop_assert!(!names.contains(&"prec".to_string()));
        prop_assert!(!names.contains(&"prec_right".to_string()));
    }
}

// ── 3. prec_right annotation detected on variant with random level ──────────

proptest! {
    #[test]
    fn prec_right_annotation_detected_on_variant(level in 0i32..=50) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_right(#lit)]
                Cons(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[1].attrs);
        prop_assert!(names.contains(&"prec_right".to_string()));
        prop_assert!(!names.contains(&"prec".to_string()));
        prop_assert!(!names.contains(&"prec_left".to_string()));
    }
}

// ── 4. Precedence value extraction for prec ─────────────────────────────────

proptest! {
    #[test]
    fn prec_value_extraction(level in 0i32..=999) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec(#lit)]
                V(i32)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec"))
            .unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 5. Precedence value extraction for prec_left ────────────────────────────

proptest! {
    #[test]
    fn prec_left_value_extraction(level in 0i32..=999) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec_left"))
            .unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 6. Precedence value extraction for prec_right ───────────────────────────

proptest! {
    #[test]
    fn prec_right_value_extraction(level in 0i32..=999) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_right(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec_right"))
            .unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 7. Multiple prec_left variants with ascending levels ────────────────────

proptest! {
    #[test]
    fn multiple_prec_left_ascending(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
                quote::quote! {
                    #[adze::prec_left(#lit)]
                    #name(Box<Expr>, Box<Expr>)
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #(#variant_tokens),*
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), count);
        for i in 0..count {
            prop_assert_eq!(info[i].2, (i as i32) + 1);
            prop_assert_eq!(&info[i].1, "prec_left");
        }
    }
}

// ── 8. Multiple prec_right variants with ascending levels ───────────────────

proptest! {
    #[test]
    fn multiple_prec_right_ascending(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let lit = proc_macro2::Literal::i32_unsuffixed(((i + 1) * 10) as i32);
                quote::quote! {
                    #[adze::prec_right(#lit)]
                    #name(Box<Expr>, Box<Expr>)
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #(#variant_tokens),*
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), count);
        for i in 0..count {
            prop_assert_eq!(info[i].2, ((i + 1) * 10) as i32);
            prop_assert_eq!(&info[i].1, "prec_right");
        }
    }
}

// ── 9. Precedence on named-field variant ────────────────────────────────────

proptest! {
    #[test]
    fn prec_on_named_field_variant(level in 1i32..=20, n_fields in 1usize..=4) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let fields: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: Box<Expr> }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                BinOp { #(#fields),* }
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[1].fields, Fields::Named(_)));
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
        if let Fields::Named(ref n) = e.variants[1].fields {
            prop_assert_eq!(n.named.len(), n_fields);
        }
    }
}

// ── 10. Mixed prec types in same enum ───────────────────────────────────────

proptest! {
    #[test]
    fn mixed_prec_types_in_enum(
        p_level in 1i32..=10,
        pl_level in 1i32..=10,
        pr_level in 1i32..=10,
    ) {
        let p_lit = proc_macro2::Literal::i32_unsuffixed(p_level);
        let pl_lit = proc_macro2::Literal::i32_unsuffixed(pl_level);
        let pr_lit = proc_macro2::Literal::i32_unsuffixed(pr_level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec(#p_lit)]
                Eq(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#pl_lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_right(#pr_lit)]
                Assign(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 3);
        prop_assert_eq!(&info[0], &("Eq".into(), "prec".into(), p_level));
        prop_assert_eq!(&info[1], &("Add".into(), "prec_left".into(), pl_level));
        prop_assert_eq!(&info[2], &("Assign".into(), "prec_right".into(), pr_level));
    }
}

// ── 11. Precedence with large numeric values ────────────────────────────────

proptest! {
    #[test]
    fn prec_with_large_values(level in 100i32..=10000) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 12. Prec annotation does not affect variant count ───────────────────────

proptest! {
    #[test]
    fn prec_does_not_affect_variant_count(
        n_plain in 1usize..=4,
        n_prec in 1usize..=4,
    ) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_plain {
            let name = syn::Ident::new(&format!("Plain{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(i32) });
        }
        for i in 0..n_prec {
            let name = syn::Ident::new(&format!("Prec{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec_left(#lit)]
                #name(i32, i32)
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_plain + n_prec);
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), n_prec);
    }
}

// ── 13. Prec with zero level is valid ───────────────────────────────────────

proptest! {
    #[test]
    fn prec_zero_level_valid(kind_idx in 0usize..=2) {
        let kinds = ["prec", "prec_left", "prec_right"];
        let kind = kinds[kind_idx];
        let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::#kind_ident(0)]
                V(i32)
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, 0);
        prop_assert_eq!(&info[0].1, kind);
    }
}

// ── 14. Same prec level on multiple variants ────────────────────────────────

proptest! {
    #[test]
    fn same_level_on_multiple_variants(count in 2usize..=5, level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::prec_left(#lit)]
                    #name(Box<Expr>, Box<Expr>)
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #(#variant_tokens),*
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), count);
        for entry in &info {
            prop_assert_eq!(entry.2, level);
        }
    }
}

// ── 15. Prec coexists with leaf on variant fields ───────────────────────────

proptest! {
    #[test]
    fn prec_coexists_with_leaf_fields(level in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(#lit)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>,
                ),
            }
        }).unwrap();
        // Variant-level prec_left
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
        // Field-level leaf on the operator
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 16. Prec on unit variant with leaf ──────────────────────────────────────

proptest! {
    #[test]
    fn prec_on_unit_variant_with_leaf(level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Token {
                #[adze::prec(#lit)]
                #[adze::leaf(text = "and")]
                And,
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[0].fields, Fields::Unit));
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert!(names.contains(&"prec".to_string()));
        prop_assert!(names.contains(&"leaf".to_string()));
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 17. Prec attr kind preserved across all three kinds ─────────────────────

proptest! {
    #[test]
    fn prec_attr_kind_preserved(kind_idx in 0usize..=2, level in 1i32..=50) {
        let kinds = ["prec", "prec_left", "prec_right"];
        let kind = kinds[kind_idx];
        let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::#kind_ident(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert_eq!(names.len(), 1);
        prop_assert_eq!(&names[0], kind);
    }
}

// ── 18. Prec value round-trips through parse_args ───────────────────────────

proptest! {
    #[test]
    fn prec_value_roundtrip_parse_args(level in 0i32..=500) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(i32)
            }
        }).unwrap();
        let attr = &e.variants[0].attrs[0];
        let expr: syn::Expr = attr.parse_args().unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(ref i), .. }) = expr {
            prop_assert_eq!(i.base10_parse::<i32>().unwrap(), level);
            prop_assert_eq!(i.to_string(), level.to_string());
        } else {
            prop_assert!(false, "Expected int literal expression");
        }
    }
}

// ── 19. Field types preserved alongside prec annotation ─────────────────────

proptest! {
    #[test]
    fn field_types_preserved_with_prec(n_box in 1usize..=3, level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for _ in 0..n_box {
            fields.push(quote::quote! { Box<Expr> });
        }
        // Add an operator field
        fields.push(quote::quote! { #[adze::leaf(text = "+")] () });
        for _ in 0..n_box {
            fields.push(quote::quote! { Box<Expr> });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                Op(#(#fields),*)
            }
        }).unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), n_box * 2 + 1);
        for i in 0..n_box {
            prop_assert_eq!(&types[i], "Box < Expr >");
        }
        prop_assert_eq!(&types[n_box], "()");
        for i in 0..n_box {
            prop_assert_eq!(&types[n_box + 1 + i], "Box < Expr >");
        }
    }
}

// ── 20. Prec on named variant preserves field names ─────────────────────────

proptest! {
    #[test]
    fn prec_named_variant_preserves_names(n_fields in 2usize..=5, level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let expected_names: Vec<String> = (0..n_fields)
            .map(|i| format!("field_{i}"))
            .collect();
        let fields: Vec<proc_macro2::TokenStream> = expected_names.iter()
            .map(|name| {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                quote::quote! { #ident: Box<Expr> }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                BinOp { #(#fields),* }
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            let actual_names: Vec<String> = n.named.iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            prop_assert_eq!(actual_names, expected_names);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 21. Descending prec levels preserved ────────────────────────────────────

proptest! {
    #[test]
    fn descending_prec_levels_preserved(count in 2usize..=5) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let level = ((count - i) * 10) as i32;
                let lit = proc_macro2::Literal::i32_unsuffixed(level);
                quote::quote! {
                    #[adze::prec_left(#lit)]
                    #name(Box<Expr>, Box<Expr>)
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #(#variant_tokens),*
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), count);
        // Verify descending order
        for i in 0..count - 1 {
            prop_assert!(info[i].2 > info[i + 1].2);
        }
    }
}

// ── 22. Prec on variant with Option field ───────────────────────────────────

proptest! {
    #[test]
    fn prec_on_variant_with_option_field(level in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_right(#lit)]
                Conditional(
                    Box<Expr>,
                    #[adze::leaf(text = "?")]
                    (),
                    Option<Box<Expr>>,
                ),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(&info[0], &("Conditional".into(), "prec_right".into(), level));
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            let has_option = u.unnamed.iter()
                .any(|f| f.ty.to_token_stream().to_string().starts_with("Option"));
            prop_assert!(has_option);
        }
    }
}

// ── 23. Prec on variant with Vec field ──────────────────────────────────────

proptest! {
    #[test]
    fn prec_on_variant_with_vec_field(level in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Call(
                    Box<Expr>,
                    #[adze::leaf(text = "(")]
                    (),
                    Vec<Expr>,
                    #[adze::leaf(text = ")")]
                    (),
                ),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            let has_vec = u.unnamed.iter()
                .any(|f| f.ty.to_token_stream().to_string().contains("Vec"));
            prop_assert!(has_vec);
        }
    }
}

// ── 24. Non-prec variants excluded from extract_prec_info ───────────────────

proptest! {
    #[test]
    fn non_prec_variants_excluded(n_plain in 1usize..=4, n_prec in 1usize..=3) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_plain {
            let name = syn::Ident::new(&format!("P{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(i32) });
        }
        for i in 0..n_prec {
            let name = syn::Ident::new(&format!("A{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec_left(#lit)]
                #name(i32, i32)
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), n_prec);
        // All extracted entries should be from annotated variants
        for entry in &info {
            prop_assert!(entry.0.starts_with('A'));
        }
    }
}

// ── 25. Prec attr ordering relative to leaf ─────────────────────────────────

proptest! {
    #[test]
    fn prec_before_leaf_order(level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Token {
                #[adze::prec(#lit)]
                #[adze::leaf(text = "not")]
                Not,
            }
        }).unwrap();
        let attrs = adze_attr_names(&e.variants[0].attrs);
        prop_assert_eq!(attrs, vec!["prec", "leaf"]);
    }
}

// ── 26. Leaf before prec attr order ─────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_before_prec_order(level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Token {
                #[adze::leaf(text = "or")]
                #[adze::prec_right(#lit)]
                Or,
            }
        }).unwrap();
        let attrs = adze_attr_names(&e.variants[0].attrs);
        prop_assert_eq!(attrs, vec!["leaf", "prec_right"]);
        // Both discoverable regardless of order
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
    }
}

// ── 27. All three prec kinds have distinct attr names ───────────────────────

proptest! {
    #[test]
    fn all_three_prec_kinds_distinct(
        p in 1i32..=10,
        pl in 1i32..=10,
        pr in 1i32..=10,
    ) {
        let p_lit = proc_macro2::Literal::i32_unsuffixed(p);
        let pl_lit = proc_macro2::Literal::i32_unsuffixed(pl);
        let pr_lit = proc_macro2::Literal::i32_unsuffixed(pr);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec(#p_lit)]
                A(i32),
                #[adze::prec_left(#pl_lit)]
                B(i32),
                #[adze::prec_right(#pr_lit)]
                C(i32),
            }
        }).unwrap();
        let all_names: Vec<String> = e.variants.iter()
            .flat_map(|v| adze_attr_names(&v.attrs))
            .collect();
        let expected = vec!["prec", "prec_left", "prec_right"];
        prop_assert_eq!(&all_names, &expected);
        let set: std::collections::HashSet<&String> = all_names.iter().collect();
        prop_assert_eq!(set.len(), 3);
    }
}

// ── 28. Prec with language attribute on enum ────────────────────────────────

proptest! {
    #[test]
    fn prec_with_language_attribute(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 29. Mixed associativity at same prec level ──────────────────────────────

proptest! {
    #[test]
    fn mixed_associativity_at_same_level(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_right(#lit)]
                Assign(Box<Expr>, Box<Expr>),
                #[adze::prec(#lit)]
                Eq(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 3);
        for entry in &info {
            prop_assert_eq!(entry.2, level);
        }
        let kinds: Vec<&str> = info.iter().map(|i| i.1.as_str()).collect();
        prop_assert_eq!(kinds, vec!["prec_left", "prec_right", "prec"]);
    }
}

// ── 30. Prec attr on variant with single field ──────────────────────────────

proptest! {
    #[test]
    fn prec_on_single_field_variant(kind_idx in 0usize..=2, level in 1i32..=30) {
        let kinds = ["prec", "prec_left", "prec_right"];
        let kind = kinds[kind_idx];
        let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::#kind_ident(#lit)]
                Unary(Box<E>)
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(&info[0].1, kind);
        prop_assert_eq!(info[0].2, level);
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), 1);
        }
    }
}

// ── 31. Prec variant ident preserved ────────────────────────────────────────

proptest! {
    #[test]
    fn prec_variant_ident_preserved(idx in 0usize..=5, level in 1i32..=10) {
        let ident_names = ["Add", "Sub", "Mul", "Div", "Pow", "Mod"];
        let name = syn::Ident::new(ident_names[idx], proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                #name(i32, i32)
            }
        }).unwrap();
        prop_assert_eq!(e.variants[0].ident.to_string(), ident_names[idx]);
        let info = extract_prec_info(&e);
        prop_assert_eq!(&info[0].0, ident_names[idx]);
    }
}

// ── 32. Prec does not introduce discriminant ────────────────────────────────

proptest! {
    #[test]
    fn prec_does_not_introduce_discriminant(level in 0i32..=50) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                A(i32),
                #[adze::prec_left(#lit)]
                B(i32, i32),
                C(i32),
            }
        }).unwrap();
        for v in &e.variants {
            prop_assert!(v.discriminant.is_none());
        }
    }
}

// ── 33. Multiple prec annotations: count matches expectation ────────────────

proptest! {
    #[test]
    fn prec_annotation_count_matches(
        n_prec in 0usize..=2,
        n_prec_left in 0usize..=2,
        n_prec_right in 0usize..=2,
    ) {
        prop_assume!(n_prec + n_prec_left + n_prec_right >= 1);
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        // Always include a plain variant
        variant_tokens.push(quote::quote! { Lit(i32) });
        for i in 0..n_prec {
            let name = syn::Ident::new(&format!("P{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec(#lit)]
                #name(i32)
            });
        }
        for i in 0..n_prec_left {
            let name = syn::Ident::new(&format!("L{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 10) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec_left(#lit)]
                #name(i32, i32)
            });
        }
        for i in 0..n_prec_right {
            let name = syn::Ident::new(&format!("R{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 20) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec_right(#lit)]
                #name(i32, i32)
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), n_prec + n_prec_left + n_prec_right);
        let prec_count = info.iter().filter(|i| i.1 == "prec").count();
        let left_count = info.iter().filter(|i| i.1 == "prec_left").count();
        let right_count = info.iter().filter(|i| i.1 == "prec_right").count();
        prop_assert_eq!(prec_count, n_prec);
        prop_assert_eq!(left_count, n_prec_left);
        prop_assert_eq!(right_count, n_prec_right);
    }
}
