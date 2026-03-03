#![allow(clippy::needless_range_loop)]

//! Property-based tests for field extraction in adze-macro.
//!
//! Uses proptest to verify that field extraction from struct and enum definitions
//! preserves field names, types, ordering, wrapper types (Option, Vec, Box),
//! and annotation semantics — the properties that `gen_field` and
//! `gen_struct_or_variant` in `expansion.rs` depend on.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, parse_quote};

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

fn struct_field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect()
}

fn struct_field_type_strings(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

/// Extract the ident_str for each field (named → ident, unnamed → index).
fn struct_field_ident_strs(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            f.ident
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or(format!("{i}"))
        })
        .collect()
}

fn variant_field_names(v: &syn::Variant) -> Vec<String> {
    match &v.fields {
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect(),
        _ => vec![],
    }
}

fn variant_field_type_strings(v: &syn::Variant) -> Vec<String> {
    v.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn variant_field_ident_strs(v: &syn::Variant) -> Vec<String> {
    match &v.fields {
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect(),
        Fields::Unnamed(u) => (0..u.unnamed.len()).map(|i| format!("{i}")).collect(),
        Fields::Unit => vec![],
    }
}

/// Find the first struct in a parsed grammar module.
fn find_struct_in_mod(m: &ItemMod) -> Option<&ItemStruct> {
    m.content.as_ref()?.1.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            Some(s)
        } else {
            None
        }
    })
}

/// Find the first enum in a parsed grammar module.
#[allow(dead_code)]
fn find_enum_in_mod(m: &ItemMod) -> Option<&ItemEnum> {
    m.content.as_ref()?.1.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item {
            Some(e)
        } else {
            None
        }
    })
}

// ── 1. Extract fields from struct with named fields ─────────────────────────

proptest! {
    #[test]
    fn extract_fields_from_struct_named(count in 1usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: String
            }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let actual = struct_field_names(s);
        prop_assert_eq!(actual, expected);
    }
}

// ── 2. Field type preserved as String ───────────────────────────────────────

proptest! {
    #[test]
    fn field_type_preserved_string(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("tok_{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let types = struct_field_type_strings(s);
        for t in &types {
            prop_assert_eq!(t, "String");
        }
    }
}

// ── 3. Field type preserved for primitives ──────────────────────────────────

proptest! {
    #[test]
    fn field_type_preserved_primitives(idx in 0usize..=3) {
        let type_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { i32 },
            quote::quote! { u64 },
            quote::quote! { f64 },
            quote::quote! { bool },
        ];
        let expected = ["i32", "u64", "f64", "bool"];
        let ty = &type_tokens[idx];
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S {
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse().unwrap())]
                    value: #ty,
                }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let types = struct_field_type_strings(s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 4. Field name preserved across struct ───────────────────────────────────

proptest! {
    #[test]
    fn field_name_preserved_across_struct(idx in 0usize..=5) {
        let names = ["value", "data", "inner", "content", "token", "result"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: String,
                }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let field_names = struct_field_names(s);
        prop_assert_eq!(field_names.len(), 1);
        prop_assert_eq!(&field_names[0], name);
    }
}

// ── 5. Optional field (Option<T>) type preserved ────────────────────────────

proptest! {
    #[test]
    fn optional_field_type_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("opt_{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                #ident: Option<i32>
            }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let types = struct_field_type_strings(s);
        for t in &types {
            prop_assert_eq!(t, "Option < i32 >");
        }
    }
}

// ── 6. Vec field (repetition) type preserved ────────────────────────────────

proptest! {
    #[test]
    fn vec_field_type_preserved(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("items_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: Vec<Number> }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    v: i32,
                }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let types = struct_field_type_strings(s);
        for t in &types {
            prop_assert_eq!(t, "Vec < Number >");
        }
    }
}

// ── 7. Box field type preserved ─────────────────────────────────────────────

proptest! {
    #[test]
    fn box_field_type_preserved(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("child_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: Box<Expr> }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
                Neg { #[adze::leaf(text = "-")] _op: (), #(#fields),* }
            }
        }).unwrap();
        let neg = e.variants.iter().find(|v| v.ident == "Neg").unwrap();
        let types = variant_field_type_strings(neg);
        // First type is _op: (), rest are Box<Expr>
        for i in 1..types.len() {
            prop_assert_eq!(&types[i], "Box < Expr >");
        }
    }
}

// ── 8. Multiple fields count preserved ──────────────────────────────────────

proptest! {
    #[test]
    fn multiple_fields_count_preserved(count in 1usize..=8) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: String
            }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 9. Field ordering preserved ─────────────────────────────────────────────

