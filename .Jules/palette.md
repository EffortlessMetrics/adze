## 2025-11-20 - Missing ARIA Roles in Interactive Components
**Learning:** The custom tab implementation in the playground relies solely on CSS classes (`active`) and lacks semantic HTML roles (`tablist`, `tab`, `tabpanel`) and states (`aria-selected`), making it completely inaccessible to screen reader users and confusing for keyboard users.
**Action:** Always implement the WAI-ARIA Authoring Practices (APG) patterns for interactive components like tabs, ensuring roles, properties, and keyboard interactions are strictly followed.
