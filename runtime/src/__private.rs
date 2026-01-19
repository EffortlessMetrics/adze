//! # DO NOT USE THIS MODULE!
//!
//! This module contains functions for use in the expanded macros produced by rust-sitter.
//! They need to be public so they can be accessed at all (\*cough\* macro hygiene), but
//! they are not intended to actually be called in any other circumstance.

use crate::Extract;
#[cfg(feature = "pure-rust")]
use core::ffi::{CStr, c_char};

#[cfg(feature = "pure-rust")]
use crate::pure_parser::ParsedNode;
#[cfg(not(feature = "pure-rust"))]
use crate::tree_sitter;

#[cfg(feature = "pure-rust")]
/// A cursor for navigating parsed nodes in pure-rust mode
pub struct TreeCursor<'a> {
    node: &'a ParsedNode,
    children: &'a [ParsedNode],
    current_index: usize,
}

#[cfg(feature = "pure-rust")]
impl<'a> TreeCursor<'a> {
    /// Creates a new tree cursor for the given node.
    pub fn new(node: &'a ParsedNode) -> Self {
        Self {
            node,
            children: &node.children,
            current_index: 0,
        }
    }

    /// Moves the cursor to the first child node.
    pub fn goto_first_child(&mut self) -> bool {
        if !self.children.is_empty() {
            self.current_index = 0;
            true
        } else {
            false
        }
    }

    /// Moves the cursor to the next sibling node.
    pub fn goto_next_sibling(&mut self) -> bool {
        if self.current_index + 1 < self.children.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Returns the current node.
    pub fn node(&self) -> &'a ParsedNode {
        if self.current_index < self.children.len() {
            &self.children[self.current_index]
        } else {
            self.node
        }
    }

