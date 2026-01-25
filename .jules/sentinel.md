## 2026-01-25 - DOM-based XSS in Playground
**Vulnerability:** `innerHTML` was used to render user-supplied test names in the playground app, allowing execution of arbitrary JavaScript via crafted test names.
**Learning:** Even in internal tools or playgrounds, treating user input as HTML by default (using `innerHTML`) creates XSS vectors. The simplicity of template literals often leads to this insecurity.
**Prevention:** Strictly enforce the usage of `textContent` or DOM creation methods (`createElement`) for any content that includes variables.
