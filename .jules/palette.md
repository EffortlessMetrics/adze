## 2025-11-20 - [Accessible Tabs Implementation]
**Learning:** Converting static button-based tabs to WAI-ARIA compliant tabs with keyboard navigation is a high-value micro-UX improvement. It requires syncing `aria-selected` and `tabindex` attributes in JavaScript, which is often overlooked but critical for keyboard users.
**Action:** Standardize on the WAI-ARIA Tab pattern (roles + arrow key navigation) for all tab-like interfaces found in legacy frontend code.
