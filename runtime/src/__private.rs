//! # DO NOT USE THIS MODULE!
//!
//! This module contains functions for use in the expanded macros produced by rust-sitter.
//! They need to be public so they can be accessed at all (\*cough\* macro hygiene), but
//! they are not intended to actually be called in any other circumstance.

use crate::Extract;

#[cfg(not(feature = "pure-rust"))]
use crate::tree_sitter;
#[cfg(feature = "pure-rust")]
use crate::pure_parser::ParsedNode;

#[cfg(feature = "pure-rust")]
/// A cursor for navigating parsed nodes in pure-rust mode
pub struct TreeCursor<'a> {
    node: &'a ParsedNode,
    children: &'a [ParsedNode],
    current_index: usize,
}

#[cfg(feature = "pure-rust")]
impl<'a> TreeCursor<'a> {
    pub fn new(node: &'a ParsedNode) -> Self {
        Self {
            node,
            children: &node.children,
            current_index: 0,
        }
    }
    
    pub fn goto_first_child(&mut self) -> bool {
        if !self.children.is_empty() {
            self.current_index = 0;
            true
        } else {
            false
        }
    }
    
    pub fn goto_next_sibling(&mut self) -> bool {
        if self.current_index + 1 < self.children.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }
    
    pub fn node(&self) -> &'a ParsedNode {
        if self.current_index < self.children.len() {
            &self.children[self.current_index]
        } else {
            self.node
        }
    }
    
    fn field_name(&self) -> Option<&str> {
        // TODO: Implement field names
        None
    }
}

#[cfg(not(feature = "pure-rust"))]
pub fn extract_struct_or_variant<T>(
    node: tree_sitter::Node,
    construct_expr: impl Fn(&mut Option<tree_sitter::TreeCursor>, &mut usize) -> T,
) -> T {
    let mut parent_cursor = node.walk();
    construct_expr(
        &mut if parent_cursor.goto_first_child() {
            Some(parent_cursor)
        } else {
            None
        },
        &mut node.start_byte(),
    )
}

#[cfg(feature = "pure-rust")]
pub fn extract_struct_or_variant<T>(
    node: &ParsedNode,
    construct_expr: impl Fn(&mut Option<TreeCursor>, &mut usize) -> T,
) -> T {
    let mut cursor = TreeCursor::new(node);
    let mut cursor_opt = if cursor.goto_first_child() {
        Some(cursor)
    } else {
        None
    };
    let mut start_byte = node.start_byte;
    construct_expr(&mut cursor_opt, &mut start_byte)
}

#[cfg(not(feature = "pure-rust"))]
pub fn extract_field<LT: Extract<T>, T>(
    cursor_opt: &mut Option<tree_sitter::TreeCursor>,
    source: &[u8],
    last_idx: &mut usize,
    field_name: &str,
    closure_ref: Option<&LT::LeafFn>,
) -> T {
    if let Some(cursor) = cursor_opt.as_mut() {
        loop {
            let n = cursor.node();
            if let Some(name) = cursor.field_name() {
                if name == field_name {
                    let out = LT::extract(Some(&n), source, *last_idx, closure_ref);

                    if !cursor.goto_next_sibling() {
                        *cursor_opt = None;
                    };

                    *last_idx = n.end_byte();

                    return out;
                } else {
                    return LT::extract(None, source, *last_idx, closure_ref);
                }
            } else {
                *last_idx = n.end_byte();
            }

            if !cursor.goto_next_sibling() {
                return LT::extract(None, source, *last_idx, closure_ref);
            }
        }
    } else {
        LT::extract(None, source, *last_idx, closure_ref)
    }
}

#[cfg(feature = "pure-rust")]
pub fn extract_field<LT: Extract<T>, T>(
    cursor_opt: &mut Option<TreeCursor>,
    source: &[u8],
    last_idx: &mut usize,
    field_name: &str,
    closure_ref: Option<&LT::LeafFn>,
) -> T {
    if let Some(cursor) = cursor_opt.as_mut() {
        loop {
            let n = cursor.node();
            // TODO: Field names are not yet supported in pure-rust parser
            // For now, we'll just get the next child
            let out = LT::extract(Some(n), source, *last_idx, closure_ref);
            
            if !cursor.goto_next_sibling() {
                *cursor_opt = None;
            }
            
            *last_idx = n.end_byte;
            return out;
        }
    } else {
        LT::extract(None, source, *last_idx, closure_ref)
    }
}

#[cfg(not(feature = "pure-rust"))]
pub fn parse<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> tree_sitter::Language,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    let mut parser = crate::tree_sitter::Parser::new();
    parser.set_language(&language()).unwrap();
    let tree = parser.parse(input, None).unwrap();
    let root_node = tree.root_node();

    if root_node.has_error() {
        let mut errors = vec![];
        crate::errors::collect_parsing_errors(&root_node, input.as_bytes(), &mut errors);

        Err(errors)
    } else {
        Ok(<T as crate::Extract<_>>::extract(
            Some(root_node),
            input.as_bytes(),
            0,
            None,
        ))
    }
}

#[cfg(feature = "pure-rust")]
pub fn parse<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    let mut parser = crate::pure_parser::Parser::new();
    parser.set_language(language()).unwrap();
    let parse_result = parser.parse_string(input);
    let root_node = match parse_result.root {
        Some(root) => root,
        None => {
            // Convert pure_parser::ParseError to errors::ParseError
            let errors = parse_result.errors.into_iter().map(|e| {
                crate::errors::ParseError {
                    reason: crate::errors::ParseErrorReason::UnexpectedToken(format!("symbol {}", e.found)),
                    start: e.position,
                    end: e.position,
                }
            }).collect();
            return Err(errors);
        }
    };

    if root_node.has_error() {
        let mut errors = vec![];
        crate::errors::collect_parsing_errors(&root_node, input.as_bytes(), &mut errors);

        Err(errors)
    } else {
        Ok(<T as crate::Extract<_>>::extract(
            Some(&root_node),
            input.as_bytes(),
            0,
            None,
        ))
    }
}
