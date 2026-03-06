//! Comprehensive tests for macro token extraction and grammar annotation patterns.
//!
//! Tests syn parsing of struct/enum/attribute patterns, token stream manipulation,
//! type path parsing, visibility, where clauses, trait bounds, and error handling.

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Expr, Field, Fields, GenericParam, ItemEnum, ItemFn, ItemMod,
    ItemStruct, ItemTrait, Lifetime, Meta, Pat, ReturnType, Type, TypePath, Visibility,
    parse_quote, parse2,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn field_types(s: &ItemStruct) -> Vec<String> {
    match &s.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unnamed(f) => f
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unit => vec![],
    }
}

fn field_names(s: &ItemStruct) -> Vec<String> {
    match &s.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect(),
        _ => vec![],
    }
}

fn attr_paths(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .map(|a| a.path().to_token_stream().to_string())
        .collect()
}

// =============================================================================
// Section 1: Struct patterns for AST nodes (tests 1–10)
// =============================================================================

#[test]
fn struct_unit() {
    let s = parse_struct(quote! { struct Unit; });
    assert_eq!(s.ident.to_string(), "Unit");
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn struct_empty_braces() {
    let s = parse_struct(quote! { struct Empty {} });
    assert_eq!(s.ident.to_string(), "Empty");
    match &s.fields {
        Fields::Named(n) => assert_eq!(n.named.len(), 0),
        _ => panic!("expected named fields"),
    }
}

#[test]
fn struct_single_field() {
    let s = parse_struct(quote! { struct Wrapper { value: i32 } });
    assert_eq!(field_names(&s), vec!["value"]);
}

#[test]
fn struct_multiple_fields() {
    let s = parse_struct(quote! {
        struct BinaryExpr {
            left: Box<Expr>,
            op: Token,
            right: Box<Expr>,
        }
    });
    assert_eq!(field_names(&s), vec!["left", "op", "right"]);
}

#[test]
fn struct_tuple_fields() {
    let s = parse_struct(quote! { struct Pair(i32, String); });
    match &s.fields {
        Fields::Unnamed(u) => assert_eq!(u.unnamed.len(), 2),
        _ => panic!("expected unnamed"),
    }
}

#[test]
fn struct_with_box_type() {
    let s = parse_struct(quote! { struct Node { child: Box<Node> } });
    let types = field_types(&s);
    assert!(types[0].contains("Box"));
    assert!(types[0].contains("Node"));
}

#[test]
fn struct_with_vec_type() {
    let s = parse_struct(quote! { struct List { items: Vec<Item> } });
    let types = field_types(&s);
    assert!(types[0].contains("Vec"));
}

#[test]
fn struct_with_option_type() {
    let s = parse_struct(quote! { struct MaybeValue { inner: Option<i32> } });
    let types = field_types(&s);
    assert!(types[0].contains("Option"));
}

#[test]
fn struct_with_nested_generics() {
    let s = parse_struct(quote! { struct Deep { data: Option<Vec<Box<Node>>> } });
    let types = field_types(&s);
    assert!(types[0].contains("Option"));
    assert!(types[0].contains("Vec"));
    assert!(types[0].contains("Box"));
}

#[test]
fn struct_with_lifetime() {
    let s = parse_struct(quote! { struct Ref<'a> { data: &'a str } });
    assert_eq!(s.generics.params.len(), 1);
    match s.generics.params.first().unwrap() {
        GenericParam::Lifetime(lt) => assert_eq!(lt.lifetime.ident.to_string(), "a"),
        _ => panic!("expected lifetime"),
    }
}

// =============================================================================
// Section 2: Enum patterns for token types (tests 11–20)
// =============================================================================

#[test]
fn enum_simple_variants() {
    let e = parse_enum(quote! {
        enum Token {
            Plus,
            Minus,
            Star,
        }
    });
    assert_eq!(e.variants.len(), 3);
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, vec!["Plus", "Minus", "Star"]);
}

