# adze-glr-core

GLR (Generalized LR) parser generation algorithms for the [Adze](https://github.com/EffortlessMetrics/adze) parser toolchain.

## Overview

`adze-glr-core` implements the core GLR parsing algorithms used by Adze. It handles FIRST/FOLLOW set computation, LR(1) item set construction, conflict detection, and GLR fork/merge logic.

## Key Components

- **FIRST/FOLLOW Sets** — Computation of lookahead sets for grammar symbols
- **LR(1) Item Sets** — Canonical collection construction
- **Conflict Detection** — Identifies shift-reduce and reduce-reduce conflicts
- **GLR Fork/Merge** — Stack forking and merging for ambiguous grammars
- **Advanced Conflict Resolution** — Precedence, associativity, and custom strategies
- **Parse Table Generation** — Produces ActionCell-based parse tables
- **GSS (Graph-Structured Stack)** — Efficient shared-stack representation
- **Serialization** — Parse table serialization via postcard (optional)

## Features

| Feature | Description |
|---------|-------------|
| `parallel` | Enable parallel processing with rayon |
| `serialization` | Enable ParseTable serialization (postcard) |
| `glr-trace` | Enable debug tracing for GLR operations |
| `glr_telemetry` | Enable performance telemetry counters |
| `perf-counters` | Enable performance monitoring counters |
| `test-api` | Expose internal APIs for integration testing |
| `strict_docs` | Enforce documentation requirements |

## Architecture

```
Grammar (adze-ir) → FIRST/FOLLOW → LR(1) Items → Parse Tables → GLR Runtime
```

The parse tables use an `ActionCell` architecture where each state/symbol pair can have multiple valid actions, enabling the GLR parser to fork on conflicts.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
