//! Comprehensive tests for token stream patterns and attribute handling
//! used in proc-macro development for adze.
//!
//! Covers:
//!   - DeriveInput parsing patterns (struct/enum/unit, fields, attributes)
//!   - Attribute value extraction (string/int/bool/path values)
//!   - Token stream generation with quote! (struct/enum/impl/fn bodies)
//!   - Type introspection patterns (Option<T>, Vec<T>, Box<T> detection)
//!   - Field pattern matching (named, unnamed, unit fields)
//!   - Ident construction and comparison
//!   - Edge cases (empty structs, empty enums, generic types)

use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Fields, GenericArgument, ItemEnum, ItemImpl, ItemStruct,
    Lit, Meta, PathArguments, Type, TypePath, parse_quote, parse2,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_derive(tokens: TokenStream) -> DeriveInput {
    parse2::<DeriveInput>(tokens).expect("failed to parse DeriveInput")
}

fn field_types(fields: &Fields) -> Vec<String> {
    fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn field_names(fields: &Fields) -> Vec<Option<String>> {
    fields
        .iter()
        .map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect()
}

/// Extract the inner type from `Option<T>`, `Vec<T>`, or `Box<T>`.
fn extract_inner_type<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(seg) = path.segments.last()
        && seg.ident == wrapper
        && let PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

fn is_wrapper_type(ty: &Type, wrapper: &str) -> bool {
    extract_inner_type(ty, wrapper).is_some()
}

fn attr_path_name(attr: &Attribute) -> String {
    attr.path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 1: DeriveInput parsing patterns (10 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn derive_input_named_struct() {
    let di = parse_derive(quote! {
        struct Foo {
            x: i32,
            y: String,
        }
    });
    assert_eq!(di.ident, "Foo");
    assert!(matches!(di.data, syn::Data::Struct(ref s) if matches!(s.fields, Fields::Named(_))));
    if let syn::Data::Struct(ref s) = di.data {
        assert_eq!(s.fields.len(), 2);
    }
}

#[test]
fn derive_input_tuple_struct() {
    let di = parse_derive(quote! {
        struct Pair(i32, i32);
    });
    assert_eq!(di.ident, "Pair");
    if let syn::Data::Struct(ref s) = di.data {
        assert!(matches!(s.fields, Fields::Unnamed(_)));
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_unit_struct() {
    let di = parse_derive(quote! {
        struct Marker;
    });
    assert_eq!(di.ident, "Marker");
    if let syn::Data::Struct(ref s) = di.data {
        assert!(matches!(s.fields, Fields::Unit));
        assert_eq!(s.fields.len(), 0);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_simple_enum() {
    let di = parse_derive(quote! {
        enum Color { Red, Green, Blue }
    });
    assert_eq!(di.ident, "Color");
    if let syn::Data::Enum(ref e) = di.data {
        assert_eq!(e.variants.len(), 3);
        let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        assert_eq!(names, vec!["Red", "Green", "Blue"]);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn derive_input_enum_with_data() {
    let di = parse_derive(quote! {
        enum Shape {
            Circle(f64),
            Rect { w: f64, h: f64 },
            Point,
        }
    });
    if let syn::Data::Enum(ref e) = di.data {
        assert_eq!(e.variants.len(), 3);
        assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
        assert!(matches!(e.variants[1].fields, Fields::Named(_)));
        assert!(matches!(e.variants[2].fields, Fields::Unit));
    } else {
        panic!("expected enum");
    }
}

#[test]
fn derive_input_struct_field_names() {
    let di = parse_derive(quote! {
        struct Record {
            name: String,
            age: u32,
            active: bool,
        }
    });
    if let syn::Data::Struct(ref s) = di.data {
        let names = field_names(&s.fields);
        assert_eq!(
            names,
            vec![
                Some("name".into()),
                Some("age".into()),
                Some("active".into())
            ]
        );
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_struct_field_types() {
    let di = parse_derive(quote! {
        struct Record {
            name: String,
            count: usize,
        }
    });
    if let syn::Data::Struct(ref s) = di.data {
        let types = field_types(&s.fields);
        assert_eq!(types, vec!["String", "usize"]);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_with_derive_attr() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone)]
        struct Wrapper(u8);
    });
    assert_eq!(di.attrs.len(), 1);
    assert_eq!(attr_path_name(&di.attrs[0]), "derive");
}

#[test]
fn derive_input_with_multiple_attrs() {
    let di = parse_derive(quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Multi {
            val: i32,
        }
    });
    assert_eq!(di.attrs.len(), 2);
    let names: Vec<_> = di.attrs.iter().map(attr_path_name).collect();
    assert_eq!(names, vec!["derive", "allow"]);
}

#[test]
fn derive_input_generics() {
    let di = parse_derive(quote! {
        struct Container<T> {
            inner: T,
        }
    });
    assert_eq!(di.generics.params.len(), 1);
    assert_eq!(di.ident, "Container");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 2: Attribute value extraction (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn attr_extract_string_value() {
    let di = parse_derive(quote! {
        #[doc = "a doc string"]
        struct S;
    });
    let attr = &di.attrs[0];
    if let Meta::NameValue(nv) = &attr.meta {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) = &nv.value
        {
            assert_eq!(s.value(), "a doc string");
        } else {
            panic!("expected string literal");
        }
    } else {
        panic!("expected name-value meta");
    }
}

#[test]
fn attr_extract_path_only() {
    let di = parse_derive(quote! {
        #[cfg_attr]
        struct S;
    });
    let attr = &di.attrs[0];
    assert!(matches!(&attr.meta, Meta::Path(_)));
    assert_eq!(attr_path_name(attr), "cfg_attr");
}

#[test]
fn attr_extract_list_meta() {
    let di = parse_derive(quote! {
        #[derive(Debug, Clone)]
        struct S;
    });
    let attr = &di.attrs[0];
    assert!(matches!(&attr.meta, Meta::List(_)));
}

#[test]
fn attr_extract_nested_path_in_list() {
    let di = parse_derive(quote! {
        #[allow(unused_variables)]
        struct S;
    });
    let attr = &di.attrs[0];
    if let Meta::List(list) = &attr.meta {
        let inner = list.tokens.to_string();
        assert!(inner.contains("unused_variables"));
    } else {
        panic!("expected list meta");
    }
}

#[test]
fn attr_multiple_name_value() {
    // Simulate parsing a struct with two doc attrs
    let di = parse_derive(quote! {
        #[doc = "line one"]
        #[doc = "line two"]
        struct S;
    });
    assert_eq!(di.attrs.len(), 2);
    for attr in &di.attrs {
        assert!(matches!(&attr.meta, Meta::NameValue(_)));
    }
}

#[test]
fn attr_parse_cfg_predicate() {
    let di = parse_derive(quote! {
        #[cfg(feature = "test-api")]
        struct S;
    });
    let attr = &di.attrs[0];
    if let Meta::List(list) = &attr.meta {
        let inner = list.tokens.to_string();
        assert!(inner.contains("feature"));
        assert!(inner.contains("test-api"));
    } else {
        panic!("expected list meta");
    }
}

#[test]
fn attr_on_enum_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[doc = "variant A"]
            A,
            B,
        }
    };
    assert_eq!(e.variants[0].attrs.len(), 1);
    assert_eq!(e.variants[1].attrs.len(), 0);
}

#[test]
fn attr_on_struct_field() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[allow(dead_code)]
            field: i32,
        }
    };
    if let Fields::Named(ref fields) = s.fields {
        assert_eq!(fields.named[0].attrs.len(), 1);
        assert_eq!(attr_path_name(&fields.named[0].attrs[0]), "allow");
    } else {
        panic!("expected named fields");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 3: Token stream generation with quote! (10 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn quote_generates_struct() {
    let name = format_ident!("Foo");
    let tokens = quote! {
        struct #name {
            x: i32,
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(item.ident, "Foo");
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn quote_generates_enum() {
    let name = format_ident!("Direction");
    let variants = vec![format_ident!("Up"), format_ident!("Down")];
    let tokens = quote! {
        enum #name {
            #(#variants),*
        }
    };
    let item: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(item.ident, "Direction");
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn quote_generates_impl_block() {
    let ty = format_ident!("Foo");
    let tokens = quote! {
        impl #ty {
            fn new() -> Self {
                Self
            }
        }
    };
    let item: ItemImpl = parse2(tokens).unwrap();
    assert_eq!(item.items.len(), 1);
}

#[test]
fn quote_generates_fn_body() {
    let fname = format_ident!("compute");
    let tokens = quote! {
        fn #fname(x: i32) -> i32 {
            x + 1
        }
    };
    let item: syn::ItemFn = parse2(tokens).unwrap();
    assert_eq!(item.sig.ident, "compute");
    assert_eq!(item.sig.inputs.len(), 1);
}

#[test]
fn quote_interpolates_repeated_fields() {
    let field_names: Vec<Ident> = vec![
        format_ident!("alpha"),
        format_ident!("beta"),
        format_ident!("gamma"),
    ];
    let field_types: Vec<TokenStream> = vec![quote! { i32 }, quote! { String }, quote! { bool }];
    let tokens = quote! {
        struct Generated {
            #(#field_names: #field_types),*
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(item.fields.len(), 3);
}

#[test]
fn quote_generates_trait_impl() {
    let ty = format_ident!("MyType");
    let tokens = quote! {
        impl std::fmt::Display for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MyType")
            }
        }
    };
    let item: ItemImpl = parse2(tokens).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn quote_conditional_field_inclusion() {
    let include_extra = true;
    let extra_field = if include_extra {
        quote! { extra: bool, }
    } else {
        quote! {}
    };
    let tokens = quote! {
        struct Cond {
            base: i32,
            #extra_field
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn quote_conditional_field_exclusion() {
    let include_extra = false;
    let extra_field = if include_extra {
        quote! { extra: bool, }
    } else {
        quote! {}
    };
    let tokens = quote! {
        struct Cond {
            base: i32,
            #extra_field
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn quote_generates_where_clause() {
    let ty = format_ident!("Container");
    let tokens = quote! {
        impl<T> #ty<T> where T: Clone + Send {
            fn cloned(&self) -> T {
                unimplemented!()
            }
        }
    };
    let item: ItemImpl = parse2(tokens).unwrap();
    assert!(item.generics.where_clause.is_some());
}

#[test]
fn quote_roundtrip_preserves_structure() {
    let original: ItemStruct = parse_quote! {
        struct Roundtrip {
            a: u32,
            b: String,
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(reparsed.ident, "Roundtrip");
    assert_eq!(reparsed.fields.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 4: Type introspection patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_detect_option() {
    let ty: Type = parse_quote!(Option<String>);
    assert!(is_wrapper_type(&ty, "Option"));
    assert!(!is_wrapper_type(&ty, "Vec"));
}

#[test]
fn type_detect_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert!(is_wrapper_type(&ty, "Vec"));
    assert!(!is_wrapper_type(&ty, "Option"));
}

#[test]
fn type_detect_box() {
    let ty: Type = parse_quote!(Box<dyn Fn()>);
    assert!(is_wrapper_type(&ty, "Box"));
}

#[test]
fn type_extract_option_inner() {
    let ty: Type = parse_quote!(Option<u64>);
    let inner = extract_inner_type(&ty, "Option").unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "u64");
}

#[test]
fn type_extract_vec_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let inner = extract_inner_type(&ty, "Vec").unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn type_plain_is_not_wrapper() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_wrapper_type(&ty, "Option"));
    assert!(!is_wrapper_type(&ty, "Vec"));
    assert!(!is_wrapper_type(&ty, "Box"));
}

#[test]
fn type_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    assert!(is_wrapper_type(&ty, "Option"));
    let inner = extract_inner_type(&ty, "Option").unwrap();
    assert!(is_wrapper_type(inner, "Vec"));
    let innermost = extract_inner_type(inner, "Vec").unwrap();
    assert_eq!(innermost.to_token_stream().to_string(), "u8");
}

#[test]
fn type_reference_not_detected_as_wrapper() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_wrapper_type(&ty, "Option"));
    assert!(!is_wrapper_type(&ty, "Vec"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 5: Field pattern matching (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn fields_named_iteration() {
    let s: ItemStruct = parse_quote! {
        struct S { a: i32, b: String, c: bool }
    };
    let names: Vec<_> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn fields_unnamed_count() {
    let s: ItemStruct = parse_quote! {
        struct Tup(u8, u16, u32, u64);
    };
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    assert_eq!(s.fields.len(), 4);
}

#[test]
fn fields_unit_is_empty() {
    let s: ItemStruct = parse_quote! {
        struct Unit;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert_eq!(s.fields.len(), 0);
    assert!(s.fields.iter().next().is_none());
}

#[test]
fn fields_named_vs_unnamed_discrimination() {
    let named: ItemStruct = parse_quote! { struct A { x: i32 } };
    let unnamed: ItemStruct = parse_quote! { struct B(i32); };
    let unit: ItemStruct = parse_quote! { struct C; };

    assert!(matches!(named.fields, Fields::Named(_)));
    assert!(matches!(unnamed.fields, Fields::Unnamed(_)));
    assert!(matches!(unit.fields, Fields::Unit));
}

#[test]
fn fields_named_with_visibility() {
    let s: ItemStruct = parse_quote! {
        struct Vis {
            pub x: i32,
            y: i32,
        }
    };
    if let Fields::Named(ref fields) = s.fields {
        let first_vis = fields.named[0].vis.to_token_stream().to_string();
        let second_vis = fields.named[1].vis.to_token_stream().to_string();
        assert!(first_vis.contains("pub"));
        assert!(second_vis.is_empty());
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn fields_enum_variant_named() {
    let e: ItemEnum = parse_quote! {
        enum E {
            V { x: i32, y: i32 },
        }
    };
    let variant = &e.variants[0];
    assert!(matches!(variant.fields, Fields::Named(_)));
    assert_eq!(variant.fields.len(), 2);
}

#[test]
fn fields_enum_variant_unnamed() {
    let e: ItemEnum = parse_quote! {
        enum E {
            Wrap(String),
        }
    };
    let variant = &e.variants[0];
    assert!(matches!(variant.fields, Fields::Unnamed(_)));
    assert_eq!(variant.fields.len(), 1);
}

#[test]
fn fields_enum_variant_unit() {
    let e: ItemEnum = parse_quote! {
        enum E {
            Empty,
        }
    };
    let variant = &e.variants[0];
    assert!(matches!(variant.fields, Fields::Unit));
    assert_eq!(variant.fields.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 6: Ident construction and comparison (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn ident_format_ident_basic() {
    let id = format_ident!("my_field");
    assert_eq!(id.to_string(), "my_field");
}

#[test]
fn ident_format_ident_with_suffix() {
    let base = "parse";
    let id = format_ident!("{}_impl", base);
    assert_eq!(id.to_string(), "parse_impl");
}

#[test]
fn ident_format_ident_with_number() {
    let id = format_ident!("field_{}", 42_usize);
    assert_eq!(id.to_string(), "field_42");
}

#[test]
fn ident_from_span() {
    let id1 = Ident::new("alpha", Span::call_site());
    let id2 = Ident::new("alpha", Span::call_site());
    // Idents with same string compare equal regardless of span
    assert_eq!(id1, id2);
}

#[test]
fn ident_inequality() {
    let id1 = Ident::new("foo", Span::call_site());
    let id2 = Ident::new("bar", Span::call_site());
    assert_ne!(id1, id2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 7: Edge cases (6 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_struct_named() {
    let di = parse_derive(quote! {
        struct Empty {}
    });
    if let syn::Data::Struct(ref s) = di.data {
        assert!(matches!(s.fields, Fields::Named(_)));
        assert_eq!(s.fields.len(), 0);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn edge_empty_enum() {
    let di = parse_derive(quote! {
        enum Never {}
    });
    if let syn::Data::Enum(ref e) = di.data {
        assert_eq!(e.variants.len(), 0);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn edge_generic_struct_multiple_params() {
    let di = parse_derive(quote! {
        struct Multi<A, B, C> {
            a: A,
            b: B,
            c: C,
        }
    });
    assert_eq!(di.generics.params.len(), 3);
    if let syn::Data::Struct(ref s) = di.data {
        assert_eq!(s.fields.len(), 3);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn edge_generic_with_lifetime() {
    let di = parse_derive(quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    });
    assert_eq!(di.generics.params.len(), 1);
    let param = &di.generics.params[0];
    assert!(matches!(param, syn::GenericParam::Lifetime(_)));
}

#[test]
fn edge_generic_with_bounds() {
    let di = parse_derive(quote! {
        struct Bounded<T: Clone + Send> {
            val: T,
        }
    });
    assert_eq!(di.generics.params.len(), 1);
    if let syn::GenericParam::Type(tp) = &di.generics.params[0] {
        assert!(!tp.bounds.is_empty());
    } else {
        panic!("expected type param");
    }
}

#[test]
fn edge_tuple_struct_single_field() {
    let di = parse_derive(quote! {
        struct Wrapper(Vec<u8>);
    });
    if let syn::Data::Struct(ref s) = di.data {
        assert!(matches!(s.fields, Fields::Unnamed(_)));
        assert_eq!(s.fields.len(), 1);
        let ty = &s.fields.iter().next().unwrap().ty;
        assert!(is_wrapper_type(ty, "Vec"));
    } else {
        panic!("expected struct");
    }
}