    /// Returns the field name for the current child node if available
    pub fn field_name(&self) -> Option<&'static str> {
        if self.current_index >= self.children.len() {
            return None;
        }
        let child = &self.children[self.current_index];
        let field_id = child.field_id?;
        let lang_ptr = self.node.language?;
        unsafe {
            let lang = &*lang_ptr;
            if field_id >= lang.field_count as u16 {
                return None;
            }
            if lang.field_names.is_null() {
                return None;
            }
            let field_names =
                core::slice::from_raw_parts(lang.field_names, lang.field_count as usize);
            let name_ptr = field_names[field_id as usize];
            if name_ptr.is_null() {
                return None;
            }
            CStr::from_ptr(name_ptr as *const c_char).to_str().ok()
        }
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

/// Extracts a struct or variant from a parsed node.
#[cfg(feature = "pure-rust")]
pub fn extract_struct_or_variant<T>(
    node: &ParsedNode,
    construct_expr: impl Fn(&mut Option<TreeCursor>, &mut usize) -> T,
) -> T {
    // Debug output commented out
    // eprintln!("DEBUG extract_struct_or_variant: node.symbol={}, children={}", node.symbol, node.children.len());
    // for (i, child) in node.children.iter().enumerate() {
    //     eprintln!("  child[{}]: symbol={}, field_name={:?}", i, child.symbol, child.field_name);
    // }

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

/// Extracts a field from the current position in the tree.
#[cfg(feature = "pure-rust")]
pub fn extract_field<LT: Extract<T>, T>(
    cursor_opt: &mut Option<TreeCursor>,
    source: &[u8],
    last_idx: &mut usize,
    field_name: &str,
    closure_ref: Option<&LT::LeafFn>,
) -> T {
    if let Some(cursor) = cursor_opt.as_mut() {
        // Handle special case where a node has no children and represents a single-field struct
        let n = cursor.node();
        if n.children.is_empty() && cursor.current_index == 0 {
            let parent_node = cursor.node;
            let end_byte = parent_node.end_byte;
            *cursor_opt = None;
            *last_idx = end_byte;
            return LT::extract(Some(parent_node), source, *last_idx, closure_ref);
        }

        loop {
            let n = cursor.node();
            if let Some(name) = cursor.field_name() {
                if name == field_name {
                    let out = LT::extract(Some(n), source, *last_idx, closure_ref);

                    if !cursor.goto_next_sibling() {
                        *cursor_opt = None;
                    }

                    *last_idx = n.end_byte;

                    return out;
                } else {
                    return LT::extract(None, source, *last_idx, closure_ref);
                }
            } else {
                *last_idx = n.end_byte;
            }

            if !cursor.goto_next_sibling() {
                return LT::extract(None, source, *last_idx, closure_ref);
            }
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

/// Parses an input string and extracts a value using the pure-rust parser.
#[cfg(feature = "pure-rust")]
pub fn parse<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    // Select parser backend based on feature flags
    use crate::parser_selection::ParserBackend;
    let backend = ParserBackend::select(T::HAS_CONFLICTS);

    match backend {
        ParserBackend::GLR => {
            // GLR parser path (parser_v4)
            parse_with_glr::<T>(input, language)
        }
        ParserBackend::PureRust => {
            // Simple LR parser path (pure_parser)
            parse_with_pure_parser::<T>(input, language)
        }
        ParserBackend::TreeSitter => {
            // This shouldn't happen with pure-rust feature, but handle gracefully
            unreachable!("TreeSitter backend selected with pure-rust feature enabled")
        }
    }
}

/// Parse using the simple LR parser (pure_parser)
#[cfg(feature = "pure-rust")]
fn parse_with_pure_parser<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    let mut parser = crate::pure_parser::Parser::new();
    let lang = language();
    parser.set_language(lang).unwrap();
    let parse_result = parser.parse_string(input);
    let root_node = match parse_result.root {
        Some(root) => root,
        None => {
            // Convert pure_parser::ParseError to errors::ParseError
            let lang = language();
            let errors = parse_result
                .errors
                .into_iter()
                .map(|e| {
                    // Get symbol name from language if available
                    // The public_symbol_map maps internal symbol to public symbol
                    let symbol_name = if (e.found as usize) < lang.symbol_count as usize {
                        unsafe {
                            // Use public_symbol_map to get the public symbol ID
                            let public_symbol = if !lang.public_symbol_map.is_null() {
                                *lang.public_symbol_map.add(e.found as usize)
                            } else {
                                e.found
                            };

                            // Now use public symbol to index into symbol_names
                            if (public_symbol as usize) < lang.symbol_count as usize {
                                let symbol_ptr = *lang.symbol_names.add(public_symbol as usize);
                                if !symbol_ptr.is_null() {
                                    std::ffi::CStr::from_ptr(symbol_ptr as *const i8)
                                        .to_string_lossy()
                                        .to_string()
                                } else {
                                    format!("symbol {} (public {})", e.found, public_symbol)
                                }
                            } else {
                                format!(
                                    "symbol {} (public {} out of bounds)",
                                    e.found, public_symbol
                                )
                            }
                        }
                    } else {
                        format!("symbol {} (out of bounds)", e.found)
                    };
                    crate::errors::ParseError {
                        reason: crate::errors::ParseErrorReason::UnexpectedToken(symbol_name),
                        start: e.position,
                        end: e.position,
                    }
                })
                .collect();
            return Err(errors);
        }
    };

    if root_node.has_error() {
        let mut errors = vec![];
        crate::errors::collect_parsing_errors(&root_node, input.as_bytes(), &mut errors);

        Err(errors)
    } else {
        // Check if the root node is source_file wrapper
        // In the augmented grammar, we have S' -> source_file -> actual_language_root
        // source_file is typically a wrapper node with a single child
        let extract_node = if root_node.kind() == "source_file" && root_node.children.len() == 1 {
            // This is source_file, extract from its first child
            &root_node.children[0]
        } else {
            // Extract from root directly
            &root_node
        };

        Ok(<T as crate::Extract<_>>::extract(
            Some(extract_node),
            input.as_bytes(),
            0,
            None,
        ))
    }
}

/// Parse using the GLR parser (parser_v4)
#[cfg(feature = "glr")]
fn parse_with_glr<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    // GLR Parser Integration (In Progress)
    //
    // Current Status:
    // ✅ parser_v4 module exists with GLR fork/merge logic
    // ✅ parser_v4::from_language() can load from TSLanguage structs
    // ✅ parser_v4::parse() executes GLR parsing algorithm
    // ❌ parser_v4::parse() returns minimal Tree struct, not parse nodes
    // ❌ No conversion from parser_v4::ParseNode to pure_parser::ParsedNode
    //
    // Blocking Issue:
    // parser_v4::parse() returns Tree { root_kind, error_count, source }
    // but we need the actual ParseNode tree for T::extract().
    //
    // ✅ IMPLEMENTED: Option B - Added parser_v4::parse_tree() method
    use crate::parser_v4::Parser;

    // Get the language
    let lang = language();

    // Create parser from TSLanguage with the correct grammar name for external scanner lookup
    let mut parser = Parser::from_language(lang, T::GRAMMAR_NAME.to_string());

    // Parse to get root ParseNode
    let root_node = parser.parse_tree(input).map_err(|e| {
        vec![crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(e.to_string()),
            start: 0,
            end: 0,
        }]
    })?;

    // Convert parser_v4::ParseNode to pure_parser::ParsedNode
    let parsed_node = convert_parse_node_v4_to_pure(&root_node, lang);

    // Extract typed AST using the Extract trait
    Ok(<T as crate::Extract<_>>::extract(
        Some(&parsed_node),
        input.as_bytes(),
        0,
        None,
    ))
}

/// Convert parser_v4::ParseNode to pure_parser::ParsedNode
#[cfg(feature = "glr")]
fn convert_parse_node_v4_to_pure(
    node: &crate::parser_v4::ParseNode,
    lang: &crate::pure_parser::TSLanguage,
) -> crate::pure_parser::ParsedNode {
    // Recursively convert children
    let children = node
        .children
        .iter()
        .map(|child| convert_parse_node_v4_to_pure(child, lang))
        .collect();

    // Read symbol metadata from TSLanguage
    // Safety: runtime guarantees symbol < symbol_count when building nodes
    let (is_named, is_extra) = unsafe {
        if !lang.symbol_metadata.is_null() && (node.symbol.0 as u32) < lang.symbol_count {
            let metadata = *lang.symbol_metadata.add(node.symbol.0 as usize);
            // Tree-sitter metadata encoding:
            // Bit 0 (0x01): visible
            // Bit 1 (0x02): named
            // Bit 2 (0x04): extra
            // Bit 3 (0x08): supertype
            let is_named = (metadata & 0x02) != 0;
            let is_extra = (metadata & 0x04) != 0;
            (is_named, is_extra)
        } else {
            // Fallback if metadata unavailable
            (true, false)
        }
    };

    crate::pure_parser::ParsedNode {
        symbol: node.symbol.0, // SymbolId.0 -> TSSymbol
        children,
        start_byte: node.start_byte,
        end_byte: node.end_byte,
        // NOTE: Points are currently stubbed for the GLR path.
        // They are not used by any public API on GLR-generated trees.
        // See docs/plans/GLR_RUNTIME_WIRING_PLAN.md for tracking.
        start_point: crate::pure_parser::Point { row: 0, column: 0 },
        end_point: crate::pure_parser::Point { row: 0, column: 0 },
        is_extra,
        is_error: node.symbol.0 == 0, // Symbol 0 typically indicates error
        is_missing: false,
        is_named,
        field_id: None, // Field ID not yet propagated from GLR parser
        language: Some(lang as *const _),
    }
}

/// Parse using the GLR parser (stub for when feature is not enabled)
#[cfg(all(feature = "pure-rust", not(feature = "glr")))]
fn parse_with_glr<T: Extract<T>>(
    _input: &str,
    _language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    unreachable!("GLR parser should not be called when glr feature is disabled")
}
