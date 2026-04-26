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
        // SAFETY: `lang_ptr` is obtained from a valid `ParsedNode` whose lifetime
        // outlives this iterator. The field_id is bounds-checked below.
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
    parser.set_language(&language()).map_err(|_| {
        vec![crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(
                "Failed to initialize TreeSitter language".to_string(),
            ),
            start: 0,
            end: 0,
        }]
    })?;

    let tree = parser.parse(input, None).ok_or_else(|| {
        vec![crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(
                "TreeSitter parser returned no tree".to_string(),
            ),
            start: 0,
            end: 0,
        }]
    })?;

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
            let errors = vec![crate::errors::ParseError {
                reason: crate::errors::ParseErrorReason::UnexpectedToken(
                    "TreeSitter backend is not supported in pure-rust mode".to_string(),
                ),
                start: 0,
                end: 0,
            }];
            Err(errors)
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
    parser.set_language(lang).map_err(|e| {
        vec![crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(e),
            start: 0,
            end: 0,
        }]
    })?;

    let parse_result = parser.parse_string(input);

    if !parse_result.errors.is_empty() {
        let errors = parse_result
            .errors
            .into_iter()
            .map(|e| {
                // Get symbol name from language if available
                let symbol_name = if (e.found as usize) < lang.symbol_count as usize {
                    // SAFETY: `e.found` is bounds-checked above against `symbol_count`.
                    // `symbol_names` and `public_symbol_map` point to static language
                    // tables that live for the entire parse.
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

    let root_node = match parse_result.root {
        Some(root_node) => root_node,
        None => {
            return Err(vec![crate::errors::ParseError {
                reason: crate::errors::ParseErrorReason::UnexpectedToken(
                    "Parsed result missing root node".to_string(),
                ),
                start: 0,
                end: 0,
            }]);
        }
    };

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
#[cfg(all(feature = "glr", feature = "pure-rust"))]
fn parse_with_glr<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    // GLR Parser Integration (In Progress)
    //
    // Current Status:
    // ✅ parser_v4 module exists with GLR fork/merge logic
    // ✅ parser_v4::from_language() can load from TSLanguage structs
    // ✅ parser_v4::parse() now returns an arena-backed Tree
    // ✅ parser_v4::parse_tree() returns ParseNode for conversion
    //
    // Design Note:
    // parser_v4::parse() now returns Tree for callers that need tree-shaped APIs,
    // while parse_tree() is used by typed extraction.
    use crate::parser_v4::Parser;

    // Get the language
    let lang = language();

    // Create parser from TSLanguage with the correct grammar name for external scanner lookup
    let mut parser = Parser::from_language(lang, T::GRAMMAR_NAME.to_string());

    // Parse to get root ParseNode and parser error count.
    let source_bytes = input.as_bytes();
    let (root_node, error_count) = parser.parse_tree_with_error_count(input).map_err(|e| {
        vec![crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(e.to_string()),
            start: 0,
            end: 0,
        }]
    })?;

    if error_count > 0 {
        // Fallback for grammars/inputs where parser_v4 still reports recoveries.
        // Keep GLR as default routing, but preserve user-visible correctness.
        return parse_with_pure_parser::<T>(input, language);
    }

    // Convert parser_v4::ParseNode to pure_parser::ParsedNode
    let parsed_node = convert_parse_node_v4_to_pure(&root_node, lang, source_bytes);

    // Match pure parser behavior: unwrap source_file wrapper when present.
    let non_extra_root_children: Vec<_> = parsed_node
        .children
        .iter()
        .filter(|c| !c.is_extra)
        .collect();
    let extract_node = if parsed_node.kind() == "source_file" && non_extra_root_children.len() == 1
    {
        non_extra_root_children[0]
    } else {
        &parsed_node
    };

    // Extract typed AST using the Extract trait
    Ok(<T as crate::Extract<_>>::extract(
        Some(extract_node),
        input.as_bytes(),
        0,
        None,
    ))
}

/// Convert parser_v4::ParseNode to pure_parser::ParsedNode
#[cfg(all(feature = "glr", feature = "pure-rust"))]
fn convert_parse_node_v4_to_pure(
    node: &crate::parser_v4::ParseNode,
    lang: &crate::pure_parser::TSLanguage,
    source: &[u8],
) -> crate::pure_parser::ParsedNode {
    let field_name_to_id = |field_name: &str| -> Option<u16> {
        if lang.field_names.is_null() || lang.field_count == 0 {
            return None;
        }

        // SAFETY: `field_names` points to `field_count` static C-string pointers
        // generated alongside the language tables.
        let field_names =
            unsafe { std::slice::from_raw_parts(lang.field_names, lang.field_count as usize) };
        for (idx, name_ptr) in field_names.iter().enumerate() {
            if name_ptr.is_null() {
                continue;
            }
            // SAFETY: `name_ptr` was null-checked and points to a NUL-terminated
            // static field name from generated language metadata.
            let name = unsafe { std::ffi::CStr::from_ptr(*name_ptr as *const i8).to_str() }.ok();
            if name == Some(field_name) {
                return u16::try_from(idx).ok();
            }
        }
        None
    };

    let is_error_symbol = |symbol: u16| {
        if symbol as u32 >= lang.symbol_count || lang.symbol_names.is_null() {
            return false;
        }

        let symbol_names =
            // SAFETY: `symbol` is bounds-checked above and `symbol_names` is not null.
            // The pointer refers to a static array of `symbol_count` entries.
            unsafe { std::slice::from_raw_parts(lang.symbol_names, lang.symbol_count as usize) };
        let name_ptr = symbol_names[symbol as usize];
        if name_ptr.is_null() {
            return false;
        }

        // SAFETY: `name_ptr` was just null-checked. It points to a static NUL-
        // terminated C string from the language tables.
        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr as *const i8).to_str() }.ok();
        matches!(name, Some("ERROR"))
    };

    // Recursively convert children
    let children = node
        .children
        .iter()
        .map(|child| convert_parse_node_v4_to_pure(child, lang, source))
        .collect();

    // Read symbol metadata from TSLanguage
    // SAFETY: `symbol_metadata` is a static array of `symbol_count` entries.
    // `node.symbol.0` is bounds-checked before dereferencing.
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
    let is_empty_error_node =
        node.symbol.0 == 0 && node.children.is_empty() && node.start_byte == node.end_byte;

    crate::pure_parser::ParsedNode {
        symbol: node.symbol.0, // SymbolId.0 -> TSSymbol
        children,
        start_byte: node.start_byte,
        end_byte: node.end_byte,
        start_point: byte_to_point(source, node.start_byte),
        end_point: byte_to_point(source, node.end_byte),
        is_extra,
        is_error: is_error_symbol(node.symbol.0) || is_empty_error_node,
        is_missing: false,
        is_named,
        field_id: node.field_name.as_deref().and_then(field_name_to_id),
        language: Some(lang as *const _),
    }
}

