//! Attribute parsing v3 tests for the adze-macro crate.
//!
//! Covers:
//! 1. DeriveInput parsing — struct, enum, unit struct
//! 2. Attribute extraction — #[language], #[leaf], #[skip]
//! 3. Type analysis — parameterized vs simple types
//! 4. Token stream roundtrip — parse → quote → parse
//! 5. Field extraction — named fields, unnamed fields
//! 6. Generics handling — lifetime, type parameters
//! 7. Visibility — pub, pub(crate), private
//! 8. Edge cases — empty struct, many fields, nested attributes

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Fields, GenericParam, ItemEnum, ItemMod, ItemStruct, Type, Visibility,
    parse_quote, parse2,
};

// ── Helper Functions ─────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn parse_derive(code: &str) -> DeriveInput {
    syn::parse_str::<DeriveInput>(code).expect("failed to parse DeriveInput")
}

fn parse_type(code: &str) -> Type {
    syn::parse_str::<Type>(code).expect("failed to parse Type")
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

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn module_items(m: &ItemMod) -> &[syn::Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn empty_skip_set() -> HashSet<&'static str> {
    HashSet::new()
}

fn box_skip_set() -> HashSet<&'static str> {
    ["Box"].into_iter().collect()
}

// ============================================================================
// 1. DERIVE INPUT PARSING — struct, enum, unit struct
// ============================================================================

#[test]
fn derive_input_named_struct() {
    let di = parse_derive("struct Foo { x: i32, y: String }");
    assert_eq!(di.ident, "Foo");
    assert!(di.generics.params.is_empty());
    if let syn::Data::Struct(s) = &di.data {
        assert!(matches!(s.fields, Fields::Named(_)));
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_unit_struct() {
    let di = parse_derive("struct Empty;");
    assert_eq!(di.ident, "Empty");
    if let syn::Data::Struct(s) = &di.data {
        assert!(matches!(s.fields, Fields::Unit));
        assert_eq!(s.fields.len(), 0);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_tuple_struct() {
    let di = parse_derive("struct Pair(i32, String);");
    assert_eq!(di.ident, "Pair");
    if let syn::Data::Struct(s) = &di.data {
        assert!(matches!(s.fields, Fields::Unnamed(_)));
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_enum_variants() {
    let di = parse_derive("enum Color { Red, Green, Blue }");
    assert_eq!(di.ident, "Color");
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 3);
        let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        assert_eq!(names, vec!["Red", "Green", "Blue"]);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn derive_input_enum_with_data() {
    let di = parse_derive("enum Expr { Num(i32), Add(Box<Expr>, Box<Expr>) }");
    assert_eq!(di.ident, "Expr");
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].fields.len(), 1);
        assert_eq!(e.variants[1].fields.len(), 2);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn derive_input_enum_mixed_variants() {
    let di = parse_derive("enum Mixed { Unit, Tuple(i32), Named { x: f64 } }");
    if let syn::Data::Enum(e) = &di.data {
        assert!(matches!(e.variants[0].fields, Fields::Unit));
        assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
        assert!(matches!(e.variants[2].fields, Fields::Named(_)));
    } else {
        panic!("expected enum");
    }
}

#[test]
fn derive_input_empty_enum() {
    let di = parse_derive("enum Nothing {}");
    if let syn::Data::Enum(e) = &di.data {
        assert!(e.variants.is_empty());
    } else {
        panic!("expected enum");
    }
}

// ============================================================================
// 2. ATTRIBUTE EXTRACTION — #[language], #[leaf], #[skip]
// ============================================================================

#[test]
fn attr_language_on_struct() {
    let s = parse_struct(quote! {
        #[adze::language]
        struct Root { items: Vec<Item> }
    });
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn attr_language_on_enum() {
    let e = parse_enum(quote! {
        #[adze::language]
        enum Expr { Num(i32) }
    });
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn attr_leaf_on_struct() {
    let s = parse_struct(quote! {
        #[adze::leaf(text = "hello")]
        struct Keyword;
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["leaf"]);
}

#[test]
fn attr_skip_on_field() {
    let s = parse_struct(quote! {
        struct Node {
            #[adze::skip(false)]
            visited: bool,
        }
    });
    if let Fields::Named(ref fields) = s.fields {
        let field = &fields.named[0];
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn attr_multiple_on_same_item() {
    let s = parse_struct(quote! {
        #[adze::language]
        #[adze::extra]
        struct Root;
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["language", "extra"]);
}

#[test]
fn attr_leaf_with_pattern() {
    let s = parse_struct(quote! {
        #[adze::leaf(pattern = r"\d+")]
        struct Number;
    });
    assert!(is_adze_attr(&s.attrs[0], "leaf"));
}

#[test]
fn attr_prec_left_on_variant() {
    let e = parse_enum(quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    });
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

#[test]
fn attr_prec_right_on_variant() {
    let e = parse_enum(quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, Box<Expr>),
        }
    });
    let variant = &e.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
}

#[test]
fn attr_extra_on_struct() {
    let s = parse_struct(quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["extra"]);
}

#[test]
fn attr_word_on_struct() {
    let s = parse_struct(quote! {
        #[adze::word]
        #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
        struct Identifier(String);
    });
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["word", "leaf"]);
}

// ============================================================================
// 3. TYPE ANALYSIS — parameterized vs simple types
// ============================================================================

#[test]
fn type_simple_not_parameterized() {
    let ty = parse_type("i32");
    assert!(!is_parameterized(&ty));
}

#[test]
fn type_string_not_parameterized() {
    let ty = parse_type("String");
    assert!(!is_parameterized(&ty));
}

#[test]
fn type_vec_is_parameterized() {
    let ty = parse_type("Vec<i32>");
    assert!(is_parameterized(&ty));
}

#[test]
fn type_option_is_parameterized() {
    let ty = parse_type("Option<String>");
    assert!(is_parameterized(&ty));
}

#[test]
fn type_box_is_parameterized() {
    let ty = parse_type("Box<Expr>");
    assert!(is_parameterized(&ty));
}

#[test]
fn type_nested_generic_is_parameterized() {
    let ty = parse_type("Vec<Option<i32>>");
    assert!(is_parameterized(&ty));
}

#[test]
fn type_reference_not_parameterized() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn type_hashmap_is_parameterized() {
    let ty = parse_type("HashMap<String, i32>");
    assert!(is_parameterized(&ty));
}

#[test]
fn try_extract_vec_inner() {
    let ty: Type = parse_quote!(Vec<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip_set());
    assert!(ok);
    assert_eq!(ts(&inner), "u64");
}

#[test]
fn try_extract_option_inner() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip_set());
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn try_extract_mismatch() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip_set());
    assert!(!ok);
}

#[test]
fn try_extract_skip_through_box() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip_set());
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn filter_inner_unwraps_box() {
    let ty: Type = parse_quote!(Box<i32>);
    let result = filter_inner_type(&ty, &box_skip_set());
    assert_eq!(ts(&result), "i32");
}

#[test]
fn filter_inner_nested_boxes() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<String>>);
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ts(&result), "String");
}

