# .grok/ reorg drain — Seat A (`integ/ci`)

**Wave:** `W-tools-swarm-to-grok`  
**Judgment:** `ready_for_port_to_grok` (`#1644@a469a8a8e`, envelopes **1.16.1** `roots.grok` → `integ/ci`)  
**Seat:** `integ/ci` (`.grok/**` via `roots.grok`)  
**Status:** **ABSORB DONE** — `tools/swarm/**` from `#1644` lineage ported to `.grok/swarm/**`.

## Completed (this rail)

- Rewrote `tools/swarm/**` (9 tracked files + `shim-bin/` symlinks) → `.grok/swarm/**`.
- Path cites inside the kit now reference `.grok/swarm/` (not `tools/swarm/`).
- Envelope authority: `destination_owns_reorg_now` absorb owner = `integ/ci`; `integ/specs` is envelopes hub only.

## Phase 2 (shrink-only — `integ/tools`, NOT this seat)

After `.grok/swarm/` verified alive on `integ/ci` tip:

1. `integ/tools` shrink-only delete `tools/swarm/**` (drain note @ `integ/tools@d0cb6fd2c`).
2. Same-wave cite rewrite on `integ/specs`: `claim_mechanical`, `anti_drift.drift_grep`, `one_shot_exceptions` paths `tools/swarm` → `.grok/swarm`.
3. Do **not** delete `tools/swarm/**` on `#1644` tip until Phase 2 — dual-home until shrink lands.

## Next leaf (NOT this slice)

- **`scripts/** → `ci/facade/**`** — next absorb candidate on this seat after swarm dual-home shrink lands.
- Do **not** port `scripts/**` in this commit; leave the tree untouched until a dedicated leaf brief opens.

## Verify

```bash
./.grok/swarm/self-check.sh
python3 ./.grok/swarm/claim_packet.py --self-test
```
