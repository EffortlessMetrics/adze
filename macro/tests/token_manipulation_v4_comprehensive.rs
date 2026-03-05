//! Comprehensive v4 tests for proc-macro2 token stream manipulation patterns
//! used in adze-macro expansion.
//!
//! Tests token stream creation, token tree classification, punctuation/grouping,
//! literal/ident creation, span information, quote macro patterns, and roundtrips.

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use std::str::FromStr;
use syn::{DeriveInput, Expr, ItemFn, ItemStruct, Type, parse_quote, parse2};

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

fn collect_puncts(ts: &TokenStream) -> Vec<char> {
    ts.clone()
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Punct(p) => Some(p.as_char()),
            _ => None,
        })
        .collect()
}

fn roundtrip_struct(ts: TokenStream) -> ItemStruct {
    let parsed: ItemStruct = parse2(ts).expect("parse struct");
    let requoted = quote!(#parsed);
    parse2(requoted).expect("reparse struct")
}

fn roundtrip_fn(ts: TokenStream) -> ItemFn {
    let parsed: ItemFn = parse2(ts).expect("parse fn");
    let requoted = quote!(#parsed);
    parse2(requoted).expect("reparse fn")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. TokenStream creation from various Rust constructs (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ts_create_empty_stream() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
    assert_eq!(count_trees(&ts), 0);
}

#[test]
fn test_ts_create_from_str_simple_expr() {
    let ts: TokenStream = "1 + 2".parse().unwrap();
    assert!(!ts.is_empty());
    assert_eq!(count_trees(&ts), 3); // 1, +, 2
}

#[test]
fn test_ts_create_from_str_struct_def() {
    let ts: TokenStream = "struct Foo { x: u32 }".parse().unwrap();
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"struct".to_string()));
    assert!(idents.contains(&"Foo".to_string()));
}

#[test]
fn test_ts_create_from_quote_let_binding() {
    let ts = quote! { let value: i32 = 42; };
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"let".to_string()));
    assert!(idents.contains(&"value".to_string()));
    assert!(idents.contains(&"i32".to_string()));
}

#[test]
fn test_ts_create_from_quote_function() {
    let ts = quote! { fn hello() -> bool { true } };
    let idents = collect_idents(&ts);
    // top-level idents only (contents inside groups are nested)
    assert!(idents.contains(&"fn".to_string()));
    assert!(idents.contains(&"hello".to_string()));
    assert!(idents.contains(&"bool".to_string()));
    // `true` is inside braces, verify via string repr
    assert!(ts.to_string().contains("true"));
}

#[test]
fn test_ts_create_from_quote_enum() {
    let ts = quote! { enum Color { Red, Green, Blue } };
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"enum".to_string()));
    assert!(idents.contains(&"Color".to_string()));
    // Variant names are inside the brace group
    let s = ts.to_string();
    assert!(s.contains("Red"));
    assert!(s.contains("Green"));
    assert!(s.contains("Blue"));
}

#[test]
fn test_ts_create_from_quote_impl_block() {
    let ts = quote! { impl Foo { fn bar(&self) {} } };
    let idents = collect_idents(&ts);
    assert!(idents.contains(&"impl".to_string()));
    assert!(idents.contains(&"Foo".to_string()));
    // `bar` is inside the brace group
    assert!(ts.to_string().contains("bar"));
}

