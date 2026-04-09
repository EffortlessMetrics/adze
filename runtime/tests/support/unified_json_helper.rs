use adze::pure_parser::TSLanguage;
use adze_glr_core::GotoIndexing;
use adze_glr_core::{Action, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{Grammar, StateId, SymbolId, Token, TokenPattern};
use anyhow::Result;
#[cfg(all(feature = "pure-rust", feature = "with-grammars"))]
use ts_bridge::{extract, schema::Action as TsAction};

#[cfg(all(feature = "pure-rust", feature = "with-grammars"))]
use crate::language_builder;

/// Return a `TSLanguage` built from the real Tree-sitter JSON grammar.
///
/// Rather than casting the upstream `tree_sitter_json` pointer into our
/// `TSLanguage` (which has a different ABI), we use the `ts-bridge` extractor
/// to decode the Tree-sitter parse tables and rebuild a fresh language using
/// our pure-Rust layout.
#[allow(dead_code)]
#[cfg(all(feature = "pure-rust", feature = "with-grammars"))]
pub fn unified_json_language() -> Result<&'static TSLanguage, anyhow::Error> {
    // Extract parse table data from upstream Tree-sitter JSON grammar
    // tree_sitter_json::LANGUAGE.into_raw() returns a function pointer that needs to be called
    let raw_lang_fn = tree_sitter_json::LANGUAGE.into_raw();

    // Add ABI compatibility verification before any unsafe operations
    ts_bridge::ffi::assert_abi_compatible();

    // Convert function pointer with proper safety checks
    // The returned function has type: unsafe extern "C" fn() -> *const tree_sitter::Language
    // We need to transmute to the ts_bridge expected signature
    let lang_fn: unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage = {
        // Validate pointer size compatibility
        if std::mem::size_of_val(&raw_lang_fn)
            != std::mem::size_of::<unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage>()
        {
            return Err(anyhow::anyhow!(
                "Function pointer size mismatch: got {} bytes, expected {} bytes",
                std::mem::size_of_val(&raw_lang_fn),
                std::mem::size_of::<unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage>()
            ));
        }
        // Use transmute (not transmute_copy) for function pointers - they're Copy by nature
        unsafe { std::mem::transmute(raw_lang_fn) }
    };

    // Call the function to get the actual language pointer and validate it
    let lang_ptr = unsafe { lang_fn() };
    if lang_ptr.is_null() {
        return Err(anyhow::anyhow!(
            "Tree-sitter JSON language pointer is null after function call"
        ));
    }
    eprintln!("Language Pointer from function: {:p}", lang_ptr);

    let data = extract(lang_fn)
        .map_err(|e| anyhow::anyhow!("Failed to extract tree-sitter json: {}", e))?;

    // Find the document symbol - this should be the start symbol for JSON
    let mut document_id = None;
    for (i, sym) in data.symbols.iter().enumerate() {
        if sym.name == "document" {
            document_id = Some(i as u16);
            break;
        }
    }

    // Build minimal Grammar with symbol names and token stubs
    let mut grammar = Grammar::new("ts_json".to_string());
    for (i, sym) in data.symbols.iter().enumerate() {
        let sid = SymbolId(i as u16);
        grammar.rule_names.insert(sid, sym.name.clone());
        if (i as u32) < data.token_count + data.external_token_count {
            grammar.tokens.insert(
                sid,
                Token {
                    name: sym.name.clone(),
                    pattern: TokenPattern::String(sym.name.clone()),
                    fragile: false,
                },
            );
        }
    }

    // Add the EOF symbol to the grammar as well
    if data.eof_symbol as usize >= data.symbols.len() {
        let eof_sid = SymbolId(data.eof_symbol);
        grammar.rule_names.insert(eof_sid, "EOF".to_string());
        grammar.tokens.insert(
            eof_sid,
            Token {
                name: "EOF".to_string(),
                pattern: TokenPattern::String("EOF".to_string()),
                fragile: false,
            },
        );
    }

    // Convert extracted data into our ParseTable representation
    let state_count = data.state_count as usize;
    let symbol_count = data.symbol_count as usize;
    let mut table = ParseTable {
        action_table: vec![vec![Vec::new(); symbol_count]; state_count],
        goto_table: vec![vec![StateId(0); symbol_count]; state_count],
        symbol_metadata: Vec::with_capacity(symbol_count),
        state_count,
        symbol_count,
        symbol_to_index: std::collections::BTreeMap::new(),
        index_to_symbol: Vec::with_capacity(symbol_count),
        external_scanner_states: vec![vec![false; data.external_token_count as usize]; state_count],
        rules: data
            .rules
            .iter()
            .map(|r| ParseRule {
                lhs: SymbolId(r.lhs),
                rhs_len: r.rhs_len,
            })
            .collect(),
        nonterminal_to_index: std::collections::BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(data.eof_symbol),
        start_symbol: SymbolId(document_id.unwrap_or(data.start_symbol)), // Use document symbol if found
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: data.token_count as usize,
        external_token_count: data.external_token_count as usize,
        lex_modes: vec![
            adze_glr_core::LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: Vec::new(),
        dynamic_prec_by_rule: vec![0; data.rules.len()],
        rule_assoc_by_rule: vec![0; data.rules.len()],
        alias_sequences: vec![vec![]; data.rules.len()],
        field_names: Vec::new(),
        field_map: std::collections::BTreeMap::new(),
    };

    for (i, sym) in data.symbols.iter().enumerate() {
        let is_terminal = (i as u32) < data.token_count + data.external_token_count;
        table.symbol_metadata.push(SymbolMetadata {
            name: sym.name.clone(),
            is_visible: sym.visible,
            is_named: sym.named,
            is_supertype: false,
            // Additional fields required by GLR core API contracts
            is_terminal,
            is_extra: false, // TODO: determine if this symbol is extra
            is_fragile: false,
            symbol_id: SymbolId(i as u16),
        });
        let sid = SymbolId(i as u16);
        table.symbol_to_index.insert(sid, i);
        table.index_to_symbol.push(sid);
        if (i as u32) >= data.token_count + data.external_token_count {
            table.nonterminal_to_index.insert(sid, i);
        }
    }

    // Add EOF symbol if it's beyond the original symbols
    if data.eof_symbol as usize >= data.symbols.len() {
        let eof_sid = SymbolId(data.eof_symbol);
        let eof_index = data.eof_symbol as usize;
        table.symbol_metadata.push(SymbolMetadata {
            name: "EOF".to_string(),
            is_visible: false,
            is_named: true,
            is_supertype: false,
            // Additional fields required by GLR core API contracts
            is_terminal: true, // EOF is typically a terminal
            is_extra: false,
            is_fragile: false,
            symbol_id: eof_sid,
        });
        table.symbol_to_index.insert(eof_sid, eof_index);
        // Note: don't push to index_to_symbol since it should maintain symbol_count size
    }

    for cell in &data.actions {
        // Map symbol correctly to handle EOF properly
        let sym = if cell.symbol == data.eof_symbol {
            // For EOF actions, we need to check if EOF is in bounds
            if (data.eof_symbol as usize) < symbol_count {
                data.eof_symbol
            } else {
                // Skip out-of-bounds EOF actions for now
                continue;
            }
        } else {
            cell.symbol
        };

        if (sym as usize) >= symbol_count {
            eprintln!(
                "Warning: symbol {} is out of bounds for symbol_count {}",
                sym, symbol_count
            );
            continue;
        }

        let row = &mut table.action_table[cell.state as usize][sym as usize];
        for a in &cell.actions {
            row.push(match a {
                TsAction::Shift { state, .. } => Action::Shift(StateId(*state)),
                TsAction::Reduce { rule, .. } => Action::Reduce(adze_ir::RuleId(*rule)),
                TsAction::Accept => Action::Accept,
                TsAction::Recover => Action::Recover,
            });
        }
    }

    for cell in &data.gotos {
        if let Some(next) = cell.next_state {
            table.goto_table[cell.state as usize][cell.symbol as usize] = StateId(next);
        }
    }

    // Normalize for Tree-sitter layout and build final language
    language_builder::normalize_table_for_ts(&mut table);

    let lang = language_builder::build_json_ts_language(&grammar, &table);
    Ok(Box::leak(Box::new(lang)))
}

#[allow(dead_code)]
#[cfg(not(all(feature = "pure-rust", feature = "with-grammars")))]
pub fn unified_json_language() -> Result<&'static TSLanguage, anyhow::Error> {
    Err(anyhow::anyhow!(
        "unified_json_language requires both `pure-rust` and `with-grammars` features"
    ))
}
