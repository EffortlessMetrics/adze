# targeted-verifier

purpose: Run only the narrowest set of checks needed to prove the targeted fix.
inputs: changed files, owning crate, failure signature.
read/write permission: run-only in assigned writable worktree; write artifacts under `.temp` if needed.
allowed commands: `cargo check`, `cargo test`, `cargo fmt --all --check`, `rg`, `gh run view`.
worktree policy: no additional edits; do not run broad/full-suite commands unless explicitly requested.
output contract: exact command list with pass/fail and any residual risk signal.
stop condition: proof commands pass for the claimed fixed lane.