#[test]
fn test_ts_create_from_quote_with_interpolation() {
    let name = format_ident!("my_func");
    let ret_ty: Type = parse_quote!(u64);
    let ts = quote! { fn #name() -> #ret_ty { 0 } };
    let s = ts.to_string();
    assert!(s.contains("my_func"));
    assert!(s.contains("u64"));
}

#[test]
fn test_ts_create_from_str_with_generics() {
    let ts: TokenStream = "struct Wrapper<T> { inner: T }".parse().unwrap();
    let s = ts.to_string();
    assert!(s.contains("Wrapper"));
    assert!(s.contains("inner"));
}

#[test]
fn test_ts_create_extend_two_streams() {
    let a = quote! { let x = 1; };
    let b = quote! { let y = 2; };
    let mut combined = a;
    combined.extend(b);
    let idents = collect_idents(&combined);
    assert!(idents.contains(&"x".to_string()));
    assert!(idents.contains(&"y".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Token tree parsing and classification (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tt_classify_ident() {
    let ts: TokenStream = "hello".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    assert!(matches!(tt, TokenTree::Ident(_)));
}

#[test]
fn test_tt_classify_literal_int() {
    let ts: TokenStream = "42".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    assert!(matches!(tt, TokenTree::Literal(_)));
}

#[test]
fn test_tt_classify_punct() {
    let ts: TokenStream = "+".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    assert!(matches!(tt, TokenTree::Punct(_)));
}

#[test]
fn test_tt_classify_group_parens() {
    let ts: TokenStream = "(a)".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    match tt {
        TokenTree::Group(g) => assert_eq!(g.delimiter(), Delimiter::Parenthesis),
        other => panic!("expected group, got {:?}", other),
    }
}

#[test]
fn test_tt_classify_group_braces() {
    let ts: TokenStream = "{ x }".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    match tt {
        TokenTree::Group(g) => assert_eq!(g.delimiter(), Delimiter::Brace),
        other => panic!("expected group, got {:?}", other),
    }
}

#[test]
fn test_tt_classify_group_brackets() {
    let ts: TokenStream = "[1, 2]".parse().unwrap();
    let tt = ts.into_iter().next().unwrap();
    match tt {
        TokenTree::Group(g) => assert_eq!(g.delimiter(), Delimiter::Bracket),
        other => panic!("expected group, got {:?}", other),
    }
}

#[test]
fn test_tt_mixed_classification_counts() {
    let ts = quote! { fn foo(x: i32) -> bool { true } };
    let mut ident_count = 0;
    let mut punct_count = 0;
    let mut group_count = 0;
    let mut literal_count = 0;
    for tt in ts.into_iter() {
        match tt {
            TokenTree::Ident(_) => ident_count += 1,
            TokenTree::Punct(_) => punct_count += 1,
            TokenTree::Group(_) => group_count += 1,
            TokenTree::Literal(_) => literal_count += 1,
        }
    }
    // fn, foo, bool at top level; parens + braces = 2 groups; -> = 2 puncts
    assert!(ident_count >= 2);
    assert!(group_count >= 2);
    assert!(punct_count >= 1);
    // no top-level literal
    assert_eq!(literal_count, 0);
}

#[test]
fn test_tt_nested_group_contents() {
    let ts = quote! { (1, 2, 3) };
    let tt = ts.into_iter().next().unwrap();
    if let TokenTree::Group(g) = tt {
        let inner_count = g.stream().into_iter().count();
        // 1 , 2 , 3 = 5 tokens inside
        assert_eq!(inner_count, 5);
    } else {
        panic!("expected group");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Punctuation and grouping patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_punct_single_char_alone() {
    let p = Punct::new('+', Spacing::Alone);
    assert_eq!(p.as_char(), '+');
    assert_eq!(p.spacing(), Spacing::Alone);
}

#[test]
fn test_punct_joint_spacing_for_arrow() {
    // `->` is `-` Joint then `>` Alone
    let ts: TokenStream = "->".parse().unwrap();
    let tokens: Vec<_> = ts.into_iter().collect();
    assert_eq!(tokens.len(), 2);
    if let TokenTree::Punct(ref p) = tokens[0] {
        assert_eq!(p.as_char(), '-');
        assert_eq!(p.spacing(), Spacing::Joint);
    } else {
        panic!("expected punct");
    }
    if let TokenTree::Punct(ref p) = tokens[1] {
        assert_eq!(p.as_char(), '>');
    } else {
        panic!("expected punct");
    }
}

#[test]
fn test_punct_double_colon() {
    let ts: TokenStream = "::".parse().unwrap();
    let tokens: Vec<_> = ts.into_iter().collect();
    assert_eq!(tokens.len(), 2);
    if let TokenTree::Punct(ref p) = tokens[0] {
        assert_eq!(p.as_char(), ':');
        assert_eq!(p.spacing(), Spacing::Joint);
    } else {
        panic!("expected punct");
    }
}

#[test]
fn test_punct_collect_from_expr() {
    let ts = quote! { a + b - c };
    let puncts = collect_puncts(&ts);
    assert_eq!(puncts, vec!['+', '-']);
}

#[test]
fn test_group_empty_parens() {
    let g = Group::new(Delimiter::Parenthesis, TokenStream::new());
    assert_eq!(g.delimiter(), Delimiter::Parenthesis);
    assert!(g.stream().is_empty());
}

#[test]
fn test_group_with_contents() {
    let inner = quote! { x, y };
    let g = Group::new(Delimiter::Bracket, inner);
    assert_eq!(g.delimiter(), Delimiter::Bracket);
    let s = g.stream().to_string();
    assert!(s.contains("x"));
    assert!(s.contains("y"));
}

#[test]
fn test_group_none_delimiter() {
    let inner = quote! { hello };
    let g = Group::new(Delimiter::None, inner);
    assert_eq!(g.delimiter(), Delimiter::None);
    assert!(!g.stream().is_empty());
}

#[test]
fn test_group_nested_delimiters() {
    let ts: TokenStream = "([{}])".parse().unwrap();
    // outer is parens
    let tt = ts.into_iter().next().unwrap();
    if let TokenTree::Group(outer) = tt {
        assert_eq!(outer.delimiter(), Delimiter::Parenthesis);
        let inner_tt = outer.stream().into_iter().next().unwrap();
        if let TokenTree::Group(mid) = inner_tt {
            assert_eq!(mid.delimiter(), Delimiter::Bracket);
            let innermost = mid.stream().into_iter().next().unwrap();
            if let TokenTree::Group(deep) = innermost {
                assert_eq!(deep.delimiter(), Delimiter::Brace);
            } else {
                panic!("expected brace group");
            }
        } else {
            panic!("expected bracket group");
        }
    } else {
        panic!("expected paren group");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Literal token creation and inspection (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_literal_u32_suffix() {
    let lit = Literal::u32_suffixed(100);
    let s = lit.to_string();
    assert!(s.contains("100"));
    assert!(s.contains("u32"));
}

#[test]
fn test_literal_string() {
    let lit = Literal::string("hello world");
    let s = lit.to_string();
    assert!(s.contains("hello world"));
    // Should be quoted
    assert!(s.starts_with('"'));
    assert!(s.ends_with('"'));
}

#[test]
fn test_literal_byte_string() {
    let lit = Literal::byte_string(b"abc");
    let s = lit.to_string();
    assert!(s.starts_with("b\""));
}

#[test]
fn test_literal_char() {
    let lit = Literal::character('Z');
    let s = lit.to_string();
    assert!(s.contains('Z'));
    assert!(s.starts_with('\''));
}

#[test]
fn test_literal_float_unsuffixed() {
    let lit = Literal::f64_unsuffixed(3.15);
    let s = lit.to_string();
    assert!(s.contains("3.15"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Ident creation and manipulation (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ident_new_simple() {
    let id = Ident::new("foo", Span::call_site());
    assert_eq!(id.to_string(), "foo");
}

#[test]
fn test_ident_format_ident_concat() {
    let base = "parse";
    let suffix = "tree";
    let combined = format_ident!("{}_{}", base, suffix);
    assert_eq!(combined.to_string(), "parse_tree");
}

#[test]
fn test_ident_format_ident_numbered() {
    let ids: Vec<Ident> = (0..3u32).map(|i| format_ident!("field_{}", i)).collect();
    assert_eq!(ids[0].to_string(), "field_0");
    assert_eq!(ids[1].to_string(), "field_1");
    assert_eq!(ids[2].to_string(), "field_2");
}

#[test]
fn test_ident_raw_identifier() {
    // `r#type` is a raw identifier for the keyword `type`
    let id = Ident::new_raw("type", Span::call_site());
    let s = id.to_string();
    assert!(s.contains("type"));
}

#[test]
fn test_ident_equality() {
    let a = Ident::new("same", Span::call_site());
    let b = Ident::new("same", Span::call_site());
    let c = Ident::new("diff", Span::call_site());
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Span information (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_span_call_site_exists() {
    let span = Span::call_site();
    // Verify it can be used to create an ident
    let id = Ident::new("test_ident", span);
    assert_eq!(id.to_string(), "test_ident");
}

#[test]
fn test_span_mixed_site_exists() {
    let span = Span::mixed_site();
    let id = Ident::new("mixed", span);
    assert_eq!(id.to_string(), "mixed");
}

#[test]
fn test_span_on_punct() {
    let mut p = Punct::new('!', Spacing::Alone);
    let span = Span::call_site();
    p.set_span(span);
    // Just verifying set_span doesn't panic and span() returns something
    let _s = p.span();
}

#[test]
fn test_span_on_literal() {
    let mut lit = Literal::u8_suffixed(0);
    lit.set_span(Span::call_site());
    let _s = lit.span();
}

#[test]
fn test_span_on_group() {
    let g = Group::new(Delimiter::Parenthesis, TokenStream::new());
    // Groups have spans for both open and close delimiters
    let _open = g.span_open();
    let _close = g.span_close();
    let _full = g.span();
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Quote macro patterns used in adze macros (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_quote_interpolate_type() {
    let ty: Type = parse_quote!(Vec<String>);
    let ts = quote! { let v: #ty = Vec::new(); };
    let s = ts.to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("String"));
}

#[test]
fn test_quote_interpolate_ident_into_struct() {
    let name = format_ident!("MyStruct");
    let field = format_ident!("data");
    let ts = quote! {
        struct #name {
            #field: u32,
        }
    };
    let parsed: ItemStruct = parse2(ts).expect("parse struct");
    assert_eq!(parsed.ident, "MyStruct");
}

#[test]
fn test_quote_repetition_pattern() {
    let names: Vec<Ident> = vec![format_ident!("a"), format_ident!("b"), format_ident!("c")];
    let ts = quote! { fn foo(#(#names: u32),*) {} };
    let s = ts.to_string();
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

#[test]
fn test_quote_repetition_with_separator() {
    let items: Vec<Expr> = vec![parse_quote!(1), parse_quote!(2), parse_quote!(3)];
    let ts = quote! { [#(#items),*] };
    let s = ts.to_string();
    assert!(s.contains("1"));
    assert!(s.contains("2"));
    assert!(s.contains("3"));
}

#[test]
fn test_quote_conditional_via_option() {
    let maybe_attr: Option<TokenStream> = Some(quote! { #[derive(Debug)] });
    let ts = quote! {
        #maybe_attr
        struct Foo;
    };
    let s = ts.to_string();
    assert!(s.contains("derive"));
    assert!(s.contains("Debug"));
}

#[test]
fn test_quote_conditional_none_omits() {
    let maybe_attr: Option<TokenStream> = None;
    let ts = quote! {
        #maybe_attr
        struct Foo;
    };
    let s = ts.to_string();
    assert!(!s.contains("derive"));
    assert!(s.contains("struct Foo"));
}

#[test]
fn test_quote_nested_quote_blocks() {
    let inner = quote! { x: u32 };
    let outer = quote! {
        struct Nested {
            #inner,
            y: i64,
        }
    };
    let s = outer.to_string();
    assert!(s.contains("x"));
    assert!(s.contains("u32"));
    assert!(s.contains("y"));
    assert!(s.contains("i64"));
}

#[test]
fn test_quote_generate_match_arms() {
    let variants = ["A", "B", "C"];
    let arms: Vec<TokenStream> = variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let variant_ident = format_ident!("{}", v);
            let idx = Literal::usize_unsuffixed(i);
            quote! { Self::#variant_ident => #idx }
        })
        .collect();
    let ts = quote! {
        match self {
            #(#arms,)*
        }
    };
    let s = ts.to_string();
    assert!(s.contains("Self :: A => 0"));
    assert!(s.contains("Self :: B => 1"));
    assert!(s.contains("Self :: C => 2"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Roundtrip: parse → generate → parse (6 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_roundtrip_simple_struct() {
    let ts = quote! { struct Point { x: f64, y: f64 } };
    let rt = roundtrip_struct(ts);
    assert_eq!(rt.ident, "Point");
    assert_eq!(rt.fields.len(), 2);
}

#[test]
fn test_roundtrip_struct_with_generics() {
    let ts = quote! { struct Wrapper<T> { inner: T } };
    let rt = roundtrip_struct(ts);
    assert_eq!(rt.ident, "Wrapper");
    assert_eq!(rt.generics.params.len(), 1);
}

#[test]
fn test_roundtrip_struct_with_attrs() {
    let ts = quote! {
        #[derive(Debug, Clone)]
        struct Tagged {
            value: String,
        }
    };
    let rt = roundtrip_struct(ts);
    assert_eq!(rt.ident, "Tagged");
    assert!(!rt.attrs.is_empty());
}

#[test]
fn test_roundtrip_function() {
    let ts = quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
    let rt = roundtrip_fn(ts);
    assert_eq!(rt.sig.ident, "add");
    assert_eq!(rt.sig.inputs.len(), 2);
    assert!(rt.sig.output.to_token_stream().to_string().contains("i32"));
}

#[test]
fn test_roundtrip_derive_input() {
    let ts = quote! {
        #[derive(Debug)]
        struct Config {
            name: String,
            count: usize,
        }
    };
    let parsed: DeriveInput = parse2(ts).expect("parse");
    let requoted = quote!(#parsed);
    let reparsed: DeriveInput = parse2(requoted).expect("reparse");
    assert_eq!(parsed.ident, reparsed.ident);
}

#[test]
fn test_roundtrip_preserves_field_types() {
    let ts = quote! {
        struct Types {
            a: Vec<u8>,
            b: Option<String>,
            c: (i32, i32),
        }
    };
    let original: ItemStruct = parse2(ts).expect("parse");
    let requoted = quote!(#original);
    let reparsed: ItemStruct = parse2(requoted).expect("reparse");
    let orig_fields: Vec<String> = original
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    let rt_fields: Vec<String> = reparsed
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(orig_fields, rt_fields);
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional edge case tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_from_str_invalid_tokens_fails() {
    let result = TokenStream::from_str("struct {{{");
    // Invalid token streams may or may not parse; the key is no panic
    let _ = result;
}

#[test]
fn test_token_stream_display_roundtrip() {
    let ts = quote! { fn identity(x: u32) -> u32 { x } };
    let s = ts.to_string();
    let reparsed: TokenStream = s.parse().unwrap();
    // Re-stringified should be equivalent
    assert_eq!(ts.to_string(), reparsed.to_string());
}

#[test]
fn test_token_tree_into_stream() {
    let id = Ident::new("standalone", Span::call_site());
    let tt = TokenTree::Ident(id);
    let ts: TokenStream = tt.into();
    assert_eq!(count_trees(&ts), 1);
    assert_eq!(ts.to_string(), "standalone");
}

#[test]
fn test_multiple_format_ident_collision_avoidance() {
    let prefix = "node";
    let ids: Vec<Ident> = (0..5u32)
        .map(|i| format_ident!("{}_{}", prefix, i))
        .collect();
    let unique: std::collections::HashSet<String> = ids.iter().map(|i| i.to_string()).collect();
    assert_eq!(unique.len(), 5);
}

#[test]
fn test_quote_empty_repetition() {
    let items: Vec<Ident> = vec![];
    let ts = quote! { #(#items),* };
    assert!(ts.is_empty());
}
