---
id: ADR-0327
status: Proposed
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-engineering
  - council-quality
  - council-foundry
  - council-product
  - council-compliance
  - council-documentation
  - axis-policy-engine
  - axis-workflow-engine
  - axis-foundry
  - ops-program-management
  - ops-sre-reliability
  - ops-compliance
supersedes: []
amends:
  - ADR-0091-multispectrum-review-doctrine.md (binds multispectrum cadence to wave-3 promotion gates)
  - ADR-0110-changeset-state-machine.md (declares the wave-3 ADRs' promotion path through changeset states)
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md (consumes substance-bar as a promotion gate input)
  - ADR-0323-multi-wave-sequencing-doctrine.md (consumes wave evidence ledger as a promotion gate input)
  - ADR-0324-anti-script-anti-template-doctrine.md (consumes anti-pattern attestations as a promotion gate input)
  - ADR-0325-capability-tier-pricing-anchors-public.md (governs the pricing anchor publication gate)
  - ADR-0326-per-tenant-data-residency-attestation.md (governs the residency attestation rollout gate)
superseded_by: []
related:
  - ADR-0063
  - ADR-0091
  - ADR-0105
  - ADR-0110
  - ADR-0111
  - ADR-0112
  - ADR-0113
  - ADR-0130
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0263
  - ADR-0297
  - ADR-0298
  - ADR-0299
  - ADR-0300
  - ADR-0301
  - ADR-0302
  - ADR-0303
  - ADR-0304
  - ADR-0305
  - ADR-0306
  - ADR-0307
  - ADR-0308
  - ADR-0309
  - ADR-0310
  - ADR-0311
  - ADR-0312
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
  - ADR-0321
  - ADR-0322
  - ADR-0323
  - ADR-0324
  - ADR-0325
  - ADR-0326
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/wave-sequencing-schema.json
  - /specs/promotion-gate-schema.json
  - /specs/adr-state-machine.json
  - /specs/audit-events/registry.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/multispectrum-review-v2.4.0.md
  - docs/standards/promotion-gate-doctrine.md
  - docs/feedback/feedback_docs_substance_not_scaffold_2026_05_20.md
  - docs/feedback/feedback_go_with_original_ambition_2026_05_20.md
inbound_citations:
  - docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - docs/decisions/ADR-0323-multi-wave-sequencing-doctrine.md
  - docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md
  - docs/decisions/ADR-0325-capability-tier-pricing-anchors-public.md
  - docs/decisions/ADR-0326-per-tenant-data-residency-attestation.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Codify what "complete" means for the Wave-3 doctrine cluster (ADRs
  0297-0321 and the ADR-0322..0327 extension). Define the named
  promotion-from-Proposed-to-Accepted criteria, the named CI lane gates,
  the named multispectrum-review v2.4.0 cadence, and the named
  rollback/abort criteria. Establish that an ADR remains in `Proposed`
  state until all of its named gates pass and that the wave-3 cluster is
  declared `complete` only when every member of the cluster has been
  promoted to `Accepted` by these gates.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-adr-promotion-gate
  - oya-governance-wave-3-completion-tracker
  - oya-governance-multispectrum-cadence-v240
  - oya-governance-adr-acceptance-evidence-ledger
  - oya-governance-promotion-rollback
decision_owner: council-foundry
---

# ADR-0327: Wave 3 Completion Criteria and Promotion Gates

## Status

Proposed (2026-05-20). The gates defined herein become effective the day
this ADR's own promotion gates are satisfied (i.e. this ADR bootstraps
the gating mechanism, then is governed by it after T+30 days as a
self-application).

## Context

### Named pressure

The Wave-3 doctrine cluster has grown to 30 ADRs (0297-0327) plus the
ADR-0322..0327 extension authored in this very session. Multiple agents
across multiple lanes have authored these ADRs in `Proposed` status,
and a non-trivial number of downstream artifacts (microservices,
journeys, IP slices, PRDs) reference the ADRs as if they were
`Accepted`. The lack of an explicit promotion gate has produced:

- **Phantom acceptance** — downstream artifacts treat `Proposed` ADRs as
  authoritative, then the ADR's substance bar review (per ADR-0322)
  forces revisions that ripple through the downstream graph.
- **Mis-versioning** — citations point to the ADR's path without pinning
  the commit SHA at which the ADR was last reviewed; subsequent edits
  to the ADR silently shift the cited content.
