#![allow(clippy::needless_range_loop)]

//! Property-based tests for delimited/separated list handling in adze-macro.
//!
//! Uses proptest to generate randomized separator choices, field counts,
//! and annotation combinations, then verifies that syn correctly parses
//! and preserves delimited attributes on `Vec` fields.

use std::collections::HashSet;

use adze_common::{FieldThenParams, NameValueExpr, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn find_leaf_attr(attrs: &[Attribute]) -> &Attribute {
    attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap()
}

fn extract_delim_text(field: &syn::Field) -> String {
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .expect("no delimited attr");
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = find_leaf_attr(&ftp.field.attrs);
    let params = leaf_params(inner_leaf);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        s.value()
    } else {
        panic!("Expected string literal in delimiter leaf");
    }
}

fn extract_delim_param_key(field: &syn::Field) -> String {
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .expect("no delimited attr");
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = find_leaf_attr(&ftp.field.attrs);
    let params = leaf_params(inner_leaf);
    params[0].path.to_string()
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[syn::Item] {
    &m.content.as_ref().expect("module has no content").1
}

// ── 1. Delimited attribute with various text separators ─────────────────────

proptest! {
    #[test]
    fn delimited_various_text_separators(idx in 0usize..=7) {
        let seps = [",", ";", "|", "::", "=>", ".", "->", "+"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(extract_delim_text(field), sep);
    }
}

// ── 2. Delimited separator specification uses "text" key ────────────────────

proptest! {
    #[test]
    fn delimited_separator_uses_text_key(idx in 0usize..=4) {
        let seps = [",", ";", "|", ".", "::"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(extract_delim_param_key(field), "text");
    }
}

// ── 3. Delimited separator specification uses "pattern" key ─────────────────

proptest! {
    #[test]
    fn delimited_separator_uses_pattern_key(idx in 0usize..=3) {
        let pats = [r"\s*,\s*", r"\s*;\s*", r"\s+", r",\s*"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(pattern = #pat)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(extract_delim_param_key(field), "pattern");
    }
}

// ── 4. Delimiter inner field type is always unit ────────────────────────────

proptest! {
    #[test]
    fn delimiter_inner_type_always_unit(idx in 0usize..=5) {
        let seps = [",", ";", "|", "::", "=>", "."];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let delim = field.attrs.iter().find(|a| is_adze_attr(a, "delimited")).unwrap();
        let ftp: FieldThenParams = delim.parse_args().unwrap();
        prop_assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    }
}

// ── 5. Delimited combined with repeat(non_empty = true) ─────────────────────

proptest! {
    #[test]
    fn delimited_combined_with_repeat_non_empty_true(idx in 0usize..=3) {
        let seps = [",", ";", "|", "::"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let names = adze_attr_names(&field.attrs);
        prop_assert!(names.contains(&"repeat".to_string()));
        prop_assert!(names.contains(&"delimited".to_string()));
        prop_assert_eq!(extract_delim_text(field), sep);
    }
}

// ── 6. Delimited combined with repeat(non_empty = false) ────────────────────

proptest! {
    #[test]
    fn delimited_combined_with_repeat_non_empty_false(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = false)]
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let repeat_attr = field.attrs.iter().find(|a| is_adze_attr(a, "repeat")).unwrap();
        let params = repeat_attr
            .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
            .unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = &params[0].expr {
            prop_assert!(!b.value);
        } else {
            prop_assert!(false, "Expected bool literal");
        }
    }
}

// ── 7. Delimited on Vec fields with different inner types ───────────────────

proptest! {
    #[test]
    fn delimited_on_vec_with_different_inner_types(idx in 0usize..=4) {
        let types = ["Number", "Expr", "Stmt", "Token", "Ident"];
        let ty_name = syn::Ident::new(types[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<#ty_name>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), types[idx]);
    }
}

// ── 8. Delimited ordering: delimited first then repeat ──────────────────────

proptest! {
    #[test]
    fn delimited_first_then_repeat_order(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                #[adze::repeat(non_empty = false)]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let names = adze_attr_names(&field.attrs);
        prop_assert_eq!(&names[0], "delimited");
        prop_assert_eq!(&names[1], "repeat");
    }
}

// ── 9. Delimited ordering: repeat first then delimited ──────────────────────

proptest! {
    #[test]
    fn repeat_first_then_delimited_order(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let names = adze_attr_names(&field.attrs);
        prop_assert_eq!(&names[0], "repeat");
        prop_assert_eq!(&names[1], "delimited");
    }
}

// ── 10. Delimited round-trip through token stream ───────────────────────────

proptest! {
    #[test]
    fn delimited_roundtrip_token_stream(idx in 0usize..=4) {
        let seps = [",", ";", "|", "::", "=>"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let token_str = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&token_str).unwrap();
        let field2 = s2.fields.iter().next().unwrap();
        prop_assert_eq!(extract_delim_text(field2), sep);
    }
}

// ── 11. Delimited combined with skip field ──────────────────────────────────

proptest! {
    #[test]
    fn delimited_with_skip_field(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
                #[adze::skip(0usize)]
                count: usize,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert_eq!(fields.len(), 2);
        prop_assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        prop_assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
    }
}

// ── 12. Delimited combined with leaf text fields ────────────────────────────

proptest! {
    #[test]
    fn delimited_with_surrounding_leaf_fields(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = "(")]
                _open: (),
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
                #[adze::leaf(text = ")")]
                _close: (),
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert_eq!(fields.len(), 3);
        prop_assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        prop_assert!(fields[2].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert_eq!(extract_delim_text(fields[1]), sep);
    }
}

// ── 13. Delimited on unnamed enum variant Vec field ─────────────────────────

proptest! {
    #[test]
    fn delimited_on_unnamed_enum_vec_field(idx in 0usize..=3) {
        let seps = [",", ";", "|", "::"];
        let sep = seps[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                List(
                    #[adze::delimited(
                        #[adze::leaf(text = #sep)]
                        ()
                    )]
                    Vec<Item>
                ),
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let field = &u.unnamed[0];
            prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
            prop_assert_eq!(extract_delim_text(field), sep);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 14. Delimited on named enum variant Vec field ───────────────────────────

proptest! {
    #[test]
    fn delimited_on_named_enum_vec_field(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Node {
                Block {
                    #[adze::leaf(text = "{")]
                    _open: (),
                    #[adze::delimited(
                        #[adze::leaf(text = #sep)]
                        ()
                    )]
                    stmts: Vec<Stmt>,
                    #[adze::leaf(text = "}")]
                    _close: (),
                },
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            let stmts = n.named.iter().find(|f| f.ident.as_ref().unwrap() == "stmts").unwrap();
            prop_assert!(stmts.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
            prop_assert_eq!(extract_delim_text(stmts), sep);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 15. Delimited combined with prec_left on enum variant ───────────────────

proptest! {
    #[test]
    fn delimited_with_prec_left(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Lit(i32),
                #[adze::prec_left(#lit)]
                Call(
                    Box<Expr>,
                    #[adze::leaf(text = "(")]
                    (),
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    Vec<Expr>,
                    #[adze::leaf(text = ")")]
                    (),
                ),
            }
        }).unwrap();
        prop_assert!(e.variants[1].attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            let vec_field = &u.unnamed[2];
            prop_assert!(vec_field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
            prop_assert_eq!(extract_delim_text(vec_field), ",");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 16. Multiple Vec fields with different delimiters ───────────────────────

proptest! {
    #[test]
    fn multiple_vec_different_delimiters(idx in 0usize..=2) {
        let sep_pairs = [(",", ";"), ("|", "::"), (".", "=>")];
        let (sep1, sep2) = sep_pairs[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep1)]
                    ()
                )]
                first: Vec<A>,
                #[adze::delimited(
                    #[adze::leaf(text = #sep2)]
                    ()
                )]
                second: Vec<B>,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert_eq!(extract_delim_text(fields[0]), sep1);
        prop_assert_eq!(extract_delim_text(fields[1]), sep2);
    }
}

// ── 17. FieldThenParams has no extra params for simple delimiters ────────────

proptest! {
    #[test]
    fn field_then_params_no_extra(idx in 0usize..=4) {
        let seps = [",", ";", "|", "::", "=>"];
        let sep = seps[idx];
        let ftp: FieldThenParams = syn::parse2(quote::quote! {
            #[adze::leaf(text = #sep)]
            ()
        }).unwrap();
        prop_assert!(ftp.comma.is_none());
        prop_assert!(ftp.params.is_empty());
        prop_assert_eq!(ftp.field.attrs.len(), 1);
    }
}

// ── 18. Delimited with Unicode separators ───────────────────────────────────

proptest! {
    #[test]
    fn delimited_unicode_separators(idx in 0usize..=4) {
        let seps = ["→", "•", "│", "╬", "∘"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(extract_delim_text(field), sep);
    }
}

// ── 19. Delimited with multi-character separators ───────────────────────────

proptest! {
    #[test]
    fn delimited_multi_char_separators(idx in 0usize..=5) {
        let seps = ["::", "=>", "->", "<>", "&&", "||"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let value = extract_delim_text(field);
        prop_assert!(value.len() >= 2);
        prop_assert_eq!(value, sep);
    }
}

// ── 20. Vec<T> inner type extraction with delimited ─────────────────────────

proptest! {
    #[test]
    fn vec_inner_type_with_delimited(idx in 0usize..=4) {
        let types = ["Number", "Expr", "Stmt", "Token", "Ident"];
        let ty_name = syn::Ident::new(types[idx], proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<#ty_name>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), types[idx]);
    }
}

// ── 21. wrap_leaf_type with Vec in skip set alongside delimited ──────────────

proptest! {
    #[test]
    fn wrap_leaf_vec_skip_with_delimited(idx in 0usize..=3) {
        let types = ["Number", "Expr", "Stmt", "Item"];
        let ty_name = syn::Ident::new(types[idx], proc_macro2::Span::call_site());
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let ty: Type = syn::parse2(quote::quote!(Vec<#ty_name>)).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("Vec < adze :: WithLeaf < {} > >", types[idx]);
        prop_assert_eq!(wrapped.to_token_stream().to_string(), expected);
    }
}

// ── 22. Delimited attr count is exactly 1 on single-attr field ──────────────

proptest! {
    #[test]
    fn delimited_only_has_one_adze_attr(idx in 0usize..=3) {
        let seps = [",", ";", "|", "::"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
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

// ── 23. Combined repeat+delimited has exactly 2 adze attrs ──────────────────

proptest! {
    #[test]
    fn repeat_delimited_has_two_adze_attrs(idx in 0usize..=3) {
        let seps = [",", ";", "|", "::"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let adze_count = field.attrs.iter()
            .filter(|a| {
                let segs: Vec<_> = a.path().segments.iter().collect();
                segs.len() == 2 && segs[0].ident == "adze"
            })
            .count();
        prop_assert_eq!(adze_count, 2);
    }
}

// ── 24. Delimited in grammar module preserves attrs ─────────────────────────

proptest! {
    #[test]
    fn delimited_in_grammar_module_preserved(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct List {
                    #[adze::delimited(
                        #[adze::leaf(text = #sep)]
                        ()
                    )]
                    items: Vec<Item>,
                }

                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                }
            }
        });
        let items = module_items(&m);
        if let syn::Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        } else {
            prop_assert!(false, "Expected struct");
        }
    }
}

// ── 25. Delimited round-trip with repeat through token stream ───────────────

proptest! {
    #[test]
    fn delimited_with_repeat_roundtrip(idx in 0usize..=3) {
        let seps = [",", ";", "|", "::"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let token_str = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&token_str).unwrap();
        let field2 = s2.fields.iter().next().unwrap();
        let names = adze_attr_names(&field2.attrs);
        prop_assert!(names.contains(&"repeat".to_string()));
        prop_assert!(names.contains(&"delimited".to_string()));
        prop_assert_eq!(extract_delim_text(field2), sep);
    }
}

// ── 26. Delimited with dynamically generated field names ────────────────────

proptest! {
    #[test]
    fn delimited_preserves_field_name(idx in 0usize..=4) {
        let field_names = ["items", "values", "elements", "entries", "rows"];
        let fname = field_names[idx];
        let ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                #ident: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), fname);
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    }
}

// ── 27. Multiple delimited fields count ─────────────────────────────────────

proptest! {
    #[test]
    fn multiple_delimited_fields_count(count in 2usize..=4) {
        let field_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                let ty = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
                quote::quote! {
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    #name: Vec<#ty>
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#field_tokens),* }
        }).unwrap();
        let delimited_count = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "delimited")))
            .count();
        prop_assert_eq!(delimited_count, count);
    }
}

// ── 28. Delimited with single-char separators preserve length ───────────────

proptest! {
    #[test]
    fn delimited_single_char_sep_length(idx in 0usize..=6) {
        let seps = [",", ";", "|", ".", ":", "+", "-"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let value = extract_delim_text(field);
        prop_assert_eq!(value.len(), 1);
    }
}

// ── 29. Delimited in enum variant with repeat + prec_right ──────────────────

proptest! {
    #[test]
    fn delimited_enum_repeat_prec_right(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                Sequence(
                    #[adze::repeat(non_empty = true)]
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    Vec<Expr>
                ),
            }
        }).unwrap();
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let field = &u.unnamed[0];
            let names = adze_attr_names(&field.attrs);
            prop_assert!(names.contains(&"repeat".to_string()));
            prop_assert!(names.contains(&"delimited".to_string()));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 30. Delimited pattern value roundtrip ───────────────────────────────────

proptest! {
    #[test]
    fn delimited_pattern_roundtrip(idx in 0usize..=3) {
        let pats = [r"\s*,\s*", r"\s*;\s*", r"\s+", r"\t"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(pattern = #pat)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let token_str = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&token_str).unwrap();
        let field2 = s2.fields.iter().next().unwrap();
        let delim = field2.attrs.iter().find(|a| is_adze_attr(a, "delimited")).unwrap();
        let ftp: FieldThenParams = delim.parse_args().unwrap();
        let inner_leaf = find_leaf_attr(&ftp.field.attrs);
        let params = leaf_params(inner_leaf);
        prop_assert_eq!(params[0].path.to_string(), "pattern");
    }
}

// ── 31. Delimited combined with language attr on struct ──────────────────────

proptest! {
    #[test]
    fn delimited_with_language_attr(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::language]
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    }
}

// ── 32. Vec<Spanned<T>> with delimited ──────────────────────────────────────

proptest! {
    #[test]
    fn delimited_on_vec_spanned(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #sep)]
                    ()
                )]
                items: Vec<Spanned<Item>>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        let skip: HashSet<&str> = ["Spanned"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), "Spanned < Item >");
    }
}

