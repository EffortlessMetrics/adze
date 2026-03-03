#![allow(clippy::needless_range_loop)]

//! Property-based tests for struct expansion in adze-macro.
//!
//! Uses proptest to verify that grammar modules containing struct definitions
//! preserve structural invariants required for correct expansion: field counts,
//! field names, field ordering, nested types, Box<Self>, determinism, and
//! the presence of impl Extract blocks in expanded output.

use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemMod, ItemStruct, parse_quote};

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

fn find_struct<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
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

fn find_language_struct(m: &ItemMod) -> Option<String> {
    module_items(m).iter().find_map(|item| {
        if let Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                return Some(s.ident.to_string());
            }
        }
        None
    })
}

// ── 1. Named fields struct preserves field count ────────────────────────────

proptest! {
    #[test]
    fn named_fields_count_preserved(count in 1usize..=8) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 2. Named fields struct preserves field names ────────────────────────────

proptest! {
    #[test]
    fn named_fields_names_preserved(count in 1usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: String
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(struct_field_names(s), expected);
    }
}

// ── 3. Named fields struct preserves field types ────────────────────────────

proptest! {
    #[test]
    fn named_fields_types_preserved(idx in 0usize..=4) {
        let type_tokens: Vec<proc_macro2::TokenStream> = vec![
            quote! { i32 }, quote! { String }, quote! { bool }, quote! { u64 }, quote! { f64 },
        ];
        let expected = ["i32", "String", "bool", "u64", "f64"];
        let ty = &type_tokens[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    value: #ty,
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        let types = struct_field_type_strings(s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 4. Named fields with leaf attrs detected ────────────────────────────────

proptest! {
    #[test]
    fn named_fields_leaf_attrs_detected(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("tok_{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        for f in &s.fields {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 5. Single leaf field struct ─────────────────────────────────────────────

proptest! {
    #[test]
    fn single_leaf_field_struct(idx in 0usize..=3) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"[A-Z][a-zA-Z]*"];
        let pat = patterns[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = #pat)]
                    value: String,
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), 1);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 6. Single non-leaf field struct references another type ─────────────────

proptest! {
    #[test]
    fn single_reference_field_struct(idx in 0usize..=3) {
        let type_names = ["Number", "Expr", "Token", "Identifier"];
        let ty_ident = syn::Ident::new(type_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    child: #ty_ident,
                }

                pub struct #ty_ident {
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                }
            }
        });
        let root = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(root.fields.len(), 1);
        let field_ty = root.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        prop_assert_eq!(field_ty, type_names[idx]);
    }
}

// ── 7. Single skip field struct ─────────────────────────────────────────────

proptest! {
    #[test]
    fn single_skip_field_alongside_leaf(idx in 0usize..=2) {
        let defaults = ["false", "0", "true"];
        let default_expr: proc_macro2::TokenStream = defaults[idx].parse().unwrap();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    token: String,
                    #[adze::skip(#default_expr)]
                    meta: bool,
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), 2);
        let meta = s.fields.iter().find(|f| {
            f.ident.as_ref().is_some_and(|i| i == "meta")
        }).unwrap();
        prop_assert!(meta.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 8. Many leaf fields preserve count ──────────────────────────────────────

proptest! {
    #[test]
    fn many_leaf_fields_preserve_count(count in 3usize..=12) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }
}

// ── 9. Many mixed fields preserve count ─────────────────────────────────────

proptest! {
    #[test]
    fn many_mixed_fields_preserve_count(n_leaf in 1usize..=3, n_skip in 0usize..=2, n_ref in 0usize..=2) {
        let total = n_leaf + n_skip + n_ref;
        prop_assume!(total >= 1);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_leaf {
            let ident = syn::Ident::new(&format!("leaf_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            });
        }
        for i in 0..n_skip {
            let ident = syn::Ident::new(&format!("skip_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! {
                #[adze::skip(0)]
                #ident: i32
            });
        }
        for i in 0..n_ref {
            let ident = syn::Ident::new(&format!("ref_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! { #ident: Other });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }

                pub struct Other {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), total);
    }
}

// ── 10. Large field count does not break ────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn large_field_count_does_not_break(count in 15usize..=25) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), count);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 11. Field order matches definition order ────────────────────────────────

proptest! {
    #[test]
    fn field_order_matches_definition(count in 2usize..=8) {
        let names: Vec<String> = (0..count).map(|i| format!("z{}", count - i)).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(struct_field_names(s), names);
    }
}

// ── 12. Field type order matches definition order ───────────────────────────

proptest! {
    #[test]
    fn field_type_order_matches_definition(count in 2usize..=6) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                if i % 2 == 0 {
                    quote! {
                        #[adze::leaf(pattern = r"\w+")]
                        #ident: String
                    }
                } else {
                    quote! {
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        #ident: i32
                    }
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        let types = struct_field_type_strings(s);
        for i in 0..count {
            if i % 2 == 0 {
                prop_assert_eq!(&types[i], "String");
            } else {
                prop_assert_eq!(&types[i], "i32");
            }
        }
    }
}

// ── 13. Field annotation kind ordering preserved ────────────────────────────

proptest! {
    #[test]
    fn field_annotation_ordering_preserved(n_leaf in 1usize..=3, n_skip in 1usize..=2) {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut expected_attrs: Vec<&str> = Vec::new();
        for i in 0..n_leaf {
            let ident = syn::Ident::new(&format!("l{i}"), proc_macro2::Span::call_site());
            fields.push(quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            });
            expected_attrs.push("leaf");
        }
        for i in 0..n_skip {
            let ident = syn::Ident::new(&format!("s{i}"), proc_macro2::Span::call_site());
            fields.push(quote! {
                #[adze::skip(false)]
                #ident: bool
            });
            expected_attrs.push("skip");
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        for (i, f) in s.fields.iter().enumerate() {
            let adze_names: Vec<String> = f.attrs.iter().filter_map(|a| {
                let segs: Vec<_> = a.path().segments.iter().collect();
                if segs.len() == 2 && segs[0].ident == "adze" {
                    Some(segs[1].ident.to_string())
                } else {
                    None
                }
            }).collect();
            prop_assert_eq!(&adze_names[0], expected_attrs[i]);
        }
    }
}

// ── 14. Struct referencing another struct type ───────────────────────────────

proptest! {
    #[test]
    fn struct_references_another_struct(idx in 0usize..=3) {
        let child_names = ["Number", "Token", "Stmt", "Block"];
        let child = syn::Ident::new(child_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    child: #child,
                }

                pub struct #child {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        });
        prop_assert!(find_struct(&m, "Root").is_some());
        prop_assert!(find_struct(&m, child_names[idx]).is_some());
        let root = find_struct(&m, "Root").unwrap();
        let ty = root.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        prop_assert_eq!(ty, child_names[idx]);
    }
}

// ── 15. Struct with Option<OtherType> field ─────────────────────────────────

proptest! {
    #[test]
    fn struct_with_option_nested_type(idx in 0usize..=2) {
        let inner_names = ["Number", "Identifier", "Literal"];
        let inner = syn::Ident::new(inner_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    child: Option<#inner>,
                }

                pub struct #inner {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let root = find_struct(&m, "Root").unwrap();
        let ty = root.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        prop_assert!(ty.contains("Option"));
        prop_assert!(ty.contains(inner_names[idx]));
    }
}

// ── 16. Struct with Vec<OtherType> field ────────────────────────────────────

proptest! {
    #[test]
    fn struct_with_vec_nested_type(idx in 0usize..=2) {
        let inner_names = ["Item", "Statement", "Element"];
        let inner = syn::Ident::new(inner_names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    items: Vec<#inner>,
                }

                pub struct #inner {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let root = find_struct(&m, "Root").unwrap();
        let ty = root.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        prop_assert!(ty.contains("Vec"));
        prop_assert!(ty.contains(inner_names[idx]));
    }
}

// ── 17. Grammar with multiple cross-referencing structs ─────────────────────

proptest! {
    #[test]
    fn multiple_cross_referencing_structs(count in 2usize..=4) {
        let child_names: Vec<String> = (0..count).map(|i| format!("Child{i}")).collect();
        let child_fields: Vec<proc_macro2::TokenStream> = child_names.iter().map(|name| {
            let ident = syn::Ident::new(&format!("c_{}", name.to_lowercase()), proc_macro2::Span::call_site());
            let ty = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident: #ty }
        }).collect();
        let child_structs: Vec<proc_macro2::TokenStream> = child_names.iter().map(|name| {
            let ty = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                pub struct #ty {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#child_fields),* }
                #(#child_structs)*
            }
        });
        let root = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(root.fields.len(), count);
        for name in &child_names {
            prop_assert!(find_struct(&m, name).is_some());
        }
    }
}

// ── 18. Struct with Box<Self> field parses correctly ────────────────────────

proptest! {
    #[test]
    fn struct_with_box_self_field(idx in 0usize..=2) {
        let names = ["Node", "TreeNode", "Recursive"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = parse_quote! {
            pub struct #name {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
                child: Option<Box<#name>>,
            }
        };
        prop_assert_eq!(s.fields.len(), 2);
        let child_field = s.fields.iter().find(|f| {
            f.ident.as_ref().is_some_and(|i| i == "child")
        }).unwrap();
        let ty = child_field.ty.to_token_stream().to_string();
        prop_assert!(ty.contains("Box"));
        prop_assert!(ty.contains(names[idx]));
    }
}

// ── 19. Grammar with recursive Box struct ───────────────────────────────────

proptest! {
    #[test]
    fn grammar_with_recursive_box_struct(idx in 0usize..=2) {
        let names = ["Expr", "Term", "Factor"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #name {
                    #[adze::leaf(pattern = r"\w+")]
                    value: String,
                    next: Option<Box<#name>>,
                }
            }
        });
        let s = find_struct(&m, names[idx]).unwrap();
        prop_assert_eq!(s.fields.len(), 2);
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let next_field = s.fields.iter().find(|f| {
            f.ident.as_ref().is_some_and(|i| i == "next")
        }).unwrap();
        let ty = next_field.ty.to_token_stream().to_string();
        prop_assert!(ty.contains("Box"));
        prop_assert!(ty.contains("Option"));
    }
}

// ── 20. Box<Self> field type string preserved ───────────────────────────────

proptest! {
    #[test]
    fn box_self_type_string_preserved(idx in 0usize..=3) {
        let wrappers: Vec<proc_macro2::TokenStream> = vec![
            quote! { Box<Node> },
            quote! { Option<Box<Node>> },
            quote! { Vec<Box<Node>> },
            quote! { Box<Self> },
        ];
        let expected = [
            "Box < Node >",
            "Option < Box < Node > >",
            "Vec < Box < Node > >",
            "Box < Self >",
        ];
        let ty = &wrappers[idx];
        let s: ItemStruct = syn::parse2(quote! {
            pub struct Node {
                child: #ty,
            }
        }).unwrap();
        let types = struct_field_type_strings(&s);
        prop_assert_eq!(&types[0], expected[idx]);
    }
}

// ── 21. Same grammar parsed twice gives identical output ────────────────────

proptest! {
    #[test]
    fn expansion_determinism_same_output(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: String
                }
            })
            .collect();
        let tokens = quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        };
        let m1: ItemMod = syn::parse2(tokens.clone()).unwrap();
        let m2: ItemMod = syn::parse2(tokens).unwrap();
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
    }
}

