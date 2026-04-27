# PR Overlap Map

- Family overlaps:
  - GLR conflict semantics (`#388`, `#389`, `#390`) all target conflict-cell semantics in `adze-glr-core`.
  - parser_v4 fallback/diagnostics (`#404`, `#405`, `#406`, `#411`) all target conflict handling behavior for `parser_v4`.
  - Field-ID preservation (`#400`, `#401`, `#402`, `#403`) all target typed-field metadata retention.
  - pure-Rust diagnostics (`#391`, `#392`, `#393`, `#394`) all target extraction-facing diagnostics in pure runtime paths.
  - Typed AST contracts (`#412`, `#414`, `#415`, `#416`) all target contract assertions for concrete AST values.
  - Criterion/bincode cleanup (`#396`, `#397`, `#398`, `#413`) all target benchmark/dependency cleanup.
  - Product proof (`#395`) depends on the core merge families.
- Duplicate closure rule after canonical merge:
  - Close all remaining PRs in the family after the canonical PR lands.
  - Closure note: `Closed as superseded by #<canonical>, which landed the canonical implementation/test for this family.`
