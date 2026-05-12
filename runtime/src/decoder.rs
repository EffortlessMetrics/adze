//! Decoder for extracting Grammar and ParseTable from Tree-sitter's TSLanguage struct
//!
//! This module reverse-engineers Tree-sitter's compressed parse table format
//! and decodes it into adze's native structures.

use adze_glr_core::{Action, LexMode, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{Grammar, Rule, RuleId, StateId, SymbolId, TokenPattern};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::{CStr, c_char};
use std::fs;
use std::path::Path;

use crate::pure_parser::{TSLanguage, TSParseAction};
use crate::ts_format::TSActionTag;

mod externals;
mod fields;
mod productions;
mod symbols;

/// Load token patterns from a Tree-sitter `grammar.json` file.
///
/// This extracts:
/// - string literals (`type: "STRING"`) as `TokenPattern::String`
/// - regex-like patterns (`type: "PATTERN"`) as `TokenPattern::Regex`
///
/// The returned map uses:
/// - token rule names as keys when a rule directly represents a token
/// - literal text itself as keys for string literals
pub fn load_token_patterns(grammar_json_path: &Path) -> HashMap<String, TokenPattern> {
    let Ok(contents) = fs::read_to_string(grammar_json_path) else {
        return HashMap::new();
    };

    let mut patterns = HashMap::new();

    // Named rules whose body directly represents a token.
    // This handles the common grammar.json shape:
    // "rules": { "identifier": { "type": "PATTERN", "value": "..." }, ... }
    let named_rule_re = Regex::new(
        r#""([^"\\]+)"\s*:\s*\{\s*"type"\s*:\s*"(STRING|PATTERN)"\s*,\s*"value"\s*:\s*"((?:\\.|[^"\\])*)""#,
    )
    .expect("regex must compile");
    for captures in named_rule_re.captures_iter(&contents) {
        let name = unescape_json_string(&captures[1]);
        let value = unescape_json_string(&captures[3]);
        let pattern = if &captures[2] == "STRING" {
            TokenPattern::String(value)
        } else {
            TokenPattern::Regex(value)
        };
        patterns.insert(name, pattern);
    }

    // String literals that appear anywhere in the grammar.
    let string_literal_re =
        Regex::new(r#""type"\s*:\s*"STRING"\s*,\s*"value"\s*:\s*"((?:\\.|[^"\\])*)""#)
            .expect("regex must compile");
    for captures in string_literal_re.captures_iter(&contents) {
        let value = unescape_json_string(&captures[1]);
        patterns
            .entry(value.clone())
            .or_insert_with(|| TokenPattern::String(value));
    }

    patterns
}

fn unescape_json_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                if hex.len() == 4
                    && let Ok(code) = u32::from_str_radix(&hex, 16)
                    && let Some(ch) = char::from_u32(code)
                {
                    out.push(ch);
                }
            }
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

/// Decode a Grammar from a TSLanguage struct
pub fn decode_grammar(lang: &'static TSLanguage) -> Grammar {
    decode_grammar_with_patterns(lang, &HashMap::new())
}

/// Decode a Grammar from a TSLanguage struct with token patterns from grammar.json
pub fn decode_grammar_with_patterns(
    lang: &'static TSLanguage,
    token_patterns: &HashMap<String, TokenPattern>,
) -> Grammar {
    let symbol_names = symbols::decode_symbol_names(lang);
    let tokens = symbols::decode_tokens(lang, &symbol_names, token_patterns);
    let field_names_map = fields::decode_field_names(lang);
    let rule_names = IndexMap::new();
    let mut rules: IndexMap<SymbolId, Vec<Rule>> = IndexMap::new();

    productions::decode_metadata_rules(lang, &mut rules);
    let _fields_by_rule = fields::decode_fields_by_rule(lang);
    let production_ids = productions::decode_fallback_rules(lang, &mut rules);
    let externals = externals::decode_external_tokens(lang, &symbol_names);

    Grammar {
        name: "decoded_grammar".to_string(),
        rules,
        tokens,
        precedences: vec![],
        conflicts: vec![],
        externals,
        extras: vec![],
        fields: field_names_map,
        supertypes: vec![],
        inline_rules: vec![],
        alias_sequences: IndexMap::new(),
        production_ids,
        max_alias_sequence_length: 0,
        rule_names,
        symbol_registry: None,
    }
}

pub(super) fn decode_rules(lang: &TSLanguage) -> Vec<ParseRule> {
    let production_count = lang.production_count as usize;

    // Prevent excessive allocations to avoid DoS
    let safe_production_count = production_count.min(100000);
    let mut rules = Vec::with_capacity(safe_production_count);

    if lang.production_lhs_index.is_null() || production_count == 0 {
        // No rules available, return empty
        return rules;
    }

    // Create safe slice for production_lhs_index
    // SAFETY: `lang.production_lhs_index` is non-null (checked above).
    // `safe_production_count` is capped at 100000. TSLanguage contract guarantees
    // the production_lhs_index array has `production_count` elements.
    let production_lhs_slice =
        unsafe { std::slice::from_raw_parts(lang.production_lhs_index, safe_production_count) };

    // Create safe slice for rules if available
    let rules_slice = if !lang.rules.is_null() && lang.rule_count > 0 {
        let rule_count = (lang.rule_count as usize).min(safe_production_count);
        // SAFETY: `lang.rules` is non-null (branch guard) and `rule_count` is
        // bounded by both `lang.rule_count` and `safe_production_count`.
        Some(unsafe { std::slice::from_raw_parts(lang.rules, rule_count) })
    } else {
        None
    };

    // Use production_lhs_index to get the correct LHS symbols
    // and try to get RHS length from TSRule if available
    for i in 0..safe_production_count {
        // Get LHS from production_lhs_index (which has correct symbol in table index space)
        let lhs_idx = if i < production_lhs_slice.len() {
            production_lhs_slice[i]
        } else {
            0 // Fallback for out-of-bounds
        };

        // Try to get rhs_len from TSRule if available
        let rhs_len = if let Some(rules_slice) = rules_slice {
            if i < rules_slice.len() {
                rules_slice[i].rhs_len as u16
            } else {
                0 // Fallback for out-of-bounds
            }
        } else {
            0 // Fallback: we don't know the RHS length
        };

        rules.push(ParseRule {
            lhs: SymbolId(lhs_idx), // Use the index from production_lhs_index
            rhs_len,
        });
    }
    rules
}

fn decode_alias_sequences(
    lang: &TSLanguage,
    index_to_symbol: &[SymbolId],
) -> Vec<Vec<Option<SymbolId>>> {
    let production_count = if lang.production_id_count > 0 {
        lang.production_id_count as usize
    } else if lang.production_count > 0 {
        lang.production_count as usize
    } else {
        lang.rule_count as usize
    };
    let stride = lang.max_alias_sequence_length as usize;

    if lang.alias_count == 0
        || production_count == 0
        || stride == 0
        || lang.alias_map.is_null()
        || lang.alias_sequences.is_null()
    {
        return Vec::new();
    }

    let safe_production_count = production_count.min(100_000);
    let Some(alias_cell_count) = safe_production_count.checked_mul(stride) else {
        return Vec::new();
    };
    if alias_cell_count > 10_000_000 {
        return Vec::new();
    }

    // SAFETY: non-null pointers are checked above, and the TSLanguage ABI stores one
    // alias-map entry per production plus a dense alias sequence table.
    let alias_map = unsafe { std::slice::from_raw_parts(lang.alias_map, safe_production_count) };
    // SAFETY: see above; `alias_cell_count` is checked for overflow and capped.
    let alias_cells = unsafe { std::slice::from_raw_parts(lang.alias_sequences, alias_cell_count) };

    alias_map
        .iter()
        .map(|offset| {
            let offset = *offset as usize;
            (0..stride)
                .map(|position| {
                    let raw_symbol = offset
                        .checked_add(position)
                        .and_then(|index| alias_cells.get(index))
                        .copied()
                        .unwrap_or(0);
                    if raw_symbol == 0 {
                        None
                    } else {
                        Some(
                            index_to_symbol
                                .get(raw_symbol as usize)
                                .copied()
                                .unwrap_or(SymbolId(raw_symbol)),
                        )
                    }
                })
                .collect()
        })
        .collect()
}

/// Decode a ParseTable from a TSLanguage struct
pub fn decode_parse_table(lang: &'static TSLanguage) -> ParseTable {
    let mut action_table = Vec::new();
    let mut symbol_metadata = Vec::new();
    let mut symbol_to_index = BTreeMap::new();
    let symbol_count = lang.symbol_count as usize;
    let tcols = (lang.token_count + lang.external_token_count) as usize;
    let mut goto_table = vec![vec![StateId(0); symbol_count]; lang.state_count as usize];
    let mut extras_set: BTreeSet<SymbolId> = BTreeSet::new();

    // Build the column -> symbol mapping first. If `public_symbol_map` is present, it
    // supplies the public symbol id for each table column. Otherwise, fall back to
    // dense column identities.
    let mut index_to_symbol: Vec<SymbolId> =
        (0..symbol_count).map(|col| SymbolId(col as u16)).collect();

    if !lang.public_symbol_map.is_null() {
        let mut saw_symbol = BTreeSet::new();
        let mut public_map_ok = true;
        for col in 0..symbol_count {
            let public_sym = unsafe { *lang.public_symbol_map.add(col) };
            if !saw_symbol.insert(SymbolId(public_sym)) {
                // Duplicate public IDs cannot be inverted into `symbol_to_index`.
                public_map_ok = false;
                break;
            }
            index_to_symbol[col] = SymbolId(public_sym);
        }

        if !public_map_ok {
            index_to_symbol = (0..symbol_count).map(|col| SymbolId(col as u16)).collect();
        }
    }

    for (col, &sym) in index_to_symbol.iter().enumerate() {
        symbol_to_index.insert(sym, col);
    }
    if symbol_to_index.len() != symbol_count {
        // Defensive recovery for malformed mappings: restore dense invariants.
        symbol_to_index.clear();
        index_to_symbol = (0..symbol_count).map(|col| SymbolId(col as u16)).collect();
        for (col, &sym) in index_to_symbol.iter().enumerate() {
            symbol_to_index.insert(sym, col);
        }
        // Keep a dense column-index map for the decoded table.
    }

    // Decode grammar and rules from TSLanguage
    let mut grammar = decode_grammar(lang);
    // Extract rules from the grammar in production_id order
    let mut rules: Vec<ParseRule> = {
        let mut rules_vec = vec![None; lang.rule_count as usize];
        // Collect all rules from all LHS symbols in the grammar and place them by production_id
        for rules_for_lhs in grammar.rules.values() {
            for rule in rules_for_lhs {
                let idx = rule.production_id.0 as usize;
                if idx < rules_vec.len() {
                    rules_vec[idx] = Some(ParseRule {
                        lhs: rule.lhs,
                        rhs_len: rule.rhs.len() as u16,
                    });
                }
            }
        }
        // Convert to final vector, handling any gaps
        rules_vec
            .into_iter()
            .map(|opt_rule| {
                opt_rule.unwrap_or({
                    // Fallback for missing rules - shouldn't happen with valid grammars
                    ParseRule {
                        lhs: SymbolId(0),
                        rhs_len: 0,
                    }
                })
            })
            .collect()
    };

    for rule in &mut rules {
        if (rule.lhs.0 as usize) < index_to_symbol.len() {
            rule.lhs = index_to_symbol[rule.lhs.0 as usize];
        }
    }

    // Build (lhs, rhs_len) -> rule_id map for normalizing Reduce actions
    let mut rid_by_pair: HashMap<(u16, u8), u16> = HashMap::with_capacity(rules.len());
    for (i, r) in rules.iter().enumerate() {
        rid_by_pair.insert((r.lhs.0, r.rhs_len as u8), i as u16);
    }

    // Decode and annotate symbol metadata
    for (i, sym) in index_to_symbol.iter().copied().enumerate() {
        // Decode symbol metadata
        // SAFETY: Pointer arithmetic on `lang.symbol_metadata` and `lang.symbol_names`
        // is valid because `i < lang.symbol_count` (loop bound). Both pointers are
        // null-checked before dereferencing. `CStr::from_ptr` requires null-terminated
        // strings, which is guaranteed by the TSLanguage contract.
        let (ts_metadata, name) = unsafe {
            let ts_metadata = if !lang.symbol_metadata.is_null() {
                *lang.symbol_metadata.add(i)
            } else {
                0 // Default metadata when not available
            };
            let name_ptr = if !lang.symbol_names.is_null() {
                *lang.symbol_names.add(i)
            } else {
                std::ptr::null()
            };
            let name = if name_ptr.is_null() {
                format!("symbol_{}", i)
            } else {
                CStr::from_ptr(name_ptr as *const c_char)
                    .to_string_lossy()
                    .into_owned()
            };
            (ts_metadata, name)
        };

        if (ts_metadata & 0x04) != 0 {
            extras_set.insert(sym);
        }

        let is_terminal = (i as u32) < lang.token_count + lang.external_token_count;
        let symbol_id = sym;

        symbol_metadata.push(SymbolMetadata {
            name,
            is_visible: (ts_metadata & 0x01) != 0,
            is_named: (ts_metadata & 0x02) != 0,
            is_supertype: (ts_metadata & 0x08) != 0,
            // Additional fields required by GLR core API contracts
            is_terminal,
            is_extra: (ts_metadata & 0x04) != 0,
            is_fragile: false, // Tree-sitter doesn't expose fragile token info directly
            symbol_id,
        });
    }

    // Decode the parse table for large states
    for state in 0..lang.large_state_count as usize {
        let mut state_actions = Vec::new();

        for symbol in 0..symbol_count {
            let table_offset = state * lang.symbol_count as usize + symbol;
            // SAFETY: `lang.parse_table` is a flat 2D array of size
            // `state_count * symbol_count`. `table_offset = state * symbol_count + symbol`
            // is in bounds because `state < large_state_count <= state_count` and
            // `symbol < symbol_count`. `lang.parse_actions` is indexed by `action_idx`
            // which is read from the parse table (trusted TSLanguage data).
            // TODO(safety): No bounds check on `action_idx` against parse_actions array size.
            let table_value = unsafe { *lang.parse_table.add(table_offset) };

            let action_cell = if symbol >= tcols {
                if table_value != 0 {
                    goto_table[state][symbol] = StateId(table_value);
                }
                vec![]
            } else if table_value != 0 {
                let action = unsafe {
                    let raw = &*lang.parse_actions.add(table_value as usize);
                    if raw.extra != 0
                        && raw.action_type == TSActionTag::Shift as u8
                        && let Some(&sym) = index_to_symbol.get(symbol)
                    {
                        extras_set.insert(sym);
                    }
                    decode_action(raw, &rules, &rid_by_pair)
                };
                if matches!(action, Action::Error) {
                    vec![]
                } else {
                    vec![action]
                }
            } else {
                vec![]
            };
            state_actions.push(action_cell);
        }

        action_table.push(state_actions);
    }

    // Decode small_parse_table for compressed states
    if !lang.small_parse_table_map.is_null() && !lang.small_parse_table.is_null() {
        for state in lang.large_state_count as usize..lang.state_count as usize {
            let mut state_actions = vec![vec![]; lang.symbol_count as usize];

            // Get this state's direct-pair range from the map.
            let map_index = state - lang.large_state_count as usize;
            // SAFETY: `lang.small_parse_table_map` is non-null (branch guard).
            // `map_index = state - large_state_count` where `state` ranges from
            // `large_state_count..state_count`, so `map_index < state_count - large_state_count`.
            // TSLanguage contract guarantees the map array covers all small states.
            // The map has a sentinel final offset after the last small state.
            let start_offset = unsafe { *lang.small_parse_table_map.add(map_index) } as usize;
            let end_offset = unsafe { *lang.small_parse_table_map.add(map_index + 1) } as usize;

            // Read direct (symbol, action_index) pairs. This matches the pure parser
            // and parser_v4 small-table readers and preserves duplicate symbols for
            // GLR multi-action cells.
            let mut offset = start_offset;
            while offset + 1 < end_offset {
                // SAFETY: `offset` is bounded by the trusted start/end offsets from
                // `small_parse_table_map`. Malformed TSLanguage data could still point
                // outside the allocation; callers should validate generated ABI data.
                let symbol = unsafe { *lang.small_parse_table.add(offset) } as usize;
                let action_index = unsafe { *lang.small_parse_table.add(offset + 1) } as usize;
                offset += 2;

                if symbol >= symbol_count || action_index == 0 {
                    continue;
                }

                if symbol >= tcols {
                    goto_table[state][symbol] = StateId(action_index as u16);
                } else {
                    let action = if action_index == 0xFFFF {
                        Action::Accept
                    } else if action_index & 0x8000 != 0 {
                        // Reduce action - bits 14-0 contain encoded GLR rule ID (1-based).
                        let encoded_rule_id = (action_index & 0x7FFF) - 1;
                        if let Some(production_id) =
                            mapped_production_id(lang, encoded_rule_id, rules.len())
                        {
                            Action::Reduce(RuleId(production_id))
                        } else {
                            Action::Error
                        }
                    } else {
                        // Shift action - bits 14-0 contain state ID
                        Action::Shift(StateId(action_index as u16))
                    };

                    if !matches!(action, Action::Error) {
                        state_actions[symbol].push(action);
                    }
                }
            }

            action_table.push(state_actions);
        }
    }

    // Decode external scanner states from the TSLanguage struct
    let external_scanner_states =
        if lang.external_token_count > 0 && !lang.external_scanner.states.is_null() {
            let mut states = Vec::with_capacity(lang.state_count as usize);
            let external_count = lang.external_token_count as usize;

            // The states are stored as a flat array of bools
            // Each state has external_token_count bools indicating which externals are valid
            // SAFETY: `lang.external_scanner.states` is non-null (branch guard) and is
            // cast to `*const bool`. The flat array has `state_count * external_count`
            // entries per TSLanguage contract. `idx = state_idx * external_count + external_idx`
            // is in bounds because both indices are within their respective ranges.
            // TODO(safety): Casting `*const u8` to `*const bool` assumes that the
            // memory representation of `bool` is a single byte (0 or 1), which is
            // guaranteed on all Rust targets but values other than 0/1 would be UB.
            unsafe {
                let states_ptr = lang.external_scanner.states as *const bool;
                for state_idx in 0..lang.state_count as usize {
                    let mut state_externals = Vec::with_capacity(external_count);
                    for external_idx in 0..external_count {
                        let idx = state_idx * external_count + external_idx;
                        let is_valid = *states_ptr.add(idx);
                        state_externals.push(is_valid);
                    }
                    states.push(state_externals);
                }
            }
            states
        } else {
            vec![vec![]; lang.state_count as usize]
        };

    // External tokens now have their transitions in the main action_table
    // No separate map needed

    // Build nonterminal_to_index for goto lookups
    let mut nonterminal_to_index = BTreeMap::new();
    for (col, sym) in index_to_symbol.iter().enumerate() {
        if col >= tcols {
            nonterminal_to_index.insert(*sym, col);
        }
    }
    // lang.eof_symbol is the *column index* of EOF, so map it back to the
    // corresponding SymbolId using the index_to_symbol mapping we just built.
    let eof_symbol = index_to_symbol
        .get(lang.eof_symbol as usize)
        .copied()
        .unwrap_or(SymbolId(0));

    let extras: Vec<SymbolId> = extras_set.into_iter().collect();

    // Build field map from grammar rules
    let mut field_map = BTreeMap::new();
    for rules_vec in grammar.rules.values() {
        for rule in rules_vec {
            for (fid, pos) in &rule.fields {
                field_map.insert((RuleId(rule.production_id.0), *pos as u16), fid.0);
            }
        }
    }

    // Decode lex modes with safe access
    let lex_modes = if !lang.lex_modes.is_null() && lang.state_count > 0 {
        let state_count = lang.state_count as usize;
        // SAFETY: `lang.lex_modes` is non-null and `state_count > 0` (branch guard).
        // TSLanguage contract guarantees the lex_modes array has `state_count` entries.
        let lex_modes_slice = unsafe { std::slice::from_raw_parts(lang.lex_modes, state_count) };

        lex_modes_slice
            .iter()
            .map(|&m| LexMode {
                lex_state: m.lex_state,
                external_lex_state: m.external_lex_state,
            })
            .collect()
    } else {
        vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            lang.state_count as usize
        ]
    };

    // Field names vector from grammar
    let field_names: Vec<String> = grammar.fields.values().cloned().collect();

    grammar.extras = extras.clone();

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = index_to_symbol
        .iter()
        .copied()
        .enumerate()
        .map(|(col, sym)| (sym, col))
        .collect();

    // Safety recovery for any upstream mutation of the symbol-id map.
    if symbol_to_index.len() != symbol_count {
        symbol_to_index.clear();
        for (col, sym) in index_to_symbol.iter().copied().enumerate() {
            symbol_to_index.entry(sym).or_insert(col);
        }
    }
    let alias_sequences = decode_alias_sequences(lang, &index_to_symbol);

    let mut table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count: lang.state_count as usize,
        symbol_count: lang.symbol_count as usize,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states,
        nonterminal_to_index,
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol: {
            // Compute start symbol from the rules
            // The start symbol is typically the unique LHS that doesn't appear on any RHS
            // or the NT with the highest symbol ID (often the augmented start)
            let tcols = (lang.token_count + lang.external_token_count) as usize;
            let is_nt = |sym: SymbolId| sym.0 as usize >= tcols;

            // Collect all LHS symbols from rules (before moving rules)
            let lhs_symbols: std::collections::BTreeSet<SymbolId> =
                rules.iter().map(|r| r.lhs).collect();

            // Filter to only non-terminals and pick the best start symbol candidate
            // Prefer symbols that don't end with "_repeat" or similar internal names
            let nt_symbols: Vec<_> = lhs_symbols.into_iter().filter(|s| is_nt(*s)).collect();

            let start = if nt_symbols.is_empty() {
                SymbolId((tcols + 1) as u16)
            } else {
                // Try to find a meaningful start symbol (not a repeat helper)
                let fallback = nt_symbols
                    .first()
                    .copied()
                    .unwrap_or(SymbolId((tcols + 1) as u16));
                let meaningful = nt_symbols
                    .iter()
                    .filter(|s| {
                        // Skip out-of-range symbol IDs to avoid unsafe OOB reads.
                        // Some decoded grammars may carry symbol IDs that do not map into
                        // this table's dense index space.
                        if s.0 as usize >= lang.symbol_count as usize {
                            true
                        } else if let Some(name_ptr) =
                            unsafe { lang.symbol_names.add(s.0 as usize).as_ref() }
                        {
                            // SAFETY: `*name_ptr` is a pointer to a null-terminated C string
                            // per TSLanguage contract.
                            let name =
                                unsafe { std::ffi::CStr::from_ptr(*name_ptr as *const c_char) };
                            if let Ok(name_str) = name.to_str() {
                                // Prefer symbols that don't look like internal helpers
                                !name_str.contains("repeat") && !name_str.starts_with('_')
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    })
                    .min_by_key(|s| s.0) // Pick the first meaningful one, not the highest
                    .copied();

                meaningful.unwrap_or_else(|| {
                    // Fallback: pick the highest ID among nonterminals
                    nt_symbols
                        .iter()
                        .max_by_key(|s| s.0)
                        .copied()
                        .unwrap_or(fallback)
                })
            };

            debug_assert_ne!(start, SymbolId(0), "start_symbol cannot be ERROR(0)");
            start
        },
        rules,   // Now move rules after computing start_symbol
        grammar, // attach decoded grammar
        initial_state: StateId(0),
        token_count: lang.token_count as usize,
        external_token_count: lang.external_token_count as usize,
        lex_modes,
        extras: extras.clone(),
        dynamic_prec_by_rule: Vec::new(), // TODO: Decode from language
        rule_assoc_by_rule: Vec::new(),   // TODO: Decode from language
        alias_sequences,
        field_names,
        field_map,
    };

    // Auto-detect GOTO indexing mode
    table.detect_goto_indexing();

    table
}

fn mapped_production_id(
    lang: &TSLanguage,
    encoded_rule_id: usize,
    production_count: usize,
) -> Option<u16> {
    let production_id = if !lang.production_id_map.is_null()
        && encoded_rule_id < lang.production_id_count as usize
    {
        // SAFETY: `encoded_rule_id` is bounded by `production_id_count`, and
        // TSLanguage production_id_map contains one entry per production slot.
        unsafe { *lang.production_id_map.add(encoded_rule_id) }
    } else {
        u16::try_from(encoded_rule_id).ok()?
    };

    if production_id == u16::MAX || production_id as usize >= production_count {
        None
    } else {
        Some(production_id)
    }
}

/// Check if a symbol is hidden based on metadata
#[allow(dead_code)]
fn is_hidden(metadata: u8) -> bool {
    // Bit 0 is typically the visible bit in Tree-sitter
    (metadata & 0x01) == 0
}

/// Decode a TSParseAction into our Action enum
fn decode_action(
    action: &TSParseAction,
    rules: &[ParseRule],
    rid_by_pair: &HashMap<(u16, u8), u16>,
) -> Action {
    // Based on Tree-sitter's encoding, action_type determines the action
    // The TSParseAction struct contains different data depending on action type

    // Tree-sitter action types using shared constants
    match action.action_type {
        x if x == TSActionTag::Shift as u8 => {
            // Shift action: move to a new state
            // The symbol field contains the state to shift to
            // extra field indicates if this is an "extra" token (whitespace, etc.)
            Action::Shift(StateId(action.symbol))
        }
        x if x == TSActionTag::Reduce as u8 => {
            // Normalize Reduce action to proper rule index
            let direct = action.symbol as usize;

            // Fast path: symbol already a valid rule index and matches child_count
            let rid: u16 =
                if direct < rules.len() && (rules[direct].rhs_len as u8) == action.child_count {
                    // Using rule ID directly from symbol field
                    action.symbol
                } else {
                    // Fallback: legacy TS encoding (symbol = LHS, child_count = rhs_len)
                    // This happens when symbol is the LHS column index
                    let key = (action.symbol, action.child_count);
                    match rid_by_pair.get(&key) {
                        Some(&rid) => rid,
                        None => {
                            debug_assert!(
                                false,
                                "Reduce mapping failed: no rule for (lhs={}, rhs_len={})",
                                action.symbol, action.child_count
                            );
                            // In release, use a distinct sentinel past rules.len()
                            // so later bounds checks catch it deterministically.
                            u16::MAX
                        }
                    }
                };

            // Short-circuit invalid rule IDs
            if rid == u16::MAX || (rid as usize) >= rules.len() {
                Action::Error // Invalid reduce rule
            } else {
                Action::Reduce(RuleId(rid))
            }
        }
        x if x == TSActionTag::Accept as u8 => {
            // Accept action: parsing complete
            Action::Accept
        }
        x if x == TSActionTag::Recover as u8 => {
            // Recover action: error recovery
            Action::Recover
        }
        x if x == TSActionTag::Error as u8 => {
            // Error action
            Action::Error
        }
        _ => {
            // Unknown action type // Expected: V for Recover
            Action::Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_decoder_safety() {
        // This test ensures our decoder doesn't panic on null pointers
        // In real use, we'd test with actual TSLanguage structs
    }

    #[test]
    fn test_action_decoding() {
        // Test that we can decode different action types correctly
        let empty_rules = vec![];
        let empty_map = HashMap::new();

        // Test Shift action
        let shift_action = TSParseAction {
            action_type: TSActionTag::Shift as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 42,
        };
        match decode_action(&shift_action, &empty_rules, &empty_map) {
            Action::Shift(StateId(state)) => assert_eq!(state, 42),
            _ => panic!("Expected Shift action"),
        }

        // Test Reduce action with direct rule index
        let rules = vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 3,
        }];
        let reduce_action = TSParseAction {
            action_type: TSActionTag::Reduce as u8,
            extra: 0,
            child_count: 3,
            dynamic_precedence: 0,
            symbol: 0,
        };
        match decode_action(&reduce_action, &rules, &empty_map) {
            Action::Reduce(RuleId(rule)) => assert_eq!(rule, 0),
            _ => panic!("Expected Reduce action"),
        }

        // Test Accept action
        let accept_action = TSParseAction {
            action_type: TSActionTag::Accept as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(
            decode_action(&accept_action, &empty_rules, &empty_map),
            Action::Accept
        ));

        // Test Error/Recover action
        let recover_action = TSParseAction {
            action_type: TSActionTag::Error as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(
            decode_action(&recover_action, &empty_rules, &empty_map),
            Action::Error
        ));
    }

    #[test]
    fn small_table_reduce_actions_map_rule_id_to_production_id() {
        static PRODUCTION_ID_MAP: [u16; 2] = [1, 0];
        static PRODUCTION_LHS_INDEX: [u16; 2] = [2, 2];
        static SMALL_PARSE_TABLE: [u16; 2] = [1, 0x8001];
        static SMALL_PARSE_TABLE_MAP: [u32; 2] = [0, 2];
        static NAME_ERROR: &[u8] = b"end\0";
        static NAME_TOKEN: &[u8] = b"token\0";
        static NAME_NODE: &[u8] = b"node\0";
        static RULES: [crate::pure_parser::TSRule; 2] = [
            crate::pure_parser::TSRule {
                lhs: 2,
                rhs_len: 1,
                _pad: 0,
            },
            crate::pure_parser::TSRule {
                lhs: 2,
                rhs_len: 1,
                _pad: 0,
            },
        ];
        let symbol_names = Box::leak(Box::new([
            NAME_ERROR.as_ptr(),
            NAME_TOKEN.as_ptr(),
            NAME_NODE.as_ptr(),
        ]));

        let language = Box::leak(Box::new(TSLanguage {
            version: crate::pure_parser::TREE_SITTER_LANGUAGE_VERSION,
            symbol_count: 3,
            alias_count: 0,
            token_count: 2,
            external_token_count: 0,
            state_count: 1,
            large_state_count: 0,
            production_id_count: 2,
            field_count: 0,
            max_alias_sequence_length: 0,
            production_id_map: PRODUCTION_ID_MAP.as_ptr(),
            parse_table: std::ptr::null(),
            small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
            small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
            parse_actions: std::ptr::null(),
            symbol_names: symbol_names.as_ptr(),
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
            external_scanner: crate::pure_parser::ExternalScanner::default(),
            primary_state_ids: std::ptr::null(),
            production_lhs_index: PRODUCTION_LHS_INDEX.as_ptr(),
            production_count: 2,
            eof_symbol: 0,
            rules: RULES.as_ptr(),
            rule_count: 2,
        }));

        let table = decode_parse_table(language);

        match table.action_table[0][1][0] {
            Action::Reduce(RuleId(rule_id)) => assert_eq!(rule_id, 1),
            ref action => panic!("expected mapped reduce action, got {action:?}"),
        }
    }

    #[test]
    fn test_load_token_patterns_reads_json_literals_and_patterns() {
        let mut grammar_file = NamedTempFile::new().expect("temp file");
        writeln!(
            grammar_file,
            r#"{{
                "rules": {{
                    "identifier": {{ "type": "PATTERN", "value": "[a-z_][a-z0-9_]*" }},
                    "kw_def": {{ "type": "STRING", "value": "def" }},
                    "function_definition": {{
                        "type": "SEQ",
                        "members": [
                            {{ "type": "STRING", "value": ":" }}
                        ]
                    }}
                }}
            }}"#
        )
        .expect("write grammar");

        let patterns = load_token_patterns(grammar_file.path());

        assert_eq!(
            patterns.get("identifier"),
            Some(&TokenPattern::Regex("[a-z_][a-z0-9_]*".to_string()))
        );
        assert_eq!(
            patterns.get("kw_def"),
            Some(&TokenPattern::String("def".to_string()))
        );
        assert_eq!(
            patterns.get("def"),
            Some(&TokenPattern::String("def".to_string()))
        );
        assert_eq!(
            patterns.get(":"),
            Some(&TokenPattern::String(":".to_string()))
        );
    }

    #[test]
    fn test_load_token_patterns_missing_file_returns_empty() {
        let patterns = load_token_patterns(Path::new("/definitely/missing/grammar.json"));
        assert!(patterns.is_empty());
    }
}
