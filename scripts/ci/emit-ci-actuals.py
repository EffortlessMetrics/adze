#!/usr/bin/env python3
"""Emit a ci-actuals.json artifact for the current workflow run.

Compares the PR Plan estimate (target/ci/ci-plan.json, if present) against
observed durations from the GitHub Actions API. The script is intentionally
defensive: if it cannot reach the API or the plan is missing, it emits a
"degraded" report rather than failing.

Inputs (env):
  - GITHUB_TOKEN          : token with actions:read on the repo
  - GITHUB_REPOSITORY     : "owner/repo"
  - GITHUB_RUN_ID         : run id to inspect
  - GITHUB_EVENT_NAME     : push / pull_request / etc.
  - GITHUB_REF            : ref name
  - PR_NUMBER             : optional, the PR number (env)

Outputs:
  - target/ci/ci-actuals.json
"""

from __future__ import annotations

import json
import os
import pathlib
import sys
import urllib.error
import urllib.parse
import urllib.request

API = "https://api.github.com"


def gh_get(path: str, token: str) -> dict | list | None:
    req = urllib.request.Request(
        f"{API}{path}",
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            return json.loads(resp.read())
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as e:
        print(f"warn: GitHub API call failed for {path}: {e}", file=sys.stderr)
        return None


def load_plan() -> dict:
    p = pathlib.Path("target/ci/ci-plan.json")
    if not p.exists():
        return {}
    try:
        return json.loads(p.read_text())
    except Exception as e:  # noqa: BLE001
        print(f"warn: unable to parse {p}: {e}", file=sys.stderr)
        return {}


def parse_iso(s: str | None) -> float | None:
    if not s:
        return None
    from datetime import datetime
    s2 = s.replace("Z", "+00:00")
    try:
        return datetime.fromisoformat(s2).timestamp()
    except ValueError:
        return None


def to_seconds(start: str | None, end: str | None) -> int | None:
    a, b = parse_iso(start), parse_iso(end)
    if a is None or b is None:
        return None
    return max(0, int(b - a))


RUNNER_MULTIPLIERS = {
    "ubuntu-latest": 1.0,
    "ubuntu-22.04": 1.0,
    "ubuntu-24.04": 1.0,
    "windows-latest": 2.0,
    "macos-latest": 10.0,
}


def runner_multiplier(labels: list[str]) -> float:
    for lbl in labels:
        if lbl in RUNNER_MULTIPLIERS:
            return RUNNER_MULTIPLIERS[lbl]
    # macOS family
    for lbl in labels:
        if lbl.startswith("macos"):
            return 10.0
        if lbl.startswith("windows"):
            return 2.0
    return 1.0


def main() -> int:
    repo = os.environ.get("GITHUB_REPOSITORY", "")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    token = os.environ.get("GITHUB_TOKEN", "")
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    ref = os.environ.get("GITHUB_REF", "")
    pr_number = os.environ.get("PR_NUMBER", "")

    out_path = pathlib.Path("target/ci/ci-actuals.json")
    out_path.parent.mkdir(parents=True, exist_ok=True)

    actuals: dict[str, object] = {
        "schema_version": 1,
        "repo": repo,
        "run_id": run_id,
        "event": event,
        "ref": ref,
        "pr": pr_number,
        "plan": load_plan(),
        "jobs": [],
        "status": "ok",
    }

    if not (repo and run_id and token):
        actuals["status"] = "degraded"
        actuals["reason"] = "missing GITHUB_REPOSITORY / GITHUB_RUN_ID / GITHUB_TOKEN"
        out_path.write_text(json.dumps(actuals, indent=2) + "\n")
        print(f"ci-actuals (degraded) -> {out_path}")
        return 0

    jobs_raw = gh_get(f"/repos/{repo}/actions/runs/{run_id}/jobs?per_page=100", token)
    if not isinstance(jobs_raw, dict) or "jobs" not in jobs_raw:
        actuals["status"] = "degraded"
        actuals["reason"] = "could not list jobs for this run"
        out_path.write_text(json.dumps(actuals, indent=2) + "\n")
        print(f"ci-actuals (degraded) -> {out_path}")
        return 0

    plan_lanes = {l.get("id"): l for l in (actuals.get("plan", {}) or {}).get("selection", {}).get("lanes", [])}  # type: ignore[union-attr]

    jobs_out = []
    for job in jobs_raw["jobs"]:
        labels = job.get("labels", []) or []
        seconds = to_seconds(job.get("started_at"), job.get("completed_at"))
        mult = runner_multiplier([str(l) for l in labels])
        actual_lem = round((seconds or 0) / 60.0 * mult, 2)
        cache_hit: bool | None = None
        for step in job.get("steps", []) or []:
            name = (step.get("name") or "").lower()
            if "cache" in name and step.get("conclusion") == "success":
                cache_hit = True
                break
        plan_lane = plan_lanes.get(job.get("name", "").strip())
        jobs_out.append(
            {
                "name": job.get("name"),
                "conclusion": job.get("conclusion"),
                "labels": labels,
                "runner_multiplier": mult,
                "actual_seconds": seconds,
                "actual_lem": actual_lem,
                "estimated_lem": plan_lane.get("lem") if plan_lane else None,
                "cache_hit": cache_hit,
            }
        )

    actuals["jobs"] = jobs_out
    out_path.write_text(json.dumps(actuals, indent=2) + "\n")
    print(f"ci-actuals -> {out_path} ({len(jobs_out)} job entries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
