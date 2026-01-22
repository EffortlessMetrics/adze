## 2026-01-22 - [Status Feedback Accessibility]
**Learning:** Simple text updates to a status element (like `textContent = msg`) are invisible to screen readers unless `aria-live` is present. Adding `aria-live="polite"` makes these updates accessible "for free" without complex focus management.
**Action:** Always check status/notification containers for `aria-live` attributes.
