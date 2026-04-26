# Runtime2 status

**Last updated:** 2026-04-26

## Support tier

`runtime2/` is currently classified as an **experimental proving ground**.

This means:
- it is intentionally outside the required supported lane (`just ci-supported`),
- APIs and behavior may change while convergence work continues,
- it should not be treated as a stable/publicly guaranteed runtime contract yet.

## What is currently proven (bounded smoke)

The repository now includes a focused smoke test at
`runtime2/tests/runtime2_smoke.rs` that proves one minimal behavior:

- a `Language` can be built with parse table + symbol metadata,
- static tokens can be attached,
- and `Parser::set_language` accepts that language.

This is intentionally narrow: it proves constructor/builder wiring only, not full
parse correctness or stability guarantees.

## Graduation criteria (high level)

Runtime2 can be reconsidered for a stronger support tier only after it has:
- a bounded, reliable CI gate in normal developer workflows,
- explicit compatibility/stability guarantees documented for consumers,
- and sustained green behavior beyond smoke-level constructor checks.
