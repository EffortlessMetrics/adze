#include "shim.h"
// For development, use stub; in production, use real tree-sitter headers
#include "tree_sitter_stub.h"

// The TSLanguage struct definition - this matches the internal layout
// and is validated by our ABI hash checks
struct TSLanguage {
  uint32_t version;
  uint32_t symbol_count;
  const char * const *symbol_names;
  const void *symbol_metadata;
  const uint16_t *parse_table;
  const uint16_t *small_parse_table;
  const uint32_t *small_parse_table_map;
  const void *parse_actions;  // TSParseActionEntry array
  const char * const *field_names;
  const void *field_map_slices;
  const void *field_map_entries;
  const void *alias_map;
  const void *alias_sequences;
  const void *lex_modes;
  bool (*lex_fn)(void *, uint32_t);
  bool (*keyword_lex_fn)(void *, uint32_t);
  uint32_t keyword_capture_token;
  void *external_scanner;
  const void *public_symbol_map;
  uint16_t state_count;
  uint16_t large_state_count;
  uint16_t production_id_count;
  uint16_t field_count;
  uint16_t max_alias_sequence_length;
  uint16_t parse_action_count;
  uint16_t language_id;
  uint16_t external_token_count;
  uint16_t tree_sitter_min_compatible_language_version;
};

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
  // Note: We're accessing internal fields here which is why we need ABI guards
  // In a real implementation, we'd use proper accessors if available
  *tokc = 0; // Will be determined from symbol metadata
  *extc = lang->external_token_count;
  
  // Count terminals by checking symbol metadata
  const TSSymbolMetadata *metadata = (const TSSymbolMetadata *)lang->symbol_metadata;
  for (uint32_t i = 0; i < *symc; i++) {
    // Terminal symbols are those that are not named or are keywords
    // This is a simplification - actual logic may need refinement
    if (i < *symc - lang->production_id_count) {
      (*tokc)++;
    }
  }
  *tokc -= *extc;  // Adjust for external tokens
}

const char* tsb_symbol_name(const TSLanguage* lang, uint32_t sym) {
  return ts_language_symbol_name(lang, (TSSymbol)sym);
}

uint32_t tsb_table_entry(const TSLanguage* lang,
                         uint32_t state, uint32_t symbol,
                         TsbEntryHeader* out_hdr) {
  // Use Tree-sitter's helper to decode the table entry
  TSParseActionEntry entry;
  uint32_t idx = ts_language_table_entry(lang, (TSStateId)state, (TSSymbol)symbol, &entry);
  
  // The entry is a union - check if it's a count or a single action
  // For simplicity, we'll treat all as sequences
  out_hdr->reusable = false;
  out_hdr->count = 1;  // Default to single action
  out_hdr->action_index = idx;
  
  // If the entry has a count field (multi-action cell)
  // This needs proper union discrimination logic
  const TSParseActionEntry *actions = (const TSParseActionEntry *)lang->parse_actions;
  if (actions && idx < lang->parse_action_count) {
    // Check if this is a multi-action entry
    // The first entry at idx might be a header with count
    out_hdr->count = 1; // Simplified - actual implementation needs union check
  }
  
  return idx;
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
  // Get the parse actions array
  const TSParseActionEntry *actions = (const TSParseActionEntry *)lang->parse_actions;
  if (!actions) return 0;
  
  uint32_t written = 0;
  for (uint32_t i = 0; i < count && written < cap; i++) {
    // Skip the header entry and read the actual actions
    TSParseActionEntry e = actions[action_index + i];
    // This is simplified - need proper union handling
    TSParseAction act;
    act.type = TSParseActionTypeShift; // Placeholder
    out[written++] = from_ts_action(act);
  }
  return written;
}

uint32_t tsb_next_state(const TSLanguage* lang, uint32_t state, uint32_t nonterm) {
  return (uint32_t)ts_language_next_state(lang, (TSStateId)state, (TSSymbol)nonterm);
}

uint32_t tsb_detect_start_symbol(const TSLanguage* lang) {
  uint32_t stc = ts_language_state_count(lang);
  for (uint32_t s = 0; s < stc; s++) {
    TSParseActionEntry entry;
    uint32_t idx = ts_language_table_entry(lang, (TSStateId)s, ts_builtin_sym_end, &entry);
    
    // Check if this entry contains an Accept action
    // This is simplified - need proper implementation
    const TSParseActionEntry *actions = (const TSParseActionEntry *)lang->parse_actions;
    if (actions && idx < lang->parse_action_count) {
      // Look for Accept action and preceding Reduce
      // For now, return a reasonable default
      return 1;
    }
  }
  // Fallback: many grammars have start at 1
  return 1;
}

// Field maps stubs - will be implemented in PR2
const TSFieldMapSlice* tsb_field_map_slices(const TSLanguage* lang) { 
  return (const TSFieldMapSlice*)lang->field_map_slices; 
}

const TSFieldMapEntry* tsb_field_map_entries(const TSLanguage* lang) { 
  return (const TSFieldMapEntry*)lang->field_map_entries; 
}