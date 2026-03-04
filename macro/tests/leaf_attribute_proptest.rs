#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::leaf]` attribute handling in adze-macro.
//!
//! Uses proptest to test leaf attribute parsing, text patterns, regex patterns,
//! enum variant application, struct application, combined attributes,
//! attribute preservation, and expansion output.

use adze_common::NameValueExpr;
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemStruct, Token};

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

fn extract_text_value(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path == "text").unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        s.value()
    } else {
        panic!("Expected string literal for text param");
    }
}

fn extract_pattern_value(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path == "pattern").unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        s.value()
    } else {
        panic!("Expected string literal for pattern param");
    }
}

fn has_param(attr: &Attribute, key: &str) -> bool {
    let params = leaf_params(attr);
    params.iter().any(|p| p.path == key)
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

// ── 1. Leaf attribute detected on struct field ──────────────────────────────

proptest! {
    #[test]
    fn leaf_attr_detected_on_struct_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\s"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 2. Leaf text param parsing ──────────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_param_parsed(idx in 0usize..=5) {
        let texts = ["+", "-", "*", "if", "else", "fn"];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 1);
        prop_assert_eq!(params[0].path.to_string(), "text");
    }
}

// ── 3. Leaf regex pattern parsing ───────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_regex_pattern_parsed(idx in 0usize..=4) {
        let pats = [r"\d+", r"[a-zA-Z_]\w*", r"\s+", r"[0-9a-fA-F]+", r".+"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                tok: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 4. Leaf on enum unit variant ────────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_on_enum_unit_variant(idx in 0usize..=4) {
        let keywords = ["if", "else", "while", "for", "return"];
        let kw = keywords[idx];
        let name = syn::Ident::new(&format!("Kw{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Keyword {
                #[adze::leaf(text = #kw)]
                #name
            }
        }).unwrap();
        prop_assert!(matches!(e.variants[0].fields, Fields::Unit));
        prop_assert_eq!(extract_text_value(find_leaf_attr(&e.variants[0].attrs)), kw);
    }
}

// ── 5. Leaf on struct with named field ──────────────────────────────────────

