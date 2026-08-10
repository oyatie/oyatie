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
| `check-daemon-hotset` | Fan `check-daemon` across ≤`hot_set_max` integ stations (`.worktrees/integ-*`) |
| `self-check.sh` | Hermetic local drift-grep + claim-mechanical pins (opt-in) |
| `claim-push.sh` | Blessed push of `HEAD` → `integ/<root>` with lease + merge-tree; refuses dirty porcelain; `--check` = preflight only |
| `claim_packet.py` | Mechanical Claim packet parse (`docs_touched`/`docs_action`) + Claim↔diff bind (`--bind-diff`); `--self-test` |
| `integ-reset-remote.sh` | Server-side reset: `origin/dev` → `integ/<root>` |
| `lane-shell.sh` | Worker shell with `shim-bin/` first on `PATH` |
| `shim-bin/` | Generated wrappers named `git`, `cargo`, `buck2` |

## Worker lane

```bash
# From a lane worktree:
./.grok/swarm/lane-shell.sh
# or one-shot:
./.grok/swarm/lane-shell.sh -- git status
```

**Git allowlist:** `status`, `diff`, `log`, `show`, `fetch`, `merge-base`,
`merge-tree`, `rev-parse`, `add <explicit paths>` (no `.` / `-A` / `-u`),
`commit`, and `push` only when `SWARM_BLESSED_PUSH=1` (blessed scripts).

**Git denylist:** `stash`, `reset`, `clean`, `restore`, `checkout`, `rebase`,
`merge`, `branch -D/-d/-f`, `update-ref`, `reflog`, `gc`, bare `push --force`,
`commit --no-verify`/`-n`, `push --no-verify`.

**Build denylist:** `cargo` and `buck2` fail fast with:

> read err.txt at repo root; use check daemon

Override only with `SWARM_ORCHESTRATOR=1` (never in worker lanes).

## Orchestrator (main checkout + hot-set integ stations)

Policy-as-data: `.grok/harness/daemon-hotset.v1.json` (cites
`specs/integ-branch-envelopes.json#merge_windows.hot_set_max` — do not invent a
second max). Advisory perimeter: `.grok/harness/perimeter.v1.json`.

```bash
export SWARM_ORCHESTRATOR=1
# Optional: SWARM_MAIN_CHECKOUT=/Users/jasonlee/Developer/oyatie
# Optional: SWARM_CHECK_MODE=workspace|per-target  (default: workspace)
# Optional: SWARM_CHECK_TARGETS=//pkg:name[check],…  (per-target / workspace override)
./.grok/swarm/check-daemon

# Hot-set early feedback (≤ hot_set_max durable integ stations):
# Always pass an explicit ≤4 list (or SWARM_HOT_SET_STATIONS). Bare discovery of
# every .worktrees/integ-* REFUSEs when count > hot_set_max.
./.grok/swarm/check-daemon-hotset integ-ci integ-specs
SWARM_HOT_SET_STATIONS=integ-ci,integ-os ./.grok/swarm/check-daemon-hotset
SWARM_MAIN_CHECKOUT=/Users/jasonlee/Developer/oyatie ./.grok/swarm/check-daemon-hotset integ-ci
```
`cargo check` is retired (founder 2026-05-29 / `tools/hooks/no-cargo-enforcer.sh`).
The daemon runs `buck2 build --keep-going //...[check]` and never invokes cargo.

**Where it may run**

| Surface | Allowed? |
|---------|----------|
| Main checkout | Yes — default warm buck2 home |
| `.worktrees/integ-<root>` (hot set ≤4) | Yes — via `SWARM_CANDIDATE_ROOT` / `check-daemon-hotset` |
| `.worktrees/lane-*` worker lanes | No — toolguard denies cargo/buck2 |
| Advisory `omx`/`omc`/`gjc`/`grok` scratch | No for daemon; those channels MUST NOT write main checkout (`.grok/harness/perimeter.v1.json`) |

Writes (per selected root):

- `err.txt` (human; grouped by crate then file)
- `.check/errors.json` (machine)
- `.check/beads-escalation.stub.md` (how to call `bd` for persistent errors)

**Workspace mode** (`//...[check]`) is the default. Use
`SWARM_CHECK_MODE=per-target` with `SWARM_CHECK_TARGETS` for a focused list.

### LSP / rust-analyzer carve-out

IDE LSP (rust-analyzer hover/goto/diagnostics) is **read-only feedback** in any
worktree. It is **not** a build, **not** merge authority, and **not** a substitute
for `check-daemon` / `oya-ci-required`. Forbidden: cargo check/build via LSP code
actions in worker lanes; equating analyzer output with daemon `err.txt`.

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

Always call real git (`GIT_REAL`, default `/usr/bin/git`). They set
`SWARM_BLESSED_PUSH=1`.

```bash
# Claim preflight only (envelope + merge-tree + dirty refuse; no push)
./.grok/swarm/claim-push.sh --check specs

# Claim: push current HEAD onto the durable integ branch (lease-protected)
./.grok/swarm/claim-push.sh os

# Mechanical Claim packet self-test (INV-DOC-1 + Claim↔diff bind)
python3 ./.grok/swarm/claim_packet.py --self-test

# Bind a Claim packet to the tip diff
python3 ./.grok/swarm/claim_packet.py --file claim.txt --bind-diff origin/dev...HEAD

# Kit self-check (anti-drift drift-grep + claim-mechanical)
./.grok/swarm/self-check.sh

# After land: server-side reset integ tip to origin/dev (no local reset)
./.grok/swarm/integ-reset-remote.sh os
# → git push --force-with-lease origin origin/dev:refs/heads/integ/os
```

## Environment reference

| Variable | Meaning |
|----------|---------|
| `GIT_REAL` | Absolute path to real git (shim forward target) |
| `SWARM_BLESSED_PUSH=1` | Allow `git push` through the shim (blessed scripts only) |
| `SWARM_ORCHESTRATOR=1` | Allow cargo/buck2 passthrough; required for `check-daemon` / `check-daemon-hotset` |
| `SWARM_LANE=1` | Set by `lane-shell.sh` |
| `SWARM_MAIN_CHECKOUT` | Override repo root for daemon outputs |
| `SWARM_CANDIDATE_ROOT` | Absolute path to an integ station for one daemon run |
| `SWARM_HOT_SET_STATIONS` | Comma-separated integ station names for `check-daemon-hotset` |
| `SWARM_CHECK_MODE` | `workspace` (default) or `per-target` |
| `SWARM_CHECK_TARGETS` | Comma-separated buck labels (required for per-target) |
| `SWARM_CHECK_DRY_RUN=1` | Skip buck2; write empty OK artifacts (smoke/tests) |
| `SWARM_ALLOW_WORKTREE=1` | Allow daemon in a linked worktree (tests only) |
| `SWARM_BEADS_ESCALATE=1` | Write two-run fingerprint ratchet state |

## Artifacts (gitignored)

`err.txt` and `.check/` are local orchestrator outputs — not committed.
