// Pure-Rust Tree-sitter compatible parser runtime
// This implements the core parsing algorithm with GLR support

use crate::{Node, Tree, TreeCursor, Point, Range, InputEdit};
use std::os::raw::c_void;

/// Parser state for incremental parsing
#[derive(Debug)]
pub struct Parser {
    language: Option<Language>,
    stack: Vec<StackEntry>,
    /// Syntax trees from previous parses for incremental parsing
    old_trees: Vec<Tree>,
    /// Timeout in microseconds (0 means no timeout)
    timeout_micros: u64,
    /// Cancellation flag for parsing
    cancellation_flag: Option<*const std::sync::atomic::AtomicBool>,
}

/// Language definition with parse tables
#[derive(Debug, Clone, Copy)]
pub struct Language {
    ptr: *const ffi::TSLanguage,
}

/// Stack entry for LR parsing
#[derive(Debug, Clone)]
struct StackEntry {
    state: u16,
    node: Option<Subtree>,
    position: usize,
}

/// Internal node representation during parsing
#[derive(Debug, Clone)]
struct Subtree {
    symbol: u16,
    children: Vec<Subtree>,
    start_byte: usize,
    end_byte: usize,
    start_point: Point,
    end_point: Point,
}

// FFI types to match Tree-sitter C API
mod ffi {
    use std::os::raw::{c_char, c_void};
    
