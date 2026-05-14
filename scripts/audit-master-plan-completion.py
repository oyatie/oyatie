#!/usr/bin/env python3
"""Audit master-plan status honesty without requiring the whole plan to be done."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
MASTERPLAN = ROOT / ".omc/specs/masterplan.json"
EVIDENCE_DIRS = [ROOT / ".omc/evidence/foundation", ROOT / ".omc/evidence/gitops-vcs", ROOT / ".omc/evidence/agentic-pipeline"]
COMPLETE_STATUSES = {"complete", "accepted", "foundation-cleared", "foundation cleared"}
INCOMPLETE_MARKERS = ("stub", "planned", "pending", "blocked", "in-flight", "probe-green")


def fail(message: str) -> int:
    print(f"audit-master-plan-completion: {message}", file=sys.stderr)
    return 1


def normalized(status: object) -> str:
    return str(status or "").strip().lower()


def is_complete(status: object) -> bool:
    value = normalized(status)
    return value in COMPLETE_STATUSES or value.endswith(" complete")


def is_incomplete(status: object) -> bool:
    value = normalized(status)
    return not is_complete(value) or any(marker in value for marker in INCOMPLETE_MARKERS)


def collect_evidence_text() -> str:
    chunks: list[str] = []
    for directory in EVIDENCE_DIRS:
        if directory.exists():
            for path in directory.glob("*.json"):
                chunks.append(path.read_text(errors="ignore"))
    return "\n".join(chunks)


def audit(data: dict) -> list[str]:
    errors: list[str] = []
    index = data["live_implementation_index"]
    evidence_text = collect_evidence_text()
    for milestone in index.get("milestones", []):
        for phase in milestone.get("phases", []):
            phase_status = phase.get("status")
            child_statuses = [ip.get("status") for ip in phase.get("implementation_plans", [])]
            if is_complete(phase_status) and any(is_incomplete(status) for status in child_statuses):
                errors.append(f"phase {phase.get('id')} is complete but has incomplete child IP")
            for ip in phase.get("implementation_plans", []):
                if is_complete(ip.get("status")) and ip.get("id") not in evidence_text:
                    # Some legacy cross-cutting rows predate evidence naming; warn only through a hard
                    # error once a row claims exact complete without any evidence reference.
                    errors.append(f"complete IP {ip.get('id')} has no evidence JSON reference")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    if not args.check:
        parser.error("only --check is currently supported")
    try:
        data = json.loads(MASTERPLAN.read_text())
    except Exception as exc:
        return fail(str(exc))
    errors = audit(data)
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return fail("completion audit failed")
    print("audit-master-plan-completion: status honesty checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