#[allow(dead_code)]
fn byte_to_point(source: &[u8], byte_pos: usize) -> crate::pure_parser::Point {
    let mut row = 0u32;
    let mut column = 0u32;
    let end = byte_pos.min(source.len());

    for &b in &source[..end] {
        if b == b'\n' {
            row = row.saturating_add(1);
            column = 0;
        } else {
            column = column.saturating_add(1);
        }
    }

    crate::pure_parser::Point { row, column }
}

/// Parse using the GLR parser (stub for when feature is not enabled)
#[cfg(all(feature = "pure-rust", not(feature = "glr")))]
fn parse_with_glr<T: Extract<T>>(
    _input: &str,
    _language: impl Fn() -> &'static crate::pure_parser::TSLanguage,
) -> core::result::Result<T, Vec<crate::errors::ParseError>> {
    Err(vec![crate::errors::ParseError {
        reason: crate::errors::ParseErrorReason::UnexpectedToken(
            "GLR parser backend is unavailable because the `glr` feature is disabled".to_string(),
        ),
        start: 0,
        end: 0,
    }])
}

#[cfg(all(test, feature = "pure-rust"))]
mod tests {
    use super::*;
    use crate::pure_parser::{
        ExternalScanner, ParsedNode, Point, TSLanguage, TSLexState, TSParseAction, TSRule,
    };
    use core::ptr;

    #[test]
    #[cfg(feature = "glr")]
    fn given_error_symbol_named_error_when_converting_parse_node_then_marked_as_error() {
        let symbol_error = b"ERROR\0";
        let symbol_root = b"root\0";
        let symbol_names = [symbol_error.as_ptr(), symbol_root.as_ptr()];
        let language = TSLanguage {
            symbol_count: 2,
            symbol_names: symbol_names.as_ptr(),
            symbol_metadata: ptr::null(),
            ..FIELD_LANGUAGE
        };
        let parse_node = crate::parser_v4::ParseNode {
            symbol: adze_ir::SymbolId(0),
            symbol_id: adze_ir::SymbolId(0),
            start_byte: 0,
            end_byte: 0,
            field_name: None,
            children: vec![],
        };

        let converted = convert_parse_node_v4_to_pure(&parse_node, &language, b"");
        assert!(converted.is_error);
    }

    #[test]
    #[cfg(feature = "glr")]
    fn given_empty_symbol_zero_node_when_name_lookup_absent_then_marked_error_by_shape() {
        let names = [c"root".as_ptr() as *const u8];
        let language = TSLanguage {
            symbol_count: 1,
            symbol_names: names.as_ptr(),
            symbol_metadata: ptr::null(),
            ..FIELD_LANGUAGE
        };
        let parse_node = crate::parser_v4::ParseNode {
            symbol: adze_ir::SymbolId(0),
            symbol_id: adze_ir::SymbolId(0),
            start_byte: 0,
            end_byte: 0,
            field_name: None,
            children: vec![],
        };

        let converted = convert_parse_node_v4_to_pure(&parse_node, &language, b"");
        assert!(converted.is_error);
    }

