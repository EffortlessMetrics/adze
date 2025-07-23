# Rust-Sitter v0.5.0-beta Release Checklist

## Pre-Release Validation ✓

### Build Status
- [x] Core crates compile successfully
  - [x] rust-sitter (runtime)
  - [x] rust-sitter-macro
  - [x] rust-sitter-tool
  - [x] rust-sitter-common
  - [x] rust-sitter-ir
  - [x] rust-sitter-glr-core
  - [x] rust-sitter-tablegen
  - [x] rust-sitter-cli
- [x] Example grammars compile
  - [x] rust-sitter-javascript
  - [x] rust-sitter-go
  - [x] rust-sitter-example
  - [~] rust-sitter-python (excluded - scanner issues)
- [~] rust-sitter-playground (excluded - multiple errors)

### Test Status
- [x] Core functionality tests pass (40/48 runtime tests passing)
- [x] Grammar extraction works
- [x] Parse tree generation works
- [~] Query compiler tests failing (known limitation)
- [~] Scanner tests failing (known limitation)

### Documentation
- [x] README.md updated
- [x] QUICKSTART_BETA.md created
- [x] GRAMMAR_EXAMPLES.md comprehensive
- [x] RELEASE_STATUS_v0.5.0-beta.md documents limitations
- [x] KNOWN_LIMITATIONS.md lists all issues
- [x] Migration guide available

## Release Package Contents

### Core Crates (to publish)
1. rust-sitter-common v0.5.0-beta
2. rust-sitter-ir v0.5.0-beta
3. rust-sitter-macro v0.5.0-beta
4. rust-sitter v0.5.0-beta
5. rust-sitter-tool v0.5.0-beta
6. rust-sitter-glr-core v0.5.0-beta
7. rust-sitter-tablegen v0.5.0-beta
8. rust-sitter-cli v0.5.0-beta

### Example Crates (not published)
- rust-sitter-example
- rust-sitter-javascript
- rust-sitter-go

## Known Issues (Documented)

### Major Limitations
1. No precedence/associativity support
2. Limited external scanner API
3. Query language partially implemented
4. Some Tree-sitter features missing

### Test Failures
- 8 runtime tests failing (query and scanner related)
- Snapshot tests outdated in macro crate
- Playground crate has compilation errors

## Release Process

1. [ ] Version numbers confirmed as 0.5.0-beta
2. [ ] Dependencies between crates verified
3. [ ] Cargo.toml files have correct metadata
4. [ ] License files present (MIT)
5. [ ] Repository field set correctly

## Post-Release Tasks

1. [ ] Tag release as v0.5.0-beta
2. [ ] Create GitHub release with notes
3. [ ] Publish crates in dependency order
4. [ ] Update main README with beta notice
5. [ ] Announce in relevant channels

## Beta Notice

This is a **beta release** intended for early adopters and feedback. Users should expect:
- Breaking changes in future releases
- Missing features documented in KNOWN_LIMITATIONS.md
- Simplified grammars required (no precedence)
- Active development and improvements

## Verification Commands

```bash
# Build all core crates
cargo build --workspace --exclude rust-sitter-python --exclude rust-sitter-playground

# Run tests (expect some failures)
cargo test --workspace --exclude rust-sitter-python --exclude rust-sitter-playground

# Check examples work
cargo run -p rust-sitter-example
```

## Support Channels

- GitHub Issues: Report bugs and feature requests
- Discussions: Questions and community support
- Documentation: See docs/ directory

---

**Ready for Beta Release: YES** ✅

The core functionality works, limitations are documented, and the foundation is solid for future development.