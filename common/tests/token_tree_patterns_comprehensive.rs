//! Comprehensive tests for proc_macro2 TokenTree and Group patterns.

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;

// ── TokenTree variants ──

#[test]
fn token_tree_ident() {
    let ts: TokenStream = "hello".parse().unwrap();
    let tree: Vec<TokenTree> = ts.into_iter().collect();
    assert_eq!(tree.len(), 1);
    assert!(matches!(&tree[0], TokenTree::Ident(_)));
}

#[test]
fn token_tree_literal() {
    let ts: TokenStream = "42".parse().unwrap();
    let tree: Vec<TokenTree> = ts.into_iter().collect();
    assert_eq!(tree.len(), 1);
    assert!(matches!(&tree[0], TokenTree::Literal(_)));
}

#[test]
fn token_tree_punct() {
    let ts: TokenStream = "+".parse().unwrap();
    let tree: Vec<TokenTree> = ts.into_iter().collect();
    assert_eq!(tree.len(), 1);
    assert!(matches!(&tree[0], TokenTree::Punct(_)));
}

// ── Group patterns ──

#[test]
fn group_parenthesis() {
    let inner: TokenStream = "x".parse().unwrap();
    let group = Group::new(Delimiter::Parenthesis, inner);
    assert_eq!(group.delimiter(), Delimiter::Parenthesis);
}

#[test]
fn group_brace() {
    let inner: TokenStream = "x".parse().unwrap();
    let group = Group::new(Delimiter::Brace, inner);
    assert_eq!(group.delimiter(), Delimiter::Brace);
}

#[test]
fn group_bracket() {
    let inner: TokenStream = "x".parse().unwrap();
    let group = Group::new(Delimiter::Bracket, inner);
    assert_eq!(group.delimiter(), Delimiter::Bracket);
}

#[test]
fn group_none() {
    let inner: TokenStream = "x".parse().unwrap();
    let group = Group::new(Delimiter::None, inner);
    assert_eq!(group.delimiter(), Delimiter::None);
}

#[test]
fn group_stream() {
    let inner: TokenStream = "hello world".parse().unwrap();
    let group = Group::new(Delimiter::Parenthesis, inner);
    let stream = group.stream();
    assert!(!stream.is_empty());
}

// ── Ident patterns ──

#[test]
fn ident_to_string() {
    let id = Ident::new("foo", Span::call_site());
    assert_eq!(id.to_string(), "foo");
}

#[test]
fn ident_eq() {
    let a = Ident::new("bar", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn ident_ne() {
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_ne!(a, b);
}

// ── Literal patterns ──

#[test]
fn literal_u8() {
    let lit = Literal::u8_suffixed(42);
    assert!(lit.to_string().contains("42"));
}

#[test]
fn literal_u16() {
    let lit = Literal::u16_suffixed(1000);
    assert!(lit.to_string().contains("1000"));
}

#[test]
fn literal_u32() {
    let lit = Literal::u32_suffixed(999);
    assert!(lit.to_string().contains("999"));
}

#[test]
fn literal_u64() {
    let lit = Literal::u64_suffixed(123456);
    assert!(lit.to_string().contains("123456"));
}

#[test]
fn literal_i32() {
    let lit = Literal::i32_suffixed(-42);
    assert!(lit.to_string().contains("42"));
}

#[test]
fn literal_f32() {
    let lit = Literal::f32_suffixed(3.14);
    assert!(lit.to_string().contains("3.14"));
}

#[test]
fn literal_f64() {
    let lit = Literal::f64_suffixed(2.718);
    assert!(lit.to_string().contains("2.718"));
}

#[test]
fn literal_string() {
    let lit = Literal::string("hello");
    assert!(lit.to_string().contains("hello"));
}

#[test]
fn literal_byte_string() {
    let lit = Literal::byte_string(b"bytes");
    let s = lit.to_string();
    assert!(s.contains("bytes"));
}

#[test]
fn literal_character() {
    let lit = Literal::character('x');
    assert!(lit.to_string().contains('x'));
}

// ── Punct patterns ──

#[test]
fn punct_alone() {
    let p = Punct::new('+', Spacing::Alone);
    assert_eq!(p.as_char(), '+');
    assert_eq!(p.spacing(), Spacing::Alone);
}

#[test]
fn punct_joint() {
    let p = Punct::new(':', Spacing::Joint);
    assert_eq!(p.as_char(), ':');
    assert_eq!(p.spacing(), Spacing::Joint);
}

// ── Quote patterns ──

#[test]
fn quote_struct_tokens() {
    let ts = quote! { struct Foo { x: u32 } };
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    assert!(trees.len() >= 2); // at least keyword and ident
}

#[test]
fn quote_fn_tokens() {
    let ts = quote! { fn bar() {} };
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    assert!(!trees.is_empty());
}

#[test]
fn quote_with_group() {
    let ts = quote! { (a, b, c) };
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    assert_eq!(trees.len(), 1); // one group
    assert!(matches!(&trees[0], TokenTree::Group(_)));
}

// ── TokenStream operations ──

#[test]
fn empty_stream() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn stream_from_str() {
    let ts: TokenStream = "let x = 1;".parse().unwrap();
    assert!(!ts.is_empty());
}

#[test]
fn stream_extend() {
    let mut ts = TokenStream::new();
    ts.extend(quote! { foo });
    assert!(!ts.is_empty());
}

#[test]
fn stream_clone() {
    let ts = quote! { hello };
    let ts2 = ts.clone();
    assert_eq!(ts.to_string(), ts2.to_string());
}

// ── TokenTree conversion ──

#[test]
fn token_tree_from_ident() {
    let id = Ident::new("myvar", Span::call_site());
    let tt: TokenTree = id.into();
    assert_eq!(tt.to_string(), "myvar");
}

#[test]
fn token_tree_from_literal() {
    let lit = Literal::u32_suffixed(7);
    let tt: TokenTree = lit.into();
    assert!(tt.to_string().contains("7"));
}

#[test]
fn token_tree_from_punct() {
    let p = Punct::new('!', Spacing::Alone);
    let tt: TokenTree = p.into();
    assert_eq!(tt.to_string(), "!");
}

#[test]
fn token_tree_from_group() {
    let inner: TokenStream = "x".parse().unwrap();
    let group = Group::new(Delimiter::Parenthesis, inner);
    let tt: TokenTree = group.into();
    let s = tt.to_string();
    assert!(s.contains("x"));
}
