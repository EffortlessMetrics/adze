//! Comprehensive attribute-parsing pattern tests for adze-macro.
//!
//! Covers syn parsing utilities used to process `#[adze::grammar]`,
//! `#[adze::language]`, `#[adze::leaf]`, and related attributes.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Expr, Field, Fields, Ident, ItemEnum, ItemStruct, Meta, Type, parse_quote, parse2,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    segments.len() == 2 && segments[0].ident == "adze" && segments[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            let segs: Vec<_> = attr.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn type_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn named_fields(s: &ItemStruct) -> Vec<&Field> {
    match &s.fields {
        Fields::Named(f) => f.named.iter().collect(),
        _ => panic!("expected named fields"),
    }
}

fn unnamed_fields(s: &ItemStruct) -> Vec<&Field> {
    match &s.fields {
        Fields::Unnamed(f) => f.unnamed.iter().collect(),
        _ => panic!("expected unnamed fields"),
    }
}

// ============================================================================
// 1. Attribute Meta Parsing (8 tests)
// ============================================================================

#[test]
fn meta_path_grammar_attribute() {
    let attr: Attribute = parse_quote!(#[adze::grammar("test")]);
    assert!(is_adze_attr(&attr, "grammar"));
}

#[test]
fn meta_path_language_attribute() {
    let attr: Attribute = parse_quote!(#[adze::language]);
    assert!(is_adze_attr(&attr, "language"));
}

#[test]
fn meta_path_leaf_attribute() {
    let attr: Attribute = parse_quote!(#[adze::leaf(text = "+")]);
    assert!(is_adze_attr(&attr, "leaf"));
}

#[test]
fn meta_path_extra_attribute() {
    let attr: Attribute = parse_quote!(#[adze::extra]);
    assert!(is_adze_attr(&attr, "extra"));
}

#[test]
fn meta_path_prec_left_attribute() {
    let attr: Attribute = parse_quote!(#[adze::prec_left(1)]);
    assert!(is_adze_attr(&attr, "prec_left"));
}

#[test]
fn meta_path_prec_right_attribute() {
    let attr: Attribute = parse_quote!(#[adze::prec_right(2)]);
    assert!(is_adze_attr(&attr, "prec_right"));
}

#[test]
fn meta_list_vs_path_discrimination() {
    let path_attr: Attribute = parse_quote!(#[adze::language]);
    let list_attr: Attribute = parse_quote!(#[adze::leaf(text = "+")]);

    assert!(matches!(path_attr.meta, Meta::Path(_)));
    assert!(matches!(list_attr.meta, Meta::List(_)));
}

#[test]
fn meta_non_adze_attribute_rejected() {
    let attr: Attribute = parse_quote!(#[derive(Debug)]);
    assert!(!is_adze_attr(&attr, "language"));
    assert!(!is_adze_attr(&attr, "grammar"));
}

// ============================================================================
// 2. Type Parsing Patterns (8 tests)
// ============================================================================

#[test]
fn type_parse_simple_ident() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(type_string(&ty), "i32");
}

#[test]
fn type_parse_box_generic() {
    let ty: Type = parse_quote!(Box<Expr>);
    assert_eq!(type_string(&ty), "Box < Expr >");
}

#[test]
fn type_parse_vec_generic() {
    let ty: Type = parse_quote!(Vec<Statement>);
    assert_eq!(type_string(&ty), "Vec < Statement >");
}

#[test]
fn type_parse_option_generic() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(type_string(&ty), "Option < String >");
}

#[test]
fn type_parse_nested_generics() {
    let ty: Type = parse_quote!(Vec<Option<Box<Expr>>>);
    assert_eq!(type_string(&ty), "Vec < Option < Box < Expr > > >");
}

#[test]
fn type_parse_tuple() {
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(type_string(&ty), "(i32 , String)");
}

#[test]
fn type_parse_unit() {
    let ty: Type = parse_quote!(());
    assert_eq!(type_string(&ty), "()");
}

#[test]
fn type_parse_reference() {
    let ty: Type = parse_quote!(&'static str);
    assert_eq!(type_string(&ty), "& 'static str");
}

// ============================================================================
// 3. Struct Field Extraction (8 tests)
// ============================================================================

#[test]
fn struct_named_field_count() {
    let s = parse_struct(quote! {
        struct Node {
            left: Box<Node>,
            right: Box<Node>,
            value: i32,
        }
    });
    assert_eq!(named_fields(&s).len(), 3);
}

#[test]
fn struct_named_field_names() {
    let s = parse_struct(quote! {
        struct Point { x: f64, y: f64, z: f64 }
    });
    let names: Vec<_> = named_fields(&s)
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, ["x", "y", "z"]);
}

#[test]
fn struct_named_field_types() {
    let s = parse_struct(quote! {
        struct Mixed { count: usize, label: String, active: bool }
    });
    let types: Vec<_> = named_fields(&s)
        .iter()
        .map(|f| type_string(&f.ty))
        .collect();
    assert_eq!(types, ["usize", "String", "bool"]);
}

#[test]
fn struct_tuple_field_extraction() {
    let s = parse_struct(quote! {
        struct Wrapper(i32, String);
    });
    let fields = unnamed_fields(&s);
    assert_eq!(fields.len(), 2);
    assert_eq!(type_string(&fields[0].ty), "i32");
    assert_eq!(type_string(&fields[1].ty), "String");
}

#[test]
fn struct_field_with_adze_attribute() {
    let s = parse_struct(quote! {
        struct Token {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    });
    let fields = named_fields(&s);
    assert_eq!(fields.len(), 1);
    let attr_names = adze_attr_names(&fields[0].attrs);
    assert_eq!(attr_names, ["leaf"]);
}

#[test]
fn struct_field_with_multiple_attrs() {
    let s = parse_struct(quote! {
        struct Repeated {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    });
    let fields = named_fields(&s);
    let attr_names = adze_attr_names(&fields[0].attrs);
    assert_eq!(attr_names, ["repeat", "delimited"]);
}

#[test]
fn struct_field_with_skip_attribute() {
    let s = parse_struct(quote! {
        struct Annotated {
            #[adze::skip(false)]
            metadata: bool,
        }
    });
    let fields = named_fields(&s);
    assert!(is_adze_attr(&fields[0].attrs[0], "skip"));
}

#[test]
fn struct_unit_has_no_fields() {
    let s = parse_struct(quote! { struct Marker; });
    assert!(matches!(s.fields, Fields::Unit));
}

// ============================================================================
// 4. Enum Variant Parsing (5 tests)
// ============================================================================

#[test]
fn enum_variant_count() {
    let e = parse_enum(quote! {
        enum Expr {
            Number(i32),
            Add(Box<Expr>, Box<Expr>),
            Neg(Box<Expr>),
        }
    });
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn enum_variant_names() {
    let e = parse_enum(quote! {
        enum Token { Plus, Minus, Star, Slash }
    });
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, ["Plus", "Minus", "Star", "Slash"]);
}

#[test]
fn enum_variant_with_adze_attribute() {
    let e = parse_enum(quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    });
    let variant = &e.variants[0];
    assert!(is_adze_attr(&variant.attrs[0], "prec_left"));
}

#[test]
fn enum_variant_field_types() {
    let e = parse_enum(quote! {
        enum Expr {
            Binary(Box<Expr>, Op, Box<Expr>),
        }
    });
    let variant = &e.variants[0];
    match &variant.fields {
        Fields::Unnamed(f) => {
            assert_eq!(f.unnamed.len(), 3);
            assert_eq!(type_string(&f.unnamed[1].ty), "Op");
        }
        _ => panic!("expected unnamed fields"),
    }
}

#[test]
fn enum_unit_variant() {
    let e = parse_enum(quote! {
        enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    });
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unit));
}

// ============================================================================
// 5. TokenStream Construction and Comparison (8 tests)
// ============================================================================

#[test]
fn token_stream_struct_roundtrip() {
    let original: ItemStruct = parse_quote! {
        struct Foo { bar: i32 }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
}

#[test]
fn token_stream_enum_roundtrip() {
    let original: ItemEnum = parse_quote! {
        enum Color { Red, Green, Blue }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(original.variants.len(), reparsed.variants.len());
}

#[test]
fn token_stream_quote_produces_valid_struct() {
    let name = Ident::new("Dynamic", Span::call_site());
    let tokens = quote! {
        struct #name {
            value: i32,
        }
    };
    let s: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(s.ident, "Dynamic");
}

#[test]
fn token_stream_quote_produces_valid_enum() {
    let name = Ident::new("Direction", Span::call_site());
    let tokens = quote! {
        enum #name { Up, Down, Left, Right }
    };
    let e: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(e.variants.len(), 4);
}

#[test]
fn token_stream_empty_is_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn token_stream_non_empty_from_quote() {
    let ts = quote! { struct S; };
    assert!(!ts.is_empty());
}

#[test]
fn token_stream_attribute_preserved_in_roundtrip() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct Root { value: i32 }
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert!(!reparsed.attrs.is_empty());
    assert!(is_adze_attr(&reparsed.attrs[0], "language"));
}

#[test]
fn token_stream_multiple_attributes_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        struct Multi {}
    };
    let tokens = s.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    let names = adze_attr_names(&reparsed.attrs);
    assert_eq!(names, ["language", "extra"]);
}

// ============================================================================
// 6. Ident and Span Handling (5 tests)
// ============================================================================

#[test]
fn ident_from_string() {
    let id = Ident::new("my_parser", Span::call_site());
    assert_eq!(id.to_string(), "my_parser");
}

#[test]
fn ident_equality() {
    let a = Ident::new("Expr", Span::call_site());
    let b = Ident::new("Expr", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn ident_inequality() {
    let a = Ident::new("Expr", Span::call_site());
    let b = Ident::new("Statement", Span::call_site());
    assert_ne!(a, b);
}

#[test]
fn ident_to_token_stream() {
    let id = Ident::new("MyType", Span::call_site());
    let ts = id.to_token_stream();
    assert_eq!(ts.to_string(), "MyType");
}

#[test]
fn ident_extracted_from_struct() {
    let s: ItemStruct = parse_quote! { struct Parser; };
    assert_eq!(s.ident, "Parser");
}

// ============================================================================
// 7. Nested Attribute Parsing (5 tests)
// ============================================================================

#[test]
fn nested_leaf_text_attribute_on_field() {
    let s = parse_struct(quote! {
        struct Op {
            #[adze::leaf(text = "+")]
            plus: (),
        }
    });
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "leaf"));
}

#[test]
fn nested_leaf_pattern_attribute_on_field() {
    let s = parse_struct(quote! {
        struct Number {
            #[adze::leaf(pattern = r"\d+")]
            digits: String,
        }
    });
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "leaf"));
}

#[test]
fn nested_leaf_with_transform_attribute() {
    let s = parse_struct(quote! {
        struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    });
    let field = &named_fields(&s)[0];
    let attr = &field.attrs[0];
    assert!(is_adze_attr(attr, "leaf"));
    // Verify it has list meta (contains arguments)
    assert!(matches!(attr.meta, Meta::List(_)));
}

#[test]
fn nested_delimited_attribute_on_vec_field() {
    let s = parse_struct(quote! {
        struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    });
    let field = &named_fields(&s)[0];
    assert!(is_adze_attr(&field.attrs[0], "delimited"));
}

#[test]
fn nested_repeat_and_delimited_together() {
    let s = parse_struct(quote! {
        struct Args {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            params: Vec<Param>,
        }
    });
    let field = &named_fields(&s)[0];
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names, ["repeat", "delimited"]);
}

// ============================================================================
// 8. Edge Cases (8 tests)
// ============================================================================

#[test]
fn edge_empty_struct_attrs() {
    let s: ItemStruct = parse_quote! { struct Bare { x: i32 } };
    assert!(s.attrs.is_empty());
}

#[test]
fn edge_multiple_adze_attrs_on_struct() {
    let s = parse_struct(quote! {
        #[adze::language]
        #[adze::word]
        struct Root {
            value: String,
        }
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, ["language", "word"]);
}

#[test]
fn edge_mixed_adze_and_non_adze_attrs() {
    let s = parse_struct(quote! {
        #[derive(Debug)]
        #[adze::language]
        #[allow(dead_code)]
        struct Mixed { val: i32 }
    });
    let all_attrs = &s.attrs;
    assert_eq!(all_attrs.len(), 3);
    let adze_names = adze_attr_names(all_attrs);
    assert_eq!(adze_names, ["language"]);
}

#[test]
fn edge_generic_struct_single_param() {
    let s = parse_struct(quote! {
        struct Wrapper<T> { inner: T }
    });
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn edge_generic_struct_multiple_params() {
    let s = parse_struct(quote! {
        struct Pair<A, B> { first: A, second: B }
    });
    assert_eq!(s.generics.params.len(), 2);
}

#[test]
fn edge_generic_struct_with_lifetime() {
    let s = parse_struct(quote! {
        struct Borrowed<'a> { data: &'a str }
    });
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn edge_enum_with_no_variants_parses() {
    let e = parse_enum(quote! { enum Empty {} });
    assert!(e.variants.is_empty());
}

#[test]
fn edge_deeply_nested_generic_type() {
    let ty: Type = parse_quote!(Option<Vec<Box<Option<i32>>>>);
    let s = type_string(&ty);
    assert!(s.contains("Option"));
    assert!(s.contains("Vec"));
    assert!(s.contains("Box"));
}

// ============================================================================
// Additional coverage: expression parsing in attributes (4 tests)
// ============================================================================

#[test]
fn expr_string_literal_parses() {
    let expr: Expr = parse_quote!("hello");
    assert!(matches!(expr, Expr::Lit(_)));
}

#[test]
fn expr_integer_literal_parses() {
    let expr: Expr = parse_quote!(42);
    assert!(matches!(expr, Expr::Lit(_)));
}

#[test]
fn expr_bool_literal_parses() {
    let expr: Expr = parse_quote!(true);
    let ts = expr.to_token_stream().to_string();
    assert_eq!(ts, "true");
}

#[test]
fn expr_closure_parses() {
    let expr: Expr = parse_quote!(|v| v.parse().unwrap());
    assert!(matches!(expr, Expr::Closure(_)));
}

// ============================================================================
// Additional coverage: visibility parsing (3 tests)
// ============================================================================

#[test]
fn visibility_pub_struct() {
    let s: ItemStruct = parse_quote! { pub struct Public; };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

#[test]
fn visibility_private_struct() {
    let s: ItemStruct = parse_quote! { struct Private; };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

#[test]
fn visibility_pub_crate_struct() {
    let s: ItemStruct = parse_quote! { pub(crate) struct Internal; };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

// ============================================================================
// Additional coverage: where clause and bounds (3 tests)
// ============================================================================

#[test]
fn generics_with_trait_bound() {
    let s = parse_struct(quote! {
        struct Container<T: Clone> { item: T }
    });
    assert_eq!(s.generics.params.len(), 1);
}

#[test]
fn generics_with_where_clause() {
    let s = parse_struct(quote! {
        struct Filtered<T> where T: Send + Sync { data: T }
    });
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn generics_with_default_type() {
    let s = parse_struct(quote! {
        struct WithDefault<T = String> { value: T }
    });
    assert_eq!(s.generics.params.len(), 1);
}
