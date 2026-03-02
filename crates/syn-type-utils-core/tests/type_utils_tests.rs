use adze_syn_type_utils_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

#[test]
fn test_try_extract_inner_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Box");
    skip_over.insert("Vec");

    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "String");

    let ty: Type = parse_quote!(Box<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "i32");

    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "bool");

    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "String");
}

#[test]
fn test_filter_inner_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Box");
    skip_over.insert("Vec");
    skip_over.insert("Option");

    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "String");

    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "i32");

    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "String");
}

#[test]
fn test_wrap_leaf_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Vec");
    skip_over.insert("Option");

    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(
        quote::quote!(#wrapped).to_string(),
        "adze :: WithLeaf < String >"
    );

    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(
        quote::quote!(#wrapped).to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );

    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(
        quote::quote!(#wrapped).to_string(),
        "Option < Vec < adze :: WithLeaf < i32 > > >"
    );
}
