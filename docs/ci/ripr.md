# ripr advisory

`ripr` is a static RIPR exposure analyzer. It sits between line coverage
(too coarse) and runtime mutation testing (too expensive) and asks a
narrower question for each behavioural delta:

> Is this changed code path exposed to a meaningful test discriminator?

It does **not** run mutants and does **not** report killed/survived
outcomes. Reading or writing ripr policy without that distinction will
produce garbled mental models — see `docs/ci/cost-and-verification-policy.md`.

## Where it sits in the ladder

`ripr` runs at Tier 1 (frontdoor advisory). It runs on every Rust-touching
PR, never on docs-only PRs, never on `merge_group`, and is configured with
`continue-on-error: true` so that failures are visible but never block.

## MSRV and provisioning

Adze pins `rust-toolchain.toml` to `1.92.0`. `ripr` requires `1.93+`. We
do not bump MSRV for this advisory check. Provisioning options, in order
of preference:

1. **Pinned prebuilt binary** — drop a `ripr` binary on the runner via a
   release asset URL or self-hosted artifact mirror.
2. **Isolated toolchain** — install a `1.93` toolchain only for the ripr
   step using `dtolnay/rust-toolchain@stable` with a different `with:
   toolchain:` value, then `cargo install --locked --root /opt/ripr ripr`.
3. **MSRV bump** — once the workspace MSRV moves past `1.93`, install
   normally via `cargo install --locked ripr`.

Until one of those is wired in, the workflow detects the absence of `ripr`
and emits a stub `ripr-report.json` with `"status": "skipped"`. The
advisory step is therefore never an obstacle to landing PRs.

## Configuration

| File | Purpose |
| --- | --- |
| `ripr.toml` | analysis mode, severities, suppression ledger pointer |
| `policy/ripr-suppressions.toml` | per-path suppressions with owner/expiry |

Severities are pinned to `notice` / `warning`. There is no `error`
severity in the adze configuration; ripr is advisory by policy and by
config, not just by workflow flag.

## Suppressions

Each suppression must declare `path-glob`, `finding`, `owner`, `reason`,
and `expires`. Unsuppressed `weakly_exposed` findings are a reviewer
prompt, not a build break.

## When to take a finding seriously

| Finding | Take seriously when |
| --- | --- |
| `exposed` | rarely — the test surface looks fine |
| `weakly_exposed` | new behavior, parser/runtime, or hot path |
| `reachable_unrevealed` | always for parser/glr-core/tablegen changes |
| `no_static_path` | almost never — it usually reflects analyzer limits |
| `*_unknown` | only if surrounding tests are also new |

## Rollback

Removing `.github/workflows/ripr.yml` removes the lane. Suppressions and
config TOML are inert without the workflow.
