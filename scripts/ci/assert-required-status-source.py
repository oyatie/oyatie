#!/usr/bin/env python3
"""Fail closed when branch protection cannot prove a required check source app.

Input is the JSON shape returned by GitHub's branch-protection
`required_status_checks` endpoint or an equivalent fixture. This check is
read-only: it never mutates branch protection and it is local/live-read evidence
only until the trusted cloud-ci/oya-ci producer is deployed and bound.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

DEFAULT_REQUIRED_CONTEXT = "oya-ci-required"


def load_input(path: str) -> dict[str, Any]:
    if path == "-":
        return json.load(sys.stdin)
    with Path(path).open() as fh:
        return json.load(fh)


def normalize_contexts(data: dict[str, Any]) -> list[str]:
    contexts = data.get("contexts")
    if isinstance(contexts, list):
        return [item for item in contexts if isinstance(item, str)]
    return []


def normalize_checks(data: dict[str, Any]) -> list[dict[str, Any]]:
    checks = data.get("checks")
    if isinstance(checks, list):
        return [item for item in checks if isinstance(item, dict)]
    return []


def app_id_value(check: dict[str, Any]) -> int | None:
    raw = check.get("app_id")
    if isinstance(raw, int):
        return raw
    if isinstance(raw, str):
        try:
            return int(raw)
        except ValueError:
            return None
    return None


def summarize(data: dict[str, Any], required_context: str, expected_app_id: int | None) -> dict[str, Any]:
    contexts = normalize_contexts(data)
    checks = normalize_checks(data)
    matching_checks = [check for check in checks if check.get("context") == required_context]
    result: dict[str, Any] = {
        "required_context": required_context,
        "expected_source_app_id": expected_app_id,
        "contexts": contexts,
        "checks": checks,
        "p0_0_green": False,
        "phase0_complete": False,
        "authority_boundary": "required-status source binding evidence only; this checker never mutates branch protection or posts statuses",
    }

    if required_context not in contexts:
        result.update(
            verdict="FAIL",
            reason="missing_required_context",
            required_context_source_app_bound=False,
        )
        return result

    if "checks" not in data or not isinstance(data.get("checks"), list):
        result.update(
            verdict="FAIL",
            reason="missing_required_status_checks_checks_array",
            required_context_source_app_bound=False,
        )
        return result

    if not matching_checks:
        result.update(
            verdict="FAIL",
            reason="required_context_not_in_checks_array",
            required_context_source_app_bound=False,
        )
        return result

    winning = matching_checks[0]
    app_id = app_id_value(winning)
    result["observed_source_app_id"] = app_id

    if app_id is None:
        result.update(
            verdict="FAIL",
            reason="missing_required_status_source_app",
            required_context_source_app_bound=False,
        )
        return result

    if app_id == -1:
        result.update(
            verdict="FAIL",
            reason="wildcard_required_status_source_app",
            required_context_source_app_bound=False,
        )
        return result

    if expected_app_id is None:
        result.update(
            verdict="FAIL",
            reason="expected_source_app_id_not_configured",
            required_context_source_app_bound=True,
            trusted_source_app_proven=False,
        )
        return result

    if app_id != expected_app_id:
        result.update(
            verdict="FAIL",
            reason="wrong_required_status_source_app",
            required_context_source_app_bound=True,
            trusted_source_app_proven=False,
        )
        return result

    result.update(
        verdict="PASS",
        reason="required_status_source_app_bound",
        required_context_source_app_bound=True,
        trusted_source_app_proven=True,
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="required_status_checks JSON path, or '-' for stdin")
    parser.add_argument("--required-context", default=DEFAULT_REQUIRED_CONTEXT)
    parser.add_argument("--expected-app-id", type=int, default=None)
    parser.add_argument("--json", action="store_true", help="Emit JSON summary to stdout")
    args = parser.parse_args()

    data = load_input(args.input)
    result = summarize(data, args.required_context, args.expected_app_id)
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