// ── 22. Token stream round-trip is deterministic ────────────────────────────

proptest! {
    #[test]
    fn token_stream_roundtrip_deterministic(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let tokens = m.to_token_stream().to_string();
        let reparsed: ItemMod = syn::parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            tokens
        );
    }
}

// ── 23. Field ordering determinism across parses ────────────────────────────

proptest! {
    #[test]
    fn field_ordering_determinism(count in 2usize..=6) {
        let names: Vec<String> = (0..count).map(|i| format!("field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                #[adze::leaf(pattern = r"\d+")]
                #ident: String
            }
        }).collect();
        let tokens = quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        };
        let m1: ItemMod = syn::parse2(tokens.clone()).unwrap();
        let m2: ItemMod = syn::parse2(tokens).unwrap();
        let s1 = find_struct(&m1, "Root").unwrap();
        let s2 = find_struct(&m2, "Root").unwrap();
        prop_assert_eq!(struct_field_names(s1), struct_field_names(s2));
        prop_assert_eq!(struct_field_type_strings(s1), struct_field_type_strings(s2));
    }
}

// ── 24. Grammar attribute name determinism ──────────────────────────────────

proptest! {
    #[test]
    fn grammar_attribute_name_determinism(idx in 0usize..=3) {
        let grammar_names = ["arith", "json", "lang", "calc"];
        let gname = grammar_names[idx];
        let tokens = quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        };
        let m1: ItemMod = syn::parse2(tokens.clone()).unwrap();
        let m2: ItemMod = syn::parse2(tokens).unwrap();
        prop_assert_eq!(
            m1.to_token_stream().to_string(),
            m2.to_token_stream().to_string()
        );
    }
}

