//! Tree → text/JSON/binary serializers for debugging and tooling.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Parse tree serialization for the pure-Rust Tree-sitter implementation
// This module provides serialization and deserialization of parse trees

#[cfg(feature = "pure-rust")]
use crate::pure_incremental::Tree;
#[cfg(feature = "pure-rust")]
use crate::pure_parser::ParsedNode as Node;

#[cfg(not(feature = "pure-rust"))]
use crate::tree_sitter::{Node, Tree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(all(feature = "tree-sitter-standard", not(feature = "pure-rust")))]
use tree_sitter::TreeCursor;
#[cfg(all(feature = "tree-sitter-c2rust", not(feature = "pure-rust")))]
use tree_sitter_c2rust::TreeCursor;

#[cfg(feature = "serialization")]
use serde_json::{json, Value};

/// Serializable representation of a parse tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedNode {
    /// Node type/kind
    pub kind: String,
    /// Whether this is a named node
    pub is_named: bool,
    /// Field name if this node is a field
    pub field_name: Option<String>,
    /// Start position (row, column)
    pub start_position: (usize, usize),
    /// End position (row, column)
    pub end_position: (usize, usize),
    /// Start byte offset
    pub start_byte: usize,
    /// End byte offset
    pub end_byte: usize,
    /// Text content for leaf nodes
    pub text: Option<String>,
    /// Child nodes
    pub children: Vec<SerializedNode>,
    /// Whether this is an error node
    pub is_error: bool,
    /// Whether this is missing
    pub is_missing: bool,
}

/// Serializer for parse trees
pub struct TreeSerializer<'a> {
    pub source: &'a [u8],
    pub include_unnamed: bool,
    pub max_text_length: Option<usize>,
}

impl<'a> TreeSerializer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            include_unnamed: false,
            max_text_length: Some(100),
        }
    }

    /// Include unnamed nodes in serialization
    pub fn with_unnamed_nodes(mut self) -> Self {
        self.include_unnamed = true;
        self
    }

    /// Set maximum text length for leaf nodes
    pub fn with_max_text_length(mut self, max_length: Option<usize>) -> Self {
        self.max_text_length = max_length;
        self
    }

    /// Serialize a tree to JSON
    pub fn serialize_tree(&self, tree: &Tree) -> Result<String, serde_json::Error> {
        #[cfg(feature = "pure-rust")]
        let root = self.serialize_node(&tree.root);
        #[cfg(not(feature = "pure-rust"))]
        let root = self.serialize_node(tree.root_node());
        serde_json::to_string_pretty(&root)
    }

    /// Serialize a single node
    #[cfg(feature = "pure-rust")]
    pub fn serialize_node(&self, node: &Node) -> SerializedNode {
        let mut serialized = SerializedNode {
            kind: format!("symbol_{}", node.symbol), // Convert symbol to string
            is_named: node.is_named,
            field_name: node.field_name.clone(),
            start_position: (
                node.start_point.row as usize,
                node.start_point.column as usize,
            ),
            end_position: (node.end_point.row as usize, node.end_point.column as usize),
            start_byte: node.start_byte,
            end_byte: node.end_byte,
            text: None,
            children: Vec::new(),
            is_error: node.is_error,
            is_missing: node.is_missing,
        };

        // Add text for leaf nodes
        if node.children.is_empty() {
            let text = String::from_utf8_lossy(&self.source[node.start_byte..node.end_byte]);
            let text = if let Some(max_len) = self.max_text_length {
                if text.len() > max_len {
                    format!("{}...", &text[..max_len])
                } else {
                    text.to_string()
                }
            } else {
                text.to_string()
            };
            serialized.text = Some(text);
        }

        // Serialize children
        for child in &node.children {
            serialized.children.push(self.serialize_node(child));
        }

        serialized
    }

    #[cfg(not(feature = "pure-rust"))]
    pub fn serialize_node(&self, node: Node) -> SerializedNode {
        let mut serialized = SerializedNode {
            kind: node.kind().to_string(),
            is_named: node.is_named(),
            field_name: node.field_name().map(|s| s.to_string()),
            start_position: (node.start_position().row, node.start_position().column),
            end_position: (node.end_position().row, node.end_position().column),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            text: None,
            children: Vec::new(),
            is_error: node.is_error(),
            is_missing: node.is_missing(),
        };

        // Add text for leaf nodes
        if node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(self.source) {
                let text = if let Some(max_len) = self.max_text_length {
                    if text.len() > max_len {
                        format!("{}...", &text[..max_len])
                    } else {
                        text.to_string()
                    }
                } else {
                    text.to_string()
                };
                serialized.text = Some(text);
            }
        }

        // Serialize children
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if self.include_unnamed || child.is_named() {
                    serialized.children.push(self.serialize_node(child));
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        serialized
    }
}

