# Cloud path drain — Wave 2 order (integ/cloud)

Envelope: `integ/cloud` → `cloud/**` only. No hub JSON. No mass delete until absorb tips exist.

Judgment authority: `#1644` / `integ/specs` tip `9a1037f13` (envelopes `1.13.1`).

| Leaf | Forever dest | `land_status` | This tip (`integ/cloud`) | Blocker |
|------|--------------|---------------|--------------------------|---------|
| `cloud/cloud-kernel/` | `kernel/` | **DONE** — dual-home deleted after absorb | tree removed (was 170 tracked files) | Absorb tip `#1659` / `integ/kernel` `674ffa4d70c261f2502336c25b8cffc5471b424c` verified (`kernel/{core,harness}/` present) |
| `cloud/cloud-os/` | `os/` | `ready_for_integ_os` | residual `manifest.json` only (0 crates here; origin/dev still has 4 crate roots / judgment cites 56 crate paths) | No `os/manifest.json` absorb on `integ/os` yet (#1643 coordination) — **park delete** |

## Order (do not invert)

1. **Kernel absorb** on owning integ → then delete `cloud/cloud-kernel/**` on `integ/cloud`. ✅ absorb `#1659` tip `674ffa4d70` → delete DONE this tip.
2. **OS absorb** after `#1643` / `integ/os` forever residual (`os/manifest.json` + missing crate bytes) → then delete `cloud/cloud-os/**` on `integ/cloud`. **PARKED.**
3. Hub retargets (capability-registry / masterplan / automation excludes) only on tip-free `integ/specs` — harvest from `#1607`, never land `agent/` branch. Elevate: do not touch root hubs / Cargo.lock / Cargo.toml on this lane.

## Supersede `#1607`

- Do **not** land `agent/reorg-cloud-os-residual-20260806`.
- Zero-crate residual claim is **FALSE** (judgment + live tree).
- Harvest intent only onto envelope-bounded integ rails above.

## This commit

Burn `cloud/cloud-kernel/**` dual-home after verified kernel absorb. `cloud/cloud-os/**` remains parked. One writer on `integ/cloud`. No merge.