- **Cluster-level fuzziness** — there is no canonical statement of
  "wave-3 is done"; agents and operators each form their own opinion;
  the merge-queue admission gate (per ADR-0111) sometimes accepts a PR
  that depends on a half-complete cluster.

The 2026-05-19/2026-05-20 remediation audits surfaced these failure modes
explicitly. The corrective directive: write a single ADR that says
exactly what completion means and how each member of the cluster gets
there.

### Named constraints

- **C-1 Substance-bar compliance** — per ADR-0322, every ADR must clear
  the substance bar before promotion.
- **C-2 Wave membership** — per ADR-0323, every ADR belongs to a named
  wave; the wave's evidence ledger captures the ADR's promotion state.
- **C-3 Anti-script compliance** — per ADR-0324, every ADR's authorship
  carries provenance and clears the anti-pattern catalog.
- **C-4 Cedar fragment present** — per ADR-0243, every ADR ships Cedar
  fragments; their parsability and non-duplication are gates.
- **C-5 Audit class registration** — per ADR-0263, claimed audit classes
  must be registered before promotion to `Accepted`.
- **C-6 Multispectrum verdict signed** — per ADR-0091 v2.4.0, the
  reviewer-agent verdicts must be present and signed.
- **C-7 Tenancy carry-through** — per ADR-0244, the ADR's named tenant
  scoping is verifiable.
- **C-8 No silent regression** — per `feedback_no_silent_regression.md`,
  any ADR that amends a prior ADR must explicitly declare the amendment.
- **C-9 Cellular interaction** — per ADR-0246 + ADR-0248, the ADR's
  named cell footprint (where applicable) must be consistent with the
  cellular topology.
- **C-10 Authority-chain attestation** — per ADR-0246, ADRs that
  introduce new principals must carry attestation entries.

### Named prior incidents

- **Incident I-1 (2026-05-12)**: PR-118 README template stamping was
  cited by a downstream microservice manifest before the README itself
  had passed substance review; cascade of broken references.
- **Incident I-2 (2026-05-18)**: ADRs 0319/0320/0321 initial drafts were
  cited in a journey artifact batch even though the ADRs themselves did
  not pass substance review until two waves later.
- **Incident I-3 (2026-05-19)**: a microservice manifest declared
  `accepted_by: ADR-0314` while ADR-0314 was still `Proposed` and
  awaiting multispectrum sign-off; the merge-queue admitted the PR
  because no explicit gate existed.
- **Incident I-4 (2026-05-19, codex-erp-ip-w2 lambda-wrap)**: the IP
  slices cited ADR-0315 even though ADR-0315 was `Proposed` and the IP
  slices themselves were the subject of the substance-bar failure that
  triggered the W2 remediation wave.

## Decision

An ADR exists in one of five named states:

- `Drafted` — author has opened a draft PR; not yet eligible for review.
- `Proposed` — substance bar passed; reviewer-agent assignment made;
  awaiting verdicts.
- `Accepted` — all promotion gates passed; ADR is authoritative.
- `Superseded` — replaced by a successor ADR; original retained for
  reference; citations point at the successor.
- `Withdrawn` — author or council retracts; ADR retained for reference;
  citations forbidden for new work.

The promotion-from-Proposed-to-Accepted gates are:

- **G-1 Substance-bar verdict** — `oya-governance-substance-bar`
  signed and stored at the expected evidence path.
- **G-2 Anti-script provenance** — `oya-governance-content-authorship-
  provenance` signed and stored.
- **G-3 Multispectrum facets signed** — at least the F1, F2, F3, F4,
  F5, F6, F7, F8, F9, A1, A2, A3, A4, A5, A6, A7 facets per v2.4.0
  (16 facets), each signed by a distinct reviewer agent (per
  `feedback_consensus_debate_spectrum_lens_subagents.md` one-facet-per-
  agent rule).
- **G-4 Cedar fragments parsed and non-duplicate** — Cedar linter green
  and ADR-0243 canonicalisation check green.
- **G-5 Audit classes registered** — every class enumerated under
  `## Audit Event Classes Emitted` is present in
  `/specs/audit-events/registry.json`.
- **G-6 Wave evidence ledger updated** — the ADR's wave evidence ledger
  (per ADR-0323) names the ADR with the verdict references.
- **G-7 Cross-reference density check** — at least 12 named ADR/spec/
  microservice/journey citations in bespoke context.
- **G-8 Amend-chain declared** — if the ADR amends or supersedes another
  ADR, the relationship is declared in frontmatter and the amended ADR
  carries a back-pointer.
