//! Comprehensive tests for derive macro expansion patterns using proc_macro2/syn/quote.
//!
//! Covers: DeriveInput parsing, field extraction, variant extraction, attribute filtering,
//! generic parameter extraction, impl block generation, where clauses, struct body kinds,
//! discriminant values, and nested type resolution.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, FieldsNamed, GenericParam, Ident, Type, Visibility, parse_quote,
    parse2,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn parse_derive(tokens: TokenStream) -> DeriveInput {
    parse2::<DeriveInput>(tokens).expect("failed to parse DeriveInput")
}

fn field_names(fields: &FieldsNamed) -> Vec<String> {
    fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect()
}

fn variant_names(input: &DeriveInput) -> Vec<String> {
    match &input.data {
        Data::Enum(e) => e.variants.iter().map(|v| v.ident.to_string()).collect(),
        _ => panic!("expected enum"),
    }
}

fn named_fields(input: &DeriveInput) -> &FieldsNamed {
    match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => n,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    }
}

fn type_string(ty: &Type) -> String {
    quote!(#ty).to_string().replace(" ", "")
}

fn attr_paths(input: &DeriveInput) -> Vec<String> {
    input
        .attrs
        .iter()
        .map(|a| {
            a.path()
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        })
        .collect()
}

// =====================================================================
// 1. DeriveInput parsing with various derives
// =====================================================================

#[test]
fn derive_input_single_derive() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        struct Foo;
    });
    assert_eq!(di.ident.to_string(), "Foo");
    assert_eq!(di.attrs.len(), 1);
}

#[test]
fn derive_input_multiple_derives() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone, PartialEq)]
        struct Bar { x: i32 }
    });
    assert_eq!(di.ident.to_string(), "Bar");
    assert_eq!(di.attrs.len(), 1);
}

#[test]
fn derive_input_separate_derive_attrs() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        #[derive(Clone)]
        struct Baz;
    });
    assert_eq!(di.attrs.len(), 2);
}

#[test]
fn derive_input_enum() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        enum Color { Red, Green, Blue }
    });
    assert!(matches!(di.data, Data::Enum(_)));
}

#[test]
fn derive_input_union() {
    let di = parse_derive(quote! {
        #[derive(Copy, Clone)]
        union MyUnion { i: i32, f: f32 }
    });
    assert!(matches!(di.data, Data::Union(_)));
}

#[test]
fn derive_input_with_doc_and_derive() {
    let di = parse_derive(quote! {
        /// Documentation
        #[derive(Debug)]
        struct Documented { val: u8 }
    });
    assert_eq!(di.attrs.len(), 2);
}

#[test]
fn derive_input_custom_derive_path() {
    let di = parse_derive(quote! {
        #[derive(serde::Serialize)]
        struct Ser { data: String }
    });
    assert_eq!(di.ident.to_string(), "Ser");
}

// =====================================================================
// 2. Field extraction from structs
// =====================================================================

#[test]
fn extract_named_field_names() {
    let di = parse_derive(quote! {
        struct Point { x: f64, y: f64, z: f64 }
    });
    let names = field_names(named_fields(&di));
    assert_eq!(names, vec!["x", "y", "z"]);
}

#[test]
fn extract_field_types() {
    let di = parse_derive(quote! {
        struct Mixed { name: String, age: u32, active: bool }
    });
    let fields = named_fields(&di);
    let types: Vec<String> = fields.named.iter().map(|f| type_string(&f.ty)).collect();
    assert_eq!(types, vec!["String", "u32", "bool"]);
}

#[test]
fn extract_field_visibility() {
    let di = parse_derive(quote! {
        struct Vis { pub a: i32, b: i32 }
    });
    let fields = named_fields(&di);
    let f: Vec<_> = fields.named.iter().collect();
    assert!(matches!(f[0].vis, Visibility::Public(_)));
    assert!(matches!(f[1].vis, Visibility::Inherited));
}

