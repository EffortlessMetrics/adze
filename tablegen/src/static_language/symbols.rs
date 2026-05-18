//! Symbol and field metadata token generation.

use adze_ir::{Grammar, TokenPattern};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn symbol_names(grammar: &Grammar) -> Vec<String> {
    let mut names = Vec::new();

    // Add terminal symbols
    for token in grammar.tokens.values() {
        names.push(token.name.clone());
    }

    // Add non-terminal symbols (rules)
    for symbol_id in grammar.rules.keys() {
        names.push(format!("rule_{}", symbol_id.0));
    }

    // Add external symbols
    for external in &grammar.externals {
        names.push(external.name.clone());
    }

    names
}

pub(crate) fn symbol_metadata(grammar: &Grammar) -> Vec<TokenStream> {
    let mut metadata = Vec::new();

    // Generate metadata for each terminal symbol
    for token in grammar.tokens.values() {
        // Hidden tokens start with underscore
        let visible = !token.name.starts_with('_');
        // Anonymous tokens (string literals) are unnamed, regex tokens can be named
        let named = matches!(&token.pattern, TokenPattern::Regex(_)) && visible;
        let supertype = false;

        metadata.push(quote! {
            adze::ffi::TSSymbolMetadata {
                visible: #visible,
                named: #named,
                supertype: #supertype,
            }
        });
    }

    // Add metadata for non-terminals (rules)
    for symbol_id in grammar.rules.keys() {
        // For now, use generated rule names until we have proper symbol mapping
        let rule_name = format!("rule_{}", symbol_id.0);
        // Hidden rules start with underscore
        let visible = !rule_name.starts_with('_');
        // Non-terminals are named unless they're hidden
        let named = visible;
        // Check if this rule is in the supertypes list
        let supertype = grammar.supertypes.contains(symbol_id);

        metadata.push(quote! {
            adze::ffi::TSSymbolMetadata {
                visible: #visible,
                named: #named,
                supertype: #supertype,
            }
        });
    }

    // Add metadata for external symbols
    for external in &grammar.externals {
        // External tokens are typically visible and named
        let visible = !external.name.starts_with('_');
        let named = visible;
        let supertype = false;

        metadata.push(quote! {
            adze::ffi::TSSymbolMetadata {
                visible: #visible,
                named: #named,
                supertype: #supertype,
            }
        });
    }

    metadata
}

pub(crate) fn field_names(grammar: &Grammar) -> Vec<String> {
    // Fields must be in lexicographic order (already validated in Grammar)
    grammar.fields.values().cloned().collect()
}
