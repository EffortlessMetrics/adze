#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for Option<T> field handling in the adze macro crate.
//!
//! Optional fields are represented as Option<T> and generate CHOICE(T, BLANK) rules.
//! Tests cover Option<T> detection on struct fields, Option<String> leaf fields,
//! Option with custom types, nested Option<Option<T>>, multiple optional fields,
//! mix of optional and required fields, Option in enum variants, and Option with
//! Box<T> inside.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn field_type_str(s: &ItemStruct, field_name: &str) -> String {
    s.fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == field_name))
        .unwrap()
        .ty
        .to_token_stream()
        .to_string()
}

fn is_option_type(ty: &Type) -> bool {
    let skip: HashSet<&str> = HashSet::new();
    let (_, extracted) = try_extract_inner_type(ty, "Option", &skip);
    extracted
}

fn extract_option_inner(ty: &Type) -> Option<Type> {
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(ty, "Option", &skip);
    if extracted { Some(inner) } else { None }
}

// ── 1. Option<T> detected on struct field ───────────────────────────────────

#[test]
fn option_detected_on_struct_field() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_option_type(&ty));
}

// ── 2. Non-option type not detected ─────────────────────────────────────────

#[test]
fn non_option_type_not_detected() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_option_type(&ty));
}

// ── 3. Option<String> inner type extraction ─────────────────────────────────

#[test]
fn option_string_inner_extraction() {
    let ty: Type = parse_quote!(Option<String>);
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ── 4. Option<i32> inner type extraction ────────────────────────────────────

#[test]
fn option_i32_inner_extraction() {
    let ty: Type = parse_quote!(Option<i32>);
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 5. Option with custom type extraction ───────────────────────────────────

#[test]
fn option_custom_type_inner_extraction() {
    let ty: Type = parse_quote!(Option<MyCustomNode>);
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "MyCustomNode");
}

// ── 6. Option<String> leaf field in struct ──────────────────────────────────

#[test]
fn option_string_leaf_field_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeIdent {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: Option<String>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(is_option_type(&field.ty));
    let inner = extract_option_inner(&field.ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ── 7. Option<i32> leaf field with transform ────────────────────────────────

#[test]
fn option_leaf_with_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeNumber {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: Option<i32>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(is_option_type(&field.ty));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 8. Option with custom struct type ───────────────────────────────────────

#[test]
fn option_custom_struct_type_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Option<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    let field = root.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Option < Child >");
    assert!(is_option_type(&field.ty));
}

// ── 9. Nested Option<Option<T>> extraction ──────────────────────────────────

#[test]
fn nested_option_option_detected() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    assert!(is_option_type(&ty));
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "Option < i32 >");
    // Inner is itself an Option
    assert!(is_option_type(&inner));
}

// ── 10. Nested Option inner of inner ────────────────────────────────────────

#[test]
fn nested_option_inner_of_inner() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let outer_inner = extract_option_inner(&ty).unwrap();
    let inner_inner = extract_option_inner(&outer_inner).unwrap();
    assert_eq!(inner_inner.to_token_stream().to_string(), "String");
}

// ── 11. Multiple optional fields in struct ──────────────────────────────────

#[test]
fn multiple_optional_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct MultiOpt {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            a: Option<i32>,
            #[adze::leaf(pattern = r"[a-z]+")]
            b: Option<String>,
            c: Option<Child>,
        }
    };
    let option_count = s.fields.iter().filter(|f| is_option_type(&f.ty)).count();
    assert_eq!(option_count, 3);
}

// ── 12. Mix of optional and required fields ─────────────────────────────────

#[test]
fn mix_optional_and_required_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedFields {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            required_num: i32,
            #[adze::leaf(pattern = r"[a-z]+")]
            optional_name: Option<String>,
            child: Child,
            maybe_child: Option<Child>,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert!(!is_option_type(&fields[0].ty));
    assert!(is_option_type(&fields[1].ty));
    assert!(!is_option_type(&fields[2].ty));
    assert!(is_option_type(&fields[3].ty));
}

