//! Tests for syn/quote parsing patterns used in adze-macro expansion.
//!
//! Covers NameValueExpr, FieldThenParams, type extraction helpers,
//! quote generation, syn type parsing, attribute argument validation,
//! TokenStream manipulation, and edge cases.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use std::collections::HashSet;
use syn::{
    DeriveInput, ItemEnum, ItemMod, ItemStruct, Type, parse_quote, parse2, punctuated::Punctuated,
    token::Comma,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn ts_contains(ts: &TokenStream, needle: &str) -> bool {
    ts.to_string().contains(needle)
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Parse attribute expressions — NameValueExpr (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nve_parse_string_value() {
    let nve: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nve.path, "text");
}

#[test]
fn nve_parse_integer_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(nve.path, "precedence");
}

#[test]
fn nve_parse_bool_value() {
    let nve: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nve.path, "non_empty");
}

#[test]
fn nve_parse_path_value() {
    let nve: NameValueExpr = parse_quote!(transform = std::str::from_utf8);
    assert_eq!(nve.path, "transform");
}

#[test]
fn nve_parse_closure_value() {
    let nve: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nve.path, "transform");
    // Expr should roundtrip through token stream
    let expr_str = nve.expr.to_token_stream().to_string();
    assert!(expr_str.contains("parse"));
}

#[test]
fn nve_parse_pattern_value() {
    let nve: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nve.path, "pattern");
}

#[test]
fn nve_parse_terminated_list() {
    let list: Punctuated<NameValueExpr, Comma> = parse_quote!(text = "+", precedence = 1);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].path, "text");
    assert_eq!(list[1].path, "precedence");
}

#[test]
fn nve_parse_single_item_list() {
    let list: Punctuated<NameValueExpr, Comma> = parse_quote!(name = "foo");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].path, "name");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Parse field parameters — FieldThenParams (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ftp_parse_type_only() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(type_to_string(&ftp.field.ty), "String");
}

#[test]
fn ftp_parse_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.params.is_empty());
    assert_eq!(type_to_string(&ftp.field.ty), "()");
}

#[test]
fn ftp_parse_type_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(u32, transform = |v| v.parse().unwrap());
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "transform");
}

#[test]
fn ftp_parse_type_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(Type, name = "test", value = 42);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path, "name");
    assert_eq!(ftp.params[1].path, "value");
}

#[test]
fn ftp_parse_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    assert!(ftp.params.is_empty());
    assert!(type_to_string(&ftp.field.ty).contains("Vec"));
}

#[test]
fn ftp_parse_box_type() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    assert!(ftp.params.is_empty());
    assert!(type_to_string(&ftp.field.ty).contains("Box"));
}

#[test]
fn ftp_parse_with_attributes() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = "+")]
        ()
    );
    assert!(!ftp.field.attrs.is_empty());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_field_preserves_visibility() {
    // Unnamed fields parsed via Field::parse_unnamed have inherited visibility
    let ftp: FieldThenParams = parse_quote!(i32);
    assert!(matches!(ftp.field.vis, syn::Visibility::Inherited));
    assert!(ftp.field.ident.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Quote generation patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn quote_struct_with_fields() {
    let name = format_ident!("MyStruct");
    let field_name = format_ident!("value");
    let field_type: Type = parse_quote!(i32);
    let ts = quote! {
        struct #name {
            #field_name: #field_type,
        }
    };
    let parsed: ItemStruct = parse2(ts).expect("valid struct");
    assert_eq!(parsed.ident, "MyStruct");
    assert_eq!(parsed.fields.len(), 1);
}

#[test]
fn quote_enum_with_variants() {
    let name = format_ident!("Token");
    let ts = quote! {
        enum #name {
            Plus,
            Minus,
            Number(i32),
        }
    };
    let parsed: ItemEnum = parse2(ts).expect("valid enum");
    assert_eq!(parsed.ident, "Token");
    assert_eq!(parsed.variants.len(), 3);
}

#[test]
fn quote_impl_block() {
    let ty = format_ident!("Parser");
    let ts = quote! {
        impl #ty {
            fn parse(&self) -> Result<(), String> {
                Ok(())
            }
        }
    };
    assert!(ts_contains(&ts, "impl"));
    assert!(ts_contains(&ts, "Parser"));
    assert!(ts_contains(&ts, "parse"));
}