#[test]
fn extract_field_attrs() {
    let di = parse_derive(quote! {
        struct WithAttr {
            #[serde(rename = "ID")]
            id: u64,
        }
    });
    let fields = named_fields(&di);
    let f = fields.named.first().unwrap();
    assert_eq!(f.attrs.len(), 1);
}

#[test]
fn extract_tuple_field_count() {
    let di = parse_derive(quote! { struct Tup(i32, i32, i32); });
    match &di.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(u) => assert_eq!(u.unnamed.len(), 3),
            _ => panic!("expected unnamed"),
        },
        _ => panic!("expected struct"),
    }
}

#[test]
fn extract_option_field_type() {
    let di = parse_derive(quote! {
        struct Opt { value: Option<String> }
    });
    let fields = named_fields(&di);
    let ty = type_string(&fields.named.first().unwrap().ty);
    assert_eq!(ty, "Option<String>");
}

#[test]
fn extract_vec_field_type() {
    let di = parse_derive(quote! {
        struct Items { list: Vec<u32> }
    });
    let fields = named_fields(&di);
    let ty = type_string(&fields.named.first().unwrap().ty);
    assert_eq!(ty, "Vec<u32>");
}

#[test]
fn extract_many_fields() {
    let di = parse_derive(quote! {
        struct Big { a: u8, b: u16, c: u32, d: u64, e: u128, f: i8, g: i16, h: i32 }
    });
    let names = field_names(named_fields(&di));
    assert_eq!(names.len(), 8);
}

// =====================================================================
// 3. Variant extraction from enums
// =====================================================================

#[test]
fn extract_variant_names() {
    let di = parse_derive(quote! {
        enum Dir { North, South, East, West }
    });
    assert_eq!(variant_names(&di), vec!["North", "South", "East", "West"]);
}

#[test]
fn variant_with_named_fields() {
    let di = parse_derive(quote! {
        enum Shape { Circle { radius: f64 }, Rect { w: f64, h: f64 } }
    });
    if let Data::Enum(e) = &di.data {
        match &e.variants[0].fields {
            Fields::Named(n) => assert_eq!(n.named.len(), 1),
            _ => panic!("expected named"),
        }
        match &e.variants[1].fields {
            Fields::Named(n) => assert_eq!(n.named.len(), 2),
            _ => panic!("expected named"),
        }
    } else {
        panic!("expected enum");
    }
}

#[test]
fn variant_with_unnamed_fields() {
    let di = parse_derive(quote! {
        enum Wrapper { Int(i32), Str(String), Pair(i32, i32) }
    });
    if let Data::Enum(e) = &di.data {
        match &e.variants[2].fields {
            Fields::Unnamed(u) => assert_eq!(u.unnamed.len(), 2),
            _ => panic!("expected unnamed"),
        }
    } else {
        panic!("expected enum");
    }
}

#[test]
fn variant_unit_and_tuple_mixed() {
    let di = parse_derive(quote! {
        enum Mixed { Unit, Tuple(i32), Named { x: i32 } }
    });
    if let Data::Enum(e) = &di.data {
        assert!(matches!(e.variants[0].fields, Fields::Unit));
        assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
        assert!(matches!(e.variants[2].fields, Fields::Named(_)));
    } else {
        panic!("expected enum");
    }
}

#[test]
fn variant_attrs() {
    let di = parse_derive(quote! {
        enum E {
            #[doc = "first"]
            A,
            #[cfg(test)]
            B,
        }
    });
    if let Data::Enum(e) = &di.data {
        assert_eq!(e.variants[0].attrs.len(), 1);
        assert_eq!(e.variants[1].attrs.len(), 1);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn variant_count_many() {
    let di = parse_derive(quote! {
        enum Big { A, B, C, D, E, F, G, H, I, J }
    });
    assert_eq!(variant_names(&di).len(), 10);
}

// =====================================================================
// 4. Attribute filtering
// =====================================================================

#[test]
fn filter_derive_attrs() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        #[derive(Clone)]
        struct S;
    });
    let derive_count = di
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(derive_count, 2);
}

