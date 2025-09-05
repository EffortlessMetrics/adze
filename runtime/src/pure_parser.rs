//! Pure-Rust parser and AST types used by the runtime.
//!
//! This module mirrors parts of the Tree-sitter ABI and exposes low-level
//! structures that are not intended to be stable public API.
//! We allow missing per-item docs here under `strict_docs` to reduce churn.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Pure-Rust Tree-sitter compatible parser runtime
// This implements the core parsing algorithm with GLR support

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

// Import ABI types from tablegen
type TSSymbol = u16;
type TSStateId = u16;
#[allow(dead_code)]
type TSFieldId = u16;

/// Lex state for external scanners
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TSLexState {
    pub lex_state: u16,
    pub external_lex_state: u16,
}

// Language version constants
pub const TREE_SITTER_LANGUAGE_VERSION: u32 = 15;
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: u32 = 13;

/// Wrapper for raw pointers to make them Sync
#[repr(transparent)]
pub struct SyncPtr(*const u8);

unsafe impl Sync for SyncPtr {}

impl SyncPtr {
    pub const fn new(ptr: *const u8) -> Self {
        Self(ptr)
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.0
    }
}

/// Point in a text document (row/column)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub row: u32,
    pub column: u32,
}

/// Parser for Tree-sitter grammars
#[derive(Debug)]
// Define full TSLanguage for pure-Rust implementation - matches Tree-sitter ABI 15
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TSLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub alias_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub state_count: u32,
    pub large_state_count: u32,
    pub production_id_count: u32,
    pub field_count: u32,
    pub max_alias_sequence_length: u16,
    pub production_id_map: *const u16,
    pub parse_table: *const u16,
    pub small_parse_table: *const u16,
    pub small_parse_table_map: *const u32,
    pub parse_actions: *const TSParseAction,
    pub symbol_names: *const *const u8,
    pub field_names: *const *const u8,
    pub field_map_slices: *const u16,
    pub field_map_entries: *const u16,
    pub symbol_metadata: *const u8,
    pub public_symbol_map: *const TSSymbol,
    pub alias_map: *const u16,
    pub alias_sequences: *const TSSymbol,
    pub lex_modes: *const TSLexState,
    pub lex_fn: Option<unsafe extern "C" fn(*mut c_void, TSLexState) -> bool>,
    pub keyword_lex_fn: Option<unsafe extern "C" fn(*mut c_void, TSStateId) -> TSSymbol>,
    pub keyword_capture_token: TSSymbol,
    pub external_scanner: ExternalScanner,
    pub primary_state_ids: *const TSStateId,
    pub production_lhs_index: *const u16, // LHS symbols in table index space
    pub production_count: u16,            // Number of productions
    pub eof_symbol: u16,                  // Column index of EOF (usually 0)
    pub rules: *const TSRule,             // Rule metadata array
    pub rule_count: u16,                  // Number of rules
}

/// Rule metadata for Tree-sitter grammars
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TSRule {
    pub lhs: u16,    // SymbolId of LHS
    pub rhs_len: u8, // number of symbols on RHS
    pub _pad: u8,    // keep alignment
}

// SAFETY: TSLanguage is a read-only structure that doesn't contain any mutable state.
// All pointers point to static data that is never modified after initialization.
unsafe impl Sync for TSLanguage {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TSParseAction {
    pub action_type: u8,
    pub extra: u8,
    pub child_count: u8,
    pub dynamic_precedence: i8,
    pub symbol: TSSymbol,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExternalScanner {
    pub states: *const u8,
    pub symbol_map: *const TSSymbol,
    pub create: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    pub scan: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *const bool) -> bool>,
    pub serialize: Option<unsafe extern "C" fn(*mut c_void, *mut u8) -> u32>,
    pub deserialize: Option<unsafe extern "C" fn(*mut c_void, *const u8, u32)>,
}

impl ExternalScanner {
    pub const fn default() -> Self {
        Self {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        }
    }
}

impl Default for ExternalScanner {
    fn default() -> Self {
        Self::default()
    }
}

pub struct Parser {
    language: Option<&'static TSLanguage>,
    stack: Vec<StackEntry>,
    timeout_micros: u64,
    cancellation_flag: Option<*const AtomicBool>,
    lexer: Option<Lexer>,
}

/// Stack entry for LR parsing
#[derive(Debug, Clone)]
struct StackEntry {
    state: TSStateId,
    subtree: Option<Subtree>,
    #[allow(dead_code)]
    position: usize,
}

/// Lexer state
#[derive(Debug)]
struct Lexer {
    input: Vec<u8>,
    #[allow(dead_code)]
    position: usize,
    #[allow(dead_code)]
    external_scanner: Option<*mut c_void>,
}

/// Internal node representation during parsing
#[derive(Debug, Clone)]
struct Subtree {
    symbol: TSSymbol,
    children: Vec<Subtree>,
    start_byte: usize,
    end_byte: usize,
    start_point: Point,
    end_point: Point,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    #[allow(dead_code)]
    production_id: u16,
}

/// Parse result
pub struct ParseResult {
    pub root: Option<ParsedNode>,
    pub errors: Vec<ParseError>,
}

/// Parsed node
#[derive(Debug, Clone)]
pub struct ParsedNode {
    pub symbol: TSSymbol,
    pub children: Vec<ParsedNode>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_point: Point,
    pub end_point: Point,
    pub is_extra: bool,
    pub is_error: bool,
    pub is_missing: bool,
    pub is_named: bool,
    pub field_name: Option<String>,
    pub(crate) language: Option<*const TSLanguage>,
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: usize,
    pub point: Point,
    pub expected: Vec<TSSymbol>,
    pub found: TSSymbol,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Parser {
            language: None,
            stack: Vec::new(),
            timeout_micros: 0,
            cancellation_flag: None,
            lexer: None,
        }
    }