#[test]
fn filter_inner_non_skip_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &box_skip_set());
    assert_eq!(ts(&result), "Vec < i32 >");
}

#[test]
fn wrap_leaf_simple_type() {
    let ty: Type = parse_quote!(i32);
    let result = wrap_leaf_type(&ty, &empty_skip_set());
    assert_eq!(ts(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_leaf_skips_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ts(&result), "Vec < adze :: WithLeaf < i32 > >");
}

// ============================================================================
// 4. TOKEN STREAM ROUNDTRIP — parse → quote → parse
// ============================================================================

#[test]
fn roundtrip_struct() {
    let original = quote! { struct Foo { x: i32 } };
    let parsed: ItemStruct = parse2(original).unwrap();
    let tokens = parsed.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(parsed.ident, reparsed.ident);
    assert_eq!(parsed.fields.len(), reparsed.fields.len());
}

#[test]
fn roundtrip_enum() {
    let original = quote! { enum Dir { Up, Down, Left, Right } };
    let parsed: ItemEnum = parse2(original).unwrap();
    let tokens = parsed.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(parsed.ident, reparsed.ident);
    assert_eq!(parsed.variants.len(), reparsed.variants.len());
}

#[test]
fn roundtrip_attributed_struct() {
    let original = quote! {
        #[adze::language]
        struct Root { items: Vec<Item> }
    };
    let parsed: ItemStruct = parse2(original).unwrap();
    let tokens = parsed.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(reparsed.attrs.len(), 1);
    assert!(is_adze_attr(&reparsed.attrs[0], "language"));
}

#[test]
fn roundtrip_type_expression() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let tokens = ty.to_token_stream();
    let reparsed: Type = parse2(tokens).unwrap();
    assert_eq!(ts(&ty), ts(&reparsed));
}

