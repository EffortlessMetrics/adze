//! # DO NOT USE THIS MODULE!
//!
//! This module contains functions for use in the expanded macros produced by adze.
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
    let has_child = parent_cursor.goto_first_child();

    let mut cursor_opt = if has_child { Some(parent_cursor) } else { None };

    // If the node has only one child and it's a wrapper, we might need to go deeper
    // But Tree-sitter cursors usually point to the immediate children.
    // The issue is likely that 'Program' kind is being matched instead of its fields.

    construct_expr(&mut cursor_opt, &mut node.start_byte())
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
            let name = cursor.field_name();
            if name == Some(field_name) || (name.is_none() && n.kind() == field_name) {
                let out = LT::extract(Some(n), source, *last_idx, closure_ref);

                if !cursor.goto_next_sibling() {
                    *cursor_opt = None;
                };

                *last_idx = n.end_byte();

                return out;
            } else if name.is_some() {
                return LT::extract(None, source, *last_idx, closure_ref);
            } else {
                // If it's an anonymous node, skip it and continue
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
            *cursor_opt = None;
            *last_idx = n.end_byte;
            return LT::extract(Some(n), source, *last_idx, closure_ref);
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
    let backend = crate::parser_selection::current_backend_for(T::HAS_CONFLICTS);

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

    if !parse_result.errors.is_empty() {
        let errors = parse_result
            .errors
            .into_iter()
            .map(|e| {
                // Get symbol name from language if available
                let symbol_name = if (e.found as usize) < lang.symbol_count as usize {
                    unsafe {
                        let public_symbol = if !lang.public_symbol_map.is_null() {
                            *lang.public_symbol_map.add(e.found as usize)
                        } else {
                            e.found
                        };

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

    let root_node = parse_result
        .root
        .expect("Root node should be present if no errors");

    // Check if the root node is source_file wrapper
    // In the augmented grammar, we have S' -> source_file -> actual_language_root
    // source_file is typically a wrapper node with a single non-extra child
    let non_extra_root_children: Vec<_> =
        root_node.children.iter().filter(|c| !c.is_extra).collect();
    let extract_node = if root_node.kind() == "source_file" && non_extra_root_children.len() == 1 {
        // This is source_file, extract from its first non-extra child
        non_extra_root_children[0]
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
        field_id: None, // TODO: Convert field_name to field_id using language field_names
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

#[cfg(all(test, feature = "pure-rust"))]
mod tests {
    use super::*;
    use crate::pure_parser::{
        ExternalScanner, ParsedNode, Point, TSLanguage, TSLexState, TSParseAction, TSRule,
    };
    use core::ptr;

    static FIELD_NAME_VALUE: &[u8] = b"value\0";
    static FIELD_NAME_NAME: &[u8] = b"name\0";

    #[repr(transparent)]
    struct FieldNames([*const u8; 2]);
    unsafe impl Sync for FieldNames {}

    static FIELD_NAMES: FieldNames =
        FieldNames([FIELD_NAME_VALUE.as_ptr(), FIELD_NAME_NAME.as_ptr()]);
    static LEX_MODES: [TSLexState; 1] = [TSLexState {
        lex_state: 0,
        external_lex_state: 0,
    }];

    static FIELD_LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 0,
        alias_count: 0,
        token_count: 0,
        external_token_count: 0,
        state_count: 0,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 2,
        max_alias_sequence_length: 0,
        production_id_map: ptr::null(),
        parse_table: ptr::null(),
        small_parse_table: ptr::null(),
        small_parse_table_map: ptr::null(),
        parse_actions: ptr::null::<TSParseAction>(),
        symbol_names: ptr::null(),
        field_names: FIELD_NAMES.0.as_ptr(),
        field_map_slices: ptr::null(),
        field_map_entries: ptr::null(),
        symbol_metadata: ptr::null(),
        public_symbol_map: ptr::null(),
        alias_map: ptr::null(),
        alias_sequences: ptr::null(),
        lex_modes: LEX_MODES.as_ptr(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner::default(),
        primary_state_ids: ptr::null(),
        production_lhs_index: ptr::null(),
        production_count: 0,
        eof_symbol: 0,
        rules: ptr::null::<TSRule>(),
        rule_count: 0,
    };

    fn node(
        symbol: u16,
        start: usize,
        end: usize,
        field_id: Option<u16>,
        children: Vec<ParsedNode>,
    ) -> ParsedNode {
        ParsedNode {
            symbol,
            children,
            start_byte: start,
            end_byte: end,
            start_point: Point {
                row: 0,
                column: start as u32,
            },
            end_point: Point {
                row: 0,
                column: end as u32,
            },
            is_extra: false,
            is_error: false,
            is_missing: false,
            is_named: true,
            field_id,
            language: None,
        }
    }

    #[test]
    fn given_parent_with_children_when_extracting_struct_then_cursor_starts_at_first_child() {
        // Given
        let first = node(11, 0, 1, None, vec![]);
        let second = node(22, 1, 2, None, vec![]);
        let root = node(99, 5, 7, None, vec![first, second]);

        // When
        let (first_symbol, initial_start_byte, can_move_to_second, second_symbol) =
            extract_struct_or_variant(&root, |cursor_opt, start_byte| {
                let cursor = cursor_opt
                    .as_mut()
                    .expect("cursor should start at first child");
                let first_symbol = cursor.node().symbol;
                let can_move_to_second = cursor.goto_next_sibling();
                let second_symbol = cursor.node().symbol;
                (first_symbol, *start_byte, can_move_to_second, second_symbol)
            });

        // Then
        assert_eq!(first_symbol, 11);
        assert_eq!(initial_start_byte, 5);
        assert!(can_move_to_second);
        assert_eq!(second_symbol, 22);
    }

    #[test]
    fn given_single_field_struct_when_extract_field_then_parent_node_is_extracted() {
        // Given
        let root = node(7, 2, 5, None, vec![]);
        let mut cursor_opt = Some(TreeCursor::new(&root));
        let mut last_idx = 0;

        // When
        let extracted: String = extract_field::<String, String>(
            &mut cursor_opt,
            b"xxabc",
            &mut last_idx,
            "value",
            None,
        );

        // Then
        assert_eq!(extracted, "abc");
        assert!(cursor_opt.is_none());
        assert_eq!(last_idx, 5);
    }

    #[test]
    fn given_unlabeled_children_when_extracting_named_field_then_result_is_default() {
        // Given
        let child1 = node(1, 0, 1, None, vec![]);
        let child2 = node(2, 1, 2, None, vec![]);
        let root = node(9, 0, 2, None, vec![child1, child2]);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());
        assert!(cursor.goto_next_sibling());
        let mut cursor_opt = Some(cursor);
        let mut last_idx = 1;

        // When
        let extracted: String = extract_field::<String, String>(
            &mut cursor_opt,
            b"ab",
            &mut last_idx,
            "missing_field",
            None,
        );

        // Then
        assert_eq!(extracted, "");
        assert_eq!(last_idx, 2);
        assert!(cursor_opt.is_some());
    }

    #[test]
    fn given_child_with_field_id_but_no_language_when_reading_field_name_then_returns_none() {
        // Given
        let child = node(2, 0, 1, Some(0), vec![]);
        let root = node(1, 0, 1, None, vec![child]);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());

        // When / Then
        assert_eq!(cursor.field_name(), None);
    }

    #[test]
    fn given_valid_field_table_when_reading_field_name_then_cursor_resolves_field_label() {
        // Given
        let child = node(2, 0, 1, Some(1), vec![]);
        let mut root = node(1, 0, 1, None, vec![child]);
        root.language = Some(&FIELD_LANGUAGE as *const _);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());

        // When / Then
        assert_eq!(cursor.field_name(), Some("name"));
    }

    #[test]
    fn given_out_of_range_field_id_when_reading_field_name_then_returns_none() {
        // Given
        let child = node(2, 0, 1, Some(2), vec![]);
        let mut root = node(1, 0, 1, None, vec![child]);
        root.language = Some(&FIELD_LANGUAGE as *const _);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());

        // When / Then
        assert_eq!(cursor.field_name(), None);
    }

    #[test]
    fn given_struct_without_children_when_extracting_variant_then_cursor_is_absent() {
        // Given
        let root = node(77, 3, 9, None, vec![]);

        // When
        let (cursor_missing, start_byte) =
            extract_struct_or_variant(&root, |cursor_opt, idx| (cursor_opt.is_none(), *idx));

        // Then
        assert!(cursor_missing);
        assert_eq!(start_byte, 3);
    }

    #[test]
    fn given_missing_cursor_when_extracting_field_then_default_is_returned_without_advancing() {
        // Given
        let mut cursor_opt: Option<TreeCursor> = None;
        let mut last_idx = 4;

        // When
        let extracted: String = extract_field::<String, String>(
            &mut cursor_opt,
            b"abcdef",
            &mut last_idx,
            "name",
            None,
        );

        // Then
        assert_eq!(extracted, "");
        assert_eq!(last_idx, 4);
    }
}