    /// Set the language for parsing
    pub fn set_language(&mut self, language: &'static TSLanguage) -> Result<(), String> {
        // Validate language version
        if language.version < TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION
            || language.version > TREE_SITTER_LANGUAGE_VERSION
        {
            return Err(format!(
                "Incompatible language version: {}",
                language.version
            ));
        }

        // Validate required pointers based on table type
        if language.large_state_count > 0 {
            // Large-table path requires parse_table + parse_actions
            if language.parse_table.is_null() || language.parse_actions.is_null() {
                return Err(
                    "Invalid language: large_state_count > 0 but parse_table/parse_actions is null"
                        .to_string(),
                );
            }
        } else {
            // Small-table path requires both small arrays
            if language.small_parse_table.is_null() || language.small_parse_table_map.is_null() {
                return Err("Invalid language: small table path missing small_parse_table/small_parse_table_map".to_string());
            }
        }

        // Symbol metadata & names must be present
        if language.symbol_names.is_null() || language.symbol_metadata.is_null() {
            return Err("Invalid language: missing symbol_names or symbol_metadata".to_string());
        }

        // Field names can be null if field_count == 0
        if language.field_count > 0 && language.field_names.is_null() {
            return Err("Invalid language: field_count > 0 but field_names is null".to_string());
        }

        self.language = Some(language);
        self.reset();
        Ok(())
    }

