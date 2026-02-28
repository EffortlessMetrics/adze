# adze-go

Go grammar definition for Adze.

## Overview

This crate provides a Go grammar definition using Adze's macro-based grammar
specification. It generates a Tree-sitter-compatible parser for Go source code
at build time.

## Features

- Go grammar coverage
- Build-time parser generation via `adze-tool`
- Snapshot testing with `insta` for parse tree verification
- Compatible with both the standard Tree-sitter runtime and Adze's pure-Rust runtime

## Usage

```rust
use adze_go::language;

let lang = language();
// Use with adze's Parser API to parse Go source code
```

## Testing

```bash
cargo test -p adze-go
```

## License

MIT OR Apache-2.0