#[test]
fn enum_variant_with_data() {
    let e = parse_enum(quote! {
        enum Expr {
            Literal(i64),
            Binary { left: Box<Expr>, right: Box<Expr> },
        }
    });
    assert_eq!(e.variants.len(), 2);
    match &e.variants[0].fields {
        Fields::Unnamed(u) => assert_eq!(u.unnamed.len(), 1),
        _ => panic!("expected unnamed"),
    }
    match &e.variants[1].fields {
        Fields::Named(n) => assert_eq!(n.named.len(), 2),
        _ => panic!("expected named"),
    }
}

#[test]
fn enum_with_discriminant() {
    let e = parse_enum(quote! {
        enum Priority {
            Low = 1,
            Medium = 2,
            High = 3,
        }
    });
    for v in &e.variants {
        assert!(v.discriminant.is_some());
    }
}

#[test]
fn enum_with_attributes() {
    let e = parse_enum(quote! {
        #[derive(Debug, Clone)]
        enum Direction {
            #[default]
            North,
            South,
        }
    });
    assert!(!e.attrs.is_empty());
    assert!(!e.variants[0].attrs.is_empty());
}

#[test]
fn enum_generic() {
    let e = parse_enum(quote! {
        enum Result<T, E> {
            Ok(T),
            Err(E),
        }
    });
    assert_eq!(e.generics.params.len(), 2);
}

#[test]
fn enum_empty() {
    let e = parse_enum(quote! { enum Empty {} });
    assert_eq!(e.variants.len(), 0);
}

#[test]
fn enum_single_unit_variant() {
    let e = parse_enum(quote! { enum Single { Only } });
    assert_eq!(e.variants.len(), 1);
    assert!(matches!(e.variants[0].fields, Fields::Unit));
}

#[test]
fn enum_mixed_variant_types() {
    let e = parse_enum(quote! {
        enum Mixed {
            Unit,
            Tuple(i32, String),
            Struct { x: f64 },
        }
    });
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn enum_variant_count_large() {
    let e = parse_enum(quote! {
        enum Large { A, B, C, D, E, F, G, H, I, J, K, L }
    });
    assert_eq!(e.variants.len(), 12);
}

#[test]
fn enum_with_doc_attrs() {
    let e = parse_enum(quote! {
        /// Top-level doc
        enum Documented {
            /// Variant doc
            First,
            Second,
        }
    });
    assert!(!e.attrs.is_empty());
    assert!(!e.variants[0].attrs.is_empty());
    assert!(e.variants[1].attrs.is_empty());
}

// =============================================================================
// Section 3: Attribute patterns (tests 21–30)
// =============================================================================

#[test]
fn attr_derive_parse() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, PartialEq)]
        struct S { x: i32 }
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p == "derive"));
}

#[test]
fn attr_adze_grammar() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("my_lang")]
        mod my_grammar {}
    };
    let paths = attr_paths(&m.attrs);
    assert!(
        paths
            .iter()
            .any(|p| p.contains("adze") && p.contains("grammar"))
    );
}

#[test]
fn attr_adze_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct Root { value: Expr }
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p.contains("language")));
}

#[test]
fn attr_adze_leaf() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "+")]
        struct Plus;
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p.contains("leaf")));
}

#[test]
fn attr_multiple_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[derive(Debug)]
        struct Root { value: i32 }
    };
    assert_eq!(s.attrs.len(), 2);
}

#[test]
fn attr_cfg_conditional() {
    let s: ItemStruct = parse_quote! {
        #[cfg(test)]
        struct TestOnly { data: String }
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p == "cfg"));
}

#[test]
fn attr_allow_lint() {
    let s: ItemStruct = parse_quote! {
        #[allow(dead_code)]
        struct Unused;
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p == "allow"));
}

#[test]
fn attr_repr_c() {
    let s: ItemStruct = parse_quote! {
        #[repr(C)]
        struct CStruct { x: i32 }
    };
    let paths = attr_paths(&s.attrs);
    assert!(paths.iter().any(|p| p == "repr"));
}