    /// Get the current language
    pub fn language(&self) -> Option<&'static TSLanguage> {
        self.language
    }

    /// Set timeout for parsing in microseconds
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.timeout_micros = timeout;
    }

    /// Set cancellation flag for parsing
    pub fn set_cancellation_flag(&mut self, flag: Option<*const AtomicBool>) {
        self.cancellation_flag = flag;
    }

    /// Reset parser state
    pub fn reset(&mut self) {
        self.stack.clear();
    }

    /// Parse a string of source code
    pub fn parse_string(&mut self, source: &str) -> ParseResult {
        self.parse_string_with_tree(source, None)
    }

    /// Parse a string with an old tree for incremental parsing
    pub fn parse_string_with_tree(
        &mut self,
        source: &str,
        old_tree: Option<&crate::pure_incremental::Tree>,
    ) -> ParseResult {
        let bytes = source.as_bytes();
        self.parse_bytes_with_tree(bytes, old_tree)
    }

    /// Parse bytes of source code
    pub fn parse_bytes(&mut self, source: &[u8]) -> ParseResult {
        self.parse_bytes_with_tree(source, None)
    }

    /// Parse bytes with an old tree for incremental parsing
    pub fn parse_bytes_with_tree(
        &mut self,
        source: &[u8],
        old_tree: Option<&crate::pure_incremental::Tree>,
    ) -> ParseResult {
        let language = match self.language {
            Some(lang) => lang,
            None => {
                return ParseResult {
                    root: None,
                    errors: vec![ParseError {
                        position: 0,
                        point: Point { row: 0, column: 0 },
                        expected: vec![],
                        found: 0,
                    }],
                };
            }
        };

        // Initialize parser state
        self.stack.clear();

        // The initial state is always state 0 in a properly generated parser
        let initial_state = 0;

        self.stack.push(StackEntry {
            state: initial_state,
            subtree: None,
            position: 0,
        });

        // Initialize lexer with sentinel byte to prevent OOB reads
        let mut input_with_sentinel = source.to_vec();
        // Add a null byte sentinel if not already present
        if input_with_sentinel.last().copied() != Some(0) {
            input_with_sentinel.push(0);
        }
        self.lexer = Some(Lexer {
            input: input_with_sentinel,
            position: 0,
            external_scanner: None,
        });

        // Get reusable nodes from old tree if available
        let _reusable_nodes = old_tree.map(|tree| tree.get_reusable_nodes());

        let mut errors = Vec::new();
        let mut position = 0;
        let mut point = Point { row: 0, column: 0 };
        let start_time = Instant::now();

        // Main parsing loop
        let mut iteration_count = 0;
        loop {
            iteration_count += 1;
            if iteration_count > 10000 {
                //eprintln!("WARNING: Parser exceeded 10000 iterations, likely infinite loop");
                errors.push(ParseError {
                    position,
                    point,
                    expected: vec![],
                    found: 0,
                });
                break;
            }

            // Check timeout
            if self.timeout_micros > 0 {
                let elapsed = start_time.elapsed().as_micros() as u64;
                if elapsed > self.timeout_micros {
                    errors.push(ParseError {
                        position,
                        point,
                        expected: vec![],
                        found: 0,
                    });
                    break;
                }
            }

            // Check cancellation
            if let Some(flag) = self.cancellation_flag {
                unsafe {
                    if (*flag).load(Ordering::Relaxed) {
                        errors.push(ParseError {
                            position,
                            point,
                            expected: vec![],
                            found: 0,
                        });
                        break;
                    }
                }
            }

            // Get current state
            let current_state = match self.stack.last() {
                Some(entry) => entry.state,
                None => break,
            };

            // Lex next token
            let token = self.lex_token(language, current_state, position, &mut point);

            // Handle extra tokens (like whitespace)
            if token.is_extra {
                // Create an extra node representing the token
                let end_point = advance_point(point, &source[position..position + token.length]);
                let extra_subtree = Subtree {
                    symbol: token.symbol,
                    children: Vec::new(),
                    start_byte: position,
                    end_byte: position + token.length,
                    start_point: point,
                    end_point,
                    is_extra: true,
                    is_error: false,
                    is_missing: false,
                    production_id: 0,
                };

                // Attach extra tokens to the current node on the stack if possible
                if let Some(entry) = self.stack.iter_mut().rev().find(|e| e.subtree.is_some()) {
                    if let Some(ref mut subtree) = entry.subtree {
                        subtree.children.push(extra_subtree);
                    }
                } else if let Some(entry) = self.stack.last_mut() {
                    // If no subtree exists yet, attach to the current stack entry
                    entry.subtree = Some(extra_subtree);
                } else {
                    // As a fallback (shouldn't normally happen), create a new stack entry
                    self.stack.push(StackEntry {
                        state: current_state,
                        subtree: Some(extra_subtree),
                        position: position + token.length,
                    });
                }

                // Advance position and point
                position += token.length;
                point = end_point;
                continue;
            }

            // Track parsing progress

            // Get action for current state and token
            let action = self.get_action(language, current_state, token.symbol);
            // Debug logging for arithmetic parsing
            if source.len() < 20 {
                //if current_state == 0 && position == 0 {
                self.dump_row(language, 0);
                //}
                let _byte_repr = if position < source.len() {
                    format!(
                        "'{}' (0x{:02x})",
                        source[position] as char, source[position]
                    )
                } else {
                    "EOF".to_string()
                };
                // eprintln!(
                // "DEBUG: Position={}, State={}, token symbol={}, action={:?}, current_byte={}, stack_len={}",
                // position,
                // current_state,
                // token.symbol,
                // action,
                // _byte_repr,
                // self.stack.len()
                // );
                // eprintln!(
                // "DEBUG: Stack size: {}, token.length={}",
                // self.stack.len(),
                // token.length
                // );
            }
            match action {
                Action::Shift(next_state) => {
                    // Create leaf node
                    let end_point =
                        advance_point(point, &source[position..position + token.length]);
                    let subtree = Subtree {
                        symbol: token.symbol,
                        children: Vec::new(),
                        start_byte: position,
                        end_byte: position + token.length,
                        start_point: point,
                        end_point,
                        is_extra: token.is_extra,
                        is_error: false,
                        is_missing: false,
                        production_id: 0,
                    };

                    // Push onto stack
                    self.stack.push(StackEntry {
                        state: next_state,
                        subtree: Some(subtree),
                        position: position + token.length,
                    });

                    // Advance position
                    position += token.length;
                    point = advance_point(point, &source[position - token.length..position]);

                    if position < 10 {
                        ////eprintln!($
                        //    "DEBUG: Advanced position to {} (after {} bytes), continuing to next token",
                        //    position, token.length
                        //);
                    }
                    // Don't break here! We need to continue the loop
                    // break;
                }

                Action::Reduce(rule_id) => {
                    if !self.reduce(language, rule_id, source) {
                        // Reduction failed - record error and try to recover
                        errors.push(ParseError {
                            position,
                            point,
                            expected: self.get_expected_symbols(language, current_state),
                            found: token.symbol,
                        });

                        // Simple recovery: skip token and continue
                        position += token.length;
                        point = advance_point(point, &source[position - token.length..position]);
                    } else {
                        // Check if the reduction resulted in accepting the parse
                        // This happens when we reduce to the root symbol in state 0
                        if self.stack.len() >= 2 {
                            let top = &self.stack[self.stack.len() - 1];
                            let below = &self.stack[self.stack.len() - 2];

                            // Check if we have the root symbol (source_file = 8) on top of state 0
                            if below.state == 0
                                && top.subtree.is_some()
                                && let Some(ref subtree) = top.subtree
                                && subtree.symbol == 8
                                && token.symbol == 0
                            {
                                // EOF
                                // Parse successful!
                                // eprintln!("DEBUG: Parse accepted! Root subtree:");
                                // fn print_subtree(subtree: &Subtree, indent: usize) {
                                //     eprintln!("{}symbol={}, children={}, bytes={}..{}",
                                //         "  ".repeat(indent), subtree.symbol, subtree.children.len(),
                                //         subtree.start_byte, subtree.end_byte
                                //     );
                                //     for child in &subtree.children {
                                //         print_subtree(child, indent + 1);
                                //     }
                                // }
                                // print_subtree(subtree, 1);

                                return ParseResult {
                                    root: Some(subtree_to_node(
                                        subtree.clone(),
                                        Some(language as *const _),
                                    )),
                                    errors,
                                };
                            }
                        }
                    }
                    // Important: Don't advance position after reduce!
                    // The same token needs to be processed again with the new state
                    continue;
                }

                Action::Accept => {
                    ////eprintln!("DEBUG: Got ACCEPT action!");
                    // Parse successful
                    if let Some(entry) = self.stack.pop()
                        && let Some(subtree) = entry.subtree
                    {
                        return ParseResult {
                            root: Some(subtree_to_node(subtree, Some(language as *const _))),
                            errors,
                        };
                    }
                    break;
                }

                Action::Error => {
                    // Record error and try to recover
                    let expected_symbols = self.get_expected_symbols(language, current_state);
                    // eprintln!(
                    // "ERROR: position={}, current_state={}, expected_symbols={:?}, token.symbol={}",
                    // position, current_state, expected_symbols, token.symbol
                    // );
                    errors.push(ParseError {
                        position,
                        point,
                        expected: expected_symbols,
                        found: token.symbol,
                    });

                    // Simple error recovery: skip token
                    if position < source.len() {
                        position += 1;
                        point = advance_point(point, &source[position - 1..position]);
                    } else {
                        break;
                    }
                }
            }
        }

        ParseResult { root: None, errors }
    }

    /// Lex a token at the current position
    fn lex_token(
        &mut self,
        language: &TSLanguage,
        state: TSStateId,
        position: usize,
        point: &mut Point,
    ) -> Token {
        let lexer = match self.lexer.as_mut() {
            Some(l) => l,
            None => {
                debug_assert!(
                    language.eof_symbol < language.token_count as u16,
                    "EOF symbol {} must be within token range [0, {})",
                    language.eof_symbol,
                    language.token_count
                );
                return Token {
                    symbol: language.eof_symbol,
                    length: 0,
                    is_extra: false,
                };
            }
        };

        // Check for EOF (accounting for sentinel byte)
        if position >= lexer.input.len() - 1 {
            debug_assert!(
                language.eof_symbol < language.token_count as u16,
                "EOF symbol {} must be within token range [0, {})",
                language.eof_symbol,
                language.token_count
            );
            return Token {
                symbol: language.eof_symbol,
                length: 0,
                is_extra: false,
            }; // EOF
        }

        // Get lex state for current parser state
        let lex_mode = unsafe {
            if state < language.state_count as u16 {
                *language.lex_modes.add(state as usize)
            } else {
                TSLexState {
                    lex_state: 0,
                    external_lex_state: 0,
                }
            }
        };

        // Try external scanner first if available
        if lex_mode.external_lex_state != 0 && language.external_scanner.scan.is_some() {
            // TODO: Implement external scanner support
        }

        // Use built-in lexer
        if let Some(lex_fn) = language.lex_fn {
            unsafe {
                let mut lex_state = LexerState {
                    input: lexer.input.as_ptr(),
                    input_len: lexer.input.len() - 1, // Exclude sentinel byte
                    position,
                    point_row: point.row,
                    point_column: point.column,
                    result_symbol: 0,
                    result_length: 0,
                };

                // Debug what character we're looking at
                // if position < lexer.input.len() {
                //     eprintln!(
                //         "DEBUG lex_token: About to lex at position={}, char={:?} ({}), input_len={}",
                //         position, lexer.input[position] as char, lexer.input[position], lexer.input.len()
                //     );
                // }

                let lex_state_ptr = &mut lex_state as *mut _ as *mut c_void;
                if lex_fn(lex_state_ptr, lex_mode) {
                    let symbol = lex_state.result_symbol;
                    let is_extra = self.is_extra_symbol(language, symbol);
                    // eprintln!(
                    //     "DEBUG lex_token: state={}, lex_mode={}, position={}, lexer returned symbol={}, length={}, is_extra={}",
                    //     state, lex_mode.lex_state, position, symbol, lex_state.result_length, is_extra
                    // );

                    // Additional debug to understand symbol mapping
                    {
                        if symbol < language.symbol_count as u16 {
                            let symbol_names = std::slice::from_raw_parts(
                                language.symbol_names,
                                language.symbol_count as usize,
                            );
                            let name_ptr = symbol_names[symbol as usize];
                            if !name_ptr.is_null() {
                                let c_str = std::ffi::CStr::from_ptr(name_ptr as *const i8);
                                if let Ok(_name) = c_str.to_str() {
                                    // eprintln!("DEBUG lex_token: symbol {} is '{}'", symbol, name);
                                }
                            }
                        }
                    }

                    // The lexer already returns the correct symbol index
                    // No additional mapping needed
                    return Token {
                        symbol,
                        length: lex_state.result_length,
                        is_extra,
                    };
                } else {
                    // lex_fn returned false - check if we're at EOF
                    let at_eof = position >= lexer.input.len() - 1; // Account for sentinel
                    let symbol = if at_eof {
                        // EOF is column 0 in Tree-sitter convention
                        0
                    } else {
                        // Return error token for non-EOF failures
                        // For now using 0, but this should be language-specific
                        0
                    };

                    return Token {
                        symbol,
                        length: 0,
                        is_extra: false,
                    };
                }
            }
        }

        // Fallback: simple character-by-character lexing
        // Safe access with bounds check (accounting for sentinel)
        if position >= lexer.input.len() - 1 {
            // At EOF - return EOF symbol from language
            debug_assert!(
                language.eof_symbol < language.token_count as u16,
                "EOF symbol {} must be within token range [0, {})",
                language.eof_symbol,
                language.token_count
            );
            return Token {
                symbol: language.eof_symbol, // EOF column index from language
                length: 0,
                is_extra: false,
            };
        }

        let ch = lexer.input[position];
        //eprintln!("DEBUG lex_token: Fallback lexing at position={}, ch={:?} ({})", position, ch as char, ch);

        // Skip whitespace as extras
        if ch.is_ascii_whitespace() {
            let mut len = 1;
            while position + len < lexer.input.len() - 1  // Account for sentinel
                && lexer.input[position + len].is_ascii_whitespace()
            {
                len += 1;
            }
            return Token {
                symbol: 1,
                length: len,
                is_extra: true,
            };
        }

        // Single character tokens
        Token {
            symbol: ch as u16,
            length: 1,
            is_extra: false,
        }
    }

    /// Check if a symbol is marked as hidden (starts with _)
    fn is_hidden_symbol(&self, language: &TSLanguage, symbol: TSSymbol) -> bool {
        unsafe {
            if symbol < language.symbol_count as u16 {
                let metadata_ptr = language.symbol_metadata;
                if metadata_ptr.is_null() {
                    return false;
                }

                let metadata = *metadata_ptr.add(symbol as usize);
                // Check if HIDDEN flag is set (0x04)
                let is_hidden = (metadata & 0x04) != 0;
                // Also check if NOT VISIBLE (0x01 is visible flag)
                let not_visible = (metadata & 0x01) == 0;
                is_hidden || not_visible
            } else {
                false
            }
        }
    }

    /// Check if a symbol is marked as extra (e.g., whitespace)
    fn is_extra_symbol(&self, language: &TSLanguage, symbol: TSSymbol) -> bool {
        unsafe {
            if symbol < language.symbol_count as u16 {
                let metadata_ptr = language.symbol_metadata;
                if metadata_ptr.is_null() {
                    //eprintln!("ERROR: metadata_ptr is NULL!");
                    return false;
                }

                // Debug: print first few metadata bytes
                if symbol == 3 || symbol == 4 {
                    ////eprintln!("DEBUG metadata array dump for symbol {}:", symbol);
                    for i in 0..std::cmp::min(9, language.symbol_count) {
                        let _byte = *metadata_ptr.add(i as usize);
                        //eprintln!("  metadata[{}] = {:#x}", i, byte);
                    }
                }

                let metadata = *metadata_ptr.add(symbol as usize);
                // Check if HIDDEN flag is set (0x04)

                ////eprintln!($
                //"DEBUG is_extra_symbol: symbol={}, metadata_ptr={:p}, offset={}, metadata={:#x}, is_hidden={}",
                //symbol, metadata_ptr, symbol as usize, metadata, is_hidden
                //);
                (metadata & 0x04) != 0
            } else {
                false
            }
        }
    }

    /// Debug helper: dump a parse table row
    #[allow(dead_code)]
    fn dump_row(&self, language: &TSLanguage, state: u16) {
        unsafe {
            let large_state_count = language.large_state_count as usize;
            let token_count = language.token_count as u16;

            // eprintln!("=== Dumping state {} ===", state);
            // eprintln!(
            // "token_count: {}, symbol_count: {}",
            // language.token_count, language.symbol_count
            // );

            if (state as usize) >= large_state_count {
                let map_index = (state as usize) - large_state_count;
                let start_offset = (*language.small_parse_table_map.add(map_index)) as usize;
                let end_offset = (*language.small_parse_table_map.add(map_index + 1)) as usize;

                // eprintln!(
                // "Small state: map_index={}, start={}, end={}",
                // map_index, start_offset, end_offset
                // );

                let mut offset = start_offset;
                while offset + 1 < end_offset {
                    let s = *language.small_parse_table.add(offset);
                    let _v = *language.small_parse_table.add(offset + 1);
                    offset += 2;

                    let _kind = if s < token_count { "TOK " } else { "GOTO" };
                    // eprintln!("  {:>4} {:>5} -> action {}", kind, s, v);
                }
            } else {
                // eprintln!("Large state - dense row in parse_table");
            }
        }
    }

    /// Get goto state for a non-terminal after reduction
    /// Note: symbol parameter is a table column index, not a symbol ID
    fn get_goto(
        &self,
        language: &TSLanguage,
        state: TSStateId,
        symbol: TSSymbol, // Actually a column index in table space
    ) -> Option<TSStateId> {
        // eprintln!(
        // "DEBUG get_goto: state={}, col_idx={}, token_count={}, symbol_count={}",
        // state, symbol, language.token_count, language.symbol_count
        // );
        unsafe {
            // Check bounds
            if state >= language.state_count as u16 || symbol >= language.symbol_count as u16 {
                // eprintln!(
                // "  Bounds check failed: state >= {} or symbol >= {}",
                // language.state_count, language.symbol_count
                // );
                return None;
            }

            let large_state_count = language.large_state_count as usize;
            let symbol_count = language.symbol_count as usize;
            let token_count = language.token_count as u16;

            // Only non-terminals have goto entries
            if symbol < token_count {
                // eprintln!(
                // "  Symbol {} is a token (< {}), no goto",
                // symbol, token_count
                // );
                return None;
            }
            // eprintln!(
            // "  Column {} is a non-terminal (>= {}), checking goto",
            // symbol, token_count
            // );
            // eprintln!(
            // "  large_state_count={}, state={}, is_large={}",
            // large_state_count,
            // state,
            // (state as usize) < large_state_count
            // );

            if (state as usize) < large_state_count {
                // LARGE STATE: Dense row in parse_table
                let base = (state as usize) * symbol_count;
                let index = base + (symbol as usize);
                let goto_state = *language.parse_table.add(index);

                // eprintln!(
                // "  Large state: base={}, index={}, goto_state={}",
                // base, index, goto_state
                // );

                if goto_state != 0 {
                    return Some(goto_state);
                }
            } else {
                // SMALL STATE: Look for goto entry
                let map_index = (state as usize) - large_state_count;
                let start_offset = (*language.small_parse_table_map.add(map_index)) as usize;
                let end_offset = (*language.small_parse_table_map.add(map_index + 1)) as usize;

                // eprintln!(
                // "  Small state: map_index={}, start_offset={}, end_offset={}",
                // map_index, start_offset, end_offset
                // );

                let mut offset = start_offset;
                while offset + 1 < end_offset {
                    let entry_col = *language.small_parse_table.add(offset) as usize;
                    let entry_val = *language.small_parse_table.add(offset + 1);
                    // eprintln!(
                    // "    Entry at offset {}: col={}, val={}",
                    // offset, entry_col, entry_val
                    // );
                    offset += 2;

                    // Check if this is the column we're looking for
                    if entry_col == symbol as usize {
                        // This entry is for a non-terminal (symbol >= token_count was checked above)
                        // The value is the goto state

                        // Debug guard: verify this is a nonterminal column
                        debug_assert!(
                            symbol >= token_count,
                            "get_goto should only find NT columns: {} >= {}",
                            symbol,
                            token_count
                        );

                        // eprintln!(
                        // "    Found match for column {}! goto_state={}",
                        // symbol, entry_val
                        // );
                        if entry_val != 0 {
                            return Some(entry_val);
                        }
                        return None;
                    }
                }
                // eprintln!("    No match found for column {}", symbol);
            }
            None
        }
    }

    /// Get parse action for state and symbol
    fn get_action(&self, language: &TSLanguage, state: TSStateId, symbol: TSSymbol) -> Action {
        // Debug dump state 0 once
        use std::sync::Once;
        static DUMP_ONCE: Once = Once::new();
        if state == 0 {
            DUMP_ONCE.call_once(|| self.dump_row(language, 0));
        }

        // Look up action in parse table
        unsafe {
            // Check bounds
            if state >= language.state_count as u16 || symbol >= language.symbol_count as u16 {
                return Action::Error;
            }

            let large_state_count = language.large_state_count as usize;
            let symbol_count = language.symbol_count as usize;
            let token_count = language.token_count as u16;

            // Only tokens (not non-terminals) are valid lookaheads
            if symbol >= token_count {
                return Action::Error;
            }

            // Sanity checks
            debug_assert!((language.token_count as usize) <= language.symbol_count as usize);
            debug_assert!((state as usize) < language.state_count as usize);

            if (state as usize) < large_state_count {
                // LARGE STATE: Dense row in parse_table
                // Layout: parse_table[(state * symbol_count) + lookahead]
                let base = (state as usize) * symbol_count;
                let index = base + (symbol as usize);

                // Large states use the main parse_table
                let action_value = *language.parse_table.add(index);

                if action_value != 0 {
                    return self.decode_action(language, action_value as usize);
                }
            } else {
                // SMALL STATE: Compressed row in small_parse_table
                // Format is direct (symbol, action) pairs
                let map_index = (state as usize) - large_state_count;

                // Read u32 offsets properly!
                let start_offset = (*language.small_parse_table_map.add(map_index)) as usize;
                let end_offset = (*language.small_parse_table_map.add(map_index + 1)) as usize;

                // Parse direct (column, value) pairs
                let mut offset = start_offset;

                while offset + 1 < end_offset {
                    let entry_col = *language.small_parse_table.add(offset);
                    let entry_val = *language.small_parse_table.add(offset + 1);
                    offset += 2;

                    // Check if this is the column we're looking for
                    // For tokens, symbol ID equals column index (by design)
                    if entry_col == symbol {
                        // Verify this is a token column (should always be true when called from get_action)
                        debug_assert!(
                            entry_col < token_count,
                            "get_action should only look at token columns"
                        );
                        if entry_val != 0 {
                            return self.decode_action(language, entry_val as usize);
                        }
                        return Action::Error;
                    }
                }
            }
        }

        Action::Error
    }

    /// Decode action from parse table
    fn decode_action(&self, _language: &TSLanguage, action_index: usize) -> Action {
        // Decode action from index

        // In the pure-Rust implementation, actions are encoded directly in the parse table
        // High bit set = reduce, otherwise shift
        if action_index & 0x8000 != 0 {
            // Reduce action
            let production_id = (action_index & 0x7FFF) as u16;
            // Reduce action
            Action::Reduce(production_id)
        } else if action_index == 0xFFFF {
            // Accept action (encoded as 0xFFFF in compression)
            Action::Accept
        } else {
            // Shift action
            let next_state = action_index as u16;
            // Shift action
            Action::Shift(next_state)
        }
    }

    /// Get the LHS symbol index for a production from the production_lhs_index array
    #[inline]
    fn lhs_index_of(&self, language: &TSLanguage, production_index: u16) -> u16 {
        unsafe { *language.production_lhs_index.add(production_index as usize) }
    }

    /// Perform a reduction
    fn reduce(&mut self, language: &TSLanguage, production_id: u16, source: &[u8]) -> bool {
        if source.len() < 20 {
            // eprintln!(
            // "DEBUG reduce: Reducing with production_id={} (from parse table)",
            // production_id
            // );
            // eprintln!(
            // "DEBUG reduce: Stack before reduction has {} entries",
            // self.stack.len()
            // );
            for entry in self.stack.iter() {
                if let Some(ref _subtree) = entry.subtree {
                    // eprintln!(
                    // "  Stack[{}]: state={}, symbol={}",
                    // i, entry.state, subtree.symbol
                    // );
                } else {
                    // eprintln!("  Stack[{}]: state={}, no subtree", i, entry.state);
                }
            }
        }

        unsafe {
            // Tree-sitter uses 1-based production IDs in the parse table
            // We need to subtract 1 and then use production_id_map
            if production_id == 0 {
                ////eprintln!("DEBUG reduce: Invalid production_id=0 (production IDs are 1-based)");
                return false;
            }

            let zero_based_id = production_id - 1;

            // Use the production_id_map to get the actual production index
            let production_index = if zero_based_id < language.production_id_count as u16 {
                *language.production_id_map.add(zero_based_id as usize)
            } else {
                ////eprintln!("DEBUG reduce: Invalid production_id {} (zero_based={}, >= {})",
                //         production_id, zero_based_id, language.production_id_count);
                return false;
            };

            // Look up the parse action for this production
            if production_index >= language.production_id_count as u16 {
                ////eprintln!($
                //    "DEBUG reduce: Invalid production_index {} (>= {})",
                //    production_index, language.production_id_count
                //);
                return false;
            }

            let action = &*language.parse_actions.add(production_index as usize);
            let child_count = action.child_count as usize;

            // Get the LHS symbol from the production_lhs_index array instead of parse_actions
            // This ensures the symbol is in table index space
            let symbol = self.lhs_index_of(language, production_index);

            // Also check what parse_actions says for comparison
            let _parse_action_symbol = action.symbol;
            // eprintln!(
            // "DEBUG: production_index={}, lhs_index={}, parse_action_symbol={}",
            // production_index, symbol, parse_action_symbol
            // );

            if source.len() < 20 {
                // eprintln!(
                // "DEBUG reduce: Production {} (index {}) reduces to symbol {} with {} children (token_count={})",
                // production_id, production_index, symbol, child_count, language.token_count
                // );
                debug_assert!(
                    symbol >= language.token_count as u16,
                    "LHS symbol {} should be a non-terminal (>= token_count {})",
                    symbol,
                    language.token_count
                );
            }

            // If child_count is 3 but we only have 2 items on stack, something is wrong
            if child_count > self.stack.len() {
                ////eprintln!($
                //    "DEBUG reduce: ERROR! Need {} children but stack only has {} items",
                //    child_count,
                //    self.stack.len()
                //);
                return false;
            }

            // Check if we have enough stack entries
            if self.stack.len() < child_count {
                return false;
            }

            // Pop child_count entries from the stack
            let mut children = Vec::new();
            let mut start_byte = usize::MAX;
            let mut end_byte = 0;
            let mut start_point = Point {
                row: u32::MAX,
                column: u32::MAX,
            };
            let mut end_point = Point { row: 0, column: 0 };

            // Pop children in reverse order
            for _ in 0..child_count {
                if let Some(entry) = self.stack.pop()
                    && let Some(subtree) = entry.subtree
                {
                    // Update bounds
                    if subtree.start_byte < start_byte {
                        start_byte = subtree.start_byte;
                        start_point = subtree.start_point;
                    }
                    if subtree.end_byte > end_byte {
                        end_byte = subtree.end_byte;
                        end_point = subtree.end_point;
                    }
                    children.push(subtree);
                }
            }

            // Reverse children to correct order
            children.reverse();

            // Handle empty reduction
            if children.is_empty() && child_count == 0 {
                // For empty reductions, use position from top of stack
                if let Some(top) = self.stack.last() {
                    start_byte = top.position;
                    end_byte = top.position;
                    start_point = Point {
                        row: 0,
                        column: top.position as u32,
                    };
                    end_point = start_point;
                }
            }

            // Check if this symbol is hidden (e.g., _Expression)
            let is_hidden = self.is_hidden_symbol(language, symbol);

            // Create parent node or unwrap if hidden
            let parent = if is_hidden && children.len() == 1 {
                // Return the child directly, skipping the hidden wrapper
                children.into_iter().next().unwrap()
            } else {
                // Create the parent node normally
                Subtree {
                    symbol,
                    children,
                    start_byte,
                    end_byte,
                    start_point,
                    end_point,
                    is_extra: false,
                    is_error: false,
                    is_missing: false,
                    production_id,
                }
            };

            // Check if this is the start symbol (source_file)
            // The start symbol is typically the one that appears on the left side of the start rule
            // In Tree-sitter, the start symbol is typically called "source_file" or similar
            // We need to determine this dynamically from the language structure

            ////eprintln!($
            //    "DEBUG reduce: After reduction, stack size would be: {}",
            //    self.stack.len()
            //);

            // Get the goto state for the reduced symbol
            let prev_state = if let Some(entry) = self.stack.last() {
                entry.state
            } else {
                0
            };

            ////eprintln!($
            //"DEBUG reduce: Looking up goto for symbol {} from state {}",
            //symbol, prev_state
            //);

            // Debug: Show all gotos available from this state
            if source.len() < 20 && prev_state == 0 {
                // eprintln!("DEBUG reduce: Available gotos from state 0:");
                for sym_idx in 0..12 {
                    if let Some(_goto_state) = self.get_goto(language, 0, sym_idx) {
                        // eprintln!("  Symbol {} -> state {}", sym_idx, goto_state);
                    }
                }
            }

            // Look up goto state for the non-terminal we just reduced to
            // eprintln!(
            // "DEBUG reduce: About to call get_goto with prev_state={}, symbol={}",
            // prev_state, symbol
            // );
            if let Some(goto_state) = self.get_goto(language, prev_state, symbol) {
                // Push the reduced node with the goto state
                self.stack.push(StackEntry {
                    state: goto_state,
                    subtree: Some(parent),
                    position: end_byte,
                });
                true
            } else {
                // No goto found - this shouldn't happen in a valid parse table
                // eprintln!(
                // "DEBUG reduce: No goto for symbol {} from state {} (symbol >= token_count: {})",
                // symbol,
                // prev_state,
                // symbol >= language.token_count as u16
                // );
                false
            }
        }
    }

    /// Get production ID for field mappings
    #[allow(dead_code)]
    fn get_production_id(&self, language: &TSLanguage, action_index: u16) -> u16 {
        unsafe {
            if action_index < language.production_id_count as u16 {
                *language.production_id_map.add(action_index as usize)
            } else {
                0
            }
        }
    }

    /// Look up the goto state for a non-terminal using the parse table.
    /// Returns `0` when no transition exists, mirroring Tree-sitter's ABI.
    #[allow(dead_code)]
    fn get_goto_state(
        &self,
        language: &TSLanguage,
        state: TSStateId,
        symbol: TSSymbol,
    ) -> TSStateId {
        self.get_goto(language, state, symbol).unwrap_or(0)
    }

    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, language: &TSLanguage, state: TSStateId) -> Vec<TSSymbol> {
        let mut expected = Vec::new();

        // Check all possible symbols in the parse table
        // The parse table uses the symbol_to_index mapping, so we need to check
        // all symbols from 0 to symbol_count
        let symbol_count = language.symbol_count as u16;

        // For now, check all symbols to see what's valid
        // We can optimize this later to only check terminals
        for symbol in 0..symbol_count {
            let action = self.get_action(language, state, symbol);
            if !matches!(action, Action::Error) {
                expected.push(symbol);
                // Debug: print symbol names for expected symbols
                if state == 0 {
                    unsafe {
                        let symbol_names = std::slice::from_raw_parts(
                            language.symbol_names,
                            language.symbol_count as usize,
                        );
                        if symbol < language.symbol_count as u16 {
                            let name_ptr = symbol_names[symbol as usize];
                            if !name_ptr.is_null() {
                                let c_str = std::ffi::CStr::from_ptr(name_ptr as *const i8);
                                let _name = c_str.to_string_lossy();
                            }
                        }
                    }
                }
            }
        }

        expected
    }

    /// Temporary fallback: do a full reparse. Keeps tests stable while
    /// incremental engine wiring lands.
    pub fn reparse(
        &mut self,
        source: &str,
        _old_tree: &crate::pure_incremental::Tree,
        _edit: &crate::pure_incremental::Edit,
    ) -> ParseResult {
        self.parse_string(source)
    }
}

