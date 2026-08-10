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
| `claim-push.sh` | Blessed push of `HEAD` → `integ/<root>` with lease |
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
`commit`, and `push` only when `SWARM_BLESSED_PUSH=1` (blessed scripts).

**Git denylist:** `stash`, `reset`, `clean`, `restore`, `checkout`, `rebase`,
`merge`, `branch -D/-d/-f`, `update-ref`, `reflog`, `gc`, bare `push --force`.

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

Always call real git (`GIT_REAL`, default `/usr/bin/git`). They set
`SWARM_BLESSED_PUSH=1`.

```bash
# Claim: push current HEAD onto the durable integ branch (lease-protected)
./tools/swarm/claim-push.sh os

# After land: server-side reset integ tip to origin/dev (no local reset)
./tools/swarm/integ-reset-remote.sh os
# → git push --force-with-lease origin origin/dev:refs/heads/integ/os
```

## Environment reference

| Variable | Meaning |
|----------|---------|
| `GIT_REAL` | Absolute path to real git (shim forward target) |
| `SWARM_BLESSED_PUSH=1` | Allow `git push` through the shim (blessed scripts only) |
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
