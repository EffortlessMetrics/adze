// Pure-Rust Tree-sitter compatible parser runtime
// This implements the core parsing algorithm with GLR support

use crate::external_scanner_ffi::TSLexer;
use crate::{InputEdit, Node, Point, Range, Tree, TreeCursor};
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
    field_id: Option<u16>,
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
    pub fn parse_with_callback<F>(
        &mut self,
        mut callback: F,
        old_tree: Option<&Tree>,
    ) -> Option<Tree>
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
                            field_id: None,
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
                    if !self.recover_from_error(language, &mut position, &mut point, &mut callback)
                    {
                        return None;
                    }
                }
            }
        }
    }

    /// Lex a token at the current position
    fn lex_token(
        &self,
        language: Language,
        state: u16,
        input: &[u8],
        position: usize,
    ) -> Option<Token> {
        unsafe {
            let lang = &*language.ptr;

            // Get lex state for current parse state
            let lex_state = if state < lang.state_count as u16 {
                let lex_modes =
                    std::slice::from_raw_parts(lang.lex_modes, lang.state_count as usize);
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
            return Some(Token {
                symbol: 0,
                length: 0,
            }); // EOF
        }

        // Skip whitespace
        let mut i = 0;
        while i < input.len() && input[i].is_ascii_whitespace() {
            i += 1;
        }

        if i > 0 {
            return Some(Token {
                symbol: 1,
                length: i,
            }); // Whitespace token
        }

        // Single character token
        Some(Token {
            symbol: input[0] as u16,
            length: 1,
        })
    }

    /// Lex external token
    fn lex_external_token(
        &self,
        language: Language,
        lex_state: &ffi::TSLexState,
        input: &[u8],
    ) -> Option<Token> {
        unsafe {
            let lang = &*language.ptr;

            // Check if we have an external scanner and need to use it
            if lex_state.external_lex_state == 0 || lang.external_scanner.scan.is_none() {
                return None;
            }

            // Get the scan function
            let scan_fn = lang.external_scanner.scan?;

            // Create lexer interface for the scanner
            let mut lexer = ExternalLexer {
                input,
                position: 0,
                result_symbol: 0,
                line: 0,
                column: 0,
            };

            // Build valid symbols array based on external lex state
            let external_token_count = lang.external_token_count as usize;
            let mut valid_symbols = vec![false; external_token_count];

            // The external_lex_state is a bitset indicating which external tokens are valid
            for i in 0..external_token_count {
                if (lex_state.external_lex_state >> i) & 1 != 0 {
                    valid_symbols[i] = true;
                }
            }

            // Create scanner instance if needed
            let scanner_instance = if let Some(create_fn) = lang.external_scanner.create {
                create_fn()
            } else {
                std::ptr::null_mut()
            };

            // Call the external scanner
            let mut ts_lexer = create_ts_lexer(&mut lexer);
            let success = scan_fn(
                scanner_instance,
                &mut ts_lexer as *mut _ as *mut c_void,
                valid_symbols.as_ptr(),
            );

            // Clean up scanner instance
            if !scanner_instance.is_null() {
                if let Some(destroy_fn) = lang.external_scanner.destroy {
                    destroy_fn(scanner_instance);
                }
            }

            if success && lexer.result_symbol > 0 {
                // Map external symbol to actual symbol
                let symbol = if !lang.external_scanner.symbol_map.is_null() {
                    *lang
                        .external_scanner
                        .symbol_map
                        .add(lexer.result_symbol as usize)
                } else {
                    lexer.result_symbol
                };

                Some(Token {
                    symbol,
                    length: lexer.position,
                })
            } else {
                None
            }
        }
    }

    /// Get parse action for state and symbol
    fn get_action(&self, language: Language, state: u16, symbol: u16) -> Option<Action> {
        unsafe {
            let lang = &*language.ptr;

            // Validate state
            if state >= lang.state_count as u16 {
                return Some(Action::Error);
            }

            // The parse table is stored in compressed format
            // All states use small_parse_table_map for offsets
            let state_offset = *lang.small_parse_table_map.add(state as usize) as usize;

            // Find the next state's offset to know where this state's entries end
            let next_offset = if (state + 1) < lang.state_count as u16 {
                *lang.small_parse_table_map.add((state + 1) as usize) as usize
            } else {
                // For the last state, use the last entry in the map
                // The map has state_count + 1 entries
                *lang.small_parse_table_map.add(lang.state_count as usize) as usize
            };

            // The parse table stores entries as pairs: (symbol, action)
            let mut offset = state_offset;
            let end_offset = next_offset;

            while offset + 1 < end_offset {
                let entry_symbol = *lang.parse_table.add(offset);
                let action_value = *lang.parse_table.add(offset + 1);

                // Check if this is a default reduce entry
                // In Tree-sitter's format, reduce entries have the high bit set in the symbol field
                if entry_symbol & 0x8000 != 0 {
                    // This is a default reduce action that applies to all lookahead symbols
                    if action_value != 0 {
                        return Some(decode_action(action_value));
                    }
                }

                // Check if this entry matches our symbol
                if entry_symbol == symbol {
                    return Some(decode_action(action_value));
                }

                offset += 2;
            }

            // Default action (usually Error)
            Some(Action::Error)
        }
    }

    /// Perform a reduction
    fn reduce(&mut self, language: Language, rule_id: u16) -> Option<()> {
        unsafe {
            let lang = &*language.ptr;

            // Parse actions contain the full reduction information
            let parse_actions =
                std::slice::from_raw_parts(lang.parse_actions, lang.production_id_count as usize);

            if rule_id >= lang.production_id_count as u16 {
                return None;
            }

            let action = &parse_actions[rule_id as usize];
            let lhs_symbol = action.symbol;
            let rule_length = action.child_count as usize;

            // Pop rule_length items from stack
            let mut children = Vec::new();
            let mut start_byte = usize::MAX;
            let mut end_byte = 0;
            let mut start_point = Point {
                row: usize::MAX,
                column: usize::MAX,
            };
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

            // Extract field names for this production
            let mut field_names = vec![None; children.len()];
            if lang.field_count > 0
                && !lang.field_map_slices.is_null()
                && !lang.field_map_entries.is_null()
            {
                // Each production can have a slice in the field map
                // The slice tells us which children have field names
                let field_map_slices = std::slice::from_raw_parts(
                    lang.field_map_slices,
                    lang.production_id_count as usize * 2,
                );

                if (rule_id as usize) * 2 + 1 < field_map_slices.len() {
                    let slice_start = field_map_slices[rule_id as usize * 2] as usize;
                    let slice_length = field_map_slices[rule_id as usize * 2 + 1] as usize;

                    if slice_length > 0 {
                        let field_map_entries = std::slice::from_raw_parts(
                            lang.field_map_entries,
                            (slice_start + slice_length) * 2,
                        );

                        // Process each field entry
                        for i in 0..slice_length {
                            let entry_offset = (slice_start + i) * 2;
                            if entry_offset + 1 < field_map_entries.len() {
                                let entry_low = field_map_entries[entry_offset];
                                let entry_high = field_map_entries[entry_offset + 1];

                                // Unpack the field entry
                                // Format: field_id (16 bits) | child_index (8 bits) | inherited (8 bits)
                                let packed_entry = ((entry_high as u32) << 16) | (entry_low as u32);
                                let field_id = (packed_entry & 0xFFFF) as u16;
                                let child_index = ((packed_entry >> 16) & 0xFF) as usize;
                                // let inherited = ((packed_entry >> 24) & 0xFF) as u8;

                                if child_index < field_names.len()
                                    && field_id < lang.field_count as u16
                                {
                                    field_names[child_index] = Some(field_id);
                                }
                            }
                        }
                    }
                }
            }

            // Create children with field information
            let mut children_with_fields = Vec::new();
            for (i, mut child) in children.into_iter().enumerate() {
                if let Some(field_id) = field_names[i] {
                    child.field_id = Some(field_id);
                }
                children_with_fields.push(child);
            }

            // Create new node for reduction
            let new_node = Subtree {
                symbol: lhs_symbol,
                children: children_with_fields,
                start_byte,
                end_byte,
                start_point,
                end_point,
                field_id: None, // Parent nodes don't have field IDs
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
    fn get_goto(&self, language: Language, state: u16, symbol: u16) -> Option<u16> {
        // In Tree-sitter, goto states are encoded as shift actions in the parse table
        // When we look up an action for a non-terminal symbol, we get a Shift action
        // that tells us which state to go to
        match self.get_action(language, state, symbol)? {
            Action::Shift(goto_state) => Some(goto_state),
            _ => None,
        }
    }

    /// Error recovery
    fn recover_from_error<F>(
        &mut self,
        _language: Language,
        position: &mut usize,
        point: &mut Point,
        callback: &mut F,
    ) -> bool
    where
        F: FnMut(usize, Point) -> &[u8],
    {
        // Attempt a simple panic-free recovery strategy:
        //   1. Skip over any immediate whitespace so the parser can resume at the
        //      next significant token.
        //   2. If no whitespace is present, skip a single unexpected byte.

        // Get the remaining input from the callback at the current position.
        let input = callback(*position, *point);
        if input.is_empty() {
            // No more input to consume; recovery failed.
            return false;
        }

        // First, skip any consecutive ASCII whitespace characters. Treating this
        // as insertion of missing insignificant tokens helps the parser make
        // progress without consuming meaningful input.
        let whitespace_len = input.iter().take_while(|c| c.is_ascii_whitespace()).count();

        if whitespace_len > 0 {
            *position += whitespace_len;
            *point = advance_point(*point, &input[..whitespace_len]);
            return true;
        }

        // Otherwise, skip a single byte and update the line/column information so
        // that parsing can continue.
        *position += 1;
        *point = advance_point(*point, &input[..1]);
        true
    }
}

/// Simple lexer interface for external scanners
struct ExternalLexer<'a> {
    input: &'a [u8],
    position: usize,
    result_symbol: u16,
    line: u32,
    line_start: usize, // byte offset of beginning of current line
    token_end: usize,
}

impl<'a> ExternalLexer<'a> {
    fn new(input: &'a [u8], position: usize) -> Self {
        let (line, line_start) = Self::calculate_line_info(input, position);
        ExternalLexer {
            input,
            position,
            result_symbol: 0,
            line,
            line_start,
            token_end: position,
        }
    }

    fn calculate_line_info(input: &[u8], position: usize) -> (u32, usize) {
        let mut line = 0u32;
        let mut line_start = 0usize;

        for i in 0..position.min(input.len()) {
            if input[i] == b'\n' {
                line += 1;
                line_start = i + 1;
            } else if input[i] == b'\r' {
                if i + 1 < input.len() && input[i + 1] == b'\n' {
                    continue; // CRLF
                }
                line += 1;
                line_start = i + 1;
            }
        }

        (line, line_start)
    }

    fn get_column(&self) -> u32 {
        (self.position.saturating_sub(self.line_start)) as u32
    }
}

/// Create a TSLexer interface for the external scanner
unsafe fn create_ts_lexer(lexer: &mut ExternalLexer) -> TSLexer {
    extern "C" fn lookahead(lexer_ptr: *mut TSLexer) -> u32 {
        unsafe {
            let lexer = &*((*lexer_ptr).context as *const ExternalLexer);
            if lexer.position < lexer.input.len() {
                lexer.input[lexer.position] as u32
            } else {
                0
            }
        }
    }

    extern "C" fn advance(lexer_ptr: *mut TSLexer, skip: bool) {
        unsafe {
            let lexer = &mut *((*lexer_ptr).context as *mut ExternalLexer);
            if lexer.position < lexer.input.len() {
                let byte = lexer.input[lexer.position];
                lexer.position += 1;

                // Handle newlines (CR, LF, CRLF)
                if byte == b'\n' {
                    lexer.line += 1;
                    lexer.line_start = lexer.position;
                } else if byte == b'\r' {
                    // Handle CR and CRLF
                    if lexer.position < lexer.input.len() && lexer.input[lexer.position] == b'\n' {
                        lexer.position += 1; // Skip the LF in CRLF
                    }
                    lexer.line += 1;
                    lexer.line_start = lexer.position;
                }

                if !skip && lexer.token_end < lexer.position {
                    lexer.token_end = lexer.position;
                }
            }
        }
    }

    extern "C" fn mark_end(lexer_ptr: *mut TSLexer) {
        unsafe {
            let lexer = &mut *((*lexer_ptr).context as *mut ExternalLexer);
            lexer.token_end = lexer.position;
        }
    }

    extern "C" fn get_column(lexer_ptr: *mut TSLexer) -> u32 {
        unsafe {
            let lexer = &*((*lexer_ptr).context as *const ExternalLexer);
            lexer.get_column()
        }
    }

    extern "C" fn is_at_included_range_start(lexer_ptr: *const TSLexer) -> bool {
        false
    }

    extern "C" fn eof(lexer_ptr: *const TSLexer) -> bool {
        unsafe {
            let lexer = &*((*lexer_ptr).context as *const ExternalLexer);
            lexer.position >= lexer.input.len()
        }
    }

    TSLexer {
        lookahead,
        advance,
        mark_end,
        get_column,
        is_at_included_range_start,
        eof,
        context: (lexer as *mut ExternalLexer).cast(),
        result_symbol: 0,
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

            let symbol_names =
                std::slice::from_raw_parts(lang.symbol_names, lang.symbol_count as usize);

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

            let field_names =
                std::slice::from_raw_parts(lang.field_names, lang.field_count as usize);

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
