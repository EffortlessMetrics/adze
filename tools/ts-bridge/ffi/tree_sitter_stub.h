// Stub definitions for Tree-sitter API
// This is a placeholder - in production, use actual tree-sitter headers

#pragma once
#include <stdint.h>
#include <stdbool.h>

// Tree-sitter version constants
#define TREE_SITTER_LANGUAGE_VERSION 15
#define TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION 13

// Basic types
typedef uint16_t TSStateId;
typedef uint16_t TSSymbol;
typedef uint16_t TSFieldId;

// Symbol for EOF
#define ts_builtin_sym_end ((TSSymbol)0)

// Parse action types
typedef enum {
  TSParseActionTypeShift,
  TSParseActionTypeReduce,
  TSParseActionTypeAccept,
  TSParseActionTypeRecover,
} TSParseActionType;

// Parse action structures
typedef struct {
  TSStateId state;
  bool extra;
  bool repetition;
} TSParseActionShift;

typedef struct {
  TSSymbol symbol;
  uint8_t child_count;
  uint16_t production_id;
  int16_t dynamic_precedence;
} TSParseActionReduce;

typedef union {
  TSParseActionType type;
  TSParseActionShift shift;
  TSParseActionReduce reduce;
} TSParseAction;

typedef union {
  uint32_t index;
  struct {
    uint8_t count;
    bool reusable;
  } entry;
  TSParseAction action;
} TSParseActionEntry;

// Symbol metadata
typedef struct {
  bool visible;
  bool named;
  bool supertype;
} TSSymbolMetadata;

// Forward declare TSLanguage
typedef struct TSLanguage TSLanguage;

// Stub API functions
uint32_t ts_language_symbol_count(const TSLanguage *);
uint32_t ts_language_state_count(const TSLanguage *);
const char *ts_language_symbol_name(const TSLanguage *, TSSymbol);
uint32_t ts_language_table_entry(const TSLanguage *, TSStateId, TSSymbol, TSParseActionEntry *);
TSStateId ts_language_next_state(const TSLanguage *, TSStateId, TSSymbol);