#[test]
fn filter_doc_attrs() {
    let di = parse_derive(quote! {
        /// Doc line 1
        /// Doc line 2
        #[derive(Debug)]
        struct S;
    });
    let doc_count = di.attrs.iter().filter(|a| a.path().is_ident("doc")).count();
    assert_eq!(doc_count, 2);
}

#[test]
fn filter_cfg_attrs() {
    let di = parse_derive(quote! {
        #[cfg(target_os = "linux")]
        #[derive(Debug)]
        struct S;
    });
    let cfg_count = di.attrs.iter().filter(|a| a.path().is_ident("cfg")).count();
    assert_eq!(cfg_count, 1);
}

#[test]
fn filter_custom_attrs() {
    let di = parse_derive(quote! {
        #[my_custom(value = "hello")]
        #[derive(Debug)]
        #[another_attr]
        struct S;
    });
    let non_derive: Vec<_> = di
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("derive"))
        .collect();
    assert_eq!(non_derive.len(), 2);
}

#[test]
fn attr_path_segments() {
    let di = parse_derive(quote! {
        #[serde(rename_all = "camelCase")]
        struct S { x: i32 }
    });
    let paths = attr_paths(&di);
    assert_eq!(paths, vec!["serde"]);
}

#[test]
fn field_attr_filtering() {
    let di = parse_derive(quote! {
        struct S {
            #[serde(skip)]
            #[doc = "hidden"]
            secret: String,
            visible: String,
        }
    });
    let fields = named_fields(&di);
    let f: Vec<_> = fields.named.iter().collect();
    assert_eq!(f[0].attrs.len(), 2);
    assert_eq!(f[1].attrs.len(), 0);
}

// =====================================================================
// 5. Generic parameters extraction
// =====================================================================

#[test]
fn extract_type_param() {
    let di = parse_derive(quote! { struct Container<T> { inner: T } });
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(
        di.generics.params.first().unwrap(),
        GenericParam::Type(_)
    ));
}

#[test]
fn extract_lifetime_param() {
    let di = parse_derive(quote! { struct Ref<'a> { data: &'a str } });
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(
        di.generics.params.first().unwrap(),
        GenericParam::Lifetime(_)
    ));
}

#[test]
fn extract_const_param() {
    let di = parse_derive(quote! { struct Arr<const N: usize> { data: [u8; N] } });
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(
        di.generics.params.first().unwrap(),
        GenericParam::Const(_)
    ));
}

#[test]
fn extract_mixed_generics() {
    let di = parse_derive(quote! {
        struct Complex<'a, T, U: Clone, const N: usize> {
            r: &'a T,
            u: U,
            arr: [u8; N],
        }
    });
    assert_eq!(di.generics.params.len(), 4);
}

#[test]
fn extract_type_param_bounds() {
    let di = parse_derive(quote! {
        struct Bounded<T: Clone + Send> { val: T }
    });
    if let GenericParam::Type(tp) = di.generics.params.first().unwrap() {
        assert_eq!(tp.bounds.len(), 2);
    } else {
        panic!("expected type param");
    }
}

#[test]
fn generics_split_for_impl() {
    let di = parse_derive(quote! {
        struct G<T: Clone> { val: T }
    });
    let (impl_gen, ty_gen, where_cl) = di.generics.split_for_impl();
    let generated = quote! {
        impl #impl_gen MyTrait for G #ty_gen #where_cl {
            fn method(&self) {}
        }
    };
    let text = generated.to_string();
    assert!(text.contains("MyTrait"));
    assert!(text.contains("for G"));
}

#[test]
fn no_generics() {
    let di = parse_derive(quote! { struct Simple { x: i32 } });
    assert!(di.generics.params.is_empty());
}

#[test]
fn multiple_type_params() {
    let di = parse_derive(quote! { struct Pair<A, B> { a: A, b: B } });
    assert_eq!(di.generics.params.len(), 2);
}

