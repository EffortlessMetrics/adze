#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! NODE_TYPES JSON metadata generation for Tree-sitter grammars.

use adze_ir::{Grammar, Symbol, TokenPattern};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[cfg(not(debug_assertions))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        if std::env::var("RUST_LOG")
            .ok()
            .unwrap_or_default()
            .contains("debug")
        {
            eprintln!($($arg)*);
        }
    };
}

/// Tree-sitter NODE_TYPES.json generator
pub struct NodeTypesGenerator<'a> {
    grammar: &'a Grammar,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeType {
    #[serde(rename = "type")]
    type_name: String,
    named: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<HashMap<String, FieldInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<ChildrenInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtypes: Option<Vec<SubtypeRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FieldInfo {
    multiple: bool,
    required: bool,
    types: Vec<TypeRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChildrenInfo {
    multiple: bool,
    required: bool,
    types: Vec<TypeRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypeRef {
    #[serde(rename = "type")]
    type_name: String,
    named: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubtypeRef {
    #[serde(rename = "type")]
    type_name: String,
    named: bool,
}

impl<'a> NodeTypesGenerator<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        Self { grammar }
    }

    /// Generate NODE_TYPES.json content
    #[must_use = "generation result must be checked"]
    pub fn generate(&self) -> Result<String, String> {
        let mut node_types = Vec::new();
        let mut symbol_names: HashMap<_, _> = HashMap::new();

        debug_trace!(
            "Debug: NodeTypesGenerator - grammar has {} rules",
            self.grammar.rules.len()
        );

        // First, collect all symbol names
        for (symbol_id, _rule) in &self.grammar.rules {
            if let Some(rule_name) = self.get_rule_name(*symbol_id) {
                debug_trace!(
                    "Debug: Adding rule name '{}' for symbol {}",
                    rule_name,
                    symbol_id.0
                );
                symbol_names.insert(*symbol_id, rule_name);
            }
        }

        // Add token names
        for (symbol_id, token) in &self.grammar.tokens {
            symbol_names.insert(*symbol_id, token.name.clone());
        }

        // Process rules to create node types
        let mut processed = HashSet::new();

        debug_trace!(
            "Debug: Processing {} rules for node types",
            self.grammar.rules.len()
        );

        // Find supertypes (rules that have other rules as alternatives)
        let _supertypes: HashMap<adze_ir::SymbolId, Vec<adze_ir::SymbolId>> = HashMap::new();

        // Analyze rule relationships to find choice patterns
        for (symbol_id, rules) in &self.grammar.rules {
            if processed.contains(symbol_id) {
                continue;
            }

            debug_trace!(
                "Debug: Processing symbol {} with {} rules",
                symbol_id.0,
                rules.len()
            );

            // Get the rule name
            if let Some(name) = self.get_rule_name(*symbol_id) {
                // Skip internal rules (starting with _)
                let is_internal = name.starts_with('_');

                // Collect fields from all rules for this symbol
                let mut fields = HashMap::new();
                for rule in rules {
                    for (field_id, position) in &rule.fields {
                        if let Some(field_name) = self.grammar.fields.get(field_id)
                            && let Some(symbol) = rule.rhs.get(*position)
                        {
                            let type_ref = self.symbol_to_type_ref(symbol, &symbol_names);
                            fields.insert(
                                field_name.clone(),
                                FieldInfo {
                                    multiple: false, // TODO: Detect repetition
                                    required: true,  // TODO: Detect optionality
                                    types: vec![type_ref],
                                },
                            );
                        }
                    }
                }

                // Add the node type if it's not internal
                if !is_internal {
                    node_types.push(NodeType {
                        type_name: name.clone(),
                        named: true,
                        fields: if fields.is_empty() {
                            None
                        } else {
                            Some(fields)
                        },
                        children: None,
                        subtypes: None,
                    });
                }
            }

            processed.insert(*symbol_id);
        }

        // Add tokens as unnamed nodes
        for (_, token) in &self.grammar.tokens {
            let (type_name, named) = match &token.pattern {
                TokenPattern::String(s) => (s.clone(), false),
                TokenPattern::Regex(_) => (token.name.clone(), true),
            };

            if !named {
                node_types.push(NodeType {
                    type_name,
                    named,
                    fields: None,
                    children: None,
                    subtypes: None,
                });
            }
        }

        // Sort for consistent output
        node_types.sort_by(|a, b| a.type_name.cmp(&b.type_name));

        // Serialize to JSON
        serde_json::to_string_pretty(&node_types)
            .map_err(|e| format!("Failed to serialize NODE_TYPES: {}", e))
    }

    fn get_rule_name(&self, symbol_id: adze_ir::SymbolId) -> Option<String> {
        // Check if this is a token first
        if let Some(token) = self.grammar.tokens.get(&symbol_id) {
            return Some(token.name.clone());
        }

        // Look up rule name
        if let Some(rule_name) = self.grammar.rule_names.get(&symbol_id) {
            return Some(rule_name.clone());
        }

        // Fallback
        Some(format!("rule_{}", symbol_id.0))
    }

    fn symbol_to_type_ref(
        &self,
        symbol: &Symbol,
        symbol_names: &HashMap<adze_ir::SymbolId, String>,
    ) -> TypeRef {
        match symbol {
            Symbol::Terminal(id) => {
                if let Some(token) = self.grammar.tokens.get(id) {
                    match &token.pattern {
                        TokenPattern::String(s) => TypeRef {
                            type_name: s.clone(),
                            named: false,
                        },
                        TokenPattern::Regex(_) => TypeRef {
                            type_name: token.name.clone(),
                            named: true,
                        },
                    }
                } else {
                    TypeRef {
                        type_name: "unknown".to_string(),
                        named: false,
                    }
                }
            }
            Symbol::NonTerminal(id) => TypeRef {
                type_name: symbol_names
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                named: true,
            },
            Symbol::External(_) => TypeRef {
                type_name: "external".to_string(),
                named: true,
            },
            Symbol::Optional(inner) => self.symbol_to_type_ref(inner, symbol_names),
            Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
                let inner_ref = self.symbol_to_type_ref(inner, symbol_names);
                TypeRef {
                    type_name: inner_ref.type_name,
                    named: inner_ref.named,
                }
            }
            Symbol::Choice(choices) => {
                // For now, just use the first choice
                if let Some(first) = choices.first() {
                    self.symbol_to_type_ref(first, symbol_names)
                } else {
                    TypeRef {
                        type_name: "empty".to_string(),
                        named: false,
                    }
                }
            }
            Symbol::Sequence(seq) => {
                // For sequences, we might want to create a composite type
                if let Some(first) = seq.first() {
                    self.symbol_to_type_ref(first, symbol_names)
                } else {
                    TypeRef {
                        type_name: "empty".to_string(),
                        named: false,
                    }
                }
            }
            Symbol::Epsilon => TypeRef {
                type_name: "empty".to_string(),
                named: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::{ProductionId, Rule, SymbolId, Token};

    #[test]
    fn test_simple_node_types() {
        let mut grammar = Grammar::new("test".to_string());

        // Add a number token
        let number_token = Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        };
        let number_token_id = SymbolId(0);
        grammar.tokens.insert(number_token_id, number_token);

        // Add a simple rule
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(number_token_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();

        let node_types: Vec<NodeType> = serde_json::from_str(&result).unwrap();
        assert!(!node_types.is_empty());
    }
}
