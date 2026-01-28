## 2025-11-20 - [Stored XSS in Playground]
**Vulnerability:** The playground frontend used `innerHTML` to render user-controlled test names and server-provided error messages, creating a Stored/DOM-based XSS vulnerability.
**Learning:** Even in "simple" or "static" frontends, directly injecting API responses or user input into `innerHTML` is a critical risk. Inline event handlers (`onclick="..."`) exacerbate this by executing code in string contexts.
**Prevention:** Strictly enforce the use of `textContent` for text data and `document.createElement` for structure. Avoid `innerHTML` entirely unless absolutely necessary and sanitized. Remove inline event handlers in favor of `addEventListener`.
