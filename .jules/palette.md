## 2025-11-20 - Vanilla JS Tab Accessibility
**Learning:** Implementing WAI-ARIA tabs in vanilla JS requires explicit management of `tabindex`, `aria-selected`, and keyboard focus (roving tabindex), which is often overlooked when not using a component library.
**Action:** For any future vanilla JS interactive components, verify keyboard navigation logic manually or with Playwright, as standard event bubbling may require specific listener attachment strategies.