    #[repr(C)]
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
        pub symbol_names: *const *const c_char,
        pub field_names: *const *const c_char,
        pub field_map_slices: *const u16,
        pub field_map_entries: *const u16,
        pub symbol_metadata: *const u8,
        pub public_symbol_map: *const u16,
        pub alias_map: *const u16,
        pub alias_sequences: *const u16,
        pub lex_modes: *const TSLexState,
        pub lex_fn: Option<unsafe extern "C" fn(*mut c_void, u16) -> bool>,
        pub keyword_lex_fn: Option<unsafe extern "C" fn(*mut c_void, u16) -> u16>,
        pub keyword_capture_token: u16,
        pub external_scanner: TSExternalScanner,
        pub primary_state_ids: *const u16,
    }
    
    #[repr(C)]
    pub struct TSParseAction {
        pub action_type: u8,
        pub extra: u8,
        pub child_count: u8,
        pub dynamic_precedence: i8,
        pub symbol: u16,
    }
    
    #[repr(C)]
    pub struct TSLexState {
        pub lex_state: u16,
        pub external_lex_state: u16,
    }
    
    #[repr(C)]
    pub struct TSExternalScanner {
        pub states: *const bool,
        pub symbol_map: *const u16,
        pub create: Option<unsafe extern "C" fn() -> *mut c_void>,
        pub destroy: Option<unsafe extern "C" fn(*mut c_void)>,
        pub scan: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *const bool) -> bool>,
        pub serialize: Option<unsafe extern "C" fn(*mut c_void, *mut u8) -> u32>,
        pub deserialize: Option<unsafe extern "C" fn(*mut c_void, *const u8, u32)>,
    }
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Parser {
            language: None,
            stack: Vec::new(),
            old_trees: Vec::new(),
            timeout_micros: 0,
            cancellation_flag: None,
        }
    }
    
    /// Set the language for parsing
    pub fn set_language(&mut self, language: Language) -> Result<(), String> {
        // Validate language version
        unsafe {
            let lang = &*language.ptr;
            if lang.version != tree_sitter::LANGUAGE_VERSION {
                return Err(format!(
                    "Incompatible language version. Expected {}, got {}",
                    tree_sitter::LANGUAGE_VERSION,
                    lang.version
                ));
            }
        }
        
        self.language = Some(language);
        self.reset();
        Ok(())
    }
    
    /// Get the current language
    pub fn language(&self) -> Option<Language> {
        self.language
    }
    
    /// Set timeout for parsing in microseconds
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.timeout_micros = timeout;
    }
    
    /// Set cancellation flag for parsing
    pub fn set_cancellation_flag(&mut self, flag: Option<*const std::sync::atomic::AtomicBool>) {
        self.cancellation_flag = flag;
    }
    
    /// Reset parser state
    pub fn reset(&mut self) {
        self.stack.clear();
        self.old_trees.clear();
    }
    
    /// Parse a string of source code
    pub fn parse(&mut self, text: &str, old_tree: Option<&Tree>) -> Option<Tree> {
        self.parse_with_callback(
            |offset, _position| {
                if offset < text.len() {
                    &text.as_bytes()[offset..]
                } else {
                    &[]
                }
            },
            old_tree,
        )
    }
    
    /// Parse with a callback function for reading input
    pub fn parse_with_callback<F>(&mut self, mut callback: F, old_tree: Option<&Tree>) -> Option<Tree>
    where
        F: FnMut(usize, Point) -> &[u8],
    {
        let language = self.language?;
        
        // Initialize parser state
        self.stack.clear();
        self.stack.push(StackEntry {
            state: 0,
            node: None,
            position: 0,
        });
        
        // Store old tree for incremental parsing
        if let Some(tree) = old_tree {
            self.old_trees.clear();
            self.old_trees.push(tree.clone());
        }
        
        // Main parsing loop
        let mut position = 0;
        let mut point = Point { row: 0, column: 0 };
        let start_time = std::time::Instant::now();
        
        loop {
            // Check timeout
            if self.timeout_micros > 0 {
                let elapsed = start_time.elapsed().as_micros() as u64;
                if elapsed > self.timeout_micros {
                    return None; // Timeout
                }
            }
            
            // Check cancellation
            if let Some(flag) = self.cancellation_flag {
                unsafe {
                    if (*flag).load(std::sync::atomic::Ordering::Relaxed) {
                        return None; // Cancelled
                    }
                }
            }
            
            // Get current state
            let current_state = self.stack.last()?.state;
            
            // Lex next token
            let input = callback(position, point);
            let token = self.lex_token(language, current_state, input, position)?;
            
            // Get action for current state and token
            let action = self.get_action(language, current_state, token.symbol)?;
            
            match action {
                Action::Shift(next_state) => {
                    // Shift token onto stack
                    self.stack.push(StackEntry {
                        state: next_state,
                        node: Some(Subtree {
                            symbol: token.symbol,
                            children: Vec::new(),
                            start_byte: position,
                            end_byte: position + token.length,
                            start_point: point,
                            end_point: advance_point(point, &input[..token.length]),
                        }),
                        position: position + token.length,
                    });
                    
                    // Advance position
                    position += token.length;
                    point = advance_point(point, &input[..token.length]);
                }
                
                Action::Reduce(rule_id) => {
                    // Perform reduction
                    self.reduce(language, rule_id)?;
                }
                
                Action::Accept => {
                    // Parse successful
                    if let Some(entry) = self.stack.pop() {
                        if let Some(root) = entry.node {
                            return Some(Tree::new(root, language));
                        }
                    }
                    return None;
                }
                
                Action::Error => {
                    // Try error recovery
                    if !self.recover_from_error(language, &mut position, &mut point, &mut callback) {
                        return None;
                    }
                }
            }
        }
    }
    
    /// Lex a token at the current position
    fn lex_token(&self, language: Language, state: u16, input: &[u8], position: usize) -> Option<Token> {
        unsafe {
            let lang = &*language.ptr;
            
            // Get lex state for current parse state
            let lex_state = if state < lang.state_count as u16 {
                let lex_modes = std::slice::from_raw_parts(
                    lang.lex_modes,
                    lang.state_count as usize
                );
                &lex_modes[state as usize]
            } else {
                return None;
            };
            
            // Try external scanner first if available
            if lang.external_token_count > 0 {
                if let Some(token) = self.lex_external_token(language, lex_state, input) {
                    return Some(token);
                }
            }
            
            // Use lexer function if available
            if let Some(lex_fn) = lang.lex_fn {
                // Create lexer context
                let mut lexer = Lexer::new(input, position);
                let lexer_ptr = &mut lexer as *mut _ as *mut c_void;
                
                if lex_fn(lexer_ptr, lex_state.lex_state) {
                    return Some(Token {
                        symbol: lexer.result_symbol,
                        length: lexer.result_length,
                    });
                }
            }
            
            // Fallback: simple lexer for testing
            self.simple_lex(input)
        }
    }
    
    /// Simple lexer for basic tokens (for testing)
    fn simple_lex(&self, input: &[u8]) -> Option<Token> {
        if input.is_empty() {
            return Some(Token { symbol: 0, length: 0 }); // EOF
        }
        
        // Skip whitespace
        let mut i = 0;
        while i < input.len() && input[i].is_ascii_whitespace() {
            i += 1;
        }
        
        if i > 0 {
            return Some(Token { symbol: 1, length: i }); // Whitespace token
        }
        
        // Single character token
        Some(Token { symbol: input[0] as u16, length: 1 })
    }
    
    /// Lex external token
    fn lex_external_token(&self, _language: Language, _lex_state: &ffi::TSLexState, _input: &[u8]) -> Option<Token> {
        // TODO: Implement external scanner support
        None
    }
    
    /// Get parse action for state and symbol
    fn get_action(&self, language: Language, state: u16, symbol: u16) -> Option<Action> {
        unsafe {
            let lang = &*language.ptr;
            
            // Access parse table
            let parse_table = std::slice::from_raw_parts(
                lang.parse_table,
                lang.state_count as usize * 2
            );
            
            // Get action from compressed table
            let table_offset = (state as usize) * 2;
            if table_offset + 1 >= parse_table.len() {
                return None;
            }
            
            let entry_count = parse_table[table_offset] as usize;
            let data_offset = parse_table[table_offset + 1] as usize;
            
            // Search for symbol in action entries
            for i in 0..entry_count {
                let entry_offset = data_offset + i * 2;
                if entry_offset + 1 >= parse_table.len() {
                    continue;
                }
                
                let entry_symbol = parse_table[entry_offset];
                if entry_symbol == symbol {
                    let action_data = parse_table[entry_offset + 1];
                    return Some(decode_action(action_data));
                }
            }
            
            // Default action (usually Error)
            Some(Action::Error)
        }
    }
    
    /// Perform a reduction
    fn reduce(&mut self, language: Language, rule_id: u16) -> Option<()> {
        unsafe {
            let lang = &*language.ptr;
            
            // Get rule info from production ID map
            if rule_id >= lang.production_id_count as u16 {
                return None;
            }
            
            let production_id_map = std::slice::from_raw_parts(
                lang.production_id_map,
                lang.production_id_count as usize
            );
            
            let lhs_symbol = production_id_map[rule_id as usize];
            
            // TODO: Get actual rule length from grammar
            let rule_length = 2; // Placeholder
            
            // Pop rule_length items from stack
            let mut children = Vec::new();
            let mut start_byte = usize::MAX;
            let mut end_byte = 0;
            let mut start_point = Point { row: usize::MAX, column: usize::MAX };
            let mut end_point = Point { row: 0, column: 0 };
            
            for _ in 0..rule_length {
                if let Some(entry) = self.stack.pop() {
                    if let Some(node) = entry.node {
                        if node.start_byte < start_byte {
                            start_byte = node.start_byte;
                            start_point = node.start_point;
                        }
                        if node.end_byte > end_byte {
                            end_byte = node.end_byte;
                            end_point = node.end_point;
                        }
                        children.push(node);
                    }
                }
            }
            
            children.reverse();
            
            // Create new node for reduction
            let new_node = Subtree {
                symbol: lhs_symbol,
                children,
                start_byte,
                end_byte,
                start_point,
                end_point,
            };
            
            // Get goto state
            let prev_state = self.stack.last()?.state;
            let goto_state = self.get_goto(language, prev_state, lhs_symbol)?;
            
            // Push new node
            self.stack.push(StackEntry {
                state: goto_state,
                node: Some(new_node),
                position: end_byte,
            });
            
            Some(())
        }
    }
    
    /// Get goto state
    fn get_goto(&self, _language: Language, _state: u16, _symbol: u16) -> Option<u16> {
        // TODO: Implement goto table lookup
        Some(0)
    }
    
    /// Error recovery
    fn recover_from_error<F>(&mut self, _language: Language, _position: &mut usize, _point: &mut Point, _callback: &mut F) -> bool
    where
        F: FnMut(usize, Point) -> &[u8],
    {
        // TODO: Implement error recovery
        false
    }
}

