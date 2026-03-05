//! Property-based tests (v4) for expansion infrastructure in `adze-macro`.
//!
//! Covers: `DeriveInput` roundtrips, field extraction properties, type analysis
//! consistency, attribute presence detection, and token stream properties.
//! API under test: `adze_common::{try_extract_inner_type, filter_inner_type,
//! wrap_leaf_type}`, `syn::parse_str`, `quote::quote!`, `proc_macro2::TokenStream`.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, DeriveInput, Fields, Type, parse_quote, parse_str};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ty(s: &str) -> Type {
    parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

// ── Strategies ──────────────────────────────────────────────────────────────

/// Valid Rust identifiers excluding keywords (including Rust 2024 reserved).
fn ident_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,12}".prop_filter("no keywords", |s| {
        !matches!(
            s.as_str(),
            "type"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
                | "self"
                | "super"
                | "crate"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "where"
                | "for"
                | "loop"
                | "while"
                | "if"
                | "else"
                | "match"
                | "return"
                | "break"
                | "continue"
                | "as"
                | "in"
                | "move"
                | "box"
                | "dyn"
                | "async"
                | "await"
                | "try"
                | "yield"
                | "macro"
                | "const"
                | "static"
                | "unsafe"
                | "extern"
                | "do"
                | "gen"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "true"
                | "false"
        )
    })
}

fn type_name_strategy() -> impl Strategy<Value = String> {
    ident_strategy().prop_map(|s| {
        let mut chars = s.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::from("T"),
        }
    })
}

fn simple_type_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
        "String", "usize", "isize",
    ])
    .prop_map(|s| s.to_string())
}

fn wrapper_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["Vec", "Option", "Box", "Arc", "Rc"]).prop_map(|s| s.to_string())
}

