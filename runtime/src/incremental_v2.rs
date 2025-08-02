// Enhanced incremental parsing with subtree reuse
// This module provides efficient reparsing by reusing unchanged subtrees

use crate::parser_v2::{ParseError, ParseNode, Token};
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::HashMap;
use std::ops::Range;

/// Edit operation on source text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Position,
    pub old_end_position: Position,
    pub new_end_position: Position,
}

/// Position in source text (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn new(row: usize, column: usize) -> Self {
        Position { row, column }
    }
}

/// A reusable subtree from a previous parse
#[derive(Debug, Clone)]
pub struct ReusableNode {
    pub node: ParseNode,
    pub byte_range: Range<usize>,
    pub token_range: Range<usize>,
    pub is_error: bool,
    pub has_changes: bool,
}

/// Tracks which subtrees can be reused
pub struct SubtreePool {
    nodes: Vec<ReusableNode>,
    by_range: HashMap<Range<usize>, usize>, // byte_range -> node index
}

impl SubtreePool {
    pub fn new() -> Self {
        SubtreePool {
            nodes: Vec::new(),
            by_range: HashMap::new(),
        }
    }

    /// Build pool from a parse tree
    pub fn build_from_tree(root: &ParseNode, tokens: &[Token]) -> Self {
        let mut pool = SubtreePool::new();
        pool.collect_subtrees(root, tokens, 0, 0);
        pool
    }

    fn collect_subtrees(
        &mut self,
        node: &ParseNode,
        tokens: &[Token],
        mut byte_offset: usize,
        mut token_offset: usize,
    ) -> (usize, usize) {
        let start_byte = byte_offset;
        let start_token = token_offset;

        if node.children.is_empty() {
            // Leaf node
            if token_offset < tokens.len() {
                let token = &tokens[token_offset];
                byte_offset += token.text.len();
                token_offset += 1;
            }
        } else {
            // Internal node
            for child in &node.children {
                let (new_byte, new_token) =
                    self.collect_subtrees(child, tokens, byte_offset, token_offset);
                byte_offset = new_byte;
                token_offset = new_token;
            }
        }

        let byte_range = start_byte..byte_offset;
        let token_range = start_token..token_offset;

        // Store reusable node
        let reusable = ReusableNode {
            node: node.clone(),
            byte_range: byte_range.clone(),
            token_range,
            is_error: false,
            has_changes: false,
        };

        let index = self.nodes.len();
        self.nodes.push(reusable);
        self.by_range.insert(byte_range, index);

        (byte_offset, token_offset)
    }

    /// Find reusable subtrees that don't overlap with edits
    pub fn find_reusable(&self, edits: &[Edit]) -> Vec<&ReusableNode> {
        self.nodes
            .iter()
            .filter(|node| {
                !edits.iter().any(|edit| {
                    // Check if node overlaps with edit
                    node.byte_range.end > edit.start_byte
                        && node.byte_range.start < edit.old_end_byte
                })
            })
            .collect()
    }

    /// Get a reusable node by byte range
    pub fn get_by_range(&self, range: &Range<usize>) -> Option<&ReusableNode> {
        self.by_range
            .get(range)
            .and_then(|&idx| self.nodes.get(idx))
    }
}

/// Incremental parser with subtree reuse
pub struct IncrementalParserV2 {
    grammar: Grammar,
    table: ParseTable,
    subtree_pool: SubtreePool,
}

impl IncrementalParserV2 {
    pub fn new(grammar: Grammar, table: ParseTable) -> Self {
        IncrementalParserV2 {
            grammar,
            table,
            subtree_pool: SubtreePool::new(),
        }
    }

    /// Parse incrementally, reusing subtrees from previous parse
    pub fn parse(
        &mut self,
        tokens: Vec<Token>,
        old_tree: Option<&ParseNode>,
        edits: &[Edit],
    ) -> Result<ParseNode, ParseError> {
        // Build subtree pool from old tree
        if let Some(tree) = old_tree {
            self.subtree_pool = SubtreePool::build_from_tree(tree, &tokens);
        }

        // Find reusable subtrees
        let reusable = self.subtree_pool.find_reusable(edits);

        // Parse with subtree reuse
        let mut parser = IncrementalParserState::new(&self.grammar, &self.table, tokens, reusable);

        parser.parse()
    }
}

