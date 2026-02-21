//! Field Name Implementation Sketch
//!
//! This file contains the implementation plan for field name propagation.
//! Copy relevant parts to the actual implementation files.

// === Step 1: Update Subtree/Node structures ===

/// In subtree.rs or equivalent - add field_id to child edges
#[derive(Clone, Debug)]
struct ChildEdge {
    node: SubtreeId,
    /// Field ID from grammar, 0xFFFF means no field
    field_id: u16,
}

impl ChildEdge {
    const NO_FIELD: u16 = 0xFFFF;

    fn new(node: SubtreeId) -> Self {
        Self {
            node,
            field_id: Self::NO_FIELD,
        }
    }

    fn with_field(node: SubtreeId, field_id: u16) -> Self {
        Self { node, field_id }
    }

    fn has_field(&self) -> bool {
        self.field_id != Self::NO_FIELD
    }
}

// === Step 2: Wire field map during reduction ===

/// In parser_v4.rs - during reduction
fn reduce_production(
    &mut self,
    production_id: ProductionId,
    children: Vec<SubtreeId>,
    language: &Language,
) -> Subtree {
    // Create child edges without fields initially
    let mut edges: Vec<ChildEdge> = children.into_iter().map(|id| ChildEdge::new(id)).collect();

    // Apply field mappings from language data
    if let Some((start, len)) = language.field_map_slices.get(production_id as usize) {
        let entries = &language.field_map_entries[*start..*start + *len];

        for entry in entries {
            // entry format: { field_id: u16, child_index: u16 }
            let field_id = entry.field_id;
            let child_idx = entry.child_index as usize;

            if child_idx < edges.len() {
                edges[child_idx].field_id = field_id;
            }
        }
    }

    // Create subtree with field-annotated edges
    Subtree::new(production_id, edges)
}

// === Step 3: Language metadata exposure ===

/// In language.rs - expose field data
#[derive(Debug)]
pub struct Language {
    // Existing fields...
    /// Field names indexed by field ID
    pub field_names: Vec<&'static str>,

    /// Field map slices: production_id -> (start, length) in field_map_entries
    pub field_map_slices: Vec<(usize, usize)>,

    /// Field map entries: { field_id, child_index } pairs
    pub field_map_entries: Vec<FieldMapEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldMapEntry {
    pub field_id: u16,
    pub child_index: u16,
}

impl Language {
    pub fn field_name(&self, field_id: u16) -> Option<&'static str> {
        if field_id == ChildEdge::NO_FIELD {
            None
        } else {
            self.field_names.get(field_id as usize).copied()
        }
    }

    pub fn field_id_for_name(&self, name: &str) -> Option<u16> {
        self.field_names
            .iter()
            .position(|&n| n == name)
            .map(|idx| idx as u16)
    }
}

// === Step 4: Node API implementation ===

/// In node.rs - public API for field access
impl Node {
    /// Get the field name for this node in its parent context
    pub fn field_name(&self) -> Option<&'static str> {
        // This requires parent to track which field this child represents
        // Stored in parent's child_edges[child_index].field_id

        if let Some(parent) = self.parent() {
            if let Some(edge) = parent.get_child_edge(self.child_index()) {
                if edge.has_field() {
                    return self.language.field_name(edge.field_id);
                }
            }
        }
        None
    }

    /// Get child node by field name
    pub fn child_by_field_name(&self, field_name: &str) -> Option<Node> {
        let field_id = self.language.field_id_for_name(field_name)?;

        for (idx, edge) in self.child_edges().enumerate() {
            if edge.field_id == field_id {
                return self.child(idx);
            }
        }
        None
    }

    /// Iterate children with their field names
    pub fn children_with_fields(&self) -> impl Iterator<Item = (Option<&'static str>, Node)> {
        self.child_edges().enumerate().map(move |(idx, edge)| {
            let field_name = if edge.has_field() {
                self.language.field_name(edge.field_id)
            } else {
                None
            };
            let child = self.child(idx).expect("Child should exist");
            (field_name, child)
        })
    }
}

// === Step 5: Fix extract_field_name stub ===

/// In pure_parser.rs:1197 - replace stub
fn extract_field_name(subtree: &Subtree, language: Option<*const TSLanguage>) -> Option<String> {
    // If we have parent context and this subtree has a field ID
    if let Some(lang_ptr) = language {
        let lang = unsafe { &*lang_ptr };

        // Get field_id from subtree's position in parent
        // This requires parent tracking, might need to pass more context
        if let Some(field_id) = subtree.field_id() {
            if let Some(name) = lang.field_name(field_id) {
                return Some(name.to_string());
            }
        }
    }
    None
}

// === Step 6: S-expression with fields ===

/// In serialization.rs - include field annotations
impl Node {
    pub fn to_sexp_with_fields(&self) -> String {
        let mut result = String::new();
        self.write_sexp_with_fields(&mut result, 0);
        result
    }

    fn write_sexp_with_fields(&self, out: &mut String, depth: usize) {
        out.push('(');
        out.push_str(self.kind());

        for (field_name, child) in self.children_with_fields() {
            out.push(' ');
            if let Some(name) = field_name {
                out.push_str(name);
                out.push(':');
            }
            child.write_sexp_with_fields(out, depth + 1);
        }

        out.push(')');
    }
}

// === Step 7: JSON with fields ===

/// In serialization.rs - JSON output
impl Node {
    pub fn to_json_with_fields(&self) -> serde_json::Value {
        use serde_json::json;

        let children: Vec<_> = self
            .children_with_fields()
            .map(|(field, child)| {
                let mut obj = child.to_json_with_fields();
                if let Some(name) = field {
                    obj["field"] = json!(name);
                }
                obj
            })
            .collect();

        json!({
            "type": self.kind(),
            "start": self.start_byte(),
            "end": self.end_byte(),
            "children": children,
        })
    }
}
