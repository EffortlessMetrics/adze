// Wave 131: Comprehensive tests for macro crate syn/quote roundtrip patterns
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, GenericParam, ItemEnum, ItemStruct, Type, parse_quote, parse2};

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2::<ItemStruct>(tokens).expect("Failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2::<ItemEnum>(tokens).expect("Failed to parse enum")
}

// =====================================================================
// Struct parsing: basic shapes
// =====================================================================

#[test]
fn parse_empty_struct() {
    let s = parse_struct(quote! { struct Empty; });
    assert_eq!(s.ident.to_string(), "Empty");
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn parse_tuple_struct() {
    let s = parse_struct(quote! { struct Pair(i32, String); });
    assert_eq!(s.ident.to_string(), "Pair");
    match &s.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 2),
        _ => panic!("Expected unnamed fields"),
    }
}

#[test]
fn parse_named_struct() {
    let s = parse_struct(quote! {
        struct Point {
            x: f64,
            y: f64,
        }
    });
    assert_eq!(s.ident.to_string(), "Point");
    match &s.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 2),
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn parse_public_struct() {
    let s = parse_struct(quote! {
        pub struct Public {
            pub field: i32,
        }
    });
    assert_eq!(s.ident.to_string(), "Public");
}

#[test]
fn parse_struct_with_doc() {
    let s = parse_struct(quote! {
        /// This is documentation
        struct Documented {
            field: String,
        }
    });
    assert!(!s.attrs.is_empty(), "Should have doc attribute");
}

#[test]
fn parse_struct_with_derive() {
    let s = parse_struct(quote! {
        #[derive(Debug, Clone)]
        struct Derived {
            value: i32,
        }
    });
    assert!(!s.attrs.is_empty());
}

// =====================================================================
// Struct parsing: generics
// =====================================================================

