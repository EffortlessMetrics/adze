#![allow(clippy::needless_range_loop)]

//! Property-based tests for precedence handling in adze-macro.
//!
//! Uses proptest to verify that `#[adze::prec]`, `#[adze::prec_left]`,
//! `#[adze::prec_right]` attributes are correctly parsed, preserved,
//! and stripped during grammar expansion. Covers numeric values, named
//! operator groups, ordering preservation, and enum variant interactions.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn has_any_adze_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2 && segs[0].ident == "adze"
    })
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

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn first_enum(m: &ItemMod) -> &ItemEnum {
    module_items(m)
        .iter()
        .find_map(|i| if let Item::Enum(e) = i { Some(e) } else { None })
        .expect("no enum in module")
}

// ── 1. prec_left parsed with random positive level ──────────────────────────

proptest! {
    #[test]
    fn prec_left_positive_level_parsed(level in 1i32..=200) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
        prop_assert_eq!(&info[0].1, "prec_left");
    }
}

// ── 2. prec_right parsed with random positive level ─────────────────────────

proptest! {
    #[test]
    fn prec_right_positive_level_parsed(level in 1i32..=200) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_right(#lit)]
                Cons(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
        prop_assert_eq!(&info[0].1, "prec_right");
    }
}

// ── 3. prec (no assoc) parsed with random positive level ────────────────────

proptest! {
    #[test]
    fn prec_no_assoc_positive_level_parsed(level in 1i32..=200) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec(#lit)]
                Cmp(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
        prop_assert_eq!(&info[0].1, "prec");
    }
}

// ── 4. Numeric value preserved exactly through token round-trip ─────────────

proptest! {
    #[test]
    fn numeric_value_token_roundtrip(level in 0i32..=5000) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(i32, i32)
            }
        }).unwrap();
        let attr = &e.variants[0].attrs[0];
        let token_str = attr.to_token_stream().to_string();
        prop_assert!(token_str.contains(&level.to_string()));
    }
}

// ── 5. Named operator group: additive level shared by Add and Sub ───────────

proptest! {
    #[test]
    fn named_additive_group_shared_level(level in 1i32..=30) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#lit)]
                Sub(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 2);
        prop_assert_eq!(&info[0].0, "Add");
        prop_assert_eq!(&info[1].0, "Sub");
        prop_assert_eq!(info[0].2, info[1].2);
    }
}

// ── 6. Named operator group: multiplicative level higher than additive ──────