- **G-9 Tenancy envelope verified** — per ADR-0244, the audit classes
  carry tenant context.
- **G-10 Authority-chain entries** — if the ADR introduces new
  principals, the attestation entries exist at
  `/specs/principals/registry.json`.

Wave-3 is `complete` when every member ADR is in `Accepted` state, the
wave's evidence ledger is finalised, the wave's multispectrum cluster-
level verdict is signed, and the wave's promotion event is emitted on
the audit chain.

## Consequences

Defining the five named ADR states and the G-1..G-N promotion gates means an ADR only becomes Accepted once its substance-bar, anti-script provenance, and multispectrum facet verdicts are signed and its promotion event is emitted on the audit chain; the detailed mechanics, SLO implications, and migration path below enumerate the operational consequences of these completion criteria.

## Detailed Mechanics

### D-1 State machine

`/specs/adr-state-machine.json` defines the canonical transitions:

| From       | To         | Trigger                                              |
|------------|------------|------------------------------------------------------|
| Drafted    | Proposed   | substance-bar pass + reviewer-agent assignment made  |
| Proposed   | Accepted   | all promotion gates G-1..G-10 pass                   |
| Proposed   | Drafted    | substance-bar regression on a subsequent commit      |
| Accepted   | Superseded | successor ADR's `supersedes` field references this   |
| Accepted   | Withdrawn  | explicit council withdrawal motion                   |
| Proposed   | Withdrawn  | author withdrawal at any time before Accepted        |
| Drafted    | Withdrawn  | author withdrawal at any time                        |

Transitions are recorded as events on the audit chain (see D-7).

### D-2 Promotion gate implementation

`oya-governance-adr-promotion-gate` is the orchestrating crate. For each
ADR in `Proposed` state, the crate:

1. Loads the ADR's current commit SHA.
2. Verifies the existence of the G-1..G-10 evidence artifacts at the
   expected paths.
3. Verifies each evidence artifact's signature against the principal
   registry.
4. Computes a transition manifest and emits
   `governance.adr.promotion.gate_check.completed` with the per-gate
   results.
5. If all gates are green, emits `governance.adr.promotion.accepted`
   and updates the ADR's frontmatter `status: Accepted` via the
   automated promotion commit.

The automated promotion commit is performed by a privileged service
principal `oyatie.governance.adr_promoter` whose Cedar permit (see D-7)
is narrowly scoped to the frontmatter status field.

### D-3 Multispectrum cadence v2.4.0

The v2.4.0 multispectrum review consists of 16 facets:

- F1 — correctness
- F2 — security
- F3 — privacy
- F4 — substance (BLOCKER class per ADR-0322)
- F5 — performance
- F6 — accessibility
- F7 — operability
- F8 — observability
- F9 — cost
- A1 — own-policy adherence: naming
- A2 — own-policy adherence: documentation
- A3 — own-policy adherence: structure
- A4 — own-policy adherence: architecture
- A5 — own-policy adherence: dependency
- A6 — own-policy adherence: schema
- A7 — own-policy adherence: algorithm

Per `feedback_consensus_debate_spectrum_lens_subagents.md`, each facet
is reviewed by a separate subagent whose lens is that single facet. The
verdicts are signed and stored at
`evidence/debate/<adr-stem>/F<N>-<facet>.signed.json` or
`evidence/debate/<adr-stem>/A<N>-<facet>.signed.json`. Conflicting
verdicts trigger a council-quality reconciliation review before the
promotion gate proceeds.

Cadence:

- Each ADR's 16-facet review runs at least once per wave window (per
  ADR-0323) and at most once per ADR commit.
- The wave-close multispectrum cluster-level verdict integrates the
  per-ADR verdicts and applies the wave-scoped facet (a 17th facet,
  W1, that reviews the cluster as a whole for coherence).
- The W1 facet is signed by council-foundry concurrence after the
  per-ADR facet verdicts are in.

### D-4 Evidence ledger

`oya-governance-adr-acceptance-evidence-ledger` maintains the per-ADR
ledger at `evidence/adr/<adr-stem>/ACCEPTANCE.md`. Required sections:

- `## State` — current state and last transition event ID.
- `## Substance-Bar Verdict` — link to G-1 evidence.
- `## Provenance Attestation` — link to G-2 evidence.
- `## Multispectrum Verdicts` — table of 16 facet verdicts with signing
  agents and timestamps.
- `## Cedar Fragments` — list of fragment paths and linter verdict
  references.
- `## Audit Classes Registered` — list of audit classes added to the
  registry by this ADR.
