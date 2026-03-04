#![allow(clippy::needless_range_loop)]

//! Property-based tests for generic type handling in adze-macro.
//!
//! Uses proptest to verify that generic wrapper types (Option, Vec, Box) and
//! nested generics are correctly preserved through grammar module parsing and
//! that the adze-common utility functions (`try_extract_inner_type`,
//! `filter_inner_type`, `wrap_leaf_type`) behave correctly for all
//! combinations relevant to grammar field definitions.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
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

fn field_type_str(s: &ItemStruct, field_name: &str) -> String {
    s.fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == field_name))
        .unwrap()
        .ty
        .to_token_stream()
        .to_string()
}

fn struct_field_type_strings(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn is_option_type(ty: &Type) -> bool {
    let skip: HashSet<&str> = HashSet::new();
    let (_, extracted) = try_extract_inner_type(ty, "Option", &skip);
    extracted
}

fn is_vec_type(ty: &Type) -> bool {
    let skip: HashSet<&str> = HashSet::new();
    let (_, extracted) = try_extract_inner_type(ty, "Vec", &skip);
    extracted
}

fn is_box_type(ty: &Type) -> bool {
    let skip: HashSet<&str> = HashSet::new();
    let (_, extracted) = try_extract_inner_type(ty, "Box", &skip);
    extracted
}

fn extract_inner(ty: &Type, wrapper: &str) -> Option<Type> {
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(ty, wrapper, &skip);
    if extracted { Some(inner) } else { None }
}

// ── 1. Option<T> field detected for various inner types ─────────────────────

proptest! {
    #[test]
    fn option_field_detected_for_inner_types(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Option<bool>),
            parse_quote!(Option<u64>),
            parse_quote!(Option<f32>),
        ];
        prop_assert!(is_option_type(&types[idx]));
    }
}

// ── 2. Option<T> inner type correctly extracted ─────────────────────────────

proptest! {
    #[test]
    fn option_inner_type_extracted(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Option<MyNode>),
            parse_quote!(Option<u8>),
        ];
        let expected = ["i32", "String", "MyNode", "u8"];
        let inner = extract_inner(&types[idx], "Option").unwrap();
        prop_assert_eq!(inner.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 3. Vec<T> field detected for various inner types ────────────────────────

proptest! {
    #[test]
    fn vec_field_detected_for_inner_types(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Vec<Number>),
            parse_quote!(Vec<Token>),
        ];
        prop_assert!(is_vec_type(&types[idx]));
    }
}

// ── 4. Vec<T> inner type correctly extracted ────────────────────────────────

proptest! {
    #[test]
    fn vec_inner_type_extracted(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Vec<Number>),
            parse_quote!(Vec<u32>),
        ];
        let expected = ["i32", "String", "Number", "u32"];
        let inner = extract_inner(&types[idx], "Vec").unwrap();
        prop_assert_eq!(inner.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 5. Box<T> field detected for various inner types ────────────────────────

proptest! {
    #[test]
    fn box_field_detected_for_inner_types(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Expr>),
            parse_quote!(Box<String>),
            parse_quote!(Box<i32>),
            parse_quote!(Box<Statement>),
        ];
        prop_assert!(is_box_type(&types[idx]));
    }
}

// ── 6. Box<T> inner type correctly extracted ────────────────────────────────

proptest! {
    #[test]
    fn box_inner_type_extracted(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Expr>),
            parse_quote!(Box<String>),
            parse_quote!(Box<i32>),
            parse_quote!(Box<Statement>),
        ];
        let expected = ["Expr", "String", "i32", "Statement"];
        let inner = extract_inner(&types[idx], "Box").unwrap();
        prop_assert_eq!(inner.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 7. Box<Self> recursive type preserved in enum ───────────────────────────

proptest! {
    #[test]
    fn box_self_recursive_in_enum(idx in 0usize..=2) {
        let variant_names = ["Neg", "Not", "Deref"];
        let vname = syn::Ident::new(variant_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        i32
                    ),
                    #vname(
                        #[adze::leaf(text = "~")]
                        (),
                        Box<Expr>
                    ),
                }
            }
        });
        let e = find_enum_in_mod(&m, "Expr").unwrap();
        let variant = e.variants.iter().find(|v| v.ident == variant_names[idx]).unwrap();
        if let Fields::Unnamed(u) = &variant.fields {
            let box_field = &u.unnamed[1];
            let ty_str = box_field.ty.to_token_stream().to_string();
            prop_assert!(ty_str.contains("Box"));
            prop_assert!(ty_str.contains("Expr"));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 8. Nested generic Option<Vec<T>> extraction via skip_over ───────────────

proptest! {
    #[test]
    fn nested_option_vec_extraction(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<Vec<i32>>),
            parse_quote!(Option<Vec<String>>),
            parse_quote!(Option<Vec<Number>>),
        ];
        let expected_inner = ["i32", "String", "Number"];
        let skip: HashSet<&str> = HashSet::from(["Option"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), expected_inner[idx]);
    }
}

