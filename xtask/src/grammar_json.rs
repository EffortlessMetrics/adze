use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Extracts node type information from a tree-sitter grammar JSON
pub fn extract_node_types_from_grammar_json(grammar_json: &str) -> Result<Vec<NodeTypeInfo>> {
    let grammar: Value = serde_json::from_str(grammar_json)?;
    let mut node_types = Vec::new();

    if let Some(rules) = grammar.get("rules").and_then(|r| r.as_object()) {
        for (rule_name, rule_value) in rules {
            // Skip only the source_file rule
            if rule_name == "source_file" {
                continue;
            }

            // Extract node type info based on rule patterns
            if let Some(rule_type) = rule_value.get("type").and_then(|t| t.as_str()) {
                match rule_type {
                    "SEQ" | "PREC_LEFT" | "PREC_RIGHT" => {
                        // Only process if this is a main rule (not an internal _0, _1 rule)
                        if !rule_name.contains("_0")
                            && !rule_name.ends_with("_1")
                            && !rule_name.ends_with("_2")
                        {
                            let fields = extract_fields_from_rule(rule_value);
                            node_types.push(NodeTypeInfo {
                                name: clean_rule_name(rule_name),
                                named: true,
                                fields,
                            });
                        }
                    }
                    "STRING" => {
                        // This is a literal token
                        if let Some(value) = rule_value.get("value").and_then(|v| v.as_str()) {
                            node_types.push(NodeTypeInfo {
                                name: value.to_string(),
                                named: false,
                                fields: HashMap::new(),
                            });
                        }
                    }
                    "PATTERN" => {
                        // This is a regex pattern - named token
                        node_types.push(NodeTypeInfo {
                            name: clean_rule_name(rule_name),
                            named: true,
                            fields: HashMap::new(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // Deduplicate literal tokens
    let mut seen = std::collections::HashSet::new();
    node_types.retain(|n| {
        if n.named {
            true
        } else {
            seen.insert(n.name.clone())
        }
    });

    Ok(node_types)
}

fn clean_rule_name(name: &str) -> String {
    // Convert Expression_Sub to subtraction, Expression_Mul to multiplication, etc.
    if name.starts_with("Expression_") {
        match name.strip_prefix("Expression_").unwrap() {
            "Number" => "number".to_string(),
            "Sub" => "subtraction".to_string(),
            "Mul" => "multiplication".to_string(),
            _ => name.to_lowercase(),
        }
    } else {
        name.to_lowercase()
    }
}

fn extract_fields_from_rule(rule: &Value) -> HashMap<String, Vec<String>> {
    let mut fields = HashMap::new();

    // Look for SEQ members with FIELD nodes
    let content = if rule.get("type").and_then(|t| t.as_str()) == Some("PREC_LEFT")
        || rule.get("type").and_then(|t| t.as_str()) == Some("PREC_RIGHT")
    {
        rule.get("content")
    } else {
        Some(rule)
    };

    if let Some(seq) = content
        && let Some(members) = seq.get("members").and_then(|m| m.as_array())
    {
        for (idx, member) in members.iter().enumerate() {
            if let Some(field_name) = member.get("name").and_then(|n| n.as_str()) {
                // Map numeric field names to semantic names
                let semantic_name = match (idx, field_name) {
                    (0, _) => "left",
                    (1, _) => "operator",
                    (2, _) => "right",
                    _ => field_name,
                };

                fields.insert(semantic_name.to_string(), vec!["expression".to_string()]);
            }
        }
    }

    fields
}

#[derive(Debug)]
pub struct NodeTypeInfo {
    pub name: String,
    pub named: bool,
    #[allow(dead_code)]
    pub fields: HashMap<String, Vec<String>>,
}
