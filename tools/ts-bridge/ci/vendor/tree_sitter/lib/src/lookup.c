#include "./language.h"
#include "./tree_sitter_internal.h"
#include "tree_sitter/parser.h"
#include <assert.h>
#include <stdio.h>

// These functions access the parse table
const TSParseAction *ts_language_actions(
  const TSLanguage *self,
  TSStateId state,
  TSSymbol symbol,
  uint32_t *count
) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    *count = 0;
    return NULL;
  }
  
  // assert(symbol < lang->token_count);
  uint32_t action_index = ts_language_lookup(self, state, symbol);
  const TSParseActionEntry *entry = &lang->parse_actions[action_index];
  *count = entry->entry.count;
  return (const TSParseAction *)(entry + 1);
}

uint32_t ts_language_lookup(
  const TSLanguage *self,
  TSStateId state,
  TSSymbol symbol
) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  
  if (state < lang->large_state_count) {
    return lang->parse_table[state * lang->symbol_count + symbol];
  }

  uint32_t index = lang->small_parse_table_map[state - lang->large_state_count];
  const uint16_t *data = &lang->small_parse_table[index];

  uint32_t group_count = *(data++);
  for (uint16_t i = 0; i < group_count; i++) {
    uint16_t value = *(data++);
    uint16_t symbol_count = *(data++);
    for (uint16_t j = 0; j < symbol_count; j++) {
      if (*(data++) == symbol) return value;
    }
  }
  return 0;
}