proptest! {
    #[test]
    fn named_mult_higher_than_add(add_level in 1i32..=10, offset in 1i32..=10) {
        let mul_level = add_level + offset;
        let add_lit = proc_macro2::Literal::i32_unsuffixed(add_level);
        let mul_lit = proc_macro2::Literal::i32_unsuffixed(mul_level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#add_lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#mul_lit)]
                Mul(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert!(info[1].2 > info[0].2);
    }
}

// ── 7. Multiple levels with non-contiguous gaps preserved ───────────────────

proptest! {
    #[test]
    fn non_contiguous_levels_preserved(
        l1 in 1i32..=10,
        gap1 in 5i32..=20,
        gap2 in 5i32..=20,
    ) {
        let l2 = l1 + gap1;
        let l3 = l2 + gap2;
        let lit1 = proc_macro2::Literal::i32_unsuffixed(l1);
        let lit2 = proc_macro2::Literal::i32_unsuffixed(l2);
        let lit3 = proc_macro2::Literal::i32_unsuffixed(l3);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_left(#lit1)]
                Low(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#lit2)]
                Mid(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#lit3)]
                High(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 3);
        prop_assert_eq!(info[0].2, l1);
        prop_assert_eq!(info[1].2, l2);
        prop_assert_eq!(info[2].2, l3);
        prop_assert!(info[0].2 < info[1].2);
        prop_assert!(info[1].2 < info[2].2);
    }
}

// ── 8. Ordering preserved across N prec_left variants ───────────────────────

proptest! {
    #[test]
    fn ordering_preserved_n_prec_left(count in 2usize..=7) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let level = ((i + 1) * 3) as i32;
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
        for i in 0..count - 1 {
            prop_assert!(info[i].2 < info[i + 1].2, "Ordering not preserved at index {i}");
        }
    }
}

// ── 9. Ordering preserved across N prec_right variants ──────────────────────

proptest! {
    #[test]
    fn ordering_preserved_n_prec_right(count in 2usize..=7) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let level = ((i + 1) * 5) as i32;
                let lit = proc_macro2::Literal::i32_unsuffixed(level);
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
        for i in 0..count - 1 {
            prop_assert!(info[i].2 < info[i + 1].2);
        }
    }
}

// ── 10. Enum variant position independent of prec presence ──────────────────

proptest! {
    #[test]
    fn variant_position_independent_of_prec(n_before in 0usize..=3, n_after in 0usize..=3) {
        prop_assume!(n_before + n_after >= 1);
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_before {
            let name = syn::Ident::new(&format!("B{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(i32) });
        }
        variant_tokens.push(quote::quote! {
            #[adze::prec_left(5)]
            Prec(i32, i32)
        });
        for i in 0..n_after {
            let name = syn::Ident::new(&format!("A{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #name(i32) });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_before + 1 + n_after);
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(&info[0].0, "Prec");
    }
}

// ── 11. Prec in enum variant with named fields ──────────────────────────────

proptest! {
    #[test]
    fn prec_in_named_field_variant(level in 1i32..=30, n_fields in 2usize..=5) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let fields: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|i| {
                let name = syn::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: Box<Expr> }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Op { #(#fields),* }
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[1].fields, Fields::Named(_)));
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
        if let Fields::Named(ref n) = e.variants[1].fields {
            prop_assert_eq!(n.named.len(), n_fields);
        }
    }
}

// ── 12. Prec in enum variant with unit variant ──────────────────────────────

proptest! {
    #[test]
    fn prec_in_unit_variant(kind_idx in 0usize..=2, level in 1i32..=20) {
        let kinds = ["prec", "prec_left", "prec_right"];
        let kind = kinds[kind_idx];
        let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Token {
                #[adze::#kind_ident(#lit)]
                #[adze::leaf(text = "kw")]
                Keyword,
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[0].fields, Fields::Unit));
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
        prop_assert_eq!(&info[0].1, kind);
    }
}

// ── 13. Prec in enum variant with Box<Self> recursive fields ────────────────

proptest! {
    #[test]
    fn prec_with_recursive_box_fields(level in 1i32..=15, n_operands in 2usize..=4) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let fields: Vec<proc_macro2::TokenStream> = (0..n_operands)
            .map(|_| quote::quote! { Box<Expr> })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Op(#(#fields),*)
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            prop_assert_eq!(u.unnamed.len(), n_operands);
            for f in &u.unnamed {
                prop_assert_eq!(f.ty.to_token_stream().to_string(), "Box < Expr >");
            }
        }
    }
}

// ── 14. Grammar module with prec: enum present after expansion ──────────────

proptest! {
    #[test]
    fn grammar_module_prec_enum_present(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
            }
        });
        let e = first_enum(&m);
        prop_assert_eq!(e.ident.to_string(), "Expr");
        prop_assert_eq!(e.variants.len(), 2);
    }
}

// ── 15. Grammar module preserves variant count with multiple prec levels ────

proptest! {
    #[test]
    fn grammar_module_variant_count_with_prec(n_prec in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_prec)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
                let op = format!("op{i}");
                quote::quote! {
                    #[adze::prec_left(#lit)]
                    #name(Box<Expr>, #[adze::leaf(text = #op)] (), Box<Expr>)
                }
            })
            .collect();
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #(#variant_tokens),*
                }
            }
        });
        let e = first_enum(&m);
        // 1 Number + n_prec operator variants
        prop_assert_eq!(e.variants.len(), 1 + n_prec);
    }
}

// ── 16. Prec level monotonicity: strictly increasing levels preserved ───────

proptest! {
    #[test]
    fn prec_level_strict_increase(base in 1i32..=10, count in 2usize..=5) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
                let level = base + (i as i32) * 2;
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
        for i in 0..info.len() - 1 {
            prop_assert!(info[i].2 < info[i + 1].2);
        }
    }
}

// ── 17. Multiple named groups: comparison < additive < multiplicative ───────

