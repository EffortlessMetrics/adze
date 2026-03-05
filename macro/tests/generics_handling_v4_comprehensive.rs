//! Comprehensive tests for generic type handling patterns in adze-macro.
//!
//! Covers: simple generic types, lifetime parameters, type parameter extraction,
//! where clauses, generic argument iteration, multiple type params, complex
//! generic patterns, and edge cases.

use quote::{ToTokens, format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, GenericArgument, GenericParam, Generics, LifetimeParam,
    PathArguments, Type, TypeParam, TypeParamBound, parse_quote, parse_str, parse2,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn extract_inner_type_name(ty: &Type, wrapper: &str) -> Option<String> {
    if let Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident == wrapper
            && let PathArguments::AngleBracketed(args) = &seg.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            return Some(type_to_string(inner));
        }
    }
    None
}

fn count_generic_params(generics: &Generics) -> usize {
    generics.params.len()
}

fn has_where_clause(generics: &Generics) -> bool {
    generics.where_clause.is_some()
}

// =====================================================================
// 1. Simple generic types (8 tests)
// =====================================================================

#[test]
fn simple_option_type_parsed() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(extract_inner_type_name(&ty, "Option").unwrap(), "String");
}

#[test]
fn simple_vec_type_parsed() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(extract_inner_type_name(&ty, "Vec").unwrap(), "i32");
}

#[test]
fn simple_box_type_parsed() {
    let ty: Type = parse_quote!(Box<dyn std::fmt::Debug>);
    assert!(extract_inner_type_name(&ty, "Box").is_some());
}

#[test]
fn option_of_vec_nested() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let inner = extract_inner_type_name(&ty, "Option").unwrap();
    assert!(inner.contains("Vec"));
}

#[test]
fn vec_of_option_nested() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let inner = extract_inner_type_name(&ty, "Vec").unwrap();
    assert!(inner.contains("Option"));
}

#[test]
fn box_of_box_nested() {
    let ty: Type = parse_quote!(Box<Box<i64>>);
    let inner = extract_inner_type_name(&ty, "Box").unwrap();
    assert!(inner.contains("Box"));
}

#[test]
fn simple_generic_not_matching_wrapper() {
    let ty: Type = parse_quote!(Vec<u32>);
    assert!(extract_inner_type_name(&ty, "Option").is_none());
}

#[test]
fn simple_non_generic_type_returns_none() {
    let ty: Type = parse_quote!(String);
    assert!(extract_inner_type_name(&ty, "Option").is_none());
}

// =====================================================================
// 2. Lifetime parameters (5 tests)
// =====================================================================

