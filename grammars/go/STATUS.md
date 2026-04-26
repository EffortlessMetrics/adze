# adze-go smoke status

## Current proof (2026-04-26)

`adze-go` now has smoke tests that:

- construct the generated language object, and
- parse a minimal fixture (`package main`) through the pure parser with no reported parse errors.

These tests prove the generated grammar can be constructed and can parse at least one concrete fixture successfully in the crate test suite.

## What is not yet proven

Known blocker: parsing `package main var answer int` currently reports declaration-position errors in the pure parser smoke test, so declaration parsing is not yet smoke-proven.

This is still **not** a stability claim. The crate still needs resolution of the declaration parse blocker, typed extraction checks, broader fixture coverage (for example function declarations, blocks, calls, and returns), negative/error-case expectations, and compatibility checks before it can be treated as stable.