- `## Wave Membership` — name of the wave that owns this ADR and link
  to the wave's evidence ledger.
- `## Amend Chain` — declared amend/supersede relationships and their
  back-pointers.
- `## Tenancy Verification` — verification record for G-9.
- `## Authority Chain` — verification record for G-10.

The ledger is authored progressively as gate evidence accumulates. The
ledger is part of the ADR's review surface; reviewer agents read it
before signing.

### D-5 Wave-3 completion tracker

`oya-governance-wave-3-completion-tracker` computes the wave-3 cluster
membership and the per-ADR promotion state. Inputs:

- All ADRs under `docs/decisions/` whose frontmatter `wave` field equals
  `w3-g` or a sub-wave thereof.
- The per-ADR ACCEPTANCE.md ledger state.
- The wave evidence ledger state.

Outputs:

- A wave-3 completion dashboard at
  `docs/waves/w3-g/COMPLETION_DASHBOARD.md`.
- An emitted event `governance.wave_3.completion.percentage` at each
  computation cycle (cadence: every 6 hours during active wave, daily
  during steady state).
- A wave-3 `complete` declaration event when every member is `Accepted`,
  the wave evidence ledger is finalised, and the W1 cluster facet is
  signed.

### D-6 Rollback and abort

Promotion is reversible:

- A bug or substance regression in an `Accepted` ADR can trigger a
  council-quality motion to revert to `Proposed`; the motion is
  recorded on the audit chain.
- The reverting commit emits `governance.adr.promotion.rolled_back`
  with detail.
- The downstream graph is notified (via a dependency-tracking
  microservice) so that citing artifacts can be reviewed.

Wave-3 abort:

- If a critical issue is discovered post-completion, the wave can be
  re-opened with a follow-up wave declared.
- The wave evidence ledger records the abort with a postmortem pointer.
- Affected downstream artifacts are flagged for review.

### D-7 Cedar policy hooks (compact summary; see Cedar section below)

The promotion gate is itself a privileged actor. The actor's permits
cover: reading evidence artifacts, signing the promotion transition,
emitting governance events. The actor is forbidden from editing any
content beyond the ADR's frontmatter `status` field; this is enforced
by a narrow Cedar fragment.

### D-8 Phased rollout self-application

This very ADR (0327) follows the rollout it defines:

- **T-0 (publication)**: ADR-0327 enters `Drafted` state with the
  promotion-gate scaffolding present but in shadow mode.
- **T+7 days**: substance-bar pass + multispectrum facet round-1 in
  shadow.
- **T+14 days**: shadow gating produces a dry-run verdict.
- **T+21 days**: full gating active; ADR-0327 transitions through its
  own gates, becoming the first ADR `Accepted` under its own doctrine.
- **T+30 days**: wave-3-completion-tracker active; the full wave-3
  cluster is evaluated and the cluster-level verdict produced.

### D-9 Acceptance of pre-W3 ADRs

ADRs older than ADR-0297 (the start of W3) are evaluated under a
relaxed schedule:

- An ADR in `Accepted` state on this doctrine's effective date remains
  `Accepted` (no retroactive demotion).
- ADRs in `Proposed` or `Drafted` state on the effective date are
  triaged within 14 days: either advanced to `Accepted` under the new
  gates, or rolled back to `Drafted` for material rework, or
  `Withdrawn` with reasons recorded.
- Citations to pre-W3 ADRs remain valid without commit-SHA pinning;
  citations to W3 and later ADRs must pin SHAs at the point of citation.

### D-10 Cluster-level promotion event

When the wave-3 completion criteria are satisfied, the tracker emits
`governance.wave_3.cluster.promoted` with payload:

- `wave_id: "w3-g"`.
- `member_adr_count: <count>` (30 at the time of this ADR's drafting;
  may grow as the wave receives additional members).
- `member_adrs: [...]`.
- `cluster_facet_verdict: "signed"`.
- `accepted_at: <timestamp>`.
- `wave_evidence_ledger_path: "docs/waves/w3-g/EVIDENCE.md"`.
- `council_concurrence: ["council-foundry", "council-architecture",
  "council-quality", "council-product", "council-compliance"]`.

The event is the canonical "wave-3 is done" signal and is consumed by
downstream microservices, dashboards, and the planning substrate at
`/specs/master-plan-sequencing.json`.

### D-11 Citation pinning post-W3

Once an ADR is `Accepted`, citations from downstream artifacts pin a
commit SHA. The pinning protocol:

