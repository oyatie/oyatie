# Hook Bootstrap — Operator Notes

Operator reference for `tools/hook-bootstrap/`. See `docs/bootstrap.md` for contributor
onboarding.

---

## What the bootstrap installs / uninstalls

| Action | install.sh | uninstall.sh |
|--------|-----------|-------------|
| `.claude/settings.json` Claude/OMC boundary | Ensures no project runtime hooks | Removes only legacy bootstrap hook entries |
| `.codex/hooks.json` | Writes (if Codex detected) | Removes (if marker present) |
| `.gemini/settings.json` | Writes (if Gemini detected) | Removes (if marker present) |
| `tools/agent-skills/` | Vendors from upstream | Prompts (preserves by default) |

Marker string: `"oya-bootstrap-v1"` — used to identify bootstrap-managed entries.

---

## File layout

```
tools/hook-bootstrap/
├── install.sh          # Idempotent installer (single entry point)
├── uninstall.sh        # Reversal (traces everything install.sh did)
└── README.md           # This file

tools/hooks/
├── no-cargo-enforcer.sh                   # PreToolUse(Bash) — Buck2/no-cargo guidance
├── injection-content-scanner.sh           # PostToolUse(Bash|Web*) — prompt-injection scan
└── vacuous-green-gate-detect.sh           # PostToolUse(Edit|Write lanes|check) — gate honesty

specs/canonical-primitives.json
└── Direct machine-readable orientation authority; no SessionStart hook.

//:governance-hook-efficacy-check
└── Rust/Buck2 ADR orphan, version-pin, vacuous-green, and buildability fixture checks

tools/hooks/spec-version-pin-suggester/src/main.rs
└── Rust/Buck2 version-pin check; not installed as a runtime hook on Darwin
    until the Buck2 host Rust toolchain target is fixed.

tools/agent-skills/                       # Vendored lifecycle skills (addyosmani/agent-skills)
```

---

## How to add a new hook

1. Prefer a Rust/Buck2 hook command at `tools/hooks/<name>/src/main.rs` with a
   `tools/hooks:...` Buck target. Use shell only for narrow bootstrap/prelude
   compatibility.

2. If a shell hook is unavoidable, create the script at `tools/hooks/<name>.sh` with:
   - Shebang: `#!/usr/bin/env bash`
   - `set -euo pipefail`
   - Top-of-file comment: trigger, purpose, behavior, non-blocking guarantee
   - `exit 0` at the end (always)
   - Timeout on any expensive operation (5–30 seconds via `timeout N`)
   - No network calls, no project-state mutation, and no agent recursion
     (`codex exec`, `claude`, `gemini`, etc.). Runtime hooks emit guidance only.

3. Add the hook entry to the first-class Codex/Gemini hook configs only when
   the command is host-portable for local agents:
   ```json
   {
     "type": "command",
     "command": "buck2 run //tools/hooks:<name> --",
     "matcher": "<ToolName>",
     "description": "...",
     "managed_by": "tools/hook-bootstrap/install.sh",
     "marker": "oya-bootstrap-v1"
   }
   ```
   Do **not** add project runtime hooks to `.claude/settings.json`; Claude hook
   orchestration is owned by OMC so `omc doctor conflicts` stays clean.

4. Add shell hook filenames to `HOOK_SCRIPTS` in `install.sh`.

5. Update `specs/agent-hook-runtime-manifest.json` when the runtime hook set or
   platform mappings change. Update `specs/canonical-primitives.json` if the hook references new canonical
   primitives.

6. Run `buck2 build //tools/hooks:<name>-tests` for Rust checks, or
   `shellcheck tools/hooks/<name>.sh` for shell compatibility hooks.

---

## How to test hooks locally

```bash
# Validate canonical primitives directly
jq empty specs/canonical-primitives.json

# Test the Rust version-pin check in Linux Buck2/Prow lanes
TOOL_INPUT='{"path":"contracts/example.yaml"}' buck2 run //tools/hooks:spec-version-pin-suggester --

# Test install dry-run
./tools/hook-bootstrap/install.sh --dry-run

# Test uninstall dry-run
./tools/hook-bootstrap/uninstall.sh --dry-run
```

---

## Idempotency guarantees

- `install.sh` run twice: identical state. The marker string prevents double-installation.
- `uninstall.sh` after `install.sh`: restores pre-install state for bootstrap-managed hooks.
- `uninstall.sh` on clean repo: exits 0, prints "nothing to remove".
- `install.sh` after `uninstall.sh`: clean reinstall for Codex/Gemini hooks and a
  clean Claude/OMC no-runtime-hook boundary.
- Runtime hook output intentionally contains no retired wrapper command hints; use plain git
  and CI/controller/reviewer status for merge readiness.

---

## Escape hatches

- `install.sh --skip-skills` — skips agent-skills vendor step (offline contributors)
- `install.sh --sync-skills` — forces re-vendor even if SHA is current