/// Parser state that can reuse subtrees
struct IncrementalParserState<'a> {
    grammar: &'a Grammar,
    table: &'a ParseTable,
    tokens: Vec<Token>,
    #[allow(dead_code)]
    reusable: Vec<&'a ReusableNode>,
    position: usize,
    stack: Vec<(StateId, Option<ParseNode>)>,
    reuse_queue: Vec<(usize, &'a ReusableNode)>, // (start_position, node)
}

impl<'a> IncrementalParserState<'a> {
    fn new(
        grammar: &'a Grammar,
        table: &'a ParseTable,
        tokens: Vec<Token>,
        reusable: Vec<&'a ReusableNode>,
    ) -> Self {
        // Sort reusable nodes by start position
        let mut reuse_queue: Vec<_> = reusable
            .into_iter()
            .map(|node| (node.token_range.start, node))
            .collect();
        reuse_queue.sort_by_key(|(pos, _)| *pos);

        IncrementalParserState {
            grammar,
            table,
            tokens,
            reusable: vec![],
            position: 0,
            stack: vec![(StateId(0), None)],
            reuse_queue,
        }
    }

    fn parse(&mut self) -> Result<ParseNode, ParseError> {
        while self.position < self.tokens.len() {
            // Check if we can reuse a subtree at current position
            if let Some(reusable) = self.try_reuse_subtree() {
                self.shift_subtree(reusable);
                continue;
            }

            // Normal parsing
            let token = &self.tokens[self.position];
            let (state, _) = self.stack.last().unwrap();

            // Get action for current state and token
            if let Some(actions) = self.table.action_table.get(state.0 as usize) {
                if let Some(action) = actions.get(token.symbol.0 as usize) {
                    match action {
                        Action::Shift(next_state) => {
                            let node = ParseNode {
                                symbol: token.symbol,
                                rule_id: None,
                                children: vec![],
                                start_byte: token.start,
                                end_byte: token.end,
                                text: Some(token.text.clone()),
                            };
                            self.stack.push((*next_state, Some(node)));
                            self.position += 1;
                        }
                        Action::Reduce(rule_id) => {
                            self.reduce(*rule_id)?;
                        }
                        Action::Accept => {
                            if let Some((_, Some(node))) = self.stack.pop() {
                                return Ok(node);
                            }
                        }
                        Action::Error => {
                            return Err(ParseError::UnexpectedToken {
                                expected: vec![],
                                found: token.symbol,
                                position: self.position,
                            });
                        }
                        Action::Fork(_) => {
                            // TODO: Implement GLR fork handling
                            return Err(ParseError::UnexpectedToken {
                                expected: vec![],
                                found: token.symbol,
                                position: self.position,
                            });
                        }
                    }
                }
            }
        }

        // End of input - check for accept
        if let Some((_, Some(node))) = self.stack.pop() {
            Ok(node)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: vec![],
                found: SymbolId(0),
                position: self.position,
            })
        }
    }

    fn try_reuse_subtree(&mut self) -> Option<&'a ReusableNode> {
        // Check if there's a reusable subtree at current position
        while let Some(&(start_pos, node)) = self.reuse_queue.first() {
            if start_pos == self.position {
                self.reuse_queue.remove(0);

                // Verify the subtree is still valid in current context
                if self.can_reuse_subtree(node) {
                    return Some(node);
                }
            } else if start_pos > self.position {
                break;
            } else {
                // Skip outdated entries
                self.reuse_queue.remove(0);
            }
        }
        None
    }

    fn can_reuse_subtree(&self, node: &ReusableNode) -> bool {
        // Check if the subtree's tokens match current position
        let end_pos = self.position + (node.token_range.end - node.token_range.start);
        if end_pos > self.tokens.len() {
            return false;
        }

        // Verify token match
        let current_tokens = &self.tokens[self.position..end_pos];
        let expected_len = node.token_range.end - node.token_range.start;

        current_tokens.len() == expected_len
    }

    fn shift_subtree(&mut self, reusable: &'a ReusableNode) {
        // Skip tokens covered by the reusable subtree
        let token_count = reusable.token_range.end - reusable.token_range.start;
        self.position += token_count;

        // Push the reused subtree onto the stack
        let (state, _) = self.stack.last().unwrap();

        // Get next state after shifting this symbol
        if let Some(gotos) = self.table.goto_table.get(state.0 as usize) {
            if let Some(&goto_state) = gotos.get(reusable.node.symbol.0 as usize) {
                self.stack.push((goto_state, Some(reusable.node.clone())));
            }
        }
    }

    fn reduce(&mut self, rule_id: RuleId) -> Result<(), ParseError> {
        // Find the rule
        let rule = self
            .grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .find(|r| r.production_id.0 == rule_id.0)
            .ok_or(ParseError::UnexpectedToken {
                expected: vec![],
                found: SymbolId(0),
                position: self.position,
            })?;

        let rhs_len = rule.rhs.len();
        let mut children = Vec::with_capacity(rhs_len);

        // Pop RHS symbols from stack
        for _ in 0..rhs_len {
            if let Some((_, Some(node))) = self.stack.pop() {
                children.push(node);
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: vec![],
                    found: SymbolId(0),
                    position: self.position,
                });
            }
        }
        children.reverse();

        // Create new node
        let start_byte = children.first().map(|n| n.start_byte).unwrap_or(0);
        let end_byte = children.last().map(|n| n.end_byte).unwrap_or(0);

        let node = ParseNode {
            symbol: rule.lhs,
            rule_id: Some(rule_id),
            children,
            start_byte,
            end_byte,
            text: None,
        };

        // Get goto state
        let (state, _) = self.stack.last().unwrap();
        if let Some(gotos) = self.table.goto_table.get(state.0 as usize) {
            if let Some(&goto_state) = gotos.get(rule.lhs.0 as usize) {
                self.stack.push((goto_state, Some(node)));
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: vec![],
                    found: rule.lhs,
                    position: self.position,
                })
            }
        } else {
            Err(ParseError::UnexpectedToken {
                expected: vec![],
                found: rule.lhs,
                position: self.position,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtree_pool_construction() {
        let root = ParseNode {
            symbol: SymbolId(0),
            rule_id: None,
            children: vec![
                ParseNode {
                    symbol: SymbolId(1),
                    rule_id: None,
                    children: vec![],
                    start_byte: 0,
                    end_byte: 5,
                    text: Some(b"hello".to_vec()),
                },
                ParseNode {
                    symbol: SymbolId(2),
                    rule_id: None,
                    children: vec![],
                    start_byte: 6,
                    end_byte: 11,
                    text: Some(b"world".to_vec()),
                },
            ],
            start_byte: 0,
            end_byte: 11,
            text: None,
        };

        let tokens = vec![
            Token {
                symbol: SymbolId(1),
                text: b"hello".to_vec(),
                start: 0,
                end: 5,
            },
            Token {
                symbol: SymbolId(2),
                text: b" ".to_vec(),
                start: 5,
                end: 6,
            },
            Token {
                symbol: SymbolId(3),
                text: b"world".to_vec(),
                start: 6,
                end: 11,
            },
        ];

        let pool = SubtreePool::build_from_tree(&root, &tokens);
        assert!(!pool.nodes.is_empty());
    }

    #[test]
    fn test_edit_filtering() {
        let node1 = ReusableNode {
            node: ParseNode {
                symbol: SymbolId(1),
                rule_id: None,
                children: vec![],
                start_byte: 0,
                end_byte: 5,
                text: Some(b"hello".to_vec()),
            },
            byte_range: 0..5,
            token_range: 0..1,
            is_error: false,
            has_changes: false,
        };

        let node2 = ReusableNode {
            node: ParseNode {
                symbol: SymbolId(2),
                rule_id: None,
                children: vec![],
                start_byte: 10,
                end_byte: 15,
                text: Some(b"world".to_vec()),
            },
            byte_range: 10..15,
            token_range: 2..3,
            is_error: false,
            has_changes: false,
        };

        let mut pool = SubtreePool::new();
        pool.nodes.push(node1);
        pool.nodes.push(node2);

        // Edit in the middle (5..10)
        let edits = vec![Edit {
            start_byte: 5,
            old_end_byte: 10,
            new_end_byte: 8,
            start_position: Position::new(0, 5),
            old_end_position: Position::new(0, 10),
            new_end_position: Position::new(0, 8),
        }];

        let reusable = pool.find_reusable(&edits);
        assert_eq!(reusable.len(), 2); // Both nodes should be reusable
    }
}
