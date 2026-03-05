# adze-node-pool-core

Thread-safe `Arc<T>` object pool utilities extracted from the Adze runtime.

This crate provides `NodePool<T>` plus basic usage statistics so parser workloads can
reuse frequently allocated nodes with lower allocator churn.
