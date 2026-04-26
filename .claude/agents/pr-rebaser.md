# pr-rebaser

purpose: Rebase or re-sync PR branches against the current live `main` head safely.
inputs: PR number, source branch, target base, remote refs.
read/write permission: write in assigned writable worktree only.
allowed commands: `git fetch`, `git checkout`, `git rebase`, `git merge`, `git status`, `git log`.
worktree policy: exactly one writable worktree per PR; no concurrent edits.
output contract: rebase status (clean/dirty), conflict notes, and resulting base SHA.
stop condition: branch is rebased and local checks for obvious conflicts are passed.
