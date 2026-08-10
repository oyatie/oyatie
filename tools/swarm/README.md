# Swarm delivery guardrails

Mechanical allowlists for worker lanes under the Swarm Delivery Law (root
`integ/<root>` branches, worktree-per-agent, no slow commands in lanes).

Doctrine: ADR-0711 D-9 and
`.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md`
(*Hyperscaler monorepo patterns*) — prefer the doctrine worktree for prose.

## Layout

| Path | Role |
|------|------|
| `git-shim` | Allowlisted `git` for workers |
| `toolguard` | Denies `cargo` / `buck2` in lanes |
| `check-daemon` | Orchestrator `buck2 build //...[check]` → `err.txt` + `.check/errors.json` |
| `self-check.sh` | Hermetic local drift-grep + claim-mechanical pins (opt-in) |
| `claim-push.sh` | Blessed push of `HEAD` → `integ/<root>` with lease + merge-tree; refuses dirty porcelain; `--check` = preflight only |
| `claim_packet.py` | Mechanical Claim packet parse (`docs_touched`/`docs_action`) + Claim↔diff bind (`--bind-diff`); `--self-test` |
| `integ-reset-remote.sh` | Server-side reset: `origin/dev` → `integ/<root>` |
| `lane-shell.sh` | Worker shell with `shim-bin/` first on `PATH` |
| `shim-bin/` | Generated wrappers named `git`, `cargo`, `buck2` |

## Worker lane

```bash
# From a lane worktree:
./tools/swarm/lane-shell.sh
# or one-shot:
./tools/swarm/lane-shell.sh -- git status
```

**Git allowlist:** `status`, `diff`, `log`, `show`, `fetch`, `merge-base`,
`merge-tree`, `rev-parse`, `add <explicit paths>` (no `.` / `-A` / `-u`),
`commit` (explicit pathspecs).

**Git denylist:** `push` (always — use `claim-push.sh`), `stash`, `reset`,
`clean`, `restore`, `checkout`, `rebase`, `merge`, `branch -D/-d/-f`,
`update-ref`, `reflog`, `gc`, `commit --no-verify`/`-n`, `-C` /
`--git-dir` / `--work-tree` (cross-worktree escape). `SWARM_BLESSED_PUSH` is
**not** an admission token; `lane-shell.sh` unsets it.

**Build denylist:** `cargo` and `buck2` fail fast with:

> read err.txt at repo root; use check daemon

Override only with `SWARM_ORCHESTRATOR=1` (never in worker lanes).

## Orchestrator (main checkout only)

```bash
export SWARM_ORCHESTRATOR=1
# Optional: SWARM_MAIN_CHECKOUT=/Users/jasonlee/Developer/oyatie
# Optional: SWARM_CHECK_MODE=workspace|per-target  (default: workspace)
# Optional: SWARM_CHECK_TARGETS=//pkg:name[check],…  (per-target / workspace override)
./tools/swarm/check-daemon
```

`cargo check` is retired (founder 2026-05-29 / `tools/hooks/no-cargo-enforcer.sh`).
The daemon runs `buck2 build --keep-going //...[check]` and never invokes cargo.

Writes:

- `err.txt` at the main-checkout root (human; grouped by crate then file)
- `.check/errors.json` (machine)
- `.check/beads-escalation.stub.md` (how to call `bd` for persistent errors)

**Workspace mode** (`//...[check]`) is the default. Use
`SWARM_CHECK_MODE=per-target` with `SWARM_CHECK_TARGETS` for a focused list.

### Beads escalation (stub)

Persistent errors across two consecutive daemon runs (no owning lane) should
become a bead. With `SWARM_BEADS_ESCALATE=1`, the daemon records fingerprints in
`.check/escalation-state.json` and documents:

```bash
bd create \
  --title "check-daemon: <crate> <file> persists" \
  --labels "swarm,check-daemon,root:<root>" \
  --description "fingerprint=<fp>; see .check/errors.json and err.txt"
```

The daemon does **not** invoke `bd` automatically in this phase.

## Blessed integrator scripts

Always call real git directly (pinned from `/usr/bin/git` or PATH allowlist).
They never admit push through the lane shim — `SWARM_BLESSED_PUSH` is retired
as an admission flag (env-escape closed).

```bash
# Claim preflight only (envelope + merge-tree + dirty refuse; no push)
./tools/swarm/claim-push.sh --check specs

# Claim: push current HEAD onto the durable integ branch (lease-protected)
./tools/swarm/claim-push.sh os

# Mechanical Claim packet self-test (INV-DOC-1 + Claim↔diff bind)
python3 ./tools/swarm/claim_packet.py --self-test

# Bind a Claim packet to the tip diff
python3 ./tools/swarm/claim_packet.py --file claim.txt --bind-diff origin/dev...HEAD

# Kit self-check (anti-drift drift-grep + claim-mechanical)
./tools/swarm/self-check.sh

# After land: server-side reset integ tip to origin/dev (no local reset)
# Accepts FF/merge ancestry OR squash tree-on-dev proof.
./tools/swarm/integ-reset-remote.sh os
# → git push --force-with-lease origin origin/dev:refs/heads/integ/os
```

## Environment reference

| Variable | Meaning |
|----------|---------|
| `GIT_REAL` | Absolute path to real git (shim forward target; ignored/refused in lane-shell when ambient retarget differs from allowlist) |
| `SWARM_BLESSED_PUSH` | **Retired as admission** — lane-shell unsets; shim always denies `push` |
| `SWARM_ORCHESTRATOR=1` | Allow cargo/buck2 passthrough; required for `check-daemon` |
| `SWARM_LANE=1` | Set by `lane-shell.sh` |
| `SWARM_MAIN_CHECKOUT` | Override repo root for daemon outputs |
| `SWARM_CHECK_MODE` | `workspace` (default) or `per-target` |
| `SWARM_CHECK_TARGETS` | Comma-separated buck labels (required for per-target) |
| `SWARM_CHECK_DRY_RUN=1` | Skip buck2; write empty OK artifacts (smoke/tests) |
| `SWARM_ALLOW_WORKTREE=1` | Allow daemon in a linked worktree (tests only) |
| `SWARM_BEADS_ESCALATE=1` | Write two-run fingerprint ratchet state |

## Artifacts (gitignored)

`err.txt` and `.check/` are local orchestrator outputs — not committed.
