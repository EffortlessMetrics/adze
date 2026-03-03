#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for leaf extraction and transform logic in the adze macro crate.
//!
//! Tests cover `#[adze::leaf]` attribute parsing (text, pattern, transform parameters),
//! type wrapping via `wrap_leaf_type`, type extraction via `try_extract_inner_type` and
//! `filter_inner_type`, `NameValueExpr`/`FieldThenParams` parsing in leaf-centric
//! contexts, and structural validation of leaf-annotated AST items.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemStruct, Token, Type, parse_quote};

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

/// The standard skip set used by `gen_field` in the expansion code.
fn expansion_skip_set() -> HashSet<&'static str> {
    ["Spanned", "Box", "Option", "Vec"].into_iter().collect()
}

// ── 1. Parse leaf text-only attribute ───────────────────────────────────────

#[test]
fn leaf_text_only_param() {
    let nv: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "+");
    } else {
        panic!("Expected string literal for text param");
    }
}

// ── 2. Parse leaf pattern-only attribute ────────────────────────────────────

#[test]
fn leaf_pattern_only_param() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), r"\d+");
    } else {
        panic!("Expected string literal for pattern param");
    }
}

// ── 3. Parse leaf with pattern + transform ──────────────────────────────────

#[test]
fn leaf_pattern_and_transform_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params[1].path.to_string(), "transform");
    assert!(matches!(params[1].expr, syn::Expr::Closure(_)));
}

// ── 4. Parse leaf with text + transform ─────────────────────────────────────

#[test]
fn leaf_text_and_transform_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(text = "true", transform = |_v| true)]
            value: bool,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "text");
    assert_eq!(params[1].path.to_string(), "transform");
}

// ── 5. Transform closure with explicit type annotation ──────────────────────

#[test]
fn leaf_transform_closure_with_type_annotation() {
    let nv: NameValueExpr = parse_quote!(transform = |v: &str| v.parse::<i32>().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
    if let syn::Expr::Closure(c) = &nv.expr {
        assert_eq!(c.inputs.len(), 1);
    } else {
        panic!("Expected closure expression");
    }
}

// ── 6. Transform closure with turbofish in body ─────────────────────────────

#[test]
fn leaf_transform_closure_with_turbofish() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse::<f64>().unwrap());
    if let syn::Expr::Closure(c) = &nv.expr {
        // Closure body should contain a turbofish method call
        assert!(c.body.to_token_stream().to_string().contains("f64"));
    } else {
        panic!("Expected closure expression");
    }
}

// ── 7. Transform closure returning complex expression ───────────────────────

#[test]
fn leaf_transform_closure_complex_body() {
    let nv: NameValueExpr = parse_quote!(
        transform = |v| {
            let n: u32 = v.parse().unwrap();
            n * 2
        }
    );
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

// ── 8. wrap_leaf_type wraps primitive i32 ───────────────────────────────────

#[test]
fn wrap_leaf_type_primitive_i32() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );
}

// ── 9. wrap_leaf_type wraps String ──────────────────────────────────────────

#[test]
fn wrap_leaf_type_string() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

// ── 10. wrap_leaf_type skips Option, wraps inner ────────────────────────────

#[test]
fn wrap_leaf_type_option_skips_wraps_inner() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

// ── 11. wrap_leaf_type skips Vec, wraps inner ───────────────────────────────

#[test]
fn wrap_leaf_type_vec_skips_wraps_inner() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

// ── 12. wrap_leaf_type skips Box, wraps inner ───────────────────────────────

#[test]
fn wrap_leaf_type_box_skips_wraps_inner() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<u64>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Box < adze :: WithLeaf < u64 > >"
    );
}

// ── 13. wrap_leaf_type nested Option<Vec<T>> ────────────────────────────────

#[test]
fn wrap_leaf_type_option_vec_nested() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Option<Vec<f32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < f32 > > >"
    );
}

// ── 14. wrap_leaf_type skips Spanned, wraps inner ───────────────────────────

#[test]
fn wrap_leaf_type_spanned_skips_wraps_inner() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Spanned<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Spanned < adze :: WithLeaf < bool > >"
    );
}

// ── 15. wrap_leaf_type wraps unit type () ────────────────────────────────────

#[test]
fn wrap_leaf_type_unit_type() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip);
    // () is not a Type::Path, so it gets wrapped entirely
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < () >"
    );
}

// ── 16. filter_inner_type strips Box from leaf type ─────────────────────────

#[test]
fn filter_inner_type_strips_box() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "i32");
}

// ── 17. filter_inner_type strips nested Box<Spanned<T>> ─────────────────────

#[test]
fn filter_inner_type_strips_nested_containers() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<Spanned<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

// ── 18. filter_inner_type leaves non-skip type unchanged ────────────────────

#[test]
fn filter_inner_type_non_skip_unchanged() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        "HashMap < String , i32 >"
    );
}

// ── 19. try_extract_inner_type extracts from Vec<i32> ───────────────────────

#[test]
fn try_extract_vec_i32() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 20. try_extract_inner_type extracts from Option<String> ─────────────────

#[test]
fn try_extract_option_string() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ── 21. try_extract_inner_type skips Box to reach Vec ───────────────────────

#[test]
fn try_extract_skips_box_to_vec() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u8");
}

// ── 22. try_extract_inner_type does not extract when target absent ───────────

#[test]
fn try_extract_no_match_returns_original() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ── 23. Leaf params: transform before pattern in source order ───────────────

#[test]
fn leaf_params_transform_before_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Rev {
            #[adze::leaf(transform = |v| v.to_uppercase(), pattern = r"[a-z]+")]
            name: String,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
    // Order in source is preserved
    assert_eq!(params[0].path.to_string(), "transform");
    assert_eq!(params[1].path.to_string(), "pattern");
}