// ── 9. Nested generic Box<Option<T>> extraction via skip_over ───────────────

proptest! {
    #[test]
    fn nested_box_option_extraction(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Option<i32>>),
            parse_quote!(Box<Option<String>>),
            parse_quote!(Box<Option<Expr>>),
        ];
        let expected_inner = ["i32", "String", "Expr"];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), expected_inner[idx]);
    }
}

// ── 10. filter_inner_type strips Box wrapper ────────────────────────────────

proptest! {
    #[test]
    fn filter_strips_box_wrapper(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Expr>),
            parse_quote!(Box<String>),
            parse_quote!(Box<i32>),
            parse_quote!(Box<Node>),
        ];
        let expected = ["Expr", "String", "i32", "Node"];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let filtered = filter_inner_type(&types[idx], &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 11. filter_inner_type strips nested wrappers ────────────────────────────

proptest! {
    #[test]
    fn filter_strips_nested_wrappers(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Box<i32>>),
            parse_quote!(Box<Box<String>>),
            parse_quote!(Box<Box<Expr>>),
        ];
        let expected = ["i32", "String", "Expr"];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let filtered = filter_inner_type(&types[idx], &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 12. wrap_leaf_type wraps non-generic type ───────────────────────────────

proptest! {
    #[test]
    fn wrap_leaf_wraps_plain_types(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(MyNode),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let wrapped = wrap_leaf_type(&types[idx], &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("WithLeaf"), "Expected WithLeaf wrapper, got: {}", s);
    }
}

// ── 13. wrap_leaf_type preserves Option wrapper ─────────────────────────────

proptest! {
    #[test]
    fn wrap_leaf_preserves_option(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Option<String>),
            parse_quote!(Option<MyNode>),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let wrapped = wrap_leaf_type(&types[idx], &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Option"), "Expected Option wrapper, got: {}", s);
        prop_assert!(s.contains("WithLeaf"), "Expected inner WithLeaf, got: {}", s);
    }
}

// ── 14. wrap_leaf_type preserves Vec wrapper ────────────────────────────────

proptest! {
    #[test]
    fn wrap_leaf_preserves_vec(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Vec<Number>),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let wrapped = wrap_leaf_type(&types[idx], &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Vec"), "Expected Vec wrapper, got: {}", s);
        prop_assert!(s.contains("WithLeaf"), "Expected inner WithLeaf, got: {}", s);
    }
}

// ── 15. wrap_leaf_type preserves Box wrapper ────────────────────────────────

proptest! {
    #[test]
    fn wrap_leaf_preserves_box(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<i32>),
            parse_quote!(Box<String>),
            parse_quote!(Box<Expr>),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let wrapped = wrap_leaf_type(&types[idx], &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Box"), "Expected Box wrapper, got: {}", s);
        prop_assert!(s.contains("WithLeaf"), "Expected inner WithLeaf, got: {}", s);
    }
}

// ── 16. Non-generic type passthrough (not detected as Option/Vec/Box) ───────

proptest! {
    #[test]
    fn non_generic_not_detected(idx in 0usize..=4) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(bool),
            parse_quote!(MyCustomType),
            parse_quote!(u64),
        ];
        prop_assert!(!is_option_type(&types[idx]));
        prop_assert!(!is_vec_type(&types[idx]));
        prop_assert!(!is_box_type(&types[idx]));
    }
}

// ── 17. Generic type determinism: same input yields same output ─────────────

proptest! {
    #[test]
    fn try_extract_deterministic(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Box<Expr>),
            parse_quote!(Option<Vec<u32>>),
        ];
        let skip: HashSet<&str> = HashSet::new();
        let (r1, e1) = try_extract_inner_type(&types[idx], "Option", &skip);
        let (r2, e2) = try_extract_inner_type(&types[idx], "Option", &skip);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(
            r1.to_token_stream().to_string(),
            r2.to_token_stream().to_string()
        );
    }
}

// ── 18. filter_inner_type determinism ───────────────────────────────────────

proptest! {
    #[test]
    fn filter_deterministic(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<i32>),
            parse_quote!(Box<Box<String>>),
            parse_quote!(String),
        ];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let r1 = filter_inner_type(&types[idx], &skip);
        let r2 = filter_inner_type(&types[idx], &skip);
        prop_assert_eq!(
            r1.to_token_stream().to_string(),
            r2.to_token_stream().to_string()
        );
    }
}

