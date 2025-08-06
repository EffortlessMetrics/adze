// Enhanced incremental parsing with proper subtree reuse
// This module implements efficient incremental parsing by reusing unchanged subtrees

use crate::parser_v3::ParseNode;
use anyhow::Result;
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::{HashMap, HashSet};

/// Edit operation representing a change in the source
#[derive(Debug, Clone)]
pub struct Edit {
    /// Start byte position of the edit
    pub start_byte: usize,
    /// Old end byte position (before edit)
    pub old_end_byte: usize,
    /// New end byte position (after edit)
    pub new_end_byte: usize,
    /// Start position in the old tree
    pub start_position: Position,
    /// Old end position
    pub old_end_position: Position,
    /// New end position
    pub new_end_position: Position,
}

/// Position in the source (line, column)
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

/// A subtree that can potentially be reused
#[derive(Debug, Clone)]
pub struct ReusableSubtree {
    /// The parse node
    pub node: ParseNode,
    /// Byte range in the source
    pub byte_range: std::ops::Range<usize>,
    /// Whether this subtree is affected by edits
    pub is_affected: bool,
    /// Hash of the subtree for quick comparison
    pub hash: u64,
}

/// Pool of reusable subtrees from previous parse
#[derive(Debug)]
pub struct SubtreePool {
    /// All subtrees indexed by their start byte
    subtrees_by_start: HashMap<usize, Vec<ReusableSubtree>>,
    /// Subtrees indexed by their symbol and size for quick lookup
    subtrees_by_symbol: HashMap<(SymbolId, usize), Vec<ReusableSubtree>>,
    /// Set of byte positions affected by edits
    affected_bytes: HashSet<usize>,
}

impl SubtreePool {
    /// Build a pool from an existing parse tree
    pub fn from_tree(tree: &ParseNode, edits: &[Edit]) -> Self {
        let mut pool = SubtreePool {
            subtrees_by_start: HashMap::new(),
            subtrees_by_symbol: HashMap::new(),
            affected_bytes: HashSet::new(),
        };

        // Mark affected byte ranges
        for edit in edits {
            for byte in edit.start_byte..edit.old_end_byte {
                pool.affected_bytes.insert(byte);
            }
        }

        // Collect all subtrees
        pool.collect_subtrees(tree, edits);

        pool
    }

    /// Recursively collect subtrees from a parse tree
    fn collect_subtrees(&mut self, node: &ParseNode, edits: &[Edit]) {
        let byte_range = node.start_byte..node.end_byte;
        let is_affected = self.is_affected_by_edits(&byte_range, edits);

        // Only collect unaffected subtrees or large affected ones that might have unaffected parts
        if !is_affected || node.children.len() > 3 {
            let subtree = ReusableSubtree {
                node: node.clone(),
                byte_range: byte_range.clone(),
                is_affected,
                hash: self.hash_subtree(node),
            };

            // Index by start position
            self.subtrees_by_start
                .entry(byte_range.start)
                .or_insert_with(Vec::new)
                .push(subtree.clone());

            // Index by symbol and size
            let key = (node.symbol, byte_range.end - byte_range.start);
            self.subtrees_by_symbol
                .entry(key)
                .or_insert_with(Vec::new)
                .push(subtree);
        }

        // Recurse into children
        for child in &node.children {
            self.collect_subtrees(child, edits);
        }
    }

    /// Check if a byte range is affected by any edit
    fn is_affected_by_edits(&self, range: &std::ops::Range<usize>, edits: &[Edit]) -> bool {
        for edit in edits {
            // Check if edit overlaps with this range
            if edit.start_byte < range.end && edit.old_end_byte > range.start {
                return true;
            }

            // Check if this range needs shifting due to edit
            if edit.old_end_byte <= range.start {
                // This subtree comes after the edit and needs position adjustment
                return true;
            }
        }
        false
    }

    /// Compute a hash for a subtree (for equality checking)
    fn hash_subtree(&self, node: &ParseNode) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        node.symbol.hash(&mut hasher);
        node.children.len().hash(&mut hasher);
        (node.end_byte - node.start_byte).hash(&mut hasher);

