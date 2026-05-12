use adze_ir::{ExternalToken, SymbolId};

use crate::pure_parser::TSLanguage;

pub(super) fn decode_external_tokens(
    lang: &TSLanguage,
    symbol_names: &[String],
) -> Vec<ExternalToken> {
    let mut externals = Vec::new();
    if lang.external_token_count == 0 || lang.external_scanner.symbol_map.is_null() {
        return externals;
    }

    let external_count = lang.external_token_count as usize;
    if external_count > 1000 {
        return externals;
    }

    // SAFETY: `lang.external_scanner.symbol_map` is non-null (branch guard) and
    // `external_count` equals `lang.external_token_count`. TSLanguage contract
    // guarantees the symbol_map array has this many elements.
    let external_symbol_map =
        unsafe { std::slice::from_raw_parts(lang.external_scanner.symbol_map, external_count) };

    for (i, &symbol_col) in external_symbol_map.iter().enumerate() {
        if (symbol_col as u32) >= lang.symbol_count {
            continue;
        }

        let name = symbol_names
            .get(symbol_col as usize)
            .cloned()
            .unwrap_or_else(|| format!("external_{}", i));
        let public_symbol = if !lang.public_symbol_map.is_null() {
            // SAFETY: `symbol_col < symbol_count` is checked above, and
            // TSLanguage guarantees one public map entry per symbol.
            unsafe { *lang.public_symbol_map.add(symbol_col as usize) }
        } else {
            symbol_col
        };
        externals.push(ExternalToken {
            name,
            symbol_id: SymbolId(public_symbol),
        });
    }

    externals
}