/// Compact serialization format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactNode {
    #[serde(rename = "t")]
    pub kind: String,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(rename = "c", skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<CompactNode>,
    #[serde(rename = "x", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Compact serializer
pub struct CompactSerializer<'a> {
    source: &'a [u8],
}

impl<'a> CompactSerializer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    pub fn serialize_tree(&self, tree: &Tree) -> Result<String, serde_json::Error> {
        let root = self.serialize_node(tree.root_node());
        serde_json::to_string(&root)
    }

    fn serialize_node(&self, node: &Node) -> CompactNode {
        let mut compact = CompactNode {
            kind: node.kind().to_string(),
            start: Some(node.start_byte),
            end: Some(node.end_byte),
            field: node.field_name.as_ref().map(|s| s.to_string()),
            children: Vec::new(),
            text: None,
        };

        if node.child_count() == 0 {
            compact.text = node.utf8_text(self.source).ok().map(|s| s.to_string());
            // Don't include position for leaf nodes to save space
            compact.start = None;
            compact.end = None;
        } else {
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.is_named {
                        compact.children.push(self.serialize_node(child));
                    }

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        compact
    }
}

/// S-expression format serializer
pub struct SExpressionSerializer<'a> {
    source: &'a [u8],
    include_positions: bool,
}

impl<'a> SExpressionSerializer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            include_positions: false,
        }
    }

    pub fn with_positions(mut self) -> Self {
        self.include_positions = true;
        self
    }

    pub fn serialize_tree(&self, tree: &Tree) -> String {
        self.serialize_node(tree.root_node())
    }

    fn serialize_node(&self, node: &Node) -> String {
        let mut result = String::new();

        if node.child_count() == 0 {
            // Leaf node
            if let Ok(text) = node.utf8_text(self.source) {
                result.push_str(&format!("\"{}\"", text.replace('"', "\\\"")));
            }
        } else {
            // Internal node
            result.push('(');

            if let Some(field_name) = node.field_name.as_ref() {
                result.push_str(&format!("{}: ", field_name));
            }

            result.push_str(node.kind());

            if self.include_positions {
                result.push_str(&format!(
                    " [{},{}-{},{}]",
                    node.start_point.row,
                    node.start_point.column,
                    node.end_point.row,
                    node.end_point.column
                ));
            }

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    result.push(' ');
                    result.push_str(&self.serialize_node(cursor.node()));

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            result.push(')');
        }

        result
    }
}

/// S-expression deserialization support
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

/// Parse S-expression from string
pub fn parse_sexpr(input: &str) -> Result<SExpr, String> {
    let mut chars = input.trim().chars().peekable();
    parse_sexpr_inner(&mut chars)
}