proptest! {
    #[test]
    fn field_ordering_preserved(count in 2usize..=7) {
        let names: Vec<String> = (0..count).map(|i| format!("z{}", count - i)).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            }
        }).collect();
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S { #(#fields),* }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let actual = struct_field_names(s);
        prop_assert_eq!(actual, names, "Field ordering must match definition order");
    }
}

// ── 10. Named fields in enum variant preserved ──────────────────────────────

proptest! {
    #[test]
    fn named_fields_in_enum_variant(n_fields in 1usize..=4) {
        let expected: Vec<String> = (0..n_fields).map(|i| format!("val_{i}")).collect();
        let field_tokens: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                V { #(#field_tokens),* }
            }
        }).unwrap();
        let actual = variant_field_names(&e.variants[0]);
        prop_assert_eq!(actual, expected);
    }
}

// ── 11. Unnamed fields get positional ident strings ─────────────────────────

proptest! {
    #[test]
    fn unnamed_fields_get_positional_ident_strs(count in 1usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { i32 })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S(#(#fields),*);
        }).unwrap();
        let ident_strs = struct_field_ident_strs(&s);
        for i in 0..count {
            prop_assert_eq!(&ident_strs[i], &format!("{i}"));
        }
    }
}

// ── 12. Optional field with non-leaf child ──────────────────────────────────

proptest! {
    #[test]
    fn optional_field_non_leaf_child(idx in 0usize..=2) {
        let field_names_arr = ["maybe_a", "maybe_b", "maybe_c"];
        let fname = field_names_arr[idx];
        let ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S {
                    #ident: Option<Number>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    v: i32,
                }
            }
        };
        let s = find_struct_in_mod(&m).unwrap();
        let types = struct_field_type_strings(s);
        prop_assert_eq!(&types[0], "Option < Number >");
        let names = struct_field_names(s);
        prop_assert_eq!(&names[0], fname);
    }
}

// ── 13. Vec field with repeat annotation detected ───────────────────────────

proptest! {
    #[test]
    fn vec_repeat_annotation_detected(non_empty in proptest::bool::ANY) {
        let s: ItemStruct = if non_empty {
            syn::parse2(quote::quote! {
                pub struct S {
                    #[adze::repeat(non_empty = true)]
                    numbers: Vec<i32>,
                }
            }).unwrap()
        } else {
            syn::parse2(quote::quote! {
                pub struct S {
                    numbers: Vec<i32>,
                }
            }).unwrap()
        };
        let f = s.fields.iter().next().unwrap();
        if non_empty {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        } else {
            prop_assert!(adze_attr_names(&f.attrs).is_empty());
        }
        let ty_str = f.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "Vec < i32 >");
    }
}

// ── 14. Skip field preserves name and type ──────────────────────────────────

proptest! {
    #[test]
    fn skip_field_preserves_name_and_type(idx in 0usize..=2) {
        let field_names_arr = ["visited", "checked", "processed"];
        let fname = field_names_arr[idx];
        let ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
                #[adze::skip(false)]
                #ident: bool,
            }
        }).unwrap();
        let names = struct_field_names(&s);
        prop_assert!(names.contains(&fname.to_string()));
        let skip_field = s.fields.iter().find(|f| {
            f.ident.as_ref().map(|id| id.to_string()) == Some(fname.to_string())
        }).unwrap();
        let ty_str = skip_field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "bool");
        prop_assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 15. Mixed Option/Vec/plain fields preserve types ────────────────────────

