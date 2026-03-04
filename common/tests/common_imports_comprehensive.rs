//! Comprehensive tests for adze-common crate imports and basic functionality.

use proc_macro2::TokenStream;
use quote::quote;

// ── TokenStream basics ──

#[test]
fn empty_token_stream() {
    let ts: TokenStream = "".parse().unwrap();
    assert!(ts.is_empty());
}

#[test]
fn non_empty_token_stream() {
    let ts: TokenStream = "fn foo() {}".parse().unwrap();
    assert!(!ts.is_empty());
}

#[test]
fn token_stream_to_string() {
    let ts: TokenStream = "let x = 42;".parse().unwrap();
    let s = ts.to_string();
    assert!(s.contains("42"));
}

#[test]
fn token_stream_clone_eq() {
    let ts: TokenStream = "struct Foo;".parse().unwrap();
    let cloned = ts.clone();
    assert_eq!(ts.to_string(), cloned.to_string());
}

// ── Quote macro ──

#[test]
fn quote_fn() {
    let ts = quote! { fn hello() -> u32 { 42 } };
    assert!(!ts.is_empty());
}

#[test]
fn quote_struct() {
    let ts = quote! { struct Point { x: f64, y: f64 } };
    let s = ts.to_string();
    assert!(s.contains("Point"));
}

#[test]
fn quote_enum() {
    let ts = quote! { enum Color { Red, Green, Blue } };
    let s = ts.to_string();
    assert!(s.contains("Color"));
}

#[test]
fn quote_impl() {
    let ts = quote! { impl Foo { fn bar(&self) {} } };
    let s = ts.to_string();
    assert!(s.contains("impl"));
}

// ── Syn parsing ──

#[test]
fn syn_parse_struct() {
    let item: syn::ItemStruct = syn::parse_str("struct Foo { x: u32 }").unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn syn_parse_enum() {
    let item: syn::ItemEnum = syn::parse_str("enum Bar { A, B }").unwrap();
    assert_eq!(item.ident, "Bar");
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn syn_parse_type() {
    let ty: syn::Type = syn::parse_str("Vec<String>").unwrap();
    let s = quote!(#ty).to_string();
    assert!(s.contains("Vec"));
}

#[test]
fn syn_parse_fn() {
    let item: syn::ItemFn = syn::parse_str("fn add(a: u32, b: u32) -> u32 { a + b }").unwrap();
    assert_eq!(item.sig.ident, "add");
}

#[test]
fn syn_parse_trait() {
    let item: syn::ItemTrait = syn::parse_str("trait Greet { fn hello(&self); }").unwrap();
    assert_eq!(item.ident, "Greet");
}

#[test]
fn syn_parse_const() {
    let item: syn::ItemConst = syn::parse_str("const N: usize = 42;").unwrap();
    assert_eq!(item.ident, "N");
}

// ── Ident operations ──

#[test]
fn ident_creation() {
    use proc_macro2::{Ident, Span};
    let id = Ident::new("my_var", Span::call_site());
    assert_eq!(id.to_string(), "my_var");
}

#[test]
fn ident_equality() {
    use proc_macro2::{Ident, Span};
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("foo", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn ident_inequality() {
    use proc_macro2::{Ident, Span};
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_ne!(a, b);
}

// ── Literal types ──

#[test]
fn literal_u32() {
    use proc_macro2::Literal;
    let lit = Literal::u32_suffixed(42);
    assert!(lit.to_string().contains("42"));
}

#[test]
fn literal_string() {
    use proc_macro2::Literal;
    let lit = Literal::string("hello world");
    assert!(lit.to_string().contains("hello"));
}

#[test]
fn literal_f64() {
    use proc_macro2::Literal;
    let lit = Literal::f64_suffixed(3.5);
    assert!(lit.to_string().contains("3.5"));
}

// ── Complex type patterns ──

#[test]
fn parse_option_type() {
    let ty: syn::Type = syn::parse_str("Option<u32>").unwrap();
    let _ = quote!(#ty).to_string();
}

#[test]
fn parse_box_type() {
    let ty: syn::Type = syn::parse_str("Box<dyn Trait>").unwrap();
    let _ = quote!(#ty).to_string();
}

#[test]
fn parse_result_type() {
    let ty: syn::Type = syn::parse_str("Result<String, Error>").unwrap();
    let _ = quote!(#ty).to_string();
}

#[test]
fn parse_tuple_type() {
    let ty: syn::Type = syn::parse_str("(u32, String, bool)").unwrap();
    let _ = quote!(#ty).to_string();
}

#[test]
fn parse_slice_type() {
    let ty: syn::Type = syn::parse_str("&[u8]").unwrap();
    let _ = quote!(#ty).to_string();
}

#[test]
fn parse_fn_pointer_type() {
    let ty: syn::Type = syn::parse_str("fn(u32) -> bool").unwrap();
    let _ = quote!(#ty).to_string();
}
