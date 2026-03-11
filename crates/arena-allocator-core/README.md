# adze-arena-allocator-core

Typed arena allocator primitives for parse tree node storage and reuse.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_arena_allocator_core::{TreeArena, TreeNode};

let mut arena = TreeArena::new();
let leaf = arena.alloc(TreeNode::leaf(42));
assert_eq!(arena.get(leaf).symbol(), 42);
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
