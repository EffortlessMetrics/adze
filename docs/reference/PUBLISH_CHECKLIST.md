# Publish Checklist

How to publish the core Adze crates to crates.io.

## Publish Order

Crates **must** be published in dependency order. Each crate must be
available on the registry before its dependents can be packaged.

| Step | Crate | Directory | Key deps |
|------|-------|-----------|----------|
| 1 | `adze-common` | `common/` | `adze-common-syntax-core` |
| 2 | `adze-ir` | `ir/` | *(external only)* |
| 3 | `adze-glr-core` | `glr-core/` | `adze-ir` |
| 4 | `adze-tablegen` | `tablegen/` | `adze-ir`, `adze-glr-core`, `adze-bdd-grid-core`, `adze-parsetable-metadata` |
| 5 | `adze-macro` | `macro/` | `adze-common` |
| 6 | `adze-tool` | `tool/` | `adze-common`, `adze-ir`, `adze-glr-core`, `adze-tablegen` |
| 7 | `adze` | `runtime/` | `adze-macro`, `adze-ir`, `adze-glr-core`, `adze-tablegen`, microcrates |

### Prerequisite micro-crates

Several governance/infrastructure micro-crates must be published **before**
the core crates that depend on them. The full set (in order) is:

1. All `adze-concurrency-*` crates (caps-contract-core, map-core, env-core,
   init-core, caps-core, etc.)
2. `adze-linecol-core`
3. `adze-stack-pool-core`
4. `adze-ts-format-core`
5. `adze-common-syntax-core`
6. `adze-bdd-scenario-core` -> `adze-bdd-grid-core`
7. `adze-governance-metadata` -> `adze-parsetable-metadata`
8. `adze-runtime-governance` -> `adze-runtime-governance-api`

## Pre-publish verification

```bash
# Run the automated check (metadata + cargo package --list)
./scripts/check-publish.sh

# Full packaging test (requires all deps on crates.io already)
cargo package --allow-dirty -p <crate>
```

## Per-crate checklist

For each crate, before running `cargo publish`:

- [ ] Version bumped from `-dev` to release (e.g., `0.8.0`)
- [ ] All path dependencies also have their version bumped
- [ ] `cargo package -p <crate>` succeeds (no `--allow-dirty`)
- [ ] README.md is present and accurate
- [ ] LICENSE-MIT and LICENSE-APACHE are present
- [ ] `description` is meaningful (not a placeholder)
- [ ] `license = "Apache-2.0 OR MIT"` matches workspace
- [ ] `repository` points to the correct GitHub URL
- [ ] `publish = true` is set (workspace default is `publish = false`)
- [ ] `include` directive lists all needed files
- [ ] No secrets or large binaries in the package (`cargo package --list`)

## Publishing a release

```bash
# 1. Ensure clean working tree
git status  # should be clean

# 2. Update versions for the release you are cutting
#    For example: 0.8.0 -> 0.9.0, including Cargo.toml files and cross-references.

# 3. Run the publish check
./scripts/check-publish.sh

# 4. Commit the version bump
git commit -am "release: vX.Y.Z"
git tag vX.Y.Z

# 5. Publish in order (wait for each to appear on crates.io)
cargo publish -p adze-common
cargo publish -p adze-ir
cargo publish -p adze-glr-core
cargo publish -p adze-tablegen
cargo publish -p adze-macro
cargo publish -p adze-tool
cargo publish -p adze

# 6. Push tags
git push origin main --tags
```

## Troubleshooting

### "no matching package named X found"

This means a path dependency hasn't been published yet. Publish dependencies
first, in the order listed above.

### "failed to verify package tarball"

The crate's `include` directive may be too restrictive. Check that all
referenced source files are included with `cargo package --list -p <crate>`.

### Version mismatch

All inter-workspace path dependencies must have matching version strings.
For example, if `adze-ir` is `0.8.0`, then `adze-glr-core`'s dep on
`adze-ir` must also say `version = "0.8.0"`.
