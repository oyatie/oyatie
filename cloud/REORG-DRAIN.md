# Cloud path drain — Wave 2 order (integ/cloud)

Envelope: `integ/cloud` → `cloud/**` only. No hub JSON. No mass delete until absorb tips exist.

Judgment authority: `#1644` / `integ/specs` tip `2adc2f038` (envelopes `1.16.2`).

| Leaf | Forever dest | `land_status` | This tip (`integ/cloud`) | Blocker |
|------|--------------|---------------|--------------------------|---------|
| `cloud/cloud-kernel/` | `kernel/` | **DONE** — dual-home deleted after absorb | tree removed (was 170 tracked files) | Absorb tip `#1659` / `integ/kernel` `674ffa4d70` + elevate `45c1e5860` verified (`kernel/{core,harness}/` present); burn on this rail `92f8b727b` |
| `cloud/cloud-os/` | `os/` | `ready_for_integ_os` | residual `manifest.json` only (0 crates here; `origin/dev` still has 4 dual-home crate roots) | **PARK/IDLE** until `#1643` lands — see residual verify + prep checklist below |

## Order (do not invert)

1. **Kernel absorb** on owning integ → then delete `cloud/cloud-kernel/**` on `integ/cloud`. ✅ absorb `#1659` tip `674ffa4d70` → delete DONE tip `92f8b727b`.
2. **OS absorb** after `#1643` / `integ/os` forever residual (`os/manifest.json` + dual-home reconcile) → then delete `cloud/cloud-os/**` on `integ/cloud`. **PARKED** behind `#1643` land — no `os/**` writer until hot slot clears.
3. Hub retargets (capability-registry / masterplan / automation excludes) only on tip-free `integ/specs` — harvest from `#1607`, never land `agent/` branch. Elevate: do not touch root hubs / Cargo.lock / Cargo.toml on this lane.

## Wave-2 residual verify (2026-08-10)

| Check | Result | Tip evidence |
|-------|--------|--------------|
| `cloud/cloud-kernel/**` on `integ/cloud` | **burned** (0 files) | `92f8b727b` |
| forever home `kernel/{core,harness}/` | **present** | `#1659` / `integ/kernel` `45c1e5860` (absorb `674ffa4d70`) |
| `cloud/cloud-os/**` on `integ/cloud` | residual `manifest.json` only | this tip |
| `os/manifest.json` on `integ/os` | **MISSING** | `#1643@4d6795b04` |
| Disjoint `os/**` absorb safe now? | **NO** | `#1643` already edits all 4 dual-home forever paths (`os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain/**`); sole live `Cargo.lock` writer; tip HOLD (green CI, mergeState BLOCKED, no APPROVE) |
| `#1607` | **CLOSED** — do not reopen | harvest patch list only (below) |

## cloud-os PARK — why not absorb now

- Forever dest pieces live under `os/**` (`integ/os` envelope), not `cloud/**`.
- Required absorb surface `os/manifest.json` is absent on both `origin/dev` and `#1643` tip.
- Dual-home crate bytes still on `origin/dev` under `cloud/cloud-os/crates/oya-cloud-os-{cluster-mgmt,kubernetes,secrets,trustd}-domain/` differ from forever `os/core/*` (rename + content drift) — reconcile must land on `integ/os` after `#1643`, not fight tip `4d6795b04`.
- No envelope-disjoint path under `cloud/**` completes the absorb; delete of residual `manifest.json` alone would orphan the dual-home crates still on trunk until a post-land absorb lands.

## Post-`#1643` prep checklist (integ/os then integ/cloud)

Do **not** start until `#1643` is landed (or an explicit tip-free `integ/os` writer window is granted). No `#1607` reopen. No root `Cargo.lock`.

1. **On `integ/os` (absorb):**
   - [ ] Copy harvest `os/manifest.json` from `#1607@83a12f532` (already retargets `destination_path` / `owned_root` → `os/`); re-point crate names to forever `os-*` / path inventory as live tree requires.
   - [ ] Reconcile dual-home drift for the 4 `origin/dev` crate roots vs `os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain` (Chesterton: tip already advanced forever homes — absorb bytes carefully; do not clobber `#1643` land).
   - [ ] Record absorb receipt under `os/**` evidence (mirror `kernel/evidence/wave2-cloud-kernel-absorb.md` pattern).
   - [ ] Local buck2 scoped to touched `//os/...` targets; no hub JSON; no root lock.
2. **On `integ/cloud` (burn):**
   - [ ] After absorb receipt exists: delete `cloud/cloud-os/**` (including residual `manifest.json`).
   - [ ] Refresh this `REORG-DRAIN.md` row to DONE.
3. **Elevate (other rails):**
   - [ ] Tip-free `integ/specs`: hub retarget harvest table below.
   - [ ] Root `Cargo.toml` exclude / reachability: owning rail only (not this lane).

## Supersede `#1607` (CLOSED — NO HARVEST merge)

- PR **#1607** `agent/reorg-cloud-os-residual-20260806` — **CLOSED** 2026-08-10; do **not** reopen or land.
- Zero-crate residual claim is **FALSE** (Wave-1 judgment + live tree: 4 dual-home crate roots remain on `origin/dev`).
- Harvest intent only onto envelope-bounded integ rails — **patch list for tip-free `integ/specs` hub retarget** (do not hub-edit from this lane):

| #1607 path | Harvest action | Owning integ |
|------------|----------------|--------------|
| `os/manifest.json` | absorb forever residual | `integ/os` (post-`#1643`) |
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

- Kernel absorb+burn: **DONE** (`integ/kernel` + `integ/cloud`).
- `cloud/cloud-os/**`: **PARK/IDLE** — blocker `#1643@4d6795b04` (TIP-GREEN HOLD; no APPROVE; dual-home forever paths not free).
- One writer on `integ/cloud`. No merge. No Cargo.lock. No `#1607` reopen.
