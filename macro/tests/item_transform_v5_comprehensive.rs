// Comprehensive tests for syn item parsing and transformation used in macro crate
use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Fields, Generics, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Visibility,
    parse_quote, parse_str,
};

// =====================================================================
// 1. ItemStruct parsing (8 tests)
// =====================================================================

#[test]
fn struct_named_fields_basic() {
    let s: ItemStruct = parse_quote! {
        struct Record {
            name: String,
            age: u32,
        }
    };
    assert_eq!(s.ident.to_string(), "Record");
    match &s.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 2),
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn struct_tuple_fields() {
    let s: ItemStruct = parse_quote! {
        struct Pair(i32, i32);
    };
    assert_eq!(s.ident.to_string(), "Pair");
    match &s.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 2),
        _ => panic!("Expected unnamed fields"),
    }
}

#[test]
fn struct_unit() {
    let s: ItemStruct = parse_quote! { struct Marker; };
    assert_eq!(s.ident.to_string(), "Marker");
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn struct_with_generics() {
    let s: ItemStruct = parse_quote! {
        struct Container<T> {
            value: T,
        }
    };
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn struct_pub_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct Exposed {
            pub data: Vec<u8>,
        }
    };
    assert!(matches!(s.vis, Visibility::Public(_)));
}

#[test]
fn struct_pub_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        pub(crate) struct Internal {
            count: usize,
        }
    };
    match &s.vis {
        Visibility::Restricted(r) => {
            assert_eq!(r.path.segments.first().unwrap().ident.to_string(), "crate");
        }
        _ => panic!("Expected restricted visibility"),
    }
}

#[test]
fn struct_with_lifetime_generic() {
    let s: ItemStruct = parse_quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    };
    assert_eq!(s.generics.params.len(), 1);
    assert!(matches!(
        s.generics.params.first().unwrap(),
        syn::GenericParam::Lifetime(_)
    ));
}

#[test]
fn struct_multiple_generics() {
    let s: ItemStruct = parse_quote! {
        struct Multi<'a, T: Clone, const N: usize> {
            items: &'a [T; N],
        }
    };
    assert_eq!(s.generics.params.len(), 3);
}

// =====================================================================
// 2. ItemEnum parsing (8 tests)
// =====================================================================

#[test]
fn enum_simple_variants() {
    let e: ItemEnum = parse_quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    };
    assert_eq!(e.ident.to_string(), "Color");
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn enum_tuple_variant_data() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            Literal(i64),
            Binary(Box<Expr>, Op, Box<Expr>),
        }
    };
    assert_eq!(e.variants.len(), 2);
    let binary = &e.variants[1];
    match &binary.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 3),
        _ => panic!("Expected unnamed fields"),
    }
}

#[test]
fn enum_struct_variant() {
    let e: ItemEnum = parse_quote! {
        enum Shape {
            Circle { radius: f64 },
            Rect { width: f64, height: f64 },
        }
    };
    let rect = &e.variants[1];
    match &rect.fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 2),
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn enum_with_discriminants() {
    let e: ItemEnum = parse_quote! {
        enum Status {
            Active = 1,
            Inactive = 0,
        }
    };
    for variant in &e.variants {
        assert!(variant.discriminant.is_some());
    }
}

#[test]
fn enum_mixed_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        enum Token {
            Eof,
            Ident(String),
            Pair { key: String, val: String },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn enum_generic() {
    let e: ItemEnum = parse_quote! {
        enum Option<T> {
            Some(T),
            None,
        }
    };
    assert_eq!(e.generics.params.len(), 1);
    assert_eq!(e.variants.len(), 2);
}

#[test]
fn enum_with_attributes() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug)]
        enum Direction {
            #[default]
            North,
            South,
        }
    };
    assert!(!e.attrs.is_empty());
    assert!(!e.variants[0].attrs.is_empty());
}

#[test]
fn enum_pub_visibility() {
    let e: ItemEnum = parse_quote! {
        pub enum Access {
            Read,
            Write,
        }
    };
    assert!(matches!(e.vis, Visibility::Public(_)));
}

// =====================================================================
// 3. ItemMod parsing (5 tests)
// =====================================================================

#[test]
fn mod_empty_declaration() {
    let m: ItemMod = parse_quote! {
        mod empty;
    };
    assert_eq!(m.ident.to_string(), "empty");
    assert!(m.content.is_none());
}

