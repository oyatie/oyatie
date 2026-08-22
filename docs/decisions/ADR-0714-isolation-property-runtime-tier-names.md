---
doc_status: published
id: ADR-0714
title: "Runtime tier rename to isolation-property names with orthogonal placement axis"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701]
amended_by: []
depends_on: [ADR-0712]
related: [ADR-0338, ADR-0710, ADR-0712, ADR-0715]
milestone: F1
masterplan_work_item: MPV2-0055
deliverables:
  - id: ADR-0714-D1
    description: "Rename RuntimeClass / tier mechanism names to isolation-property nouns shared-kernel / private-kernel / private-kernel-attested. Keep trust classification 0..3 as a separate axis. Restore Tier-3 edge/perf placement as an orthogonal placement axis. Publish deterministic pod_runtime_tier → axes migration table."
    exit_criteria: "Founder Accept; ADR-0338 remains archived — only ADR-0701 is amendable. ADR-0710's statement that the tier MODEL stands and only the mechanism changes is preserved. Migration table is complete for live 0..3 values."
    verified_by: "presubmit"
  - id: ADR-0714-D2
    description: "Migration sequencing: Kyverno→VAP (or live admission substrate) enforcement re-home MUST land BEFORE the rename. Legacy class names including live kata-cloud-hypervisor ride as deprecated aliases for exactly one wave. governance-runtime-class-allowlist lane updates in the same rename PR or fail closed."
    exit_criteria: "Accept records the sequencing invariant and one-wave alias law for kata-cloud-hypervisor; any rename PR without prior enforcement re-home is refused by review/gates."
    verified_by: "presubmit"
---
# ADR-0714: Isolation-property RuntimeClass names with orthogonal placement axis

## Status

