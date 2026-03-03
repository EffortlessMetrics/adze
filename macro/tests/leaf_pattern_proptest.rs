#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::leaf(pattern = "...")]` in adze-macro.
//!
//! Uses proptest to generate randomized pattern values, field counts, and
//! annotation combinations, then verifies that syn correctly parses and
//! preserves the leaf pattern attributes (which produce PATTERN / regex rules).

use adze_common::NameValueExpr;
use proptest::prelude::*;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemStruct, Token, parse_quote};

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

fn extract_pattern_value(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params
        .iter()
        .find(|p| p.path.to_string() == "pattern")
        .unwrap();
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

// ── 1. Leaf pattern detection on struct field ───────────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_detected_on_struct_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"[0-9]*"];
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

// ── 2. Pattern value extraction from struct field ───────────────────────────

proptest! {
    #[test]
    fn pattern_value_extracted_from_struct_field(idx in 0usize..=4) {
        let patterns = [r"\d+", r"\w+", r"[a-zA-Z_]\w*", r"\s+", r"[0-9a-fA-F]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 3. Different regex patterns ─────────────────────────────────────────────

proptest! {
    #[test]
    fn different_regex_patterns(idx in 0usize..=7) {
        let patterns = [
            r"\d+", r"\w+", r"[a-z]+", r"[A-Z][a-z]*",
            r"0x[0-9a-fA-F]+", r"[_a-zA-Z]\w*", r"\d+\.\d+", r"[^\s]+",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params[0].path.to_string(), "pattern");
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 4. Pattern combined with text in same enum ──────────────────────────────

proptest! {
    #[test]
    fn pattern_combined_with_text(n_text in 1usize..=3, n_pattern in 1usize..=3) {
        let tnames: Vec<syn::Ident> = (0..n_text)
            .map(|i| syn::Ident::new(&format!("Txt{i}"), proc_macro2::Span::call_site()))
            .collect();
        let pnames: Vec<syn::Ident> = (0..n_pattern)
            .map(|i| syn::Ident::new(&format!("Pat{i}"), proc_macro2::Span::call_site()))
            .collect();

        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_text {
            let name = &tnames[i];
            let tv = format!("kw{i}");
            tokens.push(quote::quote! {
                #[adze::leaf(text = #tv)]
                #name
            });
        }
        for i in 0..n_pattern {
            let name = &pnames[i];
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
            prop_assert!(matches!(e.variants[i].fields, Fields::Unit));
        }
        for i in 0..n_pattern {
            let v = &e.variants[n_text + i];
            if let Fields::Unnamed(ref u) = v.fields {
                let params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
                prop_assert_eq!(params[0].path.to_string(), "pattern");
            } else {
                prop_assert!(false, "Expected unnamed fields for pattern variant");
            }
        }
    }
}

// ── 5. Multiple patterns in same struct ─────────────────────────────────────

proptest! {
    #[test]
    fn multiple_patterns_in_same_struct(count in 2usize..=5) {
        let regexes = [r"\d+", r"\w+", r"[a-z]+", r"\s+", r"[A-Z]+"];
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("_f{i}"), proc_macro2::Span::call_site());
                let pat = regexes[i];
                quote::quote! {
                    #[adze::leaf(pattern = #pat)]
                    #name: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let leaf_fields: Vec<_> = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .collect();
        prop_assert_eq!(leaf_fields.len(), count);
        for i in 0..count {
            let attr = find_leaf_attr(&leaf_fields[i].attrs);
            prop_assert_eq!(extract_pattern_value(attr), regexes[i]);
        }
    }
}

// ── 6. Pattern with special regex characters ────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_special_characters(idx in 0usize..=7) {
        let patterns = [
            r"\d+\.\d+",      // escaped dot
            r"[^\n]+",        // negated char class
            r"\w+\?",         // escaped question mark
            r"a|b|c",         // alternation
            r"(foo|bar)",     // grouping
            r"\[.*\]",        // escaped brackets
            r"\{[0-9]+\}",   // escaped braces
            r"\\",            // escaped backslash
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 7. Pattern in enum variant (unnamed field) ──────────────────────────────

proptest! {
    #[test]
    fn pattern_in_enum_variant(idx in 0usize..=4) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"0x[0-9a-fA-F]+", r"\s"];
        let pat = patterns[idx];
        let name = syn::Ident::new(&format!("V{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #name(
                    #[adze::leaf(pattern = #pat)]
                    String
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 8. Pattern with transform has two params ────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_transform_has_two_params(idx in 0usize..=3) {
        let patterns = [r"\d+", r"-?\d+", r"\d+\.\d+", r"0[xX][0-9a-fA-F]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                val: i32,
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

// ── 9. Pattern value is always a string literal ─────────────────────────────

proptest! {
    #[test]
    fn pattern_value_is_str_lit(idx in 0usize..=5) {
        let patterns = [r"\d+", r"\w+", r"[a-z]", r".*", r"\s+", r"[^\n]*"];
        let pat = patterns[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { pattern = #pat }).unwrap();
        let is_str = matches!(
            nv.expr,
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(_), .. })
        );
        prop_assert!(is_str);
    }
}

// ── 10. Pattern param key is always "pattern" ───────────────────────────────

proptest! {
    #[test]
    fn pattern_param_key_is_pattern(idx in 0usize..=4) {
        let patterns = [r"\d+", r"[a-z]+", r"0x\w+", r"\S+", r".+"];
        let pat = patterns[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { pattern = #pat }).unwrap();
        prop_assert_eq!(nv.path.to_string(), "pattern");
    }
}

// ── 11. Pattern only has one param (no transform) ───────────────────────────

proptest! {
    #[test]
    fn pattern_only_has_one_param(idx in 0usize..=4) {
        let patterns = [r"\d+", r"\w+", r"[a-z]", r"\s", r".+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 1);
    }
}

// ── 12. Pattern roundtrip through token stream ──────────────────────────────

proptest! {
    #[test]
    fn pattern_roundtrip_token_stream(idx in 0usize..=5) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"\s+", r"[^\n]+", r"0x[0-9a-f]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        // Roundtrip: parse -> to_token_stream -> parse again
        let token_str = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&token_str).unwrap();
        let field = s2.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 13. Pattern with quantifiers ────────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_quantifiers(idx in 0usize..=5) {
        let patterns = [r"\d+", r"\d*", r"\d?", r"\d{3}", r"\d{2,4}", r"\d{1,}"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 14. Pattern with character classes ──────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_character_classes(idx in 0usize..=5) {
        let patterns = [
            r"[a-z]", r"[A-Z]", r"[0-9]", r"[a-zA-Z0-9_]", r"[^\s]", r"[[:alpha:]]",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 15. Pattern on unnamed enum field with transform ────────────────────────

proptest! {
    #[test]
    fn pattern_on_enum_field_with_transform(idx in 0usize..=3) {
        let patterns = [r"\d+", r"-?\d+", r"\d+\.\d+", r"[01]+"];
        let pat = patterns[idx];
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
            let params = leaf_params(attr);
            prop_assert_eq!(params.len(), 2);
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 16. Pattern on named enum variant field ─────────────────────────────────

proptest! {
    #[test]
    fn pattern_on_named_variant_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\S+"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Ident {
                    #[adze::leaf(pattern = #pat)]
                    name: String,
                }
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            let attr = find_leaf_attr(&n.named[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 17. Pattern preserves field name ────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_preserves_field_name(idx in 0usize..=3) {
        let field_names = ["name", "value", "token", "ident"];
        let fname = field_names[idx];
        let ident = syn::Ident::new(fname, proc_macro2::Span::call_site());
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\w+")]
                #ident: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        prop_assert_eq!(field.ident.as_ref().unwrap().to_string(), fname);
        prop_assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 18. Pattern field type is String ────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_field_type_is_string(idx in 0usize..=4) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\s+", r".+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "String");
    }
}

// ── 19. Pattern with anchors ────────────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_anchors(idx in 0usize..=3) {
        let patterns = [r"^\d+$", r"^[a-z]+", r"[0-9]+$", r"^\w+$"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 20. Pattern combined with prec_left ─────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_prec_left(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                #[adze::prec_left(#lit)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>
                ),
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                )
            }
        }).unwrap();
        // Check the Number variant has pattern
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), r"\d+");
        } else {
            prop_assert!(false, "Expected unnamed fields for Number");
        }
    }
}

// ── 21. Pattern on unit-type field (whitespace) ─────────────────────────────

proptest! {
    #[test]
    fn pattern_on_unit_type_field(idx in 0usize..=3) {
        let patterns = [r"\s", r"\s+", r"\n", r"\t"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Whitespace {
                #[adze::leaf(pattern = #pat)]
                _ws: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert_eq!(ty_str, "()");
    }
}

// ── 22. Multiple pattern variants in same enum ──────────────────────────────

proptest! {
    #[test]
    fn multiple_pattern_variants_in_enum(count in 2usize..=5) {
        let regexes = [r"\d+", r"\w+", r"[a-z]+", r"\s+", r"[A-Z]+"];
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let pat = regexes[i];
                quote::quote! {
                    #name(
                        #[adze::leaf(pattern = #pat)]
                        String
                    )
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), count);
        for i in 0..count {
            if let Fields::Unnamed(ref u) = e.variants[i].fields {
                let attr = find_leaf_attr(&u.unnamed[0].attrs);
                prop_assert_eq!(extract_pattern_value(attr), regexes[i]);
            } else {
                prop_assert!(false, "Expected unnamed fields");
            }
        }
    }
}

// ── 23. Pattern with alternation ────────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_alternation(idx in 0usize..=3) {
        let patterns = [r"true|false", r"yes|no", r"on|off", r"foo|bar|baz"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 24. Pattern with language attr on enum ──────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_language_attr(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            #[adze::language]
            pub enum Expr {
                Value(
                    #[adze::leaf(pattern = #pat)]
                    String
                )
            }
        }).unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 25. Pattern with extra attr on struct ───────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_extra_attr(idx in 0usize..=2) {
        let patterns = [r"\s", r"\s+", r"//[^\n]*"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::extra]
            pub struct Extra {
                #[adze::leaf(pattern = #pat)]
                _skip: (),
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 26. Pattern values are distinct across struct fields ────────────────────

proptest! {
    #[test]
    fn pattern_values_distinct_across_fields(count in 2usize..=5) {
        let regexes = [r"\d+", r"\w+", r"[a-z]+", r"\s+", r"[A-Z]+"];
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                let pat = regexes[i];
                quote::quote! {
                    #[adze::leaf(pattern = #pat)]
                    #name: String
                }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        let values: Vec<String> = s.fields.iter()
            .map(|f| extract_pattern_value(find_leaf_attr(&f.attrs)))
            .collect();
        let unique: std::collections::HashSet<_> = values.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 27. Pattern with word attr on struct ────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_word_attr(idx in 0usize..=2) {
        let patterns = [r"[a-zA-Z_]\w*", r"[a-z]+", r"\w+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = #pat)]
                name: String,
            }
        }).unwrap();
        prop_assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 28. Pattern with skip field in struct ───────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_skip_in_struct(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Node {
                #[adze::leaf(pattern = #pat)]
                val: String,
                #[adze::skip(0)]
                index: usize,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        prop_assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        prop_assert!(!fields[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 29. Pattern attr ordering preserved with prec ───────────────────────────

proptest! {
    #[test]
    fn pattern_attr_ordering_preserved(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                V(
                    Box<E>,
                    #[adze::leaf(pattern = r"\w+")]
                    String,
                    Box<E>
                )
            }
        }).unwrap();
        let variant_attrs = &e.variants[0].attrs;
        let names: Vec<String> = variant_attrs.iter()
            .filter_map(|a| {
                let segs: Vec<_> = a.path().segments.iter().collect();
                if segs.len() == 2 && segs[0].ident == "adze" {
                    Some(segs[1].ident.to_string())
                } else {
                    None
                }
            })
            .collect();
        prop_assert_eq!(names, vec!["prec_left"]);
        // Field-level leaf attr
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = find_leaf_attr(&u.unnamed[1].attrs);
            prop_assert_eq!(extract_pattern_value(attr), r"\w+");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 30. Pattern on tuple struct field ───────────────────────────────────────

proptest! {
    #[test]
    fn pattern_on_tuple_struct_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\S+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S(
                #[adze::leaf(pattern = #pat)]
                String
            );
        }).unwrap();
        if let Fields::Unnamed(ref u) = s.fields {
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 31. Long regex pattern strings ──────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_long_regex_strings(repeat in 1usize..=8) {
        let pat = "[a-z]".repeat(repeat);
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let value = extract_pattern_value(attr);
        prop_assert_eq!(value.len(), 5 * repeat);
        prop_assert_eq!(value, pat);
    }
}

// ── 32. Pattern with comment-like regex ─────────────────────────────────────

proptest! {
    #[test]
    fn pattern_comment_like_regex(idx in 0usize..=2) {
        let patterns = [r"//[^\n]*", r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", r"#[^\n]*"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct Comment {
                #[adze::leaf(pattern = #pat)]
                _comment: (),
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 33. Pattern combined with text in binary expression ─────────────────────

proptest! {
    #[test]
    fn pattern_and_text_in_binary_expr(idx in 0usize..=4) {
        let ops = ["+", "-", "*", "/", "%"];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                BinOp(
                    Box<Expr>,
                    #[adze::leaf(text = #op)]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        // Number variant: pattern
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
            prop_assert_eq!(params[0].path.to_string(), "pattern");
        } else {
            prop_assert!(false, "Expected unnamed fields for Number");
        }
        // BinOp variant: text
        if let Fields::Unnamed(ref u) = e.variants[1].fields {
            let params = leaf_params(find_leaf_attr(&u.unnamed[1].attrs));
            prop_assert_eq!(params[0].path.to_string(), "text");
        } else {
            prop_assert!(false, "Expected unnamed fields for BinOp");
        }
    }
}

// ── 34. Expansion determinism: same input yields identical token streams ─────

proptest! {
    #[test]
    fn expansion_determinism_same_tokens(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\S+"];
        let pat = patterns[idx];
        let mk = || -> proc_macro2::TokenStream {
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct S {
                    #[adze::leaf(pattern = #pat)]
                    val: String,
                }
            }).unwrap();
            s.to_token_stream()
        };
        prop_assert_eq!(mk().to_string(), mk().to_string());
    }
}

// ── 35. Pattern on Option<String> field ─────────────────────────────────────

proptest! {
    #[test]
    fn pattern_on_option_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\S+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: Option<String>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert!(ty_str.contains("Option"));
    }
}

// ── 36. Pattern on Vec field (repeated leaf) ────────────────────────────────

proptest! {
    #[test]
    fn pattern_on_vec_field(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                vals: Vec<String>,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
        let ty_str = field.ty.to_token_stream().to_string();
        prop_assert!(ty_str.contains("Vec"));
    }
}

// ── 37. Transform with explicit type annotation in closure ──────────────────

proptest! {
    #[test]
    fn transform_with_type_annotation(idx in 0usize..=3) {
        let patterns = [r"\d+", r"-?\d+", r"\d+\.\d+", r"0b[01]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v: &str| v.parse::<i64>().unwrap())]
                val: i64,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(params[0].path.to_string(), "pattern");
        prop_assert_eq!(params[1].path.to_string(), "transform");
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 38. Multiple leaf types: text and pattern in same struct ────────────────

proptest! {
    #[test]
    fn text_and_pattern_in_same_struct(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let texts = ["(", ")", ";"];
        let pat = patterns[idx];
        let txt = texts[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(text = #txt)]
                _open: (),
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        let text_attr = find_leaf_attr(&fields[0].attrs);
        let pat_attr = find_leaf_attr(&fields[1].attrs);
        let text_params = leaf_params(text_attr);
        let pat_params = leaf_params(pat_attr);
        prop_assert_eq!(text_params[0].path.to_string(), "text");
        prop_assert_eq!(pat_params[0].path.to_string(), "pattern");
    }
}

// ── 39. Leaf text on unit enum variant (string literal pattern) ─────────────

proptest! {
    #[test]
    fn leaf_text_string_literal_on_variant(idx in 0usize..=4) {
        let texts = ["true", "false", "null", "nil", "void"];
        let txt = texts[idx];
        let name = syn::Ident::new(&format!("V{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #txt)]
                #name
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params[0].path.to_string(), "text");
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &params[0].expr {
            prop_assert_eq!(s.value(), txt);
        } else {
            prop_assert!(false, "Expected string literal for text param");
        }
    }
}

// ── 40. Pattern with prec_right on enum ─────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_prec_right(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_right(#lit)]
                Cons(
                    Box<Expr>,
                    #[adze::leaf(text = "::")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        prop_assert!(e.variants[1].attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(extract_pattern_value(find_leaf_attr(&u.unnamed[0].attrs)), r"\d+");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 41. Pattern with prec (no associativity) ────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_prec_no_assoc(prec in 1i32..=5) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num(
                    #[adze::leaf(pattern = r"-?\d+")]
                    String
                ),
                #[adze::prec(#lit)]
                Cmp(
                    Box<Expr>,
                    #[adze::leaf(text = "==")]
                    (),
                    Box<Expr>
                )
            }
        }).unwrap();
        prop_assert!(e.variants[1].attrs.iter().any(|a| is_adze_attr(a, "prec")));
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(extract_pattern_value(find_leaf_attr(&u.unnamed[0].attrs)), r"-?\d+");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 42. Pattern on multiple enum variants with distinct regexes ─────────────

proptest! {
    #[test]
    fn distinct_regex_per_variant(count in 2usize..=4) {
        let regexes = [r"\d+", r"[a-zA-Z_]\w*", r"0x[0-9a-fA-F]+", r#""[^"]*""#];
        let tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Tok{i}"), proc_macro2::Span::call_site());
                let pat = regexes[i];
                quote::quote! {
                    #name(
                        #[adze::leaf(pattern = #pat)]
                        String
                    )
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Lexer { #(#tokens),* }
        }).unwrap();
        let mut seen = std::collections::HashSet::new();
        for i in 0..count {
            if let Fields::Unnamed(ref u) = e.variants[i].fields {
                let val = extract_pattern_value(find_leaf_attr(&u.unnamed[0].attrs));
                prop_assert!(seen.insert(val.clone()), "Duplicate pattern: {val}");
            }
        }
    }
}

// ── 43. Leaf pattern with unicode escape sequences ──────────────────────────

proptest! {
    #[test]
    fn pattern_with_unicode_escapes(idx in 0usize..=3) {
        let patterns = [r"[\x00-\x7f]", r"[\u0080-\u00ff]", r"[\p{L}]+", r"[\p{N}]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 44. Pattern roundtrip determinism via double-parse ──────────────────────

proptest! {
    #[test]
    fn pattern_double_parse_determinism(idx in 0usize..=4) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"//[^\n]*", r"\d+\.\d+"];
        let pat = patterns[idx];
        let parse_once = || -> String {
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct S {
                    #[adze::leaf(pattern = #pat)]
                    val: String,
                }
            }).unwrap();
            s.to_token_stream().to_string()
        };
        let first = parse_once();
        let second = parse_once();
        prop_assert_eq!(first, second, "Determinism violated");
    }
}

// ── 45. Leaf pattern combined with delimited repeat ─────────────────────────

proptest! {
    #[test]
    fn pattern_with_delimited_repeat(idx in 0usize..=2) {
        let delimiters = [",", ";", "|"];
        let delim = delimiters[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                List(
                    #[adze::leaf(text = "(")]
                    (),
                    #[adze::delimited(
                        #[adze::leaf(text = #delim)]
                        ()
                    )]
                    Vec<Item>,
                    #[adze::leaf(text = ")")]
                    ()
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let open_params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
            prop_assert_eq!(open_params[0].path.to_string(), "text");
            let close_params = leaf_params(find_leaf_attr(&u.unnamed[2].attrs));
            prop_assert_eq!(close_params[0].path.to_string(), "text");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 46. Transform returning different numeric types ─────────────────────────

proptest! {
    #[test]
    fn transform_with_various_return_types(idx in 0usize..=4) {
        let types_and_transforms: Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = vec![
            (quote::quote! { i32 }, quote::quote! { |v| v.parse::<i32>().unwrap() }),
            (quote::quote! { u64 }, quote::quote! { |v| v.parse::<u64>().unwrap() }),
            (quote::quote! { f64 }, quote::quote! { |v| v.parse::<f64>().unwrap() }),
            (quote::quote! { usize }, quote::quote! { |v| v.parse::<usize>().unwrap() }),
            (quote::quote! { i8 }, quote::quote! { |v| v.parse::<i8>().unwrap() }),
        ];
        let (ty, tr) = &types_and_transforms[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"\d+", transform = #tr)]
                val: #ty,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(extract_pattern_value(attr), r"\d+");
    }
}

// ── 47. Leaf in struct with multiple non-leaf fields referenced ─────────────

proptest! {
    #[test]
    fn pattern_coexists_with_non_leaf_fields(extra_count in 1usize..=3) {
        let extra_fields: Vec<proc_macro2::TokenStream> = (0..extra_count)
            .map(|i| {
                let name = syn::Ident::new(&format!("child{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: Box<Other> }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #(#extra_fields),*
            }
        }).unwrap();
        let leaf_count = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        prop_assert_eq!(leaf_count, 1);
        prop_assert_eq!(s.fields.len(), 1 + extra_count);
    }
}

// ── 48. Leaf pattern with empty string (edge case) ──────────────────────────

#[test]
fn pattern_empty_string_preserved() {
    let pat = "";
    let s: ItemStruct = syn::parse2(quote::quote! {
        pub struct S {
            #[adze::leaf(pattern = #pat)]
            val: String,
        }
    })
    .unwrap();
    let field = s.fields.iter().next().unwrap();
    let attr = find_leaf_attr(&field.attrs);
    assert_eq!(extract_pattern_value(attr), "");
}

// ── 49. Leaf pattern and text are mutually exclusive param names ─────────────

proptest! {
    #[test]
    fn pattern_and_text_separate_params(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let nv_pat: NameValueExpr = syn::parse2(quote::quote! { pattern = #pat }).unwrap();
        prop_assert_eq!(nv_pat.path.to_string(), "pattern");

        let texts = ["hello", "world", "foo"];
        let txt = texts[idx];
        let nv_txt: NameValueExpr = syn::parse2(quote::quote! { text = #txt }).unwrap();
        prop_assert_eq!(nv_txt.path.to_string(), "text");

        prop_assert_ne!(nv_pat.path.to_string(), nv_txt.path.to_string());
    }
}

// ── 50. Leaf with transform on enum named variant field ─────────────────────

proptest! {
    #[test]
    fn transform_on_named_enum_field(idx in 0usize..=2) {
        let patterns = [r"\d+", r"-?\d+", r"[01]+"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Num {
                    #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                    value: i32,
                }
            }
        }).unwrap();
        if let Fields::Named(ref n) = e.variants[0].fields {
            let attr = find_leaf_attr(&n.named[0].attrs);
            let params = leaf_params(attr);
            prop_assert_eq!(params.len(), 2);
            prop_assert_eq!(params[1].path.to_string(), "transform");
            prop_assert_eq!(extract_pattern_value(attr), pat);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }
}

// ── 51. Multiple text leaf variants (keyword enum validation) ───────────────

proptest! {
    #[test]
    fn multiple_text_leaf_variants(count in 2usize..=6) {
        let keywords = ["if", "else", "while", "for", "return", "break"];
        let tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("Kw{i}"), proc_macro2::Span::call_site());
                let kw = keywords[i];
                quote::quote! {
                    #[adze::leaf(text = #kw)]
                    #name
                }
            })
            .collect();
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Keywords { #(#tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), count);
        for i in 0..count {
            let attr = find_leaf_attr(&e.variants[i].attrs);
            let params = leaf_params(attr);
            prop_assert_eq!(params[0].path.to_string(), "text");
        }
    }
}

// ── 52. Pattern field ordering determinism in struct ─────────────────────────

proptest! {
    #[test]
    fn field_order_determinism(count in 2usize..=4) {
        let regexes = [r"\d+", r"\w+", r"[a-z]+", r"\S+"];
        let mk = || -> Vec<String> {
            let fields: Vec<proc_macro2::TokenStream> = (0..count)
                .map(|i| {
                    let name = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                    let pat = regexes[i];
                    quote::quote! {
                        #[adze::leaf(pattern = #pat)]
                        #name: String
                    }
                })
                .collect();
            let s: ItemStruct = syn::parse2(quote::quote! {
                pub struct S { #(#fields),* }
            }).unwrap();
            s.fields.iter()
                .map(|f| extract_pattern_value(find_leaf_attr(&f.attrs)))
                .collect()
        };
        let first = mk();
        let second = mk();
        prop_assert_eq!(first, second);
    }
}

// ── 53. Pattern with nested groups ──────────────────────────────────────────

proptest! {
    #[test]
    fn pattern_with_nested_groups(idx in 0usize..=3) {
        let patterns = [
            r"((a|b)+)",
            r"((\d+)(\.\d+)?)",
            r"([a-z]([a-z0-9]*))",
            r"((true|false)|(yes|no))",
        ];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 54. Leaf validation: param count is exactly 1 without transform ─────────

proptest! {
    #[test]
    fn leaf_pattern_exactly_one_param(idx in 0usize..=5) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\s", r".*", r"[^\n]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat)]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let params = leaf_params(find_leaf_attr(&field.attrs));
        prop_assert_eq!(params.len(), 1, "Expected exactly 1 param, got {}", params.len());
    }
}

// ── 55. Leaf validation: param count is exactly 2 with transform ────────────

proptest! {
    #[test]
    fn leaf_pattern_exactly_two_params_with_transform(idx in 0usize..=3) {
        let patterns = [r"\d+", r"-?\d+", r"\d+\.\d+", r"0[xX][0-9a-fA-F]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.to_string())]
                val: String,
            }
        }).unwrap();
        let field = s.fields.iter().next().unwrap();
        let params = leaf_params(find_leaf_attr(&field.attrs));
        prop_assert_eq!(params.len(), 2, "Expected exactly 2 params, got {}", params.len());
    }
}

// ── 56. Leaf text on struct (unit struct leaf) ──────────────────────────────

#[test]
fn leaf_text_on_unit_struct() {
    let s: ItemStruct = syn::parse2(quote::quote! {
        #[adze::leaf(text = "9")]
        pub struct BigDigit;
    })
    .unwrap();
    let attr = find_leaf_attr(&s.attrs);
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "9");
    } else {
        panic!("Expected string literal for text param");
    }
}

// ── 57. Pattern on Box<T> wrapped field in enum ─────────────────────────────

proptest! {
    #[test]
    fn pattern_alongside_boxed_fields(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Expr {
                Unary(
                    #[adze::leaf(pattern = #pat)]
                    String,
                    Box<Expr>
                )
            }
        }).unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), 2);
            let attr = find_leaf_attr(&u.unnamed[0].attrs);
            prop_assert_eq!(extract_pattern_value(attr), pat);
            // Second field has no leaf attr
            prop_assert!(!u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 58. Expansion determinism: enum with mixed leaf types ───────────────────

proptest! {
    #[test]
    fn expansion_determinism_mixed_enum(idx in 0usize..=2) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+"];
        let pat = patterns[idx];
        let mk = || -> String {
            let e: ItemEnum = syn::parse2(quote::quote! {
                pub enum E {
                    #[adze::leaf(text = "k")]
                    Keyword,
                    Pat(
                        #[adze::leaf(pattern = #pat, transform = |v| v.to_string())]
                        String
                    )
                }
            }).unwrap();
            e.to_token_stream().to_string()
        };
        prop_assert_eq!(mk(), mk(), "Expansion not deterministic");
    }
}

// ── 59. Leaf text with special characters ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_special_chars(idx in 0usize..=5) {
        let texts = ["->", "=>", "::", "&&", "||", "!="];
        let txt = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #txt)]
                Op
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &params[0].expr {
            prop_assert_eq!(s.value(), txt);
        } else {
            prop_assert!(false, "Expected string literal");
        }
    }
}

// ── 60. Leaf with repeat non_empty on containing field ──────────────────────

proptest! {
    #[test]
    fn pattern_with_repeat_non_empty(idx in 0usize..=2) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
                #[adze::leaf(pattern = #pat)]
                separator: String,
            }
        }).unwrap();
        let fields: Vec<_> = s.fields.iter().collect();
        prop_assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        let attr = find_leaf_attr(&fields[1].attrs);
        prop_assert_eq!(extract_pattern_value(attr), pat);
    }
}

// ── 61. Pattern extraction from NameValueExpr directly ──────────────────────

proptest! {
    #[test]
    fn name_value_expr_pattern_roundtrip(idx in 0usize..=4) {
        let patterns = [r"\d+", r"\w+", r"[a-z]+", r"\s+", r"[^\n]+"];
        let pat = patterns[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { pattern = #pat }).unwrap();
        prop_assert_eq!(nv.path.to_string(), "pattern");
        // Roundtrip: parse again with same input
        let reparsed_nv: NameValueExpr = syn::parse2(quote::quote! { pattern = #pat }).unwrap();
        prop_assert_eq!(nv.path.to_string(), reparsed_nv.path.to_string());
        // Value matches
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s1), .. }) = &nv.expr {
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s2), .. }) = &reparsed_nv.expr {
                prop_assert_eq!(s1.value(), s2.value());
            }
        }
    }
}

// ── 62. Enum with both text and pattern variants count preserved ────────────

proptest! {
    #[test]
    fn mixed_variant_count_preserved(n_text in 1usize..=3, n_pat in 1usize..=3) {
        let texts: Vec<&str> = vec!["a", "b", "c"];
        let pats: Vec<&str> = vec![r"\d+", r"\w+", r"[a-z]+"];
        let mut variant_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_text {
            let name = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
            let txt = texts[i];
            variant_tokens.push(quote::quote! {
                #[adze::leaf(text = #txt)]
                #name
            });
        }
        for i in 0..n_pat {
            let name = syn::Ident::new(&format!("P{i}"), proc_macro2::Span::call_site());
            let pat = pats[i];
            variant_tokens.push(quote::quote! {
                #name(
                    #[adze::leaf(pattern = #pat)]
                    String
                )
            });
        }
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E { #(#variant_tokens),* }
        }).unwrap();
        prop_assert_eq!(e.variants.len(), n_text + n_pat);
    }
}

// ── 63. Leaf text value is always a string literal in NameValueExpr ──────────

proptest! {
    #[test]
    fn text_value_is_str_lit(idx in 0usize..=4) {
        let texts = ["foo", "bar", "+", "->", "::"];
        let txt = texts[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { text = #txt }).unwrap();
        prop_assert_eq!(nv.path.to_string(), "text");
        let is_str = matches!(
            nv.expr,
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(_), .. })
        );
        prop_assert!(is_str, "text value should be a string literal");
    }
}

// ── 64. Pattern attr not found on non-leaf fields ───────────────────────────

proptest! {
    #[test]
    fn non_leaf_field_has_no_leaf_attr(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("child{i}"), proc_macro2::Span::call_site());
                quote::quote! { #name: Box<Other> }
            })
            .collect();
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S { #(#fields),* }
        }).unwrap();
        for field in s.fields.iter() {
            prop_assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 65. Leaf transform expression preserved in token stream ─────────────────

proptest! {
    #[test]
    fn transform_expr_preserved(idx in 0usize..=2) {
        let patterns = [r"\d+", r"-?\d+", r"\d+\.\d+"];
        let pat = patterns[idx];
        let s: ItemStruct = syn::parse2(quote::quote! {
            pub struct S {
                #[adze::leaf(pattern = #pat, transform = |v| v.parse().unwrap())]
                val: i32,
            }
        }).unwrap();
        // Roundtrip through token stream
        let ts = s.to_token_stream().to_string();
        let s2: ItemStruct = syn::parse_str(&ts).unwrap();
        let field = s2.fields.iter().next().unwrap();
        let attr = find_leaf_attr(&field.attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(params[1].path.to_string(), "transform");
        // The transform is a closure expression
        prop_assert!(matches!(params[1].expr, syn::Expr::Closure(_)));
    }
}
