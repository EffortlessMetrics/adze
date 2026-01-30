## 2026-01-30 - Accessible Tabs Implementation
**Learning:** Adding `role="tablist"`, `role="tab"`, and keyboard navigation significantly improves accessibility for tabs without changing visual styles. Explicitly setting `tabindex="0"` on the active tab in HTML is crucial for initial state verification and consistency.
**Action:** Always verify ARIA attributes with both static checks and interactive tests (like keyboard navigation).