#[test]
fn parse_single_lifetime_param() {
    let generics: Generics = parse_quote!(<'a>);
    assert_eq!(count_generic_params(&generics), 1);
    assert!(matches!(
        generics.params.first().unwrap(),
        GenericParam::Lifetime(_)
    ));
}

#[test]
fn parse_static_lifetime_in_type() {
    let ty: Type = parse_quote!(&'static str);
    let s = type_to_string(&ty);
    assert!(s.contains("static"));
    assert!(s.contains("str"));
}

#[test]
fn lifetime_with_type_param() {
    let generics: Generics = parse_quote!(<'a, T>);
    assert_eq!(count_generic_params(&generics), 2);
    let first = &generics.params[0];
    let second = &generics.params[1];
    assert!(matches!(first, GenericParam::Lifetime(_)));
    assert!(matches!(second, GenericParam::Type(_)));
}

#[test]
fn multiple_lifetimes() {
    let generics: Generics = parse_quote!(<'a, 'b>);
    assert_eq!(count_generic_params(&generics), 2);
    for param in &generics.params {
        assert!(matches!(param, GenericParam::Lifetime(_)));
    }
}

#[test]
fn lifetime_bound_on_lifetime() {
    let generics: Generics = parse_quote!(<'a, 'b: 'a>);
    assert_eq!(count_generic_params(&generics), 2);
    if let GenericParam::Lifetime(LifetimeParam { bounds, .. }) = &generics.params[1] {
        assert!(!bounds.is_empty());
    } else {
        panic!("Expected lifetime param with bound");
    }
}

// =====================================================================
// 3. Type parameter extraction (8 tests)
// =====================================================================

#[test]
fn extract_single_type_param_ident() {
    let generics: Generics = parse_quote!(<T>);
    if let GenericParam::Type(TypeParam { ident, .. }) = generics.params.first().unwrap() {
        assert_eq!(ident.to_string(), "T");
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn extract_bounded_type_param() {
    let generics: Generics = parse_quote!(<T: Clone>);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        assert_eq!(tp.ident.to_string(), "T");
        assert!(!tp.bounds.is_empty());
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn extract_type_param_with_default() {
    let generics: Generics = parse_quote!(<T = String>);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        assert!(tp.default.is_some());
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn angle_bracketed_args_from_path() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &seg.arguments
        {
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected angle bracketed args");
        }
    } else {
        panic!("Expected path type");
    }
}

#[test]
fn nested_angle_brackets_depth_two() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            if let Some(GenericArgument::Type(Type::Path(inner_tp))) = ab.args.first() {
                let inner_seg = inner_tp.path.segments.last().unwrap();
                assert_eq!(inner_seg.ident.to_string(), "Option");
                assert!(matches!(
                    inner_seg.arguments,
                    PathArguments::AngleBracketed(_)
                ));
            } else {
                panic!("Expected inner type path");
            }
        }
    }
}

#[test]
fn nested_angle_brackets_depth_three() {
    let ty: Type = parse_quote!(Vec<Option<Box<u8>>>);
    let s = type_to_string(&ty);
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
    assert!(s.contains("Box"));
    assert!(s.contains("u8"));
}

#[test]
fn type_param_multiple_bounds() {
    let generics: Generics = parse_quote!(<T: Clone + Send + Sync>);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        assert!(tp.bounds.len() >= 3);
    } else {
        panic!("Expected type param");
    }
}

#[test]
fn type_param_lifetime_bound() {
    let generics: Generics = parse_quote!(<T: 'static>);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        let has_lifetime = tp
            .bounds
            .iter()
            .any(|b| matches!(b, TypeParamBound::Lifetime(_)));
        assert!(has_lifetime);
    } else {
        panic!("Expected type param");
    }
}

// =====================================================================
// 4. Where clause patterns (5 tests)
// =====================================================================

#[test]
fn simple_where_clause_parsed() {
    let item: syn::ItemStruct = parse_quote!(
        struct Foo<T>
        where
            T: Clone,
        {
            val: T,
        }
    );
    assert!(has_where_clause(&item.generics));
    let wc = item.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_multiple_predicates() {
    let item: syn::ItemStruct = parse_quote!(
        struct Foo<T, U>
        where
            T: Clone,
            U: Default,
        {
            a: T,
            b: U,
        }
    );
    let wc = item.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn where_clause_complex_bound() {
    let item: syn::ItemStruct = parse_quote!(
        struct Foo<T>
        where
            T: Iterator<Item = u32>,
        {
            val: T,
        }
    );
    assert!(has_where_clause(&item.generics));
    let wc = item.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_lifetime_bound() {
    let item: syn::ItemStruct = parse_quote!(
        struct Foo<'a, T>
        where
            T: 'a,
        {
            val: &'a T,
        }
    );
    assert!(has_where_clause(&item.generics));
}

#[test]
fn no_where_clause() {
    let generics: Generics = parse_quote!(<T: Clone>);
    assert!(!has_where_clause(&generics));
}

// =====================================================================
// 5. Generic argument iteration (8 tests)
// =====================================================================

#[test]
fn iterate_single_generic_arg() {
    let ty: Type = parse_quote!(Vec<String>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            let args: Vec<_> = ab.args.iter().collect();
            assert_eq!(args.len(), 1);
        }
    }
}

#[test]
fn iterate_two_generic_args() {
    let ty: Type = parse_quote!(Result<String, Error>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            let args: Vec<_> = ab.args.iter().collect();
            assert_eq!(args.len(), 2);
        }
    }
}

#[test]
fn iterate_three_generic_args() {
    let ty: Type = parse_quote!(MyType<A, B, C>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            assert_eq!(ab.args.len(), 3);
        }
    }
}

#[test]
fn generic_arg_is_type() {
    let ty: Type = parse_quote!(Vec<u32>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            assert!(matches!(ab.args.first().unwrap(), GenericArgument::Type(_)));
        }
    }
}

#[test]
fn generic_arg_is_lifetime() {
    let ty: Type = parse_quote!(Cow<'static, str>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            let first = ab.args.first().unwrap();
            assert!(matches!(first, GenericArgument::Lifetime(_)));
        }
    }
}

