# Contributing to Rust Sitter

First off, thank you for considering contributing to Rust Sitter! It's people like you that make Rust Sitter such a great tool.

This document provides guidance for developers who want to contribute to the project.

## Code of Conduct

This project and everyone participating in it is governed by the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you are expected to uphold this code.

## How to Contribute

We welcome contributions of all kinds, including bug reports, feature requests, documentation improvements, and code contributions.

If you are new to the project, a good place to start is to look for issues tagged with `good first issue`.

## Development Environment Setup

To get started with the development of Rust Sitter, you will need to have a recent version of Rust and Cargo installed. You can find instructions on how to install Rust at [rust-lang.org](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can clone the repository and build the project:

```bash
git clone https://github.com/hydro-project/rust-sitter.git
cd rust-sitter
cargo build
```

## Common Development Commands

This project is a Rust workspace. Here are some common commands you will use during development:

### Building

```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Build a specific package
cargo build -p rust-sitter
```

### Testing

```bash
# Run all tests in the workspace
cargo test

# Run tests for a specific package
cargo test -p rust-sitter

# Update snapshot tests (uses insta)
cargo insta review
```

### Linting and Formatting

```bash
# Run clippy on all workspace members
cargo clippy --all

# Format code
cargo fmt
```

## Project Structure

Rust Sitter is a Rust workspace consisting of multiple interconnected crates. You can find a detailed overview of the architecture in the [design documents](./docs/design).

## Other Scripts

You may notice a number of shell scripts (`.sh`) in the repository. These are used for various testing and build automation tasks. For the most part, you should be able to use the `cargo` commands listed above for all your development needs. If you find yourself needing to use one of the scripts, please feel free to ask for guidance in an issue.

## Submitting a Pull Request

When you are ready to submit a pull request, please make sure you have done the following:

1.  Run `cargo fmt` to format your code.
2.  Run `cargo clippy --all` and address any warnings.
3.  Run `cargo test` to make sure all tests pass.
4.  Add a descriptive title and summary to your pull request.

Thank you for your contribution!