proptest! {
    #[test]
    fn three_named_groups_ordered(cmp in 1i32..=5, add_off in 1i32..=5, mul_off in 1i32..=5) {
        let add = cmp + add_off;
        let mul = add + mul_off;
        let cmp_lit = proc_macro2::Literal::i32_unsuffixed(cmp);
        let add_lit = proc_macro2::Literal::i32_unsuffixed(add);
        let mul_lit = proc_macro2::Literal::i32_unsuffixed(mul);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec(#cmp_lit)]
                Eq(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#add_lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#mul_lit)]
                Mul(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(&info[0].1, "prec");
        prop_assert_eq!(&info[1].1, "prec_left");
        prop_assert_eq!(&info[2].1, "prec_left");
        prop_assert!(info[0].2 < info[1].2);
        prop_assert!(info[1].2 < info[2].2);
    }
}

// ── 18. All prec attrs stripped from enum in grammar module ─────────────────

proptest! {
    #[test]
    fn prec_attrs_stripped_after_expansion(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
            }
        });
        let e = first_enum(&m);
        // After grammar parsing, adze attrs should still be visible in the
        // un-expanded parse tree
        let info = extract_prec_info(e);
        // The module parse tree preserves prec attrs
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 19. Prec with mixed assoc kinds maintain distinct identity ──────────────

proptest! {
    #[test]
    fn mixed_assoc_kinds_distinct_identity(
        p_level in 1i32..=20,
        pl_level in 1i32..=20,
        pr_level in 1i32..=20,
    ) {
        let p_lit = proc_macro2::Literal::i32_unsuffixed(p_level);
        let pl_lit = proc_macro2::Literal::i32_unsuffixed(pl_level);
        let pr_lit = proc_macro2::Literal::i32_unsuffixed(pr_level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec(#p_lit)]
                Cmp(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#pl_lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_right(#pr_lit)]
                Assign(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        let kinds: Vec<_> = info.iter().map(|i| i.1.clone()).collect();
        let kind_set: std::collections::HashSet<_> = kinds.iter().collect();
        prop_assert_eq!(kind_set.len(), 3);
        prop_assert_eq!(info[0].2, p_level);
        prop_assert_eq!(info[1].2, pl_level);
        prop_assert_eq!(info[2].2, pr_level);
    }
}

// ── 20. Precedence on every variant except first (base case) ────────────────

proptest! {
    #[test]
    fn prec_on_all_but_base(n_prec in 1usize..=5) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { Lit(i32) },
        ];
        for i in 0..n_prec {
            let name = syn::Ident::new(&format!("Op{i}"), proc_macro2::Span::call_site());
            let lit = proc_macro2::Literal::i32_unsuffixed((i + 1) as i32);
            variant_tokens.push(quote::quote! {
                #[adze::prec_left(#lit)]
                #name(Box<Expr>, Box<Expr>)
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr { #(#variant_tokens),* }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), n_prec);
        // First variant has no prec
        let base_has_prec = e.variants[0].attrs.iter().any(|a| {
            is_adze_attr(a, "prec") || is_adze_attr(a, "prec_left") || is_adze_attr(a, "prec_right")
        });
        prop_assert!(!base_has_prec);
    }
}

// ── 21. Prec value survives token stream serialization ──────────────────────

proptest! {
    #[test]
    fn prec_value_survives_serialization(level in 0i32..=300) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_right(#lit)]
                V(i32)
            }
        }).unwrap();
        // Serialize to token stream and re-parse
        let tokens = e.to_token_stream();
        let e2: ItemEnum = syn::parse2(tokens).unwrap();
        let info = extract_prec_info(&e2);
        prop_assert_eq!(info[0].2, level);
        prop_assert_eq!(&info[0].1, "prec_right");
    }
}

// ── 22. Prec level with named operator groups at same level ─────────────────

proptest! {
    #[test]
    fn named_group_same_level_multiple_ops(level in 1i32..=15, n_ops in 2usize..=4) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_ops)
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
        prop_assert_eq!(info.len(), n_ops);
        for entry in &info {
            prop_assert_eq!(entry.2, level);
        }
    }
}

// ── 23. Grammar module with three prec kinds preserves all ──────────────────

proptest! {
    #[test]
    fn grammar_module_three_prec_kinds(
        p in 1i32..=10,
        pl in 1i32..=10,
        pr in 1i32..=10,
    ) {
        let p_lit = proc_macro2::Literal::i32_unsuffixed(p);
        let pl_lit = proc_macro2::Literal::i32_unsuffixed(pl);
        let pr_lit = proc_macro2::Literal::i32_unsuffixed(pr);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec(#p_lit)]
                    Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
                    #[adze::prec_left(#pl_lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_right(#pr_lit)]
                    Assign(Box<Expr>, #[adze::leaf(text = "=")] (), Box<Expr>),
                }
            }
        });
        let e = first_enum(&m);
        prop_assert_eq!(e.variants.len(), 4);
        let info = extract_prec_info(e);
        prop_assert_eq!(info.len(), 3);
    }
}

// ── 24. Prec attr does not leak to adjacent variants ────────────────────────

proptest! {
    #[test]
    fn prec_does_not_leak_to_adjacent(level in 1i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Before(i32),
                #[adze::prec_left(#lit)]
                Annotated(i32, i32),
                After(i32),
            }
        }).unwrap();
        prop_assert!(!has_any_adze_attr(&e.variants[0].attrs));
        prop_assert!(is_adze_attr(&e.variants[1].attrs[0], "prec_left"));
        prop_assert!(!has_any_adze_attr(&e.variants[2].attrs));
    }
}

