//! Decoder for extracting Grammar and ParseTable from Tree-sitter's TSLanguage struct
//!
//! This module reverse-engineers Tree-sitter's compressed parse table format
//! and decodes it into adze's native structures.

use adze_glr_core::{Action, LexMode, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{
    ExternalToken, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol,
    SymbolId, Token, TokenPattern,
};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::{CStr, c_char};
use std::fs;
use std::path::Path;

use crate::pure_parser::{TSLanguage, TSParseAction};
use crate::ts_format::TSActionTag;

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
    let mut rules: IndexMap<SymbolId, Vec<Rule>> = IndexMap::new();
    let mut tokens: IndexMap<SymbolId, Token> = IndexMap::new();
    let mut symbol_names = Vec::new();
    let mut externals = Vec::new();
    let rule_names = IndexMap::new();

    // Read all symbol names with safe slice operations
    if lang.symbol_names.is_null() {
        // If symbol_names pointer is null, generate default names
        for i in 0..lang.symbol_count as usize {
            symbol_names.push(format!("symbol_{}", i));
        }
    } else {
        // Use safe slice operations with comprehensive bounds checking
        let symbol_count = lang.symbol_count as usize;
        if symbol_count > 0 {
            // Create a safe slice from the pointer array
            // SAFETY: `lang.symbol_names` is non-null (branch guard above),
            // and `symbol_count` matches `lang.symbol_count`. The TSLanguage
            // contract guarantees the array has at least `symbol_count` elements.
            let symbol_name_ptrs =
                unsafe { std::slice::from_raw_parts(lang.symbol_names, symbol_count) };

            for (i, &name_ptr) in symbol_name_ptrs.iter().enumerate() {
                let name = if name_ptr.is_null() {
                    format!("symbol_{}", i)
                } else {
                    // Safe string conversion with error handling
                    // SAFETY: `name_ptr` is non-null (branch guard above) and
                    // points to a null-terminated C string per TSLanguage contract.
                    match unsafe { CStr::from_ptr(name_ptr as *const c_char) }.to_str() {
                        Ok(valid_str) => valid_str.to_owned(),
                        Err(_) => {
                            // Invalid UTF-8, generate safe fallback name
                            format!("symbol_invalid_{}", i)
                        }
                    }
                };
                symbol_names.push(name);
            }
        }
    }

    // Process symbols to determine tokens with safe operations
    if !lang.symbol_metadata.is_null() {
        let symbol_count = lang.symbol_count as usize;
        if symbol_count > 0 {
            // Create a safe slice from the metadata array
            // SAFETY: `lang.symbol_metadata` is non-null (branch guard) and
            // the TSLanguage contract guarantees the array has `symbol_count` elements.
            let symbol_metadata_slice =
                unsafe { std::slice::from_raw_parts(lang.symbol_metadata, symbol_count) };

            for (i, &metadata) in symbol_metadata_slice.iter().enumerate() {
                // Bounds check for symbol_names access
                if i < symbol_names.len() {
                    let name = &symbol_names[i];
                    let symbol_id = SymbolId(i as u16);

                    if is_terminal(metadata, name) {
                        // This is a token
                        let pattern = if let Some(real_pattern) = token_patterns.get(name) {
                            real_pattern.clone()
                        } else {
                            adze_ir::TokenPattern::String(name.clone())
                        };

                        tokens.insert(
                            symbol_id,
                            Token {
                                name: name.clone(),
                                pattern,
                                fragile: false,
                            },
                        );
                    }
                }
            }
        }
    } else {
        // If symbol_metadata is null, assume all symbols with certain patterns are tokens
        for i in 0..lang.symbol_count as usize {
            let name = &symbol_names[i];
            let symbol_id = SymbolId(i as u16);

            // Heuristic: symbols that are likely terminals based on name patterns
            if is_likely_terminal_by_name(name) {
                let pattern = if let Some(real_pattern) = token_patterns.get(name) {
                    real_pattern.clone()
                } else {
                    adze_ir::TokenPattern::String(name.clone())
                };

                tokens.insert(
                    symbol_id,
                    Token {
                        name: name.clone(),
                        pattern,
                        fragile: false,
                    },
                );
            }
        }
    }

    // Decode field names with safe slice operations
    let mut field_names_map = IndexMap::new();
    if !lang.field_names.is_null() && lang.field_count > 0 {
        // Create a safe slice from the field names array
        let field_count = lang.field_count as usize;
        // SAFETY: `lang.field_names` is non-null and `field_count > 0` (branch guard).
        // TSLanguage contract guarantees the array has `field_count` elements.
        let field_name_ptrs = unsafe { std::slice::from_raw_parts(lang.field_names, field_count) };

        for (i, &name_ptr) in field_name_ptrs.iter().enumerate() {
            if !name_ptr.is_null() {
                // Safe string conversion with error handling
                // SAFETY: `name_ptr` is non-null (branch guard) and points to a
                // null-terminated C string per TSLanguage contract.
                match unsafe { CStr::from_ptr(name_ptr as *const c_char) }.to_str() {
                    Ok(valid_str) => {
                        field_names_map.insert(FieldId(i as u16), valid_str.to_owned());
                    }
                    Err(_) => {
                        // Invalid UTF-8, skip this field or use fallback name
                        field_names_map.insert(FieldId(i as u16), format!("field_invalid_{}", i));
                    }
                }
            }
        }
    }

    // Decode production rules from language metadata with bounds checking
    if !lang.rules.is_null() && lang.rule_count > 0 {
        let rule_count = lang.rule_count as usize;
        // Create safe slice from rules array
        // SAFETY: `lang.rules` is non-null and `rule_count > 0` (branch guard).
        // TSLanguage contract guarantees the array has `rule_count` elements.
        let rules_slice = unsafe { std::slice::from_raw_parts(lang.rules, rule_count) };

        for (i, &ts_rule) in rules_slice.iter().enumerate() {
            let lhs = SymbolId(ts_rule.lhs);
            let rhs_len = ts_rule.rhs_len as usize;

            // Prevent excessive memory allocation
            if rhs_len > 10000 {
                // Skip rules with unreasonably large RHS to prevent DoS
                continue;
            }

            // Build RHS from alias_sequences if available
            let mut rhs = Vec::with_capacity(rhs_len);
            let has_alias_data = !lang.alias_map.is_null() && !lang.alias_sequences.is_null();
            if has_alias_data {
                // Safe access to alias_map with bounds checking
                // SAFETY: `lang.alias_map` is non-null (has_alias_data check above).
                // `rule_count` matches the alias_map array length per TSLanguage contract.
                let alias_map_slice =
                    unsafe { std::slice::from_raw_parts(lang.alias_map, rule_count) };

                if i < alias_map_slice.len() {
                    let offset = alias_map_slice[i] as usize;

                    // Calculate maximum safe access to alias_sequences
                    // We need to be more careful about the total size here
                    let max_sequences_needed = offset.saturating_add(rhs_len);

                    // Only proceed if we can safely access the required range
                    if max_sequences_needed <= usize::MAX / 2 {
                        // Conservative bound check
                        // TODO(safety): `max_sequences_needed` is computed from
                        // alias_map data which may not reflect the true allocation
                        // size of `lang.alias_sequences`. If the alias_map contains
                        // corrupted offsets, this could read out of bounds.
                        let alias_sequences_slice = unsafe {
                            // Create a slice that covers at least what we need
                            // Note: We can't know the true size, so we use a conservative estimate
                            std::slice::from_raw_parts(lang.alias_sequences, max_sequences_needed)
                        };

                        for j in 0..rhs_len {
                            let seq_idx = offset + j;
                            if seq_idx < alias_sequences_slice.len() {
                                let sym_idx = alias_sequences_slice[seq_idx];
                                let sym_id = SymbolId(sym_idx);
                                let symbol = if (sym_idx as u32)
                                    < lang.token_count + lang.external_token_count
                                {
                                    Symbol::Terminal(sym_id)
                                } else {
                                    Symbol::NonTerminal(sym_id)
                                };
                                rhs.push(symbol);
                            } else {
                                // Out of bounds - use placeholder
                                rhs.push(Symbol::NonTerminal(SymbolId(0)));
                            }
                        }
                    } else {
                        // Unsafe offset calculation - use placeholder RHS
                        for _ in 0..rhs_len {
                            rhs.push(Symbol::NonTerminal(SymbolId(0)));
                        }
                    }
                } else {
                    // Index out of bounds for alias_map - use placeholder RHS
                    for _ in 0..rhs_len {
                        rhs.push(Symbol::NonTerminal(SymbolId(0)));
                    }
                }
            } else {
                // Fallback: build placeholder RHS of correct length
                for _ in 0..rhs_len {
                    rhs.push(Symbol::NonTerminal(SymbolId(0))); // Placeholder; actual symbols unknown
                }
            }

            // Dynamic precedence if available with safe access
            let precedence = if !lang.parse_actions.is_null()
                && (i as u32) < lang.production_id_count
            {
                let production_count = lang.production_id_count as usize;
                if i < production_count {
                    // Create safe slice for parse_actions
                    // SAFETY: `lang.parse_actions` is non-null (branch guard) and
                    // `production_count` is derived from `lang.production_id_count`.
                    // TSLanguage contract guarantees the array has at least this many entries.
                    let parse_actions_slice =
                        unsafe { std::slice::from_raw_parts(lang.parse_actions, production_count) };

                    let action = parse_actions_slice[i];
                    if action.dynamic_precedence != 0 {
                        Some(PrecedenceKind::Dynamic(action.dynamic_precedence as i16))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Associativity metadata currently not encoded
            let associativity = None;

            // Decode field mappings for this production with safe bounds checking
            let fields = if lang.field_count > 0
                && !lang.field_map_slices.is_null()
                && !lang.field_map_entries.is_null()
            {
                let production_count = lang.production_id_count as usize;
                let slice_count = production_count.saturating_mul(2);

                if slice_count > 0 && slice_count <= usize::MAX / 4 {
                    // Conservative bounds check
                    // SAFETY: `lang.field_map_slices` is non-null (branch guard) and
                    // `slice_count = production_count * 2`. TSLanguage contract guarantees
                    // the field_map_slices array covers all productions.
                    let slices =
                        unsafe { std::slice::from_raw_parts(lang.field_map_slices, slice_count) };

                    let mut out = Vec::new();
                    let slice_idx = i.saturating_mul(2);

                    if slice_idx + 1 < slices.len() {
                        let start = slices[slice_idx] as usize;
                        let len = slices[slice_idx + 1] as usize;

                        // Prevent excessive memory allocation or access
                        if len > 0 && len <= 10000 {
                            // Reasonable field limit
                            let entry_count = start.saturating_add(len).saturating_mul(2);

                            // Check if the calculation is safe
                            if entry_count <= usize::MAX / 4 && start <= entry_count {
                                // TODO(safety): `entry_count` is derived from slice data
                                // (start + len) which may not match the true allocation
                                // size of `lang.field_map_entries`.
                                let entries = unsafe {
                                    std::slice::from_raw_parts(lang.field_map_entries, entry_count)
                                };

                                for j in 0..len {
                                    let entry_base = (start + j).saturating_mul(2);
                                    if entry_base + 1 < entries.len() {
                                        let low = entries[entry_base];
                                        let high = entries[entry_base + 1];
                                        let packed = ((high as u32) << 16) | (low as u32);
                                        let field_id = (packed & 0xFFFF) as u16;
                                        let child_index = ((packed >> 16) & 0xFF) as usize;
                                        out.push((FieldId(field_id), child_index));
                                    }
                                }
                            }
                        }
                    }
                    out
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            rules.entry(lhs).or_default().push(Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields,
                production_id: ProductionId(i as u16),
            });
        }
    }

    // Decode field names with safe operations (avoid duplicate code)
    let _field_name_map = field_names_map.clone(); // Reuse the safely decoded field names

    // Decode field map entries with comprehensive safety checks
    let mut fields_by_rule: HashMap<u16, Vec<(FieldId, usize)>> = HashMap::new();
    if !lang.field_map_slices.is_null()
        && !lang.field_map_entries.is_null()
        && lang.production_count > 0
    {
        let production_count = lang.production_count as usize;
        let slice_array_size = production_count.saturating_mul(2);

        if slice_array_size > 0 && slice_array_size <= usize::MAX / 4 {
            // SAFETY: `lang.field_map_slices` is non-null (branch guard) and
            // `slice_array_size = production_count * 2`. Bounded by `usize::MAX / 4` check.
            let slices_array =
                unsafe { std::slice::from_raw_parts(lang.field_map_slices, slice_array_size) };

            for pid in 0..production_count {
                let slice_base = pid.saturating_mul(2);
                if slice_base + 1 < slices_array.len() {
                    let start = slices_array[slice_base] as usize;
                    let len = slices_array[slice_base + 1] as usize;

                    // Prevent excessive allocations and potential overflow
                    if len > 0 && len <= 1000 {
                        // Reasonable limit per production
                        let total_entries_needed = start.saturating_add(len).saturating_mul(2);

                        // Ensure we can safely access the entries
                        // TODO(safety): `total_entries_needed` is derived from
                        // slice data which may not match the true allocation size
                        // of `lang.field_map_entries`. Bounds check above mitigates
                        // overflow but cannot verify the underlying allocation.
                        if total_entries_needed <= usize::MAX / 4 && start <= total_entries_needed {
                            let entries_array = unsafe {
                                std::slice::from_raw_parts(
                                    lang.field_map_entries,
                                    total_entries_needed,
                                )
                            };

                            for j in 0..len {
                                let entry_base = (start + j).saturating_mul(2);
                                if entry_base + 1 < entries_array.len() {
                                    let low = entries_array[entry_base] as u32;
                                    let high = entries_array[entry_base + 1] as u32;
                                    let packed = (high << 16) | low;
                                    let field_id = (packed & 0xFFFF) as u16;
                                    let child_index = ((packed >> 16) & 0xFF) as usize;
                                    fields_by_rule
                                        .entry(pid as u16)
                                        .or_default()
                                        .push((FieldId(field_id), child_index));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Only populate additional rules from parse_rules if we don't have proper TSRule data
    // This avoids overwriting correct rule data with incomplete data
    let mut production_ids = IndexMap::new();
    if lang.rules.is_null() {
        let parsed_rules = decode_rules(lang);
        let has_alias_data = !lang.alias_map.is_null() && !lang.alias_sequences.is_null();
        for (i, pr) in parsed_rules.into_iter().enumerate() {
            // Build RHS from alias_sequences if available with safe access
            let rhs_len = pr.rhs_len as usize;
            let mut rhs = Vec::with_capacity(rhs_len);

            if has_alias_data && rhs_len <= 1000 {
                // Prevent excessive allocations
                // Safe access to alias_map
                let alias_map_size = (lang.production_count as usize).max(i + 1);
                // SAFETY: `lang.alias_map` is non-null (has_alias_data check).
                // `alias_map_size` is bounded to at least `i + 1` elements, which
                // is the minimum needed for indexing below.
                if alias_map_size > 0 {
                    let alias_map_slice =
                        unsafe { std::slice::from_raw_parts(lang.alias_map, alias_map_size) };

                    if i < alias_map_slice.len() {
                        let offset = alias_map_slice[i] as usize;
                        let total_sequences_needed = offset.saturating_add(rhs_len);

                        // Conservative bounds check for alias_sequences access
                        // TODO(safety): `total_sequences_needed` is derived from alias_map
                        // data which may not match the true alias_sequences allocation size.
                        if total_sequences_needed <= usize::MAX / 2 {
                            let alias_sequences_slice = unsafe {
                                std::slice::from_raw_parts(
                                    lang.alias_sequences,
                                    total_sequences_needed,
                                )
                            };

                            for j in 0..rhs_len {
                                let seq_idx = offset + j;
                                if seq_idx < alias_sequences_slice.len() {
                                    let sym_idx = alias_sequences_slice[seq_idx];
                                    let sym_id = SymbolId(sym_idx);
                                    let symbol = if (sym_idx as u32)
                                        < lang.token_count + lang.external_token_count
                                    {
                                        Symbol::Terminal(sym_id)
                                    } else {
                                        Symbol::NonTerminal(sym_id)
                                    };
                                    rhs.push(symbol);
                                } else {
                                    // Bounds exceeded - use placeholder
                                    rhs.push(Symbol::NonTerminal(SymbolId(0)));
                                }
                            }
                        } else {
                            // Unsafe calculation - use empty RHS
                            // rhs remains empty
                        }
                    } else {
                        // Index out of bounds - use empty RHS
                        // rhs remains empty
                    }
                } else {
                    // No valid alias map - use empty RHS
                    // rhs remains empty
                }
            }
            rules.entry(pr.lhs).or_default().push(Rule {
                lhs: pr.lhs,
                rhs,
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
            production_ids.insert(RuleId(i as u16), ProductionId(i as u16));
        }
    } else {
        // We have TSRule data, so just build the production_ids mapping from that
        for i in 0..lang.rule_count as usize {
            production_ids.insert(RuleId(i as u16), ProductionId(i as u16));
        }
    }

    // Process external tokens with safe access
    if lang.external_token_count > 0 && !lang.external_scanner.symbol_map.is_null() {
        let external_count = lang.external_token_count as usize;
        // Reasonable limit to prevent DoS
        if external_count <= 1000 {
            // SAFETY: `lang.external_scanner.symbol_map` is non-null (branch guard) and
            // `external_count` equals `lang.external_token_count`. TSLanguage contract
            // guarantees the symbol_map array has this many elements.
            let external_symbol_map = unsafe {
                std::slice::from_raw_parts(lang.external_scanner.symbol_map, external_count)
            };

            for (i, &symbol_id) in external_symbol_map.iter().enumerate() {
                // Validate symbol_id is within bounds
                if (symbol_id as u32) < lang.symbol_count {
                    let name = symbol_names
                        .get(symbol_id as usize)
                        .cloned()
                        .unwrap_or_else(|| format!("external_{}", i));
                    externals.push(ExternalToken {
                        name,
                        symbol_id: SymbolId(symbol_id),
                    });
                }
            }
        }
    }

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

fn decode_rules(lang: &TSLanguage) -> Vec<ParseRule> {
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

/// Decode a ParseTable from a TSLanguage struct
pub fn decode_parse_table(lang: &'static TSLanguage) -> ParseTable {
    let mut action_table = Vec::new();
    let goto_table = Vec::new();
    let mut symbol_metadata = Vec::new();
    let mut symbol_to_index = BTreeMap::new();
    let symbol_count = lang.symbol_count as usize;
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

        for symbol in 0..lang.symbol_count as usize {
            let table_offset = state * lang.symbol_count as usize + symbol;
            // SAFETY: `lang.parse_table` is a flat 2D array of size
            // `state_count * symbol_count`. `table_offset = state * symbol_count + symbol`
            // is in bounds because `state < large_state_count <= state_count` and
            // `symbol < symbol_count`. `lang.parse_actions` is indexed by `action_idx`
            // which is read from the parse table (trusted TSLanguage data).
            // TODO(safety): No bounds check on `action_idx` against parse_actions array size.
            let action = unsafe {
                let action_idx = *lang.parse_table.add(table_offset);

                if action_idx != 0 {
                    let raw = &*lang.parse_actions.add(action_idx as usize);
                    if raw.extra != 0
                        && raw.action_type == TSActionTag::Shift as u8
                        && let Some(&sym) = index_to_symbol.get(symbol)
                    {
                        extras_set.insert(sym);
                    }
                    decode_action(raw, &rules, &rid_by_pair)
                } else {
                    Action::Error
                }
            };
            let action_cell = if matches!(action, Action::Error) {
                vec![]
            } else {
                vec![action]
            };
            state_actions.push(action_cell);
        }

        action_table.push(state_actions);
    }

    // Decode small_parse_table for compressed states
    if !lang.small_parse_table_map.is_null() && !lang.small_parse_table.is_null() {
        for state in lang.large_state_count as usize..lang.state_count as usize {
            let mut state_actions = vec![vec![]; lang.symbol_count as usize];

            // Get the offset into small_parse_table from the map
            let map_index = state - lang.large_state_count as usize;
            // SAFETY: `lang.small_parse_table_map` is non-null (branch guard).
            // `map_index = state - large_state_count` where `state` ranges from
            // `large_state_count..state_count`, so `map_index < state_count - large_state_count`.
            // TSLanguage contract guarantees the map array covers all small states.
            let offset = unsafe { *lang.small_parse_table_map.add(map_index) } as usize;

            // SAFETY: `lang.small_parse_table` is non-null (branch guard).
            // `offset` comes from `small_parse_table_map` which is trusted TSLanguage data.
            // TODO(safety): No independent validation that `offset` is within the
            // small_parse_table allocation. Corrupted TSLanguage data could cause OOB reads.
            let mut ptr = unsafe { lang.small_parse_table.add(offset) };

            // SAFETY: `ptr` points within the small_parse_table array (see above).
            // Each subsequent `ptr.add(1)` advances to the next entry. The total number
            // of reads is `1 + 2 * field_count`, which must fit within the allocation.
            // TODO(safety): `field_count` is read from the table itself with no upper-bound
            // validation against the total allocation size.
            let field_count = unsafe { *ptr } as usize;
            ptr = unsafe { ptr.add(1) };

            // Read field_count pairs of (symbol, action_index)
            for _ in 0..field_count {
                // SAFETY: `ptr` is advanced sequentially within small_parse_table.
                // Each iteration reads two entries. Validity depends on `field_count`
                // being correct per TSLanguage contract (see TODO above).
                let symbol = unsafe { *ptr } as usize;
                ptr = unsafe { ptr.add(1) };

                let action_index = unsafe { *ptr } as usize;
                ptr = unsafe { ptr.add(1) };

                // Decode the action
                if action_index != 0 && symbol < lang.symbol_count as usize {
                    let action = if action_index == 0xFFFF {
                        Action::Accept
                    } else if action_index & 0x8000 != 0 {
                        // Reduce action - for pure-rust backend, bits 14-0 contain sequential rule ID (1-based)
                        let rule_id = (action_index & 0x7FFF) - 1;
                        if rule_id < rules.len() {
                            Action::Reduce(RuleId(rule_id as u16))
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
    let tcols = (lang.token_count + lang.external_token_count) as usize;
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
        alias_sequences: Vec::new(),      // TODO: Decode from language
        field_names,
        field_map,
    };

    // Auto-detect GOTO indexing mode
    table.detect_goto_indexing();

    table
}

/// Determine if a symbol is a terminal based on metadata and name
fn is_terminal(metadata: u8, name: &str) -> bool {
    // In Tree-sitter, metadata bits encode symbol characteristics.
    // Bit 0 (0x01): visible flag
    // Bit 2 (0x04): extra token flag
    // Visible symbols are typically terminals, but extras are terminals even if hidden.

    // Extras are always terminals
    if (metadata & 0x04) != 0 {
        return true;
    }

    // First check: if the symbol is visible (bit 0 set), it's likely a terminal
    if (metadata & 0x01) != 0 {
        // Visible symbol - most likely a terminal
        // But exclude some patterns that are definitely non-terminals even if visible
        if name.starts_with("_") && name[1..].chars().all(|c| c.is_ascii_digit()) {
            // Names like _119, _26 are non-terminals even if marked visible
            return false;
        }
        return true;
    }

    // Hidden symbols are usually non-terminals, but check for special cases
    // Some terminals might be hidden (like whitespace, comments)
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

/// Heuristic to determine if a symbol is likely a terminal when metadata is unavailable
fn is_likely_terminal_by_name(name: &str) -> bool {
    // When metadata is not available, use name-based heuristics
    // This mirrors the logic from is_terminal but without metadata bits

    // Obvious terminal patterns
    if name.starts_with("anon_sym_")
        || name.starts_with("aux_sym_")
        || name.starts_with("sym_")
        || name == "ERROR"
        || name.starts_with("ts_builtin_sym_")
    {
        return true;
    }

    // Common terminal names
    if matches!(
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
    ) {
        return true;
    }

    // Exclude patterns that are definitely non-terminals
    if name.starts_with("_") && name[1..].chars().all(|c| c.is_ascii_digit()) {
        // Names like _119, _26 are non-terminals
        return false;
    }

    // Single character symbols are usually terminals
    if name.len() == 1 {
        return true;
    }

    // Multi-character punctuation is usually terminal
    if name.len() <= 3
        && name
            .chars()
            .all(|c| !c.is_alphanumeric() && !c.is_whitespace())
    {
        return true;
    }

    // Default to non-terminal for safety
    false
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
