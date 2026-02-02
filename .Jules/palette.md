## 2024-05-21 - [Vanilla JS Tab Accessibility]
**Learning:** The playground uses a custom vanilla JS tab implementation without semantic markup or a framework, making accessibility retrofit manual and verbose (managing aria-selected/tabindex in JS).
**Action:** When touching other custom interactive components in this repo (like the button groups), verify if they need similar manual ARIA state management since there's no component library to handle it for us.
