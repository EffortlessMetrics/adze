#ifndef TREE_SITTER_WASM_STORE_H_
#define TREE_SITTER_WASM_STORE_H_

#ifdef __cplusplus
extern "C" {
#endif

#include "tree_sitter/api.h"

// Stub WASM support functions (not implemented for ts-bridge)
static inline void ts_wasm_language_retain(const TSLanguage *self) {
  (void)self;
}

static inline void ts_wasm_language_release(const TSLanguage *self) {
  (void)self;
}

#ifdef __cplusplus
}
#endif

#endif  // TREE_SITTER_WASM_STORE_H_