#[test]
fn quote_repetition() {
    let fields: Vec<Ident> = vec![format_ident!("a"), format_ident!("b"), format_ident!("c")];
    let ts = quote! {
        struct Generated {
            #(#fields: String,)*
        }
    };
    let parsed: ItemStruct = parse2(ts).expect("valid struct");
    assert_eq!(parsed.fields.len(), 3);
}

#[test]
fn quote_conditional_tokens() {
    let include_debug = true;
    let debug_attr = if include_debug {
        quote!(#[derive(Debug)])
    } else {
        quote!()
    };
    let ts = quote! {
        #debug_attr
        struct Foo;
    };
    assert!(ts_contains(&ts, "Debug"));
}

#[test]
fn quote_nested_interpolation() {
    let inner_name = format_ident!("Inner");
    let outer_name = format_ident!("Outer");
    let inner = quote!(struct #inner_name;);
    let ts = quote! {
        mod #outer_name {
            #inner
        }
    };
    assert!(ts_contains(&ts, "Inner"));
    assert!(ts_contains(&ts, "Outer"));
}

#[test]
fn quote_format_ident_suffix() {
    let base = "Node";
    let ident = format_ident!("{}Visitor", base);
    assert_eq!(ident, "NodeVisitor");
}

#[test]
fn quote_attribute_generation() {
    let attr_name = format_ident!("leaf");
    let ts = quote! {
        #[adze::#attr_name(text = "+")]
        ()
    };
    let s = ts.to_string();
    assert!(s.contains("adze"));
    assert!(s.contains("leaf"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Syn type parsing (7 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn syn_parse_simple_type() {
    let ty: Type = parse_quote!(String);
    assert_eq!(type_to_string(&ty), "String");
}

#[test]
fn syn_parse_generic_type() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert!(type_to_string(&ty).contains("Vec"));
    assert!(type_to_string(&ty).contains("i32"));
}

#[test]
fn syn_parse_nested_generic_type() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let s = type_to_string(&ty);
    assert!(s.contains("Option"));
    assert!(s.contains("Vec"));
    assert!(s.contains("String"));
}

#[test]
fn syn_parse_reference_type() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(type_to_string(&ty), "& str");
}

#[test]
fn syn_parse_box_type() {
    let ty: Type = parse_quote!(Box<dyn std::error::Error>);
    let s = type_to_string(&ty);
    assert!(s.contains("Box"));
    assert!(s.contains("dyn"));
}

#[test]
fn syn_parse_tuple_type() {
    let ty: Type = parse_quote!((i32, String, bool));
    let s = type_to_string(&ty);
    assert!(s.contains("i32"));
    assert!(s.contains("String"));
    assert!(s.contains("bool"));
}

#[test]
fn syn_parse_fn_pointer_type() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let s = type_to_string(&ty);
    assert!(s.contains("fn"));
    assert!(s.contains("bool"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Attribute argument validation (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn attr_leaf_with_text() {
    let item: DeriveInput = parse_quote! {
        #[adze::leaf(text = "+")]
        struct Plus;
    };
    let attr = &item.attrs[0];
    assert!(attr.path().segments.last().unwrap().ident == "leaf");
}

#[test]
fn attr_leaf_with_pattern() {
    let item: DeriveInput = parse_quote! {
        #[adze::leaf(pattern = r"\d+")]
        struct Number;
    };
    let attr = &item.attrs[0];
    let args: Punctuated<NameValueExpr, Comma> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .expect("valid args");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].path, "pattern");
}

#[test]
fn attr_leaf_with_text_and_transform() {
    let item: DeriveInput = parse_quote! {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        struct Number;
    };
    let attr = &item.attrs[0];
    let args: Punctuated<NameValueExpr, Comma> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .expect("valid args");
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].path, "pattern");
    assert_eq!(args[1].path, "transform");
}

#[test]
fn attr_prec_left_value() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(i32),
        }
    };
    let variant = &item.variants[0];
    let attr = &variant.attrs[0];
    assert!(attr.path().segments.last().unwrap().ident == "prec_left");
}

#[test]
fn attr_repeat_non_empty() {
    // Parse repeat attribute via a struct with the field
    let item: ItemStruct = parse_quote! {
        struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<i32>,
        }
    };
    let field = item.fields.iter().next().unwrap();
    let attr = &field.attrs[0];
    let args: Punctuated<NameValueExpr, Comma> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .expect("valid args");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].path, "non_empty");
}

