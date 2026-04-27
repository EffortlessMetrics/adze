# Current Failure Map

- Date: 2026-04-27
- Live board snapshot: 90 open PRs
- First actionable failure: `adze-glr-core` `driver_api_comprehensive` all-features EOF parity failures
- Current blocker: `driver_api_comprehensive`/parse-table invariants with all-features
- Merged / resolved:
  - `#420`
  - `#421`
  - `#422`
  - `#390`
  - `#388` (superseded)
  - `#389` (superseded)
  - `#404` (superseded)
  - `#405`
  - `#406` (superseded)
  - `#411` (superseded)
  - `#392`
  - `#401`
  - `#423` (superseded)
- Remaining active families:
  - typed AST contract (`#412/#414/#415/#416`)
  - product-proof + cleanup follow-ups (`#395`, `#396/#397/#398/#413`)
- Current merge order:
  1. one typed AST contract PR (`#412/#414/#415/#416`)
  2. product-proof PR (`#395`)
  3. one Criterion/bincode cleanup PR (`#396/#397/#398/#413`)
