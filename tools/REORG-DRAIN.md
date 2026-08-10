# tools/ reorg drain — Seat B (`integ/tools`)

**Wave:** `W-tools-swarm-to-grok`  
**Judgment:** `ready_for_port_to_grok` (`#1644@0c6284cdef`, envelopes `reorg_debt_freeze.rows` tools/swarm/)  
**Seat:** `integ/tools` (`tools/**` envelope only)  
**Status:** **SWARM SHRINK DONE** — `tools/swarm/**` absent on this tip; absorb verified on
`integ/ci@076e712fa` (`.grok/swarm/` forever home). No reorg absorb on `tools/**`.

## Shrink receipt (2026-08-10)

| Check | Result |
|-------|--------|
| `tools/swarm/**` on `integ/tools` tip | **0 files** (already absent; never landed from `origin/dev`) |
| Delete action this seat | **no-op** — nothing to delete on disk or in tree |
| Absorb evidence | `integ/ci@076e712fa` port + tip absorb `e4794dfbf` |
| `.grok/swarm/**` on `origin/integ/ci` | **13 paths** (12 kit files + README; + `check-daemon-hotset`) |
| This lane scope | `tools/**` shrink only — **no** `.grok/**` writes |

## Inventory (post-shrink)

| Tree | `origin/dev` | `#1644@0c6284cdef` | This tip (`integ/tools`) | `.grok/` home |
|------|-------------:|-------------------:|:------------------------:|:--------------|
| `tools/swarm/**` | **0** | **12** | **0** (shrunk) | n/a |
| `.grok/swarm/**` | — | — | — (out of envelope) | landed on `integ/ci` |

`tools/swarm/` was net-new on the `#1644` tip and never present on `integ/tools` /
`origin/dev`. Seat B's shrink obligation is satisfied by confirming absence + drain note.

## Historical port plan (executed on `integ/ci`, not this seat)

**Redesign:** `rewrite` (not git-mv). **Shape:** process-kit vacate `tools/`; swarm
shims under `.grok/swarm/` (peers of existing `.grok/bin/` mm-* family).

| Source (`#1644` tip) | Destination (integ/ci) | Notes |
|----------------------|------------------------|-------|
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

**Cite rewrites (other seats, not this shrink):**

- `specs/integ-branch-envelopes.json` (`claim_mechanical`, `adjunct_claims`, `anti_drift.drift_grep`, `one_shot_exceptions`)
- `.claude/workflows/deliver.js` (Claim parser paths)
- `.cursor/rules/swarm-agent-ritual.mdc` (if still cites `tools/swarm/`)

## Remaining `tools/**` debt (out of scope this slice)

See `tools/DISPOSITION.md` + `evidence/reorg/rr-tools-disposition-20260806.json`.
Notable `ready_for_integ_ci` leaves: `tools/oya-governance-*-app/` → `ci/facade/`.
