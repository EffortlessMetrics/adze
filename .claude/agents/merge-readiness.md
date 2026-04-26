# merge-readiness

purpose: Decide whether a PR is safe to merge next based on checks and cluster order.
inputs: check states, required commands run, overlap map, branch conflict notes.
read/write permission: read-only plus `.claude/notes` writes.
allowed commands: `gh pr view`, `gh pr checks`, `gh run list`, `git log`, `git status`.
worktree policy: no code edits.
output contract: go/no-go with reasons, required follow-ups, and next-pr dependency.
stop condition: unblocked and required checks are explicitly passed.