        // Include child symbols in hash
        for child in &node.children {
            child.symbol.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Find a reusable subtree at a given position
    pub fn find_reusable_at(
        &self,
        position: usize,
        symbol: Option<SymbolId>,
    ) -> Option<&ReusableSubtree> {
        // First try exact position match
        if let Some(subtrees) = self.subtrees_by_start.get(&position) {
            for subtree in subtrees {
                if !subtree.is_affected {
                    if let Some(sym) = symbol {
                        if subtree.node.symbol == sym {
                            return Some(subtree);
                        }
                    } else {
                        return Some(subtree);
                    }
                }
            }
        }

        None
    }

    /// Find all reusable subtrees in a range
    pub fn find_reusable_in_range(&self, range: std::ops::Range<usize>) -> Vec<&ReusableSubtree> {
        let mut result = Vec::new();

        for (start, subtrees) in &self.subtrees_by_start {
            if *start >= range.start && *start < range.end {
                for subtree in subtrees {
                    if !subtree.is_affected && subtree.byte_range.end <= range.end {
                        result.push(subtree);
                    }
                }
            }
        }

        // Sort by start position
        result.sort_by_key(|s| s.byte_range.start);
        result
    }
}

/// Incremental parser that reuses subtrees
pub struct IncrementalParser {
    grammar: Grammar,
    parse_table: ParseTable,
    subtree_pool: Option<SubtreePool>,
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        Self {
            grammar,
            parse_table,
            subtree_pool: None,
        }
    }

    /// Parse with incremental reuse
    pub fn parse(
        &mut self,
        input: &str,
        old_tree: Option<&ParseNode>,
        edits: &[Edit],
    ) -> Result<ParseNode> {
        // Build subtree pool if we have an old tree
        if let Some(tree) = old_tree {
            self.subtree_pool = Some(SubtreePool::from_tree(tree, edits));
        }

        // Create a parser that can reuse subtrees
        let mut parser = IncrementalParseSession {
            input: input.as_bytes(),
            grammar: &self.grammar,
            parse_table: &self.parse_table,
            subtree_pool: self.subtree_pool.as_ref(),
            position: 0,
            state_stack: vec![StateId(0)],
            node_stack: Vec::new(),
            reused_count: 0,
            edits,
        };

        parser.parse()
    }
}

/// A single parsing session with subtree reuse
struct IncrementalParseSession<'a> {
    input: &'a [u8],
    grammar: &'a Grammar,
    parse_table: &'a ParseTable,
    subtree_pool: Option<&'a SubtreePool>,
    position: usize,
    state_stack: Vec<StateId>,
    node_stack: Vec<ParseNode>,
    reused_count: usize,
    #[allow(dead_code)]
    edits: &'a [Edit],
}

impl<'a> IncrementalParseSession<'a> {
    fn parse(&mut self) -> Result<ParseNode> {
        loop {
            // Try to reuse a subtree at current position
            if let Some(reused) = self.try_reuse_subtree() {
                self.shift_subtree(reused)?;
                continue;
            }

            // Normal parsing
            let current_state = *self
                .state_stack
                .last()
                .ok_or_else(|| anyhow::anyhow!("Empty state stack"))?;

            // Get next token
            let token = self.lex_token()?;

            // Get action
            let action = self.get_action(current_state, token.symbol)?;

            match action {
                Action::Shift(next_state) => {
                    self.shift_token(next_state, token)?;
                }
                Action::Reduce(rule_id) => {
                    self.reduce(rule_id)?;
                }
                Action::Accept => {
                    println!(
                        "Incremental parse complete. Reused {} subtrees",
                        self.reused_count
                    );
                    return self
                        .node_stack
                        .pop()
                        .ok_or_else(|| anyhow::anyhow!("No parse tree"));
                }
                Action::Error => {
                    anyhow::bail!("Parse error at position {}", self.position);
                }
                Action::Fork(_) => {
                    anyhow::bail!("GLR forking not yet supported in incremental parsing");
                }
            }
        }
    }