#[test]
fn mod_with_items() {
    let m: ItemMod = parse_quote! {
        mod inner {
            struct Foo;
            fn bar() {}
        }
    };
    assert_eq!(m.ident.to_string(), "inner");
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn mod_nested() {
    let m: ItemMod = parse_quote! {
        mod outer {
            mod middle {
                struct Deep;
            }
        }
    };
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn mod_pub_visibility() {
    let m: ItemMod = parse_quote! {
        pub mod api {
            pub fn handler() {}
        }
    };
    assert!(matches!(m.vis, Visibility::Public(_)));
}

#[test]
fn mod_with_attributes() {
    let m: ItemMod = parse_quote! {
        #[cfg(test)]
        mod tests {
            fn test_something() {}
        }
    };
    assert!(!m.attrs.is_empty());
}

// =====================================================================
// 4. Fields extraction (8 tests)
// =====================================================================

#[test]
fn fields_named_iteration() {
    let s: ItemStruct = parse_quote! {
        struct Data {
            alpha: i32,
            beta: String,
            gamma: bool,
        }
    };
    let names: Vec<String> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(names, ["alpha", "beta", "gamma"]);
}

#[test]
fn fields_unnamed_count() {
    let s: ItemStruct = parse_quote! {
        struct Triple(u8, u16, u32);
    };
    assert_eq!(s.fields.len(), 3);
}

#[test]
fn fields_unit_is_empty() {
    let s: ItemStruct = parse_quote! { struct Empty; };
    assert_eq!(s.fields.len(), 0);
    assert!(s.fields.iter().next().is_none());
}

#[test]
fn fields_type_extraction() {
    let s: ItemStruct = parse_quote! {
        struct Typed {
            count: usize,
            label: String,
        }
    };
    let types: Vec<String> = s
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(types, ["usize", "String"]);
}

#[test]
fn fields_visibility_per_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Mixed {
            pub visible: i32,
            hidden: i32,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert!(matches!(fields[0].vis, Visibility::Public(_)));
    assert!(matches!(fields[1].vis, Visibility::Inherited));
}

#[test]
fn fields_named_from_enum_variant() {
    let e: ItemEnum = parse_quote! {
        enum Msg {
            Text { content: String, sender: String },
        }
    };
    let variant = &e.variants[0];
    match &variant.fields {
        Fields::Named(f) => {
            let names: Vec<String> = f
                .named
                .iter()
                .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
                .collect();
            assert_eq!(names, ["content", "sender"]);
        }
        _ => panic!("Expected named fields"),
    }
}

#[test]
fn fields_unnamed_from_enum_variant() {
    let e: ItemEnum = parse_quote! {
        enum Wrapper {
            Single(i32),
            Double(i32, i32),
        }
    };
    assert_eq!(e.variants[0].fields.len(), 1);
    assert_eq!(e.variants[1].fields.len(), 2);
}

#[test]
fn fields_generic_type_preserved() {
    let s: ItemStruct = parse_quote! {
        struct Holder<T> {
            inner: Vec<T>,
        }
    };
    let ty = s
        .fields
        .iter()
        .next()
        .unwrap()
        .ty
        .to_token_stream()
        .to_string();
    assert!(ty.contains("Vec"));
    assert!(ty.contains("T"));
}

// =====================================================================
// 5. Generics handling (5 tests)
// =====================================================================

#[test]
fn generics_lifetime_param() {
    let s: ItemStruct = parse_quote! {
        struct Ref<'a> {
            data: &'a str,
        }
    };
    let param = s.generics.params.first().unwrap();
    match param {
        syn::GenericParam::Lifetime(lt) => {
            assert_eq!(lt.lifetime.ident.to_string(), "a");
        }
        _ => panic!("Expected lifetime parameter"),
    }
}

#[test]
fn generics_type_param_with_bound() {
    let s: ItemStruct = parse_quote! {
        struct Bounded<T: Clone + Send> {
            val: T,
        }
    };
    let param = s.generics.params.first().unwrap();
    match param {
        syn::GenericParam::Type(tp) => {
            assert_eq!(tp.ident.to_string(), "T");
            assert!(!tp.bounds.is_empty());
        }
        _ => panic!("Expected type parameter"),
    }
}

#[test]
fn generics_where_clause() {
    let s: ItemStruct = parse_quote! {
        struct Constrained<T>
        where
            T: std::fmt::Debug,
        {
            item: T,
        }
    };
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn generics_const_param() {
    let s: ItemStruct = parse_quote! {
        struct FixedArray<const N: usize> {
            data: [u8; N],
        }
    };
    let param = s.generics.params.first().unwrap();
    assert!(matches!(param, syn::GenericParam::Const(_)));
}

#[test]
fn generics_split_for_impl() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper<'a, T: Clone>
        where
            T: Send,
        {
            inner: &'a T,
        }
    };
    let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();
    let impl_str = impl_generics.to_token_stream().to_string();
    let ty_str = ty_generics.to_token_stream().to_string();
    assert!(impl_str.contains("'a"));
    assert!(ty_str.contains("T"));
    assert!(where_clause.is_some());
}

