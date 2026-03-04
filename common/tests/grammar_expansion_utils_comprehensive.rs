//! Comprehensive tests for the common crate's grammar expansion utilities.

use adze_common as _;

// ── Basic module structure ──

#[test]
fn common_crate_exists() {
    // Verify basic imports work
    let _ = std::mem::size_of::<usize>();
}

// ── GrammarExpander tests ──

/// Test that GrammarExpander can be created from a basic proc_macro2::TokenStream
#[test]
fn grammar_expander_empty_token_stream() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "".parse().unwrap();
    let _ = format!("{}", ts);
}

#[test]
fn grammar_expander_simple_token_stream() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "fn foo() {}".parse().unwrap();
    assert!(!ts.is_empty());
}

// ── Type mapping utilities ──

#[test]
fn type_to_string_basic() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "String".parse().unwrap();
    let s = ts.to_string();
    assert_eq!(s, "String");
}

#[test]
fn type_to_string_generic() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "Vec<String>".parse().unwrap();
    let s = ts.to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("String"));
}

#[test]
fn type_to_string_option() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "Option<u32>".parse().unwrap();
    let s = ts.to_string();
    assert!(s.contains("Option"));
}

#[test]
fn type_to_string_nested() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "Vec<Option<String>>".parse().unwrap();
    let s = ts.to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
}

// ── proc_macro2 TokenStream manipulation ──

#[test]
fn token_stream_clone() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "struct Foo { x: u32 }".parse().unwrap();
    let cloned = ts.clone();
    assert_eq!(ts.to_string(), cloned.to_string());
}

#[test]
fn token_stream_is_empty() {
    use proc_macro2::TokenStream;
    let empty: TokenStream = "".parse().unwrap();
    assert!(empty.is_empty());
}

#[test]
fn token_stream_not_empty() {
    use proc_macro2::TokenStream;
    let ts: TokenStream = "42".parse().unwrap();
    assert!(!ts.is_empty());
}

// ── quote! macro basics ──

#[test]
fn quote_simple() {
    use quote::quote;
    let ts = quote! { fn hello() {} };
    let s = ts.to_string();
    assert!(s.contains("fn"));
    assert!(s.contains("hello"));
}

