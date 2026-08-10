# Cloud path drain — Wave 2 order (integ/cloud)

Envelope: `integ/cloud` → `cloud/**` only. No hub JSON. No mass delete until absorb tips exist.

Judgment authority: `#1644` / `integ/specs` (envelopes); absorb receipt `integ/os`.

| Leaf | Forever dest | `land_status` | This tip (`integ/cloud`) | Blocker |
|------|--------------|---------------|--------------------------|---------|
| `cloud/cloud-kernel/` | `kernel/` | **DONE** — dual-home deleted after absorb | tree removed | Absorb tip `#1659` / `integ/kernel` verified; burn `92f8b727b` |
| `cloud/cloud-os/` | `os/` | **DONE** — residual deleted after absorb | tree removed (was residual `manifest.json`) | Absorb tip `integ/os` @ `9ffc6496f` (`os/manifest.json` + OWNERS + receipt); founder ACK lifted park; findings `oyatie-45t0`/#1925 still open |

## Order (do not invert)

1. **Kernel absorb** on owning integ → then delete `cloud/cloud-kernel/**` on `integ/cloud`. ✅
2. **OS absorb** after `#1643` / `integ/os` forever residual (`os/manifest.json` + dual-home OWNERS) → then delete `cloud/cloud-os/**` on `integ/cloud`. ✅ absorb then burn this tip.
3. Hub retargets (capability-registry / masterplan / automation excludes) only on tip-free `integ/specs` — harvest from `#1607`, never land `agent/` branch. Elevate: do not touch root hubs / Cargo.lock / Cargo.toml on this lane.

## Wave-2 residual verify (2026-08-10 post-absorb)

| Check | Result | Tip evidence |
|-------|--------|--------------|
| `cloud/cloud-kernel/**` on `integ/cloud` | **burned** (0 files) | `92f8b727b` |
| forever home `kernel/{core,harness}/` | **present** | `#1659` |
| `cloud/cloud-os/**` on `integ/cloud` | **burned** (0 files) | this tip |
| `os/manifest.json` on `integ/os` | **present** | `9ffc6496f` |
| Dual-home crate bodies clobbered? | **NO** | absorb copied OWNERS + manifest only; #1643 forever sources retained |
| Findings `oyatie-45t0` / #1925 | **still OPEN** | absorb/drain does not close |

## Elevate (other rails)

- Tip-free `integ/specs`: hub retarget harvest table (capability-registry / masterplan / tier / automation / crate-registration / slo-coverage) from `#1607` — **not this lane**.
- No merge of `#1644` / `#1661` from this drain.
- Root `Cargo.toml` exclude / reachability: owning rail only.

## Wave 2 receipt (this lane)

- Kernel absorb+burn: **DONE**.
- OS absorb+burn: **DONE** (park lifted by founder ACK; `oyatie-45t0` findings remain tracked).
- One writer on `integ/cloud`. No merge. No Cargo.lock. No hubs/NO_RAIL.
