# adze-javascript

JavaScript grammar definition for Adze.

## Overview

This crate provides a JavaScript grammar definition using Adze's macro-based
grammar specification. It generates a Tree-sitter-compatible parser for JavaScript
source code at build time.

## Features

- JavaScript grammar coverage
- Build-time parser generation via `adze-tool`
- Snapshot testing with `insta` for parse tree verification
- Compatible with both the standard Tree-sitter runtime and Adze's pure-Rust runtime

## Usage

```rust
use adze_javascript::language;

let lang = language();
// Use with adze's Parser API to parse JavaScript source code
```

## Testing

```bash
cargo test -p adze-javascript
```

## License

MIT OR Apache-2.0
