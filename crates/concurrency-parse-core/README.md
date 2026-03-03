# adze-concurrency-parse-core

String parsing helpers for concurrency cap configuration values.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_concurrency_parse_core::parse_positive_usize_or_default;

assert_eq!(parse_positive_usize_or_default(Some("16"), 8), 16);
assert_eq!(parse_positive_usize_or_default(None, 8), 8);
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
