## 2026-01-26 - Vanilla JS Accessible Tabs
**Learning:** The playground uses vanilla JS without a framework, requiring manual management of ARIA states (`aria-selected`, `tabindex`) and keyboard event listeners for accessible components like tabs.
**Action:** When enhancing components in `playground/static`, always manually implement the WAI-ARIA authoring practices (roving tabindex) and ensure state updates are synced in the render/update methods.
