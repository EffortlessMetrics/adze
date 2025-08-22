# v0.6.1-beta Publish Checklist

## Pre-flight Checks
- [x] All core tests passing (100%)
- [x] Clippy clean
- [x] CHANGELOG updated
- [x] Regression guards in place
- [x] Release notes prepared

## Tag & Push
```bash
# Create and push tag
git tag -a v0.6.1-beta -m "Release v0.6.1-beta: Algorithmically correct GLR parser"
git push origin v0.6.1-beta
```

## Publish to crates.io
```bash
# Important: Publish in dependency order!
# Wait ~30 seconds between publishes for crates.io indexing

# 1. Core dependencies
cargo publish -p rust-sitter-ir
cargo publish -p rust-sitter-glr-core

# 2. Common utilities
cargo publish -p rust-sitter-common

# 3. Main runtime
cargo publish -p rust-sitter

# 4. Build tools
cargo publish -p rust-sitter-macro
cargo publish -p rust-sitter-tool

# 5. Optional: Examples
# cargo publish -p rust-sitter-example
```

## GitHub Release
1. Go to: https://github.com/hydro-project/rust-sitter/releases/new
2. Select tag: `v0.6.1-beta`
3. Title: `v0.6.1-beta - Algorithmically Correct GLR Parser`
4. Paste contents of `GITHUB_RELEASE.md`
5. Check "Set as a pre-release"
6. Publish

## Post-Release Tasks
- [ ] CI: Add non-blocking ts-bridge parity job
- [ ] CI: Add criterion benchmark for perf tracking
- [ ] Create tracking issues for:
  - [ ] Query predicates implementation
  - [ ] Incremental GLR equivalence suite
  - [ ] CLI runtime loading
  - [ ] External scanner linking docs
- [ ] Update README with beta banner
- [ ] Announce on relevant channels

## Verification
```bash
# Verify tag exists
git tag -l v0.6.1-beta

# Verify crates published (wait 5 minutes)
cargo search rust-sitter --limit 5

# Test installation
cd /tmp && cargo new test-install && cd test-install
echo 'rust-sitter = "0.6.1-beta"' >> Cargo.toml
cargo build
```

## Rollback (if needed)
```bash
# Delete remote tag
git push --delete origin v0.6.1-beta

# Delete local tag
git tag -d v0.6.1-beta

# Yank from crates.io (irreversible!)
# cargo yank --version 0.6.1-beta rust-sitter
```