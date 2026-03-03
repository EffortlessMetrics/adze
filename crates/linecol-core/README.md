# adze-linecol-core

Line/column byte-position tracking utilities for parser lexers and scanners.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_linecol_core::LineCol;

let lc = LineCol::at_position(b"hello\nworld", 8);
assert_eq!(lc.line, 1);
assert_eq!(lc.column(8), 2);
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