// ── 33. Delimited separator distinct across fields ──────────────────────────

proptest! {
    #[test]
    fn delimited_separators_distinct(idx in 0usize..=2) {
        let sep_pairs = [(",", ";"), ("|", "::"), (".", "=>")];
        let (s1, s2) = sep_pairs[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #s1)]
                    ()
                )]
                first: Vec<A>,
                #[adze::delimited(
                    #[adze::leaf(text = #s2)]
                    ()
                )]
                second: Vec<B>,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        let v1 = extract_delim_text(fields[0]);
        let v2 = extract_delim_text(fields[1]);
        prop_assert_ne!(v1, v2);
    }
}

// ── 34. Delimited nested grammar module structures ──────────────────────────

proptest! {
    #[test]
    fn delimited_nested_grammar_structures(idx in 0usize..=1) {
        let row_seps = ["\n", "\\n"];
        let row_sep = row_seps[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("csv")]
            mod grammar {
                #[adze::language]
                pub struct CsvFile {
                    #[adze::delimited(
                        #[adze::leaf(text = #row_sep)]
                        ()
                    )]
                    rows: Vec<Row>,
                }

                pub struct Row {
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    cells: Vec<Cell>,
                }

                pub struct Cell {
                    #[adze::leaf(pattern = r"[^,\n]+")]
                    value: String,
                }
            }
        });
        let items = module_items(&m);
        let struct_names: Vec<_> = items.iter().filter_map(|i| {
            if let syn::Item::Struct(s) = i { Some(s.ident.to_string()) } else { None }
        }).collect();
        prop_assert!(struct_names.contains(&"CsvFile".to_string()));
        prop_assert!(struct_names.contains(&"Row".to_string()));
        prop_assert!(struct_names.contains(&"Cell".to_string()));
    }
}

// ── 35. Delimited with extra annotation in grammar ──────────────────────────

proptest! {
    #[test]
    fn delimited_with_extra_in_grammar(idx in 0usize..=2) {
        let seps = [",", ";", "|"];
        let sep = seps[idx];
        let m = parse_mod(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct List {
                    #[adze::delimited(
                        #[adze::leaf(text = #sep)]
                        ()
                    )]
                    items: Vec<Item>,
                }

                pub struct Item {
                    #[adze::leaf(pattern = r"\w+")]
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
        let struct_names: Vec<_> = items.iter().filter_map(|i| {
            if let syn::Item::Struct(s) = i { Some(s.ident.to_string()) } else { None }
        }).collect();
        prop_assert!(struct_names.contains(&"List".to_string()));
        prop_assert!(struct_names.contains(&"Item".to_string()));
        prop_assert!(struct_names.contains(&"Whitespace".to_string()));
    }
}
