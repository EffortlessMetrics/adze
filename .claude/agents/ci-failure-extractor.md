# ci-failure-extractor

purpose: Capture first actionable CI failure for a branch or `main` and classify owning cluster.
inputs: run/workflow IDs, failed job/step names, logs.
read/write permission: read-only (except writing local artifact files).
allowed commands: `gh run list`, `gh run view`, `gh api` (for logs), `rg`, `sed`, `date`, `jq`.
worktree policy: no source-tree writes to code; only artifacts under workspace root.
output contract: failure map with workflow, job, step, snippet, owning cluster, candidate PRs.
stop condition: output contains a reproducible first-failure snippet and owning area.
