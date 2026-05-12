//! Native AdzeDocument JSON projection canaries.

#![cfg(all(test, feature = "pure-rust", feature = "serialization"))]

use adze::document::ADZE_DOCUMENT_JSON_SCHEMA;
use serde_json::Value;

#[test]
fn parse_document_json_has_schema_and_tree_facts() {
    use adze_example::fielded_precedence_typed_cst_contract::grammar;

    let source = "1+2";
    let document = grammar::parse_document(source)
        .expect("generated parse_document helper should return an AdzeDocument");
    let json = document.to_json_value();

    assert_eq!(json["schema"].as_str(), Some(ADZE_DOCUMENT_JSON_SCHEMA));
    assert_eq!(
        json["language"]["name"].as_str(),
        Some("fielded_precedence_typed_cst_contract")
    );
    assert_eq!(
        json["source"]["byte_len"].as_u64(),
        Some(source.len() as u64)
    );
    assert_eq!(json["metadata"]["error_count"].as_u64(), Some(0));
    assert!(json["diagnostics"].as_array().is_some_and(Vec::is_empty));
    assert!(json["ambiguities"].as_array().is_some_and(Vec::is_empty));

    let root = &json["tree"]["root"];
    assert_eq!(root["id"].as_u64(), Some(0));
    assert_eq!(root["kind"].as_str(), Some("source_file"));
    assert_eq!(root["grammar_kind"].as_str(), Some("source_file"));
    assert_eq!(root["has_alias"].as_bool(), Some(false));
    assert_eq!(root["flags"]["has_error"].as_bool(), Some(false));

    let add = find_json_node(root, "Expr_Add", 0, source.len())
        .expect("document JSON should contain the fielded add expression");
    let children = add["children"]
        .as_array()
        .expect("fielded expression should serialize child edges");
    let fields = children
        .iter()
        .filter_map(|edge| edge["field_name"].as_str())
        .collect::<Vec<_>>();

    assert_eq!(fields, vec!["left", "operator", "right"]);
    assert!(
        children
            .iter()
            .all(|edge| edge["field_id"].as_u64().is_some()),
        "fielded edges should serialize public field IDs: {children:?}"
    );
    assert!(
        children
            .iter()
            .all(|edge| edge["node"]["id"].as_u64().is_some()),
        "fielded edges should serialize nested child nodes: {children:?}"
    );

    insta::assert_snapshot!(
        "adze_document_json_fielded_precedence",
        serde_json::to_string_pretty(&json).expect("document JSON should render as pretty JSON")
    );
}

#[test]
fn parse_document_json_serializes_diagnostics_and_error_flags() {
    use adze_example::typed_ast_contract::grammar;

    let source = "1 +";
    let document = grammar::parse_document(source)
        .expect("generated parse_document helper should return partial parse facts");
    let json = document.to_json_value();

    assert_eq!(json["schema"].as_str(), Some(ADZE_DOCUMENT_JSON_SCHEMA));
    assert!(
        json["metadata"]["error_count"].as_u64().unwrap_or(0) > 0,
        "diagnostic document JSON should preserve parser error count: {json:?}"
    );
    assert_eq!(
        json["tree"]["root"]["flags"]["has_error"].as_bool(),
        Some(true)
    );

    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostic document should serialize diagnostics");
    let diagnostic = diagnostics
        .first()
        .expect("truncated expression should produce a serialized diagnostic");

    assert_eq!(diagnostic["start_byte"].as_u64(), Some(3));
    assert_eq!(diagnostic["end_byte"].as_u64(), Some(3));
    assert_eq!(diagnostic["point_range"]["start"]["row"].as_u64(), Some(0));
    assert_eq!(
        diagnostic["point_range"]["start"]["column"].as_u64(),
        Some(3)
    );
    assert!(
        diagnostic["expected"]
            .as_array()
            .expect("diagnostic should serialize expected tokens")
            .iter()
            .any(|value| value.as_str() == Some(r"/\d+/")),
        "diagnostic JSON should preserve generated expected-token names: {diagnostic:?}"
    );
    assert!(
        diagnostic["related_nodes"]
            .as_array()
            .expect("diagnostic should serialize related document nodes")
            .iter()
            .any(|value| value.as_u64().is_some()),
        "diagnostic JSON should preserve related node IDs: {diagnostic:?}"
    );
    assert!(
        diagnostic["message"]
            .as_str()
            .is_some_and(|message| message.contains("expected one of:")),
        "diagnostic JSON should preserve the diagnostic summary: {diagnostic:?}"
    );

    insta::assert_snapshot!(
        "adze_document_json_diagnostic",
        serde_json::to_string_pretty(&json)
            .expect("diagnostic document JSON should render as pretty JSON")
    );
}

