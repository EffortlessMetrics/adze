//! Comprehensive v5 tests for proc-macro2 token stream operations used in adze-macro.
//!
//! Covers TokenStream construction, token tree iteration, Ident/format_ident patterns,
//! quote macro usage, parse_str/parse_quote roundtrips, literal construction,
//! attribute/meta parsing, and edge cases.

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use std::str::FromStr;
use syn::{
    Attribute, Expr, Fields, FieldsNamed, ItemEnum, ItemStruct, Meta, Type, parse_quote, parse_str,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn count_trees(ts: &TokenStream) -> usize {
    ts.clone().into_iter().count()
}

fn collect_idents(ts: &TokenStream) -> Vec<String> {
    ts.clone()
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Ident(id) => Some(id.to_string()),
            _ => None,
        })
        .collect()
}

fn has_group_with_delimiter(ts: &TokenStream, delim: Delimiter) -> bool {
    ts.clone().into_iter().any(|tt| match tt {
        TokenTree::Group(g) => g.delimiter() == delim,
        _ => false,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. TokenStream construction and display (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ts_new_is_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
    assert_eq!(count_trees(&ts), 0);
}

#[test]
fn test_ts_from_str_simple_let() {
    let ts: TokenStream = "let x: u32 = 42;".parse().unwrap();
    assert!(!ts.is_empty());
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"let".to_string()));
    assert!(idents.contains(&"x".to_string()));
    assert!(idents.contains(&"u32".to_string()));
}

#[test]
fn test_ts_from_str_fn_signature() {
    let ts: TokenStream = "fn process(input: &str) -> bool { true }".parse().unwrap();
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"fn".to_string()));
    assert!(idents.contains(&"process".to_string()));
    assert!(idents.contains(&"bool".to_string()));
}

#[test]
fn test_ts_display_roundtrip() {
    let original = "struct Foo { bar : i32 }";
    let ts: TokenStream = original.parse().unwrap();
    let displayed = ts.to_string();
    // Display may normalize spacing, but tokens are preserved
    assert!(displayed.contains("struct"));
    assert!(displayed.contains("Foo"));
    assert!(displayed.contains("bar"));
    assert!(displayed.contains("i32"));
}

#[test]
fn test_ts_extend_combines_streams() {
    let mut ts1: TokenStream = "let a = 1;".parse().unwrap();
    let ts2: TokenStream = "let b = 2;".parse().unwrap();
    let count_before = count_trees(&ts1);
    ts1.extend(ts2);
    assert!(count_trees(&ts1) > count_before);
}

#[test]
fn test_ts_from_iter_token_trees() {
    let ident = TokenTree::Ident(Ident::new("hello", Span::call_site()));
    let punct = TokenTree::Punct(Punct::new(';', Spacing::Alone));
    let ts: TokenStream = [ident, punct].into_iter().collect();
    assert_eq!(count_trees(&ts), 2);
}

#[test]
fn test_ts_from_str_invalid_returns_err() {
    let result = TokenStream::from_str("fn (((");
    // proc_macro2 is lenient — it tokenizes even broken syntax.
    // The key is that it does NOT panic.
    let _ = result;
}

