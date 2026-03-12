//! Shared syntax helpers for parsing macro/tool attributes.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

pub use adze_common_type_ops_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};

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
