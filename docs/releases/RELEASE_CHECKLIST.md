# Release Checklist for rust-sitter v0.5.0-beta

## Pre-release Steps

- [x] Update all crate versions to 0.5.0-beta
- [ ] Ensure all tests pass
- [ ] Update CHANGELOG.md with release notes
- [ ] Create GitHub release draft
- [ ] Verify documentation is up to date
- [ ] Test example crates with new version

## Publishing Order (dependencies first)

1. [ ] Publish `rust-sitter-ir`
2. [ ] Publish `rust-sitter-glr-core` 
3. [ ] Publish `rust-sitter-tablegen`
4. [ ] Publish `rust-sitter-common`
5. [ ] Publish `rust-sitter-macro`
6. [ ] Publish `rust-sitter-tool`
7. [ ] Publish `rust-sitter` (runtime)

## Post-release Steps

- [ ] Tag release in git: `git tag v0.5.0-beta`
- [ ] Push tag: `git push origin v0.5.0-beta`
- [ ] Publish GitHub release
- [ ] Announce on relevant channels
- [ ] Update compatibility dashboard

## Verification Commands

```bash
# Run all tests
cargo test --all

# Check that crates can be published
cargo publish --dry-run -p rust-sitter-ir
cargo publish --dry-run -p rust-sitter-glr-core
cargo publish --dry-run -p rust-sitter-tablegen
cargo publish --dry-run -p rust-sitter-common
cargo publish --dry-run -p rust-sitter-macro
cargo publish --dry-run -p rust-sitter-tool
cargo publish --dry-run -p rust-sitter

# Build documentation
cargo doc --all --no-deps
```