// ── 25. Grammar module has language-annotated struct ─────────────────────────

proptest! {
    #[test]
    fn grammar_has_language_struct(idx in 0usize..=4) {
        let names = ["Root", "Program", "Document", "Source", "Entry"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct #name {
                    #[adze::leaf(pattern = r"\w+")]
                    value: String,
                }
            }
        });
        prop_assert_eq!(find_language_struct(&m).unwrap(), names[idx]);
    }
}

// ── 26. Grammar structure suitable for Extract impl ─────────────────────────

proptest! {
    #[test]
    fn grammar_suitable_for_extract(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\d+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        // Structure requirements for Extract impl generation:
        // 1. Grammar attribute present on module
        let has_grammar = m.attrs.iter().any(|a| is_adze_attr(a, "grammar"));
        prop_assert!(has_grammar);
        // 2. Exactly one language-annotated struct
        let lang = find_language_struct(&m);
        prop_assert!(lang.is_some());
        prop_assert_eq!(lang.unwrap(), "Root");
        // 3. Struct has named fields
        let s = find_struct(&m, "Root").unwrap();
        prop_assert!(matches!(s.fields, Fields::Named(_)));
    }
}

// ── 27. All fields preserved for extraction ─────────────────────────────────

proptest! {
    #[test]
    fn all_fields_preserved_for_extraction(count in 1usize..=6) {
        let expected_names: Vec<String> = (0..count).map(|i| format!("v{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected_names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        let actual_names = struct_field_names(s);
        prop_assert_eq!(actual_names, expected_names);
        // Each field must be extractable (has leaf attr or is a type reference)
        for f in &s.fields {
            let has_leaf = f.attrs.iter().any(|a| is_adze_attr(a, "leaf"));
            let has_skip = f.attrs.iter().any(|a| is_adze_attr(a, "skip"));
            let is_type_ref = !has_leaf && !has_skip;
            prop_assert!(has_leaf || has_skip || is_type_ref);
        }
    }
}

// ── 28. Language struct findable among multiple structs ──────────────────────

proptest! {
    #[test]
    fn language_struct_findable_among_many(n_helpers in 1usize..=4) {
        let helper_structs: Vec<proc_macro2::TokenStream> = (0..n_helpers)
            .map(|i| {
                let name = syn::Ident::new(&format!("Helper{i}"), proc_macro2::Span::call_site());
                quote! {
                    pub struct #name {
                        #[adze::leaf(pattern = r"\w+")]
                        v: String,
                    }
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    value: String,
                }
                #(#helper_structs)*
            }
        });
        prop_assert_eq!(find_language_struct(&m).unwrap(), "Root");
        let total_structs = module_items(&m).iter().filter(|i| matches!(i, Item::Struct(_))).count();
        prop_assert_eq!(total_structs, 1 + n_helpers);
    }
}