// ── 24. Multiple leaf fields in struct: count and identify ──────────────────

#[test]
fn struct_multiple_leaf_fields_identified() {
    let s: ItemStruct = parse_quote! {
        pub struct BinOp {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            lhs: i32,
            #[adze::leaf(text = "+")]
            _op: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            rhs: i32,
        }
    };
    let leaf_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .collect();
    assert_eq!(leaf_fields.len(), 3);

    // Verify first is pattern, second is text, third is pattern
    let first_params = leaf_params(find_leaf_attr(&leaf_fields[0].attrs));
    assert_eq!(first_params[0].path.to_string(), "pattern");

    let second_params = leaf_params(find_leaf_attr(&leaf_fields[1].attrs));
    assert_eq!(second_params[0].path.to_string(), "text");

    let third_params = leaf_params(find_leaf_attr(&leaf_fields[2].attrs));
    assert_eq!(third_params[0].path.to_string(), "pattern");
}

// ── 25. Leaf on enum unit variant detected ──────────────────────────────────

#[test]
fn leaf_on_enum_unit_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
            #[adze::leaf(text = "*")]
            Star,
        }
    };
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
        assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
    // Verify the text values
    let texts: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let params = leaf_params(find_leaf_attr(&v.attrs));
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &params[0].expr
            {
                s.value()
            } else {
                panic!("Expected string literal");
            }
        })
        .collect();
    assert_eq!(texts, vec!["+", "-", "*"]);
}

// ── 26. Leaf on enum tuple variant with single field ────────────────────────

#[test]
fn leaf_on_enum_tuple_variant_single_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                u32
            ),
        }
    };
    let variant = &e.variants[0];
    if let Fields::Unnamed(ref unnamed) = variant.fields {
        assert_eq!(unnamed.unnamed.len(), 1);
        let field = &unnamed.unnamed[0];
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        let params = leaf_params(find_leaf_attr(&field.attrs));
        assert_eq!(params.len(), 2);
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 27. Leaf on enum named variant field ────────────────────────────────────

#[test]
fn leaf_on_enum_named_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Stmt {
            Assign {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
                #[adze::leaf(text = "=")]
                _eq: (),
            },
        }
    };
    let variant = &e.variants[0];
    if let Fields::Named(ref named) = variant.fields {
        assert_eq!(named.named.len(), 2);
        for field in &named.named {
            assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    } else {
        panic!("Expected named fields");
    }
}

// ── 28. Leaf adjacent to skip field ─────────────────────────────────────────

#[test]
fn leaf_adjacent_to_skip_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Tagged {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(0)]
            index: usize,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(fields.len(), 2);
    assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
    assert!(!fields[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 29. Mixed leaf text and pattern in same enum ────────────────────────────

#[test]
fn mixed_leaf_text_and_pattern_in_enum() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "let")]
            Let,
            Ident(
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                String
            ),
            #[adze::leaf(text = "=")]
            Eq,
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i64
            ),
        }
    };
    // Unit variants have leaf text
    let let_params = leaf_params(find_leaf_attr(&e.variants[0].attrs));
    assert_eq!(let_params[0].path.to_string(), "text");

    // Tuple variant has leaf pattern
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let ident_params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
        assert_eq!(ident_params[0].path.to_string(), "pattern");
    } else {
        panic!("Expected unnamed fields");
    }

    // Another unit variant
    let eq_params = leaf_params(find_leaf_attr(&e.variants[2].attrs));
    assert_eq!(eq_params[0].path.to_string(), "text");

    // Tuple variant with pattern + transform
    if let Fields::Unnamed(ref u) = e.variants[3].fields {
        let num_params = leaf_params(find_leaf_attr(&u.unnamed[0].attrs));
        assert_eq!(num_params.len(), 2);
        assert_eq!(num_params[0].path.to_string(), "pattern");
        assert_eq!(num_params[1].path.to_string(), "transform");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 30. Delimited with inner leaf parses via FieldThenParams ────────────────

#[test]
fn delimited_inner_leaf_field_then_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Csv {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    // The inner field type is ()
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    // The inner field has a leaf attribute
    let inner_leaf = find_leaf_attr(&ftp.field.attrs);
    let params = leaf_params(inner_leaf);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), ",");
    } else {
        panic!("Expected string literal");
    }
}

// ── 31. wrap_leaf_type: Vec<Option<T>> double-nested skip ───────────────────

#[test]
fn wrap_leaf_type_vec_option_double_skip() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Vec<Option<u16>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < adze :: WithLeaf < u16 > > >"
    );
}

// ── 32. wrap_leaf_type: non-generic path type not in skip gets wrapped ──────

#[test]
fn wrap_leaf_type_custom_type_not_in_skip() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < MyCustomType >"
    );
}

// ── 33. try_extract_inner_type: Spanned<Vec<T>> with Spanned skipped ────────

#[test]
fn try_extract_spanned_vec() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Spanned<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 34. Leaf pattern with empty regex ───────────────────────────────────────

#[test]
fn leaf_pattern_empty_string() {
    let nv: NameValueExpr = parse_quote!(pattern = "");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "");
    } else {
        panic!("Expected string literal");
    }
}

// ── 35. Leaf text with multi-char operator ──────────────────────────────────

#[test]
fn leaf_text_multi_char_operator() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "===")]
            StrictEq,
            #[adze::leaf(text = "!==")]
            StrictNeq,
            #[adze::leaf(text = "<<=")]
            ShlAssign,
        }
    };
    let texts: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let params = leaf_params(find_leaf_attr(&v.attrs));
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &params[0].expr
            {
                s.value()
            } else {
                panic!("Expected string literal");
            }
        })
        .collect();
    assert_eq!(texts, vec!["===", "!==", "<<="]);
}
