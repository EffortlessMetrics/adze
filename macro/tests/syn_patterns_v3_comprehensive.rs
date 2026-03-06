// Comprehensive tests for syn parsing patterns used in macro crates.
// Covers structs, enums, generics, attributes, expressions, paths,
// function signatures, complex types, and edge cases.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Expr, GenericParam, ItemEnum, ItemFn, ItemStruct, Path, Type,
    WhereClause, parse_quote, parse_str, parse2,
};

// =====================================================================
// Helpers
// =====================================================================

fn parse_derive(tokens: TokenStream) -> DeriveInput {
    parse2::<DeriveInput>(tokens).expect("Failed to parse DeriveInput")
}

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2::<ItemStruct>(tokens).expect("Failed to parse ItemStruct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2::<ItemEnum>(tokens).expect("Failed to parse ItemEnum")
}

fn parse_fn_item(tokens: TokenStream) -> ItemFn {
    parse2::<ItemFn>(tokens).expect("Failed to parse ItemFn")
}

// =====================================================================
// 1. Struct parsing patterns (8 tests)
// =====================================================================

#[test]
fn struct_named_fields() {
    let s = parse_struct(quote! {
        struct Foo {
            x: i32,
            y: String,
        }
    });
    assert_eq!(s.ident, "Foo");
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn struct_unnamed_fields() {
    let s = parse_struct(quote! {
        struct Point(f64, f64);
    });
    assert_eq!(s.ident, "Point");
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn struct_unit() {
    let s = parse_struct(quote! {
        struct Unit;
    });
    assert_eq!(s.ident, "Unit");
    assert!(s.fields.is_empty());
}

#[test]
fn struct_with_visibility() {
    let s = parse_struct(quote! {
        pub struct Public {
            pub name: String,
            pub(crate) id: u64,
        }
    });
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn struct_with_lifetime() {
    let s = parse_struct(quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    });
    assert_eq!(s.generics.lifetimes().count(), 1);
}

#[test]
fn struct_with_generic_params() {
    let s = parse_struct(quote! {
        struct Container<T, U> {
            first: T,
            second: U,
        }
    });
    assert_eq!(s.generics.params.len(), 2);
}

#[test]
fn struct_with_where_clause() {
    let s = parse_struct(quote! {
        struct Bounded<T> where T: Clone + Send {
            value: T,
        }
    });
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn struct_with_derive_attribute() {
    let d = parse_derive(quote! {
        #[derive(Debug, Clone)]
        struct Derived {
            field: i32,
        }
    });
    assert_eq!(d.ident, "Derived");
    assert_eq!(d.attrs.len(), 1);
}

// =====================================================================
// 2. Enum parsing patterns (8 tests)
// =====================================================================

#[test]
fn enum_unit_variants() {
    let e = parse_enum(quote! {
        enum Color { Red, Green, Blue }
    });
    assert_eq!(e.ident, "Color");
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn enum_tuple_variants() {
    let e = parse_enum(quote! {
        enum Shape {
            Circle(f64),
            Rect(f64, f64),
        }
    });
    assert_eq!(e.variants.len(), 2);
    assert_eq!(e.variants[1].fields.len(), 2);
}

#[test]
fn enum_struct_variants() {
    let e = parse_enum(quote! {
        enum Event {
            Click { x: i32, y: i32 },
            Key { code: u32 },
        }
    });
    assert_eq!(e.variants.len(), 2);
    assert_eq!(e.variants[0].fields.len(), 2);
}

#[test]
fn enum_mixed_variants() {
    let e = parse_enum(quote! {
        enum Token {
            Eof,
            Number(f64),
            Ident { name: String },
        }
    });
    assert_eq!(e.variants.len(), 3);
    assert!(e.variants[0].fields.is_empty());
    assert_eq!(e.variants[1].fields.len(), 1);
    assert_eq!(e.variants[2].fields.len(), 1);
}

#[test]
fn enum_with_discriminant() {
    let e = parse_enum(quote! {
        enum Status {
            Ok = 0,
            Error = 1,
            Unknown = 255,
        }
    });
    for v in &e.variants {
        assert!(v.discriminant.is_some());
    }
}

#[test]
fn enum_with_generics() {
    let e = parse_enum(quote! {
        enum Result<T, E> {
            Ok(T),
            Err(E),
        }
    });
    assert_eq!(e.generics.params.len(), 2);
}

#[test]
fn enum_with_attributes_on_variants() {
    let e = parse_enum(quote! {
        enum Cmd {
            #[deprecated]
            OldRun,
            Run,
        }
    });
    assert_eq!(e.variants[0].attrs.len(), 1);
    assert!(e.variants[1].attrs.is_empty());
}

#[test]
fn enum_with_where_clause() {
    let e = parse_enum(quote! {
        enum Wrapper<T> where T: std::fmt::Display {
            Some(T),
            None,
        }
    });
    assert!(e.generics.where_clause.is_some());
}

// =====================================================================
// 3. Generic type parsing (8 tests)
// =====================================================================

#[test]
fn generic_lifetime_param() {
    let d = parse_derive(quote! {
        struct Ref<'a> {
            data: &'a u8,
        }
    });
    let param = d.generics.params.first().unwrap();
    assert!(matches!(param, GenericParam::Lifetime(_)));
}

#[test]
fn generic_type_param() {
    let d = parse_derive(quote! {
        struct Box<T> {
            inner: T,
        }
    });
    let param = d.generics.params.first().unwrap();
    assert!(matches!(param, GenericParam::Type(_)));
}

#[test]
fn generic_const_param() {
    let d = parse_derive(quote! {
        struct Array<const N: usize> {
            data: [u8; N],
        }
    });
    let param = d.generics.params.first().unwrap();
    assert!(matches!(param, GenericParam::Const(_)));
}

#[test]
fn generic_multiple_lifetimes() {
    let d = parse_derive(quote! {
        struct Multi<'a, 'b> {
            first: &'a str,
            second: &'b str,
        }
    });
    assert_eq!(d.generics.lifetimes().count(), 2);
}

#[test]
fn generic_bounded_type_param() {
    let d = parse_derive(quote! {
        struct Bounded<T: Clone + Send> {
            value: T,
        }
    });
    if let GenericParam::Type(tp) = d.generics.params.first().unwrap() {
        assert_eq!(tp.bounds.len(), 2);
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn generic_default_type() {
    let d = parse_derive(quote! {
        struct WithDefault<T = String> {
            val: T,
        }
    });
    if let GenericParam::Type(tp) = d.generics.params.first().unwrap() {
        assert!(tp.default.is_some());
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn generic_mixed_params() {
    let d = parse_derive(quote! {
        struct Mixed<'a, T: Clone, const N: usize> {
            reference: &'a T,
            size: [u8; N],
        }
    });
    assert_eq!(d.generics.params.len(), 3);
    assert!(matches!(&d.generics.params[0], GenericParam::Lifetime(_)));
    assert!(matches!(&d.generics.params[1], GenericParam::Type(_)));
    assert!(matches!(&d.generics.params[2], GenericParam::Const(_)));
}

#[test]
fn generic_where_clause_parsing() {
    let wc: WhereClause = parse_quote! {
        where T: Clone + Send, U: Default
    };
    assert_eq!(wc.predicates.len(), 2);
}

// =====================================================================
// 4. Attribute parsing (8 tests)
// =====================================================================

#[test]
fn attribute_outer_simple() {
    let d = parse_derive(quote! {
        #[allow(unused)]
        struct S;
    });
    assert_eq!(d.attrs.len(), 1);
}

#[test]
fn attribute_multiple() {
    let d = parse_derive(quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct S;
    });
    assert_eq!(d.attrs.len(), 2);
}

#[test]
fn attribute_path_segment() {
    let attrs: Vec<Attribute> = parse_derive(quote! {
        #[serde(rename_all = "camelCase")]
        struct S;
    })
    .attrs;
    let path = &attrs[0].path();
    assert!(path.is_ident("serde"));
}

#[test]
fn attribute_cfg_predicate() {
    let d = parse_derive(quote! {
        #[cfg(target_os = "linux")]
        struct LinuxOnly;
    });
    assert_eq!(d.attrs.len(), 1);
    assert!(d.attrs[0].path().is_ident("cfg"));
}

#[test]
fn attribute_doc_comment_equivalent() {
    let d = parse_derive(quote! {
        #[doc = "A documented struct"]
        struct Documented;
    });
    assert_eq!(d.attrs.len(), 1);
    assert!(d.attrs[0].path().is_ident("doc"));
}

#[test]
fn attribute_nested_meta() {
    let d = parse_derive(quote! {
        #[cfg(any(feature = "a", feature = "b"))]
        struct Gated;
    });
    assert_eq!(d.attrs.len(), 1);
}

#[test]
fn attribute_on_fields() {
    let s = parse_struct(quote! {
        struct S {
            #[serde(skip)]
            skipped: u8,
            normal: u8,
        }
    });
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(fields[0].attrs.len(), 1);
    assert!(fields[1].attrs.is_empty());
}

#[test]
fn attribute_repr() {
    let d = parse_derive(quote! {
        #[repr(C)]
        struct CRepr {
            x: i32,
        }
    });
    assert!(d.attrs[0].path().is_ident("repr"));
}

// =====================================================================
// 5. Expression parsing (5 tests)
// =====================================================================

#[test]
fn expr_literal_integer() {
    let e: Expr = parse_quote!(42);
    assert!(matches!(e, Expr::Lit(_)));
}

#[test]
fn expr_binary_operation() {
    let e: Expr = parse_quote!(a + b);
    assert!(matches!(e, Expr::Binary(_)));
}

#[test]
fn expr_method_call() {
    let e: Expr = parse_quote!(foo.bar(1, 2));
    assert!(matches!(e, Expr::MethodCall(_)));
}

#[test]
fn expr_closure() {
    let e: Expr = parse_quote!(|x| x + 1);
    assert!(matches!(e, Expr::Closure(_)));
}

#[test]
fn expr_if_else() {
    let e: Expr = parse_quote!(if cond { 1 } else { 2 });
    assert!(matches!(e, Expr::If(_)));
}

// =====================================================================
// 6. Path parsing (5 tests)
// =====================================================================

#[test]
fn path_simple_ident() {
    let p: Path = parse_quote!(std);
    assert_eq!(p.segments.len(), 1);
    assert_eq!(p.segments[0].ident, "std");
}

#[test]
fn path_multi_segment() {
    let p: Path = parse_quote!(std::collections::HashMap);
    assert_eq!(p.segments.len(), 3);
    assert_eq!(p.segments[2].ident, "HashMap");
}

#[test]
fn path_with_turbofish() {
    let p: Path = parse_quote!(Vec::<i32>);
    assert_eq!(p.segments.len(), 1);
    assert!(!p.segments[0].arguments.is_none());
}

#[test]
fn path_leading_colon() {
    let p: Path = parse_quote!(::std::vec::Vec);
    assert!(p.leading_colon.is_some());
    assert_eq!(p.segments.len(), 3);
}

#[test]
fn path_is_ident_check() {
    let p: Path = parse_quote!(Clone);
    assert!(p.is_ident("Clone"));
}

// =====================================================================
// 7. Function signature parsing (5 tests)
// =====================================================================

#[test]
fn fn_no_args_no_return() {
    let f = parse_fn_item(quote! {
        fn noop() {}
    });
    assert_eq!(f.sig.ident, "noop");
    assert!(f.sig.inputs.is_empty());
    assert!(matches!(f.sig.output, syn::ReturnType::Default));
}

#[test]
fn fn_with_args_and_return() {
    let f = parse_fn_item(quote! {
        fn add(a: i32, b: i32) -> i32 { a + b }
    });
    assert_eq!(f.sig.inputs.len(), 2);
    assert!(matches!(f.sig.output, syn::ReturnType::Type(..)));
}

#[test]
fn fn_generic() {
    let f = parse_fn_item(quote! {
        fn identity<T>(x: T) -> T { x }
    });
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn fn_async() {
    let f = parse_fn_item(quote! {
        async fn fetch() -> String { String::new() }
    });
    assert!(f.sig.asyncness.is_some());
}

#[test]
fn fn_unsafe() {
    let f = parse_fn_item(quote! {
        unsafe fn danger() {}
    });
    assert!(f.sig.unsafety.is_some());
}

// =====================================================================
// 8. Complex type patterns (5 tests)
// =====================================================================

#[test]
fn type_reference() {
    let t: Type = parse_quote!(&str);
    assert!(matches!(t, Type::Reference(_)));
}

#[test]
fn type_slice() {
    let t: Type = parse_quote!([u8]);
    assert!(matches!(t, Type::Slice(_)));
}

#[test]
fn type_tuple() {
    let t: Type = parse_quote!((i32, String, bool));
    if let Type::Tuple(tup) = t {
        assert_eq!(tup.elems.len(), 3);
    } else {
        panic!("Expected tuple type");
    }
}

#[test]
fn type_array() {
    let t: Type = parse_quote!([u8; 32]);
    assert!(matches!(t, Type::Array(_)));
}

#[test]
fn type_fn_pointer() {
    let t: Type = parse_quote!(fn(i32) -> bool);
    assert!(matches!(t, Type::BareFn(_)));
}

// =====================================================================
// 9. Edge cases (3 tests)
// =====================================================================

#[test]
fn edge_empty_struct() {
    let s = parse_struct(quote! {
        struct Empty {}
    });
    assert!(s.fields.is_empty());
}

#[test]
fn edge_parse_str_type() {
    let t: Type = parse_str("Option<Vec<String>>").expect("parse_str failed");
    let output = t.to_token_stream().to_string();
    assert!(output.contains("Option"));
    assert!(output.contains("Vec"));
}

#[test]
fn edge_deeply_nested_generics() {
    let t: Type = parse_quote!(Option<Result<Vec<Box<dyn Iterator<Item = u8>>>, String>>);
    let output = t.to_token_stream().to_string();
    assert!(output.contains("Option"));
    assert!(output.contains("Result"));
    assert!(output.contains("Vec"));
    assert!(output.contains("Box"));
}