// ── 13. Required field not confused with option ─────────────────────────────

#[test]
fn required_field_not_option() {
    let ty: Type = parse_quote!(String);
    assert!(!is_option_type(&ty));
    assert!(extract_option_inner(&ty).is_none());
}

// ── 14. Vec field not confused with option ──────────────────────────────────

#[test]
fn vec_field_not_detected_as_option() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert!(!is_option_type(&ty));
}

// ── 15. Option in enum tuple variant ────────────────────────────────────────

#[test]
fn option_in_enum_tuple_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            MaybeNum(Option<i32>),
        }
    };
    let variant = &e.variants[0];
    if let Fields::Unnamed(u) = &variant.fields {
        assert!(is_option_type(&u.unnamed[0].ty));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 16. Option in enum named variant ────────────────────────────────────────

#[test]
fn option_in_enum_named_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            WithOptional {
                #[adze::leaf(pattern = r"\d+")]
                required: String,
                optional_child: Option<Child>,
            },
        }
    };
    let variant = &e.variants[0];
    if let Fields::Named(n) = &variant.fields {
        assert!(!is_option_type(&n.named[0].ty));
        assert!(is_option_type(&n.named[1].ty));
    } else {
        panic!("Expected named fields");
    }
}

// ── 17. Option in multiple enum variants ────────────────────────────────────

#[test]
fn option_in_multiple_enum_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            A(Option<i32>),
            B {
                x: Option<String>,
            },
            C(String),
        }
    };
    // Variant A has optional tuple field
    if let Fields::Unnamed(u) = &e.variants[0].fields {
        assert!(is_option_type(&u.unnamed[0].ty));
    }
    // Variant B has optional named field
    if let Fields::Named(n) = &e.variants[1].fields {
        assert!(is_option_type(&n.named[0].ty));
    }
    // Variant C has required field
    if let Fields::Unnamed(u) = &e.variants[2].fields {
        assert!(!is_option_type(&u.unnamed[0].ty));
    }
}

// ── 18. Option<Box<T>> with skip_over ───────────────────────────────────────

#[test]
fn option_box_inner_type_with_skip() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    // Without Box in skip_over, inner is Box<Expr>
    let skip_empty: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_empty);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Box < Expr >");
}

// ── 19. Box<Option<T>> with Box in skip_over ────────────────────────────────

#[test]
fn box_option_with_skip_over() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 20. Option<Box<T>> field on struct ──────────────────────────────────────

#[test]
fn option_box_field_on_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapper {
            child: Option<Box<Inner>>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(is_option_type(&field.ty));
    let inner = extract_option_inner(&field.ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "Box < Inner >");
}

// ── 21. filter_inner_type does not strip Option ─────────────────────────────

#[test]
fn filter_inner_type_preserves_option() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Option < String >");
}

// ── 22. filter_inner_type strips Box inside Option ──────────────────────────

#[test]
fn filter_inner_type_does_not_recurse_into_option() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Option<Box<String>>);
    // filter_inner_type only strips top-level skip types
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        "Option < Box < String > >"
    );
}

// ── 23. wrap_leaf_type with Option in skip set ──────────────────────────────

#[test]
fn wrap_leaf_type_skips_option_wraps_inner() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < String > >"
    );
}

// ── 24. wrap_leaf_type without Option in skip set ───────────────────────────

#[test]
fn wrap_leaf_type_wraps_entire_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Option < String > >"
    );
}

// ── 25. Option field type preserved in grammar module ───────────────────────

#[test]
fn option_field_type_preserved_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: Option<i32>,
                t: Option<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert_eq!(field_type_str(root, "v"), "Option < i32 >");
    assert_eq!(field_type_str(root, "t"), "Option < Number >");
}

