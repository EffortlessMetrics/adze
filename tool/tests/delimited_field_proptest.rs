#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for delimited field generation in adze-tool.
//!
//! Covers:
//!   1. Delimited Vec with separator
//!   2. Delimiter appears in generated SEQ
//!   3. Delimiter is STRING type
//!   4. Delimited with trailing separator option
//!   5. Non-delimited Vec produces REPEAT
//!   6. Delimited with different separators (comma, semicolon)
//!   7. Delimited field naming
//!   8. Delimited generation determinism

use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn extract_one(src: &str) -> Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

fn rule_names(grammar: &Value) -> Vec<String> {
    grammar["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn get_rule<'a>(grammar: &'a Value, name: &str) -> &'a Value {
    &grammar["rules"][name]
}

fn collect_types(node: &Value) -> Vec<String> {
    let mut types = vec![];
    collect_types_inner(node, &mut types);
    types
}

fn collect_types_inner(node: &Value, acc: &mut Vec<String>) {
    if let Some(t) = node["type"].as_str() {
        acc.push(t.to_string());
    }
    if let Some(members) = node["members"].as_array() {
        for m in members {
            collect_types_inner(m, acc);
        }
    }
    if node.get("content").is_some() && !node["content"].is_null() {
        collect_types_inner(&node["content"], acc);
    }
}

fn find_node_by_type<'a>(node: &'a Value, ty: &str) -> Option<&'a Value> {
    if node["type"].as_str() == Some(ty) {
        return Some(node);
    }
    if let Some(members) = node["members"].as_array() {
        for m in members {
            if let Some(found) = find_node_by_type(m, ty) {
                return Some(found);
            }
        }
    }
    if node.get("content").is_some()
        && !node["content"].is_null()
        && let Some(found) = find_node_by_type(&node["content"], ty)
    {
        return Some(found);
    }
    None
}

// ===========================================================================
// Grammar source fragments
// ===========================================================================

