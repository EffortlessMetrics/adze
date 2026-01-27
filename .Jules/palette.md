## 2025-05-23 - Tab Component Accessibility
**Learning:** The tab system in `playground` was implemented using simple `<button>` elements with CSS classes, lacking standard WAI-ARIA roles (`tablist`, `tab`, `tabpanel`) and keyboard navigation support, making it inaccessible to screen readers and keyboard users.
**Action:** When working on UI components in this repo, always verify that interactive elements like tabs implement the full WAI-ARIA pattern (roles + keyboard support), not just visual styles.