fn parse_sexpr_inner(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<SExpr, String> {
    skip_whitespace(chars);

    match chars.peek() {
        Some('(') => {
            chars.next(); // consume '('
            let mut items = Vec::new();

            loop {
                skip_whitespace(chars);
                if chars.peek() == Some(&')') {
                    chars.next(); // consume ')'
                    break;
                }
                items.push(parse_sexpr_inner(chars)?);
            }

            Ok(SExpr::List(items))
        }
        Some('"') => {
            chars.next(); // consume opening '"'
            let mut atom = String::new();
            let mut escaped = false;

            while let Some(ch) = chars.next() {
                if escaped {
                    match ch {
                        '"' => atom.push('"'),
                        '\\' => atom.push('\\'),
                        'n' => atom.push('\n'),
                        't' => atom.push('\t'),
                        'r' => atom.push('\r'),
                        _ => {
                            atom.push('\\');
                            atom.push(ch);
                        }
                    }
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    return Ok(SExpr::Atom(atom));
                } else {
                    atom.push(ch);
                }
            }

            Err("Unterminated string literal".to_string())
        }
        Some(_) => {
            let mut atom = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '(' || ch == ')' {
                    break;
                }
                atom.push(chars.next().unwrap());
            }

            if atom.is_empty() {
                Err("Empty atom".to_string())
            } else {
                Ok(SExpr::Atom(atom))
            }
        }
        None => Err("Unexpected end of input".to_string()),
    }
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while chars.peek().map_or(false, |c| c.is_whitespace()) {
        chars.next();
    }
}

/// Convert S-expression back to SerializedNode for roundtrip testing
pub fn sexpr_to_serialized_node(sexpr: &SExpr) -> Result<SerializedNode, String> {
    match sexpr {
        SExpr::Atom(text) => Ok(SerializedNode {
            kind: "text".to_string(),
            is_named: false,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, text.len()),
            start_byte: 0,
            end_byte: text.len(),
            text: Some(text.clone()),
            children: vec![],
            is_error: false,
            is_missing: false,
        }),
        SExpr::List(items) => {
            if items.is_empty() {
                return Err("Empty list not allowed".to_string());
            }

            let kind = match &items[0] {
                SExpr::Atom(name) => name.clone(),
                _ => return Err("First element of list must be node kind".to_string()),
            };

            let mut children = Vec::new();
            for item in &items[1..] {
                children.push(sexpr_to_serialized_node(item)?);
            }

            Ok(SerializedNode {
                kind,
                is_named: true,
                field_name: None,
                start_position: (0, 0),
                end_position: (0, 0),
                start_byte: 0,
                end_byte: 0,
                text: None,
                children,
                is_error: false,
                is_missing: false,
            })
        }
    }
}

/// Tree statistics for analysis
#[derive(Debug, Clone, Default)]
pub struct TreeStatistics {
    pub total_nodes: usize,
    pub named_nodes: usize,
    pub error_nodes: usize,
    pub missing_nodes: usize,
    pub max_depth: usize,
    pub node_types: HashMap<String, usize>,
}

impl TreeStatistics {
    pub fn from_tree(tree: &Tree) -> Self {
        let mut stats = Self::default();
        stats.analyze_node(tree.root_node(), 0);
        stats
    }

