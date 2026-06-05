#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

session_context_output="$tmpdir/session-context.json"
bash "$repo_root/tools/hooks/session-start-context-inject.sh" >"$session_context_output"
python3 - "$session_context_output" >"$tmpdir/session-context.txt" <<'PYCTX'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["hookSpecificOutput"]["additionalContext"])
PYCTX
grep -q 'specs/canonical-primitives.json' "$tmpdir/session-context.txt"
grep -q 'OpenAPI 3.2.0' "$tmpdir/session-context.txt"
grep -q 'AsyncAPI 3.1.0' "$tmpdir/session-context.txt"
if grep -q 'tools/hooks/_canonical-primitives.md' "$tmpdir/session-context.txt"; then
  echo "session-start hook still points at retired markdown canonical primitives" >&2
  exit 1
fi
if grep -Eiq 'oya[[:space:]]+(git|vcs|gate|verify)|\.\/bin\/oya|bin/oya|oya --help|Oya CLI|oya CLI' "$tmpdir/session-context.txt"; then
  echo "session-start hook still emits retired wrapper command guidance" >&2
  cat "$tmpdir/session-context.txt" >&2
  exit 1
fi
if [ -e "$repo_root/tools/hooks/stale-tool-suggester.sh" ]; then
  echo "stale-tool-suggester hook should be deleted with retired wrapper command guidance" >&2
  exit 1
fi

no_cargo_output="$tmpdir/no-cargo.out"
set +e
printf '%s\n' '{"tool_input":{"command":"cargo test --workspace"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$no_cargo_output" 2>&1
no_cargo_status=$?
set -e
if [ "$no_cargo_status" -ne 0 ]; then
  echo "no-cargo hook must be advisory/non-blocking, got exit $no_cargo_status" >&2
  cat "$no_cargo_output" >&2
  exit 1
fi
grep -q 'Cargo executable lanes are retired' "$no_cargo_output"
grep -q 'advisory only' "$no_cargo_output"

python3 - "$repo_root" <<'PY'
import json
import pathlib
import re
import sys

repo = pathlib.Path(sys.argv[1])
manifest_path = repo / "specs/agent-hook-runtime-manifest.json"
with manifest_path.open(encoding="utf-8") as handle:
    manifest = json.load(handle)

config_paths = [
    repo / entry["path"]
    for entry in manifest["config_files"]
    if entry.get("first_class") is True
]
allowed_hooks = {
    entry["path"]
    for entry in manifest["runtime_hooks"]
}
allowed_commands = {
    entry.get("command", entry["path"])
    for entry in manifest["runtime_hooks"]
}
pre_errors = []
missing = []
unlisted = []
referenced = set()

claude_settings = repo / ".claude/settings.json"
if claude_settings.is_file():
    with claude_settings.open(encoding="utf-8") as handle:
        claude_data = json.load(handle)
    if claude_data.get("hooks"):
        pre_errors.append(
            "Claude project runtime hooks are intentionally disabled; OMC owns Claude hook orchestration"
        )

def walk(value):
    if isinstance(value, dict):
        command = value.get("command")
        if isinstance(command, str) and (
            command.startswith("tools/hooks/")
            or command.startswith("buck2 run //tools/hooks:")
        ):
            referenced.add(command)
            if command.startswith("tools/hooks/"):
                target = repo / command
            else:
                target_name = command.split("//tools/hooks:", 1)[1].split()[0]
                target = repo / "tools/hooks" / target_name / "src/main.rs"
            if command.startswith("tools/hooks/") and not target.is_file():
                missing.append(f"{command} referenced by active hook config")
            if command.startswith("buck2 run //tools/hooks:") and not target.is_file():
                missing.append(f"{command} referenced by active hook config without Rust source")
            if command not in allowed_commands:
                unlisted.append(f"{command} referenced by active hook config but absent from {manifest_path.name}")
        for child in value.values():
            walk(child)
    elif isinstance(value, list):
        for child in value:
            walk(child)

for config in config_paths:
    with config.open(encoding="utf-8") as handle:
        data = json.load(handle)
    if config.name == "hooks.json" and "UserPromptSubmit" in data.get("hooks", {}):
        pre_errors.append("Codex UserPromptSubmit hook is intentionally disabled; SessionStart carries canonical context")
    if config.name == "settings.json" and config.parent.name == ".gemini" and "BeforeAgent" in data.get("hooks", {}):
        pre_errors.append("Gemini BeforeAgent hook is intentionally disabled; SessionStart carries canonical context")
    walk(data)

unreferenced = sorted(allowed_commands - referenced)
non_executable = sorted(
    path for path in allowed_hooks
    if (repo / path).is_file() and not (repo / path).stat().st_mode & 0o111
    and path.endswith(".sh")
)
forbidden_patterns = [
    (re.compile(r"\b(?:curl|wget|gh|ssh|scp|nc)\b"), "network/remote command"),
    (re.compile(r"\bcodex\s+exec\b|\bclaude\s+(?:-p|--print|code|mcp|exec)\b|\bgemini\s+(?:-p|--prompt|exec)\b"), "agent recursion"),
    (re.compile(r"\bgit\s+push\b"), "git push mutation"),
    (re.compile(r"\brm\s+-rf\b"), "destructive cleanup"),
    (re.compile(r"\.omc|\.omx", re.IGNORECASE), "OMC/OMX runtime coupling"),
    (re.compile(r"_canonical-primitives\.md"), "retired markdown canonical primitives"),
    (re.compile(r"\boya\s+(?:git|vcs|gate|verify)\b|\./bin/oya|\bbin/oya\b|\boya --help\b|Oya CLI|oya CLI", re.IGNORECASE), "retired wrapper command guidance"),
]
forbidden_hits = []
for hook in sorted(allowed_hooks):
    target = repo / hook
    if not target.is_file():
        continue
    text = target.read_text(encoding="utf-8")
    for pattern, label in forbidden_patterns:
        if pattern.search(text):
            forbidden_hits.append(f"{hook}: {label}")

errors = pre_errors + missing + unlisted
errors += [f"{path} declared in {manifest_path.name} but not referenced by first-class hook configs" for path in unreferenced]
errors += [f"{path} is declared as a runtime hook but is not executable" for path in non_executable]
errors += forbidden_hits

if errors:
    for item in errors:
        print(item, file=sys.stderr)
    sys.exit(1)
PY

if rg -n \
  'Preferred drop-in surface: oya git|policy ratchet compatibility|policy-ratchet|route through `oya git`|Top-level subcommands: git|oya-git cutover|migrate plain git/drop-in docs toward oya git|oya[[:space:]]+(git|vcs|gate|verify)|\.\/bin\/oya|bin/oya|oya --help|Oya CLI|oya CLI' \
  "$repo_root/tools/hooks" \
  "$repo_root/tools/hook-bootstrap" \
  "$repo_root/.codex/hooks.json" \
  "$repo_root/.claude/settings.json" \
  "$repo_root/.gemini/settings.json"; then
  echo "active governance hook surface still contains retired wrapper guidance" >&2
  exit 1
fi

if rg -n '_canonical-primitives\.md' \
  "$repo_root/tools/hooks" \
  "$repo_root/tools/hook-bootstrap" \
  "$repo_root/tools/agent-skills/AGENTS.md" \
  "$repo_root/tools/agent-skills/INHERITANCE.md"; then
  echo "active hook guidance still references retired markdown canonical primitives" >&2
  exit 1
fi

echo "governance hook retired VCS/wrapper surface tests passed"
