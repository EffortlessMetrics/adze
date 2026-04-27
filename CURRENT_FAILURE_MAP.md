# Current Failure Map

- Date: 2026-04-27
- Live board snapshot: 96 open PRs
- First actionable failure: `adze-glr-core` action-cell semantics (`pt38_cell_conflict_iff_multi_action`, `pt82`)
- Current blocker: property failures in conflict-cell duplicate/`Error` normalization
- Merged / resolved:
  - `#420`
  - `#421`
  - `#422`
  - `#390`
  - `#392`
  - `#401`
  - `#405`
  - `#423` (superseded)
- Next active frontier:
  - GLR conflict semantics (`#388/#389` duplicates still open)
- Likely already merged:
  - `#392`
- Current merge order:
  1. one GLR conflict-semantics PR (`#388/#389` against merged `#390`)
  2. one pure-GLR routing follow-up in runtime
  3. one typed AST contract PR (`#412/#414/#415/#416`)
  4. product-proof PR (`#395`)
  5. one Criterion/bincode cleanup PR (`#396/#397/#398/#413`)
