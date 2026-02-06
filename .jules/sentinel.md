## 2025-11-20 - [DOM-based XSS in Playground]
**Vulnerability:** The playground frontend utilized `innerHTML` with template literals to render user-controlled data (e.g., test names via `prompt`). This exposed the application to Stored XSS attacks where a malicious test name could execute arbitrary JavaScript.
**Learning:** Relying on `innerHTML` for dynamic content updates is inherently risky, especially with user input. Even "internal" tools like playgrounds should treat all input as untrusted.
**Prevention:** Strictly use `document.createElement`, `textContent`, and `appendChild` (or `append`) for building DOM structures with dynamic data. Avoid `innerHTML` unless absolutely necessary and with sanitized HTML.
