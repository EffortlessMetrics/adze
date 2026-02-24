#include "shim.h"
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

// Use real Tree-sitter headers
#include <tree_sitter/api.h>
#include <tree_sitter/parser.h>

// Include internal header for accessing symbol_metadata
#include "../ci/vendor/tree_sitter/lib/src/tree_sitter_internal.h"

// ABI versions
uint32_t tsb_language_version(void) { 
  return TREE_SITTER_LANGUAGE_VERSION; 
}

uint32_t tsb_min_compatible_version(void) { 
  return TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION; 
}

void tsb_counts(const TSLanguage* lang,
                uint32_t* symc, uint32_t* stc,
                uint32_t* tokc, uint32_t* extc, uint32_t* lstc) {
  *symc = ts_language_symbol_count(lang);
  *stc  = ts_language_state_count(lang);

  // Access internal fields for token counts
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  *tokc = lang_internal->token_count;
  *extc = lang_internal->external_token_count;
  *lstc = lang_internal->large_state_count;

  // Debug: print entry 92
  const TSParseActionEntry *entry92 = &lang_internal->parse_actions[92];

  // Debug: print small_parse_table_map
  uint32_t small_state_count = lang_internal->state_count - lang_internal->large_state_count;
  for (uint32_t i = 0; i < small_state_count; i++) {
  }
}

const char* tsb_symbol_name(const TSLanguage* lang, uint32_t sym) {
  return ts_language_symbol_name(lang, (TSSymbol)sym);
}

TsbSymbolMetadata tsb_symbol_metadata(const TSLanguage* lang, uint32_t sym) {
  // Access the symbol_metadata array directly from the language struct
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  TSSymbolMetadata meta = lang_internal->symbol_metadata[sym];
  TsbSymbolMetadata result;
  result.visible = meta.visible;
  result.named = meta.named;
  return result;
}

// Forward declare the internal functions we need
// These are internal Tree-sitter functions not exposed in the public API
extern uint32_t ts_language_lookup(const TSLanguage *, TSStateId, TSSymbol);
extern TSStateId ts_language_next_state(const TSLanguage *, TSStateId, TSSymbol);
extern const TSParseAction *ts_language_actions(const TSLanguage *, TSStateId, TSSymbol, uint32_t *);

// Iterator functions (defined in language.c but might not be in api.h)
TSLookaheadIterator *ts_lookahead_iterator_new(const TSLanguage *self, TSStateId state);
void ts_lookahead_iterator_delete(TSLookaheadIterator *self);
bool ts_lookahead_iterator_next(TSLookaheadIterator *self);

uint32_t tsb_table_entry(const TSLanguage* lang,
                         uint32_t state, uint32_t symbol,
                         TsbEntryHeader* out_hdr) {
  TSLookaheadIterator *it = ts_lookahead_iterator_new(lang, (TSStateId)state);
  if (!it) {
    return 0;
  }

  uint32_t result_idx = 0;
  bool found = false;

  // The iterator is already at the first symbol after new()
  // because new() calls reset() which calls next().
  do {
    if (it->symbol == (TSSymbol)symbol) {
      const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
      const TSParseAction *actions_base = (const TSParseAction *)lang_internal->parse_actions;
      result_idx = (uint32_t)(it->actions - actions_base) - 1;

      out_hdr->count = (uint8_t)it->action_count;
      out_hdr->reusable = true;
      out_hdr->action_index = result_idx;
      found = true;
      break;
    }
  } while (ts_lookahead_iterator_next(it));

  ts_lookahead_iterator_delete(it);
  return found ? result_idx : 0;
}

static inline TsbAction from_ts_action(TSParseAction act) {
  TsbAction a = {0};
  switch (act.type) {
    case TSParseActionTypeShift:
      a.kind = TSB_ACTION_SHIFT;
      a.state = (uint16_t)act.shift.state;
      a.extra = act.shift.extra;
      a.repetition = act.shift.repetition;
      break;
    case TSParseActionTypeReduce:
      a.kind = TSB_ACTION_REDUCE;
      a.lhs = (uint16_t)act.reduce.symbol;
      a.rhs_len = act.reduce.child_count;
      a.dynamic_precedence = act.reduce.dynamic_precedence;
      a.production_id = act.reduce.production_id;
      break;
    case TSParseActionTypeAccept:
      a.kind = TSB_ACTION_ACCEPT;
      break;
    case TSParseActionTypeRecover:
      a.kind = TSB_ACTION_RECOVER;
      break;
  }
  return a;
}

uint32_t tsb_unpack_actions(const TSLanguage* lang,
                            uint32_t action_index, uint8_t count,
                            TsbAction* out, uint32_t cap) {
  // Get the actions from the parse_actions array
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  const TSParseActionEntry *entry = &lang_internal->parse_actions[action_index];
  const TSParseAction *actions = (const TSParseAction *)(entry + 1);

  uint32_t n = count < cap ? count : cap;
  for (uint32_t i = 0; i < n; i++) {
    out[i] = from_ts_action(actions[i]);
  }

  return n;
}

uint32_t tsb_next_state(const TSLanguage* lang, uint32_t state, uint32_t nonterm) {
  return (uint32_t)ts_language_next_state(lang, (TSStateId)state, (TSSymbol)nonterm);
}

uint32_t tsb_detect_start_symbol(const TSLanguage* lang) {
  uint32_t symc = ts_language_symbol_count(lang);
  uint32_t stc = ts_language_state_count(lang);
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  uint32_t tokc = lang_internal->token_count;
  uint32_t extc = lang_internal->external_token_count;
  uint32_t term_boundary = tokc + extc;

  // Search through states (prioritize 0 and 1 as they are common initial states)
  for (uint32_t state = 0; state < stc; state++) {
    for (uint32_t sym = term_boundary; sym < symc; sym++) {
      TSStateId next_state = ts_language_next_state(lang, (TSStateId)state, (TSSymbol)sym);
      if (next_state != 0) {
        uint32_t count = 0;
        const TSParseAction *actions = ts_language_actions(lang, next_state, 0, &count);
        for (uint32_t i = 0; i < count; i++) {
          if (actions[i].type == TSParseActionTypeAccept) {
            return sym;
          }
        }
      }
    }
    if (state > 10) break; // Don't search too far
  }

  // Fallback to symbol 1 if not found
  return 1;
}

// Field map accessors
// NOTE: these pointers reference data owned by the language and are valid for
// the entire lifetime of `lang`.
const struct TSFieldMapSlice* tsb_field_map_slices(const TSLanguage* lang) {
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  return (const struct TSFieldMapSlice*)lang_internal->field_map_slices;
}

const struct TSFieldMapEntry* tsb_field_map_entries(const TSLanguage* lang) {
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  return (const struct TSFieldMapEntry*)lang_internal->field_map_entries;
}