#[test]
fn parse_document_json_serializes_multibyte_diagnostic_span() {
    use adze_example::typed_ast_contract::grammar;

    let source = "1 + \u{03bb}";
    let document = grammar::parse_document(source)
        .expect("generated parse_document helper should return multibyte partial parse facts");
    let json = document.to_json_value();

    assert_eq!(json["schema"].as_str(), Some(ADZE_DOCUMENT_JSON_SCHEMA));
    assert_eq!(
        json["source"]["byte_len"].as_u64(),
        Some(source.len() as u64)
    );
    assert_eq!(
        json["tree"]["root"]["flags"]["has_error"].as_bool(),
        Some(true)
    );

    let diagnostic = json["diagnostics"]
        .as_array()
        .and_then(|diagnostics| diagnostics.first())
        .expect("multibyte bad token should serialize a diagnostic");
    assert_eq!(diagnostic["start_byte"].as_u64(), Some(4));
    assert_eq!(diagnostic["end_byte"].as_u64(), Some(6));
    assert_eq!(diagnostic["point_range"]["start"]["row"].as_u64(), Some(0));
    assert_eq!(
        diagnostic["point_range"]["start"]["column"].as_u64(),
        Some(4)
    );
    assert_eq!(diagnostic["point_range"]["end"]["row"].as_u64(), Some(0));
    assert_eq!(diagnostic["point_range"]["end"]["column"].as_u64(), Some(6));
    assert!(
        diagnostic["expected"]
            .as_array()
            .expect("diagnostic should serialize expected tokens")
            .iter()
            .any(|value| value.as_str() == Some(r"/\d+/")),
        "multibyte diagnostic JSON should preserve expected-token names: {diagnostic:?}"
    );

    let snapshot_json = serde_json::json!({
        "schema": json["schema"].clone(),
        "source": json["source"].clone(),
        "language": json["language"].clone(),
        "root_has_error": json["tree"]["root"]["flags"]["has_error"].clone(),
        "first_diagnostic": {
            "start_byte": diagnostic["start_byte"].clone(),
            "end_byte": diagnostic["end_byte"].clone(),
            "point_range": diagnostic["point_range"].clone(),
            "expected": diagnostic["expected"].clone(),
            "related_nodes": diagnostic["related_nodes"].clone(),
        },
    });

    insta::assert_snapshot!(
        "adze_document_json_multibyte_diagnostic",
        serde_json::to_string_pretty(&snapshot_json)
            .expect("multibyte diagnostic JSON summary should render as pretty JSON")
    );
}

#[test]
fn parse_document_json_serializes_multiline_diagnostic_point_range() {
    use adze_example::typed_ast_contract::grammar;

    let source = "1 +\n@";
    let document = grammar::parse_document(source)
        .expect("generated parse_document helper should return multiline partial parse facts");
    let json = document.to_json_value();

    assert_eq!(json["schema"].as_str(), Some(ADZE_DOCUMENT_JSON_SCHEMA));
    assert_eq!(
        json["source"]["byte_len"].as_u64(),
        Some(source.len() as u64)
    );
    assert_eq!(
        json["tree"]["root"]["flags"]["has_error"].as_bool(),
        Some(true)
    );

    let diagnostic = json["diagnostics"]
        .as_array()
        .and_then(|diagnostics| diagnostics.first())
        .expect("multiline bad token should serialize a diagnostic");
    assert_eq!(diagnostic["start_byte"].as_u64(), Some(4));
    assert_eq!(diagnostic["end_byte"].as_u64(), Some(5));
    assert_eq!(diagnostic["point_range"]["start"]["row"].as_u64(), Some(1));
    assert_eq!(
        diagnostic["point_range"]["start"]["column"].as_u64(),
        Some(0)
    );
    assert_eq!(diagnostic["point_range"]["end"]["row"].as_u64(), Some(1));
    assert_eq!(diagnostic["point_range"]["end"]["column"].as_u64(), Some(1));
    assert!(
        diagnostic["expected"]
            .as_array()
            .expect("diagnostic should serialize expected tokens")
            .iter()
            .any(|value| value.as_str() == Some(r"/\d+/")),
        "multiline diagnostic JSON should preserve expected-token names: {diagnostic:?}"
    );

    let snapshot_json = serde_json::json!({
        "schema": json["schema"].clone(),
        "source": json["source"].clone(),
        "language": json["language"].clone(),
        "root_has_error": json["tree"]["root"]["flags"]["has_error"].clone(),
        "first_diagnostic": {
            "start_byte": diagnostic["start_byte"].clone(),
            "end_byte": diagnostic["end_byte"].clone(),
            "point_range": diagnostic["point_range"].clone(),
            "expected": diagnostic["expected"].clone(),
            "related_nodes": diagnostic["related_nodes"].clone(),
        },
    });

    insta::assert_snapshot!(
        "adze_document_json_multiline_diagnostic",
        serde_json::to_string_pretty(&snapshot_json)
            .expect("multiline diagnostic JSON summary should render as pretty JSON")
    );
}