- Each citation in a Tier-1 or Tier-2 downstream artifact carries
  `(adr_id, commit_sha)` rather than just `adr_id`.
- The pre-commit hook resolves bare citations to the latest
  `Accepted` SHA of the named ADR.
- A subsequent edit to the ADR (e.g. typo fix without status change)
  bumps the SHA; downstream artifacts opt-in to the bump via a
  pinning-refresh sweep.
- A material edit (status change to `Proposed` or substance regression)
  invalidates all downstream pins; the dependency-tracking microservice
  notifies affected artifact owners.

The pinning protocol prevents the "phantom acceptance" failure mode
documented in Context (Incident I-3) by making the ADR's commit-bound
state explicit at every citation site.

### D-12 Downstream-impact analysis at rollback

When an `Accepted` ADR rolls back, the downstream-impact analysis
runs automatically. The analyser:

- Resolves all citations to the rolled-back ADR.
- Classifies each citing artifact as `hard-dep` (citation in
  Detailed Mechanics or Cedar fragments) or `soft-dep` (citation in
  Cross-References only).
- Notifies `hard-dep` owners that their artifact must be reviewed
  within 14 days.
- Notifies `soft-dep` owners that their artifact should be reviewed
  but is not blocked.
- Emits `governance.adr.rollback.downstream_impact_computed` with
  the per-class counts and the affected-artifact list.

The analysis is part of the rollback protocol; rollback may not
complete until the analysis runs successfully.

### D-13 Self-application proof obligation

This ADR (0327) commits to a self-application proof obligation: the
gates defined herein must apply to this ADR itself before any other
ADR uses them. The proof obligation requires:

- This ADR clears G-1 (substance bar per ADR-0322).
- This ADR carries G-2 provenance attestation per ADR-0324.
- This ADR receives 16 facet signatures per G-3.
- This ADR's Cedar fragments are parseable and non-duplicate per G-4.
- This ADR's claimed audit classes are registered per G-5.
- This ADR's wave membership is recorded per G-6.
- This ADR's cross-reference density meets G-7.
- This ADR's amend chain is declared per G-8.
- This ADR's tenancy envelope is verified per G-9.
- This ADR's authority chain entries exist per G-10.

The bootstrapping problem (this ADR must promote itself) is resolved
by the gate's shadow-mode T+0..T+21 window: during shadow mode the
gates evaluate without enforcing; at T+21 this ADR becomes the first
ADR to traverse the gates in BLOCKER mode.

### D-14 Promotion authority and council quorum

The promotion gate alone is not sufficient to declare wave-3
complete; council concurrence is required. The concurrence rules:

- Wave-3 cluster completion requires ≥4 of 5 named councils concurring
  (council-foundry, council-architecture, council-quality,
  council-product, council-compliance).
- A council may withhold concurrence by emitting a
  `governance.council.concurrence_withheld` event with a named reason.
- A withheld concurrence triggers a 14-day council-discussion window;
  if not resolved, the wave-3 cluster remains in `awaiting_concurrence`
  state and downstream artifacts remain pinned to the last accepted
  state of each member ADR.
- Concurrence is rescindable; a council that signed concurrence may
  rescind by emitting `governance.council.concurrence_rescinded`; a
  rescission below the 4-of-5 quorum reverts the cluster from
  `complete` to `awaiting_concurrence` and emits the cluster-promotion
  retraction event from F-4.

The four-of-five quorum prevents a single council from blocking
indefinitely while requiring meaningful multi-council agreement.

## Cedar Policy Hooks

```cedar
// Fragment: cedar/adr-promotion/promoter-may-flip-status.cedar
permit (
  principal == Service::"oyatie.governance.adr_promoter",
  action == Frontmatter::"write_key",
  resource is DocArtifact
) when {
  resource.doc_class == "Architecture-Decision-Record" &&
  context.key == "status" &&
  context.value in ["Proposed", "Accepted", "Superseded", "Withdrawn"] &&
  context.all_promotion_gates_green == true
};
```

```cedar
// Fragment: cedar/adr-promotion/promoter-may-read-evidence.cedar
permit (
  principal == Service::"oyatie.governance.adr_promoter",
  action == Evidence::"read",
  resource is EvidenceArtifact
) when {
  resource.evidence_class in ["substance_bar", "provenance",
                              "multispectrum_facet", "cedar_linter",
                              "audit_class_registry", "wave_evidence_ledger",
                              "tenancy_verification", "authority_chain"]
};
```

