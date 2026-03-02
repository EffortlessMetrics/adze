use adze_common::{FieldThenParams, NameValueExpr};
use syn::{Expr, parse_quote};

#[test]
fn test_name_value_expr_parsing() {
    let expr: NameValueExpr = parse_quote!(foo = 42);
    assert_eq!(expr.path.to_string(), "foo");
    assert!(matches!(expr.expr, Expr::Lit(_)));

    let expr: NameValueExpr = parse_quote!(name = "value");
    assert_eq!(expr.path.to_string(), "name");
    assert!(matches!(expr.expr, Expr::Lit(_)));

    let expr: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(expr.path.to_string(), "transform");
    assert!(matches!(expr.expr, Expr::Closure(_)));
}

#[test]
fn test_field_then_params_parsing() {
    let parsed: FieldThenParams = parse_quote!(String);
    assert!(parsed.comma.is_none());
    assert_eq!(parsed.params.len(), 0);

    let parsed: FieldThenParams = parse_quote!(String, pattern = r"\d+");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");

    let parsed: FieldThenParams = parse_quote!(Vec<String>, min = 1, max = 10);
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "min");
    assert_eq!(parsed.params[1].path.to_string(), "max");
}
