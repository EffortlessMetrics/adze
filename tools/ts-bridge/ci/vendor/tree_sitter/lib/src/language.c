#include "./language.h"
#include "./wasm_store.h"
#include "./tree_sitter_internal.h"
#include "tree_sitter/api.h"
#include <string.h>
#include <assert.h>

// Forward declarations
void ts_lookahead_iterator_reset(TSLookaheadIterator *self, TSStateId state);
bool ts_lookahead_iterator_next(TSLookaheadIterator *self);

const TSLanguage *ts_language_copy(const TSLanguage *self) {
  if (self && ts_language_is_wasm(self)) {
    ts_wasm_language_retain(self);
  }
  return self;
}

void ts_language_delete(const TSLanguage *self) {
  if (self && ts_language_is_wasm(self)) {
    ts_wasm_language_release(self);
  }
}

uint32_t ts_language_symbol_count(const TSLanguage *self) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return lang->symbol_count + lang->alias_count;
}

uint32_t ts_language_state_count(const TSLanguage *self) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return lang->state_count;
}

uint32_t ts_language_version(const TSLanguage *self) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return lang->version;
}

uint32_t ts_language_field_count(const TSLanguage *self) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return lang->field_count;
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
    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
    assert(symbol < lang->token_count);
    uint32_t action_index = ts_language_lookup(self, state, symbol);
    const TSParseActionEntry *entry = &lang->parse_actions[action_index];
    result->action_count = entry->entry.count;
    result->is_reusable = entry->entry.reusable;
    result->actions = (const TSParseAction *)(entry + 1);
  }
}

TSSymbolMetadata ts_language_symbol_metadata(
  const TSLanguage *self,
  TSSymbol symbol
) {
  if (symbol == ts_builtin_sym_error) {
    return (TSSymbolMetadata) {.visible = true, .named = true};
  } else if (symbol == ts_builtin_sym_error_repeat) {
    return (TSSymbolMetadata) {.visible = false, .named = false};
  } else {
    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
    return lang->symbol_metadata[symbol];
  }
}

TSSymbol ts_language_public_symbol(
  const TSLanguage *self,
  TSSymbol symbol
) {
  if (symbol == ts_builtin_sym_error) return symbol;
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  return lang->public_symbol_map[symbol];
}