#[test]
fn attr_grammar_name_is_string_literal() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {}
    };
    let attr = &module.attrs[0];
    let s = attr.to_token_stream().to_string();
    assert!(s.contains("arithmetic"));
}

#[test]
fn attr_multiple_on_single_item() {
    let item: DeriveInput = parse_quote! {
        #[adze::word]
        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
        struct Identifier;
    };
    assert_eq!(item.attrs.len(), 2);
}

#[test]
fn attr_extra_no_args() {
    let item: DeriveInput = parse_quote! {
        #[adze::extra]
        struct Whitespace;
    };
    let attr = &item.attrs[0];
    assert!(attr.path().segments.last().unwrap().ident == "extra");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. TokenStream manipulation (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ts_empty_stream() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn ts_from_str() {
    let ts: TokenStream = "struct Foo;".parse().expect("valid tokens");
    assert!(!ts.is_empty());
    let parsed: ItemStruct = parse2(ts).expect("valid struct");
    assert_eq!(parsed.ident, "Foo");
}

#[test]
fn ts_extend_two_streams() {
    let a = quote!(
        struct A;
    );
    let b = quote!(
        struct B;
    );
    let mut combined = TokenStream::new();
    combined.extend(a);
    combined.extend(b);
    let s = combined.to_string();
    assert!(s.contains('A'));
    assert!(s.contains('B'));
}

#[test]
fn ts_iterate_token_trees() {
    let ts = quote!(
        fn foo() {}
    );
    let trees: Vec<TokenTree> = ts.into_iter().collect();
    // fn, foo, (), {}
    assert_eq!(trees.len(), 4);
    assert!(matches!(&trees[0], TokenTree::Ident(id) if id == "fn"));
}

#[test]
fn ts_construct_from_parts() {
    let mut ts = TokenStream::new();
    ts.extend([
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("x", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::i32_suffixed(42)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
    let s = ts.to_string();
    assert!(s.contains("let"));
    assert!(s.contains("42"));
}

#[test]
fn ts_group_delimiters() {
    let inner = quote!(a, b, c);
    let paren = Group::new(Delimiter::Parenthesis, inner.clone());
    let bracket = Group::new(Delimiter::Bracket, inner.clone());
    let brace = Group::new(Delimiter::Brace, inner);
    assert_eq!(paren.delimiter(), Delimiter::Parenthesis);
    assert_eq!(bracket.delimiter(), Delimiter::Bracket);
    assert_eq!(brace.delimiter(), Delimiter::Brace);
}

#[test]
fn ts_literal_types() {
    let string_lit = Literal::string("hello");
    let int_lit = Literal::i64_suffixed(100);
    let float_lit = Literal::f64_suffixed(2.72);

    assert!(string_lit.to_string().contains("hello"));
    assert!(int_lit.to_string().contains("100"));
    assert!(float_lit.to_string().contains("2.72"));
}

#[test]
fn ts_roundtrip_through_string() {
    let original = quote! {
        struct Foo {
            x: i32,
        }
    };
    let as_string = original.to_string();
    let reparsed: TokenStream = as_string.parse().expect("reparse");
    let s1: ItemStruct = parse2(original).expect("parse original");
    let s2: ItemStruct = parse2(reparsed).expect("parse reparsed");
    assert_eq!(s1.ident, s2.ident);
    assert_eq!(s1.fields.len(), s2.fields.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_extract_inner_from_non_path_type() {
    let skip = skip_set(&[]);
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "& str");
}

#[test]
fn edge_extract_inner_vec_through_box() {
    let skip = skip_set(&["Box"]);
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_to_string(&inner), "u8");
}

#[test]
fn edge_extract_inner_no_match() {
    let skip = skip_set(&["Box"]);
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "HashMap < String , i32 >");
}

#[test]
fn edge_filter_nested_box_arc() {
    let skip = skip_set(&["Box", "Arc"]);
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn edge_filter_no_skip_returns_original() {
    let skip = skip_set(&[]);
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(type_to_string(&filtered), "Box < String >");
}

#[test]
fn edge_wrap_leaf_simple() {
    let skip = skip_set(&[]);
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn edge_wrap_leaf_skips_vec() {
    let skip = skip_set(&["Vec"]);
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn edge_wrap_leaf_nested_option_vec() {
    let skip = skip_set(&["Option", "Vec"]);
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Option < Vec < adze :: WithLeaf < i32 > > >"
    );
}
