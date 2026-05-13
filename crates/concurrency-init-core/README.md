# adze-concurrency-init-core

Rayon global thread-pool initialization policy for process-wide concurrency caps.
Also owns bootstrap-time cap normalization before process-wide Rayon initialization.

## Usage

```rust
use adze_concurrency_init_core::{ConcurrencyCaps, bootstrap_caps, init_concurrency_caps_with_caps};

let caps = bootstrap_caps(ConcurrencyCaps {
    rayon_threads: 0,
    tokio_worker_threads: 2,
});

init_concurrency_caps_with_caps(caps);
```

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
