## 2025-11-21 - [Permissive CORS in Playground]
**Vulnerability:** The playground web server was configured with `CorsLayer::permissive()`, which allowed any origin to access the API. This is a potential security risk, even for a local playground, as malicious websites could interact with the local server.
**Learning:** Developers often disable CORS during development for convenience ("it's just a playground"), but this creates bad habits and potential vulnerabilities if the code is deployed or exposed.
**Prevention:** Always use strict CORS policies or none at all if the frontend and backend share the same origin. Only enable specific origins if cross-origin access is explicitly required.
