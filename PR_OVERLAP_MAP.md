# PR Overlap Map

- Family overlaps:
  - GLR conflict semantics (`#388`, `#389`, `#390`) target conflict-cell semantics in `adze-glr-core`.
  - parser_v4 fallback/diagnostics (`#404`, `#405`, `#406`, `#411`) target conflict handling behavior for `parser_v4`.
  - Field-ID preservation (`#400`, `#401`, `#402`, `#403`) target typed-field metadata retention.
  - pure-Rust diagnostics (`#391`, `#392`, `#393`, `#394`) target extraction-facing diagnostics in pure runtime paths.
  - Typed AST contracts (`#412`, `#414`, `#415`, `#416`) target contract assertions for concrete AST values.
  - Criterion/bincode cleanup (`#396`, `#397`, `#398`, `#413`) all target benchmark/dependency cleanup.
  - Product proof (`#395`) depends on the core merge families.
- Canonical landings:
  - `#390` currently canonical for conflict-cell semantics.
  - `#405` canonical for parser_v4 conflict behavior.
  - `#401` canonical for field metadata retention.
  - `#392` canonical for pure-Rust diagnostics.
- Duplicate closure rule after canonical merge:
  - `#423` is superseded by `#420 + #422` (now closed).
  - `#388` and `#389` should be closed as superseded by `#390` once live property status for pt38/pt82 is verified green.
  - Closure note: `Closed as superseded by #<canonical>, which landed the canonical implementation/test for this family.`