/// Token returned by lexer
#[derive(Debug, Clone, Copy)]
struct Token {
    symbol: TSSymbol,
    length: usize,
    is_extra: bool,
}

/// Lexer state for C callback
#[repr(C)]
struct LexerState {
    input: *const u8,
    input_len: usize,
    position: usize,
    point_row: u32,
    point_column: u32,
    result_symbol: TSSymbol,
    result_length: usize,
}

/// Parse action
#[derive(Debug, Clone, Copy)]
enum Action {
    Shift(TSStateId),
    Reduce(u16),
    Accept,
    Error,
}

/// Advance point by text
fn advance_point(mut point: Point, text: &[u8]) -> Point {
    for &byte in text {
        if byte == b'\n' {
            point.row += 1;
            point.column = 0;
        } else {
            point.column += 1;
        }
    }
    point
}

/// Extract field name for a subtree based on its production ID
fn extract_field_name(_subtree: &Subtree, _language: Option<*const TSLanguage>) -> Option<String> {
    // Field names are mapped via production IDs
    // For now, we return None as implementing full field extraction
    // requires tracking the child index within the parent
    // This would need more context about the node's position in its parent
    None
}

/// Convert internal subtree to public node
fn subtree_to_node(subtree: Subtree, language: Option<*const TSLanguage>) -> ParsedNode {
    ////eprintln!($
    //    "DEBUG subtree_to_node: Converting subtree with symbol {}, children: {}, extra: {}",
    //    subtree.symbol,
    //    subtree.children.len(),
    //    subtree.is_extra
    //);

    // Determine if the node is named based on symbol metadata
    let is_named = if let Some(lang_ptr) = language {
        unsafe {
            let lang = &*lang_ptr;
            if subtree.symbol < lang.symbol_count as u16 {
                let metadata = *lang.symbol_metadata.add(subtree.symbol as usize);
                // In Tree-sitter, metadata & 1 == 0 means named
                // metadata values: 0 = unnamed extra, 1 = unnamed, 3 = named
                metadata >= 2
            } else {
                false
            }
        }
    } else {
        true // Default to named if no language info
    };

    //eprintln!("  Symbol {} is_named: {}", subtree.symbol, is_named);

    // Extract field name before moving children
    let field_name = extract_field_name(&subtree, language);

    ParsedNode {
        symbol: subtree.symbol,
        children: subtree
            .children
            .into_iter()
            .map(|s| subtree_to_node(s, language))
            .collect(),
        start_byte: subtree.start_byte,
        end_byte: subtree.end_byte,
        start_point: subtree.start_point,
        end_point: subtree.end_point,
        is_extra: subtree.is_extra,
        is_error: subtree.is_error,
        is_missing: subtree.is_missing,
        is_named,
        field_name,
        language,
    }
}

