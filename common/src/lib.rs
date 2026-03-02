// Common crate is pure-Rust - no unsafe needed
#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

//! Shared utilities for adze macro and tool crates.

pub use adze_macro_type_utils_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

/// Name-value expression for attribute parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameValueExpr {
    /// The parameter name
    pub path: Ident,
    /// The equals token
    pub eq_token: Token![=],
    /// The parameter value expression
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

/// Field followed by optional parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldThenParams {
    /// The field declaration
    pub field: Field,
    /// Optional comma separator
    pub comma: Option<Token![,]>,
    /// Additional parameters
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
