# adze-concurrency-caps-core

Core concurrency capping primitives for Adze.

## Overview

This microcrate provides thread pool size limits and bounded parallel execution
utilities. It ensures stable behavior under resource pressure by capping Rayon,
Tokio, and Rust test thread counts.

## Usage

```rust
use adze_concurrency_caps_core::init_concurrency_caps;

// Set up capped thread pools based on environment variables
init_concurrency_caps();
```

## Environment Variables

- `RUST_TEST_THREADS` — test thread cap (default: 2)
- `RAYON_NUM_THREADS` — Rayon pool cap (default: 4)
- `TOKIO_WORKER_THREADS` — Tokio worker cap (default: 2)

## License

MIT OR Apache-2.0
