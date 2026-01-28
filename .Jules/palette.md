## 2025-11-20 - Custom Tab Implementations Lack ARIA
**Learning:** The manual tab implementation in `playground/static` completely lacked ARIA roles (`tablist`, `tab`, `tabpanel`) and states (`aria-selected`), making it inaccessible to screen readers.
**Action:** When creating custom tab components, strictly follow the WAI-ARIA Tabs design pattern, ensuring dynamic updates to `aria-selected`.
