//! NODE_TYPES JSON generation for static language output.

use adze_ir::{Grammar, TokenPattern};
use serde_json::json;

pub(crate) fn generate(grammar: &Grammar) -> String {
    let mut types = Vec::new();

    // Generate node types for non-terminal rules
    for (symbol_id, rules) in &grammar.rules {
        let rule_name = grammar
            .rule_names
            .get(symbol_id)
            .cloned()
            .unwrap_or_else(|| format!("rule_{}", symbol_id.0));

        // Skip hidden rules (those starting with underscore)
        if rule_name.starts_with('_') {
            continue;
        }

        let mut node_type = json!({
            "type": rule_name,
            "named": true
        });

        // Collect fields from all rules for this symbol
        let mut all_fields = serde_json::Map::new();
        let mut has_children = false;

        for rule in rules {
            // Add fields if this rule has any
            for (field_id, _position) in &rule.fields {
                if let Some(field_name) = grammar.fields.get(field_id) {
                    all_fields.insert(
                        field_name.clone(),
                        json!({
                            "multiple": false,
                            "required": true,
                            "types": []
                        }),
                    );
                }
            }

            // Check if rule has children
            if !rule.rhs.is_empty() {
                has_children = true;
            }
        }

        // Add fields if any
        if !all_fields.is_empty() {
            node_type["fields"] = json!(all_fields);
        }

        // Add children if any rule has RHS
        if has_children {
            let mut children = serde_json::Map::new();
            children.insert("multiple".to_string(), json!(false));
            children.insert("required".to_string(), json!(true));
            // TODO: Add proper child types based on rule.rhs
            children.insert("types".to_string(), json!([]));
            node_type["children"] = json!(children);
        }

        // Check if this is a supertype
        if grammar.supertypes.contains(symbol_id) {
            node_type["subtypes"] = json!([]);
        }

        types.push(node_type);
    }

    // Generate node types for named tokens
    for token in grammar.tokens.values() {
        if !token.name.starts_with('_') && matches!(&token.pattern, TokenPattern::Regex(_)) {
            types.push(json!({
                "type": token.name,
                "named": true
            }));
        }
    }

    // Generate node types for external tokens
    for external in &grammar.externals {
        if !external.name.starts_with('_') {
            types.push(json!({
                "type": external.name,
                "named": true
            }));
        }
    }

    serde_json::to_string_pretty(&json!(types)).unwrap_or_else(|_| "[]".to_string())
}