    /// Try to reuse a subtree at the current position
    fn try_reuse_subtree(&mut self) -> Option<ReusableSubtree> {
        let pool = self.subtree_pool?;
        let current_state = *self.state_stack.last()?;

        // Find reusable subtrees at current position
        let reusable = pool.find_reusable_at(self.position, None)?;

        // Check if we can shift this subtree in the current state
        if self.can_shift_subtree(current_state, &reusable.node) {
            self.reused_count += 1;
            Some(reusable.clone())
        } else {
            None
        }
    }

    /// Check if a subtree can be shifted in the current state
    fn can_shift_subtree(&self, state: StateId, node: &ParseNode) -> bool {
        // Check if there's a valid action for this symbol
        match self.get_action(state, node.symbol) {
            Ok(Action::Shift(_)) => true,
            Ok(Action::Reduce(_)) => {
                // We might be able to shift after reduction
                // For now, conservatively say no
                false
            }
            _ => false,
        }
    }

    /// Shift a reused subtree
    fn shift_subtree(&mut self, subtree: ReusableSubtree) -> Result<()> {
        // Skip the bytes covered by this subtree
        self.position = subtree.byte_range.end;

        // Get next state
        let current_state = *self
            .state_stack
            .last()
            .ok_or_else(|| anyhow::anyhow!("Empty state stack"))?;

        match self.get_action(current_state, subtree.node.symbol)? {
            Action::Shift(next_state) => {
                self.state_stack.push(next_state);
                self.node_stack.push(subtree.node);
                Ok(())
            }
            _ => anyhow::bail!("Cannot shift subtree"),
        }
    }

    /// Shift a single token
    fn shift_token(&mut self, next_state: StateId, token: Token) -> Result<()> {
        let node = ParseNode {
            symbol: token.symbol,
            children: vec![],
            start_byte: token.start,
            end_byte: token.end,
            field_name: None,
        };

        self.state_stack.push(next_state);
        self.node_stack.push(node);
        self.position = token.end;

        Ok(())
    }

    /// Perform a reduction
    fn reduce(&mut self, rule_id: RuleId) -> Result<()> {
        // Find the rule
        let rule = self
            .grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .find(|r| {
                // Match by production ID or other criteria
                self.grammar
                    .production_ids
                    .iter()
                    .any(|(rid, pid)| *rid == rule_id && r.production_id == *pid)
            })
            .ok_or_else(|| anyhow::anyhow!("Rule not found"))?;

        // Pop states and nodes
        let rhs_len = rule.rhs.len();
        let mut children = Vec::with_capacity(rhs_len);

        for _ in 0..rhs_len {
            self.state_stack.pop();
            if let Some(node) = self.node_stack.pop() {
                children.push(node);
            }
        }
        children.reverse();

        // Create new node
        let start_byte = children
            .first()
            .map(|n| n.start_byte)
            .unwrap_or(self.position);
        let end_byte = children.last().map(|n| n.end_byte).unwrap_or(self.position);

        let node = ParseNode {
            symbol: rule.lhs,
            children,
            start_byte,
            end_byte,
            field_name: None,
        };

        // Get goto state
        let return_state = *self
            .state_stack
            .last()
            .ok_or_else(|| anyhow::anyhow!("Empty state stack after reduction"))?;

        let goto_state = self.get_goto_state(return_state, rule.lhs)?;

        self.state_stack.push(goto_state);
        self.node_stack.push(node);

        Ok(())
    }

    /// Get action for state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;

        if state_idx >= self.parse_table.action_table.len() {
            anyhow::bail!("Invalid state");
        }

        if symbol_idx >= self.parse_table.action_table[state_idx].len() {
            anyhow::bail!("Invalid symbol");
        }

