# xtask

Build automation tasks for the Adze workspace.

## Overview

This crate implements the [cargo xtask](https://github.com/matklad/cargo-xtask) pattern
for workspace-level automation. It provides commands for code generation, validation,
and release management.

## Usage

```bash
# Run via cargo xtask
cargo xtask <command>

# Common commands
cargo xtask codegen    # Generate code from grammar definitions
cargo xtask validate   # Validate workspace consistency
cargo xtask release    # Prepare release artifacts
```

## License

MIT OR Apache-2.0
