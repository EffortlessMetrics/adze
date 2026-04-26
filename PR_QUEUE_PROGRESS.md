# Current canonical PR decisions

## Green / merge candidates

| PR | Decision | Notes |
|---:|---|---|
| #324 | Keep | Narrow decoder/token-pattern loading fix. Runtime/adze no-run checks passed. |
| #327 | Keep | EOF column encoding fix. Runtime no-run checks passed. |
| #325 | Keep canonical | Stronger parser-table pointer guard than #326. Full tablegen suite passed. |
| #328 | Keep, re-run before merge | External scanner validity rows. Earlier tablegen checks passed. |
| #331 | Keep canonical | Alias counters from grammar alias sequences. Full tablegen suite passed. |
| #330 | Keep after pushed test-alignment commit | Compression validation / u16 guard. Local commit 6ce98c1a green. |
| #399 | Keep canonical | External scanner valid_symbols enforcement. Targeted compile checks passed. |
| #410 | Keep canonical | Lexer callback / zero-length progress hardening. Targeted checks passed. |

## Superseded / redundant

| PR | Superseded by | Reason |
|---:|---:|---|
| #326 | #325 | #325 has stronger helper-based checked accessors and overflow handling. |
| #309 | #331 | Alias-counter duplicate/older implementation. |
| #311 | #410 | Older zero-length lexer loop fix, likely covered by #410. Confirm with diff before close. |
| #407/#408/#409 | #410 | Duplicate lexer zero-length/callback hardening chain. Confirm final diff before close. |

## Blocked

| PR | Reason |
|---:|---|
| #404/#405/#406 | GLR/parser_v4 conflict lane not merge-ready. Ambiguous E2E and glr-core all-features failures remain. Needs dedicated GLR conflict-semantics contract work. |