```cedar
// Fragment: cedar/adr-promotion/promoter-may-not-edit-body.cedar
forbid (
  principal == Service::"oyatie.governance.adr_promoter",
  action,
  resource is DocArtifact
) when {
  resource.doc_class == "Architecture-Decision-Record" &&
  action != Frontmatter::"write_key"
};
```

```cedar
// Fragment: cedar/adr-promotion/reviewer-agent-may-sign-facet.cedar
permit (
  principal in Group::"oyatie.governance.reviewer_agents",
  action == FacetVerdict::"sign",
  resource is DocArtifact
) when {
  resource.doc_class == "Architecture-Decision-Record" &&
  context.facet in ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9",
                    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "W1"] &&
  context.signing_principal_assigned_to_facet == true &&
  context.distinct_subagent_per_facet == true
};
```

```cedar
// Fragment: cedar/adr-promotion/no-shared-agent-across-facets.cedar
forbid (
  principal,
  action == FacetVerdict::"sign",
  resource is DocArtifact
) when {
  context.facet_assignment_violates_one_facet_per_agent_rule == true
};
```

```cedar
// Fragment: cedar/adr-promotion/rollback-requires-council-motion.cedar
permit (
  principal in Group::"oyatie.council.quality.voters",
  action == ADR::"rollback",
  resource is DocArtifact
) when {
  context.quorum_present == true &&
  context.motion_recorded_on_audit_chain == true
};
```

```cedar
// Fragment: cedar/adr-promotion/wave-3-cluster-promotion.cedar
permit (
  principal == Service::"oyatie.governance.wave_3_completion_tracker",
  action == Wave::"declare_complete",
  resource is Wave
) when {
  resource.wave_id == "w3-g" &&
  context.all_members_accepted == true &&
  context.cluster_facet_signed == true &&
  context.council_concurrence_count >= 4
};
```

## Audit Event Classes Emitted

| Class                                                | Severity | Source crate                                   |
|------------------------------------------------------|----------|------------------------------------------------|
| governance.adr.state.drafted                         | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.state.proposed                        | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.state.accepted                        | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.state.superseded                      | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.state.withdrawn                       | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.promotion.gate_check.completed        | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.promotion.gate.blocked                | BLOCKER  | oya-governance-adr-promotion-gate              |
| governance.adr.promotion.accepted                    | INFO     | oya-governance-adr-promotion-gate              |
| governance.adr.promotion.rolled_back                 | WARN     | oya-governance-promotion-rollback              |
| governance.adr.facet.signed                          | INFO     | oya-governance-multispectrum-cadence-v240      |
| governance.adr.facet.conflict_detected               | BLOCKER  | oya-governance-multispectrum-cadence-v240      |
| governance.adr.facet.shared_agent_violation          | BLOCKER  | oya-governance-multispectrum-cadence-v240      |
| governance.adr.cluster_facet.signed                  | INFO     | oya-governance-multispectrum-cadence-v240      |
| governance.adr.ledger.updated                        | INFO     | oya-governance-adr-acceptance-evidence-ledger  |
| governance.adr.ledger.malformed                      | BLOCKER  | oya-governance-adr-acceptance-evidence-ledger  |
| governance.wave_3.completion.percentage              | INFO     | oya-governance-wave-3-completion-tracker       |
| governance.wave_3.cluster.promoted                   | INFO     | oya-governance-wave-3-completion-tracker       |
| governance.wave_3.member.added                       | INFO     | oya-governance-wave-3-completion-tracker       |
| governance.wave_3.abort.initiated                    | WARN     | oya-governance-wave-3-completion-tracker       |

## SLO Implications

`microservices/governance/adr-promotion-gate/slos/promotion-gate.openslo.yaml`:

- `gate_check_p99_latency`: ≤ 30 s per ADR.
- `multispectrum_facet_signing_completeness_per_wave`: ≥ 99% of ADRs in
  the wave have all 16 facets signed within 5 calendar days of the
  wave window.
- `promotion_rollback_rate`: ≤ 2% of `Accepted` ADRs roll back within
  90 days of acceptance.
- `wave_3_completion_freshness`: dashboard not more than 6 hours stale.
- `cluster_promotion_event_latency`: cluster `complete` declaration
  emitted within 2 hours of the last member's acceptance.

## Migration Path / Phased Rollout

- **Phase 0 (T-0, ADR Proposed)**: shadow mode; gates evaluate but do
  not enforce.
