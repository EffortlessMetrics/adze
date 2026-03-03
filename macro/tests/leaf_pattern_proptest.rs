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
