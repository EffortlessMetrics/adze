use adze_syn_type_utils::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

#[test]
fn extracts_inner_type_with_skipped_containers() {
    let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);
    let ty: Type = parse_quote!(Box<Vec<String>>);

    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);

    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "String");
}

#[test]
fn filters_skip_containers() {
    let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);
    let ty: Type = parse_quote!(Box<Arc<String>>);

    let filtered = filter_inner_type(&ty, &skip_over);

    assert_eq!(quote::quote!(#filtered).to_string(), "String");
}

#[test]
fn wraps_only_leaf_type() {
    let skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let ty: Type = parse_quote!(Option<Vec<String>>);

    let wrapped = wrap_leaf_type(&ty, &skip_over);

    assert_eq!(
        quote::quote!(#wrapped).to_string(),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}
