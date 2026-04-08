use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, FieldThenParams, NameValueExpr};
use adze_common_syntax_core::{
    filter_inner_type as core_filter_inner_type, try_extract_inner_type as core_try_extract_inner_type,
};
use quote::ToTokens;
use syn::{parse_quote, Type};

#[test]
fn facade_and_core_exports_emit_identical_filter_behavior() {
    let mut skip_over: HashSet<&str> = HashSet::from(["Box"]);
    skip_over.insert("Arc");
    let ty: Type = parse_quote!(Box<Arc<Option<String>>>);

    let facade = filter_inner_type(&ty, &skip_over);
    let direct = core_filter_inner_type(&ty, &skip_over);

    assert_eq!(
        facade.to_token_stream().to_string(),
        direct.to_token_stream().to_string()
    );
}

#[test]
fn facade_and_core_exports_emit_identical_extraction_behavior() {
    let mut skip_over: HashSet<&str> = HashSet::new();
    skip_over.insert("Option");
    let ty: Type = parse_quote!(Vec<Option<String>>);

    let (facade_inner, facade_extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    let (core_inner, core_extracted) = core_try_extract_inner_type(&ty, "Option", &skip_over);

    assert_eq!(facade_extracted, core_extracted);
    assert_eq!(
        facade_inner.to_token_stream().to_string(),
        core_inner.to_token_stream().to_string()
    );
}

#[test]
fn public_syntax_exports_still_parse() {
    let _expr: NameValueExpr = parse_quote!(key = 1);
    let _params: FieldThenParams = parse_quote!(Vec<String>, min = 2);
}