#[test]
#[cfg(feature = "glr")]
fn parse_document_json_serializes_glr_ambiguity_summary() {
    use adze_example::ambiguous_expr::grammar;

    let source = "1 + 2 + 3";
    let document = grammar::parse_document(source)
        .expect("generated parse_document helper should return an ambiguous AdzeDocument");
    let json = document.to_json_value();

    assert_eq!(json["schema"].as_str(), Some(ADZE_DOCUMENT_JSON_SCHEMA));
    assert!(json["diagnostics"].as_array().is_some_and(Vec::is_empty));
    assert_eq!(
        json["tree"]["root"]["flags"]["has_error"].as_bool(),
        Some(false)
    );

    let ambiguities = json["ambiguities"]
        .as_array()
        .expect("document JSON should serialize ambiguity summaries");
    assert_eq!(
        ambiguities.len(),
        1,
        "ambiguous expression should serialize exactly one summary: {ambiguities:?}"
    );

    let ambiguity = &ambiguities[0];
    assert_eq!(ambiguity["span"]["start_byte"].as_u64(), Some(0));
    assert_eq!(
        ambiguity["span"]["end_byte"].as_u64(),
        Some(source.len() as u64)
    );
    assert_eq!(
        ambiguity["selection_reason"].as_str(),
        Some("StableStructuralTieBreak")
    );

    let selected = ambiguity["selected"]
        .as_u64()
        .expect("ambiguity JSON should identify the selected alternative");
    let alternatives = ambiguity["alternatives"]
        .as_array()
        .expect("ambiguity JSON should serialize retained alternatives");
    assert!(
        alternatives.len() >= 2,
        "ambiguity JSON should retain multiple alternatives: {alternatives:?}"
    );

    let selected_alternative = alternatives
        .iter()
        .find(|alternative| alternative["index"].as_u64() == Some(selected))
        .expect("selected ambiguity alternative should be present in JSON");
    assert_eq!(selected_alternative["span"]["start_byte"].as_u64(), Some(0));
    assert_eq!(
        selected_alternative["span"]["end_byte"].as_u64(),
        Some(source.len() as u64)
    );
    assert!(selected_alternative["root_symbol"].as_u64().is_some());
    assert_eq!(selected_alternative["in_error"].as_bool(), Some(false));
    assert!(
        selected_alternative["node_count"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "selected alternative should preserve a structural node count: {selected_alternative:?}"
    );

    let mut snapshot_json = json;
    snapshot_json["ambiguities"][0]["selected"] = Value::String("<selected>".to_string());

    insta::assert_snapshot!(
        "adze_document_json_ambiguity",
        serde_json::to_string_pretty(&snapshot_json)
            .expect("ambiguous document JSON should render as pretty JSON")
    );
}

fn find_json_node<'a>(
    node: &'a Value,
    kind: &str,
    start_byte: usize,
    end_byte: usize,
) -> Option<&'a Value> {
    if node["kind"].as_str() == Some(kind)
        && node["range"]["start_byte"].as_u64() == Some(start_byte as u64)
        && node["range"]["end_byte"].as_u64() == Some(end_byte as u64)
    {
        return Some(node);
    }

    for edge in node["children"].as_array()? {
        if let Some(found) = find_json_node(&edge["node"], kind, start_byte, end_byte) {
            return Some(found);
        }
    }

    None
}