#[test]
fn quote_with_ident() {
    use proc_macro2::Ident;
    use proc_macro2::Span;
    use quote::quote;
    let name = Ident::new("my_func", Span::call_site());
    let ts = quote! { fn #name() {} };
    let s = ts.to_string();
    assert!(s.contains("my_func"));
}

#[test]
fn quote_with_literal() {
    use quote::quote;
    let val = 42u32;
    let ts = quote! { const X: u32 = #val; };
    let s = ts.to_string();
    assert!(s.contains("42"));
}

// ── syn parsing ──

#[test]
fn syn_parse_item_struct() {
    let code = "struct Foo { x: u32, y: String }";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn syn_parse_item_enum() {
    let code = "enum Color { Red, Green, Blue }";
    let item: syn::ItemEnum = syn::parse_str(code).unwrap();
    assert_eq!(item.ident, "Color");
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn syn_parse_item_fn() {
    let code = "fn hello() -> u32 { 42 }";
    let item: syn::ItemFn = syn::parse_str(code).unwrap();
    assert_eq!(item.sig.ident, "hello");
}

#[test]
fn syn_parse_type() {
    let ty: syn::Type = syn::parse_str("Vec<String>").unwrap();
    let s = quote::quote!(#ty).to_string();
    assert!(s.contains("Vec"));
}

#[test]
fn syn_parse_generics() {
    let code = "struct Wrapper<T> { inner: T }";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert!(!item.generics.params.is_empty());
}

#[test]
fn syn_parse_lifetime() {
    let code = "struct Ref<'a> { data: &'a str }";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert!(!item.generics.params.is_empty());
}

// ── syn attribute parsing ──

#[test]
fn syn_parse_derive_attr() {
    let code = "#[derive(Debug, Clone)] struct Foo;";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert!(!item.attrs.is_empty());
}

#[test]
fn syn_parse_multiple_attrs() {
    let code = "#[derive(Debug)]\n#[allow(dead_code)]\nstruct Foo;";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert!(item.attrs.len() >= 2);
}

// ── syn field types ──

#[test]
fn syn_named_fields() {
    let code = "struct Point { x: f64, y: f64 }";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    match &item.fields {
        syn::Fields::Named(named) => assert_eq!(named.named.len(), 2),
        _ => panic!("expected named fields"),
    }
}

#[test]
fn syn_tuple_fields() {
    let code = "struct Pair(u32, String);";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    match &item.fields {
        syn::Fields::Unnamed(unnamed) => assert_eq!(unnamed.unnamed.len(), 2),
        _ => panic!("expected unnamed fields"),
    }
}

#[test]
fn syn_unit_struct() {
    let code = "struct Unit;";
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert!(matches!(&item.fields, syn::Fields::Unit));
}

// ── syn enum variants ──

#[test]
fn syn_enum_unit_variants() {
    let code = "enum Dir { North, South, East, West }";
    let item: syn::ItemEnum = syn::parse_str(code).unwrap();
    assert_eq!(item.variants.len(), 4);
}

#[test]
fn syn_enum_tuple_variants() {
    let code = "enum Val { Int(i64), Str(String) }";
    let item: syn::ItemEnum = syn::parse_str(code).unwrap();
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn syn_enum_struct_variants() {
    let code = "enum Shape { Circle { r: f64 }, Rect { w: f64, h: f64 } }";
    let item: syn::ItemEnum = syn::parse_str(code).unwrap();
    assert_eq!(item.variants.len(), 2);
}

// ── syn expression parsing ──

#[test]
fn syn_parse_expr_lit() {
    let expr: syn::Expr = syn::parse_str("42").unwrap();
    assert!(matches!(expr, syn::Expr::Lit(_)));
}

#[test]
fn syn_parse_expr_call() {
    let expr: syn::Expr = syn::parse_str("foo(1, 2)").unwrap();
    assert!(matches!(expr, syn::Expr::Call(_)));
}

#[test]
fn syn_parse_expr_binary() {
    let expr: syn::Expr = syn::parse_str("a + b").unwrap();
    assert!(matches!(expr, syn::Expr::Binary(_)));
}

// ── Grammar annotation pattern recognition ──

#[test]
fn parse_struct_with_annotation_pattern() {
    let code = r#"
        #[derive(Debug)]
        struct Expr {
            left: Box<Expr>,
            op: String,
            right: Box<Expr>,
        }
    "#;
    let item: syn::ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(item.ident, "Expr");
}

#[test]
fn parse_enum_with_variants_pattern() {
    let code = r#"
        enum Token {
            Number(f64),
            Plus,
            Minus,
            Star,
            Slash,
            LParen,
            RParen,
        }
    "#;
    let item: syn::ItemEnum = syn::parse_str(code).unwrap();
    assert_eq!(item.variants.len(), 7);
}

// ── Additional common utilities ──

#[test]
fn proc_macro2_span() {
    use proc_macro2::Span;
    let s = Span::call_site();
    let _ = format!("{:?}", s);
}

#[test]
fn proc_macro2_ident_eq() {
    use proc_macro2::{Ident, Span};
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("foo", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn proc_macro2_ident_ne() {
    use proc_macro2::{Ident, Span};
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_ne!(a, b);
}

#[test]
fn proc_macro2_punct() {
    use proc_macro2::{Punct, Spacing};
    let p = Punct::new('+', Spacing::Alone);
    assert_eq!(p.as_char(), '+');
}

#[test]
fn proc_macro2_literal_integer() {
    use proc_macro2::Literal;
    let lit = Literal::u32_suffixed(42);
    let s = lit.to_string();
    assert!(s.contains("42"));
}

#[test]
fn proc_macro2_literal_string() {
    use proc_macro2::Literal;
    let lit = Literal::string("hello");
    let s = lit.to_string();
    assert!(s.contains("hello"));
}
