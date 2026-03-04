//! Comprehensive tests for attribute parsing v2 in adze-macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn;

// ── Derive attribute patterns ──

#[test]
fn derive_debug_v2() {
    let ts = quote! { #[derive(Debug)] struct Foo; };
    let item: syn::DeriveInput = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn derive_clone_debug_v2() {
    let ts = quote! { #[derive(Clone, Debug)] struct Bar; };
    let item: syn::DeriveInput = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}

#[test]
fn derive_with_fields_v2() {
    let ts = quote! { #[derive(Clone)] struct Baz { x: u32 } };
    let item: syn::DeriveInput = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Baz");
}

// ── Custom attribute patterns ──

#[test]
fn path_attribute_v2() {
    let ts = quote! { #[my_attr] fn foo() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}

#[test]
fn list_attribute_v2() {
    let ts = quote! { #[my_attr(x, y)] fn foo() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}

// ── Visibility patterns ──

#[test]
fn pub_struct_v2() {
    let ts = quote! { pub struct Foo; };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(matches!(item.vis, syn::Visibility::Public(_)));
}

#[test]
fn crate_struct_v2() {
    let ts = quote! { pub(crate) struct Foo; };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(matches!(item.vis, syn::Visibility::Restricted(_)));
}

// ── Field patterns ──

#[test]
fn named_fields_v2() {
    let ts = quote! { struct Point { x: f64, y: f64 } };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    if let syn::Fields::Named(fields) = &item.fields {
        assert_eq!(fields.named.len(), 2);
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn unnamed_fields_v2() {
    let ts = quote! { struct Pair(u32, u32); };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    if let syn::Fields::Unnamed(fields) = &item.fields {
        assert_eq!(fields.unnamed.len(), 2);
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn unit_struct_v2() {
    let ts = quote! { struct Unit; };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(matches!(item.fields, syn::Fields::Unit));
}

// ── Enum variant patterns ──

#[test]
fn multiple_variants_v2() {
    let ts = quote! { enum Color { Red, Green, Blue } };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn mixed_variants_v2() {
    let ts = quote! { enum Shape { Circle(f64), Rect { w: f64, h: f64 }, None } };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 3);
}

// ── Generic patterns ──

#[test]
fn lifetime_generic_v2() {
    let ts = quote! { struct Ref<'a> { data: &'a str } };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(!item.generics.params.is_empty());
}

#[test]
fn type_generic_v2() {
    let ts = quote! { struct Container<T> { inner: T } };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(!item.generics.params.is_empty());
}

#[test]
fn where_clause_v2() {
    let ts = quote! { struct Bounded<T> where T: Clone { inner: T } };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(item.generics.where_clause.is_some());
}

#[test]
fn multiple_generics_v2() {
    let ts = quote! { struct Pair<A, B> { first: A, second: B } };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 2);
}

// ── Impl blocks ──

#[test]
fn impl_block_v2() {
    let ts = quote! { impl Foo { fn bar() {} } };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
}

// ── TokenStream operations ──

#[test]
fn quote_interpolation_v2() {
    let name = quote::format_ident!("MyStruct");
    let ts = quote! { struct #name; };
    let s = ts.to_string();
    assert!(s.contains("MyStruct"));
}

#[test]
fn token_stream_extend_v2() {
    let mut ts: TokenStream = quote! { fn foo() };
    let body: TokenStream = quote! { { 42 } };
    ts.extend(body);
    let s = ts.to_string();
    assert!(s.contains("foo"));
    assert!(s.contains("42"));
}

// ── Path parsing ──

#[test]
fn simple_path_v2() {
    let path: syn::Path = syn::parse_str("std::collections::HashMap").unwrap();
    assert_eq!(path.segments.len(), 3);
}

// ── Expression patterns ──

#[test]
fn parse_match_expr_v2() {
    let expr: syn::Expr = syn::parse_str("match x { 1 => true, _ => false }").unwrap();
    assert!(matches!(expr, syn::Expr::Match(_)));
}

#[test]
fn parse_closure_v2() {
    let expr: syn::Expr = syn::parse_str("|x| x + 1").unwrap();
    assert!(matches!(expr, syn::Expr::Closure(_)));
}

// ── Const and static ──

#[test]
fn const_item_v2() {
    let ts = quote! { const N: usize = 42; };
    let item: syn::ItemConst = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "N");
}

#[test]
fn static_item_v2() {
    let ts = quote! { static X: u32 = 0; };
    let item: syn::ItemStatic = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "X");
}

// ── Use patterns ──

#[test]
fn use_simple_v2() {
    let ts = quote! { use std::collections::HashMap; };
    let _item: syn::ItemUse = syn::parse2(ts).unwrap();
}

// ── Additional patterns ──

#[test]
fn fn_with_return_type() {
    let ts = quote! { fn add(a: u32, b: u32) -> u32 { a + b } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "add");
    assert!(item.sig.output != syn::ReturnType::Default);
}

#[test]
fn fn_with_generics() {
    let ts = quote! { fn identity<T>(val: T) -> T { val } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(!item.sig.generics.params.is_empty());
}

#[test]
fn type_alias() {
    let ts = quote! { type MyVec = Vec<String>; };
    let item: syn::ItemType = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "MyVec");
}

#[test]
fn fn_with_self() {
    let ts = quote! {
        impl Foo {
            fn method(&self) -> u32 { 0 }
            fn method_mut(&mut self) {}
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 2);
}

#[test]
fn async_fn() {
    let ts = quote! { async fn do_work() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.asyncness.is_some());
}

#[test]
fn unsafe_fn() {
    let ts = quote! { unsafe fn raw_ptr(p: *const u8) -> u8 { *p } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.unsafety.is_some());
}

#[test]
fn extern_fn() {
    let ts = quote! { extern "C" fn c_func() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.abi.is_some());
}

#[test]
fn doc_attribute() {
    let ts = quote! {
        /// This is a doc comment
        struct Documented;
    };
    let item: syn::DeriveInput = syn::parse2(ts).unwrap();
    assert!(!item.attrs.is_empty());
}

#[test]
fn allow_attribute() {
    let ts = quote! { #[allow(unused)] fn unused_fn() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}

#[test]
fn repr_c_attribute() {
    let ts = quote! { #[repr(C)] struct CStruct { x: u32 } };
    let item: syn::DeriveInput = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}
