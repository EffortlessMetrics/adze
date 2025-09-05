#include "shim.h"
#include <stddef.h>
#include <stdint.h>

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
                uint32_t* tokc, uint32_t* extc) {
  *symc = ts_language_symbol_count(lang);
  *stc  = ts_language_state_count(lang);

  // Access internal fields for token counts
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  *tokc = lang_internal->token_count;
  *extc = lang_internal->external_token_count;
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

uint32_t tsb_table_entry(const TSLanguage* lang,
                         uint32_t state, uint32_t symbol,
                         TsbEntryHeader* out_hdr) {
  // Use internal knowledge of how Tree-sitter stores actions
  const TSLanguage_Internal* lang_internal = (const TSLanguage_Internal*)lang;
  
  // Special handling for error symbols
  if (symbol == (uint32_t)-1 || symbol == (uint32_t)-2) {
    out_hdr->count = 0;
    out_hdr->reusable = false;
    out_hdr->action_index = 0;
    return 0;
  }
  
  // Get the action index from the lookup table
  uint32_t action_index = ts_language_lookup(lang, (TSStateId)state, (TSSymbol)symbol);
  if (action_index == 0) {
    out_hdr->count = 0;
    out_hdr->reusable = false;
    out_hdr->action_index = 0;
    return 0;
  }
  
  // Get the entry from parse_actions
  const TSParseActionEntry *entry = &lang_internal->parse_actions[action_index];
  out_hdr->count = entry->entry.count;
  out_hdr->reusable = entry->entry.reusable;
  out_hdr->action_index = action_index;

  return action_index;
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
  // Simplified - many grammars have start at symbol 1
  return 1;
}

// Field maps stubs - will be implemented in PR2
const struct TSFieldMapSlice* tsb_field_map_slices(const TSLanguage* lang) { 
  return NULL; 
}

const struct TSFieldMapEntry* tsb_field_map_entries(const TSLanguage* lang) { 
  return NULL; 
}