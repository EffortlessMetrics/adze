# Smoke status (test-vec-wrapper)

- **Date:** 2026-04-26
- **Status:** Smoke-proven (language-object canary)

## Proven now

- The generated language object can be constructed (`grammar::language()` reports symbols).

## Not yet proven (so not stable yet)

- A strict parse fixture can be asserted without skip logic. For example, a direct
  `grammar::parse("7")` check currently fails with `ParseError { reason:
  UnexpectedToken("end"), start: 1, end: 1 }` in this environment.
- Behavior on representative larger inputs.
- Error recovery and ambiguity behavior under malformed input.
- Long-term stability of this crate's typed surface as a published contract.

Treat this as a canary-level smoke proof, not a stability guarantee.
