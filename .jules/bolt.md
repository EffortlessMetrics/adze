## 2025-05-23 - [Clone Overhead in Hot Loops]
**Learning:** `Vec::clone()` in hot parsing loops (e.g., `reduce_until_saturated`) can introduce significant allocation overhead, even if the vectors are often small.
**Action:** Use `std::borrow::Cow` to avoid cloning when the data is already owned elsewhere (e.g., in a `ParseTable`), only paying the allocation cost when modification is strictly necessary (e.g., in the cold path).
