---
id: ADR-0631
title: "A capability that spans strata has a wrong boundary, not a tier problem: split iam into iam (S1 PDP) and identity (S3 product), and rehome consent-graph to compliance"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-01
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0562, ADR-0280, ADR-0245]
amends: [ADR-0562]
related: [ADR-0615, ADR-0512, ADR-0132, ADR-0139]
milestone: W3
---

# ADR-0631: a capability that spans strata has a wrong boundary

## Status

**Proposed — 2026-08-01.** Amends the ADR-0562 closed capability registry. Landed Proposed, not
Accepted: a fresh `Accepted` status REDs cross-artifact-agreement until its evidence propagates.

## Context

PR #1481 restored tier enforcement to capability roots. It requires every root in `capability_roots`
to DECLARE `tier` + `substrate_dag_position.stratum` in `specs/capability-registry.json`, and makes
an undeclared capability RED and non-baselineable — deliberately, so a capability cannot buy silence
by staying undeclared. That is the correct fail direction and this ADR does not weaken it.

**Nine of twenty-four capabilities cannot declare a tier**, because their absorbed services disagree.
The hardest and most depended-on is `iam`, which absorbs seven directories:

    iam · cloud/cloud-iam · oya/identity · oya/oya-identity · oya/consent-graph ·
    oya/tenant-rbac · oya/oya-authn-device-firmware

`cloud/cloud-iam` and `oya/identity` are **S1**. `oya/consent-graph` is **S3**.

The unanimity rule was written to stop a plausible-looking tier being attached to an
under-enforced tree. It is doing exactly that here. The question this ADR answers is what the
disagreement MEANS.

## Decision

### D1 — a stratum span is a BOUNDARY DEFECT, not a tier defect

**If a capability's absorbed services span strata, the boundary is wrong.** A capability is
simultaneously a unit of ownership and a unit of dependency position; if its members sit at
different depths in the ADR-0280 DAG, it is not one capability.

The unanimity rule is therefore **a boundary detector**, not an obstacle to be routed around. The
remedy for a span is to SPLIT at the stratum seam, never to pick a tier that averages it.

Explicitly rejected: ranking a spanning capability by its FLOOR. That would let an S3 member inherit
S1's permissions and re-open the under-enforcement the tier gate exists to catch, with a
plausible-looking number attached.

### D2 — `iam` splits into `iam` (S1) and `identity` (S3)

| capability | tier | stratum | contents |
|---|---|---|---|
| `iam` | substrate | **S1** | PDP, principal verification, policy evaluation, tenant-RBAC role store + assignments |
| `identity` | substrate | **S3** | accounts, registration, passkeys, SCIM, profile, `oya-authn-device-firmware` |

`identity` depends on `iam`. Nothing depends on `identity` that may not.

**This restores a distinction the repo already had and the reorg erased.** The pre-reorg
`cloud/` vs `oya/` split encoded it: `cloud/cloud-iam` IS the IdP/PDP substrate, `oya/identity`
CONSUMES it. ADR-0562 absorbed both under one name because both are "identity" — a naming-driven
merge, not a capability boundary. The S1/S3 disagreement is that erasure surfacing.

**Hyperscaler precedent is unanimous on the separation.** AWS IAM is not Cognito — IAM is the
foundational control plane, Cognito is product identity built on it. Google IAM is not Identity
Platform. Azure separates the Entra directory/token service from B2C. Three independent
architectures reached the same seam.

**The dependency graph forces it independent of taste.** Capabilities at S0, S1 and S2 —
`observability`, `k8s`, `tenancy` — already depend on `iam`. Under ADR-0280 a substrate may not
depend on a higher S-rank, so a single `iam` at S3 inverts those edges immediately. `iam` must sit
at or below its lowest dependent. Splitting is the only resolution that does not either invert real
edges or attach S1 permissions to S3 material.

### D3 — `consent-graph` rehomes to `compliance` (S4), and the reason is RETENTION, not taxonomy

Consent is not an identity concern that happens to sit nearby. **IAM and consent have incompatible
data lifecycles**, and co-locating them makes a legal obligation unimplementable:

| | IAM | consent |
|---|---|---|
| question | may this principal do X on Y **now** | did this subject permit this **purpose**, on what notice, still valid |
| shape | point-in-time, recomputed per request | immutable history + notice snapshots + withdrawal events |
| history | not required | IS the artifact |

**The decisive property: consent records must OUTLIVE the identity they reference.** On withdrawal
the principal is destroyed and the consent + destruction evidence is RETAINED under legal hold —
PIPA §21③ separated retention, GDPR Art.17's carve-outs, and the "legal retention data from
withdrawn users" isolation tier in the reference model.

Held in one store those two retention policies are irreconcilable: you cannot crypto-shred the
principal's DEK and still hold verifiable consent evidence encrypted under it. **The legal
requirement forces the architectural boundary.**

**Split record from check.** `compliance` owns the immutable `consent_events` store. `iam`'s PDP MAY
consume a current-consent projection as an ABAC attribute, exactly as it consumes any other
principal attribute. The authz path stays fast and stateless; the record stays append-only under
legal hold. Consent then sits beside DPIA/RoPA, which is where the per-field data-classification
work is heading anyway.

