# Release Process

Release orchestration is now handled by [`.github/workflows/release.yml`](../../../.github/workflows/release.yml).

Use workflow dispatch with these inputs to perform the full, canonical path:
- validation
- full test suite
- artifact builds
- version/tags updates
- publish
- GitHub release creation

Workflow inputs:
- `version` (required): release version (e.g. `0.10.0`).
- `release_surface_mode` (default: `fixed`): `fixed` (use allowlist order) or `auto` (compute publishable crates).
- `release_crate_file` (optional): custom allowlist path used with fixed mode or sync regeneration.
- `dry_run` (default: `true`): run all validation and checks without publishing.
- `strict_publish_surface` (default: `false`): in `fixed` mode, fail release if extra publishable crates are not in the release allowlist.

Legacy local helpers are kept for ad-hoc use only:
- [`scripts/update-versions.sh`](../../../scripts/update-versions.sh)
- [`scripts/release.sh`](../../../scripts/release.sh)
- [`scripts/dry-run-publish.sh`](../../../scripts/dry-run-publish.sh)

For local helper runs, set:
- `RELEASE_SURFACE_MODE=fixed|auto`
- `RELEASE_CRATE_FILE=<path>`
- `STRICT_PUBLISH_SURFACE=true|false` (fixed mode only; default `false`)
- `RELEASE_CRATE_SYNC=true` (only meaningful in `auto` mode)

`release.toml` is still used for changelog/version replacement metadata.

`release.sh` and `dry-run-publish.sh` now support two release-surface modes:
- `RELEASE_SURFACE_MODE=fixed` (default): use `scripts/release-crates.txt` as a strict allowlist and publish order.
- `RELEASE_SURFACE_MODE=auto`: compute publishable workspace crates and auto-resolve dependency order.
- Optional override: `RELEASE_CRATE_FILE` to point at an alternate allowlist file (defaults to `scripts/release-crates.txt`).
- `RELEASE_CRATE_SYNC=true` with `RELEASE_SURFACE_MODE=auto`: regenerate the selected allowlist file from metadata.