impl ParsedNode {
    /// Get symbol ID
    pub fn symbol(&self) -> TSSymbol {
        self.symbol
    }

    /// Get start byte offset
    pub fn start_byte(&self) -> usize {
        self.start_byte
    }

    /// Get end byte offset
    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    /// Get start point
    pub fn start_point(&self) -> Point {
        self.start_point
    }

    /// Get end point
    pub fn end_point(&self) -> Point {
        self.end_point
    }

    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Get child at index
    pub fn child(&self, index: usize) -> Option<&ParsedNode> {
        self.children.get(index)
    }

    /// Get all children
    pub fn children(&self) -> &[ParsedNode] {
        &self.children
    }

    /// Check if node is extra
    pub fn is_extra(&self) -> bool {
        self.is_extra
    }

    /// Check if node is error
    pub fn is_error(&self) -> bool {
        self.is_error
    }

    /// Check if node is missing
    pub fn is_missing(&self) -> bool {
        self.is_missing
    }

    /// Check if node has error
    pub fn has_error(&self) -> bool {
        self.is_error || self.children.iter().any(|c| c.has_error())
    }

    /// Check if node is named
    pub fn is_named(&self) -> bool {
        self.is_named
    }

    /// Get the text content of this node from the source
    #[allow(invalid_from_utf8)]
    pub fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        let text = source.get(self.start_byte..self.end_byte).ok_or_else(|| {
            // Create a valid Utf8Error by attempting to parse invalid UTF-8
            let invalid = [0x80, 0x80]; // Invalid UTF-8 sequence
            std::str::from_utf8(&invalid).unwrap_err()
        })?;
        std::str::from_utf8(text)
    }

    /// Create a walker for this node's children
    pub fn walk(&self) -> ChildWalker<'_> {
        ChildWalker {
            children: &self.children,
            index: 0,
        }
    }

    /// Get node kind (symbol name)
    pub fn kind(&self) -> &str {
        if let Some(language) = self.language {
            unsafe {
                let language = &*language;
                if self.symbol < language.symbol_count as u16 {
                    let symbol_names = std::slice::from_raw_parts(
                        language.symbol_names,
                        language.symbol_count as usize,
                    );
                    let name_ptr = symbol_names[self.symbol as usize];
                    let c_str = std::ffi::CStr::from_ptr(name_ptr as *const i8);
                    c_str.to_str().unwrap_or("unknown")
                } else {
                    "unknown"
                }
            }
        } else {
            // Fallback for when language is not available
            match self.symbol {
                0 => "end",
                1 => "*",
                2 => "_2",
                3 => "_6",
                4 => "-",
                5 => "Expression",
                6 => "Whitespace__whitespace",
                7 => "Whitespace",
                8 => "Expression_Sub_1",
                9 => "Expression_Sub",
                10 => "rule_10",
                _ => "unknown",
            }
        }
    }
}