### D4 — the remaining eight are ruled by applying D1, not case by case

For each: **does the absorbed set have a single FLOOR, with everything above it being a CONSUMER
rather than a member?** If yes, declare the floor. If no, split at the stratum seam.

Expect most seams to fall on the dissolved `cloud/` vs `oya/` line, because that split was a
runner/seller axis standing in for a dependency axis — right answer, wrong justification, which is
why ADR-0562 discarded it and inherited this problem.

Eight independent rulings would drift. One principle applied eight times is reviewable as a single
decision.

### D5 — D1 ships as a GATE, or it is prose

A principle with no mechanical acceptance check becomes another aspirational rule, and this repo has
19 gate crates that build, pass their own tests, and are invoked by nothing. **D1 is therefore
enforced, not documented:** the tier gate gains a span detector — a capability whose
`absorbs_current_dirs` resolve to more than one stratum is RED and non-baselineable, with the
detail naming the members and their strata.

Two properties the detector must have, both learned the hard way in this repo:

- **It keys on IDENTITY, not on a name.** The R6c fix keyed on the `capability:` catalog facet
  precisely so a rename could not dodge it; an `aspirational-enforcement` scan keyed on a name prefix
  was silently emptied by a rename the same week.
- **Zero observations is RED, never dormant-and-passing.** If the detector resolves no capability
  with a span, that is a broken scan until proven otherwise — it must fail closed on an empty
  corpus, not report clean.

### D6 — migration is a POINTER, never a copy, and carries a runnable rollback

**No dual-write and no duplicated crate.** A crate lives in `iam/` or `identity/`, never both, at
every commit. The old path is deleted by the same move plan that creates the new one — the codemod's
single-active-plan invariant makes a half-migrated state unrepresentable, and that is the point.
A "temporary" copy IS the dark wiring: it silently diverges and nothing detects it.

**Rollback, runnable rather than described:** revert the move-plan commit and re-run the
materializer. That works only while `identity/` has no external dependents; once a capability outside
the split imports `identity/*`, rollback becomes a rewrite. **The reversibility window closes at the
first external dependent** — record that commit, because it is when this door finishes closing.

**Terminal states for the migration itself**, so a run's outcome names the permitted next action
rather than collapsing into red/green:

| outcome | meaning | next action |
|---|---|---|
| `split-clean` | both roots declared, gate green, inversions baselined and COUNTED | proceed |
| `verified-empty` | reclassification surfaced ZERO inversions | **investigate — not success.** A span this old surfacing nothing means the scan did not run over the moved crates |
| `blocked` | a dependent outside the split already imports `identity/*` | rollback window closed; forward-fix only |
| `no-verdict` | the gate did not reach a verdict | re-run; do NOT read as pass |

## Consequences

- `specs/capability-registry.json` gains `identity`; the closed set goes 24 -> 25. The registry stays
  CLOSED — this is an amendment, not an opening.
- `iam`'s 63 crates split across two roots. Both need `capability_roots` entries and declared tiers.
- Reclassification WILL surface real S-rank inversions currently hidden by `iam` being unenforced.
  **Those are findings, not regressions.** Baseline them advisory and report the count; do not tune
  the split to make the number small. Precedent: the first five reclassified capabilities surfaced
  9 pre-existing inversions, three of which a prior move had recorded as "burned down" when the
  edges were never fixed.
- `consent-graph` moves to `compliance`, whose absorbed set must be re-checked for a stratum span
  under D1 before it accepts a member at S3.
- One-way door: the split changes the closed registry and the crate homes of 63 crates. Reversing it
  after dependents adopt `identity/` is expensive.

## Alternatives rejected

**Rank a spanning capability by its floor.** Simplest, and wrong: S3 material inherits S1's
permitted edges, which is precisely the under-enforcement the tier gate exists to catch.

**Let a capability span strata and rank each member.** Makes the capability meaningless as a
dependency unit and requires every gate keyed on capability to become member-keyed.

**Leave all nine RED until the reorg completes.** Honest, and the status quo — but tier enforcement
stays off for their crates, and `iam` is the most depended-on capability in the tree. The cost
compounds with every move that lands beneath it.

**Keep consent in `iam` and rank `iam` S3.** Inverts the existing `observability`/`k8s`/`tenancy`
edges, and leaves the retention conflict in D3 unresolved.

## Verification

1. `specs/capability-registry.json` declares `tier` + `substrate_dag_position.stratum` for BOTH
   `iam` and `identity`; neither is RED under R6c, and both strata are RANKABLE (present in
   `stratum_rank_order`, not `forward-declared`).
2. `buck2 test //ci/facade/layer-dependency-acyclicity:ci-layer-dependency-acyclicity-gate` is green,
   with every newly-surfaced inversion baselined ADVISORY and its count reported.
3. No edge from a capability at stratum <= S1 to `identity`.
4. `consent-graph` crates resolve under `compliance`; `iam` retains no consent-record store.
5. The eight remaining capabilities are each recorded as FLOOR-DECLARED or SPLIT under D1, with the
   evidence for each.