// ── 19. wrap_leaf_type determinism ──────────────────────────────────────────

proptest! {
    #[test]
    fn wrap_leaf_deterministic(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(Option<String>),
            parse_quote!(Vec<u32>),
            parse_quote!(Box<Expr>),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let r1 = wrap_leaf_type(&types[idx], &skip);
        let r2 = wrap_leaf_type(&types[idx], &skip);
        prop_assert_eq!(
            r1.to_token_stream().to_string(),
            r2.to_token_stream().to_string()
        );
    }
}

// ── 20. Option<T> field preserved in struct module ──────────────────────────

proptest! {
    #[test]
    fn option_field_preserved_in_struct(idx in 0usize..=2) {
        let inner_names = ["i32", "String", "Number"];
        let inner_types: Vec<proc_macro2::TokenStream> = vec![
            quote!(i32), quote!(String), quote!(Number),
        ];
        let inner_ty = &inner_types[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    value: Option<#inner_ty>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        });
        let s = find_struct_in_mod(&m, "Root").unwrap();
        let ty_str = field_type_str(s, "value");
        prop_assert!(ty_str.contains("Option"), "Expected Option in type: {}", ty_str);
    }
}

// ── 21. Vec<T> field preserved in struct module ─────────────────────────────

proptest! {
    #[test]
    fn vec_field_preserved_in_struct(idx in 0usize..=1) {
        let type_names = ["Number", "Token"];
        let ty_ident = syn::Ident::new(type_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    items: Vec<#ty_ident>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }

                pub struct Token {
                    #[adze::leaf(pattern = r"\w+")]
                    t: String,
                }
            }
        });
        let s = find_struct_in_mod(&m, "Root").unwrap();
        let ty_str = field_type_str(s, "items");
        prop_assert!(ty_str.contains("Vec"), "Expected Vec in type: {}", ty_str);
        prop_assert!(ty_str.contains(type_names[idx]), "Expected inner type {} in: {}", type_names[idx], ty_str);
    }
}

// ── 22. Box<T> field preserved in enum variant ──────────────────────────────

proptest! {
    #[test]
    fn box_field_preserved_in_enum_variant(idx in 0usize..=1) {
        let ops = ["+", "-"];
        let vnames = ["Add", "Sub"];
        let op = ops[idx];
        let vname = syn::Ident::new(vnames[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        i32
                    ),
                    #[adze::prec_left(1)]
                    #vname(
                        Box<Expr>,
                        #[adze::leaf(text = #op)]
                        (),
                        Box<Expr>
                    ),
                }
            }
        });
        let e = find_enum_in_mod(&m, "Expr").unwrap();
        let variant = e.variants.iter().find(|v| v.ident == vnames[idx]).unwrap();
        if let Fields::Unnamed(u) = &variant.fields {
            let first_ty = u.unnamed[0].ty.to_token_stream().to_string();
            let last_ty = u.unnamed[2].ty.to_token_stream().to_string();
            prop_assert!(first_ty.contains("Box"), "Expected Box in first field: {}", first_ty);
            prop_assert!(last_ty.contains("Box"), "Expected Box in last field: {}", last_ty);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 23. Option<Vec<T>> nested generic in struct ─────────────────────────────

#[test]
fn option_vec_nested_in_struct_type() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    assert!(is_option_type(&ty));
    let inner = extract_inner(&ty, "Option").unwrap();
    assert!(is_vec_type(&inner));
    let innermost = extract_inner(&inner, "Vec").unwrap();
    assert_eq!(innermost.to_token_stream().to_string(), "i32");
}

// ── 24. Vec<Option<T>> nested generic ───────────────────────────────────────

#[test]
fn vec_option_nested_type() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    assert!(is_vec_type(&ty));
    let inner = extract_inner(&ty, "Vec").unwrap();
    assert!(is_option_type(&inner));
    let innermost = extract_inner(&inner, "Option").unwrap();
    assert_eq!(innermost.to_token_stream().to_string(), "String");
}

// ── 25. Box<Vec<T>> nested generic ──────────────────────────────────────────

#[test]
fn box_vec_nested_type() {
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    assert!(is_box_type(&ty));
    let inner = extract_inner(&ty, "Box").unwrap();
    assert!(is_vec_type(&inner));
    let innermost = extract_inner(&inner, "Vec").unwrap();
    assert_eq!(innermost.to_token_stream().to_string(), "u32");
}

// ── 26. wrap_leaf_type with nested Option<Vec<T>> ───────────────────────────