**Proposed.** Deliberately not Accepted: clause D-2's migration sequencing waits on the
**enforcement re-home precondition** (Kyverno→VAP path under ADR-0710's proposed substrate, while
live law still carries ADR-0701's admission gist until ADR-0710 Accept / Reject per ADR-0715).
Renaming before that re-home would strand enforcement on legacy names. The gate is
outcome-determining: either re-home evidence lands (Accept path open) or it does not (rename
encode remains forbidden). This ADR carries **no implement authority** while Proposed.

Live masterplan anchor (planning only, not implement authority):
[`/specs/masterplan.json#masterplan_v2.work_items[MPV2-0055]`](../../specs/masterplan.json)
(F1(c) isolation-names package). Local Discovery artifact `e6ec1a68` is provenance only.

## Context

Historical ADR-0338 defined pod runtime tiers `0..3` with RuntimeClass names and Kyverno
enforcement. ADR-0338 is **archived / Superseded** — its gist lives under ADR-0701. **Only
ADR-0701 is amendable**; this proposal amends ADR-0701, never resurrects ADR-0338 as a live
file.

ADR-0710 (Proposed) states that the tier **model** stands and only the admission **mechanism**
changes. Round-2/4 Discovery proposes renaming the *mechanism-facing class names* to isolation
properties so names say what the workload is isolated *from*, and restoring Tier-3's
edge/perf nodepool contract as an **orthogonal placement axis**.

Live corpus today uses `pod_runtime_tier` values `0`/`1` for Kata-class isolation
(`kata-cloud-hypervisor`), `2` for runc shared-kernel, and `3` for edge runc (`runc-edge`).
Vendor-branded isolation nouns remain a drift hazard.

Bominal inheritance: no Bominal equivalent — oyatie override for isolation-property naming.

## Decision

### D-1 — Isolation-property names + orthogonal placement

On Accept, mechanism-facing RuntimeClass names become:

| Isolation property | Meaning |
|---|---|
| `shared-kernel` | Workload shares the node guest kernel (process isolation) |
| `private-kernel` | Workload gets a private kernel (microVM / VMM path; Cloud Hypervisor) |
| `private-kernel-attested` | Private kernel plus relying-party attestation → identity/authz context |

**Day-1 attested semantics** (aligned with [ADR-0712](ADR-0712-node-kernel-pool-matrix.md) D-3):
`private-kernel-attested` is **attested-identity** (host in TCB, explicitly labeled).
**Operator-excluded confidentiality** (guest-pull) is the **F1 Isolation target**, not a day-1
claim.

**Orthogonal axes (do not collapse):**

1. **Trust classification 0..3** (who wrote the code) — remains; not renamed by this ADR.
2. **Isolation property** (`shared-kernel` / `private-kernel` / `private-kernel-attested`).
3. **Placement** (general vs edge-tuned hardware: SR-IOV, hugepages, CPU pinning) — restores
   historical Tier-3's nodepool contract without inventing a fourth isolation tier.

Pool binding (proposed; depends on ADR-0712 Accept of **D-1** specifically): `shared-kernel`
may land on Asterinas pools only after A1 is green **and** founder Accepts **D-1** (the
two-SKU co-selection). Accept of **G5** keeps Asterinas soak-only permanently and MUST NOT
authorize Asterinas production placement. Interim Linux-primary per ADR-0712.
`private-kernel*` pin to KVM-capable stripped-Linux pools; `private-kernel-attested` requires
attestation-capable pools.

### D-1a — Deterministic `pod_runtime_tier` → axes migration table

On Accept, encoders MUST apply this complete mapping for live `0..3` values. No silent
defaults; admission MUST validate the migrated fields.

| Legacy `pod_runtime_tier` | Live RuntimeClass (today) | Isolation property | Placement | Trust classification | Notes |
|---|---|---|---|---|---|
| `0` | `kata-cloud-hypervisor` (Kata-class) | `private-kernel` | `general` | unchanged (keep existing trust 0..3 field) | Same Kata isolation class as `1`; does **not** auto-upgrade to `private-kernel-attested` |
| `0` | **no-runtime / scaffold sentinel** (explicit non-pod accounting) | _(no isolation property)_ | _(n/a)_ | unchanged | Preserve the sentinel — do **not** invent a RuntimeClass or private-kernel claim. Corpus includes `cloud/cloud-os/manifest.json` and `cloud/cloud-kernel/manifest.json` where `pod_runtime_tier: 0` is reserved for scaffold accounting with no pod runtime |
| `1` | `kata-cloud-hypervisor` (Kata-class) | `private-kernel` | `general` | unchanged | Same isolation as `0`; trust axis stays separate |
| `2` | `runc` | `shared-kernel` | `general` | unchanged | First-party shared-kernel |
| `3` | `runc-edge` **or** explicit edge nodepool evidence | `shared-kernel` | `edge` | unchanged | Placement carries former Tier-3 edge/perf contract (SR-IOV, hugepages, CPU pinning) **only** when RuntimeClass / pool evidence is edge |
| `3` | ADR-0083 control-plane / kernel classification (no edge RuntimeClass) | `shared-kernel` | `general` | unchanged | **Do not** auto-assign `edge` + SR-IOV/hugepages/pinning. Corpus outliers include `k8s/managed-tenant-quota`, `k8s/managed-sla-observability`, `k8s/managed-cluster-lifecycle` (`pod_runtime_tier: 3` for control-plane/kernel, not edge workloads) |

**Placement derivation rule for legacy `3`:** `edge` placement requires **positive edge evidence**
(`runtimeClassName: runc-edge`, edge nodepool labels/selectors, or an explicit edge placement
field already present). Integer `3` alone is **insufficient** — encoders MUST audit the Tier-3
corpus and keep control-plane/kernel records on `general` placement.

**No-runtime sentinel rule for legacy `0`:** when a manifest explicitly records that
`pod_runtime_tier: 0` is scaffold/accounting only (no pod RuntimeClass / no workload runtime),
encoders MUST preserve that sentinel and MUST NOT emit `private-kernel` isolation or schedule
onto private-kernel pools.

**`private-kernel-attested` has no legacy `pod_runtime_tier` preimage.** It is introduced only by
explicit manifest/RuntimeClass selection after Accept, and only onto attestation-capable pools
(ADR-0712 D-3). Encoders MUST NOT map `0` or `1` to attested by implication.

Kata-as-a-bridge-component dissolution under the owned-shim shape is [ADR-0713](ADR-0713-node-substrate-architecture.md)
Accept (a); Cloud Hypervisor remains the VMM. That component dissolution is not a rename of
trust tiers.

### D-2 — Migration sequencing (hard order) + one-wave alias law

On Accept, encode MUST follow this order:

1. **Enforcement re-home** (Kyverno ClusterPolicy / webhook path → VAP paramKind tier map, or
   the live admission substrate's equivalent) **BEFORE** any RuntimeClass rename.
2. **Rename** RuntimeClass names + every `pod_runtime_tier` / manifest consumer in one wave,
   applying D-1a. The rename encode wave is the Accept-encode landing of masterplan work item
   **`MPV2-0055`** (machine-readable id; not prose-only).
3. **Legacy aliases** for old class names remain as **deprecated aliases for exactly one
   contiguous masterplan `execution_waves` index after the rename encode lands**, then MUST be
   removed by a **blocking removal transition** (allowlist + RuntimeClass alias deletion) in the
   **next** contiguous wave — or fail closed. This **includes** the live
   `kata-cloud-hypervisor` RuntimeClass name — it MUST alias to `private-kernel` under that
   lifetime. Lifetime contract is also recorded on
   `masterplan_v2.work_items[MPV2-0055].runtimeclass_alias_lifetime` (planning field while
   Proposed). The blocking alias-lifetime evaluator is **not** claimed as live while this ADR
   is Proposed; it MUST be implemented and wired into `presubmit` in the Accept-encode PR
   that introduces the alias — recording a prospective evaluator name alone is insufficient.
4. **`governance-runtime-class-allowlist`** lane update lands in the **same PR** as the
   rename, or the PR fails closed.

No ban on unbranded names that already exist; the cost is migration + allowlist, not a strawman
rebrand fight.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | Notes |
|---|---|---|
| RuntimeClass manifests / Helm `runtimeClassName` | update | One-wave rename + `kata-cloud-hypervisor` alias |
| `*/manifest.json` `pod_runtime_tier` consumers | update | Apply D-1a; add isolation + placement fields as sequenced |
| `governance-runtime-class-allowlist` | update | Same PR as rename |
| Admission VAP / Kyverno policies | update | Re-home before rename |

### Integration via Workflow + Ontology

Not applicable — naming/admission mechanism decision. Consumed by workload admission µservices
after Accept.

### Positive

- Names match isolation physics; Tier-3 placement restored; deterministic migration table.
- ADR-0710 model clause respected; ADR-0338 stays archived.

### Negative

- One-wave alias burden; allowlist lane coupling; blocked on admission re-home.

### Operational

- Fail-closed if allowlist drifts from renamed classes.
- CI: `presubmit`; no authority-surface citation while Proposed.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none |
| `cross-product-refusal` (LEAN-A2) | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |

No new port traits.

## Alternatives considered

**Alternative 1 — Amend archived ADR-0338 file**
- Pros: keeps historical file "alive".
- Cons: archived — only ADR-0701 amendable.
- Reason rejected.

**Alternative 2 — Collapse trust 0..3 into isolation names**
- Pros: fewer axes.
- Cons: loses who-wrote-the-code axis.
- Reason rejected.

**Alternative 3 — Fourth isolation tier for edge/perf**
- Pros: preserves single integer.
- Cons: placement is orthogonal; Tier-3 restored that way.
- Reason rejected.

**Alternative 4 — Map tier `0`/`1` to `private-kernel-attested`**
- Pros: fewer explicit opts-in.
- Cons: live Kata class is not attested-identity; would falsely claim attestation.
- Reason rejected: D-1a.

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept** | ADR-0701 mechanism-facing class names become the isolation-property set; D-1a + D-2 bind encode PRs |
| **Reject** | Live class names and ADR-0701 carried gist unchanged |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`.

## References

- Live masterplan: `MPV2-0055` in `/specs/masterplan.json#masterplan_v2.work_items`
- ADR-0338 (archived), ADR-0701 (amendable), ADR-0710 (tier model stands), ADR-0712 (pools),
  ADR-0715 (F1 Admission package / 0710 Accept gate)
- Round-2 Discovery local artifact `e6ec1a68` — provenance only
- PR #1929 Round-4 amend (renumbered from draft ADR-0713)