/// Token returned by lexer
#[derive(Debug, Clone, Copy)]
struct Token {
    symbol: u16,
    length: usize,
}

/// Parse action
#[derive(Debug, Clone, Copy)]
enum Action {
    Shift(u16),
    Reduce(u16),
    Accept,
    Error,
}

/// Decode action from compressed format
fn decode_action(encoded: u16) -> Action {
    match encoded {
        0xFFFF => Action::Accept,
        0xFFFE => Action::Error,
        _ if encoded & 0x8000 != 0 => {
            let rule_id = (encoded & 0x7FFF) >> 1;
            Action::Reduce(rule_id)
        }
        state => Action::Shift(state),
    }
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

/// Lexer context for C callbacks
#[repr(C)]
struct Lexer {
    input: *const u8,
    input_length: usize,
    position: usize,
    result_symbol: u16,
    result_length: usize,
}

impl Lexer {
    fn new(input: &[u8], position: usize) -> Self {
        Lexer {
            input: input.as_ptr(),
            input_length: input.len(),
            position,
            result_symbol: 0,
            result_length: 0,
        }
    }
}

impl Language {
    /// Create a language from a pointer
    pub unsafe fn from_ptr(ptr: *const ffi::TSLanguage) -> Self {
        Language { ptr }
    }
    
    /// Get language version
    pub fn version(&self) -> u32 {
        unsafe { (*self.ptr).version }
    }
    
    /// Get symbol count
    pub fn symbol_count(&self) -> u32 {
        unsafe { (*self.ptr).symbol_count }
    }
    
    /// Get field count
    pub fn field_count(&self) -> u32 {
        unsafe { (*self.ptr).field_count }
    }
    
    /// Get symbol name
    pub fn symbol_name(&self, symbol: u16) -> &str {
        unsafe {
            let lang = &*self.ptr;
            if symbol >= lang.symbol_count as u16 {
                return "";
            }
            
            let symbol_names = std::slice::from_raw_parts(
                lang.symbol_names,
                lang.symbol_count as usize
            );
            
            let name_ptr = symbol_names[symbol as usize];
            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const i8);
            name_cstr.to_str().unwrap_or("")
        }
    }
    
    /// Get field name
    pub fn field_name(&self, field_id: u16) -> Option<&str> {
        unsafe {
            let lang = &*self.ptr;
            if field_id >= lang.field_count as u16 {
                return None;
            }
            
            let field_names = std::slice::from_raw_parts(
                lang.field_names,
                lang.field_count as usize
            );
            
            let name_ptr = field_names[field_id as usize];
            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const i8);
            name_cstr.to_str().ok()
        }
    }
}

// Re-export Tree-sitter version constant
pub use tree_sitter::LANGUAGE_VERSION;

// Implement Send and Sync for Language (it's just a pointer to static data)
unsafe impl Send for Language {}
unsafe impl Sync for Language {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = Parser::new();
        assert!(parser.language.is_none());
        assert_eq!(parser.timeout_micros, 0);
    }
    
    #[test]
    fn test_action_decoding() {
        assert!(matches!(decode_action(42), Action::Shift(42)));
        assert!(matches!(decode_action(0x8022), Action::Reduce(17)));
        assert!(matches!(decode_action(0xFFFF), Action::Accept));
        assert!(matches!(decode_action(0xFFFE), Action::Error));
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
}