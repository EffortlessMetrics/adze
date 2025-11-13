# Installation

This chapter covers how to install and set up Rust-Sitter in your project.

## Prerequisites

- Rust 1.89 or later (with rustfmt and clippy components)
- Cargo (comes with Rust)

## Adding Dependencies

Add Rust-Sitter to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.8.0-dev"

[build-dependencies]
rust-sitter-tool = "0.8.0-dev"
```

## Choosing a Backend

Rust-Sitter offers three backend options via feature flags:

### Pure-Rust Backend (Recommended)

The pure-Rust backend generates static parsers at compile-time without C dependencies:

```toml
[dependencies]
rust-sitter = { version = "0.8.0-dev", features = ["pure-rust"] }
```

**Advantages:**
- No C dependencies
- Full WASM support
- Better compile-time optimization
- Easier cross-compilation

### C2Rust Backend

Legacy backend using transpiled C code:

```toml
[dependencies]
rust-sitter = { version = "0.8.0-dev", features = ["tree-sitter-c2rust"] }
```

### Standard Tree-sitter Backend

Uses the standard Tree-sitter C runtime:

```toml
[dependencies]
rust-sitter = { version = "0.8.0-dev", features = ["tree-sitter-standard"] }
```

## Build Configuration

Create a `build.rs` file in your project root:

```rust
use std::path::PathBuf;

fn main() {
    // Rebuild if source files change
    println!("cargo:rerun-if-changed=src");
    
    // Generate parsers from grammar definitions
    rust_sitter_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
```

## Optional Features

Additional features you can enable:

```toml
[dependencies]
rust-sitter = {
    version = "0.8.0-dev",
    features = [
        "pure-rust",      # Pure Rust backend
        "optimize",       # Enable grammar optimizer
        "parallel",       # Parallel parsing support
        "simd",          # SIMD-accelerated lexing
    ]
}
```

## Verifying Installation

Create a simple test file to verify your setup:

```rust
#[rust_sitter::grammar("test")]
mod grammar {
    #[rust_sitter::language]
    #[rust_sitter::leaf(text = "hello")]
    struct Hello;
}

fn main() {
    match grammar::parse("hello") {
        Ok(_) => println!("Rust-Sitter is working!"),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

Run with:

```bash
cargo build
cargo run
```

## Troubleshooting

### Common Issues

1. **Build fails with "cannot find macro `rust_sitter`"**
   - Ensure both `rust-sitter` and `rust-sitter-tool` are in your dependencies
   - Check that your `build.rs` is properly configured

2. **"Multiple applicable items in scope" errors**
   - This usually means you have conflicting features enabled
   - Choose only one backend feature

3. **WASM compilation fails**
   - Ensure you're using the `pure-rust` feature
   - The C-based backends don't support WASM

## Next Steps

Now that you have Rust-Sitter installed, proceed to the [Quick Start](quickstart.md) guide to create your first grammar!