// =====================================================================
// 6. impl block generation with quote!
// =====================================================================

#[test]
fn generate_simple_impl() {
    let name = format_ident!("Foo");
    let tokens = quote! {
        impl #name {
            fn new() -> Self { Self }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("impl Foo"));
    assert!(text.contains("fn new"));
}

#[test]
fn generate_trait_impl() {
    let name = format_ident!("Bar");
    let tokens = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Bar")
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("Display for Bar"));
}

#[test]
fn generate_impl_with_generics() {
    let di = parse_derive(quote! { struct W<T> { inner: T } });
    let name = &di.ident;
    let (impl_gen, ty_gen, where_cl) = di.generics.split_for_impl();
    let tokens = quote! {
        impl #impl_gen Default for #name #ty_gen #where_cl {
            fn default() -> Self {
                todo!()
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("impl < T >"));
    assert!(text.contains("for W < T >"));
}

#[test]
fn generate_impl_from_field_names() {
    let di = parse_derive(quote! { struct Config { width: u32, height: u32 } });
    let fields = named_fields(&di);
    let getters: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let fty = &f.ty;
            quote! {
                pub fn #fname(&self) -> &#fty {
                    &self.#fname
                }
            }
        })
        .collect();
    let name = &di.ident;
    let tokens = quote! {
        impl #name {
            #(#getters)*
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("fn width"));
    assert!(text.contains("fn height"));
}

#[test]
fn generate_enum_match_arms() {
    let di = parse_derive(quote! { enum Status { Ok, Err, Pending } });
    let name = &di.ident;
    let arms: Vec<TokenStream> = variant_names(&di)
        .iter()
        .map(|v| {
            let vi = format_ident!("{}", v);
            let lower = v.to_lowercase();
            quote! { #name::#vi => #lower }
        })
        .collect();
    let tokens = quote! {
        impl #name {
            fn as_str(&self) -> &'static str {
                match self {
                    #(#arms,)*
                }
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("Status :: Ok"));
    assert!(text.contains("Status :: Err"));
    assert!(text.contains("Status :: Pending"));
}

#[test]
fn generate_from_impl() {
    let name = format_ident!("Wrapper");
    let inner = format_ident!("String");
    let tokens = quote! {
        impl From<#inner> for #name {
            fn from(val: #inner) -> Self {
                #name(val)
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("From < String > for Wrapper"));
}

#[test]
fn generate_multiple_impls() {
    let name = format_ident!("Multi");
    let impls = quote! {
        impl Clone for #name {
            fn clone(&self) -> Self { todo!() }
        }
        impl Default for #name {
            fn default() -> Self { todo!() }
        }
    };
    let text = impls.to_string();
    assert!(text.contains("Clone for Multi"));
    assert!(text.contains("Default for Multi"));
}

// =====================================================================
// 7. Where clause handling
// =====================================================================

#[test]
fn parse_where_clause() {
    let di = parse_derive(quote! {
        struct S<T> where T: Clone { val: T }
    });
    assert!(di.generics.where_clause.is_some());
}