    #[test]
    #[cfg(feature = "glr")]
    fn given_parse_node_with_known_field_name_when_converting_then_field_id_is_preserved() {
        let parse_node = crate::parser_v4::ParseNode {
            symbol: adze_ir::SymbolId(1),
            symbol_id: adze_ir::SymbolId(1),
            start_byte: 0,
            end_byte: 1,
            field_name: Some("value".to_string()),
            children: vec![],
        };

        let converted = convert_parse_node_v4_to_pure(&parse_node, &FIELD_LANGUAGE, b"x");
        assert_eq!(converted.field_id, Some(0));
    }

    #[test]
    #[cfg(feature = "glr")]
    fn given_nested_parse_node_with_known_field_name_when_converting_then_child_field_id_is_set() {
        let parse_node = crate::parser_v4::ParseNode {
            symbol: adze_ir::SymbolId(1),
            symbol_id: adze_ir::SymbolId(1),
            start_byte: 0,
            end_byte: 3,
            field_name: None,
            children: vec![crate::parser_v4::ParseNode {
                symbol: adze_ir::SymbolId(2),
                symbol_id: adze_ir::SymbolId(2),
                start_byte: 1,
                end_byte: 2,
                field_name: Some("name".to_string()),
                children: vec![],
            }],
        };

        let converted = convert_parse_node_v4_to_pure(&parse_node, &FIELD_LANGUAGE, b"abc");
        assert_eq!(converted.children.len(), 1);
        assert_eq!(converted.children[0].field_id, Some(1));
    }

    #[test]
    #[cfg(feature = "glr")]
    fn given_parse_node_with_unknown_field_name_when_converting_then_field_id_is_none() {
        let parse_node = crate::parser_v4::ParseNode {
            symbol: adze_ir::SymbolId(1),
            symbol_id: adze_ir::SymbolId(1),
            start_byte: 0,
            end_byte: 1,
            field_name: Some("missing".to_string()),
            children: vec![],
        };

        let converted = convert_parse_node_v4_to_pure(&parse_node, &FIELD_LANGUAGE, b"x");
        assert_eq!(converted.field_id, None);
    }

    static FIELD_NAME_VALUE: &[u8] = b"value\0";
    static FIELD_NAME_NAME: &[u8] = b"name\0";

    #[repr(transparent)]
    struct FieldNames([*const u8; 2]);
    // SAFETY: The pointers refer to static byte string literals (`b"value\0"` and
    // `b"name\0"`) that are immutable and valid for the lifetime of the program.
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
    fn given_mismatched_labeled_child_when_extracting_field_then_returns_default() {
        // Given
        let child = node(1, 0, 1, Some(1), vec![node(11, 0, 1, None, vec![])]); // "name"
        let mut root = node(9, 0, 1, None, vec![child]);
        root.language = Some(&FIELD_LANGUAGE as *const _);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());
        let mut cursor_opt = Some(cursor);
        let mut last_idx = 0;

        // When
        let extracted: String =
            extract_field::<String, String>(&mut cursor_opt, b"a", &mut last_idx, "value", None);

        // Then
        assert_eq!(extracted, "");
        assert_eq!(last_idx, 0);
    }

    #[test]
    fn given_anonymous_child_before_named_field_when_extracting_then_anonymous_is_skipped() {
        // Given
        let anon = node(1, 0, 1, None, vec![node(10, 0, 1, None, vec![])]);
        let named = node(2, 1, 2, Some(0), vec![]); // "value"
        let mut root = node(9, 0, 2, None, vec![anon, named]);
        root.language = Some(&FIELD_LANGUAGE as *const _);
        let mut cursor = TreeCursor::new(&root);
        assert!(cursor.goto_first_child());
        let mut cursor_opt = Some(cursor);
        let mut last_idx = 0;

        // When
        let extracted: String =
            extract_field::<String, String>(&mut cursor_opt, b"ab", &mut last_idx, "value", None);

        // Then
        assert_eq!(extracted, "b");
        assert_eq!(last_idx, 2);
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

    #[test]
    fn byte_to_point_tracks_newlines_and_columns() {
        let source = b"ab\ncde\nf";
        assert_eq!(byte_to_point(source, 0), Point { row: 0, column: 0 });
        assert_eq!(byte_to_point(source, 1), Point { row: 0, column: 1 });
        assert_eq!(byte_to_point(source, 2), Point { row: 0, column: 2 });
        assert_eq!(byte_to_point(source, 3), Point { row: 1, column: 0 });
        assert_eq!(byte_to_point(source, 4), Point { row: 1, column: 1 });
        assert_eq!(byte_to_point(source, 7), Point { row: 2, column: 0 });
        assert_eq!(byte_to_point(source, 99), Point { row: 2, column: 1 });
    }
}
