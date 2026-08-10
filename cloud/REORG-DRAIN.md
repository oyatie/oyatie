# Cloud path drain — Wave 2 order (integ/cloud)

Envelope: `integ/cloud` → `cloud/**` only. No hub JSON. No mass delete until absorb tips exist.

Judgment authority: `#1644` / `integ/specs` tip `9a1037f13` (envelopes `1.13.1`).

| Leaf | Forever dest | `land_status` | This tip (`integ/cloud` @ base `b97b54315`) | Blocker |
|------|--------------|---------------|---------------------------------------------|---------|
| `cloud/cloud-kernel/` | `kernel/` | `ready_for_integ_kernel` | 21 `Cargo.toml` still present | No `integ/kernel` absorb tip yet — **park delete** |
| `cloud/cloud-os/` | `os/` | `ready_for_integ_os` | residual `manifest.json` only (0 crates here; origin/dev still has 4 crate roots / judgment cites 56 crate paths) | No `os/manifest.json` absorb on `integ/os` yet (#1643 coordination) — **park delete** |

## Order (do not invert)

1. **Kernel absorb** on owning integ → then delete `cloud/cloud-kernel/**` on `integ/cloud`.
2. **OS absorb** after `#1643` / `integ/os` forever residual (`os/manifest.json` + missing crate bytes) → then delete `cloud/cloud-os/**` on `integ/cloud`.
3. Hub retargets (capability-registry / masterplan / automation excludes) only on tip-free `integ/specs` — harvest from `#1607`, never land `agent/` branch.

## Supersede `#1607`

- Do **not** land `agent/reorg-cloud-os-residual-20260806`.
- Zero-crate residual claim is **FALSE** (judgment + live tree).
- Harvest intent only onto envelope-bounded integ rails above.

## This commit

Evidence note only. Deletes intentionally parked. One writer on `integ/cloud`.
