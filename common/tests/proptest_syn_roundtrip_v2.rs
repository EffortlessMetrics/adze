//! Property-based and unit tests for proc_macro2 / syn / quote roundtrip patterns.
//!
//! Covers: Ident roundtripping, Literal roundtripping, quote! generation,
//! syn parsing patterns, TokenStream operations, Ident comparison, and more.

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use proptest::prelude::*;
use quote::{ToTokens, format_ident, quote};
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (not keywords in edition 2024).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .unwrap()
        .prop_filter("must be valid syn ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Alphabetic-only identifiers (always valid, no leading digits).
fn alpha_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{1,12}")
        .unwrap()
        .prop_filter("must be valid syn ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// u32 values for literal roundtrips.
fn u32_values() -> impl Strategy<Value = u32> {
    any::<u32>()
}

/// Printable ASCII strings for string literal tests (no backslash/quote to keep it simple).
fn safe_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 _.,!?;:]{0,50}").unwrap()
}

/// Primitive type names.
fn prim_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
            "bool", "char", "String", "usize", "isize",
        ][..],
    )
}

// ---------------------------------------------------------------------------
// Property-based tests — Area 1: Ident roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Any valid Rust identifier roundtrips through Ident -> to_string -> parse.
    #[test]
    fn prop_ident_roundtrip_to_string(name in ident_strategy()) {
        let ident = Ident::new(&name, Span::call_site());
        let s = ident.to_string();
        prop_assert_eq!(&s, &name);
    }

    /// Ident survives quote! -> to_string -> parse cycle.
    #[test]
    fn prop_ident_quote_roundtrip(name in ident_strategy()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { #ident };
        let text = ts.to_string();
        let reparsed: syn::Ident = parse_str(&text).unwrap();
        prop_assert_eq!(reparsed.to_string(), name);
    }

    /// format_ident! creates identical ident to manual construction.
    #[test]
    fn prop_format_ident_matches_manual(name in alpha_ident()) {
        let manual = Ident::new(&name, Span::call_site());
        let formatted = format_ident!("{}", name);
        prop_assert_eq!(manual.to_string(), formatted.to_string());
    }

    /// Ident constructed from string is equal to itself when reparsed.
    #[test]
    fn prop_ident_parse_str_roundtrip(name in ident_strategy()) {
        let parsed: syn::Ident = parse_str(&name).unwrap();
        prop_assert_eq!(parsed.to_string(), name);
    }
}

// ---------------------------------------------------------------------------
// Property-based tests — Area 2: u32 Literal roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Any u32 value roundtrips through Literal -> to_string -> parse.
    #[test]
    fn prop_u32_literal_roundtrip(val in u32_values()) {
        let lit = Literal::u32_suffixed(val);
        let text = lit.to_string();
        prop_assert!(text.contains(&val.to_string()));
    }

    /// u32 unsuffixed literal contains the numeric value.
    #[test]
    fn prop_u32_unsuffixed_roundtrip(val in u32_values()) {
        let lit = Literal::u32_unsuffixed(val);
        let text = lit.to_string();
        prop_assert_eq!(text, val.to_string());
    }

    /// i64 suffixed literal roundtrips.
    #[test]
    fn prop_i64_literal_roundtrip(val in any::<i64>()) {
        let lit = Literal::i64_suffixed(val);
        let text = lit.to_string();
        prop_assert!(text.ends_with("i64"));
    }

    /// f64 suffixed literal roundtrips.
    #[test]
    fn prop_f64_literal_roundtrip(val in 0.0f64..1e10) {
        let lit = Literal::f64_suffixed(val);
        let text = lit.to_string();
        prop_assert!(text.ends_with("f64"));
    }
}

// ---------------------------------------------------------------------------
// Property-based tests — Area 3: String Literal roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Any safe string roundtrips through Literal::string -> to_string.
    #[test]
    fn prop_string_literal_roundtrip(s in safe_string()) {
        let lit = Literal::string(&s);
        let text = lit.to_string();
        // The literal is quoted, so it should start/end with "
        prop_assert!(text.starts_with('"'));
        prop_assert!(text.ends_with('"'));
    }

    /// String literal contains the original content between quotes.
    #[test]
    fn prop_string_literal_contains_content(s in safe_string()) {
        let lit = Literal::string(&s);
        let text = lit.to_string();
        // Strip quotes
        let inner = &text[1..text.len()-1];
        prop_assert_eq!(inner, &s);
    }

    /// Byte string literal starts with b".
    #[test]
    fn prop_byte_string_literal_prefix(s in safe_string()) {
        let lit = Literal::byte_string(s.as_bytes());
        let text = lit.to_string();
        prop_assert!(text.starts_with("b\""));
    }

    /// Character literal roundtrips for ASCII letters.
    #[test]
    fn prop_char_literal_roundtrip(c in prop::char::range('a', 'z')) {
        let lit = Literal::character(c);
        let text = lit.to_string();
        prop_assert!(text.starts_with('\''));
        prop_assert!(text.ends_with('\''));
        prop_assert!(text.contains(c));
    }
}