const DELIMITED_COMMA: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct NumberList {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        numbers: Vec<Number>,
    }

    pub struct Number {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: i32,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

const NON_DELIMITED_VEC: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct NumberList {
        #[adze::repeat(non_empty = true)]
        numbers: Vec<Number>,
    }

    pub struct Number {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: i32,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

const DELIMITED_SEMICOLON: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct StmtList {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = ";")]
            ()
        )]
        stmts: Vec<Stmt>,
    }

    pub struct Stmt {
        #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
        name: String,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

const DELIMITED_PIPE: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct Alternatives {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = "|")]
            ()
        )]
        items: Vec<Item>,
    }

    pub struct Item {
        #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
        name: String,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

const DELIMITED_EMPTY_ALLOWED: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct NumberList {
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        numbers: Vec<Number>,
    }

    pub struct Number {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: i32,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

const DELIMITED_ARROW: &str = r#"
#[adze::grammar("test")]
pub mod grammar {
    #[adze::language]
    pub struct Chain {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = "->")]
            ()
        )]
        links: Vec<Link>,
    }

    pub struct Link {
        #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
        name: String,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
"#;

// ===========================================================================
// 1. Delimited Vec with separator
// ===========================================================================

#[test]
fn delimited_vec_produces_seq_with_repeat() {
    let g = extract_one(DELIMITED_COMMA);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    assert_eq!(
        contents["type"].as_str(),
        Some("SEQ"),
        "delimited Vec should wrap in SEQ"
    );
}

#[test]
fn delimited_vec_first_member_is_field() {
    let g = extract_one(DELIMITED_COMMA);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let members = contents["members"].as_array().expect("SEQ needs members");
    assert_eq!(members[0]["type"].as_str(), Some("FIELD"));
}

#[test]
fn delimited_vec_second_member_is_repeat() {
    let g = extract_one(DELIMITED_COMMA);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let members = contents["members"].as_array().expect("SEQ needs members");
    let repeat_type = members[1]["type"].as_str().unwrap();
    assert!(
        repeat_type == "REPEAT" || repeat_type == "REPEAT1",
        "second member should be REPEAT or REPEAT1, got {repeat_type}"
    );
}

// ===========================================================================
// 2. Delimiter appears in generated SEQ
// ===========================================================================

#[test]
fn delimiter_rule_exists_in_grammar() {
    let g = extract_one(DELIMITED_COMMA);
    let names = rule_names(&g);
    assert!(
        names.iter().any(|n| n.contains("delimiter")),
        "expected a delimiter rule, found: {names:?}"
    );
}

#[test]
fn delimiter_appears_inside_repeat_seq() {
    let g = extract_one(DELIMITED_COMMA);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let members = contents["members"].as_array().unwrap();
    let repeat_content = &members[1]["content"];
    assert_eq!(
        repeat_content["type"].as_str(),
        Some("SEQ"),
        "inner repeat should be SEQ"
    );
    let inner = repeat_content["members"].as_array().unwrap();
    assert!(
        inner.len() >= 2,
        "inner SEQ should have delimiter + element"
    );
}

// ===========================================================================
// 3. Delimiter is STRING type
// ===========================================================================

#[test]
fn delimiter_rule_is_string_type() {
    let g = extract_one(DELIMITED_COMMA);
    let delim_rule = get_rule(&g, "NumberList_numbers_vec_delimiter");
    assert_eq!(
        delim_rule["type"].as_str(),
        Some("STRING"),
        "delimiter rule should be STRING"
    );
}

#[test]
fn delimiter_string_value_is_comma() {
    let g = extract_one(DELIMITED_COMMA);
    let delim_rule = get_rule(&g, "NumberList_numbers_vec_delimiter");
    assert_eq!(delim_rule["value"].as_str(), Some(","));
}

#[test]
fn semicolon_delimiter_string_value() {
    let g = extract_one(DELIMITED_SEMICOLON);
    let delim_rule = get_rule(&g, "StmtList_stmts_vec_delimiter");
    assert_eq!(
        delim_rule["type"].as_str(),
        Some("STRING"),
        "semicolon delimiter should be STRING"
    );
    assert_eq!(delim_rule["value"].as_str(), Some(";"));
}

// ===========================================================================
// 4. Delimited with trailing separator option (empty-allowed Vec)
// ===========================================================================

#[test]
fn delimited_empty_allowed_has_choice_wrapper() {
    let g = extract_one(DELIMITED_EMPTY_ALLOWED);
    let root = get_rule(&g, "NumberList");
    let types = collect_types(root);
    assert!(
        types.contains(&"CHOICE".to_string()),
        "empty-allowed delimited Vec should produce CHOICE wrapper"
    );
}

#[test]
fn delimited_empty_allowed_has_blank_branch() {
    let g = extract_one(DELIMITED_EMPTY_ALLOWED);
    let root = get_rule(&g, "NumberList");
    let blank = find_node_by_type(root, "BLANK");
    assert!(
        blank.is_some(),
        "empty-allowed delimited Vec should include BLANK branch"
    );
}

#[test]
fn delimited_non_empty_has_no_blank_at_top() {
    let g = extract_one(DELIMITED_COMMA);
    let root = get_rule(&g, "NumberList");
    // For non_empty = true, the top-level rule should be a direct SYMBOL reference
    let root_type = root["type"].as_str().unwrap_or("");
    // Either it's a SYMBOL or a FIELD — but NOT a CHOICE with BLANK
    if root_type == "CHOICE" {
        let members = root["members"].as_array().unwrap();
        let has_blank = members.iter().any(|m| m["type"].as_str() == Some("BLANK"));
        assert!(
            !has_blank,
            "non_empty delimited Vec should not have BLANK at top level"
        );
    }
}

// ===========================================================================
// 5. Non-delimited Vec produces REPEAT
// ===========================================================================

#[test]
fn non_delimited_vec_produces_repeat1() {
    let g = extract_one(NON_DELIMITED_VEC);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let types = collect_types(contents);
    assert!(
        types.contains(&"REPEAT1".to_string()),
        "non-delimited Vec with non_empty should contain REPEAT1, got {types:?}"
    );
}

#[test]
fn non_delimited_vec_has_no_delimiter_rule() {
    let g = extract_one(NON_DELIMITED_VEC);
    let names = rule_names(&g);
    assert!(
        !names.iter().any(|n| n.contains("delimiter")),
        "non-delimited Vec should not have delimiter rule"
    );
}

#[test]
fn non_delimited_vec_content_is_field() {
    let g = extract_one(NON_DELIMITED_VEC);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let field = find_node_by_type(contents, "FIELD");
    assert!(
        field.is_some(),
        "non-delimited Vec should contain FIELD node"
    );
}

#[test]
fn non_delimited_vec_no_seq_at_top() {
    let g = extract_one(NON_DELIMITED_VEC);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    assert_ne!(
        contents["type"].as_str(),
        Some("SEQ"),
        "non-delimited Vec should not use SEQ at top (no delimiter)"
    );
}

// ===========================================================================
// 6. Delimited with different separators
// ===========================================================================

#[test]
fn pipe_separator_string_value() {
    let g = extract_one(DELIMITED_PIPE);
    let delim_rule = get_rule(&g, "Alternatives_items_vec_delimiter");
    assert_eq!(delim_rule["type"].as_str(), Some("STRING"));
    assert_eq!(delim_rule["value"].as_str(), Some("|"));
}

#[test]
fn arrow_separator_string_value() {
    let g = extract_one(DELIMITED_ARROW);
    let delim_rule = get_rule(&g, "Chain_links_vec_delimiter");
    assert_eq!(delim_rule["type"].as_str(), Some("STRING"));
    assert_eq!(delim_rule["value"].as_str(), Some("->"));
}

#[test]
fn different_separators_share_same_structure() {
    let g_comma = extract_one(DELIMITED_COMMA);
    let g_semi = extract_one(DELIMITED_SEMICOLON);
    let g_pipe = extract_one(DELIMITED_PIPE);

    let c_contents = get_rule(&g_comma, "NumberList_numbers_vec_contents");
    let s_contents = get_rule(&g_semi, "StmtList_stmts_vec_contents");
    let p_contents = get_rule(&g_pipe, "Alternatives_items_vec_contents");

    // All delimited grammars should have SEQ at top
    assert_eq!(c_contents["type"].as_str(), Some("SEQ"));
    assert_eq!(s_contents["type"].as_str(), Some("SEQ"));
    assert_eq!(p_contents["type"].as_str(), Some("SEQ"));
}

#[test]
fn multi_char_separator_arrow() {
    let g = extract_one(DELIMITED_ARROW);
    let delim = get_rule(&g, "Chain_links_vec_delimiter");
    assert_eq!(delim["value"].as_str(), Some("->"));
    assert_eq!(delim["type"].as_str(), Some("STRING"));
}

// ===========================================================================
// 7. Delimited field naming
// ===========================================================================

#[test]
fn delimiter_rule_name_follows_convention() {
    let g = extract_one(DELIMITED_COMMA);
    let names = rule_names(&g);
    assert!(
        names.contains(&"NumberList_numbers_vec_delimiter".to_string()),
        "expected 'NumberList_numbers_vec_delimiter' rule, found: {names:?}"
    );
}

#[test]
fn element_field_name_follows_convention() {
    let g = extract_one(DELIMITED_COMMA);
    let contents = get_rule(&g, "NumberList_numbers_vec_contents");
    let field = find_node_by_type(contents, "FIELD").expect("should have FIELD");
    assert_eq!(
        field["name"].as_str(),
        Some("NumberList_numbers_vec_element"),
        "element field should be named '<Struct>_<field>_vec_element'"
    );
}

#[test]
fn contents_rule_name_follows_convention() {
    let g = extract_one(DELIMITED_COMMA);
    let names = rule_names(&g);
    assert!(
        names.contains(&"NumberList_numbers_vec_contents".to_string()),
        "expected 'NumberList_numbers_vec_contents' rule, found: {names:?}"
    );
}

#[test]
fn semicolon_field_naming() {
    let g = extract_one(DELIMITED_SEMICOLON);
    let names = rule_names(&g);
    assert!(names.contains(&"StmtList_stmts_vec_delimiter".to_string()));
    assert!(names.contains(&"StmtList_stmts_vec_contents".to_string()));
}

#[test]
fn pipe_field_naming() {
    let g = extract_one(DELIMITED_PIPE);
    let names = rule_names(&g);
    assert!(names.contains(&"Alternatives_items_vec_delimiter".to_string()));
    assert!(names.contains(&"Alternatives_items_vec_contents".to_string()));
}

// ===========================================================================
// 8. Delimited generation determinism
// ===========================================================================

#[test]
fn delimited_grammar_deterministic_across_runs() {
    let g1 = extract_one(DELIMITED_COMMA);
    let g2 = extract_one(DELIMITED_COMMA);
    assert_eq!(g1, g2, "same input should produce identical grammar JSON");
}

#[test]
fn non_delimited_grammar_deterministic() {
    let g1 = extract_one(NON_DELIMITED_VEC);
    let g2 = extract_one(NON_DELIMITED_VEC);
    assert_eq!(g1, g2, "non-delimited grammar should be deterministic");
}

#[test]
fn delimited_semicolon_deterministic() {
    let g1 = extract_one(DELIMITED_SEMICOLON);
    let g2 = extract_one(DELIMITED_SEMICOLON);
    assert_eq!(g1, g2);
}

#[test]
fn delimiter_rule_stable_across_calls() {
    let g1 = extract_one(DELIMITED_COMMA);
    let g2 = extract_one(DELIMITED_COMMA);
    let d1 = get_rule(&g1, "NumberList_numbers_vec_delimiter");
    let d2 = get_rule(&g2, "NumberList_numbers_vec_delimiter");
    assert_eq!(d1, d2, "delimiter rule should be stable across calls");
}

#[test]
fn contents_rule_stable_across_calls() {
    let g1 = extract_one(DELIMITED_PIPE);
    let g2 = extract_one(DELIMITED_PIPE);
    let c1 = get_rule(&g1, "Alternatives_items_vec_contents");
    let c2 = get_rule(&g2, "Alternatives_items_vec_contents");
    assert_eq!(c1, c2, "contents rule should be stable across calls");
}
