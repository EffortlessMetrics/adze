use adze_common_syntax_core::{FieldThenParams, NameValueExpr};
use syn::parse_quote;

#[test]
fn given_field_then_params_with_multiple_pairs_when_parsing_then_key_names_round_trip() {
    let input: FieldThenParams = parse_quote!(String, pattern = r"\\d+", max = 4);

    assert_eq!(input.params[0].path.to_string(), "pattern");
    assert_eq!(input.params[1].path.to_string(), "max");
}

#[test]
fn given_name_value_expr_with_transform_when_parsed_then_expr_is_preserved() {
    let expr: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(expr.path.to_string(), "transform");
}
