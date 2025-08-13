#ifndef TREE_SITTER_WASM_STORE_H_
#define TREE_SITTER_WASM_STORE_H_

#include "tree_sitter/api.h"

// WASM support stubs (not needed for ts-bridge)
static inline void ts_wasm_language_retain(const TSLanguage *self) {}
static inline void ts_wasm_language_release(const TSLanguage *self) {}

#endif // TREE_SITTER_WASM_STORE_H_