// =====================================================================
// 6. Visibility patterns (5 tests)
// =====================================================================

#[test]
fn visibility_public() {
    let s: ItemStruct = parse_quote! { pub struct Pub; };
    assert!(matches!(s.vis, Visibility::Public(_)));
}

#[test]
fn visibility_inherited_is_private() {
    let s: ItemStruct = parse_quote! { struct Priv; };
    assert!(matches!(s.vis, Visibility::Inherited));
}

#[test]
fn visibility_pub_crate() {
    let s: ItemStruct = parse_quote! { pub(crate) struct Crated; };
    match &s.vis {
        Visibility::Restricted(r) => {
            assert_eq!(r.path.to_token_stream().to_string(), "crate");
        }
        _ => panic!("Expected pub(crate)"),
    }
}

#[test]
fn visibility_pub_super() {
    let s: ItemStruct = parse_quote! { pub(super) struct Parent; };
    match &s.vis {
        Visibility::Restricted(r) => {
            assert_eq!(r.path.to_token_stream().to_string(), "super");
        }
        _ => panic!("Expected pub(super)"),
    }
}

#[test]
fn visibility_roundtrip_tokens() {
    let s: ItemStruct = parse_quote! { pub(in crate::inner) struct Scoped; };
    let vis_str = s.vis.to_token_stream().to_string();
    assert!(vis_str.contains("crate"));
    assert!(vis_str.contains("inner"));
}

// =====================================================================
// 7. TokenStream transformation (8 tests)
// =====================================================================

