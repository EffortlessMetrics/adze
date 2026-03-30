//! Shared syntax helpers for parsing macro/tool attributes.

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
            match &type_segment.arguments {
                PathArguments::AngleBracketed(p) => {
                    if let GenericArgument::Type(t) = p.args.first().unwrap().clone() {
                        (t, true)
                    } else {
                        panic!("Argument in angle brackets must be a type")
                    }
                }
                _ => (ty.clone(), false),
            }
        } else if skip_over.contains(type_segment.ident.to_string().as_str()) {
            match &type_segment.arguments {
                PathArguments::AngleBracketed(p) => {
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
                }
                _ => (ty.clone(), false),
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
            match &type_segment.arguments {
                PathArguments::AngleBracketed(p) => {
                    if let GenericArgument::Type(t) = p.args.first().unwrap().clone() {
                        filter_inner_type(&t, skip_over)
                    } else {
                        panic!("Argument in angle brackets must be a type")
                    }
                }
                _ => ty.clone(),
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
            match &mut type_segment.arguments {
                PathArguments::AngleBracketed(args) => {
                    for a in args.args.iter_mut() {
                        if let syn::GenericArgument::Type(t) = a {
                            *t = wrap_leaf_type(t, skip_over);
                        }
                    }

                    ty
                }
                _ => ty,
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
}
