## 2025-11-20 - Semantic Structure in Demos
**Learning:** Demos often use `<label>` for non-form elements (like result divs) to achieve a visual look, but this breaks semantics for screen readers.
**Action:** Replace misuse of `<label>` with headings (`h2`/`h3`) and `aria-live` regions for outputs, reusing CSS classes to maintain visual consistency.