#[test]
fn attr_serde_rename() {
    let s: ItemStruct = parse_quote! {
        #[serde(rename_all = "camelCase")]
        struct Config { my_field: String }
    };
    assert_eq!(s.attrs.len(), 1);
}

#[test]
fn attr_inner_on_field() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[serde(default)]
            field: i32,
        }
    };
    match &s.fields {
        Fields::Named(f) => assert!(!f.named[0].attrs.is_empty()),
        _ => panic!("expected named"),
    }
}

// =============================================================================
// Section 4: Struct field types - Box<T>, Vec<T>, Option<T> (tests 31–38)
// =============================================================================

fn extract_outer_type_name(ty: &Type) -> Option<String> {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

#[test]
fn field_type_box() {
    let s: ItemStruct = parse_quote! { struct S { f: Box<Expr> } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("Box".to_string()));
}

#[test]
fn field_type_vec() {
    let s: ItemStruct = parse_quote! { struct S { f: Vec<Item> } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("Vec".to_string()));
}

#[test]
fn field_type_option() {
    let s: ItemStruct = parse_quote! { struct S { f: Option<i32> } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("Option".to_string()));
}

#[test]
fn field_type_option_vec() {
    let s: ItemStruct = parse_quote! { struct S { f: Option<Vec<i32>> } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("Option".to_string()));
}

#[test]
fn field_type_vec_box() {
    let s: ItemStruct = parse_quote! { struct S { f: Vec<Box<Node>> } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("Vec".to_string()));
}

#[test]
fn field_type_plain_ident() {
    let s: ItemStruct = parse_quote! { struct S { f: String } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert_eq!(extract_outer_type_name(ty), Some("String".to_string()));
}

#[test]
fn field_type_reference() {
    let s: ItemStruct = parse_quote! { struct S<'a> { f: &'a str } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert!(matches!(ty, Type::Reference(_)));
}

#[test]
fn field_type_tuple() {
    let s: ItemStruct = parse_quote! { struct S { f: (i32, String) } };
    let ty = &s.fields.iter().next().unwrap().ty;
    assert!(matches!(ty, Type::Tuple(_)));
}

// =============================================================================
// Section 5: Function signatures for grammar rules (tests 39–45)
// =============================================================================

#[test]
fn fn_no_args_no_return() {
    let f: ItemFn = parse_quote! { fn rule() {} };
    assert_eq!(f.sig.ident.to_string(), "rule");
    assert!(f.sig.inputs.is_empty());
    assert!(matches!(f.sig.output, ReturnType::Default));
}

#[test]
fn fn_with_return_type() {
    let f: ItemFn = parse_quote! { fn parse() -> Expr {} };
    assert!(matches!(f.sig.output, ReturnType::Type(..)));
}

#[test]
fn fn_with_args() {
    let f: ItemFn = parse_quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
    assert_eq!(f.sig.inputs.len(), 2);
}

#[test]
fn fn_with_self_receiver() {
    let f: ItemFn = parse_quote! { fn method(&self) -> bool { true } };
    assert_eq!(f.sig.inputs.len(), 1);
}

#[test]
fn fn_async() {
    let f: ItemFn = parse_quote! { async fn fetch() -> String { String::new() } };
    assert!(f.sig.asyncness.is_some());
}

#[test]
fn fn_with_generics() {
    let f: ItemFn = parse_quote! { fn identity<T>(x: T) -> T { x } };
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn fn_with_where_clause() {
    let f: ItemFn = parse_quote! {
        fn display<T>(x: T) -> String where T: std::fmt::Display { x.to_string() }
    };
    assert!(f.sig.generics.where_clause.is_some());
}

// =============================================================================
// Section 6: Token stream manipulation (tests 46–52)
// =============================================================================

#[test]
fn token_stream_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn token_stream_from_quote() {
    let ts = quote! { let x = 42; };
    assert!(!ts.is_empty());
}

#[test]
fn token_stream_clone_eq() {
    let ts = quote! { fn foo() {} };
    let cloned = ts.clone();
    assert_eq!(ts.to_string(), cloned.to_string());
}

#[test]
fn token_stream_extend() {
    let mut ts = quote! { let a = 1; };
    let extra = quote! { let b = 2; };
    ts.extend(extra);
    let s = ts.to_string();
    assert!(s.contains("a"));
    assert!(s.contains("b"));
}

#[test]
fn token_stream_iter_count() {
    let ts = quote! { a + b };
    let count = ts.into_iter().count();
    assert_eq!(count, 3); // ident, punct, ident
}

#[test]
fn token_stream_nested_groups() {
    let ts = quote! { { (1 + 2) } };
    let first = ts.into_iter().next().unwrap();
    assert!(matches!(first, TokenTree::Group(_)));
}

#[test]
fn token_stream_round_trip() {
    let original = quote! { struct Foo { bar: i32 } };
    let text = original.to_string();
    let reparsed: TokenStream = text.parse().unwrap();
    assert_eq!(original.to_string(), reparsed.to_string());
}

// =============================================================================
// Section 7: Ident creation and comparison (tests 53–58)
// =============================================================================

#[test]
fn ident_create_and_eq() {
    let a = Ident::new("hello", Span::call_site());
    let b = Ident::new("hello", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn ident_not_equal() {
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("bar", Span::call_site());
    assert_ne!(a, b);
}

#[test]
fn ident_to_string() {
    let id = Ident::new("my_ident", Span::call_site());
    assert_eq!(id.to_string(), "my_ident");
}

#[test]
fn ident_from_parse_quote() {
    let id: Ident = parse_quote!(some_name);
    assert_eq!(id, "some_name");
}

#[test]
fn ident_in_quote_interpolation() {
    let name = Ident::new("dynamic", Span::call_site());
    let ts = quote! { let #name = 42; };
    assert!(ts.to_string().contains("dynamic"));
}

#[test]
fn ident_raw_keyword() {
    let id = Ident::new_raw("match", Span::call_site());
    assert!(id.to_string().contains("match"));
}

// =============================================================================
// Section 8: Literal types (tests 59–64)
// =============================================================================

#[test]
fn literal_string() {
    let lit = Literal::string("hello");
    assert!(lit.to_string().contains("hello"));
}

#[test]
fn literal_integer() {
    let lit = Literal::i32_suffixed(42);
    assert!(lit.to_string().contains("42"));
}

#[test]
fn literal_float() {
    let lit = Literal::f64_suffixed(3.15);
    let s = lit.to_string();
    assert!(s.contains("3.15"));
}

#[test]
fn literal_byte_string() {
    let lit = Literal::byte_string(b"bytes");
    let s = lit.to_string();
    assert!(s.contains("bytes"));
}

#[test]
fn literal_char() {
    let lit = Literal::character('z');
    let s = lit.to_string();
    assert!(s.contains('z'));
}

#[test]
fn literal_unsuffixed_integer() {
    let lit = Literal::i64_unsuffixed(999);
    assert_eq!(lit.to_string(), "999");
}

// =============================================================================
// Section 9: Type path parsing (tests 65–70)
// =============================================================================

#[test]
fn type_simple_path() {
    let ty: Type = parse_quote!(String);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments.len(), 1);
        assert_eq!(tp.path.segments[0].ident.to_string(), "String");
    } else {
        panic!("expected Type::Path");
    }
}

#[test]
fn type_qualified_path() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments.len(), 3);
    } else {
        panic!("expected Type::Path");
    }
}

#[test]
fn type_generic_single() {
    let ty: Type = parse_quote!(Vec<i32>);
    if let Type::Path(tp) = &ty {
        let seg = &tp.path.segments[0];
        assert_eq!(seg.ident.to_string(), "Vec");
        assert!(!seg.arguments.is_empty());
    } else {
        panic!("expected Type::Path");
    }
}

#[test]
fn type_nested_generic() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments[0].ident.to_string(), "Option");
    } else {
        panic!("expected Type::Path");
    }
}