- **Phase 1 (T+7 days)**: gates enforce for W3-G member ADRs only.
- **Phase 2 (T+14 days)**: gates enforce for all ADRs in `Proposed` or
  newer.
- **Phase 3 (T+21 days)**: this ADR self-promotes through its own gates.
- **Phase 4 (T+30 days)**: wave-3 cluster completion tracker active;
  cluster `complete` declaration eligible.
- **Phase 5 (T+60 days)**: rollback protocol exercise (a planned
  drill) to validate the rollback mechanism.

## Failure Modes + Recovery

### F-1: Reviewer agent unavailable for a facet

A reviewer agent assigned to a facet is offline at signing time.
Recovery: the wave descriptor names a backup reviewer agent per facet;
the backup signs within 48 hours; if neither is available the wave
extends per ADR-0323 D-7 re-plan.

### F-2: Cedar fragment regression

A previously-`Accepted` ADR's Cedar fragments fail re-linting under a
newer Cedar version. Recovery: the ADR rolls back to `Proposed`
automatically; the council-quality decides whether to update the ADR
or pin the Cedar linter version.

### F-3: Substance-bar regression on edit

An `Accepted` ADR is edited and the new content fails substance bar.
Recovery: the ADR rolls back to `Drafted` automatically; the edit is
reverted or reauthored; a new full promotion cycle ensues.

### F-4: Cluster completion premature declaration

The completion tracker emits `cluster.promoted` while a member is
actually still `Proposed`. Recovery: the event is retracted via
`governance.wave_3.completion.retracted`; the tracker re-computes;
postmortem opens.

### F-5: Promoter principal compromised

The `oyatie.governance.adr_promoter` principal's key is compromised.
Recovery: per ADR-0247 self-modification doctrine, the principal is
rotated; any promotions signed during the compromise window are
reviewed individually.

### F-6: Facet-conflict deadlock

Two reviewer agents sign opposing facets and council reconciliation
deadlocks. Recovery: the council escalates to council-architecture
chair for a tie-break; the resolution is recorded in the per-ADR
ledger.

### F-7: Audit class registration drift

An ADR claims an audit class that the registry rejects (e.g. naming
collision with another class). Recovery: the gate BLOCKs promotion;
the ADR's claimed class is renamed; the registry update lands and the
gate re-evaluates.

## Verification

Named CI checks:

- `oya-governance-adr-promotion-gate/g1-substance-bar`
- `oya-governance-adr-promotion-gate/g2-provenance`
- `oya-governance-adr-promotion-gate/g3-multispectrum`
- `oya-governance-adr-promotion-gate/g4-cedar-fragments`
- `oya-governance-adr-promotion-gate/g5-audit-classes`
- `oya-governance-adr-promotion-gate/g6-wave-evidence`
- `oya-governance-adr-promotion-gate/g7-cross-references`
- `oya-governance-adr-promotion-gate/g8-amend-chain`
- `oya-governance-adr-promotion-gate/g9-tenancy`
- `oya-governance-adr-promotion-gate/g10-authority-chain`
- `oya-governance-wave-3-completion-tracker`
- `oya-governance-multispectrum-cadence-v240`
- `oya-governance-adr-acceptance-evidence-ledger`
- `oya-governance-promotion-rollback`

Named crates:

- `oya-governance-adr-promotion-gate`
- `oya-governance-wave-3-completion-tracker`
- `oya-governance-multispectrum-cadence-v240`
- `oya-governance-adr-acceptance-evidence-ledger`
- `oya-governance-promotion-rollback`

Verification fixtures live at `tests/governance/adr-promotion/` and
include: a synthetic ten-ADR wave promotion, a facet-conflict
escalation scenario, a rollback drill, a registry-collision scenario,
and a cluster-complete declaration happy path.

## Cross-References

### Other ADRs

The wave-3 cluster members (each ADR is named here to make the cluster
explicit and to satisfy the G-7 cross-reference density check):

