#![allow(clippy::needless_range_loop)]

//! Stress tests for adze-common functions with unusual and extreme inputs.
//!
//! Exercises `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! `NameValueExpr`, and `FieldThenParams` with very long names, deeply nested
//! generics, many generic parameters, special identifiers, large token streams,
//! repeated operations, and memory-safety-under-stress scenarios.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ===========================================================================
// Helper: build a deeply nested type like Box<Box<Box<...<Leaf>...>>>
// ===========================================================================

fn nested_type(wrapper: &str, depth: usize, leaf: &str) -> Type {
    let leaf_ident = syn::Ident::new(leaf, proc_macro2::Span::call_site());
    let mut ty: Type = parse_quote!(#leaf_ident);
    let wrapper_ident = syn::Ident::new(wrapper, proc_macro2::Span::call_site());
    for _ in 0..depth {
        ty = parse_quote!(#wrapper_ident<#ty>);
    }
    ty
}

// ===========================================================================
// 1. Very long type names
// ===========================================================================

#[test]
fn stress_very_long_type_name_1000_chars() {
    let long_name = "T".repeat(1000);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(#ident);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    let s = wrapped.to_token_stream().to_string();
    assert!(s.contains(&long_name));
    assert!(s.starts_with("adze :: WithLeaf"));
}

#[test]
fn stress_long_name_extract_from_option() {
    let long_name = "MyVeryLongTypeName".repeat(50);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Option<#ident>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), long_name);
}

#[test]
fn stress_long_name_filter_through_box() {
    let long_name = "Z".repeat(2000);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Box<#ident>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), long_name);
}

#[test]
fn stress_long_name_in_field_then_params() {
    let long_name = "Field".repeat(200);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let parsed: FieldThenParams = parse_quote!(#ident, precedence = 42);
    assert_eq!(parsed.field.ty.to_token_stream().to_string(), long_name);
    assert_eq!(parsed.params.len(), 1);
}

// ===========================================================================
// 2. Deeply nested generics (10+ levels)
// ===========================================================================

#[test]
fn stress_nested_box_10_levels_filter() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = nested_type("Box", 10, "Core");
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Core");
}

#[test]
fn stress_nested_box_15_levels_filter() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = nested_type("Box", 15, "Leaf");
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn stress_nested_box_20_levels_extract() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    // Box<Box<...<Option<Inner>>...>> with 20 Box layers then Option
    let inner_ident = syn::Ident::new("Inner", proc_macro2::Span::call_site());
    let mut ty: Type = parse_quote!(Option<#inner_ident>);
    let wrapper = syn::Ident::new("Box", proc_macro2::Span::call_site());
    for _ in 0..20 {
        ty = parse_quote!(#wrapper<#ty>);
    }
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Inner");
}

#[test]
fn stress_nested_mixed_wrappers_12_levels() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    // Build Box<Arc<Rc<Box<Arc<Rc<Box<Arc<Rc<Box<Arc<Rc<Leaf>>>>>>>>>>>>
    let wrappers = ["Box", "Arc", "Rc"];
    let leaf_ident = syn::Ident::new("Leaf", proc_macro2::Span::call_site());
    let mut ty: Type = parse_quote!(#leaf_ident);
    for i in 0..12 {
        let w = syn::Ident::new(wrappers[i % 3], proc_macro2::Span::call_site());
        ty = parse_quote!(#w<#ty>);
    }
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn stress_nested_wrap_leaf_10_levels() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    // Vec<Option<Vec<Option<Vec<Option<Vec<Option<Vec<Option<Token>>>>>>>>>>
    let wrappers = ["Vec", "Option"];
    let leaf_ident = syn::Ident::new("Token", proc_macro2::Span::call_site());
    let mut ty: Type = parse_quote!(#leaf_ident);
    for i in 0..10 {
        let w = syn::Ident::new(wrappers[i % 2], proc_macro2::Span::call_site());
        ty = parse_quote!(#w<#ty>);
    }
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    // The leaf Token should be wrapped with WithLeaf
    assert!(s.contains("adze :: WithLeaf < Token >"));
    // The outer containers should be preserved
    assert!(s.starts_with("Option"));
}

// ===========================================================================
// 3. Types with many generic parameters
// ===========================================================================

#[test]
fn stress_result_two_generics_wrapped() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, Error>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    assert!(s.contains("adze :: WithLeaf < String >"));
    assert!(s.contains("adze :: WithLeaf < Error >"));
}

#[test]
fn stress_custom_type_three_generics_wrapped() {
    let skip: HashSet<&str> = ["Triple"].into_iter().collect();
    let ty: Type = parse_quote!(Triple<A, B, C>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    assert!(s.contains("adze :: WithLeaf < A >"));
    assert!(s.contains("adze :: WithLeaf < B >"));
    assert!(s.contains("adze :: WithLeaf < C >"));
}

#[test]
fn stress_many_generic_params_not_in_skip_set() {
    // When a multi-param type is NOT in the skip set, the whole thing gets wrapped
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    assert!(s.starts_with("adze :: WithLeaf < HashMap"));
}

// ===========================================================================
// 4. Single-character and underscore-prefixed identifiers
// ===========================================================================

#[test]
fn stress_single_char_type_name() {
    let ty: Type = parse_quote!(X);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < X >"
    );
}

#[test]
fn stress_underscore_prefixed_type_name() {
    let ty: Type = parse_quote!(_Private);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < _Private >"
    );
}

#[test]
fn stress_underscore_prefixed_in_skip_set() {
    let skip: HashSet<&str> = ["_Wrapper"].into_iter().collect();
    let ty: Type = parse_quote!(_Wrapper<Inner>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Inner");
}

// ===========================================================================
// 5. Special identifier patterns (unicode, numeric suffixes)
// ===========================================================================

#[test]
fn stress_unicode_identifiers_in_all_positions() {
    let skip: HashSet<&str> = ["Контейнер"].into_iter().collect();
    let ty: Type = parse_quote!(Контейнер<Содержимое>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Содержимое");

    let (inner, extracted) = try_extract_inner_type(&ty, "Контейнер", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Содержимое");
}

#[test]
fn stress_numeric_suffix_type_names() {
    let ty: Type = parse_quote!(Type123<Type456>);
    let skip: HashSet<&str> = ["Type123"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Type456");
}

// ===========================================================================
// 6. Large-scale FieldThenParams parsing
// ===========================================================================

#[test]
fn stress_field_then_params_many_params() {
    let parsed: FieldThenParams = parse_quote!(
        String,
        a = 1,
        b = 2,
        c = 3,
        d = 4,
        e = 5,
        f = 6,
        g = 7,
        h = 8,
        i = 9,
        j = 10
    );
    assert_eq!(parsed.params.len(), 10);
    for (idx, param) in parsed.params.iter().enumerate() {
        let expected_name = (b'a' + idx as u8) as char;
        assert_eq!(param.path.to_string(), expected_name.to_string());
    }
}

#[test]
fn stress_name_value_expr_long_string_value() {
    let long_val = "x".repeat(5000);
    let expr: NameValueExpr = syn::parse_str(&format!("pattern = \"{}\"", long_val)).unwrap();
    assert_eq!(expr.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &expr.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value().len(), 5000);
        } else {
            panic!("Expected string literal");
        }
    } else {
        panic!("Expected literal expression");
    }
}

// ===========================================================================
// 7. Repeated operations on the same type (idempotency, stability)
// ===========================================================================

#[test]
fn stress_repeated_filter_is_idempotent() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let first = filter_inner_type(&ty, &skip);
    let second = filter_inner_type(&first, &skip);
    let third = filter_inner_type(&second, &skip);
    // Once Box is stripped, further applications leave String unchanged
    assert_eq!(first.to_token_stream().to_string(), "String");
    assert_eq!(second.to_token_stream().to_string(), "String");
    assert_eq!(third.to_token_stream().to_string(), "String");
}

#[test]
fn stress_repeated_wrap_accumulates() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Leaf);
    let once = wrap_leaf_type(&ty, &skip);
    let twice = wrap_leaf_type(&once, &skip);
    let thrice = wrap_leaf_type(&twice, &skip);
    // Each wrap adds another WithLeaf layer
    assert_eq!(
        once.to_token_stream().to_string(),
        "adze :: WithLeaf < Leaf >"
    );
    assert_eq!(
        twice.to_token_stream().to_string(),
        "adze :: WithLeaf < adze :: WithLeaf < Leaf > >"
    );
    assert_eq!(
        thrice.to_token_stream().to_string(),
        "adze :: WithLeaf < adze :: WithLeaf < adze :: WithLeaf < Leaf > > >"
    );
}

#[test]
fn stress_repeated_extract_same_type_100_times() {
    let ty: Type = parse_quote!(Option<String>);
    for _ in 0..100 {
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "String");
    }
}

#[test]
fn stress_repeated_operations_do_not_mutate_original() {
    let ty: Type = parse_quote!(Box<Vec<Inner>>);
    let original_str = ty.to_token_stream().to_string();

    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    for _ in 0..50 {
        let _ = filter_inner_type(&ty, &skip);
        let _ = try_extract_inner_type(&ty, "Vec", &skip);
        let _ = wrap_leaf_type(&ty, &["Vec"].into_iter().collect());
    }
    // Original type should be unchanged after all operations
    assert_eq!(ty.to_token_stream().to_string(), original_str);
}

// ===========================================================================
// 8. Memory safety under stress — large batch operations
// ===========================================================================

#[test]
fn stress_batch_wrap_200_types() {
    let skip: HashSet<&str> = HashSet::new();
    let results: Vec<String> = (0..200)
        .map(|i| {
            let name = syn::Ident::new(&format!("Type{i}"), proc_macro2::Span::call_site());
            let ty: Type = parse_quote!(#name);
            wrap_leaf_type(&ty, &skip).to_token_stream().to_string()
        })
        .collect();
    assert_eq!(results.len(), 200);
    assert_eq!(results[0], "adze :: WithLeaf < Type0 >");
    assert_eq!(results[199], "adze :: WithLeaf < Type199 >");
}

#[test]
fn stress_batch_filter_200_types() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let results: Vec<String> = (0..200)
        .map(|i| {
            let name = syn::Ident::new(&format!("Inner{i}"), proc_macro2::Span::call_site());
            let ty: Type = parse_quote!(Box<#name>);
            filter_inner_type(&ty, &skip).to_token_stream().to_string()
        })
        .collect();
    assert_eq!(results.len(), 200);
    for i in 0..200 {
        assert_eq!(results[i], format!("Inner{i}"));
    }
}

#[test]
fn stress_batch_extract_200_types() {
    let results: Vec<(String, bool)> = (0..200)
        .map(|i| {
            let name = syn::Ident::new(&format!("Val{i}"), proc_macro2::Span::call_site());
            let ty: Type = if i % 2 == 0 {
                parse_quote!(Option<#name>)
            } else {
                parse_quote!(#name)
            };
            let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
            (inner.to_token_stream().to_string(), extracted)
        })
        .collect();
    assert_eq!(results.len(), 200);
    for i in 0..200 {
        if i % 2 == 0 {
            assert!(results[i].1, "Expected extraction at index {i}");
            assert_eq!(results[i].0, format!("Val{i}"));
        } else {
            assert!(!results[i].1, "Expected no extraction at index {i}");
        }
    }
}

// ===========================================================================
// 9. Edge-case skip sets
// ===========================================================================

#[test]
fn stress_large_skip_set() {
    let wrappers: Vec<String> = (0..50).map(|i| format!("Wrapper{i}")).collect();
    let skip: HashSet<&str> = wrappers.iter().map(|s| s.as_str()).collect();

    // Only Wrapper0 is actually used as the outermost wrapper
    let ty: Type = parse_quote!(Wrapper0<Payload>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Payload");
}

#[test]
fn stress_skip_set_with_no_matching_types() {
    let skip: HashSet<&str> = ["Nonexistent", "AlsoMissing", "NotHere"]
        .into_iter()
        .collect();
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let filtered = filter_inner_type(&ty, &skip);
    // None of the wrappers match the skip set, so type is returned as-is
    assert_eq!(
        filtered.to_token_stream().to_string(),
        "Box < Vec < Option < String > > >"
    );
}

// ===========================================================================
// 10. Non-path types under stress
// ===========================================================================

#[test]
fn stress_reference_types_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    // Reference type is not Type::Path, so all ops return it unchanged/wrapped
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "& str");

    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "& str");

    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn stress_tuple_types_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!((i32, u64, String));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "(i32 , u64 , String)");

    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < (i32 , u64 , String) >"
    );
}

#[test]
fn stress_array_type_passthrough() {
    let ty: Type = parse_quote!([u8; 256]);
    let filtered = filter_inner_type(&ty, &["Box"].into_iter().collect());
    assert_eq!(filtered.to_token_stream().to_string(), "[u8 ; 256]");

    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < [u8 ; 256] >"
    );
}

// ===========================================================================
// 11. Fully qualified paths
// ===========================================================================

#[test]
fn stress_fully_qualified_path_type() {
    // std::vec::Vec<T> — last segment is Vec, so it should work with skip set
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(std::vec::Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // The last segment is Vec, which is in skip set
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn stress_crate_path_extract() {
    let ty: Type = parse_quote!(std::option::Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ===========================================================================
// 12. Combining operations in sequence
// ===========================================================================

#[test]
fn stress_extract_then_wrap_then_filter_chain() {
    let ty: Type = parse_quote!(Option<Box<Leaf>>);
    let extract_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();

    // Step 1: extract Option
    let (after_extract, extracted) = try_extract_inner_type(&ty, "Option", &extract_skip);
    assert!(extracted);
    assert_eq!(after_extract.to_token_stream().to_string(), "Box < Leaf >");

    // Step 2: filter Box
    let after_filter = filter_inner_type(&after_extract, &filter_skip);
    assert_eq!(after_filter.to_token_stream().to_string(), "Leaf");

    // Step 3: wrap
    let after_wrap = wrap_leaf_type(&after_filter, &wrap_skip);
    assert_eq!(
        after_wrap.to_token_stream().to_string(),
        "adze :: WithLeaf < Leaf >"
    );
}