#[test]
fn type_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    assert!(matches!(ty, Type::BareFn(_)));
}

#[test]
fn type_array() {
    let ty: Type = parse_quote!([u8; 32]);
    assert!(matches!(ty, Type::Array(_)));
}

// =============================================================================
// Section 10: Visibility parsing (tests 71–76)
// =============================================================================

#[test]
fn vis_private() {
    let s: ItemStruct = parse_quote! { struct Private; };
    assert!(matches!(s.vis, Visibility::Inherited));
}

#[test]
fn vis_pub() {
    let s: ItemStruct = parse_quote! { pub struct Public; };
    assert!(matches!(s.vis, Visibility::Public(_)));
}

#[test]
fn vis_pub_crate() {
    let s: ItemStruct = parse_quote! { pub(crate) struct CrateVis; };
    match &s.vis {
        Visibility::Restricted(r) => {
            assert_eq!(r.path.to_token_stream().to_string(), "crate");
        }
        _ => panic!("expected restricted visibility"),
    }
}

#[test]
fn vis_pub_super() {
    let s: ItemStruct = parse_quote! { pub(super) struct SuperVis; };
    match &s.vis {
        Visibility::Restricted(r) => {
            assert_eq!(r.path.to_token_stream().to_string(), "super");
        }
        _ => panic!("expected restricted visibility"),
    }
}

