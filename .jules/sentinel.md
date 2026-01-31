## 2026-01-31 - DOM XSS in Vanilla JS
**Vulnerability:** Widespread use of `innerHTML` to render user-controlled data (test names, error messages) in `playground/static/app.js`, leading to stored and reflected XSS.
**Learning:** Vanilla JS apps without frameworks often default to `innerHTML` for convenience, missing the auto-escaping protections provided by modern frameworks like React or Vue.
**Prevention:** Strictly prohibit `innerHTML` for user content. Use `textContent` or `innerText` for text, and `document.createElement` for structure. Use a helper function to make DOM creation less verbose.
