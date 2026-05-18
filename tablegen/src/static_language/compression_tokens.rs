//! TokenStream generation for compressed parse and goto tables.

use crate::{
    CompressedActionTable, CompressedGotoEntry, CompressedGotoTable, CompressedTables,
    TableCompressor,
};
use adze_glr_core::ParseTable;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn compressed_tables(
    parse_table: &ParseTable,
    compressed: &CompressedTables,
) -> (TokenStream, TokenStream) {
    // Generate compressed tables using Tree-sitter's format
    if parse_table.state_count < compressed.small_table_threshold {
        small_compressed_tables(compressed)
    } else {
        large_compressed_tables(compressed)
    }
}

pub(crate) fn small_compressed_tables(compressed: &CompressedTables) -> (TokenStream, TokenStream) {
    // Generate Tree-sitter's small table format
    // Action table: flat array of u16 values with encoded actions
    // Goto table: flat array of u16 state IDs
    let action_entries = small_action_entries(&compressed.action_table);
    let goto_entries = small_goto_entries(&compressed.goto_table);

    let action_count = compressed.action_table.data.len();
    let goto_count = count_goto_entries(&compressed.goto_table);

    let action_table = quote! {
        static SMALL_PARSE_TABLE: &[u16; #action_count] = &[#(#action_entries),*];
        static SMALL_PARSE_TABLE_MAP: &[u16] = &[/* row offsets */];
    };

    let goto_table = quote! {
        static GOTO_TABLE: &[u16; #goto_count] = &[#(#goto_entries),*];
    };

    (action_table, goto_table)
}

pub(crate) fn large_compressed_tables(compressed: &CompressedTables) -> (TokenStream, TokenStream) {
    // For large tables, use pointer arrays.
    // This is rarely needed but essential for grammars like C++.
    small_compressed_tables(compressed) // Simplified for now
}

pub(crate) fn small_action_entries(action_table: &CompressedActionTable) -> Vec<TokenStream> {
    let mut entries = Vec::new();
    let compressor = TableCompressor::new();

    for entry in &action_table.data {
        if let Ok(encoded) = compressor.encode_action_small(&entry.action) {
            let symbol = entry.symbol;
            entries.push(quote! { #symbol }); // Symbol index
            entries.push(quote! { #encoded }); // Encoded action
        }
    }

    entries
}

pub(crate) fn small_goto_entries(goto_table: &CompressedGotoTable) -> Vec<TokenStream> {
    let mut entries = Vec::new();

    for entry in &goto_table.data {
        match entry {
            CompressedGotoEntry::Single(state) => {
                entries.push(quote! { #state });
            }
            CompressedGotoEntry::RunLength { state, count } => {
                // Expand run-length encoded entries
                for _ in 0..*count {
                    entries.push(quote! { #state });
                }
            }
        }
    }

    entries
}

pub(crate) fn count_goto_entries(goto_table: &CompressedGotoTable) -> usize {
    goto_table
        .data
        .iter()
        .map(|entry| match entry {
            CompressedGotoEntry::Single(_) => 1,
            CompressedGotoEntry::RunLength { count, .. } => *count as usize,
        })
        .sum()
}
