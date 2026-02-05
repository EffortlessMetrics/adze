## 2026-02-05 - Vanilla JS Template Injection
**Vulnerability:** Widespread use of `innerHTML` with template literals in `playground/static/app.js` allowed for XSS via test names and parser outputs.
**Learning:** The lack of a frontend framework or templating engine led to unsafe manual DOM manipulation using strings.
**Prevention:** Strictly use `document.createElement`, `textContent`, and `appendChild` for all dynamic content in the playground. Avoid `innerHTML` entirely unless rendering trusted static HTML.
