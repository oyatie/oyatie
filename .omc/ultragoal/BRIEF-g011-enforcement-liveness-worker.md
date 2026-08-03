# Worker brief — G011 enforcement-liveness gate (FRIC-012 part c; one worker, one PR)

Friction: FRIC-012 in `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/friction-ledger.jsonl` — the no-cargo enforcer hook existed 11 days wired only into `.codex/hooks.json`, so every Claude session ran unenforced (enforcement existed, liveness did not). Parts (a)/(b) are done: `.claude/settings.json` is tracked and mirrors `.codex/hooks.json`. Part (c) — make hook liveness mechanically impossible to lose — is THIS lane.

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-enforcement-liveness` (branch `agent/g011-enforcement-liveness`, base = current origin/dev @ a8797a4df). NEVER touch the main checkout. Never run omc orphan-cleanup.

## Verified facts (2026-06-10)
- 11 files in `tools/hooks/`: 9 live (each referenced in BOTH `.claude/settings.json` and `.codex/hooks.json`) + 2 deliberate no-op compatibility stubs (`session-start-context-inject.sh`, `userprompt-canonical-primer.sh` — header literally says "Compatibility stub only" and body is `exit 0`). Debt today = 0 ⇒ the gate is born-blocking with frozen-empty baselines.
- `.claude/settings.json` `_note` mandates keeping the two wiring files in sync via governance PR — that mirror invariant is currently convention-only.
- `.claude/settings.local.json` is NOT gitignored (FRIC-012 fix (b) wanted it ignored; a local file would shadow/extend tracked wiring invisibly).

## Deliverables (one PR)
1. New single-concern gate crate `cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app` mirroring the freshness/target-parity pattern (producer face rows in `oya-cloud-ci-accounting-registry-app`, pure policy gate, BUCK, matrix line, registrations in `oya-ci.toml` + `libs/oya-ci-config` (count + `gate-disposition.json`) + `docs/oya-ci/gate-catalog.md`):
   - Face rows from tracked files: per `tools/hooks/*.sh` → `{hook_path, wired_in_claude: bool, wired_in_codex: bool, stub_marked: bool}` (stub detection = header comment contains "Compatibility stub only"); per wiring-file command reference → `{wiring_file, command_path, target_exists: bool}`.
   - Violation codes (all born-blocking, frozen-empty):
     `hook_unwired_without_stub_marker` (a hook in tools/hooks/ not referenced by BOTH wiring files and not stub-marked — the FRIC-012 class),
     `hook_wiring_mirror_drift` (live hook referenced in one wiring file but not the other),
     `wired_hook_missing_file` (a wiring file references a command path that is not a tracked file).
   - Remediation text per code names the exact file to edit and the governance-PR requirement.
   - Tests: GREEN fixture (current tree shape) + RED fixture per code. Cited tests must exist.
2. `.gitignore`: add `.claude/settings.local.json` with a one-line comment (local wiring is session-scoped, never tracked, never authoritative).
3. PR body cites FRIC-012 + ADR-0539/0540 gate precedents (no new ADR — this is a gate addition under the established G011 ratchet pattern; cite the enforcement-layering doctrine line from the friction row).

## Rules
- buck2 build + buck2 test = green signal (cargo supplementary). Lock refresh ONLY via `cargo metadata >/dev/null` (new crate is auto-membered by the gates/* glob — zero root Cargo.toml edits; lock gains one package).
- OBEY THE SETTLE PROTOCOL: content commit(s) FIRST → run `infra/ci/materialize-cloud-ci-generated-faces.sh .` (or the new `oya-cloud-ci-face-settle --settle` bin) → FACES-ONLY settle commit. Faces regenerate from TRACKED paths — `git add` everything before materializing. Never hand-edit `*.generated.json`.
- SSH-signed commits; `git push -u origin agent/g011-enforcement-liveness`; open PR to dev with `gh` including buck2 evidence.
