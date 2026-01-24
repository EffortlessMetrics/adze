## 2024-05-22 - [Button-based Tabs Accessibility]
**Learning:** Standard `<button>` elements used for tabs provide keyboard accessibility (tab order) by default but lack semantic meaning for screen readers, leading to a confusing experience where users don't know they are navigating a tab set.
**Action:** When identifying tab-like interfaces implemented with buttons, prioritize adding `role="tablist"`, `role="tab"`, and dynamic `aria-selected` attributes before attempting complex keyboard handler refactoring (roving tabindex), as this provides high value with minimal risk.