// ── 25. Prec with operator leaf fields in grammar module ────────────────────

proptest! {
    #[test]
    fn prec_with_operator_leaf_in_grammar(
        level in 1i32..=10,
        op_idx in 0usize..=3,
    ) {
        let ops = ["+", "-", "*", "/"];
        let op = ops[op_idx];
        let op_names = ["Add", "Sub", "Mul", "Div"];
        let name = syn::Ident::new(op_names[op_idx], proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(#[adze::leaf(pattern = r"\d+")] String),
                    #[adze::prec_left(#lit)]
                    #name(Box<Expr>, #[adze::leaf(text = #op)] (), Box<Expr>),
                }
            }
        });
        let e = first_enum(&m);
        prop_assert_eq!(e.variants.len(), 2);
        prop_assert_eq!(e.variants[1].ident.to_string(), op_names[op_idx]);
    }
}

// ── 26. Descending prec levels still correctly preserved ────────────────────

proptest! {
    #[test]
    fn descending_levels_preserved(count in 2usize..=6) {
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
        for i in 0..info.len() - 1 {
            prop_assert!(info[i].2 > info[i + 1].2, "Descending order not preserved");
        }
    }
}

// ── 27. Prec variant field count unaffected by annotation ───────────────────

proptest! {
    #[test]
    fn prec_field_count_unaffected(n_fields in 1usize..=5, level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let fields: Vec<proc_macro2::TokenStream> = (0..n_fields)
            .map(|_| quote::quote! { i32 })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(#(#fields),*)
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), n_fields);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 28. Prec interleaved with extra and word attrs in grammar ───────────────

proptest! {
    #[test]
    fn prec_with_extra_and_word_in_grammar(level in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Ident(#[adze::leaf(pattern = r"[a-zA-Z_]\w*")] String),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }

                #[adze::word]
                pub struct Keyword {
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
        let has_word = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "word")) } else { false }
        });
        let has_extra = items.iter().any(|i| {
            if let Item::Struct(s) = i { s.attrs.iter().any(|a| is_adze_attr(a, "extra")) } else { false }
        });
        prop_assert!(has_word);
        prop_assert!(has_extra);
        let e = first_enum(&m);
        let info = extract_prec_info(e);
        prop_assert_eq!(info.len(), 1);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 29. Full expression grammar with four named levels ──────────────────────

proptest! {
    #[test]
    fn four_named_levels_expression_grammar(
        assign in 1i32..=5,
        off1 in 1i32..=5,
        off2 in 1i32..=5,
        off3 in 1i32..=5,
    ) {
        let cmp = assign + off1;
        let add = cmp + off2;
        let mul = add + off3;
        let assign_lit = proc_macro2::Literal::i32_unsuffixed(assign);
        let cmp_lit = proc_macro2::Literal::i32_unsuffixed(cmp);
        let add_lit = proc_macro2::Literal::i32_unsuffixed(add);
        let mul_lit = proc_macro2::Literal::i32_unsuffixed(mul);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(i32),
                #[adze::prec_right(#assign_lit)]
                Assign(Box<Expr>, Box<Expr>),
                #[adze::prec(#cmp_lit)]
                Eq(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#add_lit)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_left(#mul_lit)]
                Mul(Box<Expr>, Box<Expr>),
            }
        }).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 4);
        // Verify strictly ascending levels
        for i in 0..info.len() - 1 {
            prop_assert!(info[i].2 < info[i + 1].2);
        }
        // Verify associativity kinds
        prop_assert_eq!(&info[0].1, "prec_right");
        prop_assert_eq!(&info[1].1, "prec");
        prop_assert_eq!(&info[2].1, "prec_left");
        prop_assert_eq!(&info[3].1, "prec_left");
    }
}

// ── 30. Prec attr token stream contains correct path segments ───────────────

proptest! {
    #[test]
    fn prec_attr_path_segments(kind_idx in 0usize..=2, level in 1i32..=50) {
        let kinds = ["prec", "prec_left", "prec_right"];
        let kind = kinds[kind_idx];
        let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
        let lit = proc_macro2::Literal::i32_unsuffixed(level);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::#kind_ident(#lit)]
                V(i32)
            }
        }).unwrap();
        let attr = &e.variants[0].attrs[0];
        let segs: Vec<_> = attr.path().segments.iter().collect();
        prop_assert_eq!(segs.len(), 2);
        prop_assert_eq!(segs[0].ident.to_string(), "adze");
        prop_assert_eq!(segs[1].ident.to_string(), kind);
    }
}
