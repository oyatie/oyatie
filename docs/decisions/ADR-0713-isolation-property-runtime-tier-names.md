---
doc_status: published
id: ADR-0713
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
depends_on: []
related: [ADR-0338, ADR-0710, ADR-0711]
milestone: F1
deliverables:
  - id: ADR-0713-D1
    description: "Rename RuntimeClass / tier mechanism names to isolation-property nouns shared-kernel / private-kernel / private-kernel-attested. Keep trust classification 0..3 as a separate axis. Restore Tier-3 edge/perf placement (SR-IOV, hugepages, CPU pinning) as an orthogonal placement axis, not a fourth isolation tier."
    exit_criteria: "Founder Accept; ADR-0338 remains archived — only ADR-0701 is amendable. ADR-0710's statement that the tier MODEL stands and only the mechanism changes is preserved."
    verified_by: "oya-ci-required"
  - id: ADR-0713-D2
    description: "Migration sequencing: Kyverno→VAP (or live admission substrate) enforcement re-home MUST land BEFORE the rename. Legacy class names ride as deprecated aliases for exactly one wave. oya-governance-runtime-class-allowlist lane updates in the same rename PR or fail closed."
    exit_criteria: "Accept records the sequencing invariant; any rename PR without prior enforcement re-home is refused by review/gates."
    verified_by: "oya-ci-required"
---
# ADR-0713: Runtime tier rename to isolation-property names with orthogonal placement axis

## Status

**Proposed.** Deliberately not Accepted: clause D-2's migration sequencing waits on the
enforcement re-home precondition (Kyverno→VAP path under ADR-0710's proposed substrate, while
live law still carries ADR-0701's admission gist until ADR-0710 Accept). Renaming before that
re-home would strand enforcement on legacy names. This ADR carries **no implement authority**
while Proposed.

Discovery input (not law): Round-2 synthesis in the local planning artifact
the Round-2 node forever-shape Discovery plan (local artifact id e6ec1a68) (founder F1(c)).

## Context

Historical ADR-0338 defined pod runtime tiers `0..3` with RuntimeClass names and Kyverno
enforcement. ADR-0338 is **archived / Superseded** — its gist lives under ADR-0701. **Only
ADR-0701 is amendable**; this proposal amends ADR-0701, never resurrects ADR-0338 as a live
file.

ADR-0710 (Proposed) states that the tier **model** stands and only the admission **mechanism**
changes. Round-2 Discovery proposes renaming the *mechanism-facing class names* to isolation
properties so names say what the workload is isolated *from*, and restoring Tier-3's
edge/perf nodepool contract as an **orthogonal placement axis** rather than collapsing it into
a fourth isolation tier.

Live RuntimeClass names today include unbranded forms such as `runc`, `runc-edge`, and
`kata-cloud-hypervisor`. Vendor-branded isolation nouns remain a drift hazard (`kata` is itself
a vendor brand).

## Decision (proposed)

### D-1 — Isolation-property names + orthogonal placement

On Accept, mechanism-facing RuntimeClass names become:

| Isolation property | Meaning |
|---|---|
| `shared-kernel` | Workload shares the node guest kernel (process isolation) |
| `private-kernel` | Workload gets a private kernel (microVM / VMM path; Cloud Hypervisor) |
| `private-kernel-attested` | Private kernel plus relying-party attestation → identity/authz context |

**Orthogonal axes (do not collapse):**

1. **Trust classification 0..3** (who wrote the code) — remains; not renamed by this ADR.
2. **Isolation property** (shared-kernel / private-kernel / private-kernel-attested).
3. **Placement** (general vs edge-tuned hardware: SR-IOV, hugepages, CPU pinning) — restores
   historical Tier-3's nodepool contract without inventing a fourth isolation tier.

Pool binding (proposed; depends on ADR-0711 Accept): `shared-kernel` may land on Asterinas
pools; `private-kernel*` pin to KVM-capable stripped-Linux pools. VAP (or live substrate)
forbids mismatches.

Kata-as-a-bridge-component dissolves under the owned-shim shape (ADR-0712); Cloud Hypervisor
remains the VMM. That component dissolution is not a rename of trust tiers.

### D-2 — Migration sequencing (hard order)

On Accept, encode MUST follow this order:

1. **Enforcement re-home** (Kyverno ClusterPolicy / webhook path → VAP paramKind tier map, or
   the live admission substrate's equivalent) **BEFORE** any RuntimeClass rename.
2. **Rename** RuntimeClass names + every `pod_runtime_tier` / manifest consumer in one wave.
3. **Legacy aliases** for old class names remain as **deprecated aliases for exactly one wave**,
   then removed.
4. **`oya-governance-runtime-class-allowlist`** lane update lands in the **same PR** as the
   rename, or the PR fails closed.

No ban on unbranded names that already exist; the cost is migration + allowlist, not a strawman
rebrand fight.

## Consequences

- Positive: names match isolation physics; Tier-3 placement restored; ADR-0710 model clause
  respected; ADR-0338 stays archived.
- Negative: one-wave alias burden; allowlist lane coupling; blocked on admission re-home.
- Operational: fail-closed if allowlist drifts from renamed classes.

## Rejected alternatives (proposed framing)

| Option | Why not |
|---|---|
| Amend archived ADR-0338 file | Archived — only ADR-0701 amendable |
| Collapse trust 0..3 into isolation names | Loses who-wrote-the-code axis |
| Fourth isolation tier for edge/perf | Placement is orthogonal; Tier-3 restored that way |
| Rename before Kyverno→VAP re-home | Strands enforcement; D-2 forbids |

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept** | ADR-0701 mechanism-facing class names become the isolation-property set; migration sequencing binds encode PRs |
| **Reject** | Live class names and ADR-0701 carried gist unchanged |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`.
