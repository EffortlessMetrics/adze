# Friction Log

**Last updated:** 2026-02-21

If it happens twice, it's not "user error".
It's friction we own until we remove it or document it well enough that it stops recurring.

---

## How to use this

### When you hit friction
1. Open an issue with: what you did, what you expected, what happened, repro.
2. Add/update an entry here and link the issue.

### When you fix friction
- Mark it resolved
- Link the PR
- Add the guardrail that prevents recurrence (docs, error message, CI hint)

---

## Active friction

| ID | Area | Symptom | Impact | Status | Link |
|---:|------|---------|--------|--------|------|
| FR-001 | Docs | Docs drift from dev head (README/book/guides disagree) | Users follow dead paths | Open | (issue) |
| FR-002 | CI | Too many workflows fail/cancel simultaneously on PRs | Signal is noisy | Open | (issue) |
| FR-003 | Dev loop | Supported gate is still heavy on constrained machines | Local iteration cost | Mitigated | (issue) |
| FR-004 | Status | Supported-lane exclusions aren't obvious | Confusing contributor loop | Open | (issue) |

---

## Entry template

### FR-XXX — <short title>

**Area:** docs / ci / tooling / runtime / publishing
**Symptom:** what the user experiences
**Expected:** what they thought would happen
**Actual:** what happened
**Repro:** exact commands + environment
**Fix:** what removes this friction
**Status:** Open / Mitigated / Resolved
**Links:** issue, PR, related docs
