//! # DO NOT USE THIS MODULE!
//!
//! This module contains functions for use in the expanded macros produced by rust-sitter.
//! They need to be public so they can be accessed at all (\*cough\* macro hygiene), but
//! they are not intended to actually be called in any other circumstance.

use crate::Extract;

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

    #[allow(dead_code)]
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
    _field_name: &str,
    closure_ref: Option<&LT::LeafFn>,
) -> T {
    if let Some(cursor) = cursor_opt.as_mut() {
        // Since field names are not available in pure-rust parser,
        // we extract from the current child and advance the cursor
        let n = cursor.node();

        // Check if we're dealing with a node that has no children
        // This happens when a struct has a single leaf field - the node IS the field value
        if n.children.is_empty() && cursor.current_index == 0 {
            // eprintln!("  Special case: node has no children, likely a single-field struct");
            // The parent node itself contains the field value
            // Don't advance cursor since there are no siblings
            let parent_node = cursor.node;
            let end_byte = parent_node.end_byte;
            *cursor_opt = None;
            *last_idx = end_byte;
            return LT::extract(Some(parent_node), source, *last_idx, closure_ref);
        }

        let out = LT::extract(Some(n), source, *last_idx, closure_ref);

        if !cursor.goto_next_sibling() {
            *cursor_opt = None;
        }

        *last_idx = n.end_byte;

        out
    } else {
        // eprintln!("DEBUG extract_field: No cursor for field '{}'", _field_name);
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
                                format!("symbol {} (public {} out of bounds)", e.found, public_symbol)
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
    _input: &str,
    _language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    // TODO: Implement GLR parsing using parser_v4
    //
    // Implementation plan:
    // 1. Deserialize Grammar from T::GRAMMAR_JSON
    // 2. Construct ParseTable from generated static data
    // 3. Create parser_v4::Parser instance
    // 4. Parse input to get parse tree
    // 5. Extract typed AST using T::extract()
    //
    // For now, we return an error indicating GLR is not yet fully implemented
    Err(vec![crate::errors::ParseError {
        reason: crate::errors::ParseErrorReason::UnexpectedToken(
            "GLR parser integration not yet complete (Step 3 in progress)".to_string(),
        ),
        start: 0,
        end: 0,
    }])
}

/// Parse using the GLR parser (stub for when feature is not enabled)
#[cfg(all(feature = "pure-rust", not(feature = "glr")))]
fn parse_with_glr<T: Extract<T>>(
    _input: &str,
    _language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    unreachable!("GLR parser should not be called when glr feature is disabled")
}