proptest! {
    #[test]
    fn leaf_on_struct_named_field(idx in 0usize..=3) {
        let field_names = ["_op", "_sep", "_delim", "_tok"];
        let fname = field_names[idx];
        let ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = "+")]
                #ident: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), fname);
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 6. Leaf combined with prec_left ─────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_prec_left(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        let v = &e.variants[0];
        prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
        if let Fields::Unnamed(ref u) = v.fields {
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 7. Leaf combined with prec_right ────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_prec_right(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_right(#lit)]
                Cons(
                    Box<Expr>,
                    #[adze::leaf(text = "::")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = find_leaf_attr(&u.unnamed[1].attrs);
            prop_assert_eq!(extract_text_value(attr), "::");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 8. Leaf attribute preserved after roundtrip ─────────────────────────────

proptest! {
    #[test]
    fn leaf_attr_preserved_after_roundtrip(idx in 0usize..=5) {
        let texts = ["hello", "world", "+", "!=", "let", "::"];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        let token_str = e.to_token_stream().to_string();
        let e2: ItemEnum = syn::parse_str(&token_str).unwrap();
        let attr2 = find_leaf_attr(&e2.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr2), t);
    }
}

// ── 9. Leaf pattern preserved after roundtrip ───────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_preserved_after_roundtrip(idx in 0usize..=3) {
        let pats = [r"\d+", r"\w+", r"[a-z]+", r"\s"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let token_str = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&token_str).unwrap();
        let field2 = s2.fields.iter().next().unwrap();
        let attr2 = find_leaf_attr(&field2.attrs);
        prop_assert_eq!(extract_pattern_value(attr2), pat);
    }
}

// ── 10. Leaf with transform has two params ──────────────────────────────────

proptest! {
    #[test]
    fn leaf_with_transform_has_two_params(idx in 0usize..=2) {
        let pats = [r"\d+", r"[0-9]+", r"\w+"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                value: i32,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(params[0].path.to_string(), "pattern");
        prop_assert_eq!(params[1].path.to_string(), "transform");
    }
}

// ── 11. Leaf text on multiple unit variants ─────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_on_multiple_unit_variants(count in 2usize..=8) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
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
        for i in 0..count {
            let attr = find_leaf_attr(&e.variants[i].attrs);
            prop_assert_eq!(extract_text_value(attr), format!("kw{i}"));
        }
    }
}

// ── 12. Leaf on enum variant with unnamed fields ────────────────────────────

proptest! {
    #[test]
    fn leaf_on_enum_variant_unnamed_field(idx in 0usize..=3) {
        let pats = [r"\d+", r"\w+", r"[a-z]+", r"[0-9]*"];
        let pat = pats[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                    i32
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
            prop_assert!(has_param(attr, "transform"));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 13. Leaf on enum variant with named fields ──────────────────────────────

proptest! {
    #[test]
    fn leaf_on_enum_variant_named_field(idx in 0usize..=3) {
        let ops = ["=", "+=", "-=", "*="];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Stmt {
                Assign {
                    #[adze::leaf(text = #op)]
                    _op: (),
                    value: Box<Expr>,
                }
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            let op_field = &n.named[0];
            let attr = find_leaf_attr(&op_field.attrs);
            prop_assert_eq!(extract_text_value(attr), op);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 14. Leaf combined with language attr on enum ────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_language_attr(count in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("kw{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        for v in &e.variants {
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 15. Leaf combined with extra attr on struct ─────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_extra_attr(idx in 0usize..=2) {
        let pats = [r"\s", r"\s+", r"//[^\n]*"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 16. Leaf combined with word attr on struct ──────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_word_attr(idx in 0usize..=2) {
        let pats = [r"[a-zA-Z_]\w*", r"[a-z]+", r"\w+"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let names = adze_attr_names(&s.attrs);
        prop_assert!(names.contains(&"word".to_string()));
        let field = s.fields.iter().next().unwrap();
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 17. Leaf text with special characters ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_special_characters(idx in 0usize..=5) {
        let specials = ["->", "=>", "::", "<<", ">>", ".."];
        let sp = specials[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #sp)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), sp);
    }
}

// ── 18. Leaf text with Unicode ──────────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_unicode_values(idx in 0usize..=5) {
        let unicode = ["→", "λ", "∀", "≠", "∈", "α"];
        let u = unicode[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #u)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), u);
    }
}

// ── 19. Leaf text field type is unit ────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_field_is_unit_type(idx in 0usize..=4) {
        let ops = ["+", "-", "=", ";", ":"];
        let op = ops[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #op)]
                _op: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "()");
    }
}

// ── 20. Leaf pattern field type is String ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_field_is_string_type(idx in 0usize..=3) {
        let pats = [r"\d+", r"\w+", r"[a-z]+", r".+"];
        let pat = pats[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                tok: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "String");
    }
}

// ── 21. Leaf in binary expression variant ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_in_binary_expression_variant(idx in 0usize..=4) {
        let ops = ["+", "-", "*", "/", "%"];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                BinOp(
                    Box<Expr>,
                    #[adze::leaf(text = #op)]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), 3);
            prop_assert!(!u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            prop_assert!(!u.unnamed[2].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            prop_assert_eq!(extract_text_value(find_leaf_attr(&u.unnamed[1].attrs)), op);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 22. Multiple leaf fields in same struct ─────────────────────────────────

proptest! {
    #[test]
    fn multiple_leaf_fields_in_struct(count in 2usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("_f{i}"), proc_macro2::Span::call_site());
                let text = format!("t{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name: ()
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let leaf_count = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        prop_assert_eq!(leaf_count, count);
    }
}

// ── 23. Leaf attr ordering preserved on variant ─────────────────────────────

proptest! {
    #[test]
    fn leaf_attr_ordering_preserved(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                #[adze::leaf(text = "+")]
                V
            }
        }).unwrap();
        let names = adze_attr_names(&e.variants[0].attrs);
        prop_assert_eq!(names, vec!["prec_left", "leaf"]);
    }
}

// ── 24. Leaf text with empty string ─────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_empty_string(_dummy in 0usize..=1) {
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = "")]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), "");
    }
}

// ── 25. Leaf with text and transform together ───────────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_transform(idx in 0usize..=2) {
        let texts = ["true", "false", "null"];
        let t = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #t, transform = |_v| true)]
                val: bool,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert!(has_param(attr, "text"));
        prop_assert!(has_param(attr, "transform"));
        prop_assert_eq!(extract_text_value(attr), t);
    }
}

// ── 26. Leaf mixed text and pattern in same enum ────────────────────────────

