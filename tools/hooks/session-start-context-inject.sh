#!/usr/bin/env bash
# tools/hooks/session-start-context-inject.sh
#
# Trigger:  Codex/Gemini SessionStart
# Purpose:  Inject canonical primitives into the new session's context.
# Behavior: Emits JSON hook output with additionalContext rendered from
#           specs/canonical-primitives.json (single source of truth), while
#           omitting retired wrapper command hints from hook output.
# Non-blocking guarantee: exits 0 always; never modifies any project state.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PRIMITIVES_JSON="$REPO_ROOT/specs/canonical-primitives.json"

if command -v python3 >/dev/null 2>&1; then
  PRIMITIVES_JSON="$PRIMITIVES_JSON" python3 <<'PY'
import json
import os
import sys

path = os.environ["PRIMITIVES_JSON"]

lines = [
    "============================================================",
    " OYATIE CANONICAL PRIMITIVES — session context",
    "============================================================",
    "",
]

try:
    with open(path, encoding="utf-8") as handle:
        data = json.load(handle)
    meta = data.get("_meta", {})
    version = meta.get("version", "unknown")
    updated_at = meta.get("updated_at", "unknown")
    lines.append(f"Source: specs/canonical-primitives.json (version {version}, updated {updated_at})")
    lines.append("Hook note: retired wrapper command hints are intentionally not emitted; use plain git and CI/controller/reviewer status.")
    purpose = meta.get("purpose")
    if purpose:
        lines.append(f"Purpose: {purpose}")
    lines.append("")

    for section in data.get("sections", []):
        title = section.get("title", "Untitled")
        section_lines = [str(line) for line in section.get("lines", [])]
        if not section_lines:
            continue
        lines.append(f"## {title}")
        for line in section_lines:
            lines.append(f"- {line}")
        authority = section.get("authority") or []
        if authority:
            lines.append(f"  Authority: {', '.join(authority)}")
        supersedes = section.get("supersedes") or []
        if supersedes:
            lines.append(f"  Supersedes: {', '.join(supersedes)}")
        lines.append("")

    pointers = data.get("pointers") or {}
    if pointers:
        lines.append("## Pointers")
        note = pointers.get("_note")
        if note:
            lines.append(f"- {note}")
        for key, value in pointers.items():
            if key == "_note":
                continue
            text = f"{key}: {value}"
            lines.append(f"- {text}")
except Exception:
    lines.extend([
        "Canonical primitives JSON exists but could not be rendered: specs/canonical-primitives.json",
        "Fallback: plain git for VCS; CI/controller/reviewer status for merge readiness; OpenAPI 3.2.0 + AsyncAPI 3.1.0.",
    ])

lines.extend([
    "",
    "============================================================",
    " LIFECYCLE SKILL MAP",
    "============================================================",
    "",
    "Vendored skills at tools/agent-skills/skills/ (MIT — Addy Osmani and contributors)",
    "",
    "Define:  interview-me | idea-refine | spec-driven-development",
    "Plan:    planning-and-task-breakdown",
    "Build:   incremental-implementation | test-driven-development | source-driven-development",
    "         doubt-driven-development | context-engineering | api-and-interface-design",
    "         frontend-ui-engineering",
    "Verify:  browser-testing-with-devtools | debugging-and-error-recovery",
    "Review:  code-review-and-quality | code-simplification | security-and-hardening",
    "         performance-optimization",
    "Ship:    git-workflow-and-versioning | ci-cd-and-automation | deprecation-and-migration",
    "         documentation-and-adrs | shipping-and-launch",
    "",
    "Persona agents (tools/agent-skills/agents/):",
    "  review   → code-reviewer",
    "  security → security-auditor",
    "  tests    → test-engineer",
    "",
    "Discovery rule: invoke the skill matching the task phase BEFORE producing output.",
    "Process skills (Define/Plan) come before implementation skills (Build/Verify/Ship).",
    "",
    "============================================================",
    " Hooks are GUIDANCE only. CI/controller gates enforce.",
    " ADR-0221: hooks are guidance infrastructure, not enforcement infrastructure.",
    "============================================================",
])

print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "\n".join(lines),
    }
}, ensure_ascii=False))
PY
else
  cat <<'JSON'
{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"OYATIE CANONICAL PRIMITIVES — session context\nSource: specs/canonical-primitives.json\nHook note: retired wrapper command hints are intentionally not emitted; use plain git and CI/controller/reviewer status.\nContracts: OpenAPI 3.2.0 + AsyncAPI 3.1.0.\nHooks are GUIDANCE only. CI/controller gates enforce."}}
JSON
fi

exit 0
