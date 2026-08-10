#!/usr/bin/env python3
"""Mechanical Claim packet parse — keep in sync with deliver.js parseClaimPacket.

claim-mechanical + anti-drift-claim-fields (ADR-0711 Amendment D / INV-DOC-1).
Claim↔diff bind (fix-1644-critic-rc): docs_touched paths must appear in
`git diff --name-only` (or CLAIM remains partial theater).
`/^CLAIM/` substring matches are NOT sufficient.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from typing import Any


_FIELD_RE = {
    "docs_touched": re.compile(r"^docs_touched\s*:\s*(.+)$", re.I | re.M),
    "docs_action": re.compile(r"^docs_action\s*:\s*(.+)$", re.I | re.M),
    "docs_action_why": re.compile(r"^docs_action_why\s*:\s*(.+)$", re.I | re.M),
    "paths": re.compile(r"^paths\s*:\s*(.+)$", re.I | re.M),
}
_ALLOWED_ACTIONS = {"update", "add", "delete", "n/a"}


def parse_path_list(raw: str | None) -> list[str] | None:
    """Parse `[a, b]` / `a, b` / `n/a`. Returns None when literal n/a."""
    if raw is None:
        return []
    s = raw.strip()
    if not s:
        return []
    if s.lower() == "n/a":
        return None
    if s.startswith("[") and s.endswith("]"):
        s = s[1:-1].strip()
        if not s:
            return []
        if s.lower() == "n/a":
            return None
    parts: list[str] = []
    for p in re.split(r"[,;]", s):
        item = p.strip().strip("'\"`")
        if item and item.lower() != "n/a":
            parts.append(item)
    return parts


def path_in_diff(declared: str, changed: set[str]) -> bool:
    if declared in changed:
        return True
    # Allow basename / suffix cites when agents list short forms.
    for c in changed:
        if c == declared or c.endswith("/" + declared) or c.endswith(declared):
            return True
        if declared.endswith("/" + c) or declared.endswith(c):
            return True
    return False


def bind_paths_to_diff(
    declared: list[str] | None,
    changed_paths: list[str],
    *,
    field_name: str,
) -> list[str]:
    """Every declared path must appear in git diff --name-only inventory."""
    if declared is None:
        return []  # n/a
    errors: list[str] = []
    changed = set(changed_paths)
    for p in declared:
        if not path_in_diff(p, changed):
            errors.append(
                f"{field_name} path {p!r} not in git diff --name-only "
                "(Claim↔diff bind)"
            )
    return errors


def parse_claim_packet(
    summary: str | None,
    *,
    changed_paths: list[str] | None = None,
    bind_diff: bool = False,
) -> dict[str, Any]:
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
    paths_raw = field("paths")

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

        if bind_diff:
            if changed_paths is None:
                errors.append(
                    "Claim↔diff bind requested but changed_paths missing "
                    "(git diff --name-only)"
                )
            else:
                action = (docs_action or "").lower()
                touched_list = parse_path_list(docs_touched)
                if action != "n/a" and touched_list is not None:
                    errors.extend(
                        bind_paths_to_diff(
                            touched_list, changed_paths, field_name="docs_touched"
                        )
                    )
                # Optional path inventory field — bind when present (not n/a).
                if paths_raw:
                    path_list = parse_path_list(paths_raw)
                    if path_list is not None:
                        errors.extend(
                            bind_paths_to_diff(
                                path_list, changed_paths, field_name="paths"
                            )
                        )

    return {
        "ok": verdict == "CLAIM" and not errors,
        "verdict": verdict,
        "docs_touched": docs_touched,
        "docs_action": docs_action,
        "docs_action_why": docs_action_why,
        "paths": paths_raw,
        "errors": errors,
        "bind_diff": bind_diff,
    }


def git_diff_name_only(diff_range: str, *, repo: str | None = None) -> list[str]:
    cmd = ["git"]
    if repo:
        cmd.extend(["-C", repo])
    cmd.extend(["diff", "--name-only", diff_range])
    out = subprocess.check_output(cmd, text=True)
    return [ln.strip() for ln in out.splitlines() if ln.strip()]


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

    # Claim↔diff bind cases
    bind_ok = parse_claim_packet(
        "CLAIM\ndocs_touched: [docs/decisions/ADR-0711.md]\ndocs_action: update\n",
        changed_paths=["docs/decisions/ADR-0711.md", "specs/integ-branch-envelopes.json"],
        bind_diff=True,
    )
    if not bind_ok["ok"]:
        print(f"FAIL bind_ok expected true got={bind_ok}", file=sys.stderr)
        failed += 1
    bind_bad = parse_claim_packet(
        "CLAIM\ndocs_touched: [docs/decisions/ADR-9999.md]\ndocs_action: update\n",
        changed_paths=["docs/decisions/ADR-0711.md"],
        bind_diff=True,
        )
    if bind_bad["ok"]:
        print(f"FAIL bind_bad expected false got={bind_bad}", file=sys.stderr)
        failed += 1
    bind_na = parse_claim_packet(
        "CLAIM\ndocs_touched: n/a\ndocs_action: n/a\ndocs_action_why: code only\n",
        changed_paths=["tools/swarm/claim-push.sh"],
        bind_diff=True,
    )
    if not bind_na["ok"]:
        print(f"FAIL bind_na expected true got={bind_na}", file=sys.stderr)
        failed += 1
    paths_bind = parse_claim_packet(
        "CLAIM\ndocs_touched: n/a\ndocs_action: n/a\ndocs_action_why: kit\n"
        "paths: [tools/swarm/claim-push.sh]\n",
        changed_paths=["tools/swarm/claim-push.sh"],
        bind_diff=True,
    )
    if not paths_bind["ok"]:
        print(f"FAIL paths_bind expected true got={paths_bind}", file=sys.stderr)
        failed += 1
    paths_unbound = parse_claim_packet(
        "CLAIM\ndocs_touched: n/a\ndocs_action: n/a\ndocs_action_why: kit\n"
        "paths: [tools/swarm/missing.sh]\n",
        changed_paths=["tools/swarm/claim-push.sh"],
        bind_diff=True,
    )
    if paths_unbound["ok"]:
        print(f"FAIL paths_unbound expected false got={paths_unbound}", file=sys.stderr)
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
    p.add_argument(
        "--bind-diff",
        metavar="RANGE",
        help="Bind docs_touched/paths to `git diff --name-only RANGE` (e.g. origin/dev...HEAD)",
    )
    p.add_argument("--repo", help="git -C repo for --bind-diff")
    args = p.parse_args(argv)
    if args.self_test:
        return _self_test()
    if args.file:
        raw = sys.stdin.read() if args.file == "-" else open(args.file, encoding="utf-8").read()
        changed: list[str] | None = None
        bind = False
        if args.bind_diff:
            bind = True
            changed = git_diff_name_only(args.bind_diff, repo=args.repo)
        result = parse_claim_packet(raw, changed_paths=changed, bind_diff=bind)
        json.dump(result, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0 if result["ok"] else 2
    p.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
