# adze-concurrency-bounded-map-core

Bounded parallel map implementation details.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_concurrency_bounded_map_core::bounded_parallel_map;

let input: Vec<i32> = (0..10).collect();
let mut result = bounded_parallel_map(input, 4, |x| x * 2);
result.sort();
assert_eq!(result, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
