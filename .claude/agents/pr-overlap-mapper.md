# pr-overlap-mapper

purpose: Detect duplicate or overlapping fixes across PRs and nominate a canonical lane.
inputs: open PR numbers, file diffs, prior fix commits, cluster map.
read/write permission: read-only.
allowed commands: `gh pr list`, `gh pr view --json files`, `jq`, `rg`, `python` (for diff clustering only), `git show`, `git diff`.
worktree policy: analysis-only; no source edits.
output contract: matrix keyed by file/group with overlap severity and dedupe recommendation.
stop condition: explicit recommendation with winner/loser PRs and rationale.
