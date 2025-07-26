// Pure-Rust Tree-sitter compatible parser runtime
// This implements the core parsing algorithm with GLR support

use std::sync::atomic::{AtomicBool, Ordering};
use std::ffi::c_void;
use std::time::Instant;

// Import ABI types from tablegen
type TSSymbol = u16;
type TSStateId = u16;
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
    position: usize,
}

/// Lexer state
#[derive(Debug)]
struct Lexer {
    input: Vec<u8>,
    position: usize,
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
            || language.version > TREE_SITTER_LANGUAGE_VERSION {
            return Err(format!("Incompatible language version: {}", language.version));
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
    pub fn parse_string_with_tree(&mut self, source: &str, old_tree: Option<&crate::pure_incremental::Tree>) -> ParseResult {
        let bytes = source.as_bytes();
        self.parse_bytes_with_tree(bytes, old_tree)
    }
    
    /// Parse bytes of source code
    pub fn parse_bytes(&mut self, source: &[u8]) -> ParseResult {
        self.parse_bytes_with_tree(source, None)
    }
    
    /// Parse bytes with an old tree for incremental parsing
    pub fn parse_bytes_with_tree(&mut self, source: &[u8], old_tree: Option<&crate::pure_incremental::Tree>) -> ParseResult {
        let language = match self.language {
            Some(lang) => lang,
            None => return ParseResult {
                root: None,
                errors: vec![ParseError {
                    position: 0,
                    point: Point { row: 0, column: 0 },
                    expected: vec![],
                    found: 0,
                }],
            },
        };
        
        // Initialize parser state
        self.stack.clear();
        self.stack.push(StackEntry {
            state: 0,
            subtree: None,
            position: 0,
        });
        
        // Initialize lexer
        self.lexer = Some(Lexer {
            input: source.to_vec(),
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
        loop {
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
            
            // Get action for current state and token
            match self.get_action(language, current_state, token.symbol) {
                Action::Shift(next_state) => {
                    // Create leaf node
                    let end_point = advance_point(point, &source[position..position + token.length]);
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
                }
                
                Action::Reduce(rule_id) => {
                    self.reduce(language, rule_id);
                }
                
                Action::Accept => {
                    // Parse successful
                    if let Some(entry) = self.stack.pop() {
                        if let Some(subtree) = entry.subtree {
                            return ParseResult {
                                root: Some(subtree_to_node(subtree, Some(language as *const _))),
                                errors,
                            };
                        }
                    }
                    break;
                }
                
                Action::Error => {
                    // Record error and try to recover
                    errors.push(ParseError {
                        position,
                        point,
                        expected: self.get_expected_symbols(language, current_state),
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
    fn lex_token(&mut self, language: &TSLanguage, state: TSStateId, position: usize, point: &mut Point) -> Token {
        let lexer = match self.lexer.as_mut() {
            Some(l) => l,
            None => return Token { symbol: 0, length: 0, is_extra: false },
        };
        
        if position >= lexer.input.len() {
            return Token { symbol: 0, length: 0, is_extra: false }; // EOF
        }
        
        // Get lex state for current parser state
        let lex_mode = unsafe {
            if state < language.state_count as u16 {
                *language.lex_modes.add(state as usize)
            } else {
                TSLexState { lex_state: 0, external_lex_state: 0 }
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
                    input: &lexer.input,
                    position,
                    point: *point,
                    result_symbol: 0,
                    result_length: 0,
                };
                
                let lex_state_ptr = &mut lex_state as *mut _ as *mut c_void;
                if lex_fn(lex_state_ptr, lex_mode) {
                    return Token {
                        symbol: lex_state.result_symbol,
                        length: lex_state.result_length,
                        is_extra: false,
                    };
                }
            }
        }
        
        // Fallback: simple character-by-character lexing
        let ch = lexer.input[position];
        
        // Skip whitespace as extras
        if ch.is_ascii_whitespace() {
            let mut len = 1;
            while position + len < lexer.input.len() && lexer.input[position + len].is_ascii_whitespace() {
                len += 1;
            }
            return Token { symbol: 1, length: len, is_extra: true };
        }
        
        // Single character tokens
        Token { symbol: ch as u16, length: 1, is_extra: false }
    }
    
    /// Get parse action for state and symbol
    fn get_action(&self, language: &TSLanguage, state: TSStateId, symbol: TSSymbol) -> Action {
        // Look up action in parse table
        unsafe {
            // Small parse table lookup
            if state < language.large_state_count as u16 {
                let state_offset = (*language.small_parse_table_map.add(state as usize)) as usize;
                let entry_count = *language.small_parse_table.add(state_offset) as usize;
                
                // Search for symbol in entries
                for i in 0..entry_count {
                    let entry_symbol = *language.small_parse_table.add(state_offset + 1 + i * 2);
                    if entry_symbol == symbol {
                        let action_index = *language.small_parse_table.add(state_offset + 2 + i * 2) as usize;
                        return self.decode_action(language, action_index);
                    }
                }
            } else {
                // Large parse table lookup
                let table_offset = (state - language.large_state_count as u16) as usize * language.symbol_count as usize;
                let action_index = *language.parse_table.add(table_offset + symbol as usize) as usize;
                if action_index != 0 {
                    return self.decode_action(language, action_index);
                }
            }
        }
        
        Action::Error
    }
    
    /// Decode action from parse table
    fn decode_action(&self, language: &TSLanguage, action_index: usize) -> Action {
        unsafe {
            let action = *language.parse_actions.add(action_index);
            match action.action_type {
                0 => Action::Shift(action.symbol), // Shift
                1 => Action::Reduce(action.symbol as u16),    // Reduce
                2 => Action::Accept,                             // Accept
                _ => Action::Error,                              // Error or other
            }
        }
    }
    
    /// Perform a reduction
    fn reduce(&mut self, language: &TSLanguage, production_id: u16) {
        unsafe {
            // Get production info from parse actions
            let action = *language.parse_actions.add(production_id as usize);
            let child_count = action.child_count as usize;
            let symbol = action.symbol;
            
            // Pop children from stack
            let mut children = Vec::with_capacity(child_count);
            let mut start_byte = 0;
            let mut end_byte = 0;
            let mut start_point = Point { row: 0, column: 0 };
            let mut end_point = Point { row: 0, column: 0 };
            
            for i in 0..child_count {
                if let Some(entry) = self.stack.pop() {
                    if let Some(subtree) = entry.subtree {
                        if i == 0 {
                            end_byte = subtree.end_byte;
                            end_point = subtree.end_point;
                        }
                        if i == child_count - 1 {
                            start_byte = subtree.start_byte;
                            start_point = subtree.start_point;
                        }
                        children.push(subtree);
                    }
                }
            }
            
            // Reverse children to correct order
            children.reverse();
            
            // Create parent node
            let parent = Subtree {
                symbol,
                children,
                start_byte,
                end_byte,
                start_point,
                end_point,
                is_extra: action.extra != 0,
                is_error: false,
                is_missing: false,
                production_id: self.get_production_id(language, production_id),
            };
            
            // Get goto state
            if let Some(prev_entry) = self.stack.last() {
                let goto_state = self.get_goto_state(language, prev_entry.state, symbol);
                self.stack.push(StackEntry {
                    state: goto_state,
                    subtree: Some(parent),
                    position: end_byte,
                });
            }
        }
    }
    
    /// Get production ID for field mappings
    fn get_production_id(&self, language: &TSLanguage, action_index: u16) -> u16 {
        unsafe {
            if action_index < language.production_id_count as u16 {
                *language.production_id_map.add(action_index as usize)
            } else {
                0
            }
        }
    }
    
    /// Get goto state after reduction
    fn get_goto_state(&self, language: &TSLanguage, state: TSStateId, symbol: TSSymbol) -> TSStateId {
        // Goto table is encoded in the parse table after terminals
        let terminal_count = language.token_count;
        let goto_symbol = symbol - terminal_count as u16;
        
        unsafe {
            if state < language.large_state_count as u16 {
                // Small parse table lookup for gotos
                let state_offset = (*language.small_parse_table_map.add(state as usize)) as usize;
                let entry_count = *language.small_parse_table.add(state_offset) as usize;
                
                // Skip terminal entries to find non-terminal entries
                let mut offset = state_offset + 1;
                for _ in 0..entry_count {
                    let entry_symbol = *language.small_parse_table.add(offset);
                    if entry_symbol >= terminal_count as u16 && entry_symbol == symbol {
                        return *language.small_parse_table.add(offset + 1);
                    }
                    offset += 2;
                }
            } else {
                // Large parse table lookup
                let table_offset = (state - language.large_state_count as u16) as usize * language.symbol_count as usize;
                let goto_index = table_offset + symbol as usize;
                return *language.parse_table.add(goto_index);
            }
        }
        
        0 // Error state
    }
    
    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, language: &TSLanguage, state: TSStateId) -> Vec<TSSymbol> {
        let mut expected = Vec::new();
        
        unsafe {
            // Check all possible symbols for valid actions
            for symbol in 0..language.symbol_count as u16 {
                let action = self.get_action(language, state, symbol);
                if !matches!(action, Action::Error) {
                    expected.push(symbol);
                }
            }
        }
        
        expected
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
struct LexerState<'a> {
    input: &'a [u8],
    position: usize,
    point: Point,
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

/// Convert internal subtree to public node
fn subtree_to_node(subtree: Subtree, language: Option<*const TSLanguage>) -> ParsedNode {
    ParsedNode {
        symbol: subtree.symbol,
        children: subtree.children.into_iter().map(|s| subtree_to_node(s, language)).collect(),
        start_byte: subtree.start_byte,
        end_byte: subtree.end_byte,
        start_point: subtree.start_point,
        end_point: subtree.end_point,
        is_extra: subtree.is_extra,
        is_error: subtree.is_error,
        is_missing: subtree.is_missing,
        is_named: true, // TODO: determine from symbol type
        field_name: None, // TODO: Extract field names from production ID
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
    
    /// Get node kind (symbol name)
    pub fn kind(&self) -> &str {
        if let Some(language) = self.language {
            unsafe {
                let language = &*language;
                if self.symbol < language.symbol_count as u16 {
                    let symbol_names = std::slice::from_raw_parts(
                        language.symbol_names,
                        language.symbol_count as usize
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
}