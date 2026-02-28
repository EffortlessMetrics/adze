# adze-concurrency-init-core

Rayon global thread-pool initialization policy for process-wide concurrency caps.

## Overview

Manages Rayon thread pool initialization with configurable caps. Ensures the global
thread pool is sized according to environment variables (`RAYON_NUM_THREADS`) and
system pressure, preventing resource exhaustion during parallel operations.

## License

MIT OR Apache-2.0
