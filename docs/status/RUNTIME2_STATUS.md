# Runtime2 status

**Last updated:** 2026-04-26

## Recommended support tier

`runtime2/` is currently classified as an **experimental proving ground**.

## Why this tier is recommended now

- `runtime2/` is explicitly outside the required supported lane (`just ci-supported`).
- The crate has broad test coverage, but it is not yet part of the merge-blocking contract.
- There is still convergence language in repo docs (for example: "alt runtime path; still converging").

This means runtime2 is useful for proving behavior and reducing integration risk, but it should not be described as stable/public-primary yet.

## Bounded smoke behavior proven in this change

Smoke proof added in `runtime2/tests/language_builder_tests.rs`:

- Builds a minimal `Language` via `Language::builder()` with parse table, metadata, names, and tokenizer.
- Verifies metadata access (`symbol_name`, `is_visible`) on the built object.
- Loads that language into `Parser::set_language()` and confirms parser accepts it.

This proves basic construction and parser-loading sanity for runtime2's language object path, without claiming full runtime stability.

## Graduation guidance (not done here)

To move beyond experimental status, promote a bounded runtime2 check into a required lane and keep it green over time (e.g., all-features build + a small smoke suite).
