## 2025-11-20 - GLR Parser Stack Reuse
**Learning:** In hot path loops processing owned items (like `Vec<ParseStack>`), if an item is cloned for multiple branches (e.g. ambiguity forks), the last branch can often reuse the original allocation instead of cloning.
**Action:** Use `Option<T>` to wrap the item and `.take()` it for the last iteration/branch to transfer ownership, while using `.as_ref()` for prior read-only access or cloning.