#[test]
fn vis_field_level() {
    let s: ItemStruct = parse_quote! {
        struct S {
            pub x: i32,
            y: i32,
        }
    };
    let fields: Vec<&Field> = s.fields.iter().collect();
    assert!(matches!(fields[0].vis, Visibility::Public(_)));
    assert!(matches!(fields[1].vis, Visibility::Inherited));
}

#[test]
fn vis_pub_in_path() {
    let s: ItemStruct = parse_quote! { pub(in crate::module) struct PathVis; };
    assert!(matches!(s.vis, Visibility::Restricted(_)));
}

// =============================================================================
// Section 11: Where clause parsing (tests 77–80)
// =============================================================================

#[test]
fn where_single_bound() {
    let f: ItemFn = parse_quote! { fn f<T>() where T: Clone {} };
    let wc = f.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_multiple_bounds() {
    let f: ItemFn = parse_quote! {
        fn f<T, U>() where T: Clone, U: Default {}
    };
    let wc = f.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn where_complex_bound() {
    let f: ItemFn = parse_quote! {
        fn f<T>() where T: Clone + Send + 'static {}
    };
    let wc = f.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_on_struct() {
    let s: ItemStruct = parse_quote! {
        struct Container<T> where T: Clone {
            value: T,
        }
    };
    assert!(s.generics.where_clause.is_some());
}

// =============================================================================
// Section 12: Trait bound parsing (tests 81–85)
// =============================================================================

#[test]
fn trait_def_basic() {
    let t: ItemTrait = parse_quote! {
        trait Parseable {
            fn parse(input: &str) -> Self;
        }
    };
    assert_eq!(t.ident.to_string(), "Parseable");
    assert_eq!(t.items.len(), 1);
}

#[test]
fn trait_with_supertraits() {
    let t: ItemTrait = parse_quote! {
        trait MyTrait: Clone + Send {}
    };
    assert!(!t.supertraits.is_empty());
}

#[test]
fn generic_type_bound() {
    let s: ItemStruct = parse_quote! {
        struct S<T: Clone + Default> { value: T }
    };
    match s.generics.params.first().unwrap() {
        GenericParam::Type(tp) => assert!(!tp.bounds.is_empty()),
        _ => panic!("expected type param"),
    }
}

#[test]
fn lifetime_bound() {
    let s: ItemStruct = parse_quote! {
        struct S<'a, T: 'a> { data: &'a T }
    };
    assert_eq!(s.generics.params.len(), 2);
}

#[test]
fn const_generic_param() {
    let s: ItemStruct = parse_quote! {
        struct Array<const N: usize> { data: [i32; N] }
    };
    match s.generics.params.first().unwrap() {
        GenericParam::Const(c) => assert_eq!(c.ident.to_string(), "N"),
        _ => panic!("expected const param"),
    }
}

// =============================================================================
// Section 13: Pattern matching on syn types (tests 86–92)
// =============================================================================

#[test]
fn match_derive_input_struct() {
    let di: DeriveInput = parse_quote! {
        struct Foo { x: i32 }
    };
    assert!(matches!(di.data, syn::Data::Struct(_)));
}

#[test]
fn match_derive_input_enum() {
    let di: DeriveInput = parse_quote! {
        enum Bar { A, B }
    };
    assert!(matches!(di.data, syn::Data::Enum(_)));
}

#[test]
fn match_expr_lit() {
    let e: Expr = parse_quote!(42);
    assert!(matches!(e, Expr::Lit(_)));
}

#[test]
fn match_expr_binary() {
    let e: Expr = parse_quote!(a + b);
    assert!(matches!(e, Expr::Binary(_)));
}

#[test]
fn match_expr_call() {
    let e: Expr = parse_quote!(foo(1, 2));
    assert!(matches!(e, Expr::Call(_)));
}

#[test]
fn match_pat_ident() {
    let p: Pat = parse_quote!(x);
    assert!(matches!(p, Pat::Ident(_)));
}

#[test]
fn match_pat_tuple() {
    let p: Pat = parse_quote!((a, b));
    assert!(matches!(p, Pat::Tuple(_)));
}

// =============================================================================
// Section 14: Error handling for invalid syntax (tests 93–100)
// =============================================================================

#[test]
fn invalid_struct_parse_fails() {
    let ts = quote! { not a struct at all };
    assert!(parse2::<ItemStruct>(ts).is_err());
}

#[test]
fn invalid_enum_parse_fails() {
    let ts = quote! { 123 invalid };
    assert!(parse2::<ItemEnum>(ts).is_err());
}

#[test]
fn invalid_type_parse_fails() {
    let ts = quote! { + + + };
    assert!(parse2::<Type>(ts).is_err());
}

#[test]
fn invalid_fn_parse_fails() {
    let ts = quote! { struct NotAFunction {} };
    assert!(parse2::<ItemFn>(ts).is_err());
}

#[test]
fn empty_stream_struct_fails() {
    let ts = TokenStream::new();
    assert!(parse2::<ItemStruct>(ts).is_err());
}

#[test]
fn empty_stream_enum_fails() {
    let ts = TokenStream::new();
    assert!(parse2::<ItemEnum>(ts).is_err());
}

#[test]
fn parse_error_has_message() {
    let ts = quote! { not valid };
    let err = parse2::<ItemStruct>(ts).unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn partial_struct_extra_tokens() {
    let ts = quote! { struct S { x: i32 } extra_garbage };
    assert!(parse2::<ItemStruct>(ts).is_err());
}

// =============================================================================
// Section 15: Additional patterns (tests 101–106)
// =============================================================================

#[test]
fn meta_path_extraction() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        struct S;
    };
    let meta = &s.attrs[0].meta;
    assert!(matches!(meta, Meta::List(_)));
}

#[test]
fn module_with_items() {
    let m: ItemMod = parse_quote! {
        mod grammar {
            struct Expr { value: i32 }
            enum Op { Add, Sub }
        }
    };
    assert_eq!(m.ident.to_string(), "grammar");
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn lifetime_parsing() {
    let lt: Lifetime = parse_quote!('static);
    assert_eq!(lt.ident.to_string(), "static");
}

#[test]
fn type_slice() {
    let ty: Type = parse_quote!([u8]);
    assert!(matches!(ty, Type::Slice(_)));
}

#[test]
fn type_reference_mut() {
    let ty: Type = parse_quote!(&mut Vec<i32>);
    if let Type::Reference(r) = &ty {
        assert!(r.mutability.is_some());
    } else {
        panic!("expected reference type");
    }
}

#[test]
fn type_impl_trait() {
    let ty: Type = parse_quote!(impl Iterator<Item = i32>);
    assert!(matches!(ty, Type::ImplTrait(_)));
}
