# Release Checklist for adze v0.5.0-beta

## Pre-release Steps

- [x] Update all crate versions to 0.5.0-beta
- [ ] Ensure all tests pass
- [ ] Update CHANGELOG.md with release notes
- [ ] Create GitHub release draft
- [ ] Verify documentation is up to date
- [ ] Test example crates with new version

## Publishing Order (dependencies first)

1. [ ] Publish `adze-ir`
2. [ ] Publish `adze-glr-core` 
3. [ ] Publish `adze-tablegen`
4. [ ] Publish `adze-common`
5. [ ] Publish `adze-macro`
6. [ ] Publish `adze-tool`
7. [ ] Publish `adze` (runtime)

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
cargo publish --dry-run -p adze-ir
cargo publish --dry-run -p adze-glr-core
cargo publish --dry-run -p adze-tablegen
cargo publish --dry-run -p adze-common
cargo publish --dry-run -p adze-macro
cargo publish --dry-run -p adze-tool
cargo publish --dry-run -p adze

# Build documentation
cargo doc --all --no-deps
```