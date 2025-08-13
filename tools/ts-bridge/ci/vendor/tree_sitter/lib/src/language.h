#ifndef TREE_SITTER_LANGUAGE_H_
#define TREE_SITTER_LANGUAGE_H_

#include "tree_sitter/api.h"
#include <stdbool.h>

// Language lookup implementation
uint32_t ts_language_lookup(const TSLanguage *, TSStateId, TSSymbol);

// Action retrieval
const TSParseAction *ts_language_actions(
  const TSLanguage *,
  TSStateId,
  TSSymbol,
  uint32_t *count
);

// Memory allocation functions
void *ts_malloc(size_t);
void *ts_calloc(size_t, size_t);
void *ts_realloc(void *, size_t);
void ts_free(void *);

// Lookahead iterator internals
void ts_lookahead_iterator__next(TSLookaheadIterator *);

// Check if language is WASM
static inline bool ts_language_is_wasm(const TSLanguage *self) {
  return (uintptr_t)self->create != 0 && (uintptr_t)self->create < 65536;
}

#endif // TREE_SITTER_LANGUAGE_H_