// ── 26. Option<()> unit type field ──────────────────────────────────────────

#[test]
fn option_unit_type_field() {
    let ty: Type = parse_quote!(Option<()>);
    assert!(is_option_type(&ty));
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "()");
}

// ── 27. Option field coexists with Vec field ────────────────────────────────

#[test]
fn option_field_coexists_with_vec_field() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedContainers {
            items: Vec<Item>,
            maybe_label: Option<String>,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    // Vec field is not Option
    assert!(!is_option_type(&fields[0].ty));
    // Option field is detected
    assert!(is_option_type(&fields[1].ty));
    // Vec extraction works on first field
    let skip: HashSet<&str> = HashSet::new();
    let (_, is_vec) = try_extract_inner_type(&fields[0].ty, "Vec", &skip);
    assert!(is_vec);
}

// ── 28. Option field with skip annotation ───────────────────────────────────

#[test]
fn option_field_with_skip_annotation() {
    let s: ItemStruct = parse_quote! {
        pub struct WithSkip {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(None)]
            cached: Option<i32>,
        }
    };
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "cached"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    assert!(is_option_type(&skip_field.ty));
}

// ── 29. Option enum variant in grammar module ───────────────────────────────

#[test]
fn option_in_enum_variant_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                WithOpt {
                    base: Base,
                    suffix: Option<Suffix>,
                },
            }

            pub struct Base {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }

            pub struct Suffix {
                #[adze::leaf(text = "+")]
                _plus: (),
            }
        }
    });
    let expr = find_enum_in_mod(&m, "Expr").unwrap();
    let variant = &expr.variants[0];
    if let Fields::Named(n) = &variant.fields {
        assert!(!is_option_type(&n.named[0].ty));
        assert!(is_option_type(&n.named[1].ty));
    } else {
        panic!("Expected named fields");
    }
}

// ── 30. Option<Box<T>> in enum variant ──────────────────────────────────────

#[test]
fn option_box_in_enum_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Recursive(Option<Box<Expr>>),
        }
    };
    let variant = &e.variants[0];
    if let Fields::Unnamed(u) = &variant.fields {
        let ty = &u.unnamed[0].ty;
        assert!(is_option_type(ty));
        let inner = extract_option_inner(ty).unwrap();
        assert_eq!(inner.to_token_stream().to_string(), "Box < Expr >");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 31. try_extract_inner_type with Spanned<Option<T>> ──────────────────────

#[test]
fn spanned_option_extraction_with_skip() {
    let ty: Type = parse_quote!(Spanned<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Spanned"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 32. Option extraction fails for non-Option generic ──────────────────────

#[test]
fn option_extraction_fails_for_result_type() {
    let ty: Type = parse_quote!(Result<String, Error>);
    assert!(!is_option_type(&ty));
    assert!(extract_option_inner(&ty).is_none());
}

// ── 33. Option<bool> extraction ─────────────────────────────────────────────

#[test]
fn option_bool_extraction() {
    let ty: Type = parse_quote!(Option<bool>);
    let inner = extract_option_inner(&ty).unwrap();
    assert_eq!(inner.to_token_stream().to_string(), "bool");
}

// ── 34. All fields optional in struct ───────────────────────────────────────

#[test]
fn all_fields_optional_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct AllOptional {
            #[adze::leaf(pattern = r"\d+")]
            a: Option<String>,
            b: Option<Child>,
            #[adze::leaf(text = ";")]
            c: Option<()>,
        }
    };
    for field in &s.fields {
        assert!(
            is_option_type(&field.ty),
            "Field {:?} should be optional",
            field.ident
        );
    }
}

// ── 35. wrap_leaf_type with Option and Vec in skip set ──────────────────────

#[test]
fn wrap_leaf_type_option_and_vec_in_skip_set() {
    let skip: HashSet<&str> = HashSet::from(["Option", "Vec"]);
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}
