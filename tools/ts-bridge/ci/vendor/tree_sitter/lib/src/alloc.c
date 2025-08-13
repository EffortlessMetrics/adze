// Minimal memory allocation functions for Tree-sitter runtime

#include <stdlib.h>
#include <stdbool.h>

// Tree-sitter allows custom allocators, but we'll use the standard ones
void *(*ts_current_malloc)(size_t) = malloc;
void *(*ts_current_calloc)(size_t, size_t) = calloc;
void *(*ts_current_realloc)(void *, size_t) = realloc;
void (*ts_current_free)(void *) = free;

void *ts_malloc(size_t size) {
  return ts_current_malloc(size);
}

void *ts_calloc(size_t count, size_t size) {
  return ts_current_calloc(count, size);
}

void *ts_realloc(void *ptr, size_t size) {
  return ts_current_realloc(ptr, size);
}

void ts_free(void *ptr) {
  ts_current_free(ptr);
}

void ts_set_allocator(
  void *(*new_malloc)(size_t),
  void *(*new_calloc)(size_t, size_t),
  void *(*new_realloc)(void *, size_t),
  void (*new_free)(void *)
) {
  ts_current_malloc = new_malloc;
  ts_current_calloc = new_calloc;
  ts_current_realloc = new_realloc;
  ts_current_free = new_free;
}