#[test]
fn generic_args_collect_types_only() {
    let ty: Type = parse_quote!(Cow<'a, [u8]>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            let types: Vec<_> = ab
                .args
                .iter()
                .filter_map(|a| {
                    if let GenericArgument::Type(t) = a {
                        Some(t)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(types.len(), 1);
        }
    }
}

#[test]
fn generic_args_empty_when_no_brackets() {
    let ty: Type = parse_quote!(i32);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        assert!(matches!(seg.arguments, PathArguments::None));
    }
}

#[test]
fn generic_args_binding_form() {
    let ty: Type = parse_quote!(Iterator<Item = u32>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            assert_eq!(ab.args.len(), 1);
            // AssocType in newer syn
            assert!(!matches!(
                ab.args.first().unwrap(),
                GenericArgument::Type(_)
            ));
        }
    }
}

// =====================================================================
// 6. Multiple type params (5 tests)
// =====================================================================

#[test]
fn two_type_params() {
    let generics: Generics = parse_quote!(<A, B>);
    assert_eq!(count_generic_params(&generics), 2);
    let names: Vec<_> = generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    assert_eq!(names, ["A", "B"]);
}

#[test]
fn three_type_params_with_bounds() {
    let generics: Generics = parse_quote!(<T: Clone, U: Default, V: Send>);
    assert_eq!(count_generic_params(&generics), 3);
    for tp in generics.type_params() {
        assert!(!tp.bounds.is_empty());
    }
}

#[test]
fn mixed_lifetime_and_type_params() {
    let generics: Generics = parse_quote!(<'a, T, U>);
    assert_eq!(count_generic_params(&generics), 3);
    assert_eq!(generics.lifetimes().count(), 1);
    assert_eq!(generics.type_params().count(), 2);
}

#[test]
fn type_params_iteration_order() {
    let generics: Generics = parse_quote!(<X, Y, Z>);
    let names: Vec<_> = generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    assert_eq!(names, ["X", "Y", "Z"]);
}

#[test]
fn type_param_with_trait_and_lifetime_bound() {
    let generics: Generics = parse_quote!(<T: Clone + 'static>);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        assert!(tp.bounds.len() >= 2);
        let has_trait = tp
            .bounds
            .iter()
            .any(|b| matches!(b, TypeParamBound::Trait(_)));
        let has_lt = tp
            .bounds
            .iter()
            .any(|b| matches!(b, TypeParamBound::Lifetime(_)));
        assert!(has_trait);
        assert!(has_lt);
    } else {
        panic!("Expected type param");
    }
}

// =====================================================================
// 7. Complex generic patterns (8 tests)
// =====================================================================

#[test]
fn result_type_two_params() {
    let ty: Type = parse_quote!(Result<String, std::io::Error>);
    let ok = extract_inner_type_name(&ty, "Result").unwrap();
    assert_eq!(ok, "String");
}

#[test]
fn hashmap_type_two_params() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    if let Type::Path(tp) = &ty {
        let seg = tp.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            assert_eq!(ab.args.len(), 2);
        }
    }
}

#[test]
fn btreemap_qualified_path() {
    let ty: Type = parse_quote!(std::collections::BTreeMap<String, u64>);
    let s = type_to_string(&ty);
    assert!(s.contains("BTreeMap"));
    assert!(s.contains("String"));
    assert!(s.contains("u64"));
}

#[test]
fn fn_trait_generic_arg() {
    let ty: Type = parse_quote!(Box<dyn Fn(i32) -> bool>);
    let s = type_to_string(&ty);
    assert!(s.contains("Fn"));
    assert!(s.contains("bool"));
}

#[test]
fn tuple_type_in_generic() {
    let ty: Type = parse_quote!(Vec<(String, i32)>);
    let inner = extract_inner_type_name(&ty, "Vec").unwrap();
    assert!(inner.contains("String"));
    assert!(inner.contains("i32"));
}

#[test]
fn array_type_in_generic() {
    let ty: Type = parse_quote!(Vec<[u8; 4]>);
    let inner = extract_inner_type_name(&ty, "Vec").unwrap();
    assert!(inner.contains("u8"));
}

