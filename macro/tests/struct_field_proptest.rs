#![allow(clippy::needless_range_loop)]

//! Property-based tests for struct field handling in adze-macro.
//!
//! Uses proptest to generate randomized struct definitions and verify that
//! syn correctly parses and preserves field counts, names, types, annotations,
//! ordering, visibility, and other structural properties specific to structs.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemStruct, Visibility};

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

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

// ── 1. Struct with one named field ──────────────────────────────────────────

proptest! {
    #[test]
    fn struct_one_named_field(idx in 0usize..=4) {
        let names = ["value", "data", "inner", "content", "token"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #ident: i32 }
        }).unwrap();
        let field_names = struct_field_names(&s);
        prop_assert_eq!(field_names.len(), 1);
        prop_assert_eq!(&field_names[0], name);
    }
}

// ── 2. Struct with many named fields ────────────────────────────────────────

proptest! {
    #[test]
    fn struct_many_named_fields(count in 2usize..=10) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 3. Named field extraction preserves all names ───────────────────────────

proptest! {
    #[test]
    fn named_fields_extraction(count in 1usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: String }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, expected);
    }
}

// ── 4. Field type recognition for primitive types ───────────────────────────

proptest! {
    #[test]
    fn field_type_recognition_primitives(idx in 0usize..=5) {
        let types_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { i32 },
            quote::quote! { u64 },
            quote::quote! { bool },
            quote::quote! { String },
            quote::quote! { f64 },
            quote::quote! { u8 },
        ];
        let expected = ["i32", "u64", "bool", "String", "f64", "u8"];
        let ty = &types_tokens[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { value: #ty }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 5. Field type recognition for generic wrappers ──────────────────────────

proptest! {
    #[test]
    fn field_type_recognition_generic_wrappers(idx in 0usize..=3) {
        let type_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote::quote! { Vec<i32> },
            quote::quote! { Option<String> },
            quote::quote! { Box<Expr> },
            quote::quote! { Vec<Option<i32>> },
        ];
        let expected = ["Vec < i32 >", "Option < String >", "Box < Expr >", "Vec < Option < i32 > >"];
        let ty = &type_tokens[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { value: #ty }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 6. Field with leaf annotation detected ──────────────────────────────────

proptest! {
    #[test]
    fn field_with_leaf_annotation(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("tok_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for f in &s.fields {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 7. Field with skip annotation detected ──────────────────────────────────

proptest! {
    #[test]
    fn field_with_skip_annotation(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("meta_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::skip(false)]
                    #ident: bool
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for f in &s.fields {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
        }
    }
}

// ── 8. Field with prec annotation on struct-level ───────────────────────────

proptest! {
    #[test]
    fn struct_with_prec_annotation(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::prec(#lit)]
            pub struct S {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "prec")));
        let attr = s.attrs.iter().find(|a| is_adze_attr(a, "prec")).unwrap();
        let expr: syn::Expr = attr.parse_args().unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) = expr {
            prop_assert_eq!(i.base10_parse::<i32>().unwrap(), prec);
        } else {
            prop_assert!(false, "Expected int literal");
        }
    }
}

// ── 9. Field ordering preservation ──────────────────────────────────────────

proptest! {
    #[test]
    fn field_ordering_preserved(count in 2usize..=8) {
        let names: Vec<String> = (0..count)
            .map(|i| format!("z{}", count - i))
            .collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names, "Field ordering must match definition order");
    }
}

// ── 10. Struct visibility: pub ──────────────────────────────────────────────

proptest! {
    #[test]
    fn struct_pub_visibility(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(is_pub(&s.vis));
    }
}

// ── 11. Struct visibility: inherited (no vis keyword) ───────────────────────

proptest! {
    #[test]
    fn struct_inherited_visibility(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(matches!(s.vis, Visibility::Inherited));
    }
}

// ── 12. Struct visibility: pub(crate) ───────────────────────────────────────

proptest! {
    #[test]
    fn struct_pub_crate_visibility(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: i32 }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub(crate) struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(matches!(s.vis, Visibility::Restricted(_)));
    }
}

// ── 13. Mixed field annotations preserve names ──────────────────────────────

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

// ── 14. Leaf with transform annotation round-trips ──────────────────────────

proptest! {
    #[test]
    fn leaf_with_transform_roundtrips(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("num_{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    #ident: i32
                }
            })
            .collect();
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

// ── 15. Leaf with text annotation round-trips ───────────────────────────────

proptest! {
    #[test]
    fn leaf_with_text_roundtrips(idx in 0usize..=4) {
        let texts = ["+", "-", "==", "!=", "::"];
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

// ── 16. Struct with Vec field type preserved ────────────────────────────────

proptest! {
    #[test]
    fn struct_vec_field_type_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("items_{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: Vec<i32> }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        for t in &types {
            prop_assert_eq!(t, "Vec < i32 >");
        }
    }
}

// ── 17. Struct with Option field type preserved ─────────────────────────────

proptest! {
    #[test]
    fn struct_option_field_type_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("opt_{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: Option<String> }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        for t in &types {
            prop_assert_eq!(t, "Option < String >");
        }
    }
}

// ── 18. Struct with Box field type preserved ────────────────────────────────

proptest! {
    #[test]
    fn struct_box_field_type_preserved(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("child_{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: Box<Node> }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        for t in &types {
            prop_assert_eq!(t, "Box < Node >");
        }
    }
}

// ── 19. Struct name preserved ───────────────────────────────────────────────

proptest! {
    #[test]
    fn struct_name_preserved(idx in 0usize..=4) {
        let names = ["Program", "NumberList", "Identifier", "Statement", "Node"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct #name { value: i32 }
        }).unwrap();
        prop_assert_eq!(s.ident.to_string(), names[idx]);
    }
}

// ── 20. Unit struct has no fields ───────────────────────────────────────────

proptest! {
    #[test]
    fn unit_struct_has_no_fields(idx in 0usize..=3) {
        let names = ["Token", "Marker", "Sentinel", "EndOfFile"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct #name;
        }).unwrap();
        prop_assert!(matches!(s.fields, Fields::Unit));
        prop_assert_eq!(s.fields.len(), 0);
    }
}

// ── 21. Tuple struct field count matches ────────────────────────────────────

proptest! {
    #[test]
    fn tuple_struct_field_count(count in 1usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { i32 })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S(#(#fields),*);
        }).unwrap();
        prop_assert!(matches!(s.fields, Fields::Unnamed(_)));
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 22. Struct with language attribute ───────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_language_attribute(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: String }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 23. Struct with extra attribute ─────────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_extra_attribute(idx in 0usize..=2) {
        let patterns = [r"\s", r"\t", r"//[^\n]*"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        prop_assert_eq!(s.fields.len(), 1);
    }
}

// ── 24. Struct with word attribute ──────────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_word_attribute(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"[a-z]+", r"\w+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    }
}

// ── 25. Struct with external attribute ──────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_external_attribute(idx in 0usize..=2) {
        let names = ["IndentToken", "DedentToken", "NewlineToken"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::external]
            struct #name;
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
        prop_assert!(matches!(s.fields, Fields::Unit));
    }
}

// ── 26. Struct with delimited field ─────────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_delimited_field(idx in 0usize..=3) {
        let delimiters = [",", ";", "|", ":"];
        let delim = delimiters[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #delim)]
                    ()
                )]
                items: Vec<i32>,
            }
        }).unwrap();
        let f = s.fields.iter().next().unwrap();
        prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), "items");
    }
}

