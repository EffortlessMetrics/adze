//! Deterministic symbol ID assignment via an ordered registry.

use crate::{SymbolId, SymbolMetadata};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A centralized registry for symbol ID assignment and metadata.
/// Ensures consistent, deterministic symbol ordering across all components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolRegistry {
    /// Ordered map of symbol names to IDs (maintains insertion order)
    symbols: IndexMap<String, SymbolId>,
    /// Reverse lookup: ID to name
    ids: HashMap<SymbolId, String>,
    /// Metadata for each symbol
    metadata: HashMap<SymbolId, SymbolMetadata>,
    /// Next available symbol ID
    next_id: u16,
}

/// Metadata about a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolInfo {
    /// Symbol ID
    pub id: SymbolId,
    /// Symbol metadata
    pub metadata: SymbolMetadata,
}

impl SymbolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        let mut registry = Self {
            symbols: IndexMap::new(),
            ids: HashMap::new(),
            metadata: HashMap::new(),
            next_id: 0,
        };

        // EOF is always symbol 0
        registry.register(
            "end",
            SymbolMetadata {
                visible: true,
                named: false,
                hidden: false,
                terminal: true,
            },
        );

        registry
    }

    /// Register a symbol with automatic ID assignment
    pub fn register(&mut self, name: &str, metadata: SymbolMetadata) -> SymbolId {
        if let Some(&id) = self.symbols.get(name) {
            // Update metadata if symbol already exists
            self.metadata.insert(id, metadata);
            return id;
        }

        let id = SymbolId(self.next_id);
        self.next_id += 1;

        self.symbols.insert(name.to_string(), id);
        self.ids.insert(id, name.to_string());
        self.metadata.insert(id, metadata);

        id
    }

    /// Get symbol ID by name
    pub fn get_id(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Get symbol name by ID
    pub fn get_name(&self, id: SymbolId) -> Option<&str> {
        self.ids.get(&id).map(String::as_str)
    }

    /// Get metadata for a symbol
    pub fn get_metadata(&self, id: SymbolId) -> Option<SymbolMetadata> {
        self.metadata.get(&id).copied()
    }

    /// Check if a symbol ID exists
    pub fn contains_id(&self, id: SymbolId) -> bool {
        self.ids.contains_key(&id)
    }

    /// Get total number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Iterate over all symbols in order
    pub fn iter(&self) -> impl Iterator<Item = (&str, SymbolInfo)> {
        self.symbols.iter().map(move |(name, &id)| {
            let metadata = self.metadata[&id];
            (name.as_str(), SymbolInfo { id, metadata })
        })
    }

    /// Create a symbol-to-index mapping for parse table generation
    pub fn to_index_map(&self) -> HashMap<SymbolId, usize> {
        self.symbols
            .values()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect()
    }

    /// Create an index-to-symbol mapping for parse table decompression
    pub fn to_symbol_map(&self) -> HashMap<usize, SymbolId> {
        self.symbols
            .values()
            .enumerate()
            .map(|(idx, &id)| (idx, id))
            .collect()
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_deterministic() {
        let mut reg1 = SymbolRegistry::new();
        let mut reg2 = SymbolRegistry::new();

        // Register symbols in same order
        for name in ["number", "plus", "minus", "expr"] {
            let meta = SymbolMetadata {
                visible: true,
                named: name == "expr",
                hidden: false,
                terminal: name != "expr",
            };
            reg1.register(name, meta);
            reg2.register(name, meta);
        }

        // Should have same IDs
        for name in ["number", "plus", "minus", "expr"] {
            assert_eq!(reg1.get_id(name), reg2.get_id(name));
        }
    }

    #[test]
    fn test_eof_is_zero() {
        let registry = SymbolRegistry::new();
        assert_eq!(registry.get_id("end"), Some(SymbolId(0)));
    }
}
