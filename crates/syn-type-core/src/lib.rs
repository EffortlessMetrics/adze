//! Type-shape helpers shared by adze code generation crates.

#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::collections::HashSet;
use syn::{GenericArgument, PathArguments, Type, parse_quote};

/// Attempts to extract the inner type from a container type.
pub fn try_extract_inner_type(
    ty: &Type,
    inner_of: &str,
    skip_over: &HashSet<&str>,
) -> (Type, bool) {
    if let Type::Path(p) = &ty {
        let type_segment = p.path.segments.last().unwrap();
        if type_segment.ident == inner_of {
            let leaf_type = if let PathArguments::AngleBracketed(p) = &type_segment.arguments {
                if let GenericArgument::Type(t) = p.args.first().unwrap().clone() {
                    t
                } else {
                    panic!("Argument in angle brackets must be a type")
                }
            } else {
                panic!("Expected angle bracketed path");
            };

            (leaf_type, true)
        } else if skip_over.contains(type_segment.ident.to_string().as_str()) {
            if let PathArguments::AngleBracketed(p) = &type_segment.arguments {
                if let GenericArgument::Type(t) = p.args.first().unwrap().clone() {
                    try_extract_inner_type(&t, inner_of, skip_over)
                } else {
                    panic!("Argument in angle brackets must be a type")
                }
            } else {
                panic!("Expected angle bracketed path");
            }
        } else {
            (ty.clone(), false)
        }
    } else {
        (ty.clone(), false)
    }
}

/// Filters a type by removing specified container types.
pub fn filter_inner_type(ty: &Type, skip_over: &HashSet<&str>) -> Type {
    if let Type::Path(p) = &ty {
        let type_segment = p.path.segments.last().unwrap();
        if skip_over.contains(type_segment.ident.to_string().as_str()) {
            if let PathArguments::AngleBracketed(p) = &type_segment.arguments {
                if let GenericArgument::Type(t) = p.args.first().unwrap().clone() {
                    filter_inner_type(&t, skip_over)
                } else {
                    panic!("Argument in angle brackets must be a type")
                }
            } else {
                panic!("Expected angle bracketed path");
            }
        } else {
            ty.clone()
        }
    } else {
        ty.clone()
    }
}

/// Wraps a leaf type in a Box if it's not already wrapped in specified container types.
pub fn wrap_leaf_type(ty: &Type, skip_over: &HashSet<&str>) -> Type {
    let mut ty = ty.clone();
    if let Type::Path(p) = &mut ty {
        let type_segment = p.path.segments.last_mut().unwrap();
        if skip_over.contains(type_segment.ident.to_string().as_str()) {
            if let PathArguments::AngleBracketed(args) = &mut type_segment.arguments {
                for a in args.args.iter_mut() {
                    if let syn::GenericArgument::Type(t) = a {
                        *t = wrap_leaf_type(t, skip_over);
                    }
                }

                ty
            } else {
                panic!("Expected angle bracketed path");
            }
        } else {
            parse_quote!(adze::WithLeaf<#ty>)
        }
    } else {
        parse_quote!(adze::WithLeaf<#ty>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use syn::parse_quote;

    #[test]
    fn test_try_extract_inner_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

        let ty: Type = parse_quote!(Vec<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "String");

        let ty: Type = parse_quote!(Option<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(!extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "Option < String >");

        let ty: Type = parse_quote!(Box<Vec<String>>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "String");
    }

    #[test]
    fn test_filter_inner_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

        let ty: Type = parse_quote!(Box<String>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");

        let ty: Type = parse_quote!(Box<Arc<String>>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");

        let ty: Type = parse_quote!(String);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");
    }

    #[test]
    fn test_wrap_leaf_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);

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

        let ty: Type = parse_quote!(Option<Vec<String>>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "Option < Vec < adze :: WithLeaf < String > > >"
        );
    }
}