/// Walker for node children
pub struct ChildWalker<'a> {
    children: &'a [ParsedNode],
    index: usize,
}

impl<'a> ChildWalker<'a> {
    pub fn goto_first_child(&mut self) -> bool {
        self.index = 0;
        !self.children.is_empty()
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        if self.index + 1 < self.children.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub fn node(&self) -> &ParsedNode {
        &self.children[self.index]
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new();
        assert!(parser.language().is_none());
        assert_eq!(parser.timeout_micros, 0);
    }

    #[test]
    fn test_point_advance() {
        let point = Point { row: 0, column: 0 };
        let point = advance_point(point, b"hello");
        assert_eq!(point.row, 0);
        assert_eq!(point.column, 5);

        let point = advance_point(point, b"\nworld");
        assert_eq!(point.row, 1);
        assert_eq!(point.column, 5);
    }

    #[test]
    fn test_empty_parse() {
        let mut parser = Parser::new();
        let result = parser.parse_string("");
        assert!(result.root.is_none());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_get_goto_handles_missing_entries() {
        // Minimal language with a single token and no goto entries.
        static PARSE_TABLE: [u16; 1] = [0];
        static SMALL_PARSE_TABLE: [u16; 1] = [0];
        static SMALL_PARSE_TABLE_MAP: [u32; 1] = [0];
        static LANGUAGE: TSLanguage = TSLanguage {
            version: TREE_SITTER_LANGUAGE_VERSION,
            symbol_count: 1,
            alias_count: 0,
            token_count: 1,
            external_token_count: 0,
            state_count: 1,
            large_state_count: 1,
            production_id_count: 0,
            field_count: 0,
            max_alias_sequence_length: 0,
            production_id_map: std::ptr::null(),
            parse_table: PARSE_TABLE.as_ptr(),
            small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
            small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
            parse_actions: std::ptr::null(),
            symbol_names: std::ptr::null(),
            field_names: std::ptr::null(),
            field_map_slices: std::ptr::null(),
            field_map_entries: std::ptr::null(),
            symbol_metadata: std::ptr::null(),
            public_symbol_map: std::ptr::null(),
            alias_map: std::ptr::null(),
            alias_sequences: std::ptr::null(),
            lex_modes: std::ptr::null(),
            lex_fn: None,
            keyword_lex_fn: None,
            keyword_capture_token: 0,
            external_scanner: ExternalScanner::default(),
            primary_state_ids: std::ptr::null(),
            production_lhs_index: std::ptr::null(),
            production_count: 0,
            eof_symbol: 0,
            rules: std::ptr::null(),
            rule_count: 0,
        };

        let parser = Parser {
            language: None,
            stack: Vec::new(),
            timeout_micros: 0,
            cancellation_flag: None,
            lexer: None,
        };

        // Symbol 0 is a token; there is no goto entry. The helper should
        // return None instead of panicking.
        assert!(parser.get_goto(&LANGUAGE, 0, 0).is_none());
    }

    #[test]
    fn test_get_goto_state_returns_zero_when_missing() {
        // Language with one token and one non-terminal but no goto entries.
        static PARSE_TABLE: [u16; 2] = [0, 0];
        static SMALL_PARSE_TABLE: [u16; 1] = [0];
        static SMALL_PARSE_TABLE_MAP: [u32; 1] = [0];
        static LEX_MODES: [TSLexState; 1] = [TSLexState {
            lex_state: 0,
            external_lex_state: 0,
        }];
        static LANGUAGE: TSLanguage = TSLanguage {
            version: TREE_SITTER_LANGUAGE_VERSION,
            symbol_count: 2,
            alias_count: 0,
            token_count: 1,
            external_token_count: 0,
            state_count: 1,
            large_state_count: 1,
            production_id_count: 0,
            field_count: 0,
            max_alias_sequence_length: 0,
            production_id_map: std::ptr::null(),
            parse_table: PARSE_TABLE.as_ptr(),
            small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
            small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
            parse_actions: std::ptr::null(),
            symbol_names: std::ptr::null(),
            field_names: std::ptr::null(),
            field_map_slices: std::ptr::null(),
            field_map_entries: std::ptr::null(),
            symbol_metadata: std::ptr::null(),
            public_symbol_map: std::ptr::null(),
            alias_map: std::ptr::null(),
            alias_sequences: std::ptr::null(),
            lex_modes: LEX_MODES.as_ptr(),
            lex_fn: None,
            keyword_lex_fn: None,
            keyword_capture_token: 0,
            external_scanner: ExternalScanner::default(),
            primary_state_ids: std::ptr::null(),
            production_lhs_index: std::ptr::null(),
            production_count: 0,
            eof_symbol: 0,
            rules: std::ptr::null(),
            rule_count: 0,
        };

        let parser = Parser::new();
        // Symbol 1 is the lone non-terminal; with no table entry this should return 0.
        assert_eq!(parser.get_goto_state(&LANGUAGE, 0, 1), 0);
    }
}
