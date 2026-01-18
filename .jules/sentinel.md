# Sentinel's Journal

## 2025-11-20 - Self-Referential Struct Vulnerability
**Vulnerability:** Found a self-referential struct pattern in `TsLexFnAdapter` where a struct field (`ts.data`) held a raw pointer to another field (`backing`) of the same struct. When the struct moves (e.g. returned from `new`), the pointer becomes dangling.
**Learning:** Rust's move semantics invalidate self-references unless pinned or heap-allocated. While the code patched the pointer just-in-time in `next_internal`, this is fragile and relies on implementation details that could easily change, leading to Use-After-Free.
**Prevention:** Use `Box<T>` or `Pin<Box<T>>` for the referenced data so its memory address remains stable even when the owner struct moves.
