# tools/ reorg drain — Seat B (`integ/tools`)

**Wave:** `W-tools-swarm-to-grok`  
**Judgment:** `ready_for_port_to_grok` (`#1644@0c6284cdef`, envelopes `reorg_debt_freeze.rows` tools/swarm/)  
**Seat:** `integ/tools` (`tools/**` envelope only)  
**Status:** **BLOCKED** — `.grok/**` outside this envelope; port requires `.grok/` writer seat.

## Inventory (2026-08-10)

| Tree | `origin/dev` | `#1644@0c6284cdef` | Duplicated in `.grok/`? |
|------|-------------:|-------------------:|:------------------------|
| `tools/swarm/**` | **0** | **12** | **No** — net-new on #1644 tip |
| `.grok/**` | **208** | **208** | n/a (destination) |
| `tools/**` (total) | **130** | — | — |

`tools/swarm/` is absent on `origin/dev` until `integ/specs#1644` squash-merges. No
delete-only residue is provable on this tip.

## Envelope block

`specs/integ-branch-envelopes.json#roots.tools`:

- **Branch:** `integ/tools`
- **Globs:** `tools/**` only
- **`.grok/**` is NOT in this envelope** — writing there from this seat is an illegal
  cross-envelope edit.

No active adjunct on `integ/tools` authorizes `tools/swarm → .grok/` port. The
`integ/specs` adjunct for `tools/swarm/**` expires at `wave-after-integ-tools-first-land`.

## Exact port plan (for `.grok/` writer)

**Redesign:** `rewrite` (not git-mv). **Shape:** process-kit vacate `tools/`; swarm
shims land under `.grok/swarm/` (peers of existing `.grok/bin/` mm-* family).

| Source (`#1644` tip) | Proposed destination | Notes |
|----------------------|----------------------|-------|
| `tools/swarm/README.md` | `.grok/swarm/README.md` | Merge swarm guardrail prose; cite PORTABLE-SWARM-CONTRACT |
| `tools/swarm/git-shim` | `.grok/swarm/git-shim` | Worker git allowlist |
| `tools/swarm/toolguard` | `.grok/swarm/toolguard` | cargo/buck2 deny in lanes |
| `tools/swarm/check-daemon` | `.grok/swarm/check-daemon` | Orchestrator buck2 check → `err.txt` |
| `tools/swarm/self-check.sh` | `.grok/swarm/self-check.sh` | Anti-drift drift-grep |
| `tools/swarm/claim-push.sh` | `.grok/swarm/claim-push.sh` | Blessed integ push + lease |
| `tools/swarm/claim_packet.py` | `.grok/swarm/claim_packet.py` | Claim packet parse + diff bind |
| `tools/swarm/integ-reset-remote.sh` | `.grok/swarm/integ-reset-remote.sh` | Server-side integ reset |
| `tools/swarm/lane-shell.sh` | `.grok/swarm/lane-shell.sh` | Worker shell; update `shim-bin/` paths |
| `tools/swarm/shim-bin/git` | `.grok/swarm/shim-bin/git` | PATH shim |
| `tools/swarm/shim-bin/cargo` | `.grok/swarm/shim-bin/cargo` | PATH shim |
| `tools/swarm/shim-bin/buck2` | `.grok/swarm/shim-bin/buck2` | PATH shim |

**After port (same wave, multi-seat):**

1. Delete `tools/swarm/**` (reorg-move-out bead).
2. Rewrite path cites in:
   - `specs/integ-branch-envelopes.json` (`claim_mechanical`, `adjunct_claims`, `anti_drift.drift_grep`, `one_shot_exceptions`)
   - `.claude/workflows/deliver.js` (Claim parser paths)
   - `.cursor/rules/swarm-agent-ritual.mdc` (if still cites `tools/swarm/`)
3. Run `.grok/swarm/self-check.sh` (renamed) as drift-grep authority.

## Elevate: who owns `.grok/`?

| Signal | Value |
|--------|-------|
| `.grok/OWNERS` | `cloud-ci-platform` |
| Envelope registration | **Gap** — `.grok/**` not in `#roots` or `#planes` globs on `origin/dev` |
| Judgment `destination_integ` (ledger) | `integ/specs` (coordination only) |
| PROCESS_KIT peer plane | `planes.process_meta` → `integ/ci` (`.github/**`, `.claude/**`, `.cursor/**` — **not** `.grok/**` yet) |
| Northstar | `.grok/` = `keep_forever` PROCESS_KIT (`reorg_debt_freeze.rows`) |

**Required before port execute:**

1. **Envelope owner** registers `.grok/**` (recommend `integ/ci` process_meta extension or
   forward-declared `integ/grok` root) in `specs/integ-branch-envelopes.json` via `integ/specs`.
2. **`.grok/` writer seat** opens `integ/ci` (or registered owner) lane, executes port table
   above, then `integ/tools` deletes `tools/swarm/**`.

Until both seats land, this drain note is the authoritative prep artifact for Seat B.

## Remaining `tools/**` debt (out of scope this slice)

See `tools/DISPOSITION.md` + `evidence/reorg/rr-tools-disposition-20260806.json`.
Notable `ready_for_integ_ci` leaves: `tools/oya-governance-*-app/` → `ci/facade/`.
