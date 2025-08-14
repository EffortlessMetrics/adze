// Stub implementations for Tree-sitter functions
// These are placeholders - in production, link against actual tree-sitter library

#include "tree_sitter_stub.h"
#include <stddef.h>

// Stub implementations that return dummy values
uint32_t ts_language_symbol_count(const TSLanguage *lang) {
    // Return a dummy value for testing
    return 50;
}

uint32_t ts_language_state_count(const TSLanguage *lang) {
    // Return a dummy value for testing
    return 100;
}

const char *ts_language_symbol_name(const TSLanguage *lang, TSSymbol symbol) {
    // Return dummy names
    static const char* names[] = {
        "EOF", "start", "identifier", "number", "string",
        "plus", "minus", "star", "slash", "expression"
    };
    if (symbol < 10) {
        return names[symbol];
    }
    return "unknown";
}

TSSymbolMetadata ts_language_symbol_metadata(const TSLanguage *lang, TSSymbol symbol) {
    // Return dummy metadata
    TSSymbolMetadata meta;
    meta.visible = (symbol % 2 == 0);
    meta.named = (symbol % 3 == 0);
    meta.supertype = false;
    return meta;
}

uint32_t ts_language_table_entry(const TSLanguage *lang, TSStateId state, TSSymbol symbol, TSParseActionEntry *entry) {
    // Return dummy entry
    entry->entry.count = 0;
    entry->entry.reusable = false;
    return 0;
}

TSStateId ts_language_next_state(const TSLanguage *lang, TSStateId state, TSSymbol symbol) {
    // Return dummy next state
    return 0;
}