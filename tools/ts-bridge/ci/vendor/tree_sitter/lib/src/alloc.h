#ifndef TREE_SITTER_ALLOC_H_
#define TREE_SITTER_ALLOC_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdlib.h>

// Allow clients to override allocation functions
extern void *(*ts_current_malloc)(size_t size);
extern void *(*ts_current_calloc)(size_t count, size_t size);
extern void *(*ts_current_realloc)(void *ptr, size_t size);
extern void (*ts_current_free)(void *ptr);

// Convenience macros for calling these functions
#define ts_malloc ts_current_malloc
#define ts_calloc ts_current_calloc
#define ts_realloc ts_current_realloc
#define ts_free ts_current_free

#ifdef __cplusplus
}
#endif

#endif  // TREE_SITTER_ALLOC_H_