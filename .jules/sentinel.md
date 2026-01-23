## 2025-11-20 - DOM-based XSS in Playground
**Vulnerability:** Unsanitized user input (`test.name`) was inserted into the DOM using `innerHTML` in `playground/static/app.js`.
**Learning:** The playground frontend uses vanilla JavaScript without a framework that automatically escapes content, making it prone to DOM-based XSS.
**Prevention:** Use `textContent` and `document.createElement` for user-controlled data, or use a sanitizer library if HTML rendering is required.
