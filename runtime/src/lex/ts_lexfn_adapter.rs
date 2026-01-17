use super::token_source::{Token, TokenSource};
use core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
/// Wrapper for the Tree-sitter lexer state value.
pub struct TSLexState(pub u32);

#[repr(C)]
/// FFI-compatible lexer structure used by Tree-sitter.
pub struct TsLexer {
    /// Function pointer to obtain the next lookahead character.
    pub lookahead: unsafe extern "C" fn(*mut TsLexer) -> u32,
    /// Function pointer to advance the lexer.
    pub advance: unsafe extern "C" fn(*mut TsLexer, bool),
    /// Function pointer to mark the end of the current token.
    pub mark_end: unsafe extern "C" fn(*mut TsLexer),
    /// Resulting symbol from the lexer.
    pub result_symbol: u16,
    /// Opaque pointer to our backing state.
    pub data: *mut c_void,
}

struct Backing<'a> {
    input: &'a [u8],
    pos: usize,
    mark: usize,
    // scratch for last token len computed from mark_end
    tok_len: usize,
}

// SAFETY: All extern fns below only touch `Backing` via `data`.
unsafe extern "C" fn lookahead(lex: *mut TsLexer) -> u32 {
    unsafe {
        let st = &*((*lex).data as *const Backing);
        if st.pos < st.input.len() {
            st.input[st.pos] as u32
        } else {
            0
        }
    }
}

unsafe extern "C" fn advance(lex: *mut TsLexer, skip: bool) {
    unsafe {
        let st = &mut *((*lex).data as *mut Backing);
        if !skip && st.pos < st.input.len() {
            st.pos += 1;
        } else if skip {
            // skip mode: still move forward one byte if any
            if st.pos < st.input.len() {
                st.pos += 1;
            }
        }
    }
}

unsafe extern "C" fn mark_end(lex: *mut TsLexer) {
    unsafe {
        let st = &mut *((*lex).data as *mut Backing);
        st.mark = st.pos;
    }
}

/// Adapter that exposes Tree-sitter's C-style lexing API over a byte slice.
pub struct TsLexFnAdapter<'a, S: Copy> {
    lang_lex: unsafe extern "C" fn(*mut c_void, S) -> bool,
    backing: Backing<'a>,
    ts: TsLexer,
    state_tag: S,
    // cache of last token
    look: Option<Token>,
}

impl<'a, S: Copy> TsLexFnAdapter<'a, S> {
    /// Create a new [`TsLexFnAdapter`] for the given input and language lexer.
    pub fn new(
        input: &'a [u8],
        lang_lex: unsafe extern "C" fn(*mut c_void, S) -> bool,
        initial_state: S,
    ) -> Self {
        let backing = Backing {
            input,
            pos: 0,
            mark: 0,
            tok_len: 0,
        };
        // create TsLexer pointing to backing
        let ts = TsLexer {
            lookahead,
            advance,
            mark_end,
            result_symbol: u16::MAX,
            data: std::ptr::null_mut(), // will be set below
        };

        let mut adapter = Self {
            lang_lex,
            backing,
            ts,
            state_tag: initial_state,
            look: None,
        };

        // Now set the data pointer to our backing
        adapter.ts.data = &mut adapter.backing as *mut _ as *mut c_void;

        adapter
    }

    /// Set the lexer state to use for the next token.
    pub fn set_state(&mut self, state: S) {
        self.state_tag = state;
        // Invalidate the lookahead token because it might have been lexed with the wrong state
        // if we are switching state before consuming it.
        // However, standard usage is: consume token, switch state, peek/consume next token.
        // If we peeked, then switched state, the peeked token is invalid.
        self.look = None;
    }

    fn next_internal(&mut self) -> Option<Token> {
        // skip whitespace up-front so scanners that don't skip still work
        while self.backing.pos < self.backing.input.len() {
            let c = self.backing.input[self.backing.pos];
            if matches!(c, b' ' | b'\n' | b'\r' | b'\t') {
                self.backing.pos += 1;
            } else {
                break;
            }
        }
        if self.backing.pos >= self.backing.input.len() {
            return None;
        }

        // Remember the start position before calling the lexer
        let token_start = self.backing.pos;

        // Prepare for a new token
        self.backing.mark = self.backing.pos;
        self.backing.tok_len = 0;
        self.ts.result_symbol = u16::MAX;

        // Update the data pointer to ensure it's pointing to our backing
        self.ts.data = &mut self.backing as *mut _ as *mut c_void;

        // Call the language lexer
        let ok = unsafe { (self.lang_lex)(&mut self.ts as *mut _ as *mut c_void, self.state_tag) };

        if ok && self.ts.result_symbol != u16::MAX {
            // The lexer should have called mark_end to indicate where the token ends
            // If mark_end > token_start, use that range. Otherwise use current pos.
            let end = if self.backing.mark > token_start {
                self.backing.mark
            } else {
                self.backing.pos
            };
            let len = end - token_start;
            let tok = Token {
                sym: self.ts.result_symbol,
                start: token_start,
                len,
            };
            self.backing.pos = end;
            Some(tok)
        } else {
            // Safety hatch to avoid infinite loops on "no token recognized"
            // Consume one byte as an error token
            let tok = Token {
                sym: u16::MAX,
                start: token_start,
                len: 1,
            };
            self.backing.pos = token_start + 1;
            Some(tok)
        }
    }
}

impl<'a, S: Copy> TokenSource for TsLexFnAdapter<'a, S> {
    fn peek(&mut self) -> Option<Token> {
        if self.look.is_none() {
            self.look = self.next_internal();
        }
        self.look
    }

    fn bump(&mut self) {
        self.look = None;
    }

    fn offset(&self) -> usize {
        self.backing.pos
    }
}