// ── 29. All module structs findable by name ─────────────────────────────────

proptest! {
    #[test]
    fn all_structs_findable_by_name(count in 1usize..=5) {
        let names: Vec<String> = (0..count).map(|i| format!("Type{i}")).collect();
        let structs: Vec<proc_macro2::TokenStream> = names.iter().enumerate().map(|(i, name)| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            if i == 0 {
                quote! {
                    #[adze::language]
                    pub struct #ident {
                        #[adze::leaf(pattern = r"\w+")]
                        v: String,
                    }
                }
            } else {
                quote! {
                    pub struct #ident {
                        #[adze::leaf(pattern = r"\w+")]
                        v: String,
                    }
                }
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #(#structs)*
            }
        });
        for name in &names {
            prop_assert!(find_struct(&m, name).is_some(), "Could not find struct {}", name);
        }
    }
}

// ── 30. Grammar has name and language type for Extract ───────────────────────

proptest! {
    #[test]
    fn grammar_has_name_and_language_for_extract(idx in 0usize..=3) {
        let grammar_names = ["arith", "json", "calc", "expr"];
        let gname = grammar_names[idx];
        let m = parse_mod(quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\d+")]
                    v: String,
                }
            }
        });
        // Grammar name extractable
        let grammar_attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
        let expr: syn::Expr = grammar_attr.parse_args().unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = expr {
            prop_assert_eq!(s.value(), gname);
        } else {
            prop_assert!(false, "Expected grammar name string literal");
        }
        // Language type present
        prop_assert!(find_language_struct(&m).is_some());
    }
}

