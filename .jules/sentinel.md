# Sentinel's Security Journal

## 2025-11-20 - [DOM XSS in Playground]
**Vulnerability:** DOM-based Cross-Site Scripting (XSS) in `playground/static/app.js` due to unsafe usage of `innerHTML` when rendering user-controlled test case names.
**Learning:** Vanilla JavaScript applications without a framework (like React/Vue) often default to `innerHTML` for convenience, leading to XSS. Input from "Import" features is just as dangerous as network input.
**Prevention:** Strictly use `document.createElement`, `textContent`, and `appendChild` for dynamic content. Avoid `innerHTML` unless absolutely necessary and sanitized.