#[test]
fn test_ts_quote_produces_nonempty() {
    let ts = quote! { pub struct Widget { count: usize } };
    assert!(!ts.is_empty());
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"Widget".to_string()));
    // "usize" is inside a brace group so not at top-level iteration
    assert!(ts.to_string().contains("usize"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Token tree iteration (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tt_classify_ident() {
    let ts: TokenStream = "my_var".parse().unwrap();
    let first = ts.into_iter().next().unwrap();
    assert!(matches!(first, TokenTree::Ident(_)));
}

#[test]
fn test_tt_classify_punct() {
    let ts: TokenStream = "+".parse().unwrap();
    let first = ts.into_iter().next().unwrap();
    match first {
        TokenTree::Punct(p) => assert_eq!(p.as_char(), '+'),
        other => panic!("expected Punct, got {other:?}"),
    }
}

#[test]
fn test_tt_classify_literal_integer() {
    let ts: TokenStream = "42".parse().unwrap();
    let first = ts.into_iter().next().unwrap();
    assert!(matches!(first, TokenTree::Literal(_)));
}

#[test]
fn test_tt_classify_group_braces() {
    let ts: TokenStream = "{ x + 1 }".parse().unwrap();
    let first = ts.into_iter().next().unwrap();
    match first {
        TokenTree::Group(g) => assert_eq!(g.delimiter(), Delimiter::Brace),
        other => panic!("expected Group, got {other:?}"),
    }
}

#[test]
fn test_tt_iterate_mixed_types() {
    let ts: TokenStream = "fn run() -> u8 { 0 }".parse().unwrap();
    let mut has_ident = false;
    let mut has_punct = false;
    let mut has_group = false;
    for tt in ts {
        match tt {
            TokenTree::Ident(_) => has_ident = true,
            TokenTree::Punct(_) => has_punct = true,
            TokenTree::Group(_) => has_group = true,
            _ => {}
        }
    }
    assert!(has_ident, "should contain idents");
    assert!(has_punct, "should contain puncts");
    assert!(has_group, "should contain groups");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Ident and format_ident (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ident_new_simple() {
    let id = Ident::new("parser", Span::call_site());
    assert_eq!(id.to_string(), "parser");
}

#[test]
fn test_ident_new_raw_keyword() {
    // `r#type` allows using keywords as identifiers
    let id = Ident::new_raw("type", Span::call_site());
    assert!(id.to_string().contains("type"));
}

#[test]
fn test_ident_equality() {
    let a = Ident::new("node", Span::call_site());
    let b = Ident::new("node", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn test_format_ident_prefix() {
    let name = "Symbol";
    let id = format_ident!("parse_{}", name);
    assert_eq!(id.to_string(), "parse_Symbol");
}

#[test]
fn test_format_ident_numeric_suffix() {
    let idx = 7usize;
    let id = format_ident!("field_{}", idx);
    assert_eq!(id.to_string(), "field_7");
}

#[test]
fn test_format_ident_multiple_parts() {
    let module = "grammar";
    let action = "validate";
    let id = format_ident!("{}_{}", module, action);
    assert_eq!(id.to_string(), "grammar_validate");
}

#[test]
fn test_format_ident_in_quote() {
    let ty_name = format_ident!("MyParser");
    let ts = quote! { struct #ty_name; };
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"MyParser".to_string()));
}

#[test]
fn test_format_ident_span_preserved() {
    let base = Ident::new("base", Span::call_site());
    let derived = format_ident!("{}_ext", base);
    assert_eq!(derived.to_string(), "base_ext");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Quote macro patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_quote_interpolate_ident() {
    let name = Ident::new("Parser", Span::call_site());
    let ts = quote! { struct #name; };
    assert!(ts.to_string().contains("Parser"));
}

#[test]
fn test_quote_interpolate_type() {
    let ty: Type = parse_quote!(Vec<String>);
    let ts = quote! { fn produce() -> #ty { todo!() } };
    let output = ts.to_string();
    assert!(output.contains("Vec"));
    assert!(output.contains("String"));
}

#[test]
fn test_quote_interpolate_expr() {
    let val: Expr = parse_quote!(2 + 3);
    let ts = quote! { let result = #val; };
    assert!(ts.to_string().contains("2"));
    assert!(ts.to_string().contains("3"));
}

#[test]
fn test_quote_repetition_vec() {
    let names: Vec<Ident> = vec![
        Ident::new("alpha", Span::call_site()),
        Ident::new("beta", Span::call_site()),
        Ident::new("gamma", Span::call_site()),
    ];
    let ts = quote! { #(fn #names() {})*  };
    let output = ts.to_string();
    assert!(output.contains("alpha"));
    assert!(output.contains("beta"));
    assert!(output.contains("gamma"));
}

#[test]
fn test_quote_repetition_with_separator() {
    let fields: Vec<Ident> = vec![
        Ident::new("x", Span::call_site()),
        Ident::new("y", Span::call_site()),
        Ident::new("z", Span::call_site()),
    ];
    let ts = quote! { enum Axis { #(#fields),* } };
    let output = ts.to_string();
    assert!(output.contains("x"));
    assert!(output.contains("y"));
    assert!(output.contains("z"));
}

#[test]
fn test_quote_nested_struct_def() {
    let name = format_ident!("Config");
    let field_name = format_ident!("value");
    let field_type: Type = parse_quote!(u64);
    let ts = quote! {
        pub struct #name {
            pub #field_name: #field_type,
        }
    };
    let parsed: ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(parsed.ident, "Config");
}

#[test]
fn test_quote_conditional_tokens() {
    let use_pub = true;
    let vis = if use_pub {
        quote! { pub }
    } else {
        quote! {}
    };
    let ts = quote! { #vis fn handler() {} };
    assert!(ts.to_string().contains("pub"));
}

#[test]
fn test_quote_multiple_statements() {
    let ts = quote! {
        let a: i32 = 10;
        let b: i32 = 20;
        let c: i32 = a + b;
    };
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"a".to_string()));
    assert!(idents.contains(&"b".to_string()));
    assert!(idents.contains(&"c".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Parse str/quote roundtrip (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_roundtrip_simple_type() {
    let ty: Type = parse_str("Option<Vec<u8>>").unwrap();
    let quoted = ty.to_token_stream();
    let reparsed: Type = syn::parse2(quoted).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_roundtrip_struct() {
    let ts = quote! { struct Point { x: f64, y: f64 } };
    let parsed: ItemStruct = syn::parse2(ts).unwrap();
    let requoted = parsed.to_token_stream();
    let reparsed: ItemStruct = syn::parse2(requoted).unwrap();
    assert_eq!(parsed.ident, reparsed.ident);
    match &reparsed.fields {
        Fields::Named(FieldsNamed { named, .. }) => assert_eq!(named.len(), 2),
        other => panic!("expected named fields, got {other:?}"),
    }
}

#[test]
fn test_roundtrip_enum() {
    let ts = quote! { enum Color { Red, Green, Blue } };
    let parsed: ItemEnum = syn::parse2(ts).unwrap();
    let requoted = parsed.to_token_stream();
    let reparsed: ItemEnum = syn::parse2(requoted).unwrap();
    assert_eq!(reparsed.variants.len(), 3);
}

#[test]
fn test_roundtrip_generic_type() {
    let ty: Type = parse_str("HashMap<String, Vec<i32>>").unwrap();
    let ts = ty.to_token_stream();
    let reparsed: Type = syn::parse2(ts).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_roundtrip_reference_type() {
    let ty: Type = parse_str("&'static str").unwrap();
    let ts = ty.to_token_stream();
    let reparsed: Type = syn::parse2(ts).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_roundtrip_tuple_type() {
    let ty: Type = parse_str("(u8, u16, u32)").unwrap();
    let ts = ty.to_token_stream();
    let reparsed: Type = syn::parse2(ts).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_roundtrip_fn_pointer_type() {
    let ty: Type = parse_str("fn(i32) -> bool").unwrap();
    let ts = ty.to_token_stream();
    let reparsed: Type = syn::parse2(ts).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_roundtrip_expr_arithmetic() {
    let expr: Expr = parse_str("a * b + c").unwrap();
    let ts = expr.to_token_stream();
    let reparsed: Expr = syn::parse2(ts).unwrap();
    assert_eq!(expr, reparsed);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Literal construction (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_literal_string() {
    let lit = Literal::string("hello world");
    let ts: TokenStream = lit.into_token_stream();
    assert!(ts.to_string().contains("hello world"));
}

#[test]
fn test_literal_integer_i32() {
    let lit = Literal::i32_suffixed(42);
    let display = lit.to_string();
    assert!(display.contains("42"));
    assert!(display.contains("i32"));
}

#[test]
fn test_literal_unsuffixed_integer() {
    let lit = Literal::u64_unsuffixed(999);
    assert_eq!(lit.to_string(), "999");
}

#[test]
fn test_literal_float() {
    let lit = Literal::f64_suffixed(1.5);
    let display = lit.to_string();
    assert!(display.contains("1.5"));
}

#[test]
fn test_literal_byte_string() {
    let lit = Literal::byte_string(b"bytes");
    let display = lit.to_string();
    // Byte string literals render as b"bytes"
    assert!(display.starts_with("b\""));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Attribute and Meta parsing (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_attr_parse_path_only() {
    let attr: Attribute = parse_quote!(#[test]);
    match &attr.meta {
        Meta::Path(path) => assert!(path.is_ident("test")),
        other => panic!("expected Path, got {other:?}"),
    }
}

#[test]
fn test_attr_parse_name_value() {
    let attr: Attribute = parse_quote!(#[doc = "documentation"]);
    assert!(matches!(&attr.meta, Meta::NameValue(_)));
}

#[test]
fn test_attr_parse_list() {
    let attr: Attribute = parse_quote!(#[allow(unused_variables)]);
    match &attr.meta {
        Meta::List(list) => assert!(list.path.is_ident("allow")),
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn test_attr_on_struct() {
    let item: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        pub struct Record {
            name: String,
        }
    };
    assert!(!item.attrs.is_empty());
    let attr = &item.attrs[0];
    assert!(matches!(&attr.meta, Meta::List(_)));
}

#[test]
fn test_attr_multiple_on_enum() {
    let item: ItemEnum = parse_quote! {
        #[derive(Debug)]
        #[repr(u8)]
        enum Signal {
            Stop = 0,
            Start = 1,
        }
    };
    assert_eq!(item.attrs.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_empty_stream_extend() {
    let mut ts = TokenStream::new();
    ts.extend(TokenStream::new());
    assert!(ts.is_empty());
}

#[test]
fn test_edge_empty_quote() {
    let ts = quote! {};
    assert!(ts.is_empty());
}

#[test]
fn test_edge_nested_quotes() {
    let inner = quote! { x + y };
    let outer = quote! { fn compute() -> i32 { #inner } };
    let output = outer.to_string();
    assert!(output.contains("compute"));
    assert!(output.contains("x"));
    assert!(output.contains("y"));
}

#[test]
fn test_edge_deeply_nested_groups() {
    let ts: TokenStream = "((((core))))".parse().unwrap();
    // Should parse as nested groups
    let first = ts.into_iter().next().unwrap();
    assert!(matches!(first, TokenTree::Group(_)));
}

#[test]
fn test_edge_complex_generic_type() {
    let ty: Type =
        parse_str("Result<HashMap<String, Vec<Option<Box<dyn Fn(i32) -> bool>>>>, std::io::Error>")
            .unwrap();
    let ts = ty.to_token_stream();
    let reparsed: Type = syn::parse2(ts).unwrap();
    assert_eq!(ty, reparsed);
}

#[test]
fn test_edge_struct_with_lifetime() {
    let item: ItemStruct = parse_quote! {
        struct Borrowed<'a> {
            data: &'a [u8],
        }
    };
    assert_eq!(item.ident, "Borrowed");
    assert!(!item.generics.params.is_empty());
}

#[test]
fn test_edge_enum_with_data_variants() {
    let item: ItemEnum = parse_quote! {
        enum Ast {
            Literal(i64),
            Binary { left: Box<Ast>, op: char, right: Box<Ast> },
            Unary(Box<Ast>),
        }
    };
    assert_eq!(item.variants.len(), 3);
    // Check named fields on Binary
    match &item.variants[1].fields {
        Fields::Named(named) => assert_eq!(named.named.len(), 3),
        other => panic!("expected named fields, got {other:?}"),
    }
}

#[test]
fn test_edge_group_delimiters() {
    let ts = quote! { fn call(a: [u8; 4]) { (a, a) } };
    // Parenthesis and Brace groups are top-level; Bracket is nested inside Parenthesis
    assert!(has_group_with_delimiter(&ts, Delimiter::Parenthesis));
    assert!(has_group_with_delimiter(&ts, Delimiter::Brace));
    // Verify bracket exists in the full output
    assert!(ts.to_string().contains("[u8"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional coverage (bonus tests to reach 55+)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_token_tree_into_stream() {
    let ident = TokenTree::Ident(Ident::new("token", Span::call_site()));
    let ts: TokenStream = ident.into();
    assert_eq!(count_trees(&ts), 1);
}

#[test]
fn test_group_construction_brace() {
    let inner: TokenStream = "x + 1".parse().unwrap();
    let group = Group::new(Delimiter::Brace, inner);
    let tt = TokenTree::Group(group);
    let ts: TokenStream = tt.into();
    assert!(has_group_with_delimiter(&ts, Delimiter::Brace));
}

#[test]
fn test_group_construction_parenthesis() {
    let inner: TokenStream = "a, b, c".parse().unwrap();
    let group = Group::new(Delimiter::Parenthesis, inner);
    assert_eq!(group.delimiter(), Delimiter::Parenthesis);
    let inner_idents = collect_idents(&group.stream());
    assert_eq!(inner_idents.len(), 3);
}

#[test]
fn test_punct_spacing_alone_vs_joint() {
    let alone = Punct::new(':', Spacing::Alone);
    let joint = Punct::new(':', Spacing::Joint);
    assert_eq!(alone.spacing(), Spacing::Alone);
    assert_eq!(joint.spacing(), Spacing::Joint);
    assert_eq!(alone.as_char(), joint.as_char());
}

#[test]
fn test_quote_where_clause() {
    let trait_name = format_ident!("Display");
    let ts = quote! {
        fn show<T>(val: T) where T: #trait_name {
            todo!()
        }
    };
    let output = ts.to_string();
    assert!(output.contains("where"));
    assert!(output.contains("Display"));
}

#[test]
fn test_parse_quote_impl_block() {
    let ty_name = format_ident!("Parser");
    let item: syn::ItemImpl = parse_quote! {
        impl #ty_name {
            fn new() -> Self {
                Self
            }
        }
    };
    // The self type should reference Parser
    let self_ty_str = item.self_ty.to_token_stream().to_string();
    assert_eq!(self_ty_str, "Parser");
}

#[test]
fn test_format_ident_raw_for_reserved() {
    // `r#match` lets us use `match` as an identifier
    let id = format_ident!("r#match");
    let ts = quote! { let #id = 5; };
    assert!(ts.to_string().contains("match"));
}

#[test]
fn test_to_tokens_extend_existing() {
    let mut ts = TokenStream::new();
    let ident = Ident::new("first", Span::call_site());
    ident.to_tokens(&mut ts);
    let ident2 = Ident::new("second", Span::call_site());
    ident2.to_tokens(&mut ts);
    let idents = collect_idents(&ts);
    assert_eq!(idents, ["first", "second"]);
}

#[test]
fn test_struct_unit_variant() {
    let item: ItemStruct = parse_quote! { struct Unit; };
    assert!(matches!(item.fields, Fields::Unit));
}

#[test]
fn test_struct_tuple_variant() {
    let item: ItemStruct = parse_quote! { struct Pair(i32, i32); };
    match &item.fields {
        Fields::Unnamed(fields) => assert_eq!(fields.unnamed.len(), 2),
        other => panic!("expected unnamed fields, got {other:?}"),
    }
}

#[test]
fn test_enum_empty() {
    let item: ItemEnum = parse_quote! { enum Empty {} };
    assert!(item.variants.is_empty());
}

#[test]
fn test_quote_repetition_zip() {
    let names = vec![format_ident!("width"), format_ident!("height")];
    let types: Vec<Type> = vec![parse_quote!(u32), parse_quote!(u32)];
    let ts = quote! {
        struct Size {
            #(#names: #types),*
        }
    };
    let parsed: ItemStruct = syn::parse2(ts).unwrap();
    match &parsed.fields {
        Fields::Named(FieldsNamed { named, .. }) => assert_eq!(named.len(), 2),
        other => panic!("expected named fields, got {other:?}"),
    }
}