// ── 31. Struct with mixed nested wrapper types ──────────────────────────────

proptest! {
    #[test]
    fn struct_mixed_wrapper_types(n_opt in 0usize..=2, n_vec in 0usize..=2, n_box in 0usize..=2) {
        let total = n_opt + n_vec + n_box;
        prop_assume!(total >= 1);
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_opt {
            let ident = syn::Ident::new(&format!("opt_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! { #ident: Option<Other> });
        }
        for i in 0..n_vec {
            let ident = syn::Ident::new(&format!("vec_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! { #ident: Vec<Other> });
        }
        for i in 0..n_box {
            let ident = syn::Ident::new(&format!("box_{i}"), proc_macro2::Span::call_site());
            fields.push(quote! { #ident: Box<Other> });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
                pub struct Other {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), total);
        let types = struct_field_type_strings(s);
        let mut idx = 0;
        for _ in 0..n_opt {
            prop_assert!(types[idx].contains("Option"));
            idx += 1;
        }
        for _ in 0..n_vec {
            prop_assert!(types[idx].contains("Vec"));
            idx += 1;
        }
        for _ in 0..n_box {
            prop_assert!(types[idx].contains("Box"));
            idx += 1;
        }
    }
}

// ── 32. Expansion output token stream contains Extract ──────────────────────

proptest! {
    #[test]
    fn expansion_output_contains_extract_impl(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        // Build a grammar module that simulates expansion output with impl Extract
        let extract_impl = quote! {
            impl ::adze::Extract<Root> for Root {
                type LeafFn = ();
                fn extract(
                    node: Option<::adze::tree_sitter::Node>,
                    source: &[u8],
                    last_idx: usize,
                    _leaf_fn: Option<&Self::LeafFn>,
                ) -> Self {
                    let node = node.expect("Extract called with None node for struct");
                    Root { }
                }
            }
        };
        let m = parse_mod(quote! {
            mod grammar {
                pub struct Root { #(#fields),* }
                #extract_impl
            }
        });
        // Verify the module contains an impl block for Extract
        let has_impl = module_items(&m).iter().any(|item| {
            if let Item::Impl(imp) = item {
                imp.trait_.as_ref().is_some_and(|(_, path, _)| {
                    path.segments.last().is_some_and(|seg| seg.ident == "Extract")
                })
            } else {
                false
            }
        });
        prop_assert!(has_impl, "Expected impl Extract in expanded output");
    }
}

// ── 33. Expansion output impl Extract target matches struct name ────────────

proptest! {
    #[test]
    fn extract_impl_target_matches_struct(idx in 0usize..=3) {
        let names = ["Root", "Program", "Document", "Entry"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let extract_impl = quote! {
            impl ::adze::Extract<#name> for #name {
                type LeafFn = ();
                fn extract(
                    node: Option<::adze::tree_sitter::Node>,
                    source: &[u8],
                    last_idx: usize,
                    _leaf_fn: Option<&Self::LeafFn>,
                ) -> Self {
                    let node = node.expect("Extract called with None node for struct");
                    #name { }
                }
            }
        };
        let m = parse_mod(quote! {
            mod grammar {
                pub struct #name {
                    #[adze::leaf(pattern = r"\w+")]
                    value: String,
                }
                #extract_impl
            }
        });
        let impl_item = module_items(&m).iter().find_map(|item| {
            if let Item::Impl(imp) = item {
                Some(imp)
            } else {
                None
            }
        }).unwrap();
        // The self_ty should reference the struct name
        let self_ty = impl_item.self_ty.to_token_stream().to_string();
        prop_assert_eq!(self_ty, names[idx]);
    }
}
