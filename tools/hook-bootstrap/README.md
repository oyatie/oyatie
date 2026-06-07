# Hook Bootstrap — Operator Notes

Operator reference for `tools/hook-bootstrap/`. See `docs/bootstrap.md` for contributor
onboarding.

---

## What the bootstrap installs / uninstalls

| Action | install.sh | uninstall.sh |
|--------|-----------|-------------|
| Hook entries in `.claude/settings.json` | Writes/merges | Removes by marker |
| `.codex/hooks.json` | Writes (if Codex detected) | Removes (if marker present) |
| `PATH_add bin` in `.envrc` | N/A (file is VCS-tracked) | Removes line if present |
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
├── _canonical-primitives.md               # Historical canonical reference; not injected by hooks
├── session-start-context-inject.sh        # Compatibility no-op; not registered
├── userprompt-canonical-primer.sh         # Compatibility no-op; not registered
├── stop-did-you-forget-suggester.sh       # Stop — incomplete-work check
├── stale-tool-suggester.sh                # PreToolUse(Bash) — retired tool suggestion
├── pre-dispatch-guide.sh                  # PreToolUse(Task) — dispatch quality guide
├── vertical-slice-scope-suggester.sh      # PreToolUse(Write|Edit) — vertical scope
├── cargo-verify-on-rust-edit.sh           # PostToolUse(Edit|Write .rs) — cargo check
├── spec-version-pin-suggester.sh          # PostToolUse(Edit|Write contracts/) — version
├── buildability-line-count.sh             # PostToolUse(Write µservice docs) — line count
├── adr-orphan-detect.sh                   # PostToolUse(Edit|Write .md|.json) — ADR refs
├── microservice-quality-bar.sh            # PostToolUse(Write µservice) + Stop — artifact count
└── vacuous-green-gate-detect.sh           # PostToolUse(Edit|Write lanes|check) — gate honesty

bin/oya                                    # CLI wrapper (PATH_add bin via .envrc)
tools/completions/bash/_oya               # Bash completions
tools/completions/zsh/_oya                # Zsh completions
tools/completions/fish/oya.fish           # Fish completions
tools/agent-skills/                       # Vendored lifecycle skills (addyosmani/agent-skills)
```

---

## How to add a new hook

1. Create the script at `tools/hooks/<name>.sh` with:
   - Shebang: `#!/usr/bin/env bash`
   - `set -euo pipefail`
   - Top-of-file comment: trigger, purpose, behavior, non-blocking guarantee
   - `exit 0` at the end (always)
   - Timeout on any expensive operation (5–30 seconds via `timeout N`)

2. Add the hook entry to `.claude/settings.json` (source-of-truth version in repo):
   ```json
   {
     "type": "command",
     "command": "tools/hooks/<name>.sh",
     "matcher": "<ToolName>",
     "description": "...",
     "managed_by": "tools/hook-bootstrap/install.sh",
     "marker": "oya-bootstrap-v1"
   }
   ```

3. Add the script filename to the `HOOK_SCRIPTS` array in `install.sh` (for executable
   verification on bootstrap).

4. Update `tools/hooks/_canonical-primitives.md` if the hook references new canonical
   primitives.

5. Run `shellcheck tools/hooks/<name>.sh` and fix all warnings.

---

## How to test hooks locally

```bash
# Test stale-tool suggester with mock input
TOOL_INPUT='{"command":"oya git status --short"}' bash tools/hooks/stale-tool-suggester.sh

# Test spec version suggester on a real file
TOOL_INPUT='{"path":"contracts/example.yaml"}' bash tools/hooks/spec-version-pin-suggester.sh

# Test cargo verify on a Rust file
TOOL_INPUT='{"path":"crates/oya-dev-cli/src/lib.rs"}' bash tools/hooks/cargo-verify-on-rust-edit.sh

# Test install dry-run
./tools/hook-bootstrap/install.sh --dry-run

# Test uninstall dry-run
./tools/hook-bootstrap/uninstall.sh --dry-run
```

---

## Idempotency guarantees

- `install.sh` run twice: identical state. The marker string prevents double-installation.
- `uninstall.sh` after `install.sh`: restores pre-install state (hooks removed, Codex
  hooks removed, `PATH_add bin` removed from `.envrc` if we added it).
- `uninstall.sh` on clean repo: exits 0, prints "nothing to remove".
- `install.sh` after `uninstall.sh`: clean reinstall.

---

## Escape hatches

- `tools/hooks/.cargo-verify-disabled` — zero-byte flag file; skips `cargo-verify-on-rust-edit.sh`
  (useful during large refactors with many intermediate broken states)
- `install.sh --skip-skills` — skips agent-skills vendor step (offline contributors)
- `install.sh --sync-skills` — forces re-vendor even if SHA is current
