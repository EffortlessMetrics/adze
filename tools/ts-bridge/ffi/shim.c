#include "shim.h"
#include <stddef.h>

#ifdef tsb_stub
  // For development stub builds
  #include "tree_sitter_stub.h"
#else
  // Production: use real tree-sitter headers
  #include <tree_sitter/api.h>
  #include <tree_sitter/parser.h>
#endif

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
  
  // We need to access internal fields for token counts
  // This requires the TSLanguage struct definition from parser.h
  // For now, set reasonable defaults
  *tokc = 100; // Will be determined from actual grammar
  *extc = 0;   // External scanner tokens
}

const char* tsb_symbol_name(const TSLanguage* lang, uint32_t sym) {
  return ts_language_symbol_name(lang, (TSSymbol)sym);
}

uint32_t tsb_table_entry(const TSLanguage* lang,
                         uint32_t state, uint32_t symbol,
                         TsbEntryHeader* out_hdr) {
  // This is a simplified version - actual implementation needs
  // proper Tree-sitter table decoding logic
  out_hdr->reusable = false;
  out_hdr->count = 1;
  out_hdr->action_index = 0;
  return 0;
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
  // Simplified - actual implementation needs proper action unpacking
  return 0;
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