#[test]
fn wrap_leaf_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    assert!(s.starts_with("Option"), "Expected Option wrapper, got: {s}");
    assert!(s.contains("Vec"), "Expected Vec in wrapped: {s}");
    assert!(s.contains("WithLeaf"), "Expected WithLeaf in wrapped: {s}");
}

// ── 27. Non-generic passthrough in filter_inner_type ────────────────────────

proptest! {
    #[test]
    fn filter_passthrough_non_generic(idx in 0usize..=3) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(MyNode),
            parse_quote!(bool),
        ];
        let expected = ["i32", "String", "MyNode", "bool"];
        let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
        let filtered = filter_inner_type(&types[idx], &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 28. Non-generic passthrough in wrap_leaf_type wraps entirely ────────────

proptest! {
    #[test]
    fn wrap_leaf_wraps_non_generic_entirely(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(String),
            parse_quote!(CustomType),
        ];
        let skip: HashSet<&str> = HashSet::from(["Option", "Vec", "Box", "Spanned"]);
        let wrapped = wrap_leaf_type(&types[idx], &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("adze"), "Expected adze:: prefix, got: {}", s);
        prop_assert!(s.contains("WithLeaf"), "Expected WithLeaf, got: {}", s);
    }
}

// ── 29. try_extract_inner_type returns false for wrong wrapper ──────────────

proptest! {
    #[test]
    fn extract_wrong_wrapper_returns_false(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Box<Expr>),
        ];
        let wrong_wrappers = ["Vec", "Box", "Option"];
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&types[idx], wrong_wrappers[idx], &skip);
        prop_assert!(!extracted);
    }
}

// ── 30. Box<Self> recursive type in named variant field ─────────────────────

#[test]
fn box_self_in_named_variant_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                Neg {
                    #[adze::leaf(text = "!")]
                    _bang: (),
                    value: Box<Expr>,
                }
            }
        }
    });
    let e = find_enum_in_mod(&m, "Expr").unwrap();
    let neg = e.variants.iter().find(|v| v.ident == "Neg").unwrap();
    if let Fields::Named(n) = &neg.fields {
        let value_field = n
            .named
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == "value")
            .unwrap();
        let ty_str = value_field.ty.to_token_stream().to_string();
        assert!(ty_str.contains("Box"));
        assert!(ty_str.contains("Expr"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 31. Multiple generic fields in a single struct ──────────────────────────

#[test]
fn multiple_generic_fields_in_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                opt_val: Option<i32>,
                items: Vec<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let s = find_struct_in_mod(&m, "Root").unwrap();
    let types = struct_field_type_strings(s);
    assert!(
        types[0].contains("Option"),
        "First field should be Option: {}",
        types[0]
    );
    assert!(
        types[1].contains("Vec"),
        "Second field should be Vec: {}",
        types[1]
    );
}

// ── 32. Generic type not confused with similarly-named non-generic type ─────

#[test]
fn generic_not_confused_with_similar_name() {
    let option_ty: Type = parse_quote!(Option<i32>);
    let non_option_ty: Type = parse_quote!(OptionLike);
    assert!(is_option_type(&option_ty));
    assert!(!is_option_type(&non_option_ty));
}

// ── 33. Vec extraction through Box skip ─────────────────────────────────────

proptest! {
    #[test]
    fn vec_extraction_through_box_skip(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Box<Vec<i32>>),
            parse_quote!(Box<Vec<String>>),
            parse_quote!(Box<Vec<Expr>>),
        ];
        let expected = ["i32", "String", "Expr"];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], "Vec", &skip);
        prop_assert!(extracted, "Should extract Vec through Box");
        prop_assert_eq!(inner.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 34. Option extraction through Spanned skip ──────────────────────────────

proptest! {
    #[test]
    fn option_extraction_through_spanned_skip(idx in 0usize..=1) {
        let types: Vec<Type> = vec![
            parse_quote!(Spanned<Option<i32>>),
            parse_quote!(Spanned<Option<String>>),
        ];
        let expected = ["i32", "String"];
        let skip: HashSet<&str> = HashSet::from(["Spanned"]);
        let (inner, extracted) = try_extract_inner_type(&types[idx], "Option", &skip);
        prop_assert!(extracted, "Should extract Option through Spanned");
        prop_assert_eq!(inner.to_token_stream().to_string(), expected[idx]);
    }
}

// ── 35. filter_inner_type does not strip non-skip types ─────────────────────

proptest! {
    #[test]
    fn filter_does_not_strip_non_skip(idx in 0usize..=2) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<i32>),
            parse_quote!(Vec<String>),
            parse_quote!(Result<u32>),
        ];
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let filtered = filter_inner_type(&types[idx], &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            types[idx].to_token_stream().to_string(),
            "Non-skip types should pass through unchanged"
        );
    }
}
