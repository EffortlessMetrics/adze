# adze-concurrency-init-rayon-core

Idempotent Rayon global thread-pool initialization primitives.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_concurrency_init_rayon_core::init_rayon_global_once;

init_rayon_global_once(4).expect("rayon init failed");
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
