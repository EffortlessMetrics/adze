#![allow(clippy::needless_range_loop)]

//! Property-based tests for field name handling in adze-macro.
//!
//! Uses proptest to generate randomized struct and enum structures and verify
//! that field names from Rust types are correctly extracted, preserved, and
//! mapped to grammar field name strings — the key mapping that `gen_field`
//! relies on in `expansion.rs`.

use proptest::prelude::*;
use quote::ToTokens;
use syn::{Attribute, Fields, ItemEnum, ItemStruct, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn struct_field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect()
}

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

// ── 1. Field name extraction from struct ────────────────────────────────────

proptest! {
    #[test]
    fn field_name_extraction_from_struct(count in 1usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, expected);
    }
}

// ── 2. Field name in grammar rule string ────────────────────────────────────

proptest! {
    #[test]
    fn field_name_maps_to_ident_str(count in 1usize..=5) {
        let names: Vec<String> = (0..count).map(|i| format!("my_field_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: String }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let ident_strs = struct_field_ident_strs(&s);
        for i in 0..count {
            prop_assert_eq!(&ident_strs[i], &names[i]);
        }
    }
}

// ── 3. Multiple field names preserved ───────────────────────────────────────

proptest! {
    #[test]
    fn multiple_field_names_all_preserved(n in 2usize..=8) {
        let names: Vec<String> = (0..n).map(|i| format!("f{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: u32 }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual.len(), n);
        prop_assert_eq!(actual, names);
    }
}

// ── 4. Field name with underscores ──────────────────────────────────────────

proptest! {
    #[test]
    fn field_name_with_underscores(segments in 1usize..=4) {
        let name = (0..segments).map(|i| format!("part{i}")).collect::<Vec<_>>().join("_");
        let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #ident: i32 }
        }).unwrap();
        let names = struct_field_names(&s);
        prop_assert_eq!(names.len(), 1);
        prop_assert_eq!(&names[0], &name);
    }
}

// ── 5. Field name ordering preserved ────────────────────────────────────────

proptest! {
    #[test]
    fn field_name_ordering_preserved(count in 2usize..=7) {
        let names: Vec<String> = (0..count).map(|i| format!("z{}", count - i)).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: bool }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names, "Field ordering must match definition order");
    }
}

// ── 6. Field names in enum variants ─────────────────────────────────────────

proptest! {
    #[test]
    fn field_names_in_enum_named_variant(field_count in 1usize..=5) {
        let expected: Vec<String> = (0..field_count).map(|i| format!("val_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V { #(#fields),* } }
        }).unwrap();
        let actual = variant_field_names(&e.variants[0]);
        prop_assert_eq!(actual, expected);
    }
}

// ── 7. Duplicate field names across enum variants ───────────────────────────

proptest! {
    #[test]
    fn duplicate_field_names_across_variants(n_variants in 2usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_variants).map(|i| {
            let vname = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
            quote::quote! { #vname { value: i32, name: String } }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        for i in 0..n_variants {
            let names = variant_field_names(&e.variants[i]);
            prop_assert_eq!(names, vec!["value".to_string(), "name".to_string()]);
        }
    }
}

// ── 8. Field name case conversion (snake_case preserved) ────────────────────

proptest! {
    #[test]
    fn field_name_snake_case_preserved(idx in 0usize..=5) {
        let cases = [
            "simple",
            "two_words",
            "three_word_name",
            "a_b_c_d",
            "with_123_numbers",
            "trailing_",
        ];
        let name = cases[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #ident: i32 }
        }).unwrap();
        let actual = struct_field_ident_strs(&s);
        prop_assert_eq!(&actual[0], name);
    }
}

// ── 9. Unnamed fields get index-based ident_str ─────────────────────────────

proptest! {
    #[test]
    fn unnamed_fields_get_index_ident_str(count in 1usize..=6) {
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

// ── 10. Unnamed enum variant fields get index ident_strs ────────────────────

proptest! {
    #[test]
    fn unnamed_variant_fields_get_index_ident_strs(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|_| quote::quote! { String })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V(#(#fields),*) }
        }).unwrap();
        let ident_strs = variant_field_ident_strs(&e.variants[0]);
        for i in 0..count {
            prop_assert_eq!(&ident_strs[i], &format!("{i}"));
        }
    }
}

// ── 11. Field name with leaf attribute preserved ────────────────────────────