#[test]
fn roundtrip_name_value_expr() {
    let nv: NameValueExpr = parse_quote!(pattern = "hello");
    assert_eq!(nv.path, "pattern");
    let path = &nv.path;
    let expr = &nv.expr;
    let tokens = quote! { #path = #expr };
    assert!(!tokens.is_empty());
}

#[test]
fn roundtrip_unit_struct() {
    let original = quote! { struct Marker; };
    let parsed: ItemStruct = parse2(original).unwrap();
    let tokens = parsed.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(parsed.ident, reparsed.ident);
    assert!(matches!(reparsed.fields, Fields::Unit));
}

#[test]
fn roundtrip_tuple_struct() {
    let original = quote! { struct Wrapper(String); };
    let parsed: ItemStruct = parse2(original).unwrap();
    let tokens = parsed.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(reparsed.fields.len(), 1);
}

// ============================================================================
// 5. FIELD EXTRACTION — named fields, unnamed fields
// ============================================================================

#[test]
fn named_field_idents() {
    let s = parse_struct(quote! {
        struct Point { x: f64, y: f64 }
    });
    if let Fields::Named(ref fields) = s.fields {
        let names: Vec<_> = fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["x", "y"]);
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn unnamed_field_types() {
    let s = parse_struct(quote! {
        struct Pair(i32, String);
    });
    if let Fields::Unnamed(ref fields) = s.fields {
        let types: Vec<_> = fields.unnamed.iter().map(|f| ts(&f.ty)).collect();
        assert_eq!(types, vec!["i32", "String"]);
    } else {
        panic!("expected unnamed fields");
    }
}

#[test]
fn enum_variant_named_fields() {
    let e = parse_enum(quote! {
        enum Shape {
            Circle { radius: f64 },
            Rect { w: f64, h: f64 },
        }
    });
    assert_eq!(e.variants[0].fields.len(), 1);
    assert_eq!(e.variants[1].fields.len(), 2);
}

#[test]
fn field_with_attribute() {
    let s = parse_struct(quote! {
        struct Node {
            #[adze::leaf(pattern = r"\d+")]
            value: i32,
            name: String,
        }
    });
    if let Fields::Named(ref fields) = s.fields {
        assert!(!fields.named[0].attrs.is_empty());
        assert!(fields.named[1].attrs.is_empty());
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn enum_variant_unnamed_field_count() {
    let e = parse_enum(quote! {
        enum Op {
            Unary(Box<Expr>),
            Binary(Box<Expr>, Box<Expr>),
            Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
        }
    });
    assert_eq!(e.variants[0].fields.len(), 1);
    assert_eq!(e.variants[1].fields.len(), 2);
    assert_eq!(e.variants[2].fields.len(), 3);
}

#[test]
fn field_then_params_bare() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "String");
}

#[test]
fn field_then_params_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(i32, transform = |v| v.parse().unwrap());
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "transform");
}

#[test]
fn field_then_params_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(u64, name = "count", priority = 5);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path, "name");
    assert_eq!(ftp.params[1].path, "priority");
}

// ============================================================================
// 6. GENERICS HANDLING — lifetime, type parameters
// ============================================================================

#[test]
fn generic_type_param() {
    let di = parse_derive("struct Wrapper<T> { inner: T }");
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(di.generics.params[0], GenericParam::Type(_)));
}

#[test]
fn generic_lifetime_param() {
    let di = parse_derive("struct Ref<'a> { data: &'a str }");
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(di.generics.params[0], GenericParam::Lifetime(_)));
}

#[test]
fn generic_mixed_params() {
    let di = parse_derive("struct Mixed<'a, T, U> { x: &'a T, y: U }");
    assert_eq!(di.generics.params.len(), 3);
    assert!(matches!(di.generics.params[0], GenericParam::Lifetime(_)));
    assert!(matches!(di.generics.params[1], GenericParam::Type(_)));
    assert!(matches!(di.generics.params[2], GenericParam::Type(_)));
}

#[test]
fn generic_with_bounds() {
    let di = parse_derive("struct Bounded<T: Clone + Send> { val: T }");
    assert_eq!(di.generics.params.len(), 1);
    if let GenericParam::Type(tp) = &di.generics.params[0] {
        assert!(!tp.bounds.is_empty());
    } else {
        panic!("expected type param");
    }
}

#[test]
fn generic_where_clause() {
    let di = parse_derive("struct Constrained<T> where T: std::fmt::Debug { val: T }");
    assert!(di.generics.where_clause.is_some());
}

#[test]
fn enum_with_generics() {
    let di = parse_derive("enum Result2<T, E> { Ok(T), Err(E) }");
    assert_eq!(di.generics.params.len(), 2);
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 2);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn generic_const_param() {
    let di = parse_derive("struct Array<const N: usize> { data: [u8; N] }");
    assert_eq!(di.generics.params.len(), 1);
    assert!(matches!(di.generics.params[0], GenericParam::Const(_)));
}

// ============================================================================
// 7. VISIBILITY — pub, pub(crate), private
// ============================================================================

#[test]
fn vis_pub_struct() {
    let s = parse_struct(quote! { pub struct Public { x: i32 } });
    assert!(matches!(s.vis, Visibility::Public(_)));
}