proptest! {
    #[test]
    fn mixed_wrapper_fields_types(n_opt in 0usize..=2, n_vec in 0usize..=2, n_plain in 1usize..=2) {
        let mut field_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut expected_types: Vec<String> = Vec::new();
        for i in 0..n_opt {
            let ident = syn::Ident::new(&format!("opt_{i}"), proc_macro2::Span::call_site());
            field_tokens.push(quote::quote! { #ident: Option<i32> });
            expected_types.push("Option < i32 >".to_string());
        }
        for i in 0..n_vec {
            let ident = syn::Ident::new(&format!("v_{i}"), proc_macro2::Span::call_site());
            field_tokens.push(quote::quote! { #ident: Vec<String> });
            expected_types.push("Vec < String >".to_string());
        }
        for i in 0..n_plain {
            let ident = syn::Ident::new(&format!("p_{i}"), proc_macro2::Span::call_site());
            field_tokens.push(quote::quote! { #ident: String });
            expected_types.push("String".to_string());
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#field_tokens),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(types, expected_types);
    }
}

// ── 16. Enum unit leaf variant has no fields ────────────────────────────────

proptest! {
    #[test]
    fn enum_unit_leaf_variant_no_fields(count in 1usize..=5) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let name = syn::Ident::new(&format!("K{i}"), proc_macro2::Span::call_site());
            let text = format!("kw{i}");
            quote::quote! {
                #[adze::leaf(text = #text)]
                #name
            }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        for v in &e.variants {
            prop_assert!(matches!(v.fields, Fields::Unit));
        }
    }
}

// ── 17. Box fields in unnamed enum variant ──────────────────────────────────

proptest! {
    #[test]
    fn box_unnamed_variant_fields(count in 1usize..=4) {
        let box_fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { Box<Expr> })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
                Recurse(#(#box_fields),*)
            }
        }).unwrap();
        let recurse = e.variants.iter().find(|v| v.ident == "Recurse").unwrap();
        prop_assert_eq!(recurse.fields.len(), count);
        let types = variant_field_type_strings(recurse);
        for t in &types {
            prop_assert_eq!(t, "Box < Expr >");
        }
    }
}

// ── 18. Delimited annotation preserves field name and type ──────────────────

proptest! {
    #[test]
    fn delimited_field_name_and_type_preserved(idx in 0usize..=3) {
        let names = ["items", "elements", "values", "entries"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                #ident: Vec<i32>,
            }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual.len(), 1);
        prop_assert_eq!(&actual[0], name);
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(&types[0], "Vec < i32 >");
    }
}

// ── 19. Enum mixed named and unnamed variant field counts ───────────────────

proptest! {
    #[test]
    fn enum_mixed_variant_field_counts(n_named in 1usize..=3) {
        let named_fields: Vec<proc_macro2::TokenStream> = (0..n_named).map(|i| {
            let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: String }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Leaf(#[adze::leaf(pattern = r"\d+")] String),
                Complex { #(#named_fields),* }
            }
        }).unwrap();
        let leaf = e.variants.iter().find(|v| v.ident == "Leaf").unwrap();
        let complex = e.variants.iter().find(|v| v.ident == "Complex").unwrap();
        prop_assert_eq!(leaf.fields.len(), 1);
        prop_assert_eq!(complex.fields.len(), n_named);
    }
}

// ── 20. Enum variant ordering preserved ─────────────────────────────────────

proptest! {
    #[test]
    fn enum_variant_ordering_preserved(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
            let text = format!("v{i}");
            quote::quote! {
                #[adze::leaf(text = #text)]
                #name
            }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        let actual: Vec<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        let expected: Vec<String> = (0..count).map(|i| format!("V{i}")).collect();
        prop_assert_eq!(actual, expected);
    }
}

// ── 21. Named vs unnamed field ident_strs ───────────────────────────────────

proptest! {
    #[test]
    fn named_vs_unnamed_ident_strs(n_named in 1usize..=3, n_unnamed in 1usize..=3) {
        let named_fields: Vec<proc_macro2::TokenStream> = (0..n_named).map(|i| {
            let ident = syn::Ident::new(&format!("x_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let unnamed_fields: Vec<proc_macro2::TokenStream> = (0..n_unnamed)
            .map(|_| quote::quote! { String })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                Named { #(#named_fields),* },
                Unnamed(#(#unnamed_fields),*)
            }
        }).unwrap();
        let named_strs = variant_field_ident_strs(&e.variants[0]);
        let unnamed_strs = variant_field_ident_strs(&e.variants[1]);
        for i in 0..n_named {
            prop_assert_eq!(&named_strs[i], &format!("x_{i}"));
        }
        for i in 0..n_unnamed {
            prop_assert_eq!(&unnamed_strs[i], &format!("{i}"));
        }
    }
}

// ── 22. Leaf annotation on field detected for extraction ────────────────────

proptest! {
    #[test]
    fn leaf_annotation_on_field_detected(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("tok_{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: String
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for f in &s.fields {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 23. Leaf with transform preserves param names ───────────────────────────

proptest! {
    #[test]
    fn leaf_transform_params_preserved(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("num_{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                #ident: i32
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for f in &s.fields {
            let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
                attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();
            let param_names: Vec<String> = params.iter().map(|p| p.path.to_string()).collect();
            prop_assert!(param_names.contains(&"pattern".to_string()));
            prop_assert!(param_names.contains(&"transform".to_string()));
        }
    }
}

// ── 24. Field annotation stays on field, not struct ─────────────────────────

proptest! {
    #[test]
    fn field_attrs_do_not_leak_to_struct(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: i32
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(adze_attr_names(&s.attrs).is_empty(),
            "Field-level attrs should not appear on struct");
        for f in &s.fields {
            prop_assert!(!adze_attr_names(&f.attrs).is_empty());
        }
    }
}

// ── 25. Mixed leaf, skip, and plain fields preserve names ───────────────────

proptest! {
    #[test]
    fn mixed_annotations_preserve_names(n_leaf in 1usize..=2, n_skip in 0usize..=2, n_plain in 1usize..=2) {
        let mut all_names: Vec<String> = Vec::new();
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_leaf {
            let name = format!("leaf_{i}");
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            tokens.push(quote::quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            });
            all_names.push(name);
        }
        for i in 0..n_skip {
            let name = format!("skip_{i}");
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            tokens.push(quote::quote! {
                #[adze::skip(0)]
                #ident: i32
            });
            all_names.push(name);
        }
        for i in 0..n_plain {
            let name = format!("plain_{i}");
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            tokens.push(quote::quote! { #ident: String });
            all_names.push(name);
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#tokens),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, all_names);
    }
}

// ── 26. Vec<Option<T>> nested wrapper type preserved ────────────────────────

proptest! {
    #[test]
    fn nested_wrapper_type_preserved(idx in 0usize..=2) {
        let type_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { Vec<Option<i32>> },
            quote::quote! { Option<Vec<String>> },
            quote::quote! { Option<Box<Expr>> },
        ];
        let expected = [
            "Vec < Option < i32 > >",
            "Option < Vec < String > >",
            "Option < Box < Expr > >",
        ];
        let ty = &type_tokens[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { value: #ty }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 27. Struct with language + extra in module ──────────────────────────────

proptest! {
    #[test]
    fn language_and_extra_in_module(idx in 0usize..=2) {
        let patterns = [r"\s", r"\t", r"//[^\n]*"];
        let pat = patterns[idx];
        let m: ItemMod = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct S {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }

                #[adze::extra]
                struct Ws {
                    #[adze::leaf(pattern = #pat)]
                    _ws: (),
                }
            }
        };
        let items = &m.content.as_ref().unwrap().1;
        let struct_count = items.iter().filter(|i| matches!(i, syn::Item::Struct(_))).count();
        prop_assert!(struct_count >= 2, "Module should contain at least 2 structs");
    }
}

// ── 28. Leaf text attribute value round-trips ───────────────────────────────

proptest! {
    #[test]
    fn leaf_text_value_roundtrips(idx in 0usize..=5) {
        let texts = ["+", "-", "==", "!=", "::", "->"];
        let text = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #text)]
                op: (),
            }
        }).unwrap();
        let attr = s.fields.iter().next().unwrap().attrs.iter()
            .find(|a| is_adze_attr(a, "leaf")).unwrap();
        let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
            attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();
        prop_assert_eq!(params[0].path.to_string(), "text");
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &params[0].expr {
            prop_assert_eq!(s.value(), text);
        } else {
            prop_assert!(false, "Expected string literal");
        }
    }
}

// ── 29. Leaf on enum field is on field, not variant ─────────────────────────

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

// ── 30. Field names unique within struct ────────────────────────────────────

proptest! {
    #[test]
    fn field_names_unique_within_struct(count in 2usize..=8) {
        let names: Vec<String> = (0..count).map(|i| format!("unique_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        let mut deduped = actual.clone();
        deduped.sort();
        deduped.dedup();
        prop_assert_eq!(actual.len(), deduped.len(), "Field names must be unique");
    }
}

// ── 31. Fields with leading underscore preserved ────────────────────────────

proptest! {
    #[test]
    fn fields_with_leading_underscore(count in 1usize..=4) {
        let names: Vec<String> = (0..count).map(|i| format!("_hidden_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: () }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names);
    }
}

// ── 32. Mixed wrapper types Vec+Option+Box preserve names ───────────────────

proptest! {
    #[test]
    fn mixed_wrapper_types_preserve_names(n_vec in 0usize..=2, n_opt in 0usize..=2, n_box in 0usize..=2) {
        prop_assume!(n_vec + n_opt + n_box >= 1);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut expected_names: Vec<String> = Vec::new();
        for i in 0..n_vec {
            let ident = syn::Ident::new(&format!("v{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Vec<i32> });
            expected_names.push(format!("v{i}"));
        }
        for i in 0..n_opt {
            let ident = syn::Ident::new(&format!("o{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Option<String> });
            expected_names.push(format!("o{i}"));
        }
        for i in 0..n_box {
            let ident = syn::Ident::new(&format!("b{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Box<Node> });
            expected_names.push(format!("b{i}"));
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, expected_names);
    }
}