proptest! {
    #[test]
    fn field_name_with_leaf_attr_preserved(count in 1usize..=4) {
        let names: Vec<String> = (0..count).map(|i| format!("tok_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names);
        for f in &s.fields {
            prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 12. Field name with skip attribute preserved ────────────────────────────

proptest! {
    #[test]
    fn field_name_with_skip_attr_preserved(count in 1usize..=3) {
        let names: Vec<String> = (0..count).map(|i| format!("meta_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::skip(false)]
                #ident: bool
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names);
    }
}

// ── 13. Mixed leaf, skip, and plain fields preserve names ───────────────────

proptest! {
    #[test]
    fn mixed_field_annotations_preserve_names(n_leaf in 1usize..=2, n_skip in 0usize..=2, n_plain in 1usize..=2) {
        let mut all_names: Vec<String> = Vec::new();
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_leaf {
            let name = format!("leaf_{i}");
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            tokens.push(quote::quote! {
                #[adze::leaf(pattern = r"\d+")]
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

// ── 14. Field count matches struct field count ──────────────────────────────

proptest! {
    #[test]
    fn field_name_count_matches_struct(count in 0usize..=8) {
        if count == 0 {
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct S;
            }).unwrap();
            prop_assert_eq!(struct_field_names(&s).len(), 0);
        } else {
            let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote::quote! { #ident: u8 }
            }).collect();
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct S { #(#fields),* }
            }).unwrap();
            prop_assert_eq!(struct_field_names(&s).len(), count);
        }
    }
}

// ── 15. Field name types preserved alongside names ──────────────────────────

proptest! {
    #[test]
    fn field_types_alongside_names(count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                quote::quote! { #ident: String }
            } else {
                quote::quote! { #ident: i32 }
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for (i, f) in s.fields.iter().enumerate() {
            prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), format!("f{i}"));
            let ty_str = f.ty.to_token_stream().to_string();
            if i % 2 == 0 {
                prop_assert_eq!(ty_str, "String");
            } else {
                prop_assert_eq!(ty_str, "i32");
            }
        }
    }
}

// ── 16. Multiple structs field names independent ────────────────────────────

proptest! {
    #[test]
    fn multiple_structs_field_names_independent(n_a in 1usize..=3, n_b in 1usize..=3) {
        let fields_a: Vec<proc_macro2::TokenStream> = (0..n_a).map(|i| {
            let ident = syn::Ident::new(&format!("a_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let fields_b: Vec<proc_macro2::TokenStream> = (0..n_b).map(|i| {
            let ident = syn::Ident::new(&format!("b_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: String }
        }).collect();
        let sa: ItemStruct = syn::parse2(quote::quote! {
            pub struct A { #(#fields_a),* }
        }).unwrap();
        let sb: ItemStruct = syn::parse2(quote::quote! {
            pub struct B { #(#fields_b),* }
        }).unwrap();
        let names_a = struct_field_names(&sa);
        let names_b = struct_field_names(&sb);
        prop_assert_eq!(names_a.len(), n_a);
        prop_assert_eq!(names_b.len(), n_b);
        for i in 0..n_a {
            prop_assert_eq!(&names_a[i], &format!("a_{i}"));
        }
        for i in 0..n_b {
            prop_assert_eq!(&names_b[i], &format!("b_{i}"));
        }
    }
}

// ── 17. Field name with Vec<T> typed field ──────────────────────────────────

proptest! {
    #[test]
    fn field_name_with_vec_type(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("items_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: Vec<i32> }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let names = struct_field_names(&s);
        for i in 0..count {
            prop_assert_eq!(&names[i], &format!("items_{i}"));
        }
    }
}

// ── 18. Field name with Option<T> typed field ───────────────────────────────

proptest! {
    #[test]
    fn field_name_with_option_type(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("opt_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: Option<String> }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let names = struct_field_names(&s);
        for i in 0..count {
            prop_assert_eq!(&names[i], &format!("opt_{i}"));
        }
    }
}

// ── 19. Field name with Box<T> typed field ──────────────────────────────────

proptest! {
    #[test]
    fn field_name_with_box_type(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("child_{i}"), proc_macro2::Span::call_site());
            quote::quote! { #ident: Box<Expr> }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let names = struct_field_names(&s);
        for i in 0..count {
            prop_assert_eq!(&names[i], &format!("child_{i}"));
        }
    }
}

// ── 20. Enum variant named vs unnamed field ident_strs ──────────────────────

proptest! {
    #[test]
    fn enum_named_vs_unnamed_ident_strs(n_named in 1usize..=3, n_unnamed in 1usize..=3) {
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

// ── 21. Field name ident.to_string() matches expected ───────────────────────

proptest! {
    #[test]
    fn field_ident_to_string_matches(idx in 0usize..=5) {
        let names = ["x", "my_value", "a_b_c", "item0", "the_field", "long_snake_case_name"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #ident: i32 }
        }).unwrap();
        let field_ident = s.fields.iter().next().unwrap().ident.as_ref().unwrap();
        prop_assert_eq!(field_ident.to_string(), name);
    }
}

// ── 22. Field name with leading underscore ──────────────────────────────────

proptest! {
    #[test]
    fn field_name_with_leading_underscore(count in 1usize..=4) {
        let names: Vec<String> = (0..count).map(|i| format!("_{i}")).collect();
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

// ── 23. Field name with numbers in name ─────────────────────────────────────

proptest! {
    #[test]
    fn field_name_with_numbers(num in 0u32..=999) {
        let name = format!("field_{num}");
        let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #ident: u32 }
        }).unwrap();
        let names = struct_field_names(&s);
        prop_assert_eq!(names.len(), 1);
        prop_assert_eq!(&names[0], &name);
    }
}

// ── 24. Enum variant field names with prec attribute ────────────────────────

proptest! {
    #[test]
    fn field_names_with_prec_attr(prec in 1i32..=10, n_fields in 1usize..=3) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let expected: Vec<String> = (0..n_fields).map(|i| format!("op_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V { #(#fields),* }
            }
        }).unwrap();
        let names = variant_field_names(&e.variants[0]);
        prop_assert_eq!(names, expected);
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// ── 25. Unit struct has no field names ───────────────────────────────────────

proptest! {
    #[test]
    fn unit_struct_no_field_names(idx in 0usize..=3) {
        let names = ["A", "MyStruct", "Token", "Node"];
        let ident = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct #ident;
        }).unwrap();
        prop_assert_eq!(struct_field_names(&s).len(), 0);
        prop_assert_eq!(struct_field_ident_strs(&s).len(), 0);
    }
}

// ── 26. Field names from enum with multiple named variants ──────────────────

proptest! {
    #[test]
    fn field_names_across_multiple_named_variants(n_variants in 2usize..=4, n_fields in 1usize..=3) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..n_variants).map(|vi| {
            let vname = syn::Ident::new(&format!("V{vi}"), proc_macro2::Span::call_site());
            let fields: Vec<proc_macro2::TokenStream> = (0..n_fields).map(|fi| {
                let fname = syn::Ident::new(
                    &format!("v{vi}_f{fi}"),
                    proc_macro2::Span::call_site(),
                );
                quote::quote! { #fname: i32 }
            }).collect();
            quote::quote! { #vname { #(#fields),* } }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        for vi in 0..n_variants {
            let names = variant_field_names(&e.variants[vi]);
            prop_assert_eq!(names.len(), n_fields);
            for fi in 0..n_fields {
                prop_assert_eq!(&names[fi], &format!("v{vi}_f{fi}"));
            }
        }
    }
}

// ── 27. Struct field ordering matches definition order ──────────────────────

proptest! {
    #[test]
    fn struct_field_order_is_definition_order(count in 2usize..=6) {
        // Use reverse-alphabetical names to verify ordering is positional, not sorted
        let names: Vec<String> = (0..count)
            .map(|i| format!("{}_field", (b'z' - i as u8) as char))
            .collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: i32 }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names);
    }
}

// ── 28. Leaf annotation on named field doesn't change name ──────────────────

proptest! {
    #[test]
    fn leaf_annotation_does_not_change_field_name(count in 1usize..=4) {
        let names: Vec<String> = (0..count).map(|i| format!("tok_{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = names.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! {
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                #ident: i32
            }
        }).collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual, names);
    }
}

// ── 29. Enum with mixed named and unit variants ─────────────────────────────

proptest! {
    #[test]
    fn enum_mixed_named_and_unit_variants(n_named in 1usize..=3, n_unit in 1usize..=3) {
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_named {
            let vname = syn::Ident::new(&format!("N{i}"), proc_macro2::Span::call_site());
            let fname = syn::Ident::new(&format!("val_{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #vname { #fname: i32 } });
        }
        for i in 0..n_unit {
            let vname = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            variant_tokens.push(quote::quote! { #vname });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        for i in 0..n_named {
            let names = variant_field_names(&e.variants[i]);
            prop_assert_eq!(names, vec![format!("val_{i}")]);
        }
        for i in 0..n_unit {
            let names = variant_field_names(&e.variants[n_named + i]);
            prop_assert!(names.is_empty());
        }
    }
}

// ── 30. Field names unique within a single struct ───────────────────────────

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

// ── 31. Delimited attribute preserves field name ────────────────────────────

proptest! {
    #[test]
    fn delimited_attr_preserves_field_name(idx in 0usize..=3) {
        let names = ["items", "elements", "values", "entries"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                #ident: Vec<i32>
            }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual.len(), 1);
        prop_assert_eq!(&actual[0], name);
    }
}

// ── 32. Repeat attribute preserves field name ───────────────────────────────

proptest! {
    #[test]
    fn repeat_attr_preserves_field_name(idx in 0usize..=3) {
        let names = ["numbers", "tokens", "stmts", "exprs"];
        let name = names[idx];
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                #ident: Vec<i32>
            }
        }).unwrap();
        let actual = struct_field_names(&s);
        prop_assert_eq!(actual.len(), 1);
        prop_assert_eq!(&actual[0], name);
    }
}

// ── 33. Enum variant field ordering matches definition ──────────────────────

proptest! {
    #[test]
    fn enum_variant_field_ordering(count in 2usize..=5) {
        let expected: Vec<String> = (0..count)
            .rev()
            .map(|i| format!("field_{i}"))
            .collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote::quote! { #ident: u64 }
        }).collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { V { #(#fields),* } }
        }).unwrap();
        let actual = variant_field_names(&e.variants[0]);
        prop_assert_eq!(actual, expected);
    }
}
