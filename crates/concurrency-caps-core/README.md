# adze-concurrency-caps-core

Concurrency cap primitives for bounded parallel execution.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_concurrency_caps_core::{ConcurrencyCaps, parse_positive_usize_or_default};

let caps = ConcurrencyCaps::from_lookup(|_| None);
assert_eq!(caps.rayon_threads, 4); // default
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
