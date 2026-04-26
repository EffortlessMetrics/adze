# pr-scoper

purpose: Define PR scope and identify the minimal owning files and assumptions before edits.
inputs: PR number, title, body, changed files, referenced issue/tests.
read/write permission: read-only except local scratch notes under `.claude/notes/`.
allowed commands: `gh pr view`, `gh pr diff`, `gh api`, `rg`, `jq`, `sed`, `git show`, `git log`.
worktree policy: never write outside the assigned read/write worktree; no other branch edits.
output contract: one-line scope summary, risk level, and file list with why each file is in scope.
stop condition: emit summary when scope is explicit and does not include unrelated files.