#[test]
fn deeply_nested_generic() {
    let ty: Type = parse_quote!(Option<Result<Vec<Box<String>>, std::io::Error>>);
    let inner = extract_inner_type_name(&ty, "Option").unwrap();
    assert!(inner.contains("Result"));
    assert!(inner.contains("Vec"));
    assert!(inner.contains("Box"));
}

#[test]
fn generic_with_reference_inner() {
    let ty: Type = parse_quote!(Vec<&str>);
    let inner = extract_inner_type_name(&ty, "Vec").unwrap();
    assert!(inner.contains("str"));
}

// =====================================================================
// 8. Edge cases (8 tests)
// =====================================================================

#[test]
fn no_generics_empty() {
    let generics: Generics = parse_quote!();
    assert_eq!(count_generic_params(&generics), 0);
    assert!(!has_where_clause(&generics));
}

#[test]
fn phantom_data_type() {
    let ty: Type = parse_quote!(std::marker::PhantomData<T>);
    let s = type_to_string(&ty);
    assert!(s.contains("PhantomData"));
    assert!(s.contains("T"));
}

#[test]
fn const_generic_param() {
    let generics: Generics = parse_quote!(<const N: usize>);
    assert_eq!(count_generic_params(&generics), 1);
    assert!(matches!(
        generics.params.first().unwrap(),
        GenericParam::Const(_)
    ));
}

#[test]
fn const_generic_with_type_param() {
    let generics: Generics = parse_quote!(<T, const N: usize>);
    assert_eq!(count_generic_params(&generics), 2);
    assert!(matches!(&generics.params[0], GenericParam::Type(_)));
    assert!(matches!(&generics.params[1], GenericParam::Const(_)));
}

#[test]
fn unit_type_no_generics() {
    let ty: Type = parse_quote!(());
    assert!(extract_inner_type_name(&ty, "Option").is_none());
}

#[test]
fn reference_type_not_path() {
    let ty: Type = parse_quote!(&u32);
    assert!(extract_inner_type_name(&ty, "Option").is_none());
}

#[test]
fn slice_type_not_path() {
    let ty: Type = parse_quote!([u8]);
    assert!(extract_inner_type_name(&ty, "Vec").is_none());
}

#[test]
fn raw_pointer_type_not_path() {
    let ty: Type = parse_quote!(*const u8);
    assert!(extract_inner_type_name(&ty, "Box").is_none());
}

// =====================================================================
// Additional coverage: roundtrip via quote/parse (3 tests)
// =====================================================================

#[test]
fn quote_roundtrip_generic_type() {
    let original: Type = parse_quote!(Option<Vec<String>>);
    let tokens = original.to_token_stream();
    let parsed: Type = parse2(tokens).unwrap();
    assert_eq!(type_to_string(&original), type_to_string(&parsed));
}

#[test]
fn quote_roundtrip_generics_with_where() {
    let item: syn::ItemStruct = parse_quote!(
        struct Foo<T: Clone>
        where
            T: Default,
        {
            val: T,
        }
    );
    let generics = &item.generics;
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let tokens = quote!(struct Bar #impl_generics #where_clause { val: T });
    let reparsed: syn::ItemStruct = parse2(tokens).unwrap();
    assert_eq!(count_generic_params(&reparsed.generics), 1);
    assert!(has_where_clause(&reparsed.generics));
}

#[test]
fn format_ident_creates_valid_type_param() {
    let name = format_ident!("MyParam");
    let generics: Generics = parse2(quote!(<#name>)).unwrap();
    assert_eq!(count_generic_params(&generics), 1);
    if let GenericParam::Type(tp) = generics.params.first().unwrap() {
        assert_eq!(tp.ident.to_string(), "MyParam");
    }
}

// =====================================================================
// Additional: parse_str based parsing (2 tests)
// =====================================================================

#[test]
fn parse_str_option_type() {
    let ty: Type = parse_str("Option<i64>").unwrap();
    assert_eq!(extract_inner_type_name(&ty, "Option").unwrap(), "i64");
}

#[test]
fn parse_str_result_type() {
    let ty: Type = parse_str("Result<(), String>").unwrap();
    assert_eq!(extract_inner_type_name(&ty, "Result").unwrap(), "()");
}
