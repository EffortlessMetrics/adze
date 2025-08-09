/// Field-aware tree structures where field names are stored as edge properties
/// This design correctly models that field-ness is a property of parent→child relationships

use crate::ffi::TSSymbol;
use std::sync::Arc;

/// A child node with an optional field identifier
#[derive(Debug, Clone)]
pub struct ParsedChild {
    /// The child node
    pub node: ParsedNode,
    /// Optional field ID for this child (None if not a named field)
    pub field_id: Option<u16>,
}

impl ParsedChild {
    /// Get the field name for this child if it has one
    pub fn field_name<'a>(&self, language: &'a TSLanguage) -> Option<&'a str> {
        self.field_id.and_then(|id| language.field_name(id))
    }
}

/// A parsed node in the syntax tree
#[derive(Debug, Clone)]
pub struct ParsedNode {
    /// Symbol/kind ID for this node
    pub symbol: TSSymbol,
    /// Child nodes with their field information
    pub children: Vec<ParsedChild>,
    /// Byte range in source text
    pub start_byte: usize,
    pub end_byte: usize,
    /// Position in lines/columns
    pub start_point: Point,
    pub end_point: Point,
    /// Node flags
    pub is_extra: bool,
    pub is_error: bool,
    pub is_missing: bool,
    pub is_named: bool,
    /// Reference to the language for symbol/field name lookups
    pub language: Option<Arc<TSLanguage>>,
}

impl ParsedNode {
    /// Get a child by field name
    pub fn child_by_field_name<'a>(&'a self, name: &str) -> Option<&'a ParsedNode> {
        let language = self.language.as_ref()?;
        let field_id = language.field_id_for_name(name)?;
        
        self.children
            .iter()
            .find(|c| c.field_id == Some(field_id))
            .map(|c| &c.node)
    }
    
    /// Get all children with a specific field name
    pub fn children_by_field_name<'a>(&'a self, name: &str) -> Vec<&'a ParsedNode> {
        let language = match self.language.as_ref() {
            Some(l) => l,
            None => return Vec::new(),
        };
        
        let field_id = match language.field_id_for_name(name) {
            Some(id) => id,
            None => return Vec::new(),
        };
        
        self.children
            .iter()
            .filter(|c| c.field_id == Some(field_id))
            .map(|c| &c.node)
            .collect()
    }
    
    /// Get the node's kind/type name
    pub fn kind<'a>(&self, language: &'a TSLanguage) -> &'a str {
        language.symbol_name(self.symbol)
    }
    
    /// Iterate over all named children (skip anonymous/extra nodes)
    pub fn named_children(&self) -> impl Iterator<Item = &ParsedChild> {
        self.children.iter().filter(|c| c.node.is_named)
    }
    
    /// Get the Nth child (if it exists)
    pub fn child(&self, index: usize) -> Option<&ParsedNode> {
        self.children.get(index).map(|c| &c.node)
    }
    
    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

/// Point in the source text (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

impl Point {
    pub fn new(row: usize, column: usize) -> Self {
        Point { row, column }
    }
}

/// Language definition with field name tables
pub struct TSLanguage {
    /// Field names table (sorted lexicographically)
    pub field_names: Vec<&'static str>,
    /// Symbol names table
    pub symbol_names: Vec<&'static str>,
    /// Production field mappings: production_id -> child_index -> field_id
    pub production_field_map: Vec<Vec<Option<u16>>>,
    // ... other language data
}

impl TSLanguage {
    /// Look up a field ID by name (binary search since names are sorted)
    pub fn field_id_for_name(&self, name: &str) -> Option<u16> {
        self.field_names
            .binary_search_by_key(&name, |&n| n)
            .ok()
            .map(|idx| idx as u16)
    }
    
    /// Get field name by ID
    pub fn field_name(&self, id: u16) -> Option<&'static str> {
        self.field_names.get(id as usize).copied()
    }
    
    /// Get symbol name by ID
    pub fn symbol_name(&self, symbol: TSSymbol) -> &'static str {
        self.symbol_names
            .get(symbol as usize)
            .copied()
            .unwrap_or("ERROR")
    }
    
    /// Get field mappings for a production
    pub fn production_fields(&self, production_id: u16) -> &[Option<u16>] {
        self.production_field_map
            .get(production_id as usize)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Builder functions for creating nodes during parsing
impl ParsedNode {
    /// Create a node from a reduce action
    pub fn from_reduction(
        symbol: TSSymbol,
        production_id: u16,
        children: Vec<ParsedNode>,
        language: Arc<TSLanguage>,
    ) -> Self {
        // Get field mappings for this production
        let field_map = language.production_fields(production_id);
        
        // Build children with field IDs
        let parsed_children: Vec<ParsedChild> = children
            .into_iter()
            .enumerate()
            .map(|(i, child)| ParsedChild {
                node: child,
                field_id: field_map.get(i).and_then(|&f| f),
            })
            .collect();
        
        // Compute byte ranges from children
        let (start_byte, end_byte) = if parsed_children.is_empty() {
            (0, 0)
        } else {
            let start = parsed_children[0].node.start_byte;
            let end = parsed_children.last().unwrap().node.end_byte;
            (start, end)
        };
        
        // Compute point ranges from children
        let (start_point, end_point) = if parsed_children.is_empty() {
            (Point::new(0, 0), Point::new(0, 0))
        } else {
            let start = parsed_children[0].node.start_point;
            let end = parsed_children.last().unwrap().node.end_point;
            (start, end)
        };
        
        ParsedNode {
            symbol,
            children: parsed_children,
            start_byte,
            end_byte,
            start_point,
            end_point,
            is_extra: false,
            is_error: false,
            is_missing: false,
            is_named: true, // TODO: Get from symbol metadata
            language: Some(language),
        }
    }
}