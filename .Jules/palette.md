## 2026-01-23 - Static App Accessibility Gap
**Learning:** The `playground` component uses vanilla HTML/JS without any component abstractions or build-time a11y checks, resulting in interactive elements (like tabs) lacking basic ARIA roles and keyboard support.
**Action:** When modifying the `playground` or `wasm-demo` frontends, manually enforce semantic HTML and ARIA attributes as the first step, as no framework will handle this automatically.
