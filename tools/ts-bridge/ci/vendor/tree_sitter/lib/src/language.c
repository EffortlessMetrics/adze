// Minimal Tree-sitter runtime for ts-bridge
// Only includes functions needed by the shim

#include "./language.h"
#include "./wasm_store.h"
#include "tree_sitter/api.h"
#include <string.h>

uint32_t ts_language_symbol_count(const TSLanguage *self) {
  return self->symbol_count + self->alias_count;
}

uint32_t ts_language_state_count(const TSLanguage *self) {
  return self->state_count;
}

uint32_t ts_language_version(const TSLanguage *self) {
  return self->version;
}

uint32_t ts_language_field_count(const TSLanguage *self) {
  return self->field_count;
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

TSSymbol ts_language_public_symbol(const TSLanguage *self, TSSymbol symbol) {
  if (symbol == ts_builtin_sym_error) return symbol;
  return self->public_symbol_map[symbol];
}

TSSymbolType ts_language_symbol_type(const TSLanguage *self, TSSymbol symbol) {
  TSSymbolMetadata metadata = ts_language_symbol_metadata(self, symbol);
  if (metadata.visible) {
    if (metadata.named) {
      return TSSymbolTypeRegular;
    } else {
      return TSSymbolTypeAnonymous;
    }
  } else {
    if (metadata.supertype) {
      return TSSymbolTypeSupertype;
    } else {
      return TSSymbolTypeAuxiliary;
    }
  }
}

const char *ts_language_field_name_for_id(const TSLanguage *self, TSFieldId id) {
  uint32_t count = ts_language_field_count(self);
  if (count && id <= count) {
    return self->field_names[id];
  } else {
    return NULL;
  }
}

TSFieldId ts_language_field_id_for_name(
  const TSLanguage *self,
  const char *name,
  uint32_t name_length
) {
  uint32_t count = ts_language_field_count(self);
  for (TSSymbol i = 1; i < count + 1; i++) {
    switch (strncmp(name, self->field_names[i], name_length)) {
      case 0:
        if (self->field_names[i][name_length] == 0) return i;
        break;
      default:
        break;
    }
  }
  return 0;
}

TSSymbolMetadata ts_language_symbol_metadata(const TSLanguage *self, TSSymbol symbol) {
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    return (TSSymbolMetadata) {.visible = true, .named = true};
  } else if (symbol < ts_language_symbol_count(self)) {
    return self->symbol_metadata[symbol];
  } else {
    return (TSSymbolMetadata) {0};
  }
}

TSSymbol ts_language_symbol_for_name(
  const TSLanguage *self,
  const char *string,
  uint32_t length,
  bool is_named
) {
  if (!strncmp(string, "ERROR", length)) return ts_builtin_sym_error;
  uint16_t count = (uint16_t)ts_language_symbol_count(self);
  for (TSSymbol i = 0; i < count; i++) {
    TSSymbolMetadata metadata = ts_language_symbol_metadata(self, i);
    if (
      metadata.visible &&
      (metadata.named == is_named) &&
      !strncmp(string, ts_language_symbol_name(self, i), length) &&
      !ts_language_symbol_name(self, i)[length]
    ) {
      return self->public_symbol_map[i];
    }
  }
  return 0;
}

TSLookaheadIterator *ts_lookahead_iterator_new(const TSLanguage *self, TSStateId state) {
  if (state >= self->state_count) return NULL;

  TSLookaheadIterator *iterator = (TSLookaheadIterator *)ts_calloc(1, sizeof(TSLookaheadIterator));
  if (!iterator) return NULL;

  iterator->language = self;
  iterator->state = state;
  iterator->next_state = state;
  iterator->symbol = 0;
  iterator->symbol_count = self->symbol_count;
  iterator->end_of_non_terminal_extras = UINT16_MAX;

  const TSParseAction *actions = self->parse_actions + 1;
  for (unsigned i = 0; i < self->token_count; i++) {
    uint32_t action_index = ts_language_lookup(self, state, i);
    const TSParseActionEntry *entry = &self->parse_actions[action_index];
    if (entry->entry.count > 0) {
      actions = (const TSParseAction *)(entry + 1);
      iterator->actions = actions;
      iterator->action_count = entry->entry.count;
      iterator->symbol = i;
      ts_lookahead_iterator__next(iterator);
      break;
    }
  }

  return iterator;
}

void ts_lookahead_iterator_delete(TSLookaheadIterator *self) {
  ts_free(self);
}

void ts_lookahead_iterator_reset_state(TSLookaheadIterator *self, TSStateId state) {
  if (state >= self->language->state_count) return;

  self->state = state;
  self->next_state = state;
  self->symbol = 0;
  self->action_count = 0;
  self->actions = NULL;

  const TSParseAction *actions = self->language->parse_actions + 1;
  for (unsigned i = 0; i < self->language->token_count; i++) {
    uint32_t action_index = ts_language_lookup(self->language, state, i);
    const TSParseActionEntry *entry = &self->language->parse_actions[action_index];
    if (entry->entry.count > 0) {
      actions = (const TSParseAction *)(entry + 1);
      self->actions = actions;
      self->action_count = entry->entry.count;
      self->symbol = i;
      ts_lookahead_iterator__next(self);
      break;
    }
  }
}

bool ts_lookahead_iterator_reset(TSLookaheadIterator *self, const TSLanguage *language, TSStateId state) {
  if (language->version != self->language->version) return false;
  ts_lookahead_iterator_reset_state(self, state);
  return true;
}

TSStateId ts_language_next_state(const TSLanguage *self, TSStateId state, TSSymbol symbol) {
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    return 0;
  } else if (symbol < self->token_count) {
    uint32_t count;
    const TSParseAction *actions = ts_language_actions(self, state, symbol, &count);
    if (count > 0) {
      TSParseAction action = actions[count - 1];
      if (action.type == TSParseActionTypeShift) {
        return action.shift.extra ? state : action.shift.state;
      }
    }
    return 0;
  } else {
    return ts_language_lookup(self, state, symbol);
  }
}

const TSParseAction *ts_language_actions(
  const TSLanguage *self,
  TSStateId state,
  TSSymbol symbol,
  uint32_t *count
) {
  *count = 0;
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    return NULL;
  }

  uint32_t action_index = ts_language_lookup(self, state, symbol);
  const TSParseActionEntry *entry = &self->parse_actions[action_index];
  *count = entry->entry.count;
  if (*count > 0) {
    return (const TSParseAction *)(entry + 1);
  }
  return NULL;
}

uint32_t ts_language_lookup(const TSLanguage *self, TSStateId state, TSSymbol symbol) {
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

TSSymbolMetadata ts_builtin_sym_error_metadata = {
  .visible = true,
  .named = true,
  .extra = false,
  .structural = false,
  .supertype = false
};

// Lookahead iterator implementation
void ts_lookahead_iterator__next(TSLookaheadIterator *self) {
  // Simplified implementation for ts-bridge
  // The full implementation would handle all edge cases
  if (!self || !self->actions || self->action_count == 0) return;
  
  // Move to next action
  self->actions++;
  self->action_count--;
  
  // If we've exhausted current symbol's actions, move to next symbol
  while (self->action_count == 0 && self->symbol < self->symbol_count - 1) {
    self->symbol++;
    uint32_t action_index = ts_language_lookup(self->language, self->state, self->symbol);
    const TSParseActionEntry *entry = &self->language->parse_actions[action_index];
    if (entry->entry.count > 0) {
      self->actions = (const TSParseAction *)(entry + 1);
      self->action_count = entry->entry.count;
      break;
    }
  }
}