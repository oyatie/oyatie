# Cloud path drain — Wave 2 order (integ/cloud)

Envelope: `integ/cloud` → `cloud/**` only. No hub JSON. No mass delete until absorb tips exist.

Judgment authority: `#1644` / `integ/specs` tip `2adc2f038` (envelopes `1.16.2`).

| Leaf | Forever dest | `land_status` | This tip (`integ/cloud`) | Blocker |
|------|--------------|---------------|--------------------------|---------|
| `cloud/cloud-kernel/` | `kernel/` | **DONE** — dual-home deleted after absorb | tree removed (was 170 tracked files) | Absorb tip `#1659` / `integ/kernel` `674ffa4d70c261f2502336c25b8cffc5471b424c` verified (`kernel/{core,harness}/` present) |
| `cloud/cloud-os/` | `os/` | `ready_for_integ_os` | residual `manifest.json` only (0 crates here; origin/dev still has 4 crate roots / judgment cites 56 crate paths) | No `os/manifest.json` absorb on `integ/os` yet (#1643 coordination) — **park delete** |

## Order (do not invert)

1. **Kernel absorb** on owning integ → then delete `cloud/cloud-kernel/**` on `integ/cloud`. ✅ absorb `#1659` tip `674ffa4d70` → delete DONE tip `92f8b727b`.
2. **OS absorb** after `#1643` / `integ/os` forever residual (`os/manifest.json` + missing crate bytes) → then delete `cloud/cloud-os/**` on `integ/cloud`. **PARKED** behind `#1643` land — no `os/**` writer until hot slot clears.
3. Hub retargets (capability-registry / masterplan / automation excludes) only on tip-free `integ/specs` — harvest from `#1607`, never land `agent/` branch. Elevate: do not touch root hubs / Cargo.lock / Cargo.toml on this lane.

## Supersede `#1607` (CLOSED — NO HARVEST merge)

- PR **#1607** `agent/reorg-cloud-os-residual-20260806` — **CLOSED** 2026-08-10; do **not** reopen or land.
- Zero-crate residual claim is **FALSE** (Wave-1 judgment + live tree).
- Harvest intent only onto envelope-bounded integ rails — **patch list for tip-free `integ/specs` hub retarget** (do not hub-edit from this lane):

| #1607 path | Harvest action | Owning integ |
|------------|----------------|--------------|
| `os/manifest.json` | absorb forever residual | `integ/os` (#1643) |
| `specs/capability-registry.json` | retarget cloud-os → os/ | `integ/specs` (tip-free) |
| `specs/masterplan.json` | retarget cloud-os residual | `integ/specs` (tip-free) |
| `specs/microservice-tier-classification.json` | retarget cloud-os entries | `integ/specs` (tip-free) |
| `ci/facade/automation-language-policy/rust-first-automation-policy.json` | exclude cloud-os paths | `integ/specs` (tip-free) |
| `ci/facade/automation-language-policy/tests/rust_first_automation_hygiene.rs` | exclude cloud-os paths | `integ/specs` (tip-free) |
| `ci/facade/crate-registration/src/tests.rs` | retarget cloud-os crate refs | `integ/specs` (tip-free) |
| `ci/facade/cross-artifact-agreement/src/lib.rs` | retarget cloud-os refs | `integ/specs` (tip-free) |
| `ci/facade/slo-coverage/tests/slo_coverage.rs` | retarget cloud-os SLO paths | `integ/specs` (tip-free) |
| `evidence/reorg/rr-dual-cloud-os-residual-20260806.json` | evidence receipt only | `integ/specs` or `integ/docs` |

## Wave 2 receipt (this lane)

- `integ/cloud` tip: burn `cloud/cloud-kernel/**` after verified kernel absorb (`92f8b727b`).
- `cloud/cloud-os/**` remains parked (`manifest.json` residual only).
- One writer on `integ/cloud`. No merge. No Cargo.lock.