#[test]
fn vis_private_struct() {
    let s = parse_struct(quote! { struct Private { x: i32 } });
    assert!(matches!(s.vis, Visibility::Inherited));
}

#[test]
fn vis_pub_crate_struct() {
    let s = parse_struct(quote! { pub(crate) struct Internal { x: i32 } });
    assert!(matches!(s.vis, Visibility::Restricted(_)));
}

#[test]
fn vis_pub_enum() {
    let e = parse_enum(quote! { pub enum Dir { Up, Down } });
    assert!(matches!(e.vis, Visibility::Public(_)));
}

#[test]
fn vis_field_level() {
    let s = parse_struct(quote! {
        struct Mixed {
            pub public_field: i32,
            private_field: String,
            pub(crate) internal_field: bool,
        }
    });
    if let Fields::Named(ref fields) = s.fields {
        assert!(matches!(fields.named[0].vis, Visibility::Public(_)));
        assert!(matches!(fields.named[1].vis, Visibility::Inherited));
        assert!(matches!(fields.named[2].vis, Visibility::Restricted(_)));
    } else {
        panic!("expected named fields");
    }
}

// ============================================================================
// 8. EDGE CASES — empty struct, many fields, nested attributes
// ============================================================================

#[test]
fn edge_empty_named_struct() {
    let s = parse_struct(quote! { struct Empty {} });
    assert_eq!(s.fields.len(), 0);
    assert!(matches!(s.fields, Fields::Named(_)));
}

#[test]
fn edge_single_field_struct() {
    let s = parse_struct(quote! { struct Single { only: u8 } });
    assert_eq!(s.fields.len(), 1);
}

#[test]
fn edge_many_fields() {
    let s = parse_struct(quote! {
        struct Wide {
            a: i32, b: i32, c: i32, d: i32, e: i32,
            f: i32, g: i32, h: i32, i: i32, j: i32,
        }
    });
    assert_eq!(s.fields.len(), 10);
}

#[test]
fn edge_nested_generic_types() {
    let ty = parse_type("Vec<Option<Box<HashMap<String, Vec<i32>>>>>");
    assert!(is_parameterized(&ty));
}

#[test]
fn edge_field_then_params_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, name = "items");
    assert_eq!(ts(&ftp.field.ty), "Vec < String >");
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn edge_name_value_expr_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"[a-z]+");
    assert_eq!(nv.path, "pattern");
}

#[test]
fn edge_name_value_expr_closure() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nv.path, "transform");
    let expr_str = nv.expr.to_token_stream().to_string();
    assert!(expr_str.contains("parse"));
}

#[test]
fn edge_non_path_type_filter() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &box_skip_set());
    assert_eq!(ts(&result), "& str");
}

#[test]
fn edge_wrap_leaf_reference_type() {
    let ty: Type = parse_quote!(&str);
    let result = wrap_leaf_type(&ty, &empty_skip_set());
    assert_eq!(ts(&result), "adze :: WithLeaf < & str >");
}

#[test]
fn edge_extract_from_simple_type() {
    let ty: Type = parse_quote!(i32);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip_set());
    assert!(!ok);
}

#[test]
fn edge_grammar_module_parsing() {
    let m: ItemMod = parse2(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            struct Root;

            #[adze::extra]
            struct Ws;
        }
    })
    .unwrap();
    assert_eq!(module_items(&m).len(), 2);
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
}

#[test]
fn edge_enum_single_variant() {
    let e = parse_enum(quote! { enum Singleton { Only(i32) } });
    assert_eq!(e.variants.len(), 1);
}

#[test]
fn edge_multiple_attrs_on_field() {
    let s = parse_struct(quote! {
        struct Node {
            #[adze::leaf(text = "+")]
            #[adze::skip(())]
            op: (),
        }
    });
    if let Fields::Named(ref fields) = s.fields {
        let names = adze_attr_names(&fields.named[0].attrs);
        assert_eq!(names, vec!["leaf", "skip"]);
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn edge_derive_input_preserves_attrs() {
    let di = parse_derive(r#"#[derive(Debug, Clone)] struct Foo { x: i32 }"#);
    assert!(!di.attrs.is_empty());
    assert_eq!(di.ident, "Foo");
}

#[test]
fn edge_extract_skip_multiple_layers() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<u16>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u16");
}

#[test]
fn edge_filter_multi_wrapper() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ts(&result), "String");
}

#[test]
fn edge_wrap_leaf_nested_skip() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ts(&result), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

#[test]
fn edge_name_value_integer() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path, "precedence");
}

#[test]
fn edge_name_value_bool() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path, "non_empty");
}
