# adze-wasm-demo

WebAssembly demo of Adze parsing in the browser.

## Overview

This crate demonstrates Adze's pure-Rust parsing capabilities running in a web browser
via WebAssembly. It showcases real-time syntax parsing using Adze-generated grammars
compiled to WASM.

## Features

- Browser-based parsing demo
- Real-time syntax tree visualization
- Python and example grammar support
- Pure-Rust implementation (no C dependencies)

## Building

```bash
# Build WASM package
wasm-pack build --target web wasm-demo

# Or use cargo directly
cargo build -p adze-wasm-demo --target wasm32-unknown-unknown
```

## License

MIT OR Apache-2.0
