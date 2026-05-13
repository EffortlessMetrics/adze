# 0.9.0 Release Checklist

**Status:** in progress
**Owner:** Adze maintainers
**Created:** 2026-05-13

## Spec and documentation

- [x] API foundation proposal (ADZE-PROP-0002)
- [x] Canonical parse document spec (ADZE-SPEC-0003)
- [x] Typed CST and AST projection spec (ADZE-SPEC-0004)
- [x] Diagnostics and recovery spec (ADZE-SPEC-0005)
- [x] Tree-sitter compatibility adapter spec (ADZE-SPEC-0006)
- [x] GLR ambiguity summary spec (ADZE-SPEC-0007)
- [x] JSON CLI WASM projection spec (ADZE-SPEC-0008)
- [x] Incremental document lifecycle spec (ADZE-SPEC-0009, proposed)
- [x] Language metadata and node-types spec (ADZE-SPEC-0010)
- [x] Document artifact ledger (policy/doc-artifacts.toml)
- [x] Product proof map (docs/status/PRODUCT_PROOF_MAP.md)
- [x] Artifact templates (proposals, specs, ADRs, plans, goals, handoffs)
- [ ] Agent implementation contract reviewed

## CI and policy

- [x] CI economics docs reconciled (#690)
- [x] Package boundary audit (#694)
- [x] CI economics verifier (#695)
- [ ] Doc artifact checker in xtask
- [ ] Active goal checker in xtask
- [ ] CI policy lane runs artifact checks

## Proof PRs

- [x] EOF missing recovery child (#680)
- [x] Multibyte parse diagnostic (#678)
- [x] Anonymous alias named-child filtering (#679)
- [x] Advisory node_types_json projection (#677)

## Workspace

- [x] Parser feature facade crate retired (#697)
- [x] Inline GLR versioning crate retired (#696)
- [ ] Microcrate collapse (in progress)

## Release readiness

- [ ] `just ci-supported` green on main
- [ ] All spec files present and parseable
- [ ] README claims mapped to PRODUCT_PROOF_MAP
- [ ] SUPPORT_TIERS updated
- [ ] KNOWN_RED updated
- [ ] CHANGELOG updated
- [ ] Versions consistent across Cargo.toml files
- [ ] Package dry run: `cargo package -p adze --allow-dirty`
- [ ] Package dry run: `cargo package -p adze-tool --allow-dirty`
- [ ] Known gaps documented in KNOWN_GAPS_0_9.md

## Known gaps for 0.9

- AdzeDocument ABI is not stable
- Typed CST API is not stable
- GLR forest/raw ambiguity API is not exposed
- Full Tree-sitter compatibility is not claimed
- Query parity is not promised
- Incremental parsing falls back to fresh parsing
- Full error-tree parity beyond EOF is not implemented
- node_types_json() is advisory

## Version bump

- [ ] Update version in root Cargo.toml
- [ ] Update version in all publishable crate Cargo.toml files
- [ ] Run `cargo check --workspace`
- [ ] Run `just ci-supported`
- [ ] Tag release
