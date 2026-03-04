//! Property-based and unit tests for TokenStream manipulation patterns.
//!
//! Covers: identifier parsing, clone equality, deterministic stringification,
//! quote! generation, syn type parsing, literal types, Ident creation/comparison,
//! and multi-stream concatenation.

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers that are NOT reserved keywords in edition 2024.
fn valid_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,11}")
        .unwrap()
        .prop_filter("must parse as syn::Ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Primitive type names suitable for `syn::parse_str::<Type>`.
fn prim_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Simple numeric literal strings.
fn numeric_lit() -> impl Strategy<Value = String> {
    prop_oneof![
        (0u64..10_000).prop_map(|v| v.to_string()),
        (0u64..10_000).prop_map(|v| format!("{v}u32")),
        (0u64..10_000).prop_map(|v| format!("{v}i64")),
    ]
}

/// Small repetition counts.
fn small_n() -> impl Strategy<Value = usize> {
    1usize..=15
}

// ---------------------------------------------------------------------------
// Property-based tests (proptest)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // --- Area 1: any valid Rust identifier can be parsed by syn ---

    #[test]
    fn prop_valid_ident_parsed_by_syn(name in valid_ident()) {
        let parsed: syn::Ident = syn::parse_str(&name).unwrap();
        prop_assert_eq!(parsed.to_string(), name);
    }

    #[test]
    fn prop_ident_roundtrips_through_token_stream(name in valid_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = ident.to_token_stream();
        let reparsed: syn::Ident = syn::parse2(ts).unwrap();
        prop_assert_eq!(reparsed.to_string(), name);
    }

    // --- Area 2: TokenStream clone equals original ---

    #[test]
    fn prop_clone_equals_original_simple(ty in prim_type()) {
        let ts: TokenStream = ty.parse().unwrap();
        let cloned = ts.clone();
        prop_assert_eq!(ts.to_string(), cloned.to_string());
    }

    #[test]
    fn prop_clone_equals_original_complex(name in valid_ident()) {
        let ts = quote! { struct #(Ident::new(&name, Span::call_site())) { x: u32 } };
        // Build via ident interpolation
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { fn #ident() -> bool { true } };
        let cloned = ts.clone();
        prop_assert_eq!(ts.to_string(), cloned.to_string());
    }

    // --- Area 3: TokenStream to_string is deterministic ---

    #[test]
    fn prop_to_string_deterministic(ty in prim_type()) {
        let ts: TokenStream = ty.parse().unwrap();
        let s1 = ts.to_string();
        let s2 = ts.to_string();
        prop_assert_eq!(s1, s2);
    }

    #[test]
    fn prop_to_string_deterministic_quote(name in valid_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { let #ident = 42; };
        let s1 = ts.to_string();
        let s2 = ts.to_string();
        prop_assert_eq!(s1, s2);
    }

    // --- Area 4: quote! with various idents produces non-empty streams ---

    #[test]
    fn prop_quote_with_ident_nonempty(name in valid_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { #ident };
        prop_assert!(!ts.is_empty());
    }

    #[test]
    fn prop_quote_fn_def_nonempty(name in valid_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        let ts = quote! { fn #ident() {} };
        prop_assert!(!ts.is_empty());
        prop_assert!(ts.to_string().contains(&name));
    }

    #[test]
    fn prop_quote_let_binding_nonempty(name in valid_ident(), ty in prim_type()) {
        let ident = Ident::new(&name, Span::call_site());
        let parsed_ty: Type = parse_str(ty).unwrap();
        let ts = quote! { let #ident: #parsed_ty; };
        prop_assert!(!ts.is_empty());
        prop_assert!(ts.to_string().contains(&name));
        prop_assert!(ts.to_string().contains(ty));
    }

    // --- Area 5: syn parse_str for valid types succeeds ---

    #[test]
    fn prop_parse_str_primitive_type(ty in prim_type()) {
        let parsed: Type = parse_str(ty).unwrap();
        prop_assert_eq!(parsed.to_token_stream().to_string(), ty);
    }

    #[test]
    fn prop_parse_str_vec_of_type(ty in prim_type()) {
        let src = format!("Vec<{ty}>");
        let parsed: Type = parse_str(&src).unwrap();
        let s = parsed.to_token_stream().to_string();
        prop_assert!(s.contains("Vec"));
        prop_assert!(s.contains(ty));
    }

    #[test]
    fn prop_parse_str_option_type(ty in prim_type()) {
        let src = format!("Option<{ty}>");
        let parsed: Type = parse_str(&src).unwrap();
        let s = parsed.to_token_stream().to_string();
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains(ty));
    }

    #[test]
    fn prop_parse_str_tuple_type(a in prim_type(), b in prim_type()) {
        let src = format!("({a}, {b})");
        let parsed: Type = parse_str(&src).unwrap();
        let s = parsed.to_token_stream().to_string();
        prop_assert!(s.contains(a));
        prop_assert!(s.contains(b));
    }

    #[test]
    fn prop_parse_str_reference_type(ty in prim_type()) {
        let src = format!("&{ty}");
        let parsed: Type = parse_str(&src).unwrap();
        let s = parsed.to_token_stream().to_string();
        prop_assert!(s.contains(ty));
    }

    // --- Area 9: multiple quote! concatenation ---

    #[test]
    fn prop_quote_extend_preserves_all(count in small_n()) {
        let mut combined = TokenStream::new();
        for i in 0..count {
            let lit = Literal::usize_unsuffixed(i);
            combined.extend(quote! { #lit });
        }
        let s = combined.to_string();
        for i in 0..count {
            prop_assert!(s.contains(&i.to_string()));
        }
    }

    #[test]
    fn prop_quote_concat_order(count in small_n()) {
        let idents: Vec<Ident> = (0..count)
            .map(|i| Ident::new(&format!("x{i}"), Span::call_site()))
            .collect();
        let ts = quote! { #(#idents),* };
        let s = ts.to_string();
        for i in 0..count.saturating_sub(1) {
            let pos_a = s.find(&format!("x{i}")).unwrap();
            let pos_b = s.find(&format!("x{}", i + 1)).unwrap();
            prop_assert!(pos_a < pos_b);
        }
    }

    #[test]
    fn prop_multiple_quote_stmts_concat(a in valid_ident(), b in valid_ident()) {
        let id_a = Ident::new(&a, Span::call_site());
        let id_b = Ident::new(&b, Span::call_site());
        let ts1 = quote! { let #id_a = 1; };
        let ts2 = quote! { let #id_b = 2; };
        let mut combined = TokenStream::new();
        combined.extend(ts1);
        combined.extend(ts2);
        let s = combined.to_string();
        prop_assert!(s.contains(&a));
        prop_assert!(s.contains(&b));
    }

    // --- Area 10: Ident comparison properties ---

    #[test]
    fn prop_ident_eq_same_name(name in valid_ident()) {
        let a = Ident::new(&name, Span::call_site());
        let b = Ident::new(&name, Span::call_site());
        prop_assert_eq!(a, b);
    }

    #[test]
    fn prop_ident_ne_different_names(a in valid_ident(), b in valid_ident()) {
        prop_assume!(a != b);
        let id_a = Ident::new(&a, Span::call_site());
        let id_b = Ident::new(&b, Span::call_site());
        prop_assert_ne!(id_a, id_b);
    }

    #[test]
    fn prop_ident_eq_string(name in valid_ident()) {
        let ident = Ident::new(&name, Span::call_site());
        prop_assert!(ident == name.as_str());
    }

    // --- Additional property tests ---

    #[test]
    fn prop_token_stream_from_numeric_lit(lit in numeric_lit()) {
        let ts: TokenStream = lit.parse().unwrap();
        prop_assert!(!ts.is_empty());
    }

    #[test]
    fn prop_token_stream_reparsable(ty in prim_type()) {
        let ts: TokenStream = ty.parse().unwrap();
        let s = ts.to_string();
        let ts2: TokenStream = s.parse().unwrap();
        prop_assert_eq!(ts.to_string(), ts2.to_string());
    }

    #[test]
    fn prop_struct_def_produces_tokens(name in valid_ident(), field_count in 1usize..=5) {
        let upper = capitalize(&name);
        let fields: Vec<String> = (0..field_count).map(|i| format!("f{i}: u32")).collect();
        let src = format!("struct {upper} {{ {} }}", fields.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        prop_assert!(!ts.is_empty());
        prop_assert!(ts.to_string().contains(&upper));
    }

    #[test]
    fn prop_enum_def_produces_tokens(name in valid_ident(), variant_count in 1usize..=6) {
        let upper = capitalize(&name);
        let variants: Vec<String> = (0..variant_count).map(|i| format!("V{i}")).collect();
        let src = format!("enum {upper} {{ {} }}", variants.join(", "));
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        prop_assert!(!ts.is_empty());
    }

    #[test]
    fn prop_fn_item_roundtrip(name in valid_ident(), ret in prim_type()) {
        let src = format!("fn {name}() -> {ret} {{ todo!() }}");
        let item: syn::Item = parse_str(&src).unwrap();
        let ts = item.to_token_stream();
        let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
        prop_assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
    }

    #[test]
    fn prop_nested_option_vec(ty in prim_type()) {
        let src = format!("Option<Vec<{ty}>>");
        let parsed: Type = parse_str(&src).unwrap();
        let s = parsed.to_token_stream().to_string();
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains("Vec"));
        prop_assert!(s.contains(ty));
    }

    #[test]
    fn prop_token_tree_count_at_least_one(ty in prim_type()) {
        let ts: TokenStream = ty.parse().unwrap();
        prop_assert!(ts.into_iter().count() >= 1);
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[test]
fn unit_empty_token_stream_is_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
    assert_eq!(ts.to_string(), "");
}

#[test]
fn unit_empty_string_parses_to_empty() {
    let ts: TokenStream = "".parse().unwrap();
    assert!(ts.is_empty());
}

#[test]
fn unit_single_ident_token_stream() {
    let ts: TokenStream = "hello".parse().unwrap();
    assert_eq!(ts.to_string(), "hello");
}

#[test]
fn unit_ident_new_basic() {
    let ident = Ident::new("foo", Span::call_site());
    assert_eq!(ident.to_string(), "foo");
}

#[test]
fn unit_ident_new_underscore_prefix() {
    let ident = Ident::new("_bar", Span::call_site());
    assert_eq!(ident.to_string(), "_bar");
}

#[test]
fn unit_ident_new_with_numbers() {
    let ident = Ident::new("x123", Span::call_site());
    assert_eq!(ident.to_string(), "x123");
}

#[test]
fn unit_ident_equality_same_name() {
    let a = Ident::new("test_name", Span::call_site());
    let b = Ident::new("test_name", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn unit_ident_inequality_diff_name() {
    let a = Ident::new("alpha", Span::call_site());
    let b = Ident::new("beta", Span::call_site());
    assert_ne!(a, b);
}

#[test]
fn unit_ident_eq_str() {
    let ident = Ident::new("hello", Span::call_site());
    assert!(ident == "hello");
}

#[test]
fn unit_literal_u32() {
    let lit = Literal::u32_suffixed(42);
    assert_eq!(lit.to_string(), "42u32");
}

#[test]
fn unit_literal_i64() {
    let lit = Literal::i64_suffixed(-7);
    assert_eq!(lit.to_string(), "-7i64");
}

#[test]
fn unit_literal_f64() {
    let lit = Literal::f64_suffixed(3.14);
    assert!(lit.to_string().starts_with("3.14"));
}

#[test]
fn unit_literal_string() {
    let lit = Literal::string("hello world");
    assert_eq!(lit.to_string(), "\"hello world\"");
}

#[test]
fn unit_literal_byte_string() {
    let lit = Literal::byte_string(b"abc");
    assert!(lit.to_string().contains("abc"));
}

#[test]
fn unit_literal_character() {
    let lit = Literal::character('Z');
    assert_eq!(lit.to_string(), "'Z'");
}

#[test]
fn unit_literal_usize_unsuffixed() {
    let lit = Literal::usize_unsuffixed(100);
    assert_eq!(lit.to_string(), "100");
}

#[test]
fn unit_token_stream_extend() {
    let mut ts = TokenStream::new();
    let part1: TokenStream = "a".parse().unwrap();
    let part2: TokenStream = "b".parse().unwrap();
    ts.extend(part1);
    ts.extend(part2);
    assert_eq!(ts.to_string(), "a b");
}

#[test]
fn unit_token_stream_from_iter() {
    let trees: Vec<TokenTree> = "x + y"
        .parse::<TokenStream>()
        .unwrap()
        .into_iter()
        .collect();
    let rebuilt: TokenStream = trees.into_iter().collect();
    assert_eq!(rebuilt.to_string(), "x + y");
}

#[test]
fn unit_quote_simple_struct() {
    let ts = quote! { struct Foo { x: u32 } };
    let s = ts.to_string();
    assert!(s.contains("struct"));
    assert!(s.contains("Foo"));
    assert!(s.contains("u32"));
}

#[test]
fn unit_quote_interpolation() {
    let name = Ident::new("my_var", Span::call_site());
    let ts = quote! { let #name = 42; };
    assert!(ts.to_string().contains("my_var"));
    assert!(ts.to_string().contains("42"));
}

#[test]
fn unit_quote_repeated_interpolation() {
    let names: Vec<Ident> = vec![
        Ident::new("a", Span::call_site()),
        Ident::new("b", Span::call_site()),
        Ident::new("c", Span::call_site()),
    ];
    let ts = quote! { #(#names),* };
    let s = ts.to_string();
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

#[test]
fn unit_syn_parse_str_type() {
    let ty: Type = parse_str("Vec<String>").unwrap();
    let s = ty.to_token_stream().to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("String"));
}

#[test]
fn unit_syn_parse_str_nested_type() {
    let ty: Type = parse_str("HashMap<String, Vec<u32>>").unwrap();
    let s = ty.to_token_stream().to_string();
    assert!(s.contains("HashMap"));
    assert!(s.contains("String"));
    assert!(s.contains("Vec"));
    assert!(s.contains("u32"));
}

#[test]
fn unit_syn_parse_str_fn_pointer() {
    let ty: Type = parse_str("fn(i32) -> bool").unwrap();
    let s = ty.to_token_stream().to_string();
    assert!(s.contains("fn"));
    assert!(s.contains("i32"));
    assert!(s.contains("bool"));
}

#[test]
fn unit_token_stream_clone_independence() {
    let original: TokenStream = "let x = 1;".parse().unwrap();
    let cloned = original.clone();
    // Extending original doesn't affect clone
    let mut extended = original;
    extended.extend("let y = 2;".parse::<TokenStream>().unwrap());
    assert!(extended.to_string().contains("y"));
    assert!(!cloned.to_string().contains("y"));
}

#[test]
fn unit_multiple_extends() {
    let mut ts = TokenStream::new();
    for i in 0..5 {
        let part: TokenStream = format!("item_{i}").parse().unwrap();
        ts.extend(part);
    }
    let s = ts.to_string();
    for i in 0..5 {
        assert!(s.contains(&format!("item_{i}")));
    }
}

#[test]
fn unit_token_tree_variants() {
    let ts: TokenStream = "foo(42, \"hi\")".parse().unwrap();
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    assert!(matches!(&trees[0], TokenTree::Ident(_)));
    assert!(matches!(&trees[1], TokenTree::Group(_)));
}

#[test]
fn unit_group_delimiter_paren() {
    let ts: TokenStream = "(a, b)".parse().unwrap();
    let tree = ts.into_iter().next().unwrap();
    if let TokenTree::Group(g) = tree {
        assert_eq!(g.delimiter(), proc_macro2::Delimiter::Parenthesis);
    } else {
        panic!("expected Group");
    }
}

#[test]
fn unit_group_delimiter_brace() {
    let ts: TokenStream = "{ x }".parse().unwrap();
    let tree = ts.into_iter().next().unwrap();
    if let TokenTree::Group(g) = tree {
        assert_eq!(g.delimiter(), proc_macro2::Delimiter::Brace);
    } else {
        panic!("expected Group");
    }
}

#[test]
fn unit_group_delimiter_bracket() {
    let ts: TokenStream = "[1, 2]".parse().unwrap();
    let tree = ts.into_iter().next().unwrap();
    if let TokenTree::Group(g) = tree {
        assert_eq!(g.delimiter(), proc_macro2::Delimiter::Bracket);
    } else {
        panic!("expected Group");
    }
}

#[test]
fn unit_punct_tokens() {
    let ts: TokenStream = "a + b".parse().unwrap();
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    assert_eq!(trees.len(), 3);
    assert!(matches!(&trees[1], TokenTree::Punct(p) if p.as_char() == '+'));
}

#[test]
fn unit_invalid_token_stream_parse() {
    // Unclosed delimiter is an error
    let result: Result<TokenStream, _> = "fn foo( {".parse();
    assert!(result.is_err());
}

#[test]
fn unit_quote_empty_block() {
    let ts = quote! {};
    assert!(ts.is_empty());
}

#[test]
fn unit_syn_parse_item_roundtrip() {
    let src = "const X: u32 = 42;";
    let item: syn::Item = parse_str(src).unwrap();
    let ts = item.to_token_stream();
    let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
    assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
}

#[test]
fn unit_syn_parse_impl_roundtrip() {
    let src = "impl Foo { fn bar(&self) -> u32 { 0 } }";
    let item: syn::Item = parse_str(src).unwrap();
    let ts = item.to_token_stream();
    let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
    assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
}

#[test]
fn unit_syn_parse_trait_roundtrip() {
    let src = "trait MyTrait { fn do_stuff(&self); }";
    let item: syn::Item = parse_str(src).unwrap();
    let ts = item.to_token_stream();
    let reparsed: syn::Item = syn::parse2(ts.clone()).unwrap();
    assert_eq!(ts.to_string(), reparsed.to_token_stream().to_string());
}

#[test]
fn unit_literal_i32_zero() {
    let lit = Literal::i32_suffixed(0);
    assert_eq!(lit.to_string(), "0i32");
}

#[test]
fn unit_literal_bool_via_ident() {
    // `true` and `false` are Idents in proc_macro2, not Literals
    let ts: TokenStream = "true".parse().unwrap();
    let tree = ts.into_iter().next().unwrap();
    assert!(matches!(tree, TokenTree::Ident(_)));
}
