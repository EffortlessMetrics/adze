# Handoff Template

Use this template when creating a handoff document in `docs/handoffs/`.

## Naming

```
docs/handoffs/<milestone>-<descriptive-name>.md
```

Example:

```
docs/handoffs/0.9-contract-convergence-closeout.md
```

## Template

```markdown
# Handoff: <title>

Milestone:
Date:
Owner:
Status: draft | final

## What Shipped

What was delivered.

## What Did Not Ship

What was planned but not delivered, and why.

## Proof Commands

```bash
# Verify the delivered state
just ci-supported
```

## Support-Tier Changes

Which SUPPORT_TIERS.md rows changed and how.

## Policy Changes

Which policy/*.toml files changed and how.

## Known Gaps

What remains incomplete or advisory.

## Follow-Up Issues

Issues created for future work.

## Lessons Learned

What went well, what to improve.
```

## Source Of Truth

Handoffs own:

- what was delivered vs planned
- proof state at closeout
- known gaps and follow-ups
- lessons learned

Other artifacts own:

- ongoing behavior contracts: `docs/specs/`
- active execution state: `.adze/goals/active.toml`
- product claim proof mapping: `docs/status/SUPPORT_TIERS.md`
