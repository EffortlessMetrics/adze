#ifndef TREE_SITTER_INTERNAL_H_
#define TREE_SITTER_INTERNAL_H_

#include "tree_sitter/api.h"
#include "tree_sitter/parser.h"
#include <stdbool.h>

// Missing constant definition
#ifndef ts_builtin_sym_error_repeat
#define ts_builtin_sym_error_repeat ((TSSymbol)-1)
#endif

// TableEntry type used in ts_language_table_entry
typedef struct {
  uint32_t action_count;
  bool is_reusable;
  const TSParseAction *actions;
} TableEntry;

// Internal TSLookaheadIterator structure
struct TSLookaheadIterator {
  const TSLanguage *language;
  TSStateId state;
  TSStateId next_state;
  TSSymbol symbol;
  TSSymbol symbol_count;
  const bool *end_of_nonterminal_extras;
  const TSParseAction *actions;
  uint32_t action_count;
};

// TSLanguage internal fields needed by our code
// Note: This is a partial definition containing only fields we need
typedef struct TSLanguage_Internal {
  uint32_t version;
  uint32_t symbol_count;
  uint32_t alias_count;
  uint32_t token_count;
  uint32_t external_token_count;
  uint32_t state_count;
  uint32_t large_state_count;
  uint32_t production_id_count;
  uint32_t field_count;
  uint16_t max_alias_sequence_length;
  const uint16_t *parse_table;
  const uint16_t *small_parse_table;
  const uint32_t *small_parse_table_map;
  const TSParseActionEntry *parse_actions;
  const char * const *symbol_names;
  const char * const *field_names;
  const TSFieldMapSlice *field_map_slices;
  const TSFieldMapEntry *field_map_entries;
  const TSSymbolMetadata *symbol_metadata;
  const TSSymbol *public_symbol_map;
  const uint16_t *alias_map;
  const TSSymbol *alias_sequences;
  const TSLexMode *lex_modes;
  bool (*lex)(TSLexer *, TSStateId);
  bool (*keyword_lex)(TSLexer *, TSStateId);
  TSSymbol keyword_capture_token;
  struct {
    const bool *states;
    const TSSymbol *symbol_map;
    void *(*create)(void);
    void (*destroy)(void *);
    bool (*scan)(void *, TSLexer *, const bool *symbol_whitelist);
    unsigned (*serialize)(void *, char *);
    void (*deserialize)(void *, const char *, unsigned);
  } external_scanner;
  const TSStateId *primary_state_ids;
  // For WASM support - not used in ts-bridge
  void *create;
} TSLanguage_Internal;

#endif  // TREE_SITTER_INTERNAL_H_