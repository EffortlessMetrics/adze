# v0.6.1-beta Publish Checklist

## Pre-flight Checks
- [x] All core tests passing (100%)
- [x] Clippy clean
- [x] CHANGELOG updated
- [x] Regression guards in place
- [x] Release notes prepared

## Tag & Push
```bash
# Create annotated tag
git tag -a v0.6.1-beta -m "Release v0.6.1-beta: Algorithmically correct GLR parser"

# Push tag to origin
git push origin v0.6.1-beta
```

## Create GitHub Release
1. Go to https://github.com/EffortlessMetrics/rust-sitter/releases/new
2. Select tag: `v0.6.1-beta`
3. Title: `v0.6.1-beta - Algorithmically Correct GLR Parser`
4. Copy contents from `GITHUB_RELEASE.md` into description
5. Mark as pre-release (beta)
6. Publish release

## Publish to crates.io

**Important**: Follow dependency order to avoid publish failures

```bash
# 1. Core crates (no dependencies)
cargo publish -p rust-sitter-glr-core

# Wait 1-2 minutes for crates.io indexing

# 2. IR and common crates
cargo publish -p rust-sitter-ir
cargo publish -p rust-sitter-common

# Wait 1-2 minutes

# 3. Runtime and macro crates
cargo publish -p rust-sitter
cargo publish -p rust-sitter-macro

# Wait 1-2 minutes

# 4. Tool crate
cargo publish -p rust-sitter-tool

# 5. Optional: example crate (if publishing)
# cargo publish -p rust-sitter-example
```

## Post-Release Verification
```bash
# Verify crates are available
cargo search rust-sitter --limit 10

# Test installation in a new project
cd /tmp
cargo new test-rust-sitter
cd test-rust-sitter
echo 'rust-sitter = "0.6.1-beta"' >> Cargo.toml
cargo build
```

## Announce Release

### Quick announcement
```
rust-sitter v0.6.1-beta released! 🚀

✅ Algorithmically correct GLR parser
✅ 100% pass rate on core test suites
✅ 6 critical correctness fixes
✅ True fork/merge with multi-action cells
✅ Stable query results

Upgrade: rust-sitter = "0.6.1-beta"
Release notes: https://github.com/EffortlessMetrics/rust-sitter/releases/tag/v0.6.1-beta
```

### Channels to announce
- [ ] GitHub Discussions
- [ ] Discord/Slack (if applicable)
- [ ] Twitter/X (if applicable)
- [ ] Reddit r/rust (if major release)

## Future CI Improvements (non-blocking)
- [ ] Add ts-bridge parity testing (non-blocking)
- [ ] Add performance benchmarks with alerts
- [ ] Add safe-dedup threshold tuning
- [ ] Monitor regression guards in nightly builds