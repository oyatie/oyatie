#!/usr/bin/env python3
"""Mechanical Claim packet parse — keep in sync with deliver.js parseClaimPacket.

claim-mechanical + anti-drift-claim-fields (ADR-0711 Amendment D / INV-DOC-1).
`/^CLAIM/` substring matches are NOT sufficient.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from typing import Any


_FIELD_RE = {
    "docs_touched": re.compile(r"^docs_touched\s*:\s*(.+)$", re.I | re.M),
    "docs_action": re.compile(r"^docs_action\s*:\s*(.+)$", re.I | re.M),
    "docs_action_why": re.compile(r"^docs_action_why\s*:\s*(.+)$", re.I | re.M),
}
_ALLOWED_ACTIONS = {"update", "add", "delete", "n/a"}


def parse_claim_packet(summary: str | None) -> dict[str, Any]:
    errors: list[str] = []
    text = summary or ""
    lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
    first = lines[0] if lines else ""
    token = first.split()[0] if first else ""
    verdict: str | None = None
    if re.fullmatch(r"CLAIM", token, flags=re.I):
        verdict = "CLAIM"
    elif re.fullmatch(r"REFUSE", token, flags=re.I):
        verdict = "REFUSE"
    else:
        errors.append(
            f"first non-empty line must start with exactly CLAIM or REFUSE, got {first!r}"
            if first
            else "first non-empty line must start with exactly CLAIM or REFUSE, got '(empty)'"
        )

    def field(name: str) -> str | None:
        m = _FIELD_RE[name].search(text)
        return m.group(1).strip() if m else None

    docs_touched = field("docs_touched")
    docs_action = field("docs_action")
    docs_action_why = field("docs_action_why")

    if verdict == "CLAIM":
        if not docs_touched:
            errors.append("missing docs_touched: [...] (INV-DOC-1)")
        if not docs_action:
            errors.append("missing docs_action: update|add|delete|n/a (INV-DOC-1)")
        elif docs_action.lower() not in _ALLOWED_ACTIONS:
            errors.append(
                f"docs_action must be update|add|delete|n/a, got {docs_action!r}"
            )
        elif docs_action.lower() == "n/a" and not docs_action_why:
            errors.append("docs_action=n/a requires docs_action_why (INV-DOC-1)")

    return {
        "ok": verdict == "CLAIM" and not errors,
        "verdict": verdict,
        "docs_touched": docs_touched,
        "docs_action": docs_action,
        "docs_action_why": docs_action_why,
        "errors": errors,
    }


def _self_test() -> int:
    cases = [
        ("CLAIM\ndocs_touched: [ADR-0711]\ndocs_action: update\n", True),
        ("CLAIM\ndocs_touched: n/a\ndocs_action: n/a\ndocs_action_why: no docs\n", True),
        ("CLAIM\ndocs_touched: [x]\ndocs_action: n/a\n", False),  # missing why
        ("CLAIMED\ndocs_touched: [x]\ndocs_action: update\n", False),  # false-green
        ("not CLAIM\ndocs_touched: [x]\ndocs_action: update\n", False),
        ("REFUSE\nbecause hubs collide\n", False),  # refuse is not ok
        ("all green CLAIM\ndocs_touched: [x]\ndocs_action: update\n", False),
        ("CLAIM\ndocs_action: update\n", False),  # missing docs_touched
    ]
    failed = 0
    for text, expect_ok in cases:
        got = parse_claim_packet(text)
        if got["ok"] != expect_ok:
            print(f"FAIL expect_ok={expect_ok} got={got}", file=sys.stderr)
            failed += 1
    if failed:
        print(f"claim_packet self-test: {failed} failures", file=sys.stderr)
        return 1
    print("claim_packet self-test: OK")
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Parse / self-test Claim packets")
    p.add_argument("--self-test", action="store_true")
    p.add_argument("--file", help="Read packet text from file (- for stdin)")
    args = p.parse_args(argv)
    if args.self_test:
        return _self_test()
    if args.file:
        raw = sys.stdin.read() if args.file == "-" else open(args.file, encoding="utf-8").read()
        result = parse_claim_packet(raw)
        json.dump(result, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0 if result["ok"] else 2
    p.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
