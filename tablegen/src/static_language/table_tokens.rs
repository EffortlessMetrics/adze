//! TokenStream generation for uncompressed parse and goto tables.

use adze_glr_core::{Action, ParseTable};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn uncompressed_tables(parse_table: &ParseTable) -> (TokenStream, TokenStream) {
    // Generate uncompressed action and goto tables
    let action_entries = action_table_entries(parse_table);
    let goto_entries = goto_table_entries(parse_table);

    let action_table = quote! {
        static ACTION_TABLE: &[&[adze::ffi::TSParseActionEntry]] = &[#(#action_entries),*];
    };

    let goto_table = quote! {
        static GOTO_TABLE: &[&[u16]] = &[#(#goto_entries),*];
    };

    (action_table, goto_table)
}

pub(crate) fn action_table_entries(parse_table: &ParseTable) -> Vec<TokenStream> {
    let mut entries = Vec::new();

    for state_actions in &parse_table.action_table {
        let actions: Vec<TokenStream> = state_actions
            .iter()
            .flat_map(|action_cell| {
                // For each action cell, generate entries for all actions
                action_cell.iter().map(action_entry)
            })
            .collect();

        entries.push(quote! { &[#(#actions),*] });
    }

    entries
}

fn action_entry(action: &Action) -> TokenStream {
    match action {
        Action::Shift(state) => {
            let state_id = state.0;
            quote! {
                adze::ffi::TSParseActionEntry {
                    type_: adze::ffi::TSParseActionType::Shift,
                    state: #state_id,
                    symbol: 0,
                    child_count: 0,
                    dynamic_precedence: 0,
                    fragile: false,
                }
            }
        }
        Action::Reduce(rule) => {
            let rule_id = rule.0;
            quote! {
                adze::ffi::TSParseActionEntry {
                    type_: adze::ffi::TSParseActionType::Reduce,
                    state: 0,
                    symbol: #rule_id,
                    child_count: 0, // Will be filled with actual child count
                    dynamic_precedence: 0,
                    fragile: false,
                }
            }
        }
        Action::Accept => {
            quote! {
                adze::ffi::TSParseActionEntry {
                    type_: adze::ffi::TSParseActionType::Accept,
                    state: 0,
                    symbol: 0,
                    child_count: 0,
                    dynamic_precedence: 0,
                    fragile: false,
                }
            }
        }
        Action::Error | Action::Recover => {
            // Treat Recover as Error for FFI compatibility
            error_entry()
        }
        Action::Fork(actions) => {
            // For GLR fork points, we'll need to handle multiple actions.
            // For now, just take the first shift action.
            if let Some(Action::Shift(state)) = actions.first() {
                let state_id = state.0;
                quote! {
                    adze::ffi::TSParseActionEntry {
                        type_: adze::ffi::TSParseActionType::Shift,
                        state: #state_id,
                        symbol: 0,
                        child_count: 0,
                        dynamic_precedence: 0,
                        fragile: false,
                    }
                }
            } else {
                error_entry()
            }
        }
        _ => {
            // Unknown action type // Expected: V for Recover
            error_entry()
        }
    }
}

fn error_entry() -> TokenStream {
    quote! {
        adze::ffi::TSParseActionEntry {
            type_: adze::ffi::TSParseActionType::Error,
            state: 0,
            symbol: 0,
            child_count: 0,
            dynamic_precedence: 0,
            fragile: false,
        }
    }
}

pub(crate) fn goto_table_entries(parse_table: &ParseTable) -> Vec<TokenStream> {
    let mut entries = Vec::new();

    for state_gotos in &parse_table.goto_table {
        let gotos: Vec<u16> = state_gotos.iter().map(|state| state.0).collect();
        entries.push(quote! { &[#(#gotos),*] });
    }

    entries
}
