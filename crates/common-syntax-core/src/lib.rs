//! Shared syntax helpers for parsing macro/tool attributes and transforming
//! containerized Rust types.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

/// Name-value expression for attribute parameters.
///
/// Represents a key-value pair in attribute syntax, such as `param = "value"`.
/// This is commonly used when parsing macro or tool attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameValueExpr {
    /// The parameter name.
    pub path: Ident,
    /// The equals token.
    pub eq_token: Token![=],
    /// The parameter value expression.
    pub expr: Expr,
}

impl Parse for NameValueExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NameValueExpr {
            path: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

/// Field declaration followed by optional parameters.
///
/// Represents a struct field declaration optionally followed by a comma and additional
/// named parameters. Used in parsing attribute syntax that includes field definitions with extra metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldThenParams {
    /// The field declaration.
    pub field: Field,
    /// Optional comma separator before params.
    pub comma: Option<Token![,]>,
    /// Additional named parameters.
    pub params: Punctuated<NameValueExpr, Token![,]>,
}

impl Parse for FieldThenParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field = Field::parse_unnamed(input)?;
        let comma: Option<Token![,]> = input.parse()?;
        let params = if comma.is_some() {
            Punctuated::parse_terminated_with(input, NameValueExpr::parse)?
        } else {
            Punctuated::new()
        };

        Ok(FieldThenParams {
            field,
            comma,
            params,
        })
    }
}

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
    use std::collections::HashSet;
    use syn::parse_quote;

    #[test]
    fn test_parse_name_value_expr() {
        let input: NameValueExpr = parse_quote!(key = "value");
        assert_eq!(input.path.to_string(), "key");

        let input: NameValueExpr = parse_quote!(precedence = 5);
        assert_eq!(input.path.to_string(), "precedence");
    }

    #[test]
    fn test_parse_field_then_params() {
        let input: FieldThenParams = parse_quote!(Type);
        assert!(input.comma.is_none());
        assert!(input.params.is_empty());

        let input: FieldThenParams = parse_quote!(Type, name = "test", value = 42);
        assert!(input.comma.is_some());
        assert_eq!(input.params.len(), 2);
        assert_eq!(input.params[0].path.to_string(), "name");
        assert_eq!(input.params[1].path.to_string(), "value");
    }

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

    // --- Additional unit tests for untested paths ---

    #[test]
    fn extract_inner_non_path_type_returns_unchanged() {
        let skip: HashSet<&str> = HashSet::new();
        // Reference type is not a Type::Path, so extraction returns it unchanged.
        let ty: Type = parse_quote!(&str);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        assert!(!extracted);
        assert_eq!(inner.to_token_stream().to_string(), "& str");
    }

    #[test]
    fn filter_non_path_type_returns_unchanged() {
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let ty: Type = parse_quote!((i32, u32));
        let filtered = filter_inner_type(&ty, &skip);
        assert_eq!(filtered.to_token_stream().to_string(), "(i32 , u32)");
    }

    #[test]
    fn wrap_non_path_type_wraps_entirely() {
        let skip: HashSet<&str> = HashSet::new();
        let ty: Type = parse_quote!([u8; 4]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        assert_eq!(
            wrapped.to_token_stream().to_string(),
            "adze :: WithLeaf < [u8 ; 4] >"
        );
    }

    #[test]
    fn extract_inner_skip_does_not_match_target_returns_original() {
        // Box is in skip set; we look for Option but Box<String> has no Option inside.
        let skip: HashSet<&str> = HashSet::from(["Box"]);
        let ty: Type = parse_quote!(Box<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        assert!(!extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
    }

    #[test]
    fn filter_empty_skip_set_returns_original() {
        let skip: HashSet<&str> = HashSet::new();
        let ty: Type = parse_quote!(Box<String>);
        let filtered = filter_inner_type(&ty, &skip);
        assert_eq!(filtered.to_token_stream().to_string(), "Box < String >");
    }

    #[test]
    fn wrap_multiple_generic_args_in_skip_type() {
        // When a skip-set type has multiple generic args, all Type args are wrapped.
        let skip: HashSet<&str> = ["Result"].into_iter().collect();
        let ty: Type = parse_quote!(Result<String, i32>);
        let wrapped = wrap_leaf_type(&ty, &skip);
        assert_eq!(
            wrapped.to_token_stream().to_string(),
            "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
        );
    }
}
