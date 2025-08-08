// Common crate is pure-Rust - no unsafe needed
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Shared utilities for rust-sitter macro and tool crates

use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameValueExpr {
    pub path: Ident,
    pub eq_token: Token![=],
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldThenParams {
    pub field: Field,
    pub comma: Option<Token![,]>,
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
            parse_quote!(rust_sitter::WithLeaf<#ty>)
        }
    } else {
        parse_quote!(rust_sitter::WithLeaf<#ty>)
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
            "rust_sitter :: WithLeaf < String >"
        );

        // Test skipping over Vec
        let ty: Type = parse_quote!(Vec<String>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "Vec < rust_sitter :: WithLeaf < String > >"
        );

        // Test nested skipping
        let ty: Type = parse_quote!(Option<Vec<String>>);
        let wrapped = wrap_leaf_type(&ty, &skip_over);
        assert_eq!(
            quote::quote!(#wrapped).to_string(),
            "Option < Vec < rust_sitter :: WithLeaf < String > > >"
        );
    }
}