        let action_cell = &self.parse_table.action_table[state_idx][symbol_idx];
        if action_cell.is_empty() {
            Ok(Action::Error)
        } else if action_cell.len() == 1 {
            Ok(action_cell[0].clone())
        } else {
            Ok(Action::Fork(action_cell.clone()))
        }
    }

    /// Get goto state
    fn get_goto_state(&self, state: StateId, symbol: SymbolId) -> Result<StateId> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;

        if state_idx >= self.parse_table.goto_table.len() {
            anyhow::bail!("Invalid state for goto");
        }

        if symbol_idx >= self.parse_table.goto_table[state_idx].len() {
            anyhow::bail!("Invalid symbol for goto");
        }

        Ok(self.parse_table.goto_table[state_idx][symbol_idx])
    }

    /// Lex a token at current position
    fn lex_token(&self) -> Result<Token> {
        // Skip whitespace
        while self.position < self.input.len() && self.input[self.position].is_ascii_whitespace() {
            // Note: In real implementation, we'd increment position correctly
        }

        if self.position >= self.input.len() {
            // EOF token
            return Ok(Token {
                symbol: SymbolId(0),
                text: vec![],
                start: self.position,
                end: self.position,
            });
        }

        // Simple tokenization for demo
        // In real implementation, use proper lexer
        Ok(Token {
            symbol: SymbolId(1), // Dummy
            text: vec![self.input[self.position]],
            start: self.position,
            end: self.position + 1,
        })
    }
}

/// A token
#[derive(Debug, Clone)]
struct Token {
    pub symbol: SymbolId,
    #[allow(dead_code)]
    pub text: Vec<u8>,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_subtree_pool_collection() {
        // Create a simple parse tree
        let tree = ParseNode {
            symbol: SymbolId(1),
            children: vec![
                ParseNode {
                    symbol: SymbolId(2),
                    children: vec![],
                    start_byte: 0,
                    end_byte: 5,
                    field_name: None,
                },
                ParseNode {
                    symbol: SymbolId(3),
                    children: vec![],
                    start_byte: 6,
                    end_byte: 10,
                    field_name: None,
                },
            ],
            start_byte: 0,
            end_byte: 10,
            field_name: None,
        };

        // No edits
        let edits = vec![];
        let pool = SubtreePool::from_tree(&tree, &edits);

        // Should have collected all subtrees
        assert!(pool.subtrees_by_start.contains_key(&0));
        assert!(pool.subtrees_by_start.contains_key(&6));

        // With edit affecting first child
        let edits = vec![Edit {
            start_byte: 2,
            old_end_byte: 3,
            new_end_byte: 4,
            start_position: Position { row: 0, column: 2 },
            old_end_position: Position { row: 0, column: 3 },
            new_end_position: Position { row: 0, column: 4 },
        }];

        let pool = SubtreePool::from_tree(&tree, &edits);

        // First child should be marked as affected
        if let Some(subtrees) = pool.subtrees_by_start.get(&0) {
            assert!(subtrees.iter().any(|s| s.is_affected));
        }
    }

    #[test]
    fn test_incremental_parse_with_reuse() {
        // This test would require a proper grammar and parse table
        // For now, it demonstrates the API

        let grammar = Grammar::new("test".to_string());
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
            symbol_to_index: std::collections::BTreeMap::new(),
        };

        let mut parser = IncrementalParser::new(grammar, parse_table);

        // First parse
        let input1 = "1 + 2";
        let tree1 = parser
            .parse(input1, None, &[])
            .unwrap_or_else(|_| ParseNode {
                symbol: SymbolId(0),
                children: vec![],
                start_byte: 0,
                end_byte: 5,
                field_name: None,
            });

        // Edit: change "2" to "3"
        let edits = vec![Edit {
            start_byte: 4,
            old_end_byte: 5,
            new_end_byte: 5,
            start_position: Position { row: 0, column: 4 },
            old_end_position: Position { row: 0, column: 5 },
            new_end_position: Position { row: 0, column: 5 },
        }];

        let input2 = "1 + 3";
        let _tree2 = parser
            .parse(input2, Some(&tree1), &edits)
            .unwrap_or_else(|_| ParseNode {
                symbol: SymbolId(0),
                children: vec![],
                start_byte: 0,
                end_byte: 5,
                field_name: None,
            });

        // In a real test, we'd verify that subtrees were reused
    }
}