#[test]
fn transform_add_field_to_struct() {
    let mut s: ItemStruct = parse_quote! {
        struct Data {
            existing: i32,
        }
    };
    if let Fields::Named(ref mut fields) = s.fields {
        let new_field: syn::Field = parse_quote! { added: String };
        fields.named.push(new_field);
    }
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn transform_rename_struct() {
    let mut s: ItemStruct = parse_quote! { struct Original; };
    s.ident = format_ident!("Renamed");
    assert_eq!(s.ident.to_string(), "Renamed");
}

#[test]
fn transform_add_derive_attribute() {
    let mut s: ItemStruct = parse_quote! { struct Plain; };
    let attr: syn::Attribute = parse_quote! { #[derive(Debug, Clone)] };
    s.attrs.push(attr);
    assert_eq!(s.attrs.len(), 1);
    let attr_str = s.attrs[0].to_token_stream().to_string();
    assert!(attr_str.contains("Debug"));
}

#[test]
fn transform_change_visibility() {
    let mut s: ItemStruct = parse_quote! { struct Hidden; };
    assert!(matches!(s.vis, Visibility::Inherited));
    s.vis = parse_quote! { pub };
    assert!(matches!(s.vis, Visibility::Public(_)));
}

#[test]
fn transform_add_variant_to_enum() {
    let mut e: ItemEnum = parse_quote! {
        enum Color {
            Red,
            Green,
        }
    };
    let new_variant: syn::Variant = parse_quote! { Blue };
    e.variants.push(new_variant);
    assert_eq!(e.variants.len(), 3);
    assert_eq!(e.variants[2].ident.to_string(), "Blue");
}

#[test]
fn transform_wrap_struct_in_mod() {
    let s: ItemStruct = parse_quote! { struct Inner; };
    let mod_name = format_ident!("wrapper");
    let wrapped = quote! {
        mod #mod_name {
            #s
        }
    };
    let m: ItemMod = syn::parse2(wrapped).expect("Failed to parse module");
    assert_eq!(m.ident.to_string(), "wrapper");
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn transform_struct_to_token_stream_roundtrip() {
    let original: ItemStruct = parse_quote! {
        pub struct Roundtrip<T> {
            value: T,
        }
    };
    let tokens = original.to_token_stream();
    let parsed: ItemStruct = syn::parse2(tokens).expect("Roundtrip failed");
    assert_eq!(parsed.ident.to_string(), "Roundtrip");
    assert_eq!(parsed.generics.params.len(), 1);
}

#[test]
fn transform_generate_impl_block() {
    let s: ItemStruct = parse_quote! {
        struct Counter {
            count: usize,
        }
    };
    let name = &s.ident;
    let impl_block = quote! {
        impl #name {
            fn new() -> Self {
                Self { count: 0 }
            }
        }
    };
    let parsed: ItemImpl = syn::parse2(impl_block).expect("Failed to parse impl");
    assert!(parsed.trait_.is_none());
    assert_eq!(parsed.items.len(), 1);
}

// =====================================================================
// 8. Edge cases (8 tests)
// =====================================================================

#[test]
fn edge_empty_named_struct() {
    let s: ItemStruct = parse_quote! { struct Empty {} };
    assert!(matches!(s.fields, Fields::Named(_)));
    assert_eq!(s.fields.len(), 0);
}

#[test]
fn edge_empty_tuple_struct() {
    let s: ItemStruct = parse_quote! { struct Empty(); };
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    assert_eq!(s.fields.len(), 0);
}

#[test]
fn edge_complex_nested_generics() {
    let s: ItemStruct = parse_quote! {
        struct Complex<T: Iterator<Item = Vec<u8>>> {
            data: T,
        }
    };
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn edge_multiple_attributes() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        #[cfg(feature = "test")]
        struct MultiAttr;
    };
    assert_eq!(s.attrs.len(), 3);
}

#[test]
fn edge_item_fn_parsing() {
    let f: ItemFn = parse_quote! {
        fn compute(input: &str) -> Result<i32, String> {
            Ok(42)
        }
    };
    assert_eq!(f.sig.ident.to_string(), "compute");
    assert_eq!(f.sig.inputs.len(), 1);
    assert!(
        f.sig
            .output
            .to_token_stream()
            .to_string()
            .contains("Result")
    );
}

#[test]
fn edge_item_trait_parsing() {
    let t: ItemTrait = parse_quote! {
        trait Parseable {
            fn parse(input: &str) -> Self;
        }
    };
    assert_eq!(t.ident.to_string(), "Parseable");
    assert_eq!(t.items.len(), 1);
}

#[test]
fn edge_item_impl_with_trait() {
    let i: ItemImpl = parse_quote! {
        impl std::fmt::Display for MyType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MyType")
            }
        }
    };
    assert!(i.trait_.is_some());
    assert_eq!(i.items.len(), 1);
}

#[test]
fn edge_parse_str_struct() {
    let s: ItemStruct = parse_str("struct FromStr { field: u64 }").expect("parse_str failed");
    assert_eq!(s.ident.to_string(), "FromStr");
    assert_eq!(s.fields.len(), 1);
}

// =====================================================================
// Additional coverage tests
// =====================================================================

#[test]
fn additional_format_ident_span() {
    let ident = format_ident!("dynamic_name");
    assert_eq!(ident.to_string(), "dynamic_name");
}

#[test]
fn additional_span_call_site() {
    let span = Span::call_site();
    let ident = proc_macro2::Ident::new("test_ident", span);
    assert_eq!(ident.to_string(), "test_ident");
}

#[test]
fn additional_generics_default_is_empty() {
    let g = Generics::default();
    assert!(g.params.is_empty());
    assert!(g.where_clause.is_none());
}

#[test]
fn additional_enum_no_variants_parses() {
    let e: ItemEnum = parse_quote! { enum Nothing {} };
    assert!(e.variants.is_empty());
}

#[test]
fn additional_struct_with_phantom() {
    let s: ItemStruct = parse_quote! {
        struct Tagged<T> {
            _marker: std::marker::PhantomData<T>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ident.as_ref().unwrap().to_string(), "_marker");
}
