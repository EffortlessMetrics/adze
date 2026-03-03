# Architecture

Adze transforms annotated Rust types into high-performance GLR parsers. This page explains the crate organization, build pipeline, and runtime.

## Crate Organization

The workspace is split into focused microcrates (see the [Microcrate Guide](microcrates.md) for the full list). At the highest level they fall into three layers:

```text
┌─ Grammar Definition ──────────┐
│  macro/      proc-macro attrs  │
│  common/     shared expansion  │
└────────────────────────────────┘
          │  extracts rules
          ▼
┌─ Build-Time Generation ───────┐
│  ir/         grammar IR        │
│  glr-core/   LR(1) + GLR      │
│  tablegen/   table compression │
│  tool/       build.rs driver   │
└────────────────────────────────┘
          │  emits parser code
          ▼
┌─ Runtime Execution ───────────┐
│  runtime/    legacy + Extract  │
│  runtime2/   production GLR    │
└────────────────────────────────┘
```

### Dependency flow

```text
tool ─┐
      ├─▶ common ─▶ ir ─▶ glr-core ─▶ tablegen
macro ┘
```

Build-time crates are **never** linked into the user's final binary; only the runtime crate is.

## Build Pipeline

When you run `cargo build` on a project that uses Adze:

```text
Rust types          IR               Parse tables        Compiled parser
with annotations ──▶ Grammar ──▶ LR(1) automaton ──▶ compressed ──▶ linked into
                    (ir/)       (glr-core/)          (tablegen/)    binary
```

### Stage 1 — Macro Expansion (`macro/`, `common/`)

`#[adze::grammar]` and friends collect type information. The `common` crate contains the actual expansion logic shared between the proc-macro and the build tool.

### Stage 2 — IR Construction (`ir/`)

The tool reads the annotated source and builds a `Grammar` IR. Complex symbols (`Optional`, `Repeat`, `Choice`, `Sequence`) are normalized into auxiliary rules via `Grammar::normalize()`.

### Stage 3 — GLR Analysis (`glr-core/`)

FIRST/FOLLOW sets are computed, then the canonical LR(1) collection is built. Conflicts are **preserved** (not eliminated) to support GLR parsing — each state/symbol cell can hold multiple actions.

### Stage 4 — Table Generation (`tablegen/`)

Parse tables are compressed using Tree-sitter-compatible algorithms and emitted as static `Language` structs with FFI-compatible layout. The output also includes `NODE_TYPES` JSON metadata.

### Stage 5 — Code Emission (`tool/`)

`adze_tool::build_parsers()` ties the stages together. It writes generated Rust (or C) source files and compile instructions for Cargo.

## Runtime

### The `Extract` trait

The runtime crate (`runtime/`) provides `Extract`, the core trait that converts a raw parse tree node into a typed Rust value. The generated code implements `Extract` for every type in your grammar module.

### GLR Engine (`runtime2/`)

`runtime2` is the production runtime:

| Component | File | Purpose |
|---|---|---|
| Parser API | `parser.rs` | Tree-sitter-compatible `Parser` struct |
| GLR Engine | `engine.rs` | Fork/merge driver over parse tables |
| Tree Builder | `builder.rs` | Converts GLR forests to trees |
| Tree | `tree.rs` | Node API with incremental edit support |

Parsing flow:

```text
source text
    │
    ▼
  Lexer ──▶ Token stream
                │
                ▼
          GLR Driver ──▶ Parse forest (may contain ambiguity)
                              │
                              ▼
                        Tree Builder ──▶ Tree-sitter compatible Tree
                                              │
                                              ▼
                                        Extract ──▶ Typed AST
```

### Performance monitoring

Set `ADZE_LOG_PERFORMANCE=true` to log forest-to-tree conversion statistics (node count, tree depth, elapsed time).

## Key Design Decisions

1. **Two-stage processing** — macros mark types; the build tool generates the parser. This avoids proc-macro limitations (no file I/O, no cross-crate state).
2. **Conflict preservation** — GLR tables keep all shift/reduce and reduce/reduce conflicts so the parser can fork at runtime, enabling ambiguous-grammar support.
3. **Tree-sitter ABI compatibility** — generated `Language` structs match Tree-sitter's C ABI exactly, allowing interop with existing Tree-sitter tooling.
4. **Bounded concurrency** — all parallel work respects configurable caps (`RUST_TEST_THREADS`, `RAYON_NUM_THREADS`) to prevent resource exhaustion.

## Further Reading

- [Microcrate Guide](microcrates.md) — detailed per-crate responsibilities
- [Development Architecture](development/architecture.md) — deeper diagrams and data-flow details
- [GLR Parsing](advanced/glr-parsing.md) — how the GLR algorithm works in Adze