- ADR-0297 (abuse-defence-baseline-anti-bot-spoof-scrape)
- ADR-0298 (emergency-services-bypass-life-safety)
- ADR-0299 (account-recovery-resilience)
- ADR-0300 (whistleblower-press-freedom-anonymity)
- ADR-0301 (survivor-safety-domestic-abuse-mode)
- ADR-0302 (deceased-user-inheritance-doctrine)
- ADR-0303 (cognitive-impairment-decision-resilience)
- ADR-0304 (cross-jurisdiction-conflict-resolution)
- ADR-0305 (delegated-agent-authority-chain)
- ADR-0306 (disaster-mode-cell-resilience)
- ADR-0307 (detection-substrate-streaming-batch)
- ADR-0308 (ml-model-lifecycle-ai-act-compliance)
- ADR-0309 (detection-fairness-audit-civil-rights)
- ADR-0310 (investigation-case-management)
- ADR-0311 (dual-tenant-identity-personal-vs-work-boundary)
- ADR-0312 (court-warrant-scoped-piercing)
- ADR-0313 (conglomerate-tenant-hierarchy-sovereign-children)
- ADR-0314 (marketplace-as-universal-deal-settlement)
- ADR-0315 (erp-coverage-doctrine-sap-parity)
- ADR-0316 (capability-tier-over-product-fragmentation)
- ADR-0317 (role-based-projection-unified-ux-shell)
- ADR-0318 (collar-color-workspace-universality)
- ADR-0319 (front-middle-back-office-information-barrier)
- ADR-0320 (apprentice-intern-resident-fellow-transient-identity)
- ADR-0321 (b2b-saas-industry-leader-coverage)
- ADR-0322 (substance-bar-as-doctrine-and-CI-enforcement)
- ADR-0323 (multi-wave-sequencing-doctrine)
- ADR-0324 (anti-script-anti-template-doctrine)
- ADR-0325 (capability-tier-pricing-anchors-public)
- ADR-0326 (per-tenant-data-residency-attestation)

Substrate ADRs consulted:

- ADR-0063 (doc-coverage-enforcement) — substrate.
- ADR-0091 (multispectrum-review-doctrine) — v2.4.0 binding.
- ADR-0105 (layer-enum 13-canonical) — governance lane layer.
- ADR-0110 (changeset state machine) — state-machine alignment.
- ADR-0111 (merge-queue projected state) — admission-gate composition.
- ADR-0112 (webhook-driven Foundry invocation) — event substrate.
- ADR-0113 (VCS orchestrator end-to-end) — VCS primitives.
- ADR-0130 (observability SLO-gated promotion) — SLO substrate.
- ADR-0131 (per-microservice flat layout) — crate layout.
- ADR-0132 (suite dissolution) — shape alignment.
- ADR-0145 (inter-microservice reform) — direct gRPC invariants.

Keystone bundle (ADRs 0242-0255):

- ADR-0242 (oyatie-is-a-tenant)
- ADR-0243 (Cedar universal gate)
- ADR-0244 (tenant scoping)
- ADR-0245 (substrate-product layering)
- ADR-0246 (cellular topology)
- ADR-0247 (self-modification doctrine)
- ADR-0248 (Amazon-shape cellular)
- ADR-0249 (multi-category marketplace)
- ADR-0250 (build ahead of certification)
- ADR-0251 (compliance packs)
- ADR-0252 (HLC default)
- ADR-0253 (HTTP/3 default)
- ADR-0254 (K8s + Cloud Hypervisor)
- ADR-0255 (intelligence two-layer)

- ADR-0263 (audit-event registry).

### Standards

- `docs/standards/promotion-gate-doctrine.md` (W2 companion standard).
- `docs/standards/multispectrum-review-v2.4.0.md`.
- `docs/standards/documentation-rigor.md`.

### Microservices

- `microservices/governance/adr-promotion-gate/`.
- `microservices/governance/wave-3-completion-tracker/`.
- `microservices/governance/multispectrum-cadence/`.
- `microservices/observability/`.
- `microservices/audit-chain/`.

### Journeys

- `journeys/governance/jou-2026-05-20-promote-an-adr/`.
- `journeys/governance/jou-2026-05-20-rollback-an-adr/`.
- `journeys/governance/jou-2026-05-20-declare-wave-complete/`.

### Specs

- `/specs/adr-state-machine.json`
- `/specs/promotion-gate-schema.json`
- `/specs/master-plan-sequencing.json` (updated).
- `/specs/wave-sequencing-schema.json`.

### Feedback notes consumed

- `feedback_docs_substance_not_scaffold_2026_05_20.md`
- `feedback_go_with_original_ambition_2026_05_20.md`
- `feedback_consensus_debate_spectrum_lens_subagents.md`
- `feedback_multispectrum_review_v22.md`
- `feedback_multispectrum_adherence_facets.md`
- `feedback_no_silent_regression.md`
- `feedback_pipeline_clog_gotchas_2026_05_17.md`
- `feedback_self_merge_via_contract_path.md`
- `feedback_automate_everything.md`
- `feedback_milestone_phase_hierarchy.md`