#[test]
fn parse_generic_struct() {
    let s = parse_struct(quote! {
        struct Container<T> {
            value: T,
        }
    });
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn parse_multi_generic_struct() {
    let s = parse_struct(quote! {
        struct Pair<A, B> {
            first: A,
            second: B,
        }
    });
    assert_eq!(s.generics.params.len(), 2);
}

#[test]
fn parse_bounded_generic() {
    let s = parse_struct(quote! {
        struct Bounded<T: Clone + Send> {
            value: T,
        }
    });
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn parse_where_clause() {
    let s = parse_struct(quote! {
        struct WithWhere<T>
        where
            T: std::fmt::Debug,
        {
            value: T,
        }
    });
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn parse_lifetime_struct() {
    let s = parse_struct(quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    });
    assert_eq!(s.generics.params.len(), 1);
    match &s.generics.params[0] {
        GenericParam::Lifetime(_) => {}
        _ => panic!("Expected lifetime parameter"),
    }
}

#[test]
fn parse_const_generic() {
    let s = parse_struct(quote! {
        struct Array<const N: usize> {
            data: [u8; N],
        }
    });
    assert_eq!(s.generics.params.len(), 1);
}

// =====================================================================
// Enum parsing
// =====================================================================

#[test]
fn parse_unit_enum() {
    let e = parse_enum(quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    });
    assert_eq!(e.ident.to_string(), "Color");
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn parse_enum_with_data() {
    let e = parse_enum(quote! {
        enum Shape {
            Circle(f64),
            Rectangle { width: f64, height: f64 },
            Point,
        }
    });
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn parse_enum_with_discriminant() {
    let e = parse_enum(quote! {
        enum Status {
            Active = 1,
            Inactive = 0,
        }
    });
    assert_eq!(e.variants.len(), 2);
    assert!(e.variants[0].discriminant.is_some());
}

#[test]
fn parse_generic_enum() {
    let e = parse_enum(quote! {
        enum Option<T> {
            Some(T),
            None,
        }
    });
    assert_eq!(e.generics.params.len(), 1);
    assert_eq!(e.variants.len(), 2);
}

#[test]
fn parse_enum_with_attrs() {
    let e = parse_enum(quote! {
        #[derive(Debug)]
        #[repr(u8)]
        enum Flags {
            A = 1,
            B = 2,
        }
    });
    assert_eq!(e.attrs.len(), 2);
}

// =====================================================================
// Type parsing
// =====================================================================

#[test]
fn parse_simple_type() {
    let ty: Type = parse_quote!(String);
    let s = quote!(#ty).to_string();
    assert_eq!(s, "String");
}

#[test]
fn parse_vec_type() {
    let ty: Type = parse_quote!(Vec<i32>);
    let s = quote!(#ty).to_string();
    assert!(s.contains("Vec"));
}

#[test]
fn parse_option_type() {
    let ty: Type = parse_quote!(Option<String>);
    let s = quote!(#ty).to_string();
    assert!(s.contains("Option"));
}

#[test]
fn parse_result_type() {
    let ty: Type = parse_quote!(Result<String, std::io::Error>);
    let s = quote!(#ty).to_string();
    assert!(s.contains("Result"));
}

#[test]
fn parse_reference_type() {
    let ty: Type = parse_quote!(&str);
    let s = quote!(#ty).to_string();
    assert!(s.contains("str"));
}

#[test]
fn parse_mutable_reference_type() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    let s = quote!(#ty).to_string();
    assert!(s.contains("mut"));
}

#[test]
fn parse_array_type() {
    let ty: Type = parse_quote!([u8; 32]);
    let s = quote!(#ty).to_string();
    assert!(s.contains("u8"));
}

#[test]
fn parse_tuple_type() {
    let ty: Type = parse_quote!((i32, String, bool));
    let s = quote!(#ty).to_string();
    assert!(s.contains("i32"));
}

#[test]
fn parse_fn_pointer_type() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let s = quote!(#ty).to_string();
    assert!(s.contains("fn"));
}

#[test]
fn parse_nested_generic_type() {
    let ty: Type = parse_quote!(Vec<Vec<Option<String>>>);
    let s = quote!(#ty).to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
}

// =====================================================================
// Token stream roundtrip
// =====================================================================

#[test]
fn struct_roundtrip() {
    let tokens = quote! {
        struct Foo {
            bar: i32,
            baz: String,
        }
    };
    let item: ItemStruct = parse2(tokens.clone()).unwrap();
    let regenerated = quote!(#item);
    let item2: ItemStruct = parse2(regenerated).unwrap();
    assert_eq!(item.ident, item2.ident);
}

#[test]
fn enum_roundtrip() {
    let tokens = quote! {
        enum Direction {
            North,
            South,
            East,
            West,
        }
    };
    let item: ItemEnum = parse2(tokens.clone()).unwrap();
    let regenerated = quote!(#item);
    let item2: ItemEnum = parse2(regenerated).unwrap();
    assert_eq!(item.ident, item2.ident);
    assert_eq!(item.variants.len(), item2.variants.len());
}

#[test]
fn derive_input_roundtrip() {
    let tokens = quote! {
        #[derive(Debug, Clone)]
        struct Data {
            id: u64,
            name: String,
        }
    };
    let di: DeriveInput = parse2(tokens).unwrap();
    let regenerated = quote!(#di);
    let di2: DeriveInput = parse2(regenerated).unwrap();
    assert_eq!(di.ident, di2.ident);
}

// =====================================================================
// Complex struct patterns
// =====================================================================

#[test]
fn parse_struct_many_fields() {
    let s = parse_struct(quote! {
        struct ManyFields {
            f1: i32, f2: i32, f3: i32, f4: i32, f5: i32,
            f6: i32, f7: i32, f8: i32, f9: i32, f10: i32,
        }
    });
    match &s.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 10),
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn parse_struct_complex_field_types() {
    let s = parse_struct(quote! {
        struct Complex {
            map: std::collections::HashMap<String, Vec<Option<i32>>>,
            callback: Box<dyn Fn(i32) -> bool + Send + Sync>,
            nested: (Vec<String>, Option<Box<[u8]>>),
        }
    });
    match &s.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 3),
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn parse_struct_with_multiple_attrs() {
    let s = parse_struct(quote! {
        #[derive(Debug)]
        #[derive(Clone)]
        #[cfg(test)]
        #[allow(dead_code)]
        struct MultiAttr {
            value: i32,
        }
    });
    assert_eq!(s.attrs.len(), 4);
}

// =====================================================================
// Edge cases
// =====================================================================

#[test]
fn parse_struct_single_unnamed_field() {
    let s = parse_struct(quote! { struct Wrapper(i32); });
    match &s.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 1),
        _ => panic!("Expected unnamed field"),
    }
}

#[test]
fn parse_empty_named_struct() {
    let s = parse_struct(quote! { struct Empty {} });
    match &s.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 0),
        _ => panic!("Expected empty named fields"),
    }
}

#[test]
fn parse_struct_phantom_data() {
    let s = parse_struct(quote! {
        struct Tagged<T> {
            _phantom: std::marker::PhantomData<T>,
        }
    });
    assert_eq!(s.generics.params.len(), 1);
}
