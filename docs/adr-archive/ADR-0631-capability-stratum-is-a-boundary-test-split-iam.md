---
id: ADR-0631
title: "A capability that spans strata has a wrong boundary, not a tier problem: split iam into iam (S1 PDP) and identity (S3 product), consolidate the Cedar+ReBAC decision plane into policy, and re-derive the burn-down record"
status: Superseded
doc_status: published
planning_impact: true
deciders: founder
date: 2026-08-01
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0562, ADR-0280, ADR-0245]
amends: [ADR-0562]
related: [ADR-0615, ADR-0512, ADR-0132, ADR-0139]
milestone: W3
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


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

### D7 — the authorization DECISION PLANE consolidates into `policy/`; `iam` keeps the IdP

`policy/` contains **zero files** while its material sits in `iam/**` and `libs/**`. That is not a
tier question. Applying D1: `iam` cannot be S1 while holding S3 decision-plane material, so this is
the other half of the same span D2 addresses — moving `consent-graph` alone does not resolve it.

**The registry already declares this boundary; only the tree contradicts it.** `policy`'s charter
reads *"The Cedar-backed PBAC+ReBAC authorization decision plane (**standalone**, cell-distributed)"*
while `iam`'s reads *"Produces the verified principal."* This decision makes the tree match the
registry, it does not invent a new seam.

