// Common crate is pure-Rust - no unsafe needed
#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

//! Shared utilities for the `adze-macro` and `adze-tool` crates.
//!
//! This crate provides parsing helpers and type-manipulation functions used by
//! both the proc-macro frontend (`adze-macro`) and the build-time code generator
//! (`adze-tool`). It is not intended for direct use by end users.
//!
//! # Contents
//!
//! - [`NameValueExpr`] — Parses `key = value` attribute parameters.
//! - [`FieldThenParams`] — Parses a field declaration followed by optional
//!   comma-separated parameters.
//! - [`try_extract_inner_type`] — Unwraps a container type (e.g. `Vec<T>` → `T`),
//!   optionally skipping through wrapper types.
//! - [`filter_inner_type`] — Strips specified wrapper types from a type.
//! - [`wrap_leaf_type`] — Wraps leaf types in `adze::WithLeaf<T>`, preserving
//!   container types.

use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

/// A name-value expression of the form `key = expr`, used to represent
/// individual attribute parameters such as `precedence = 5` or
/// `pattern = r"\d+"`.
///
/// Implements [`syn::parse::Parse`] so it can be used with
/// [`syn::parse_quote!`] or parsed from a [`syn::parse::ParseStream`].
///
/// # Example
///
/// ```
/// use adze_common::NameValueExpr;
/// use syn::parse_quote;
///
/// let nv: NameValueExpr = parse_quote!(precedence = 5);
/// assert_eq!(nv.path.to_string(), "precedence");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameValueExpr {
    /// The parameter name (left-hand side of `=`).
    pub path: Ident,
    /// The `=` token.
    pub eq_token: Token![=],
    /// The value expression (right-hand side of `=`).
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

/// A parsed field declaration optionally followed by comma-separated
/// [`NameValueExpr`] parameters.
///
/// This is the main parsing structure for adze attribute arguments like
/// `#[adze::leaf(String, pattern = r"\d+")]` where `String` is the field
/// type and `pattern = r"\d+"` is an additional parameter.
///
/// # Example
///
/// ```
/// use adze_common::FieldThenParams;
/// use syn::parse_quote;
///
/// let ftp: FieldThenParams = parse_quote!(Vec<String>, min = 1, max = 10);
/// assert!(ftp.comma.is_some());
/// assert_eq!(ftp.params.len(), 2);
/// assert_eq!(ftp.params[0].path.to_string(), "min");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldThenParams {
    /// The field declaration (an unnamed field parsed via [`Field::parse_unnamed`]).
    pub field: Field,
    /// Optional comma separating the field from additional parameters.
    pub comma: Option<Token![,]>,
    /// Zero or more `key = value` parameters.
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

/// Attempts to extract the inner type from a container type named `inner_of`,
/// optionally looking through wrapper types listed in `skip_over`.
///
/// Returns a tuple of `(inner_type, true)` when the target container is found,
/// or `(original_type, false)` when it is not.
///
/// # Example
///
/// ```
/// use adze_common::try_extract_inner_type;
/// use std::collections::HashSet;
/// use syn::{Type, parse_quote};
///
/// let skip = HashSet::from(["Box"]);
///
/// // Direct extraction: Vec<String> → String
/// let ty: Type = parse_quote!(Vec<String>);
/// let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
/// assert!(found);
/// assert_eq!(quote::quote!(#inner).to_string(), "String");
///
/// // Skips through Box: Box<Vec<i32>> → i32
/// let ty: Type = parse_quote!(Box<Vec<i32>>);
/// let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
/// assert!(found);
/// assert_eq!(quote::quote!(#inner).to_string(), "i32");
///
/// // No match: returns original type unchanged
/// let ty: Type = parse_quote!(String);
/// let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
/// assert!(!found);
/// ```
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

/// Recursively strips wrapper types listed in `skip_over` from `ty`,
/// returning the innermost non-wrapper type.
///
/// For example, `Box<Arc<String>>` with `skip_over = {"Box", "Arc"}` yields
/// `String`. If `ty` is not in `skip_over`, it is returned unchanged.
///
/// # Example
///
/// ```
/// use adze_common::filter_inner_type;
/// use std::collections::HashSet;
/// use syn::{Type, parse_quote};
///
/// let skip = HashSet::from(["Box", "Arc"]);
///
/// let ty: Type = parse_quote!(Box<Arc<String>>);
/// let inner = filter_inner_type(&ty, &skip);
/// assert_eq!(quote::quote!(#inner).to_string(), "String");
///
/// let ty: Type = parse_quote!(String);
/// let inner = filter_inner_type(&ty, &skip);
/// assert_eq!(quote::quote!(#inner).to_string(), "String");
/// ```
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

/// Wraps the leaf (innermost) type in `adze::WithLeaf<T>`, preserving any
/// container types listed in `skip_over`.
///
/// Container types in `skip_over` are kept as-is, while the innermost type
/// that is *not* in `skip_over` gets wrapped. For example,
/// `Option<Vec<String>>` with `skip_over = {"Option", "Vec"}` becomes
/// `Option<Vec<adze::WithLeaf<String>>>`.
///
/// # Example
///
/// ```
/// use adze_common::wrap_leaf_type;
/// use std::collections::HashSet;
/// use syn::{Type, parse_quote};
///
/// let skip = HashSet::from(["Vec"]);
///
/// let ty: Type = parse_quote!(Vec<String>);
/// let wrapped = wrap_leaf_type(&ty, &skip);
/// assert_eq!(
///     quote::quote!(#wrapped).to_string(),
///     "Vec < adze :: WithLeaf < String > >"
/// );
///
/// let ty: Type = parse_quote!(i32);
/// let wrapped = wrap_leaf_type(&ty, &skip);
/// assert_eq!(
///     quote::quote!(#wrapped).to_string(),
///     "adze :: WithLeaf < i32 >"
/// );
/// ```
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

        // Test extracting from Vec
        let ty: Type = parse_quote!(Vec<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "String");

        // Test not extracting when type doesn't match
        let ty: Type = parse_quote!(Option<String>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(!extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "Option < String >");

        // Test skipping over Box
        let ty: Type = parse_quote!(Box<Vec<String>>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
        assert!(extracted);
        assert_eq!(quote::quote!(#inner).to_string(), "String");
    }

    #[test]
    fn test_filter_inner_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

        // Test filtering Box
        let ty: Type = parse_quote!(Box<String>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");

        // Test filtering nested
        let ty: Type = parse_quote!(Box<Arc<String>>);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");

        // Test no filtering needed
        let ty: Type = parse_quote!(String);
        let filtered = filter_inner_type(&ty, &skip_over);
        assert_eq!(quote::quote!(#filtered).to_string(), "String");
    }

    #[test]
    fn test_wrap_leaf_type() {
        let skip_over: HashSet<&str> = HashSet::from(["Vec", "Option"]);

        // Test wrapping simple type
        let ty: Type = parse_quote!(String);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "adze :: WithLeaf < String >"
        );

        // Test skipping over Vec
        let ty: Type = parse_quote!(Vec<String>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "Vec < adze :: WithLeaf < String > >"
        );

        // Test nested skipping
        let ty: Type = parse_quote!(Option<Vec<String>>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "Option < Vec < adze :: WithLeaf < String > > >"
        );
    }
}
