// Minimal Tree-sitter runtime for ts-bridge
// Only includes functions actually used by our shim

#include "tree_sitter/api.h"
#include "tree_sitter/parser.h"
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>

// Define the missing constant if not already defined
#ifndef ts_builtin_sym_error_repeat
#define ts_builtin_sym_error_repeat ((TSSymbol)-1)
#endif

// TableEntry is in api.h but let's make sure
typedef struct {
  uint32_t action_count;
  bool is_reusable;
  const TSParseAction *actions;
} TableEntry;

// Memory functions
void *ts_malloc(size_t size) { return malloc(size); }
void *ts_calloc(size_t count, size_t size) { return calloc(count, size); }
void *ts_realloc(void *ptr, size_t size) { return realloc(ptr, size); }
void ts_free(void *ptr) { free(ptr); }

// Language functions used by shim
uint32_t ts_language_symbol_count(const TSLanguage *self) {
  return self->symbol_count + self->alias_count;
}

uint32_t ts_language_state_count(const TSLanguage *self) {
  return self->state_count;
}

const char *ts_language_symbol_name(const TSLanguage *self, TSSymbol symbol) {
  if (symbol == ts_builtin_sym_error) {
    return "ERROR";
  } else if (symbol == ts_builtin_sym_error_repeat) {
    return "_ERROR";
  } else if (symbol < ts_language_symbol_count(self)) {
    return self->symbol_names[symbol];
  } else {
    return NULL;
  }
}

// Lookup implementation (needed by ts_language_table_entry and ts_language_next_state)
static inline uint32_t ts_language_lookup(const TSLanguage *self, TSStateId state, TSSymbol symbol) {
  if (state >= self->large_state_count) {
    uint32_t index = self->small_parse_table_map[state - self->large_state_count];
    const uint16_t *data = &self->small_parse_table[index];
    uint16_t group_count = *(data++);
    for (unsigned i = 0; i < group_count; i++) {
      uint16_t section_value = *(data++);
      uint16_t symbol_count = section_value & 0xFF;
      TSSymbol section_symbol = (section_value >> 8);
      if (section_symbol <= symbol && symbol < section_symbol + symbol_count) {
        return *(data + symbol - section_symbol);
      }
      data += symbol_count;
    }
    return 0;
  } else {
    return self->parse_table[state * self->symbol_count + symbol];
  }
}

void ts_language_table_entry(
  const TSLanguage *self,
  TSStateId state,
  TSSymbol symbol,
  TableEntry *result
) {
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    result->action_count = 0;
    result->is_reusable = false;
    result->actions = NULL;
  } else {
    uint32_t action_index = ts_language_lookup(self, state, symbol);
    const TSParseActionEntry *entry = &self->parse_actions[action_index];
    result->action_count = entry->entry.count;
    result->is_reusable = entry->entry.reusable;
    result->actions = (const TSParseAction *)(entry + 1);
  }
}

TSStateId ts_language_next_state(const TSLanguage *self, TSStateId state, TSSymbol symbol) {
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    return 0;
  } else if (symbol < self->token_count) {
    uint32_t action_index = ts_language_lookup(self, state, symbol);
    const TSParseActionEntry *entry = &self->parse_actions[action_index];
    if (entry->entry.count > 0) {
      const TSParseAction *actions = (const TSParseAction *)(entry + 1);
      TSParseAction action = actions[entry->entry.count - 1];
      if (action.type == TSParseActionTypeShift) {
        return action.shift.extra ? state : action.shift.state;
      }
    }
    return 0;
  } else {
    return ts_language_lookup(self, state, symbol);
  }
}