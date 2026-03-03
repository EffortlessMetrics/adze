# adze-stack-pool-core

Thread-local and reusable stack pool utilities for parser workloads.

Part of the [adze](https://github.com/EffortlessMetrics/adze) workspace.

## Usage

```rust
use adze_stack_pool_core::StackPool;

let pool: StackPool<u32> = StackPool::new(4);
let mut stack = pool.acquire();
stack.push(42);
pool.release(stack);
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT License](../../LICENSE-MIT) at your option.