#[test]
fn where_clause_predicate_count() {
    let di = parse_derive(quote! {
        struct S<T, U> where T: Clone, U: Send + Sync { a: T, b: U }
    });
    let wc = di.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn where_clause_in_generated_impl() {
    let di = parse_derive(quote! {
        struct S<T> where T: std::fmt::Debug { val: T }
    });
    let name = &di.ident;
    let (impl_gen, ty_gen, where_cl) = di.generics.split_for_impl();
    let tokens = quote! {
        impl #impl_gen #name #ty_gen #where_cl {
            fn show(&self) { }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("where"));
    assert!(text.contains("Debug"));
}

#[test]
fn no_where_clause() {
    let di = parse_derive(quote! { struct Plain { x: i32 } });
    assert!(di.generics.where_clause.is_none());
}

#[test]
fn where_clause_with_lifetime() {
    let di = parse_derive(quote! {
        struct S<'a, T> where T: 'a { r: &'a T }
    });
    let wc = di.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_preserved_in_roundtrip() {
    let di = parse_derive(quote! {
        struct S<T> where T: Clone + Send { val: T }
    });
    let requoted = quote!(#di);
    let reparsed: DeriveInput = parse2(requoted).unwrap();
    assert!(reparsed.generics.where_clause.is_some());
}

// =====================================================================
// 8. Named vs unnamed vs unit struct bodies
// =====================================================================

#[test]
fn unit_struct_body() {
    let di = parse_derive(quote! { struct Unit; });
    assert!(matches!(di.data, Data::Struct(ref s) if matches!(s.fields, Fields::Unit)));
}

#[test]
fn named_struct_body() {
    let di = parse_derive(quote! { struct Named { a: i32, b: String } });
    match &di.data {
        Data::Struct(s) => assert!(matches!(s.fields, Fields::Named(_))),
        _ => panic!("expected struct"),
    }
}

#[test]
fn unnamed_struct_body() {
    let di = parse_derive(quote! { struct Tuple(i32, String); });
    match &di.data {
        Data::Struct(s) => assert!(matches!(s.fields, Fields::Unnamed(_))),
        _ => panic!("expected struct"),
    }
}

#[test]
fn distinguish_struct_kinds_from_data() {
    let inputs = vec![
        (quote! { struct A; }, "unit"),
        (quote! { struct B(i32); }, "unnamed"),
        (quote! { struct C { x: i32 } }, "named"),
    ];
    for (tokens, expected) in inputs {
        let di = parse_derive(tokens);
        let kind = match &di.data {
            Data::Struct(s) => match &s.fields {
                Fields::Unit => "unit",
                Fields::Unnamed(_) => "unnamed",
                Fields::Named(_) => "named",
            },
            _ => panic!("expected struct"),
        };
        assert_eq!(kind, expected, "struct {} mismatch", di.ident);
    }
}

#[test]
fn empty_named_struct() {
    let di = parse_derive(quote! { struct Empty {} });
    match &di.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => assert!(n.named.is_empty()),
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    }
}

#[test]
fn single_unnamed_field() {
    let di = parse_derive(quote! { struct Newtype(i64); });
    match &di.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(u) => assert_eq!(u.unnamed.len(), 1),
            _ => panic!("expected unnamed"),
        },
        _ => panic!("expected struct"),
    }
}

// =====================================================================
// 9. Discriminant values on enum variants
// =====================================================================

#[test]
fn explicit_discriminant() {
    let di = parse_derive(quote! {
        enum Code { Ok = 0, NotFound = 404, Error = 500 }
    });
    if let Data::Enum(e) = &di.data {
        assert!(e.variants[0].discriminant.is_some());
        assert!(e.variants[1].discriminant.is_some());
        assert!(e.variants[2].discriminant.is_some());
    } else {
        panic!("expected enum");
    }
}

#[test]
fn mixed_discriminant_presence() {
    let di = parse_derive(quote! {
        enum M { A = 1, B, C = 10, D }
    });
    if let Data::Enum(e) = &di.data {
        assert!(e.variants[0].discriminant.is_some());
        assert!(e.variants[1].discriminant.is_none());
        assert!(e.variants[2].discriminant.is_some());
        assert!(e.variants[3].discriminant.is_none());
    } else {
        panic!("expected enum");
    }
}

#[test]
fn discriminant_expr_value() {
    let di = parse_derive(quote! {
        enum V { X = 42 }
    });
    if let Data::Enum(e) = &di.data {
        let (_, expr) = e.variants[0].discriminant.as_ref().unwrap();
        let text = quote!(#expr).to_string();
        assert_eq!(text, "42");
    } else {
        panic!("expected enum");
    }
}

#[test]
fn no_discriminants() {
    let di = parse_derive(quote! { enum Simple { A, B, C } });
    if let Data::Enum(e) = &di.data {
        for v in &e.variants {
            assert!(v.discriminant.is_none());
        }
    } else {
        panic!("expected enum");
    }
}

#[test]
fn negative_discriminant() {
    let di = parse_derive(quote! {
        enum Neg { A = -1 }
    });
    if let Data::Enum(e) = &di.data {
        assert!(e.variants[0].discriminant.is_some());
    } else {
        panic!("expected enum");
    }
}

// =====================================================================
// 10. Nested type resolution
// =====================================================================

#[test]
fn nested_option_vec() {
    let di = parse_derive(quote! {
        struct S { data: Option<Vec<String>> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert_eq!(ty, "Option<Vec<String>>");
}

#[test]
fn nested_result_type() {
    let di = parse_derive(quote! {
        struct S { result: Result<Vec<u8>, String> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert_eq!(ty, "Result<Vec<u8>,String>");
}

#[test]
fn nested_hashmap_type() {
    let di = parse_derive(quote! {
        struct S { map: std::collections::HashMap<String, Vec<i32>> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("HashMap"));
    assert!(ty.contains("Vec<i32>"));
}

#[test]
fn tuple_type_field() {
    let di = parse_derive(quote! {
        struct S { pair: (i32, String) }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("i32"));
    assert!(ty.contains("String"));
}

#[test]
fn reference_type_field() {
    let di = parse_derive(quote! {
        struct S<'a> { data: &'a [u8] }
    });
    let fields = named_fields(&di);
    let ty = type_string(&fields.named.first().unwrap().ty);
    assert!(ty.contains("&"));
    assert!(ty.contains("[u8]"));
}

#[test]
fn fn_pointer_field() {
    let di = parse_derive(quote! {
        struct S { callback: fn(i32) -> bool }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("fn"));
    assert!(ty.contains("bool"));
}

#[test]
fn box_type_field() {
    let di = parse_derive(quote! {
        struct Node { children: Vec<Box<Node>> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert_eq!(ty, "Vec<Box<Node>>");
}

#[test]
fn deeply_nested_generics() {
    let di = parse_derive(quote! {
        struct Deep { val: Option<Result<Vec<Box<String>>, std::io::Error>> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("Option"));
    assert!(ty.contains("Result"));
    assert!(ty.contains("Vec"));
    assert!(ty.contains("Box<String>"));
}

// =====================================================================
// Additional edge cases and patterns
// =====================================================================

#[test]
fn format_ident_generation() {
    let base = "my_field";
    let getter = format_ident!("get_{}", base);
    let setter = format_ident!("set_{}", base);
    assert_eq!(getter.to_string(), "get_my_field");
    assert_eq!(setter.to_string(), "set_my_field");
}

#[test]
fn quote_interpolation_vec() {
    let names: Vec<Ident> = vec!["alpha", "beta", "gamma"]
        .into_iter()
        .map(|s| format_ident!("{}", s))
        .collect();
    let tokens = quote! { fn test() { #(println!("{}", #names);)* } };
    let text = tokens.to_string();
    assert!(text.contains("alpha"));
    assert!(text.contains("beta"));
    assert!(text.contains("gamma"));
}

#[test]
fn quote_repetition_with_separator() {
    let fields: Vec<Ident> = vec!["a", "b", "c"]
        .into_iter()
        .map(|s| format_ident!("{}", s))
        .collect();
    let tokens = quote! { struct S { #(#fields: i32),* } };
    let text = tokens.to_string();
    assert!(text.contains("a : i32"));
    assert!(text.contains("c : i32"));
}

#[test]
fn parse_quote_derive_input() {
    let di: DeriveInput = parse_quote! {
        #[derive(Debug)]
        struct Foo { x: i32 }
    };
    assert_eq!(di.ident.to_string(), "Foo");
}

#[test]
fn parse_quote_type() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let text = type_string(&ty);
    assert_eq!(text, "Vec<Option<String>>");
}

#[test]
fn roundtrip_struct_through_quote() {
    let original = quote! { struct S { a: i32, b: String } };
    let parsed: DeriveInput = parse2(original).unwrap();
    let requoted = quote!(#parsed);
    let reparsed: DeriveInput = parse2(requoted).unwrap();
    assert_eq!(parsed.ident, reparsed.ident);
    match (&parsed.data, &reparsed.data) {
        (Data::Struct(s1), Data::Struct(s2)) => {
            assert_eq!(s1.fields.len(), s2.fields.len());
        }
        _ => panic!("expected structs"),
    }
}

#[test]
fn roundtrip_enum_through_quote() {
    let original = quote! { enum E { A(i32), B { x: String }, C } };
    let parsed: DeriveInput = parse2(original).unwrap();
    let requoted = quote!(#parsed);
    let reparsed: DeriveInput = parse2(requoted).unwrap();
    assert_eq!(variant_names(&parsed), variant_names(&reparsed));
}

#[test]
fn generate_impl_for_each_variant() {
    let di = parse_derive(quote! { enum Animal { Cat, Dog, Fish } });
    let name = &di.ident;
    let checks: Vec<TokenStream> = variant_names(&di)
        .iter()
        .map(|v| {
            let vi = format_ident!("{}", v);
            let method = format_ident!("is_{}", v.to_lowercase());
            quote! {
                pub fn #method(&self) -> bool {
                    matches!(self, #name::#vi)
                }
            }
        })
        .collect();
    let tokens = quote! {
        impl #name {
            #(#checks)*
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("is_cat"));
    assert!(text.contains("is_dog"));
    assert!(text.contains("is_fish"));
}

#[test]
fn visibility_pub_crate() {
    let di = parse_derive(quote! {
        pub(crate) struct Internal { pub(crate) field: i32 }
    });
    assert!(matches!(di.vis, Visibility::Restricted(_)));
}

#[test]
fn enum_with_generics_roundtrip() {
    let di = parse_derive(quote! {
        enum Result2<T, E> { Ok(T), Err(E) }
    });
    assert_eq!(di.generics.params.len(), 2);
    assert_eq!(variant_names(&di), vec!["Ok", "Err"]);
    let requoted = quote!(#di);
    let reparsed: DeriveInput = parse2(requoted).unwrap();
    assert_eq!(reparsed.generics.params.len(), 2);
}

#[test]
fn generate_builder_pattern() {
    let di = parse_derive(quote! {
        struct Config { width: u32, height: u32, title: String }
    });
    let fields = named_fields(&di);
    let setters: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let fty = &f.ty;
            quote! {
                pub fn #fname(mut self, val: #fty) -> Self {
                    self.#fname = val;
                    self
                }
            }
        })
        .collect();
    let name = &di.ident;
    let tokens = quote! {
        impl #name {
            #(#setters)*
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("fn width"));
    assert!(text.contains("fn height"));
    assert!(text.contains("fn title"));
    assert!(text.contains("-> Self"));
}

#[test]
fn const_generic_with_default() {
    let di = parse_derive(quote! {
        struct Buffer<const N: usize = 1024> { data: [u8; N] }
    });
    if let GenericParam::Const(cp) = di.generics.params.first().unwrap() {
        assert_eq!(cp.ident.to_string(), "N");
        assert!(cp.default.is_some());
    } else {
        panic!("expected const param");
    }
}

#[test]
fn type_param_with_default() {
    let di = parse_derive(quote! {
        struct Alloc<A = std::alloc::Global> { alloc: A }
    });
    if let GenericParam::Type(tp) = di.generics.params.first().unwrap() {
        assert!(tp.default.is_some());
    } else {
        panic!("expected type param");
    }
}

#[test]
fn multiple_lifetime_params() {
    let di = parse_derive(quote! {
        struct Multi<'a, 'b> { a: &'a str, b: &'b str }
    });
    let lifetimes: Vec<_> = di
        .generics
        .params
        .iter()
        .filter(|p| matches!(p, GenericParam::Lifetime(_)))
        .collect();
    assert_eq!(lifetimes.len(), 2);
}

#[test]
fn struct_with_phantom_data() {
    let di = parse_derive(quote! {
        struct Tagged<T> { _marker: std::marker::PhantomData<T> }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("PhantomData"));
}

#[test]
fn enum_variant_complex_fields() {
    let di = parse_derive(quote! {
        enum Msg {
            Text { content: String, sender: u64 },
            Binary(Vec<u8>),
            Ack,
        }
    });
    if let Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 3);
        match &e.variants[0].fields {
            Fields::Named(n) => assert_eq!(n.named.len(), 2),
            _ => panic!("expected named"),
        }
    } else {
        panic!("expected enum");
    }
}

#[test]
fn generate_debug_impl() {
    let di = parse_derive(quote! { struct Point { x: f64, y: f64 } });
    let name = &di.ident;
    let name_str = name.to_string();
    let fields = named_fields(&di);
    let debug_fields: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let fname_str = fname.as_ref().unwrap().to_string();
            quote! { .field(#fname_str, &self.#fname) }
        })
        .collect();
    let tokens = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#name_str)
                    #(#debug_fields)*
                    .finish()
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("debug_struct"));
    assert!(text.contains("\"Point\""));
    assert!(text.contains("\"x\""));
    assert!(text.contains("\"y\""));
}

#[test]
fn data_kind_classification() {
    let cases: Vec<(TokenStream, &str)> = vec![
        (quote! { struct A; }, "struct"),
        (quote! { enum B { X } }, "enum"),
        (quote! { union C { x: i32 } }, "union"),
    ];
    for (tokens, expected) in cases {
        let di = parse_derive(tokens);
        let kind = match &di.data {
            Data::Struct(_) => "struct",
            Data::Enum(_) => "enum",
            Data::Union(_) => "union",
        };
        assert_eq!(kind, expected);
    }
}

#[test]
fn attr_meta_parsing() {
    let di = parse_derive(quote! {
        #[repr(C)]
        struct Repr { x: i32 }
    });
    let attr = &di.attrs[0];
    assert!(attr.path().is_ident("repr"));
}

#[test]
fn generate_impl_with_where_bounds_added() {
    let di = parse_derive(quote! { struct S<T> { val: T } });
    let name = &di.ident;
    let generics = &di.generics;
    let mut extended = generics.clone();
    let where_clause = extended.make_where_clause();
    where_clause
        .predicates
        .push(parse_quote!(T: std::fmt::Debug));
    let (impl_gen, ty_gen, where_cl) = extended.split_for_impl();
    let tokens = quote! {
        impl #impl_gen #name #ty_gen #where_cl {
            fn debug_val(&self) { }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("where"));
    assert!(text.contains("Debug"));
}

#[test]
fn enum_all_unit_variants() {
    let di = parse_derive(quote! {
        enum Level { Trace, Debug, Info, Warn, Error }
    });
    if let Data::Enum(e) = &di.data {
        assert!(e.variants.iter().all(|v| matches!(v.fields, Fields::Unit)));
    } else {
        panic!("expected enum");
    }
}

#[test]
fn struct_with_array_field() {
    let di = parse_derive(quote! {
        struct Matrix { data: [[f64; 4]; 4] }
    });
    let ty = type_string(&named_fields(&di).named.first().unwrap().ty);
    assert!(ty.contains("[f64;4]"));
}

#[test]
fn generate_into_impl_for_newtype() {
    let name = format_ident!("Id");
    let inner_ty: Type = parse_quote!(u64);
    let tokens = quote! {
        impl From<#name> for #inner_ty {
            fn from(val: #name) -> #inner_ty {
                val.0
            }
        }
    };
    let text = tokens.to_string();
    assert!(text.contains("From < Id > for u64"));
}
