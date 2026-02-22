# Roadmap

**Published:** 0.6.x (beta)
**Dev head:** 0.8.0-dev (workspace)
**MSRV:** 1.92 (Rust 2024 edition)

This file is the durable plan: outcomes and boundaries.

For rolling execution, see:
- `docs/status/NOW_NEXT_LATER.md`
- `docs/status/FRICTION_LOG.md`

---

## What Adze is (one paragraph)

Adze is a Rust-native grammar toolchain that turns Rust type definitions into parse machinery (IR + tables) and returns typed Rust values at runtime. The goal is a compilation pipeline that makes parsing a build artifact, with interoperability where it helps.

---

## Near-term milestone: publishable baseline (0.8.x)

**User-facing outcomes**
- One recommended "happy path" for:
  - defining grammars
  - generating tables in `build.rs`
  - parsing + typed extraction at runtime
- Docs that match dev head (features, flags, examples)
- A clear "supported vs experimental" contract

**Engineering outcomes**
- A single merge-blocking CI contract (computed, stable)
- Benchmarks are reproducible and published as a baseline
- Supported-lane exclusions are explicit (`docs/status/KNOWN_RED.md`)

**Shipping outcomes**
- Decide which crates publish and which remain internal
- `cargo package` clean for publishable crates
- Version/feature consistency across publishable set

---

## Next milestone: ecosystem hardening (0.9.x)

Focus: reduce integration cost and make contributions cheap.

- CLI becomes useful for real workflows (validate, inspect, debug)
- Golden tests become a maintained contract (not just a demo)
- Grammar crates can be consumed downstream with minimal ceremony
- LSP generator and playground move from "prototype" to "useful for one or two grammars"

---

## Stability milestone: 1.0

Focus: a stability contract you can keep.

- Public API stability guarantees + deprecation policy
- Clear boundaries:
  - stable surface
  - experimental surface (feature-gated)
  - internal crates/contracts
- Documented performance envelopes and failure modes

---

## Non-goals (to avoid thrash)

- "Replace tree-sitter everywhere" — not the objective
- "Be faster than the C runtime" — the objective is competitive and predictable
- "Support every grammar" — the objective is a repeatable pipeline and validation story