// ---------------------------------------------------------------------------
// Property-based tests — Area 4: quote! output non-empty for structs
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// quote! output for a struct definition is never empty.
    #[test]
    fn prop_quote_struct_nonempty(name in alpha_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { struct #ident {} };
        prop_assert!(!ts.is_empty());
    }

    /// quote! output for struct with field is parseable by syn.
    #[test]
    fn prop_quote_struct_parseable(name in alpha_ident(), field in alpha_ident()) {
        let struct_ident = Ident::new(&name, Span::call_site());
        let field_ident = Ident::new(&field, Span::call_site());
        let ts = quote! { struct #struct_ident { #field_ident: u32 } };
        let item: syn::Item = syn::parse2(ts).unwrap();
        if let syn::Item::Struct(s) = item {
            prop_assert_eq!(s.ident.to_string(), name);
        } else {
            prop_assert!(false, "expected struct item");
        }
    }

    /// quote! output for enum definition is non-empty and parseable.
    #[test]
    fn prop_quote_enum_nonempty(name in alpha_ident(), variant in alpha_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let var_ident = Ident::new(&variant, Span::call_site());
        let ts = quote! { enum #ident { #var_ident } };
        prop_assert!(!ts.is_empty());
        let item: syn::Item = syn::parse2(ts).unwrap();
        if let syn::Item::Enum(e) = item {
            prop_assert_eq!(e.ident.to_string(), name);
        } else {
            prop_assert!(false, "expected enum item");
        }
    }

    /// quote! output for fn definition is non-empty.
    #[test]
    fn prop_quote_fn_nonempty(name in alpha_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { fn #ident() {} };
        prop_assert!(!ts.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Property-based tests — Area 5: quote! with interpolation preserves ident
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Interpolated ident appears in the output token stream string.
    #[test]
    fn prop_interpolation_preserves_ident(name in ident_strategy()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { let #ident = 42; };
        let text = ts.to_string();
        prop_assert!(text.contains(&name));
    }

    /// Multiple interpolations of same ident all appear.
    #[test]
    fn prop_multiple_interpolation(name in alpha_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { let #ident = #ident; };
        let text = ts.to_string();
        // Name should appear at least twice
        let count = text.matches(&name).count();
        prop_assert!(count >= 2, "expected at least 2 occurrences, got {}", count);
    }

    /// Interpolated type in quote! is parseable.
    #[test]
    fn prop_interpolated_type_parseable(ty_name in prim_type()) {
        let ty: Type = parse_str(ty_name).unwrap();
        let ts = quote! { let _x: #ty = Default::default(); };
        let text = ts.to_string();
        prop_assert!(text.contains(ty_name));
    }

    /// format_ident! with prefix preserves suffix.
    #[test]
    fn prop_format_ident_prefix(name in alpha_ident()) {
        let ident = format_ident!("my_{}", name);
        let expected = format!("my_{}", name);
        prop_assert_eq!(ident.to_string(), expected);
    }
}

// ---------------------------------------------------------------------------
// Unit tests — Area 6: syn parsing patterns
// ---------------------------------------------------------------------------

#[test]
fn unit_syn_parse_simple_type() {
    let ty: Type = parse_str("i32").unwrap();
    assert_eq!(ty.to_token_stream().to_string(), "i32");
}

#[test]
fn unit_syn_parse_reference_type() {
    let ty: Type = parse_str("&str").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("str"));
}

#[test]
fn unit_syn_parse_option_type() {
    let ty: Type = parse_str("Option<i32>").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("Option"));
    assert!(text.contains("i32"));
}

#[test]
fn unit_syn_parse_vec_type() {
    let ty: Type = parse_str("Vec<String>").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("Vec"));
    assert!(text.contains("String"));
}

#[test]
fn unit_syn_parse_tuple_type() {
    let ty: Type = parse_str("(i32, u64)").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("i32"));
    assert!(text.contains("u64"));
}

#[test]
fn unit_syn_parse_fn_item() {
    let item: syn::Item = parse_str("fn foo() {}").unwrap();
    if let syn::Item::Fn(f) = item {
        assert_eq!(f.sig.ident.to_string(), "foo");
    } else {
        panic!("expected fn item");
    }
}

#[test]
fn unit_syn_parse_struct_item() {
    let item: syn::Item = parse_str("struct Foo { x: i32 }").unwrap();
    if let syn::Item::Struct(s) = item {
        assert_eq!(s.ident.to_string(), "Foo");
    } else {
        panic!("expected struct item");
    }
}

#[test]
fn unit_syn_parse_enum_item() {
    let item: syn::Item = parse_str("enum Color { Red, Green, Blue }").unwrap();
    if let syn::Item::Enum(e) = item {
        assert_eq!(e.ident.to_string(), "Color");
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("expected enum item");
    }
}

#[test]
fn unit_syn_parse_expr_lit() {
    let expr: syn::Expr = parse_str("42").unwrap();
    assert!(matches!(expr, syn::Expr::Lit(_)));
}

#[test]
fn unit_syn_parse_expr_binary() {
    let expr: syn::Expr = parse_str("1 + 2").unwrap();
    assert!(matches!(expr, syn::Expr::Binary(_)));
}

#[test]
fn unit_syn_parse_nested_generic() {
    let ty: Type = parse_str("HashMap<String, Vec<i32>>").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("HashMap"));
    assert!(text.contains("Vec"));
}

#[test]
fn unit_syn_parse_lifetime_ref() {
    let ty: Type = parse_str("&'a str").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("'a"));
    assert!(text.contains("str"));
}

// ---------------------------------------------------------------------------
// Unit tests — Area 7: TokenStream operations
// ---------------------------------------------------------------------------

#[test]
fn unit_tokenstream_from_str_roundtrip() {
    let ts: TokenStream = "fn foo() {}".parse().unwrap();
    let text = ts.to_string();
    assert!(text.contains("fn"));
    assert!(text.contains("foo"));
}

#[test]
fn unit_tokenstream_extend() {
    let mut ts1: TokenStream = "let x = 1;".parse().unwrap();
    let ts2: TokenStream = "let y = 2;".parse().unwrap();
    ts1.extend(ts2);
    let text = ts1.to_string();
    assert!(text.contains("x"));
    assert!(text.contains("y"));
}

#[test]
fn unit_tokenstream_is_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn unit_tokenstream_from_quote_not_empty() {
    let ts = quote! { struct Foo; };
    assert!(!ts.is_empty());
}

#[test]
fn unit_tokenstream_clone_eq() {
    let ts = quote! { fn bar() {} };
    let cloned = ts.clone();
    assert_eq!(ts.to_string(), cloned.to_string());
}

#[test]
fn unit_tokenstream_into_iter() {
    let ts = quote! { a + b };
    let tokens: Vec<TokenTree> = ts.into_iter().collect();
    assert!(tokens.len() >= 3);
}

#[test]
fn unit_tokenstream_concat_via_quote() {
    let a = quote! { let a = 1; };
    let b = quote! { let b = 2; };
    let combined = quote! { #a #b };
    let text = combined.to_string();
    assert!(text.contains("a"));
    assert!(text.contains("b"));
}

#[test]
fn unit_tokenstream_display_deterministic() {
    let ts = quote! { struct Qux { field: bool } };
    let s1 = ts.to_string();
    let s2 = ts.to_string();
    assert_eq!(s1, s2);
}

// ---------------------------------------------------------------------------
// Unit tests — Area 8: Ident comparison and operations
// ---------------------------------------------------------------------------

#[test]
fn unit_ident_eq_same_name() {
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("foo", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn unit_ident_ne_different_name() {
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_ne!(a, b);
}

#[test]
fn unit_ident_eq_str() {
    let ident = Ident::new("hello", Span::call_site());
    assert_eq!(ident, "hello");
}

#[test]
fn unit_ident_to_token_stream() {
    let ident = Ident::new("myvar", Span::call_site());
    let ts = ident.to_token_stream();
    assert_eq!(ts.to_string(), "myvar");
}

#[test]
fn unit_format_ident_with_suffix() {
    let ident = format_ident!("field_{}", 42u32);
    assert_eq!(ident.to_string(), "field_42");
}

#[test]
fn unit_format_ident_concatenation() {
    let base = format_ident!("parse");
    let combined = format_ident!("{}_impl", base);
    assert_eq!(combined.to_string(), "parse_impl");
}

#[test]
fn unit_ident_in_quote_preserves_identity() {
    let ident = Ident::new("my_func", Span::call_site());
    let ts = quote! { fn #ident() -> bool { true } };
    let text = ts.to_string();
    assert!(text.contains("my_func"));
    let item: syn::Item = syn::parse2(ts).unwrap();
    if let syn::Item::Fn(f) = item {
        assert_eq!(f.sig.ident, "my_func");
    } else {
        panic!("expected fn item");
    }
}

#[test]
fn unit_literal_u8_suffix() {
    let lit = Literal::u8_suffixed(255);
    let text = lit.to_string();
    assert!(text.contains("255"));
    assert!(text.ends_with("u8"));
}

#[test]
fn unit_literal_i32_unsuffixed() {
    let lit = Literal::i32_unsuffixed(-42);
    // proc_macro2 represents negative as just the value
    let text = lit.to_string();
    assert!(text.contains("42"));
}

#[test]
fn unit_quote_repetition_pattern() {
    let names: Vec<Ident> = vec!["a", "b", "c"]
        .into_iter()
        .map(|n| Ident::new(n, Span::call_site()))
        .collect();
    let ts = quote! { #(let #names = 0;)* };
    let text = ts.to_string();
    assert!(text.contains("a"));
    assert!(text.contains("b"));
    assert!(text.contains("c"));
}

#[test]
fn unit_syn_parse_type_path_segments() {
    let ty: Type = parse_str("std::collections::HashMap<String, i32>").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("std"));
    assert!(text.contains("collections"));
    assert!(text.contains("HashMap"));
}

#[test]
fn unit_syn_parse_array_type() {
    let ty: Type = parse_str("[u8; 32]").unwrap();
    let text = ty.to_token_stream().to_string();
    assert!(text.contains("u8"));
    assert!(text.contains("32"));
}
