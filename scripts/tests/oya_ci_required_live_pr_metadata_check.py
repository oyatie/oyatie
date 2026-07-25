#!/usr/bin/env python3
"""Keep PR traceability admission bound to metadata fetched when the job runs."""
from __future__ import annotations

import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "oya-ci-required.yml"
STEP_START = "      - name: Pre-provision pinned rust toolchain and PR metadata preflight\n"
STEP_END = "      # Restore buck-out read-only"
LIVE_PR_ENDPOINT = 'gh api "repos/${GITHUB_REPOSITORY}/pulls/${PR_NUMBER}"'


class ContractViolation(Exception):
    pass


def fail(message: str) -> None:
    raise ContractViolation(message)


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def metadata_step(workflow: str) -> str:
    try:
        return workflow.split(STEP_START, 1)[1].split(STEP_END, 1)[0]
    except IndexError:
        fail("PR metadata preflight step is missing or no longer delimited")
        raise AssertionError("unreachable")


def validate(workflow: str) -> None:
    step = metadata_step(workflow)
    require("GH_TOKEN: ${{ github.token }}" in step, "live GitHub API read must remain authenticated")
    require('if [ "${EVENT_NAME}" = "pull_request" ]; then' in step, "live metadata read must stay scoped to pull_request")
    require(step.count(LIVE_PR_ENDPOINT) >= 2, "title and body must be fetched from the live PR API")
    require("--jq '.title'" in step, "admission title must come from the live PR API")
    require("--jq '.body // \"\"' > \"${body_path}\"" in step, "admission body must come from the live PR API")
    require('--pr-title "${pr_title}"' in step, "admission binary must receive the live title")
    require('--pr-body "${body_path}"' in step, "admission binary must receive the live body")
    require("github.event.pull_request.title" not in step, "frozen event title must not drive admission")
    require("github.event.pull_request.body" not in step, "frozen event body must not drive admission")


def main() -> None:
    workflow = WORKFLOW.read_text(encoding="utf-8")
    validate(workflow)

    # Regression proof: the old event-payload body path must fail this contract.
    stale = workflow.replace(
        "gh api \"repos/${GITHUB_REPOSITORY}/pulls/${PR_NUMBER}\" --jq '.body // \"\"' > \"${body_path}\"",
        'printf \'%s\\n\' "${{ github.event.pull_request.body }}" > "${body_path}"',
        1,
    )
    try:
        validate(stale)
    except ContractViolation:
        pass
    else:
        fail("stale event payload regression was accepted")

    print(f"oya-ci-required live PR metadata check passed: {WORKFLOW.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    try:
        main()
    except ContractViolation as exc:
        print(f"oya-ci-required live PR metadata check failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
