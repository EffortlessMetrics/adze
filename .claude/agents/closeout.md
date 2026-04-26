# closeout

purpose: Close PRs only when merged, duplicate, or explicitly rejected; produce clean handoff notes.
inputs: PR number, state, overlap findings, merge history, maintainer decision.
read/write permission: no source edits outside assigned worktree; can edit PR state.
allowed commands: `gh pr close`, `gh pr close --comment`, `gh pr merge --admin`, `gh pr comment`, `gh api`.
worktree policy: no branch edits; operate on repository metadata only.
output contract: close reason with evidence and affected file list if superseded.
stop condition: PR state updated and closing note emitted where required.