**It is two engines, not a Cedar feature of IAM.** The plane is an owned merge of Cedar (PBAC —
"does policy permit this action given these attributes") and Zanzibar-style ReBAC ("is there a
relationship path from this subject to this object"). Neither subsumes the other, which is why AWS
ships Verified Permissions separately from IAM, Azure ships Azure Policy separately from Entra, and
Google ships the Zanzibar/Check path separately from IAM. Three independent architectures put the
decision plane outside the identity service. The ReBAC half is real, not aspirational:
`policy-cedar-domain/src/rebac.rs` + `tests/rebac_tuple_port.rs`.

A PDP and an IdP also differ on every axis that defines a capability boundary: change cadence
(policies churn, identity schema does not), owner (security/compliance vs platform), scaling profile
(read-heavy hot path vs write-sensitive), and blast radius (a bad policy denies one action; a bad IdP
change locks everyone out). Different cadence and owner is the Conway test; different scaling profile
is the independent-deployability test.

**MOVE (7):** `iam/core/policy-cedar-domain` · `iam/core/cloud-pdp-kernel` · `iam/adapters/pdp-cedar`
· `iam/adapters/cloud-pdp-bundle-file` · `iam/ports/policy-cedar-api` · `iam/facade/cloud-pdp-app`
· `libs/oya-shared-pdp-kernel`.

**DO NOT MOVE — the per-capability PEP adapters STAY** (`k8s/adapters/*-cedar`,
`tenancy/adapters/tenant-lifecycle-authz-pdp`, `intelligence/adapters/authz-cedar-adapter`,
`oya/ci-webhook-gateway/**/authz-cedar-adapter`). A Policy ENFORCEMENT Point belongs with the
capability it protects; only the DECISION point centralizes. Collapsing PEPs into `policy/` would
invert the dependency and recreate the monolith this splits.

**MISFILED BY NAME, and NOT to `policy` (2):** `iam/core/tenant-rbac-tenant-admission-policy` and
`iam/ports/tenant-rbac-tenant-egress-policy-contract` describe themselves as **review-only** crates
defining *"Kubernetes admission guardrails"* and *"network egress guardrails"*. That is `k8s` and
`network` material; the token `policy` in their names made them look like decision-plane crates.
Dispositioned separately — do not sweep them into this move.

**ARGUABLE (1):** `iam/adapters/identity-workload-authz-cedar` evaluates an `AuthorizationRequest`
(decision-plane work) but is scoped to workload identity. Decide by **which port it implements** — an
adapter belongs with the capability owning its port — not by its name.

### D8 — `policy` is itself two strata: the G face and the C0 face

The canonical DAG (`specs/substrate-dependency-dag.json`) declares `policy-engine` depending on
`cell`(S2), `identity` and `tenancy`(S2), while `audit-chain`(S0) and `observability`(S0) depend on
`policy-engine`. Applying D1's arithmetic: `max(deps)=S2 <= rank <= min(dependents)=S0` — an **EMPTY
RANGE**. The SSOT contains an inversion.

**The charter explains the contradiction and names the fix.** PEPs are specified to consume a
*last-known-good signed snapshot*, under an explicit static-stability invariant: *"a stale snapshot
denies or routes to the authoritative shard, never silently authorizes."* But the DAG encodes
`permit-audit-chain-call-policy-engine-v1` — a runtime **CALL**. A data plane that calls its control
plane on the hot path is the availability anti-pattern static stability exists to prevent.

So `policy` splits along the faces its own charter already names:
- **G face** — policy + ReBAC-tuple authoring / signing / distribution. Control plane. Depends on
  `cell`/`tenancy`, so it sits ABOVE them (~S3).
- **C0 face** — per-cell runtime PDP + versioned snapshot store. Data plane. Consumed by every PEP,
  so it must sit at or below its lowest consumer.

The G→C0 relationship is **snapshot publication, not a call**, and therefore not a build-graph edge.
Encoding that removes the inversion without weakening any rule — which is the test D1 sets for a
correct split, as against picking a number that makes the gate quiet.

### D9 — the pre-#1481 burn-down record is not audited; it is RE-DERIVED

Three recorded burn-downs were proven to be relocations rather than repairs (the gate's own test
recorded 3 `cloud-kms -> residency` inversions as *"burned down by move-19"* when the edges were
never fixed — a move relocated one endpoint into an unenforced root and the violation stopped being
computed). ~19 capabilities' records were produced by the same mechanism, now closed by #1481.

**Those records will NOT be individually audited.** The burn-down record is a derived claim about
history; what matters operationally is the CURRENT violation set, which #1481 now computes correctly.
So: finish the declarations, re-run the gate over the fully-reclassified tree, and take the resulting
violation set as ground truth. Every real inversion appears; every false "burned down" claim resurfaces
as a live violation. **The audit is a byproduct of enforcement rather than a project.**

This is strictly better than auditing nineteen records on four counts: it is mechanical rather than
archaeological; it yields an actionable list instead of a historical verdict; it is self-correcting,
because the re-derivation need not be trusted the way an auditor would; and auditing history tells you
WHO was wrong while re-deriving tells you WHAT to fix. The same reason one rebuilds a corrupted index
instead of auditing what the index used to say.

**Sequencing consequence:** re-derivation is complete only over ENFORCED roots, so it is valid only
after every capability is declared. That is a reason to finish the declarations first, not a caveat
against the method.

**One thing IS worth doing:** record in `tier-dependency-acyclicity-baseline.json` that pre-#1481
burn-down claims are unreliable. Not to verify them — to stop a future reader citing them as evidence.
A record known to be unreliable and marked so is safe; one merely known to be unreliable is not.

### D10 — placement must be MECHANICAL, and today it is inferred from names

Everything above was expensive because **a crate's correct home is currently decided by reading it**.
That is the root cause, not a side effect, and the evidence is all from this one exercise:

- `tenant-rbac-tenant-admission-policy` and `tenant-rbac-tenant-egress-policy-contract` read as
  decision-plane crates. Their module docs say **Kubernetes admission** and **network egress**
  guardrails. The token `policy` in the NAME was wrong about the CONTENT.
- `oya-check-cost-budget` reads as a gate. It is a runtime budget-ledger consumed by production code;
  moving it to a gate root would have inverted product -> ci.
- `identity-workload-authz-cedar` cannot be classified from its name at all — it needs a port check.
- 12 crates were nearly deleted as dead; six were `status: active` with ADR mandates.

A name is a lossy encoding of a decision nobody recorded.

**The hyperscaler answer is not a smarter linter — it is the BUILD SYSTEM.** In google3 the
directory IS the declaration (one BUILD per package) and `visibility` is the enforcement; nobody
asks whether a target "belongs" somewhere, because a target that reaches outside its allowed view
does not build. Buck2 carries the same primitives plus `within_view`, the downward constraint.

That distinction is load-bearing here, not stylistic. **A gate keyed on names is exactly what a
rename silently emptied this week** — the `aspirational-enforcement` scan went to zero sites and
reported clean. A `visibility`/`within_view` constraint has **no probe to get wrong**: it is
evaluated by buck2 itself, and it fails at BUILD-FILE EVALUATION — earlier than analysis, before any
test runs. Verified in-tree 2026-07-31: a forbidden dep yields
`Target's within_view attribute does not allow dependency ...`.

So placement is DECLARED at birth and ENFORCED BY THE GRAPH, never re-inferred:

1. **Capability boundaries become `visibility` declarations.** A capability's `core/` is visible to
   its own `ports/`/`adapters/`/`facade/` and to nothing else; `ports/` is the only face with
   cross-capability visibility. That single rule makes "which capability owns this crate?"
   answerable by whether it links — the ports-and-adapters seam expressed in the build graph rather
   than in prose.
2. **`within_view` pins the DAG direction per stratum.** A capability at S_n may view S_n and below.
   That is ADR-0280's rule, enforced by the build system instead of re-derived by a gate from a
   crate-graph scan.
3. **The catalog row records the answer so it survives renames.** `register_crate` already makes a
   crate born-accounted with a `registry/catalog/<crate_id>.yaml` row; that row — not the path, not
   the name — carries the owning capability. Precedent: R6c and the aspirational-enforcement fix both
   key on the catalog `capability:` facet **precisely so a rename cannot change the answer**.
4. **Each face then has ONE mechanical test**, and none of them requires reading the crate:
   - `adapters/` — owned by the capability owning the PORT it implements. Read off the dep edge.
   - `core/` — its non-`libs` dependencies must not leave its capability. Read off the dep set.
   - `ports/` — owned by the capability whose seam it defines; implementors may live anywhere.
   - `facade/` — owned by the capability whose surface it sells.

**A crate whose declared capability disagrees with its visibility-permitted dependency set does not
build.** That is strictly stronger than a gate, because a gate can be emptied, skipped, or left
unwired — this repo found 19 gates in exactly that state — whereas a build constraint cannot be
satisfied while violated. Every item in the list above would have failed at the moment it landed.

**Migration is incremental and non-breaking**: `visibility` defaults are permissive, so tightening
one capability at a time surfaces its real violations without a big-bang cutover. Start with a
capability whose boundary is already clean (`audit` or `secrets`, both S0 with small absorbed sets)
to validate the pattern before touching `iam`.

**Run the check at CREATION, not at tier-declaration time.** Seven of the eight untiered capabilities
turned out to be derivable, and both genuine "spans" were single misplaced crates — so the unanimity
rule has been doing per-crate boundary detection nobody asked it for, years after the fact. Catching a
misplaced crate the day it lands is a one-line move; catching it at tier-declaration means it has
already accumulated dependents and the fix is a migration.

The goal is that "where does this crate go?" has a mechanical answer derivable from what the crate
DEPENDS ON and what DEPENDS ON IT — with the catalog row recording the answer so it survives renames,
and the gate failing closed when the two disagree.


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

### From D7-D10

- `policy/` stops being an empty registered root. 7 crates move in (6 from `iam/**`, 1 from
  `libs/**`); the ~8 per-capability PEP adapters stay where they are.
- **`iam` shrinks twice** — D2 removes product identity, D7 removes the decision plane. What remains
  is the IdP plus the tenant-RBAC role store, which is what its charter already claims.
- The `policy/*/*` glob in `tier-dependency-acyclicity-policy.json` starts matching crates instead of
  nothing, and `policy` leaves `unclassified_roots`. Both are currently false-greens: the gate
  declares it is skipping edges for a directory that does not exist.
- **D8 requires an SSOT edit.** `specs/substrate-dependency-dag.json` encodes
  `permit-audit-chain-call-policy-engine-v1` and `permit-observability-call-policy-engine-v1` as
  CALLS. Under the static-stability invariant those are snapshot reads. Correcting them removes the
  empty range without weakening a rule; leaving them encodes an inversion in the single source of truth.
- **D9 costs nothing now** and forecloses a wrong future citation. The one-line unreliability note in
  the baseline is the entire deliverable until the declarations finish.
- **D10 is the only item here that prevents recurrence.** D2/D3/D7 fix three boundaries; D10 stops the
  fourth from being created.

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
