## 2026-01-21 - Accessible Tabs Pattern
**Learning:** Static HTML tabs often use class-based toggling (`active`) which is invisible to screen readers, creating a major accessibility gap in otherwise functional UIs.
**Action:** Always pair `classList.toggle('active')` with `setAttribute('aria-selected', isSelected)` when implementing custom tab interfaces.