fn attr_name_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("language"),
        Just("extra"),
        Just("leaf"),
        Just("skip"),
        Just("prec"),
        Just("word"),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: DeriveInput roundtrip properties (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// A DeriveInput for a unit struct survives a token → parse roundtrip.
    #[test]
    fn derive_input_unit_struct_roundtrip(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! { struct #ident; };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        let rt_tokens = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt_tokens).unwrap();
        prop_assert_eq!(di2.ident.to_string(), name);
    }

    /// A DeriveInput for a named-field struct preserves the struct name.
    #[test]
    fn derive_input_named_fields_roundtrip(
        name in type_name_strategy(),
        field in ident_strategy(),
    ) {
        let sname = format_ident!("{}", name);
        let fname = format_ident!("{}", field);
        let tokens = quote! { struct #sname { #fname: u32 } };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert_eq!(di2.ident.to_string(), name);
    }

    /// A DeriveInput for a tuple struct preserves the struct name.
    #[test]
    fn derive_input_tuple_struct_roundtrip(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! { struct #ident(u32); };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert_eq!(di2.ident.to_string(), name);
    }

    /// A DeriveInput enum roundtrips through token streams.
    #[test]
    fn derive_input_enum_roundtrip(
        name in type_name_strategy(),
        variant in type_name_strategy(),
    ) {
        prop_assume!(name != variant);
        let ename = format_ident!("{}", name);
        let vname = format_ident!("{}", variant);
        let tokens = quote! { enum #ename { #vname } };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert_eq!(di2.ident.to_string(), name);
    }

    /// DeriveInput preserves visibility after roundtrip.
    #[test]
    fn derive_input_preserves_visibility(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! { pub struct #ident; };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        let rt = di.to_token_stream();
        let s = rt.to_string();
        prop_assert!(s.contains("pub"));
    }

    /// DeriveInput preserves derive attributes after roundtrip.
    #[test]
    fn derive_input_preserves_derive_attrs(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! {
            #[derive(Debug, Clone)]
            struct #ident;
        };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        prop_assert!(!di.attrs.is_empty());
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert!(!di2.attrs.is_empty());
    }

    /// A struct with generics roundtrips and keeps the generic param.
    #[test]
    fn derive_input_generic_struct_roundtrip(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! { struct #ident<T> { value: T } };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        prop_assert!(!di.generics.params.is_empty());
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert!(!di2.generics.params.is_empty());
    }

    /// An enum with multiple variants preserves variant count.
    #[test]
    fn derive_input_enum_variant_count(
        name in type_name_strategy(),
        v1 in type_name_strategy(),
        v2 in type_name_strategy(),
    ) {
        prop_assume!(v1 != v2);
        prop_assume!(name != v1);
        prop_assume!(name != v2);
        let ename = format_ident!("{}", name);
        let vn1 = format_ident!("{}", v1);
        let vn2 = format_ident!("{}", v2);
        let tokens = quote! { enum #ename { #vn1, #vn2 } };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            prop_assert_eq!(data.variants.len(), 2);
        } else {
            prop_assert!(false, "expected enum");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Field extraction properties (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Named fields from a struct have the correct count.
    #[test]
    fn named_fields_count(
        f1 in ident_strategy(),
        f2 in ident_strategy(),
    ) {
        prop_assume!(f1 != f2);
        let i1 = format_ident!("{}", f1);
        let i2 = format_ident!("{}", f2);
        let di: DeriveInput = parse_quote! {
            struct S { #i1: u32, #i2: String }
        };
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Named(ref nf) = ds.fields {
                    prop_assert_eq!(nf.named.len(), 2);
                } else {
                    prop_assert!(false, "expected named fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }

    /// Tuple struct fields have the correct count.
    #[test]
    fn tuple_fields_count(count in 1usize..=4) {
        let field_types: Vec<Type> = (0..count).map(|_| parse_quote!(u32)).collect();
        let tokens = quote! { struct S(#(#field_types),*); };
        let di: DeriveInput = syn::parse2(tokens).unwrap();
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Unnamed(ref uf) = ds.fields {
                    prop_assert_eq!(uf.unnamed.len(), count);
                } else {
                    prop_assert!(false, "expected unnamed fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }

    /// Unit struct has no fields.
    #[test]
    fn unit_struct_has_no_fields(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let di: DeriveInput = parse_quote! { struct #ident; };
        match &di.data {
            syn::Data::Struct(ds) => {
                prop_assert!(matches!(&ds.fields, Fields::Unit));
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }

    /// Field names are preserved through extraction.
    #[test]
    fn field_name_preserved(field_name in ident_strategy()) {
        let fname = format_ident!("{}", field_name);
        let di: DeriveInput = parse_quote! {
            struct S { #fname: i32 }
        };
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Named(ref nf) = ds.fields {
                    let name = nf.named.first().unwrap().ident.as_ref().unwrap().to_string();
                    prop_assert_eq!(name, field_name);
                } else {
                    prop_assert!(false, "expected named fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }

    /// Field type is preserved through extraction.
    #[test]
    fn field_type_preserved(type_name in simple_type_name()) {
        let ty_tok: Type = parse_str(&type_name).unwrap();
        let di: DeriveInput = parse_quote! {
            struct S { value: #ty_tok }
        };
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Named(ref nf) = ds.fields {
                    let field_ty = &nf.named.first().unwrap().ty;
                    let s = field_ty.to_token_stream().to_string();
                    prop_assert!(s.contains(&type_name));
                } else {
                    prop_assert!(false, "expected named fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }

    /// Enum variant fields are accessible.
    #[test]
    fn enum_variant_field_accessible(
        variant in type_name_strategy(),
        field in ident_strategy(),
    ) {
        let vn = format_ident!("{}", variant);
        let fn_ = format_ident!("{}", field);
        let di: DeriveInput = parse_quote! {
            enum E { #vn { #fn_: u32 } }
        };
        match &di.data {
            syn::Data::Enum(data) => {
                let v = data.variants.first().unwrap();
                if let Fields::Named(ref nf) = v.fields {
                    prop_assert_eq!(nf.named.len(), 1);
                } else {
                    prop_assert!(false, "expected named fields in variant");
                }
            }
            _ => prop_assert!(false, "expected enum"),
        }
    }

    /// Enum tuple variant fields are accessible.
    #[test]
    fn enum_tuple_variant_accessible(variant in type_name_strategy()) {
        let vn = format_ident!("{}", variant);
        let di: DeriveInput = parse_quote! {
            enum E { #vn(u32, String) }
        };
        match &di.data {
            syn::Data::Enum(data) => {
                let v = data.variants.first().unwrap();
                if let Fields::Unnamed(ref uf) = v.fields {
                    prop_assert_eq!(uf.unnamed.len(), 2);
                } else {
                    prop_assert!(false, "expected unnamed fields in variant");
                }
            }
            _ => prop_assert!(false, "expected enum"),
        }
    }

    /// Empty struct braces yields named fields with zero count.
    #[test]
    fn empty_named_struct_zero_fields(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let di: DeriveInput = parse_quote! { struct #ident {} };
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Named(ref nf) = ds.fields {
                    prop_assert!(nf.named.is_empty());
                } else {
                    prop_assert!(false, "expected named fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Type analysis consistency (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Simple types are not parameterized.
    #[test]
    fn simple_types_not_parameterized(name in simple_type_name()) {
        let t = ty(&name);
        prop_assert!(!is_parameterized(&t));
    }

    /// Wrapped types are parameterized.
    #[test]
    fn wrapped_types_are_parameterized(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        prop_assert!(is_parameterized(&t));
    }

    /// try_extract_inner_type on a non-matching type returns false.
    #[test]
    fn extract_non_matching_returns_false(name in simple_type_name()) {
        let t = ty(&name);
        let empty = skip_set(&[]);
        let (_, extracted) = try_extract_inner_type(&t, "Vec", &empty);
        prop_assert!(!extracted);
    }

    /// try_extract_inner_type on a matching wrapper returns the inner type.
    #[test]
    fn extract_matching_returns_inner(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let empty = skip_set(&[]);
        let (result, extracted) = try_extract_inner_type(&t, &wrapper, &empty);
        prop_assert!(extracted);
        prop_assert_eq!(ts(&result), inner);
    }

    /// filter_inner_type with empty skip set returns type unchanged.
    #[test]
    fn filter_empty_skip_is_identity(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let t = ty(&src);
        let empty = skip_set(&[]);
        let filtered = filter_inner_type(&t, &empty);
        prop_assert_eq!(ts(&filtered), ts(&t));
    }

    /// filter_inner_type strips a single skip wrapper.
    #[test]
    fn filter_strips_single_wrapper(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let wrapper_ref = wrapper.as_str();
        let arr = [wrapper_ref];
        let s = skip_set(&arr);
        let filtered = filter_inner_type(&t, &s);
        prop_assert_eq!(ts(&filtered), inner);
    }

    /// wrap_leaf_type wraps a simple type in adze::WithLeaf.
    #[test]
    fn wrap_simple_type_adds_with_leaf(name in simple_type_name()) {
        let t = ty(&name);
        let empty = skip_set(&[]);
        let wrapped = wrap_leaf_type(&t, &empty);
        let s = ts(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(&name));
    }

    /// wrap_leaf_type preserves skip-set containers.
    #[test]
    fn wrap_preserves_skip_container(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let wrapper_ref = wrapper.as_str();
        let arr = [wrapper_ref];
        let s = skip_set(&arr);
        let wrapped = wrap_leaf_type(&t, &s);
        let result = ts(&wrapped);
        prop_assert!(result.contains(&wrapper));
        prop_assert!(result.contains("adze :: WithLeaf"));
        prop_assert!(result.contains(&inner));
    }

    /// Extracting then wrapping the inner type produces a WithLeaf type.
    #[test]
    fn extract_then_wrap_produces_with_leaf(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let empty = skip_set(&[]);
        let (extracted_ty, ok) = try_extract_inner_type(&t, &wrapper, &empty);
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted_ty, &empty);
        let s = ts(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    /// filter_inner_type is idempotent: filtering twice equals filtering once.
    #[test]
    fn filter_is_idempotent(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let wrapper_ref = wrapper.as_str();
        let arr = [wrapper_ref];
        let s = skip_set(&arr);
        let once = filter_inner_type(&t, &s);
        let twice = filter_inner_type(&once, &s);
        prop_assert_eq!(ts(&once), ts(&twice));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: Attribute presence detection (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// adze::X attribute is detected by is_adze_attr.
    #[test]
    fn adze_attr_detected(attr_name in attr_name_strategy()) {
        let attr_ident = format_ident!("{}", attr_name);
        let di: DeriveInput = parse_quote! {
            #[adze::#attr_ident]
            struct S;
        };
        prop_assert!(!di.attrs.is_empty());
        prop_assert!(is_adze_attr(&di.attrs[0], attr_name));
    }

    /// A non-adze attribute is not detected as adze.
    #[test]
    fn non_adze_attr_not_detected(attr_name in attr_name_strategy()) {
        let di: DeriveInput = parse_quote! {
            #[derive(Debug)]
            struct S;
        };
        prop_assert!(!di.attrs.is_empty());
        prop_assert!(!is_adze_attr(&di.attrs[0], attr_name));
    }

    /// Multiple attributes can coexist, adze attrs are detected among them.
    #[test]
    fn adze_attr_found_among_multiple(attr_name in attr_name_strategy()) {
        let attr_ident = format_ident!("{}", attr_name);
        let di: DeriveInput = parse_quote! {
            #[derive(Debug)]
            #[adze::#attr_ident]
            struct S;
        };
        prop_assert_eq!(di.attrs.len(), 2);
        let adze_attrs: Vec<_> = di.attrs.iter().filter(|a| is_adze_attr(a, attr_name)).collect();
        prop_assert_eq!(adze_attrs.len(), 1);
    }

    /// Different adze attribute names are distinguishable.
    #[test]
    fn different_adze_attrs_distinguishable(
        a in attr_name_strategy(),
        b in attr_name_strategy(),
    ) {
        prop_assume!(a != b);
        let ai = format_ident!("{}", a);
        let di: DeriveInput = parse_quote! {
            #[adze::#ai]
            struct S;
        };
        prop_assert!(is_adze_attr(&di.attrs[0], a));
        prop_assert!(!is_adze_attr(&di.attrs[0], b));
    }

    /// Attribute on an enum is detected.
    #[test]
    fn adze_attr_on_enum(attr_name in attr_name_strategy()) {
        let attr_ident = format_ident!("{}", attr_name);
        let di: DeriveInput = parse_quote! {
            #[adze::#attr_ident]
            enum E { A }
        };
        prop_assert!(is_adze_attr(&di.attrs[0], attr_name));
    }

    /// Attribute count is preserved through roundtrip.
    #[test]
    fn attr_count_preserved_roundtrip(attr_name in attr_name_strategy()) {
        let attr_ident = format_ident!("{}", attr_name);
        let di: DeriveInput = parse_quote! {
            #[adze::#attr_ident]
            #[derive(Clone)]
            struct S;
        };
        let orig_count = di.attrs.len();
        let rt = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(rt).unwrap();
        prop_assert_eq!(di2.attrs.len(), orig_count);
    }

    /// A struct with no attributes has an empty attrs vec.
    #[test]
    fn no_attrs_is_empty(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let di: DeriveInput = parse_quote! { struct #ident; };
        prop_assert!(di.attrs.is_empty());
    }

    /// Attribute path segments for adze::X always have length 2.
    #[test]
    fn adze_attr_path_has_two_segments(attr_name in attr_name_strategy()) {
        let attr_ident = format_ident!("{}", attr_name);
        let di: DeriveInput = parse_quote! {
            #[adze::#attr_ident]
            struct S;
        };
        let seg_count = di.attrs[0].path().segments.len();
        prop_assert_eq!(seg_count, 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Token stream properties (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// quote! of a struct produces parseable tokens.
    #[test]
    fn quoted_struct_is_parseable(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let tokens = quote! { struct #ident { value: u32 } };
        let _: DeriveInput = syn::parse2(tokens).unwrap();
    }

    /// Token stream from DeriveInput is non-empty.
    #[test]
    fn derive_input_tokens_non_empty(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let di: DeriveInput = parse_quote! { struct #ident; };
        let tokens = di.to_token_stream();
        prop_assert!(!tokens.is_empty());
    }

    /// Token stream string contains the struct name.
    #[test]
    fn token_stream_contains_name(name in type_name_strategy()) {
        let ident = format_ident!("{}", name);
        let di: DeriveInput = parse_quote! { struct #ident; };
        let s = di.to_token_stream().to_string();
        prop_assert!(s.contains(&name));
    }

    /// Two different struct names produce different token streams.
    #[test]
    fn different_names_different_tokens(
        a in type_name_strategy(),
        b in type_name_strategy(),
    ) {
        prop_assume!(a != b);
        let ia = format_ident!("{}", a);
        let ib = format_ident!("{}", b);
        let ta = quote! { struct #ia; }.to_string();
        let tb = quote! { struct #ib; }.to_string();
        prop_assert_ne!(ta, tb);
    }

    /// Type token stream roundtrip: parse_str → tokens → parse2 is stable.
    #[test]
    fn type_token_roundtrip_stable(name in simple_type_name()) {
        let t1 = ty(&name);
        let tokens = t1.to_token_stream();
        let t2: Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ts(&t1), ts(&t2));
    }

    /// Wrapped type token roundtrip is stable.
    #[test]
    fn wrapped_type_token_roundtrip_stable(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let t1 = ty(&src);
        let tokens = t1.to_token_stream();
        let t2: Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ts(&t1), ts(&t2));
    }

    /// quote! interpolation of a type preserves the type string.
    #[test]
    fn quote_type_interpolation_preserves(name in simple_type_name()) {
        let t: Type = parse_str(&name).unwrap();
        let tokens = quote! { #t };
        let t2: Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ts(&t), ts(&t2));
    }

    /// Token stream from an enum with adze attr is re-parseable.
    #[test]
    fn enum_with_attr_reparseable(
        attr_name in attr_name_strategy(),
        variant in type_name_strategy(),
    ) {
        let ai = format_ident!("{}", attr_name);
        let vn = format_ident!("{}", variant);
        let di: DeriveInput = parse_quote! {
            #[adze::#ai]
            enum E { #vn }
        };
        let tokens = di.to_token_stream();
        let di2: DeriveInput = syn::parse2(tokens).unwrap();
        prop_assert_eq!(di2.ident.to_string(), "E");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Cross-cutting consistency (8+ tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// wrap_leaf_type is idempotent for simple types:
    /// wrapping an already-wrapped type wraps it again but the pattern is consistent.
    #[test]
    fn wrap_then_rewrap_contains_nested_with_leaf(name in simple_type_name()) {
        let t = ty(&name);
        let empty = skip_set(&[]);
        let wrapped = wrap_leaf_type(&t, &empty);
        let rewrapped = wrap_leaf_type(&wrapped, &empty);
        let s = ts(&rewrapped);
        // Should contain nested WithLeaf wrappers
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(&name));
    }

    /// extract + filter chain: extract inner then filter is consistent.
    #[test]
    fn extract_then_filter_consistent(
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("Vec<Box<{inner}>>"));
        let skip = skip_set(&["Box"]);
        let (extracted, ok) = try_extract_inner_type(&t, "Vec", &skip);
        prop_assert!(ok);
        let filtered = filter_inner_type(&extracted, &skip);
        prop_assert_eq!(ts(&filtered), inner);
    }

    /// is_parameterized agrees with try_extract_inner_type success.
    #[test]
    fn parameterized_agrees_with_extraction(
        wrapper in wrapper_name(),
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("{wrapper}<{inner}>"));
        let empty = skip_set(&[]);
        let (_, extracted) = try_extract_inner_type(&t, &wrapper, &empty);
        prop_assert!(is_parameterized(&t));
        prop_assert!(extracted);
    }

    /// Non-parameterized types fail extraction.
    #[test]
    fn non_parameterized_fails_extraction(name in simple_type_name()) {
        let t = ty(&name);
        let empty = skip_set(&[]);
        prop_assert!(!is_parameterized(&t));
        let (_, extracted) = try_extract_inner_type(&t, "Vec", &empty);
        prop_assert!(!extracted);
    }

    /// try_extract_inner_type through skip-over finds nested target.
    #[test]
    fn extract_through_skip_finds_nested(
        inner in simple_type_name(),
    ) {
        let t = ty(&format!("Box<Vec<{inner}>>"));
        let skip = skip_set(&["Box"]);
        let (result, extracted) = try_extract_inner_type(&t, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ts(&result), inner);
    }

    /// filter_inner_type strips nested skip wrappers.
    #[test]
    fn filter_strips_nested_wrappers(inner in simple_type_name()) {
        let t = ty(&format!("Box<Arc<{inner}>>"));
        let skip = skip_set(&["Box", "Arc"]);
        let filtered = filter_inner_type(&t, &skip);
        prop_assert_eq!(ts(&filtered), inner);
    }

    /// wrap_leaf_type with nested skip containers wraps only the leaf.
    #[test]
    fn wrap_nested_skip_wraps_leaf_only(inner in simple_type_name()) {
        let t = ty(&format!("Vec<Option<{inner}>>"));
        let skip = skip_set(&["Vec", "Option"]);
        let wrapped = wrap_leaf_type(&t, &skip);
        let s = ts(&wrapped);
        prop_assert!(s.contains("Vec"));
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(&inner));
    }

    /// DeriveInput field type survives extraction → wrap → token roundtrip.
    #[test]
    fn field_type_extract_wrap_roundtrip(inner in simple_type_name()) {
        let inner_ty: Type = parse_str(&inner).unwrap();
        let di: DeriveInput = parse_quote! {
            struct S { value: Vec<#inner_ty> }
        };
        match &di.data {
            syn::Data::Struct(ds) => {
                if let Fields::Named(ref nf) = ds.fields {
                    let field_ty = &nf.named.first().unwrap().ty;
                    let empty = skip_set(&[]);
                    let (extracted, ok) = try_extract_inner_type(field_ty, "Vec", &empty);
                    prop_assert!(ok);
                    let wrapped = wrap_leaf_type(&extracted, &empty);
                    let s = ts(&wrapped);
                    prop_assert!(s.contains("adze :: WithLeaf"));
                    prop_assert!(s.contains(&inner));
                } else {
                    prop_assert!(false, "expected named fields");
                }
            }
            _ => prop_assert!(false, "expected struct"),
        }
    }
}
