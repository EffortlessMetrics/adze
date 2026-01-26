## 2026-01-26 - [DOM-based XSS in Vanilla JS]
**Vulnerability:** `innerHTML` was used to render user-controlled data (test names, error messages) in the Playground, allowing XSS. Inline event handlers (`onclick`) were also vulnerable to injection.
**Learning:** In vanilla JS projects without frameworks (which often auto-escape), manual DOM manipulation is significantly safer than string concatenation into `innerHTML`. Inline handlers are particularly dangerous when combined with string interpolation.
**Prevention:** Strictly use `textContent` for text, `createElement` for structure, and `addEventListener` for events. Avoid `innerHTML` unless absolutely necessary and properly sanitized.