proptest! {
    #[test]
    fn leaf_mixed_text_and_pattern(n_text in 1usize..=3, n_pattern in 1usize..=3) {
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_text {
            let name = syn::Ident::new(&format!("Txt{i}"), proc_macro2::Span::call_site());
            let tv = format!("kw{i}");
            tokens.push(quote::quote! {
                #[adze::leaf(text = #tv)]
                #name
            });
        }
        for i in 0..n_pattern {
            let name = syn::Ident::new(&format!("Pat{i}"), proc_macro2::Span::call_site());
            tokens.push(quote::quote! {
                #name(
                    #[adze::leaf(pattern = r"\w+")]
                    String
                )
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_text + n_pattern);
        for i in 0..n_text {
            let params = leaf_params(find_leaf_attr(&e.variants[i].attrs));
            prop_assert_eq!(params[0].path.to_string(), "text");
        }
        for i in 0..n_pattern {
            let v = &e.variants[n_text + i];
            if let Fields::Unnamed(ref u) = v.fields {
                let params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
                prop_assert_eq!(params[0].path.to_string(), "pattern");
            } else {
                prop_assert!(false, "Expected unnamed");
            }
        }
    }
}

// ── 27. Leaf combined with skip field ───────────────────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_skip_field(idx in 0usize..=2) {
        let texts = [":", "=", ";"];
        let t = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Node {
                #[adze::leaf(text = #t)]
                _sep: (),
                #[adze::skip(0)]
                index: usize,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert!(!fields[0].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert!(!fields[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 28. Leaf text in grammar module parses correctly ─────────────────────────

proptest! {
    #[test]
    fn leaf_text_in_grammar_module(idx in 0usize..=2) {
        use syn::ItemMod;
        let keywords = ["if", "else", "while"];
        let kw = keywords[idx];
        let m: ItemMod = syn::parse2(quote::quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Keyword {
                    #[adze::leaf(text = #kw)]
                    Kw,
                }
            }
        }).unwrap();
        let items = &m.content.as_ref().unwrap().1;
        let e = items.iter().find_map(|i| {
            if let syn::Item::Enum(e) = i { Some(e) } else { None }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), kw);
    }
}

// ── 29. Leaf on tuple struct field ──────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_on_tuple_struct_field(idx in 0usize..=3) {
        let texts = ["+", "->", "::", "=>"];
        let t = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Op(
                #[adze::leaf(text = #t)]
                ()
            );
        }).unwrap();
        if let Fields::Unnamed(ref u) = s.fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_text_value(attr), t);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 30. Leaf param count is exactly 1 for text-only ─────────────────────────

proptest! {
    #[test]
    fn leaf_text_only_has_one_param(idx in 0usize..=4) {
        let texts = ["a", "bb", "ccc", "dd", "eeeee"];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(leaf_params(attr).len(), 1);
    }
}

// ── 31. Leaf text values distinct across variants ───────────────────────────

proptest! {
    #[test]
    fn leaf_text_values_are_distinct(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("unique_{i}");
                quote::quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        let values: Vec<String> = e.variants.iter()
            .map(|v| extract_text_value(find_leaf_attr(&v.attrs)))
            .collect();
        let unique: std::collections::HashSet<_> = values.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 32. Leaf with long string text ──────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_with_long_string_text(repeat in 1usize..=10) {
        let text = "xy".repeat(repeat);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #text)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let value = extract_text_value(attr);
        prop_assert_eq!(value.len(), repeat * 2);
        prop_assert_eq!(value, text);
    }
}

// ── 33. Leaf in delimited attribute ─────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_in_delimited_attribute(idx in 0usize..=2) {
        let delims = [",", ";", "|"];
        let d = delims[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::delimited(
                    #[adze::leaf(text = #d)]
                    ()
                )]
                items: Vec<Item>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let delim_attr = field.attrs.iter()
            .find(|a| is_adze_attr(a, "delimited"))
            .unwrap();
        let ftp: adze_common::FieldThenParams = delim_attr.parse_args().unwrap();
        let inner_leaf = find_leaf_attr(&ftp.field.attrs);
        prop_assert_eq!(extract_text_value(inner_leaf), d);
    }
}

// ── 34. Leaf combined with prec (no associativity) ──────────────────────────

proptest! {
    #[test]
    fn leaf_combined_with_prec_no_assoc(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec(#lit)]
                Compare(
                    Box<Expr>,
                    #[adze::leaf(text = "==")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        let v = &e.variants[0];
        prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "prec")));
        if let Fields::Unnamed(ref u) = v.fields {
            let attr = find_leaf_attr(&u.unnamed[1].attrs);
            prop_assert_eq!(extract_text_value(attr), "==");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 35. Leaf pattern in grammar module parses correctly ──────────────────────

proptest! {
    #[test]
    fn leaf_pattern_in_grammar_module(idx in 0usize..=2) {
        use syn::ItemMod;
        let names = ["arith", "lang", "calc"];
        let gname = names[idx];
        let m: ItemMod = syn::parse2(quote::quote! {
            #[adze::grammar(#gname)]
            mod grammar {
                #[adze::language]
                pub struct Token {
                    #[adze::leaf(pattern = r"\w+")]
                    tok: String,
                }
            }
        }).unwrap();
        let items = &m.content.as_ref().unwrap().1;
        let s = items.iter().find_map(|i| {
            if let syn::Item::Struct(s) = i { Some(s) } else { None }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), r"\w+");
        // Verify grammar name is on the module
        let grammar_attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
        let grammar_name: syn::LitStr = grammar_attr.parse_args().unwrap();
        prop_assert_eq!(grammar_name.value(), gname);
    }
}
