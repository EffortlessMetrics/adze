# Droid Review Rules for Adze

Rules governing Droid review behavior in this repository.

## Workflow Baseline
- **Safe action ref**: `EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f`
- **Debug artifacts**: Disabled (`upload_debug_artifacts: false`)
- **Model**: MiniMax M2.7 via BYOK (`custom:MiniMax-M2.7-0`)
- **Review depth**: Shallow
- **Security threshold**: High (security_block_on_critical: true)
- **Auto-review gate**: Same-repo PRs only (no fork PRs)
- **Manual @droid gate**: OWNER/MEMBER/COLLABORATOR only

## Review Scope
Droid auto-review runs on:
- Pull requests from the same repository
- Not on draft PRs (unless explicitly triggered)
- Not on PRs with `[skip-review]` in title

## Finding Format
Structured findings with:
1. Priority level [P0|P1|P2]
2. Short title (one line)
3. Failure mode (why it matters)
4. Why here (context)
5. Fix direction (actionable steps)
6. Validation (how to verify)
7. Confidence (High/Medium/Low)

## Clean Review Format
When no actionable findings:
- Inspection surfaces covered
- Checks performed
- Why no comments
- Residual risk
- Validation signals (Observed/Reported/Not verified split)

## Model Behavior Expectations
- MiniMax M2.7 with `review_depth: shallow` prioritizes high-signal findings
- No arbitrary comment caps — findings are complete
- Shallow depth avoids diving into build artifacts or generated code
- Evidence provenance is explicit (observed vs. reported vs. unverified)

## Manual Trigger (@droid)
Trusted actors can trigger:
- `@droid review` — Manual code review
- `@droid security` — Security-focused review

Requires comment from OWNER, MEMBER, or COLLABORATOR.

## Security Scan
Weekly scheduled scan (Monday 8 AM UTC) or manual dispatch via `workflow_dispatch`.
Focuses on security-severity medium and above.

## No Raw Artifact Upload
Droid workflows must not upload:
- `$HOME/.factory/**` (raw settings with resolved credentials)
- `droid-prompts/**` (raw prompt templates)
- `droid-review-debug-<run_id>` (raw debug artifacts)

Expected artifact state: None. All review occurs via comments and PR reviews.
