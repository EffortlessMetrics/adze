#![allow(clippy::needless_range_loop)]

//! Property-based tests for `#[adze::leaf(text = "...")]` in adze-macro.
//!
//! Uses proptest to generate randomized text values, field counts, and
//! annotation combinations, then verifies that syn correctly parses and
//! preserves the leaf text attributes (which produce STRING rules).

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

fn extract_text_value(attr: &Attribute) -> String {
    let params = leaf_params(attr);
    let nv = params.iter().find(|p| p.path.to_string() == "text").unwrap();
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

// ── 1. Leaf text detection on unit variant field ────────────────────────────

proptest! {
    #[test]
    fn leaf_text_detected_on_unit_variant(idx in 0usize..=4) {
        let keywords = ["if", "else", "while", "for", "return"];
        let kw = keywords[idx];
        let name = syn::Ident::new(&format!("K{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Kw {
                #[adze::leaf(text = #kw)]
                #name
            }
        }).unwrap();
        prop_assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 2. Text value extraction from unit variant ──────────────────────────────

proptest! {
    #[test]
    fn text_value_extracted_from_unit_variant(idx in 0usize..=5) {
        let operators = ["+", "-", "*", "/", "==", "!="];
        let op = operators[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #op)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let value = extract_text_value(attr);
        prop_assert_eq!(value, op);
    }
}

// ── 3. Keyword text patterns ────────────────────────────────────────────────

proptest! {
    #[test]
    fn keyword_text_patterns(idx in 0usize..=7) {
        let keywords = ["let", "fn", "struct", "enum", "impl", "pub", "mod", "use"];
        let kw = keywords[idx];
        let name = syn::Ident::new(&format!("K{idx}"), proc_macro2::Span::call_site());
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Kw {
                #[adze::leaf(text = #kw)]
                #name
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let params = leaf_params(attr);
        prop_assert_eq!(params[0].path.to_string(), "text");
        prop_assert_eq!(extract_text_value(attr), kw);
    }
}

// ── 4. Operator text patterns ───────────────────────────────────────────────

proptest! {
    #[test]
    fn operator_text_patterns(idx in 0usize..=9) {
        let ops = ["+", "-", "*", "/", "%", "&&", "||", "<<", ">>", "^"];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #op)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), op);
    }
}

// ── 5. Symbol text patterns (brackets, parens, etc.) ────────────────────────

proptest! {
    #[test]
    fn symbol_text_patterns(idx in 0usize..=7) {
        let symbols = ["(", ")", "{", "}", "[", "]", ";", ","];
        let sym = symbols[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Sym {
                #[adze::leaf(text = #sym)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), sym);
    }
}

// ── 6. Multiple leaf text fields in same struct ─────────────────────────────

proptest! {
    #[test]
    fn multiple_leaf_text_fields_in_struct(count in 2usize..=5) {
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
        let leaf_fields: Vec<_> = s.fields.iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .collect();
        prop_assert_eq!(leaf_fields.len(), count);
        for (i, f) in leaf_fields.iter().enumerate() {
            let attr = find_leaf_attr(&f.attrs);
            let expected = format!("t{i}");
            prop_assert_eq!(extract_text_value(attr), expected);
        }
    }
}

// ── 7. Multiple leaf text unit variants in same enum ────────────────────────

proptest! {
    #[test]
    fn multiple_leaf_text_unit_variants(count in 2usize..=8) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("op{i}");
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
        for (i, v) in e.variants.iter().enumerate() {
            let attr = find_leaf_attr(&v.attrs);
            let expected = format!("op{i}");
            prop_assert_eq!(extract_text_value(attr), expected);
        }
    }
}

// ── 8. Leaf text with special characters (escapes) ──────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_special_chars(idx in 0usize..=4) {
        let specials = ["\\n", "\\t", "\\\\", "\\\"", "\\0"];
        let spec = specials[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #spec)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), spec);
    }
}

// ── 9. Leaf text with Unicode strings ───────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_unicode(idx in 0usize..=5) {
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

// ── 10. Leaf text combined with prec_left ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_prec_left(prec in 1i32..=10) {
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
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let op_field = &u.unnamed[1];
            prop_assert!(op_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            let attr = find_leaf_attr(&op_field.attrs);
            prop_assert_eq!(extract_text_value(attr), "+");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 11. Leaf text combined with prec_right ──────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_prec_right(prec in 1i32..=10) {
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
            let op_field = &u.unnamed[1];
            let attr = find_leaf_attr(&op_field.attrs);
            prop_assert_eq!(extract_text_value(attr), "::");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 12. Leaf text on named variant field ────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_on_named_variant_field(idx in 0usize..=3) {
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

// ── 13. Leaf text param is always "text" key ────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_param_key_is_text(idx in 0usize..=6) {
        let texts = ["a", "bc", "def", ".", "->", "::", "=>"];
        let t = texts[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { text = #t }).unwrap();
        prop_assert_eq!(nv.path.to_string(), "text");
    }
}

// ── 14. Leaf text value is always a string literal ──────────────────────────

proptest! {
    #[test]
    fn leaf_text_value_is_str_lit(idx in 0usize..=5) {
        let texts = ["+", "let", "(", "->", "!=", "..."];
        let t = texts[idx];
        let nv: NameValueExpr = syn::parse2(quote::quote! { text = #t }).unwrap();
        let is_str = matches!(
            nv.expr,
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(_), .. })
        );
        prop_assert!(is_str);
    }
}

// ── 15. Multi-character operator text values ────────────────────────────────

proptest! {
    #[test]
    fn multi_char_operator_text(idx in 0usize..=7) {
        let ops = ["===", "!==", "<<=", ">>=", "&&=", "||=", "**=", "..="];
        let op = ops[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #op)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let value = extract_text_value(attr);
        prop_assert!(value.len() >= 3);
        prop_assert_eq!(value, op);
    }
}

// ── 16. Single-character text values ────────────────────────────────────────

proptest! {
    #[test]
    fn single_char_text_value(idx in 0usize..=9) {
        let chars = ["+", "-", "*", "/", "%", "^", "&", "|", "~", "!"];
        let ch = chars[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum Op {
                #[adze::leaf(text = #ch)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        let value = extract_text_value(attr);
        prop_assert_eq!(value.len(), 1);
        prop_assert_eq!(value, ch);
    }
}

// ── 17. Leaf text on struct field (unnamed tuple struct) ────────────────────

proptest! {
    #[test]
    fn leaf_text_on_tuple_struct_field(idx in 0usize..=3) {
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

// ── 18. Leaf text mixed with leaf pattern in same enum ──────────────────────

proptest! {
    #[test]
    fn leaf_text_mixed_with_pattern(n_text in 1usize..=3, n_pattern in 1usize..=3) {
        let tnames: Vec<syn::Ident> = (0..n_text)
            .map(|i| syn::Ident::new(&format!("Txt{i}"), proc_macro2::Span::call_site()))
            .collect();
        let pnames: Vec<syn::Ident> = (0..n_pattern)
            .map(|i| syn::Ident::new(&format!("Pat{i}"), proc_macro2::Span::call_site()))
            .collect();
        let text_vals: Vec<String> = (0..n_text).map(|i| format!("kw{i}")).collect();

        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_text {
            let name = &tnames[i];
            let tv = &text_vals[i];
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
        // Text variants are unit with leaf attr
        for i in 0..n_text {
            prop_assert!(matches!(e.variants[i].fields, Fields::Unit));
            let params = leaf_params(find_leaf_attr(&e.variants[i].attrs));
            prop_assert_eq!(params[0].path.to_string(), "text");
        }
        // Pattern variants are unnamed with leaf attr on field
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

// ── 19. Leaf text combined with skip field in struct ────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_skip_in_struct(idx in 0usize..=2) {
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
        prop_assert!(!fields[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 20. Leaf text value roundtrip through token stream ──────────────────────

proptest! {
    #[test]
    fn leaf_text_roundtrip_token_stream(idx in 0usize..=5) {
        let texts = ["hello", "world", "foo_bar", "123", "a b c", "x+y"];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        // Roundtrip: parse -> to_token_stream -> parse again
        let token_str = e.to_token_stream().to_string();
        let e2: ItemEnum = syn::parse_str(&token_str).unwrap();
        let attr2 = find_leaf_attr(&e2.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr2), t);
    }
}

// ── 21. Leaf text with empty string ─────────────────────────────────────────

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

// ── 22. Leaf text with CJK Unicode ──────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_cjk_unicode(idx in 0usize..=3) {
        let cjk = ["中", "日", "韓", "漢"];
        let c = cjk[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #c)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), c);
    }
}

// ── 23. Leaf text with emoji ────────────────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_emoji(idx in 0usize..=3) {
        let emojis = ["🔥", "⚡", "🎉", "✅"];
        let em = emojis[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #em)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), em);
    }
}

// ── 24. Leaf text param count is exactly 1 (text-only) ──────────────────────

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
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 1);
    }
}

// ── 25. Leaf text combined with transform has 2 params ──────────────────────

proptest! {
    #[test]
    fn leaf_text_with_transform_has_two_params(idx in 0usize..=2) {
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
        let params = leaf_params(attr);
        prop_assert_eq!(params.len(), 2);
        prop_assert_eq!(params[0].path.to_string(), "text");
        prop_assert_eq!(params[1].path.to_string(), "transform");
    }
}

// ── 26. Leaf text on each field in binary expression variant ────────────────

proptest! {
    #[test]
    fn leaf_text_binary_expr_pattern(idx in 0usize..=4) {
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
            // Middle field has leaf text
            let mid = &u.unnamed[1];
            prop_assert!(mid.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            prop_assert_eq!(extract_text_value(find_leaf_attr(&mid.attrs)), op);
            // First and last don't have leaf
            prop_assert!(!u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            prop_assert!(!u.unnamed[2].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 27. Leaf text with whitespace-containing strings ────────────────────────

proptest! {
    #[test]
    fn leaf_text_with_spaces(idx in 0usize..=3) {
        let texts = ["a b", "hello world", " ", "  x  "];
        let t = texts[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::leaf(text = #t)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), t);
    }
}

// ── 28. Leaf text combined with language attr on enum ───────────────────────

proptest! {
    #[test]
    fn leaf_text_with_language_attr(count in 1usize..=4) {
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
        // Enum-level language attr preserved
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        // Variant-level leaf attrs preserved
        for v in &e.variants {
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 29. Leaf text values are distinct across variants ───────────────────────

proptest! {
    #[test]
    fn leaf_text_values_distinct(count in 2usize..=6) {
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
        // All values should be unique
        let unique: std::collections::HashSet<_> = values.iter().collect();
        prop_assert_eq!(unique.len(), count);
    }
}

// ── 30. Leaf text on struct field preserves field name ───────────────────────

proptest! {
    #[test]
    fn leaf_text_preserves_field_name(idx in 0usize..=3) {
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

// ── 31. Leaf text field type is unit () ─────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_field_type_is_unit(idx in 0usize..=4) {
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

// ── 32. Leaf text in delimited attribute ────────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_in_delimited_attr(idx in 0usize..=2) {
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

// ── 33. Leaf text with mathematical symbols ─────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_math_symbols(idx in 0usize..=5) {
        let syms = ["≤", "≥", "≠", "±", "÷", "×"];
        let sym = syms[idx];
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum MathOp {
                #[adze::leaf(text = #sym)]
                V
            }
        }).unwrap();
        let attr = find_leaf_attr(&e.variants[0].attrs);
        prop_assert_eq!(extract_text_value(attr), sym);
    }
}

// ── 34. Leaf text preserves ordering among variant attrs ────────────────────

proptest! {
    #[test]
    fn leaf_text_attr_ordering_preserved(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let e: ItemEnum = syn::parse2(quote::quote! {
            pub enum E {
                #[adze::prec_left(#lit)]
                #[adze::leaf(text = "+")]
                V
            }
        }).unwrap();
        let attrs = &e.variants[0].attrs;
        let names: Vec<String> = attrs.iter()
            .filter_map(|a| {
                let segs: Vec<_> = a.path().segments.iter().collect();
                if segs.len() == 2 && segs[0].ident == "adze" {
                    Some(segs[1].ident.to_string())
                } else {
                    None
                }
            })
            .collect();
        prop_assert_eq!(names, vec!["prec_left", "leaf"]);
    }
}

// ── 35. Leaf text with long string values ───────────────────────────────────

proptest! {
    #[test]
    fn leaf_text_long_strings(repeat in 1usize..=10) {
        let text = "ab".repeat(repeat);
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
