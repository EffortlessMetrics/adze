## 2025-05-20 - [DOM-based XSS in Playground]
**Vulnerability:** The playground frontend utilized `innerHTML` to render user-controlled data (test names, error messages) without sanitization.
**Learning:** Even in "static" or "playground" apps, `innerHTML` poses a severe risk, especially when data can be imported/exported (Stored XSS vector).
**Prevention:** Strictly use `textContent` for text and `document.createElement()` for structure. Avoid `innerHTML` unless absolutely necessary and with sanitization.