TSStateId ts_language_next_state(
  const TSLanguage *self,
  TSStateId state,
  TSSymbol symbol
) {
  if (symbol == ts_builtin_sym_error || symbol == ts_builtin_sym_error_repeat) {
    return 0;
  } else if (symbol < ((const TSLanguage_Internal *)self)->token_count) {
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

const char *ts_language_symbol_name(
  const TSLanguage *self,
  TSSymbol symbol
) {
  if (symbol == ts_builtin_sym_error) {
    return "ERROR";
  } else if (symbol == ts_builtin_sym_error_repeat) {
    return "_ERROR";
  } else if (symbol < ts_language_symbol_count(self)) {
    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
    return lang->symbol_names[symbol];
  } else {
    return NULL;
  }
}

TSSymbol ts_language_symbol_for_name(
  const TSLanguage *self,
  const char *string,
  uint32_t length,
  bool is_named
) {
  if (!strncmp(string, "ERROR", length)) return ts_builtin_sym_error;
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  for (TSSymbol i = 0; i < lang->symbol_count; i++) {
    TSSymbolMetadata metadata = ts_language_symbol_metadata(self, i);
    if ((!metadata.visible && !metadata.supertype) || metadata.named != is_named) continue;
    const char *symbol_name = lang->symbol_names[i];
    if (!strncmp(symbol_name, string, length) && !symbol_name[length]) {
      return lang->public_symbol_map[i];
    }
  }
  return 0;
}

TSSymbolType ts_language_symbol_type(
  const TSLanguage *self,
  TSSymbol symbol
) {
  TSSymbolMetadata metadata = ts_language_symbol_metadata(self, symbol);
  if (metadata.named && metadata.visible) {
    return TSSymbolTypeRegular;
  } else if (metadata.visible) {
    return TSSymbolTypeAnonymous;
  } else {
    return TSSymbolTypeAuxiliary;
  }
}

const char *ts_language_field_name_for_id(
  const TSLanguage *self,
  TSFieldId id
) {
  uint32_t count = ts_language_field_count(self);
  if (count && id <= count) {
    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
    return lang->field_names[id];
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
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  for (TSSymbol i = 1; i < count + 1; i++) {
    switch (strncmp(name, lang->field_names[i], name_length)) {
      case 0:
        if (lang->field_names[i][name_length] == 0) return i;
        break;
      case -1:
        return 0;
      default:
        break;
    }
  }
  return 0;
}

TSLookaheadIterator *ts_lookahead_iterator_new(const TSLanguage *self, TSStateId state) {
  const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self;
  if (state >= lang->state_count) return NULL;

  TSLookaheadIterator *iterator = ts_malloc(sizeof(TSLookaheadIterator));
  *iterator = (TSLookaheadIterator) {
    .language = self,
    .state = state,
    .next_state = 0,
    .symbol = 0,
    .symbol_count = lang->symbol_count,
    .end_of_nonterminal_extras = (const bool *)lang->symbol_metadata + lang->symbol_count,
    .actions = NULL,
    .action_count = 0,
  };
  ts_lookahead_iterator_reset(iterator, state);
  return iterator;
}

void ts_lookahead_iterator_delete(TSLookaheadIterator *self) {
  ts_free(self);
}

void ts_lookahead_iterator_reset(TSLookaheadIterator *self, TSStateId state) {
  self->state = state;
  self->next_state = 0;
  self->symbol = 0;
  self->action_count = 0;
  self->actions = NULL;
  ts_lookahead_iterator_next(self);
}

TSStateId ts_lookahead_iterator_reset_state(TSLookaheadIterator *self) {
  return self->state;
}

const TSLanguage *ts_lookahead_iterator_language(const TSLookaheadIterator *self) {
  return self->language;
}

bool ts_lookahead_iterator_next(TSLookaheadIterator *self) {
  // For error-repair nodes, the state is stored in the first 2 bytes of the tree,
  // so this "state" value can exceed UINT16_MAX and thus exceed the language's
  // state count.
  if (self->state >= self->language->state_count) {
    self->symbol = 0;
    self->action_count = 0;
    return false;
  }

  while (self->action_count == 0) {
    if (self->symbol >= self->symbol_count) return false;

    uint32_t entry_index = ts_language_lookup(
      self->language,
      self->state,
      self->symbol
    );

    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self->language;
    const TSParseActionEntry *entry = &lang->parse_actions[entry_index];

    self->action_count = entry->entry.count;
    self->actions = (const TSParseAction *)(entry + 1);

    if (self->action_count) {
      self->next_state = self->state;
      return true;
    } else {
      const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self->language;
      if (
        self->symbol >= lang->token_count &&
        self->end_of_nonterminal_extras[self->symbol - lang->token_count]
      ) return false;
      self->symbol++;
    }
  }

  do {
    self->actions++;
    self->action_count--;
  } while (
    self->action_count > 0 && (
      self->actions->type == TSParseActionTypeRecover ||
      self->actions->type == TSParseActionTypeReduce
    )
  );

  if (self->action_count) {
    self->next_state = self->actions->type == TSParseActionTypeShift
      ? self->actions->shift.state
      : self->state;
    return true;
  } else {
    const TSLanguage_Internal *lang = (const TSLanguage_Internal *)self->language;
    if (
      self->symbol >= lang->token_count &&
      self->end_of_nonterminal_extras[self->symbol - lang->token_count]
    ) return false;
    self->symbol++;
    return ts_lookahead_iterator_next(self);
  }
}

TSSymbol ts_lookahead_iterator_current_symbol(const TSLookaheadIterator *self) {
  if (self->action_count && self->actions->type != TSParseActionTypeReduce) {
    return self->symbol;
  } else {
    return 0;
  }
}

const char *ts_lookahead_iterator_current_symbol_name(const TSLookaheadIterator *self) {
  return ts_language_symbol_name(self->language, self->symbol);
}