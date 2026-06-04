#!/usr/bin/env python3
"""Fail closed when a PR status rollup does not prove oya-ci-required success.

Input is the JSON shape returned by `gh pr view --json headRefOid,statusCheckRollup`
or an equivalent fixture. This check is non-mutating: it never posts commit
statuses and it must not be used to turn local evidence into protected-branch or
Phase-0 exit authority.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

DEFAULT_REQUIRED_CONTEXT = "oya-ci-required"
DEFAULT_TRUSTED_PRODUCER = "cloud-ci/oya-ci"
SUCCESS_VALUES = {"success", "successful", "passed", "pass"}
LEGACY_CONTEXTS = {
    "cargo-fmt",
    "cargo-check",
    "cargo-clippy",
    "cargo-nextest",
    "cargo-deny",
    "oya-verify",
    "oya-gate",
    "buck2-affected-only",
}
PRODUCER_KEYS = (
    "workflow",
    "workflowName",
    "workflow_name",
    "producer",
    "producerName",
    "producer_name",
    "provider",
    "service",
    "source",
    "app",
    "appName",
    "app_name",
)
PRODUCER_NESTED_KEYS = (
    "app",
    "checkRun",
    "checkSuite",
    "statusContext",
    "workflowRun",
)


def load_input(path: str) -> dict[str, Any]:
    if path == "-":
        return json.load(sys.stdin)
    with Path(path).open() as fh:
        return json.load(fh)


def as_rollup_items(data: dict[str, Any]) -> list[dict[str, Any]]:
    rollup = data.get("statusCheckRollup", [])
    if rollup is None:
        return []
    if isinstance(rollup, list):
        return [item for item in rollup if isinstance(item, dict)]
    if isinstance(rollup, dict):
        nodes = rollup.get("nodes")
        if isinstance(nodes, list):
            return [item for item in nodes if isinstance(item, dict)]
        return [rollup]
    return []


def context_name(item: dict[str, Any]) -> str:
    for key in ("name", "context", "checkName"):
        value = item.get(key)
        if isinstance(value, str) and value:
            return value
    # GraphQL rollup nodes may nest checkRun/statusContext shapes.
    for key in ("statusContext", "checkRun"):
        nested = item.get(key)
        if isinstance(nested, dict):
            nested_name = context_name(nested)
            if nested_name:
                return nested_name
    return ""


def state_value(item: dict[str, Any]) -> str:
    # Prefer a terminal conclusion over a generic status/state. GitHub check runs
    # commonly expose status=COMPLETED plus conclusion=SUCCESS/FAILURE;
    # "completed" alone is not a pass signal.
    for key in ("conclusion", "state", "bucket", "status"):
        value = item.get(key)
        if isinstance(value, str) and value:
            return value.lower().replace("-", "_")
    for key in ("statusContext", "checkRun"):
        nested = item.get(key)
        if isinstance(nested, dict):
            nested_state = state_value(nested)
            if nested_state:
                return nested_state
    return ""


def producer_values(item: dict[str, Any]) -> list[str]:
    values: list[str] = []
    for key in PRODUCER_KEYS:
        value = item.get(key)
        if isinstance(value, str) and value:
            values.append(value)
        elif isinstance(value, dict):
            for nested_key in ("name", "slug", "login"):
                nested_value = value.get(nested_key)
                if isinstance(nested_value, str) and nested_value:
                    values.append(nested_value)
    for key in PRODUCER_NESTED_KEYS:
        nested = item.get(key)
        if isinstance(nested, dict):
            values.extend(producer_values(nested))
    # Preserve first-seen order while removing duplicates.
    return list(dict.fromkeys(values))


def is_success(item: dict[str, Any]) -> bool:
    value = state_value(item)
    return value in SUCCESS_VALUES


def is_trusted_producer(item: dict[str, Any], trusted_producer: str) -> bool:
    values = {value.lower() for value in producer_values(item)}
    expected = trusted_producer.lower()
    if expected in values:
        return True
    if "/" in expected:
        left, right = expected.split("/", 1)
        return left in values and right in values
    return False


def summarize(data: dict[str, Any], required_context: str, trusted_producer: str) -> dict[str, Any]:
    items = as_rollup_items(data)
    contexts = [context_name(item) for item in items]
    legacy_present = sorted(name for name in contexts if name in LEGACY_CONTEXTS)
    matches = [item for item in items if context_name(item) == required_context]

    result: dict[str, Any] = {
        "required_context": required_context,
        "headRefOid": data.get("headRefOid"),
        "contexts": contexts,
        "legacy_contexts_present": legacy_present,
        "trusted_producer": trusted_producer,
        "p0_0_green": False,
        "phase0_complete": False,
        "authority_boundary": "status-rollup evidence only; this checker never posts statuses",
    }

    if not items:
        result.update(
            verdict="FAIL",
            reason="no_status_checks_reported",
            required_context_status="missing",
        )
        return result

    if not matches:
        result.update(
            verdict="FAIL",
            reason="missing_required_context",
            required_context_status="missing",
        )
        return result

    winning = matches[0]
    state = state_value(winning) or "unknown"
    producer = producer_values(winning)
    result["required_context_status"] = state
    result["required_context_producer_values"] = producer
    if not is_success(winning):
        result.update(
            verdict="FAIL",
            reason="required_context_not_success",
        )
        return result
    if not producer:
        result.update(
            verdict="FAIL",
            reason="missing_required_context_producer",
            required_context_trusted_producer=False,
        )
        return result
    if not is_trusted_producer(winning, trusted_producer):
        result.update(
            verdict="FAIL",
            reason="untrusted_required_context_producer",
            required_context_trusted_producer=False,
        )
        return result

    result.update(
        verdict="PASS",
        reason="required_context_success",
        required_context_proven=True,
        required_context_trusted_producer=True,
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="PR view/status rollup JSON path, or '-' for stdin")
    parser.add_argument("--required-context", default=DEFAULT_REQUIRED_CONTEXT)
    parser.add_argument("--trusted-producer", default=DEFAULT_TRUSTED_PRODUCER)
    parser.add_argument("--json", action="store_true", help="Emit JSON summary to stdout")
    args = parser.parse_args()

    data = load_input(args.input)
    result = summarize(data, args.required_context, args.trusted_producer)
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
