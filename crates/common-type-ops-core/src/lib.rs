//! Single-responsibility helpers for transforming containerized Rust types.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::collections::HashSet;

use syn::{GenericArgument, PathArguments, Type, parse_quote};

/// Extract the innermost generic argument from a container type.
///
/// # Arguments
/// * `ty` - The type to extract from
/// * `inner_of` - The target generic type to extract (e.g., "Vec", "Option")
/// * `skip_over` - Set of container types to skip through (e.g., "Box", "Arc")
///
/// # Returns
/// A tuple `(inner_type, was_extracted)` where `inner_type` is the extracted or original type,
/// and `was_extracted` indicates whether the target type was found and extracted.
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
                    let (inner, extracted) = try_extract_inner_type(&t, inner_of, skip_over);
                    if extracted {
                        (inner, true)
                    } else {
                        (ty.clone(), false)
                    }
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

/// Remove configured container wrappers from a type.
///
/// # Arguments
/// * `ty` - The type to filter
/// * `skip_over` - Set of container types to unwrap (e.g., "Box", "Arc")
///
/// # Returns
/// The type with all specified container wrappers removed. If the type is not a container type
/// in the skip set, returns the original type unchanged.
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

/// Wrap leaf types in `adze::WithLeaf` unless they are in the skip set.
///
/// # Arguments
/// * `ty` - The type to potentially wrap
/// * `skip_over` - Set of container types to skip wrapping (e.g., "Vec", "Option")
///
/// # Returns
/// The type with leaf types wrapped in `adze::WithLeaf`, or the original type if it's
/// a container type in the skip set. For skipped containers, recursively wraps their inner generic arguments.
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
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_try_extract_inner_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

        let ty: Type = parse_quote!(Vec<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "String");

        let ty: Type = parse_quote!(Option<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(!extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Option < String >");

        let ty: Type = parse_quote!(Box<Vec<String>>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "String");
    }

    #[test]
    fn test_filter_inner_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

        let ty: Type = parse_quote!(Box<String>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(filtered.to_token_stream().to_string(), "String");

        let ty: Type = parse_quote!(Box<Arc<String>>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(filtered.to_token_stream().to_string(), "String");
    }

    #[test]
    fn test_wrap_leaf_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);

        let ty: Type = parse_quote!(String);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            wrapped.to_token_stream().to_string(),
            "adze :: WithLeaf < String >"
        );

        let ty: Type = parse_quote!(Vec<String>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            wrapped.to_token_stream().to_string(),
            "Vec < adze :: WithLeaf < String > >"
        );
    }
}
