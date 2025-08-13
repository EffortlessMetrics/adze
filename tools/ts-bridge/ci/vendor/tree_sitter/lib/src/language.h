#ifndef TREE_SITTER_LANGUAGE_H_
#define TREE_SITTER_LANGUAGE_H_

#ifdef __cplusplus
extern "C" {
#endif

#include "./alloc.h"
#include "./tree_sitter_internal.h"
#include "tree_sitter/api.h"
#include "tree_sitter/parser.h"
#include <stdbool.h>
#include <stdint.h>

// Language functions
const TSParseAction *ts_language_actions(const TSLanguage *, TSStateId, TSSymbol, uint32_t *);
uint32_t ts_language_lookup(const TSLanguage *, TSStateId, TSSymbol);

// WASM support check
static inline bool ts_language_is_wasm(const TSLanguage *self) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return (uintptr_t)lang->create != 0 && (uintptr_t)lang->create < 65536;
}

#ifdef __cplusplus
}
#endif

#endif  // TREE_SITTER_LANGUAGE_H_