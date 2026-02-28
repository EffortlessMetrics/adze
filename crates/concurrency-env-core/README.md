# adze-concurrency-env-core

Environment-backed concurrency cap policy and parsing utilities.

## Overview

Reads concurrency limits from environment variables and provides a normalized API
for querying caps. Supports `RUST_TEST_THREADS`, `RAYON_NUM_THREADS`,
`TOKIO_WORKER_THREADS`, and other concurrency-related settings.

## License

MIT OR Apache-2.0
