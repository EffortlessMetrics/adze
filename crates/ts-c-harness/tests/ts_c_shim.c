#include <stdint.h>

typedef struct TSLanguage TSLanguage;

// Internal runtime functions (we re-declare the signatures)
uint32_t ts_language_lookup(const TSLanguage*, uint16_t state, uint16_t symbol);
uint16_t ts_language_next_state(const TSLanguage*, uint16_t state, uint16_t symbol);

// Thin exported wrappers
uint32_t tsb_lookup(const TSLanguage* l, uint16_t s, uint16_t y) { return ts_language_lookup(l, s, y); }
uint16_t tsb_next_state(const TSLanguage* l, uint16_t s, uint16_t y) { return ts_language_next_state(l, s, y); }