// ── 27. Struct with repeat annotation ───────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_repeat_annotation(non_empty in proptest::bool::ANY) {
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
    }
}

// ── 28. Field type alternation across fields ────────────────────────────────

proptest! {
    #[test]
    fn field_type_alternation(count in 2usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                if i % 2 == 0 {
                    quote::quote! { #ident: String }
                } else {
                    quote::quote! { #ident: i32 }
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        for i in 0..count {
            if i % 2 == 0 {
                prop_assert_eq!(&types[i], "String");
            } else {
                prop_assert_eq!(&types[i], "i32");
            }
        }
    }
}

// ── 29. Field names unique within struct ────────────────────────────────────

proptest! {
    #[test]
    fn field_names_unique(count in 2usize..=8) {
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
        prop_assert_eq!(actual.len(), deduped.len());
    }
}

// ── 30. Struct with multiple adze attributes ────────────────────────────────

proptest! {
    #[test]
    fn struct_with_multiple_attrs(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            #[adze::prec(#lit)]
            pub struct S {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }).unwrap();
        let attr_names = adze_attr_names(&s.attrs);
        prop_assert!(attr_names.contains(&"language".to_string()));
        prop_assert!(attr_names.contains(&"prec".to_string()));
        prop_assert_eq!(attr_names.len(), 2);
    }
}

// ── 31. Struct fields with leading underscore names ─────────────────────────

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

// ── 32. Struct field annotations do not leak to struct level ─────────────────

proptest! {
    #[test]
    fn field_attrs_do_not_leak_to_struct(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: i32
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        prop_assert!(adze_attr_names(&s.attrs).is_empty());
        for f in &s.fields {
            prop_assert!(!adze_attr_names(&f.attrs).is_empty());
        }
    }
}

// ── 33. Struct with mixed wrapper types ─────────────────────────────────────

proptest! {
    #[test]
    fn struct_mixed_wrapper_types(n_vec in 0usize..=2, n_opt in 0usize..=2, n_box in 0usize..=2) {
        prop_assume!(n_vec + n_opt + n_box >= 1);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut expected_types: Vec<String> = Vec::new();
        for i in 0..n_vec {
            let ident = syn::Ident::new(&format!("v{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Vec<i32> });
            expected_types.push("Vec < i32 >".to_string());
        }
        for i in 0..n_opt {
            let ident = syn::Ident::new(&format!("o{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Option<String> });
            expected_types.push("Option < String >".to_string());
        }
        for i in 0..n_box {
            let ident = syn::Ident::new(&format!("b{i}"), proc_macro2::Span::call_site());
            fields.push(quote::quote! { #ident: Box<Node> });
            expected_types.push("Box < Node >".to_string());
        }
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(types, expected_types);
    }
}
