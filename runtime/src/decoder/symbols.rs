use adze_ir::{SymbolId, Token, TokenPattern};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::ffi::{CStr, c_char};

use crate::pure_parser::TSLanguage;

pub(super) fn decode_symbol_names(lang: &TSLanguage) -> Vec<String> {
    let mut symbol_names = Vec::new();

    if lang.symbol_names.is_null() {
        for i in 0..lang.symbol_count as usize {
            symbol_names.push(format!("symbol_{}", i));
        }
        return symbol_names;
    }

    let symbol_count = lang.symbol_count as usize;
    if symbol_count == 0 {
        return symbol_names;
    }

    // SAFETY: `lang.symbol_names` is non-null (branch guard above), and
    // `symbol_count` matches `lang.symbol_count`. The TSLanguage contract
    // guarantees the array has at least `symbol_count` elements.
    let symbol_name_ptrs = unsafe { std::slice::from_raw_parts(lang.symbol_names, symbol_count) };

    for (i, &name_ptr) in symbol_name_ptrs.iter().enumerate() {
        let name = if name_ptr.is_null() {
            format!("symbol_{}", i)
        } else {
            // SAFETY: `name_ptr` is non-null (branch guard above) and points to a
            // null-terminated C string per TSLanguage contract.
            match unsafe { CStr::from_ptr(name_ptr as *const c_char) }.to_str() {
                Ok(valid_str) => valid_str.to_owned(),
                Err(_) => format!("symbol_invalid_{}", i),
            }
        };
        symbol_names.push(name);
    }

    symbol_names
}

pub(super) fn decode_tokens(
    lang: &TSLanguage,
    symbol_names: &[String],
    token_patterns: &HashMap<String, TokenPattern>,
) -> IndexMap<SymbolId, Token> {
    if !lang.symbol_metadata.is_null() {
        return decode_tokens_from_metadata(lang, symbol_names, token_patterns);
    }

    decode_tokens_from_names(lang, symbol_names, token_patterns)
}

fn decode_tokens_from_metadata(
    lang: &TSLanguage,
    symbol_names: &[String],
    token_patterns: &HashMap<String, TokenPattern>,
) -> IndexMap<SymbolId, Token> {
    let mut tokens = IndexMap::new();
    let symbol_count = lang.symbol_count as usize;
    if symbol_count == 0 {
        return tokens;
    }

    // SAFETY: `lang.symbol_metadata` is non-null (caller branch) and the
    // TSLanguage contract guarantees the array has `symbol_count` elements.
    let symbol_metadata_slice =
        unsafe { std::slice::from_raw_parts(lang.symbol_metadata, symbol_count) };

    for (i, &metadata) in symbol_metadata_slice.iter().enumerate() {
        if let Some(name) = symbol_names.get(i)
            && is_terminal(metadata, name)
        {
            insert_token(&mut tokens, SymbolId(i as u16), name, token_patterns);
        }
    }

    tokens
}

fn decode_tokens_from_names(
    lang: &TSLanguage,
    symbol_names: &[String],
    token_patterns: &HashMap<String, TokenPattern>,
) -> IndexMap<SymbolId, Token> {
    let mut tokens = IndexMap::new();

    for i in 0..lang.symbol_count as usize {
        let Some(name) = symbol_names.get(i) else {
            continue;
        };

        if is_likely_terminal_by_name(name) {
            insert_token(&mut tokens, SymbolId(i as u16), name, token_patterns);
        }
    }

    tokens
}

fn insert_token(
    tokens: &mut IndexMap<SymbolId, Token>,
    symbol_id: SymbolId,
    name: &str,
    token_patterns: &HashMap<String, TokenPattern>,
) {
    let pattern = token_patterns
        .get(name)
        .cloned()
        .unwrap_or_else(|| TokenPattern::String(name.to_owned()));

    tokens.insert(
        symbol_id,
        Token {
            name: name.to_owned(),
            pattern,
            fragile: false,
        },
    );
}

/// Determine if a symbol is a terminal based on metadata and name.
fn is_terminal(metadata: u8, name: &str) -> bool {
    if (metadata & 0x04) != 0 {
        return true;
    }

    if (metadata & 0x01) != 0 {
        if name.starts_with('_') && name[1..].chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        return true;
    }

    is_special_terminal_name(name)
}

/// Heuristic to determine if a symbol is likely a terminal when metadata is unavailable.
fn is_likely_terminal_by_name(name: &str) -> bool {
    if is_special_terminal_name(name) {
        return true;
    }

    if name.starts_with('_') && name[1..].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    if name.len() == 1 {
        return true;
    }

    name.len() <= 3
        && name
            .chars()
            .all(|c| !c.is_alphanumeric() && !c.is_whitespace())
}

fn is_special_terminal_name(name: &str) -> bool {
    name.starts_with("anon_sym_")
        || name.starts_with("aux_sym_")
        || name.starts_with("sym_")
        || name == "ERROR"
        || name.starts_with("ts_builtin_sym_")
        || matches!(
            name,
            "identifier"
                | "integer"
                | "float"
                | "string"
                | "comment"
                | "newline"
                | "indent"
                | "dedent"
                | "string_start"
                | "string_content"
                | "string_end"
        )
}
