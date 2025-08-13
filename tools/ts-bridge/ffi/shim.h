#pragma once
#include <stdint.h>
#include <stdbool.h>

// Forward declarations
typedef struct TSLanguage TSLanguage;

// --- ABI version helpers ---
uint32_t tsb_language_version(void);
uint32_t tsb_min_compatible_version(void);

// --- Plain actions exported to Rust (no unions over FFI) ---
typedef enum {
  TSB_ACTION_SHIFT   = 0,
  TSB_ACTION_REDUCE  = 1,
  TSB_ACTION_ACCEPT  = 2,
  TSB_ACTION_RECOVER = 3
} TsbActionKind;

typedef struct {
  TsbActionKind kind;
  uint16_t      state;               // for SHIFT
  uint16_t      lhs;                 // for REDUCE
  uint16_t      rhs_len;             // for REDUCE
  int16_t       dynamic_precedence;  // for REDUCE
  uint16_t      production_id;       // for REDUCE (maps to field_map slice)
  bool          extra;               // SHIFT.extra
  bool          repetition;          // SHIFT.repetition
} TsbAction;

// --- Counts & symbol names ---
void tsb_counts(const TSLanguage* lang,
                uint32_t* symc, uint32_t* stc,
                uint32_t* tokc, uint32_t* extc);

const char* tsb_symbol_name(const TSLanguage* lang, uint32_t sym);

// --- Table access ---
typedef struct {
  uint32_t action_index;  // index into lang->parse_actions
  uint8_t  count;         // number of actions in the cell
  bool     reusable;
} TsbEntryHeader;

uint32_t tsb_table_entry(const TSLanguage* lang,
                         uint32_t state, uint32_t symbol,
                         TsbEntryHeader* out_hdr);

// Unpack 'count' actions starting at 'action_index' into 'out' (cap-limited).
uint32_t tsb_unpack_actions(const TSLanguage* lang,
                            uint32_t action_index, uint8_t count,
                            TsbAction* out, uint32_t cap);

uint32_t tsb_next_state(const TSLanguage* lang, uint32_t state, uint32_t nonterm);

// Start symbol detection via EOF cell containing ACCEPT (+ last REDUCE.lhs if present).
uint32_t tsb_detect_start_symbol(const TSLanguage* lang);

// --- Field map pointers (wired later in PR2) ---
// These types are defined in tree_sitter/parser.h, so we just forward declare
struct TSFieldMapSlice;
struct TSFieldMapEntry;

const struct TSFieldMapSlice* tsb_field_map_slices(const TSLanguage* lang);
const struct TSFieldMapEntry* tsb_field_map_entries(const TSLanguage* lang);