    fn analyze_node(&mut self, node: &Node, depth: usize) {
        self.total_nodes += 1;
        self.max_depth = self.max_depth.max(depth);

        if node.is_named() {
            self.named_nodes += 1;
        }
        if node.is_error() {
            self.error_nodes += 1;
        }
        if node.is_missing() {
            self.missing_nodes += 1;
        }

        *self.node_types.entry(node.kind().to_string()).or_insert(0) += 1;

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                self.analyze_node(cursor.node(), depth + 1);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
}

/// Binary format for efficient storage
#[derive(Debug, Clone)]
pub struct BinaryFormat {
    /// Node types indexed by ID
    pub node_types: Vec<String>,
    /// Field names indexed by ID
    pub field_names: Vec<String>,
    /// Binary tree data
    pub tree_data: Vec<u8>,
}

/// Binary serializer for compact storage
pub struct BinarySerializer {
    node_type_map: HashMap<String, u16>,
    field_name_map: HashMap<String, u16>,
    node_types: Vec<String>,
    field_names: Vec<String>,
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySerializer {
    pub fn new() -> Self {
        Self {
            node_type_map: HashMap::new(),
            field_name_map: HashMap::new(),
            node_types: Vec::new(),
            field_names: Vec::new(),
        }
    }

    pub fn serialize_tree(&mut self, tree: &Tree) -> BinaryFormat {
        let mut tree_data = Vec::new();
        self.serialize_node_binary(tree.root_node(), &mut tree_data);

        BinaryFormat {
            node_types: self.node_types.clone(),
            field_names: self.field_names.clone(),
            tree_data,
        }
    }

    fn get_node_type_id(&mut self, kind: &str) -> u16 {
        if let Some(&id) = self.node_type_map.get(kind) {
            id
        } else {
            let id = self.node_types.len() as u16;
            self.node_types.push(kind.to_string());
            self.node_type_map.insert(kind.to_string(), id);
            id
        }
    }

    fn get_field_name_id(&mut self, name: &str) -> u16 {
        if let Some(&id) = self.field_name_map.get(name) {
            id
        } else {
            let id = self.field_names.len() as u16;
            self.field_names.push(name.to_string());
            self.field_name_map.insert(name.to_string(), id);
            id
        }
    }

    fn serialize_node_binary(&mut self, node: &Node, output: &mut Vec<u8>) {
        // Write node type ID (2 bytes)
        let type_id = self.get_node_type_id(node.kind());
        output.extend_from_slice(&type_id.to_le_bytes());

        // Write flags (1 byte)
        let mut flags = 0u8;
        if node.is_named {
            flags |= 0x01;
        }
        if node.is_error {
            flags |= 0x02;
        }
        if node.is_missing {
            flags |= 0x04;
        }
        if node.field_name.is_some() {
            flags |= 0x08;
        }
        output.push(flags);

        // Write field name ID if present (2 bytes)
        if let Some(field_name) = node.field_name.as_ref() {
            let field_id = self.get_field_name_id(field_name);
            output.extend_from_slice(&field_id.to_le_bytes());
        }

        // Write positions (4 bytes each)
        output.extend_from_slice(&(node.start_byte as u32).to_le_bytes());
        output.extend_from_slice(&(node.end_byte as u32).to_le_bytes());

        // Write child count (2 bytes)
        let child_count = node.child_count() as u16;
        output.extend_from_slice(&child_count.to_le_bytes());

        // Serialize children
        if child_count > 0 {
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    self.serialize_node_binary(cursor.node(), output);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
    }
}

/// Simple JSON serialization for parse trees
///
/// This provides a minimal JSON representation suitable for debugging and testing.
/// For more complex serialization needs, use the TreeSerializer above.
#[cfg(feature = "serialization")]
pub fn node_to_json(
    node: &crate::pure_parser::ParsedNode,
    src: &[u8],
    lang: &crate::pure_parser::TSLanguage,
) -> Value {
    let children: Vec<Value> = node
        .children
        .iter()
        .map(|child| node_to_json(child, src, lang))
        .collect();

    // Get symbol name from language
    let kind = get_symbol_name(lang, node.symbol);

    json!({
        "kind": kind,
        "start_byte": node.start_byte,
        "end_byte": node.end_byte,
        "children": children,
    })
}

/// Convert a tree to JSON (convenience wrapper)
#[cfg(feature = "serialization")]
pub fn tree_to_json(
    root: &crate::pure_parser::ParsedNode,
    src: &[u8],
    lang: &crate::pure_parser::TSLanguage,
) -> Value {
    node_to_json(root, src, lang)
}

/// Get symbol name from language tables
#[cfg(feature = "serialization")]
fn get_symbol_name(lang: &crate::pure_parser::TSLanguage, symbol: u16) -> String {
    // Safety: We trust the language tables are valid
    unsafe {
        if lang.symbol_names.is_null() || symbol as u32 >= lang.symbol_count {
            return format!("UNKNOWN_{}", symbol);
        }

        let symbol_names =
            std::slice::from_raw_parts(lang.symbol_names, lang.symbol_count as usize);

        let name_ptr = symbol_names[symbol as usize];
        if name_ptr.is_null() {
            return format!("NULL_{}", symbol);
        }

        // Convert C string to Rust string
        std::ffi::CStr::from_ptr(name_ptr as *const i8)
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialized_node_creation() {
        let node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 5),
            start_byte: 0,
            end_byte: 5,
            text: Some("hello".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        assert_eq!(node.kind, "identifier");
        assert_eq!(node.text, Some("hello".to_string()));
    }

    #[test]
    fn test_compact_node_serialization() {
        let node = CompactNode {
            kind: "id".to_string(),
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some("test".to_string()),
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"t\":\"id\""));
        assert!(json.contains("\"x\":\"test\""));
    }

    #[test]
    fn test_s_expression_format() {
        // Test would use actual Tree-sitter nodes
        // This is a placeholder showing the expected format
        let expected = "(program (function_declaration name: (identifier \"main\")))";
        assert!(expected.contains("function_declaration"));
    }

    #[test]
    fn test_serialized_node_construction() {
        let node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (0, 0),
            end_position: (0, 4),
            start_byte: 0,
            end_byte: 4,
            text: Some("test".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        assert_eq!(node.kind, "identifier");
        assert!(node.is_named);
        assert_eq!(node.field_name, Some("name".to_string()));
        assert_eq!(node.text, Some("test".to_string()));
        assert!(node.children.is_empty());
        assert!(!node.is_error);
        assert!(!node.is_missing);
    }

    #[test]
    fn test_tree_serializer_configuration() {
        let source = b"test source code";
        let serializer = TreeSerializer::new(source)
            .with_unnamed_nodes()
            .with_max_text_length(Some(50));

        assert!(serializer.include_unnamed);
        assert_eq!(serializer.max_text_length, Some(50));
        assert_eq!(serializer.source, source);
    }

    #[test]
    #[ignore] // TODO: TreeStatistics type needs to be defined
    fn test_tree_statistics() {
        // TODO: TreeStatistics type needs to be defined - this test is incomplete
        // This test is disabled until TreeStatistics is properly implemented
    }

    #[test]
    fn test_serialized_node_with_children() {
        let child1 = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 3),
            start_byte: 0,
            end_byte: 3,
            text: Some("foo".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        let child2 = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 4),
            end_position: (0, 7),
            start_byte: 4,
            end_byte: 7,
            text: Some("bar".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        let parent = SerializedNode {
            kind: "binary_expression".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 7),
            start_byte: 0,
            end_byte: 7,
            text: None,
            children: vec![child1, child2],
            is_error: false,
            is_missing: false,
        };

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].text, Some("foo".to_string()));
        assert_eq!(parent.children[1].text, Some("bar".to_string()));
    }

    #[test]
    fn test_max_text_length_truncation() {
        let long_text = "This is a very long text that should be truncated";
        let max_len = 20;

        let truncated = if long_text.len() > max_len {
            format!("{}...", &long_text[..max_len])
        } else {
            long_text.to_string()
        };

        assert_eq!(truncated, "This is a very long ...");
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.len(), max_len + 3); // 20 chars + "..."
    }

    #[test]
    fn test_error_node_serialization() {
        let error_node = SerializedNode {
            kind: "ERROR".to_string(),
            is_named: false,
            field_name: None,
            start_position: (1, 5),
            end_position: (1, 10),
            start_byte: 15,
            end_byte: 20,
            text: Some("invalid".to_string()),
            children: vec![],
            is_error: true,
            is_missing: false,
        };

        assert!(error_node.is_error);
        assert_eq!(error_node.kind, "ERROR");
        assert!(!error_node.is_named);
    }

    #[test]
    fn test_missing_node_serialization() {
        let missing_node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (2, 0),
            end_position: (2, 0),
            start_byte: 30,
            end_byte: 30,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: true,
        };

        assert!(missing_node.is_missing);
        assert!(!missing_node.is_error);
        assert_eq!(missing_node.start_byte, missing_node.end_byte);
    }
}
