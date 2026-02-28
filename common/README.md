# adze-common

Shared logic for the [Adze](https://github.com/EffortlessMetrics/adze) macro and tool crates.

## Overview

`adze-common` contains grammar expansion logic shared between the `adze-macro` proc-macro crate and the `adze-tool` build-time code generator. It processes Rust syntax definitions annotated with Adze attributes into grammar representations.

## Key Responsibilities

- **Attribute Parsing** — Interprets `#[adze::*]` attributes on Rust types
- **Grammar Extraction** — Converts annotated Rust types into grammar rules
- **Type Mapping** — Maps Rust types to grammar symbols (terminals, non-terminals)
- **Validation** — Checks grammar definitions for common errors

## Usage

This crate is an internal dependency and not intended for direct use. It is consumed by:
- `adze-macro` — For compile-time grammar processing
- `adze-tool` — For build-time grammar extraction

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
