## 2025-05-15 - Optimizing Hot Loops with Ownership Transfer
**Learning:** In GLR parsing, the hot loop processes actions for a token. Often, only one action exists (deterministic case). By using `slice::split_last()`, we can separate the last action processing from the rest. This allows passing the heavy `ParseStack` by value (move) to the last action, avoiding an expensive `clone()` and allocation, while still cloning for any prior actions (ambiguity).
**Action:** When optimizing loops where heavy objects are cloned for each iteration but consumed, refactor to treat the last iteration specially to reuse the original object. Extract the loop body into a helper method to maintain clean code.

## 2025-05-15 - Managing Borrows in extracted helper methods
**Learning:** Extracting logic into a helper method `&mut self` can trigger borrow checker errors if the call site is iterating over a reference to a field of `self` (like `self.table`).
**Action:** To resolve this, clone the necessary data from the borrowed field (e.g., the specific `Action` or `Vec<Action>`) to drop the immutable borrow before calling the helper method that requires `&mut self`.
