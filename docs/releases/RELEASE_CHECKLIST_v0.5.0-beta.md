# Adze v0.5.0-beta Release Checklist

## Pre-Release Validation ✓

### Build Status (Updated 2025-08-02)
- [x] Core crates compile successfully
  - [x] adze (runtime) - with GLR parser implementation
  - [x] adze-macro
  - [x] adze-tool
  - [x] adze-common
  - [x] adze-ir
  - [x] adze-glr-core - new GLR core
  - [x] adze-tablegen
  - [x] adze-cli
- [x] Example crate compiles successfully
- [~] Grammar crates have empty rule issues
  - [~] adze-javascript (EmptyString error)
  - [~] adze-go (EmptyString error)
  - [~] adze-python (EmptyString error)

### Test Status
- [x] All core tests compile successfully
- [x] GLR parser tests functional
- [x] Error recovery tests working
- [x] Benchmark suite operational
- [~] Some tests need API updates
- [~] Grammar crates blocked by empty rule issue

### Documentation (Updated)
- [x] README.md updated with v0.5.0-beta status
- [x] CHANGELOG.md created with comprehensive changes
- [x] Migration guide included in CHANGELOG
- [x] GLR visualization guide created
- [x] Stabilization summary documented
- [x] Release checklist updated

## Release Package Contents

### Core Crates (to publish)
1. adze-common v0.5.0-beta
2. adze-ir v0.5.0-beta
3. adze-macro v0.5.0-beta
4. adze v0.5.0-beta
5. adze-tool v0.5.0-beta
6. adze-glr-core v0.5.0-beta
7. adze-tablegen v0.5.0-beta
8. adze-cli v0.5.0-beta

### Example Crates (not published)
- adze-example
- adze-javascript
- adze-go

## Known Issues (Documented)

### Critical Issues
1. **Empty Production Rules**: Vec<T> fields cause EmptyString errors
   - Blocks Python, JavaScript, Go grammars
   - Workaround: Use Option<T> fields
   - Fix needed in tree-sitter-generate crate

### Architecture Changes
1. GLR parser uses two-phase algorithm
2. New API for GLRParser and GLRLexer
3. Enhanced error recovery configuration
4. Some tests need API migration

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
cargo build --workspace --exclude adze-python --exclude adze-playground

# Run tests (expect some failures)
cargo test --workspace --exclude adze-python --exclude adze-playground

# Check examples work
cargo run -p adze-example
```

## Support Channels

- GitHub Issues: Report bugs and feature requests
- Discussions: Questions and community support
- Documentation: See docs/ directory

---

**Ready for Beta Release: YES** ✅

The core functionality works, limitations are documented, and the foundation is solid for future development.