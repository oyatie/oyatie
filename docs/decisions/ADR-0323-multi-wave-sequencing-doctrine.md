---
id: ADR-0323
status: Proposed
date: 2026-05-20
owners:
  - council-architecture
  - council-engineering
  - council-product
  - council-quality
  - council-foundry
  - axis-workflow-engine
  - axis-foundry
  - axis-policy-engine
  - ops-sre-reliability
  - ops-program-management
supersedes: []
amends:
  - ADR-0091-multispectrum-review-doctrine.md (sequences multispectrum review across waves)
  - ADR-0132-product-platform-and-bundle-dissolution.md (clarifies wave-scoped batch-size discipline)
  - ADR-0145-inter-microservice-communication-reform.md (sequences inter-service changes across waves to preserve direct gRPC invariants)
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md (wave-scoped substance bar coordination)
superseded_by: []
related:
  - ADR-0091
  - ADR-0110
  - ADR-0111
  - ADR-0112
  - ADR-0113
  - ADR-0130
  - ADR-0132
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0263
  - ADR-0316
  - ADR-0321
  - ADR-0322
  - ADR-0324
  - ADR-0327
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/wave-sequencing-schema.json
  - /specs/microservices/manifest-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/multispectrum-review-v2.4.0.md
  - docs/feedback/feedback_go_with_original_ambition_2026_05_20.md
  - docs/feedback/feedback_docs_substance_not_scaffold_2026_05_20.md
  - docs/feedback/feedback_pipeline_clog_gotchas_2026_05_17.md
inbound_citations:
  - docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - docs/feedback/feedback_go_with_original_ambition_2026_05_20.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Codify multi-wave sequencing as the canonical pattern for authoring
  substantive content (ADR clusters, journey artifact batches, IP slice
  batches, microservice README seeds, PRD wave drops) and for shipping
  cross-cutting refactors that touch multiple substrate microservices.
  Define a named batch-size calibration function that depends on
  substance-density, on the number of agents available for parallel
  authorship, and on the pipeline backpressure observed during the
  preceding wave. The doctrine prohibits unbounded fan-out batches and
  forbids agents from skipping the per-wave verification cadence.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-wave-sequencing
  - oya-governance-wave-batch-size-cap
  - oya-governance-wave-verification-cadence
  - oya-governance-wave-evidence-ledger
  - oya-governance-wave-pipeline-backpressure-respect
decision_owner: council-foundry
---

# ADR-0323: Multi-Wave Sequencing Doctrine

## Status

Proposed (2026-05-20). The doctrine applies retroactively to in-flight waves
W1..W4 of the post-keystone work; existing waves are mapped to this
doctrine's named slots in the migration plan (D-10).

## Context

### Named pressure

The 2026-05-17 pipeline-clog incident (documented in
`feedback_pipeline_clog_gotchas_2026_05_17.md`) and the 2026-05-19 substance
audit (documented in `feedback_docs_substance_not_scaffold_2026_05_20.md`)
together exposed a recurring failure mode where the right way to author
substantive content is multi-wave sequencing, while the agent fleet's
preferred default is unbounded fan-out. The fan-out pattern produces:

- **O(N²) merge-queue contention** — each PR rebases against every other
  in-flight PR in its lane; at N≥12 in-flight PRs in a single lane the
  rebase storm consumes more reviewer-agent budget than the substance
  authoring did.
- **Substance regression** — agents under fan-out pressure resort to
  template stamping (banned by ADR-0322) because per-artifact authoring
  budget collapses.
- **Inconsistent cross-references** — artifacts authored in parallel often
  miss citations to each other or contradict each other's claims.
- **Verification debt** — without per-wave verification cadence, the agent
  fleet accumulates an unverified backlog that ultimately blocks promotion.

The corrective directive captured in
`feedback_go_with_original_ambition_2026_05_20.md` is to keep the original
ambition (broad coverage) but achieve it via sequential waves where each
wave is small, fully verified, and signed off before the next wave starts.

### Named constraints

- **C-1 Substance-bar prerequisite** — per ADR-0322, every artifact in a
  wave must clear the substance bar; this fixes a per-artifact effort floor
  that, multiplied by batch size, must fit within the wave window.
- **C-2 Pipeline cap** — per `feedback_pipeline_clog_gotchas_2026_05_17.md`,
  the in-flight PR cap is 12 per lane. Wave batch sizes that exceed the cap
  trigger pipeline saturation.
- **C-3 Multispectrum cadence** — per ADR-0091 and the v2.4.0 update, each
  wave must run multispectrum review at the wave boundary, not per artifact;
  this fixes a per-wave reviewer-agent budget.
- **C-4 Authority chain** — per ADR-0145, wave-scope changes must compose
  with direct gRPC invariants; ADR-0145 specifically forbids forced-adapter
  patterns introduced as wave-glue.
- **C-5 Foundry pipeline** — per `feedback_foundry_pipeline_canonical.md`,
  all wave PRs enter through the Foundry pipeline; the doctrine inherits the
  M-CC-P11 substrate.
- **C-6 No silent regression** — per `feedback_no_silent_regression.md`, no
  wave may silently change a public contract; each wave's contract diff is
  declared up front.

### Named prior incidents

- **Incident I-1 (2026-05-17)**: 14 PRs landed in a single 6-hour window in
  the `oya-shared-workflow-*` lane; rebase storm consumed reviewer-agent
  budget and forced the cap-12 rule into existence.
- **Incident I-2 (2026-05-19, batch JA-2026-05-19-A)**: 47 journey
  artifacts produced in a fan-out batch; 14 failed substance bar; required
  three remediation waves to clear.
- **Incident I-3 (2026-05-19, codex-erp-ip-w2)**: 18 IP slices produced by a
  shell-loop lambda-wrap; required full rewrite plus the ADR-0324 anti-script
  doctrine.
- **Incident I-4 (2026-05-18)**: ADR cluster 0297-0321 (25 ADRs) authored in
  a single conceptual batch but landed across five waves with mismatched
  cross-references; subsequent W3-remediation wave added 122 missing
  citations.

### Scope

This doctrine governs:

- Documentation authorship (ADR clusters, journey batches, IP slices, README
  seeds, PRDs, RFCs, specs).
- Cross-cutting substrate refactors (renames, lane migrations, dependency
  bumps) where >5 microservices are touched.
- Authority-chain-sensitive policy migrations (Cedar fragment changes,
  audit-event registry expansions, governance lane introductions).

It does not govern:

- Hotfix PRs to a single file (no wave required).
- Per-microservice feature work bounded to one lane and ≤3 files.
- Routine dependency updates without contract changes.

## Decision

A wave is the named unit of work delivery. Each wave has:

- **W-Name** — a kebab-case identifier with a date suffix (e.g.
  `w3-g-keystone-doctrine-2026-05-20`).
- **W-Ambition** — a single sentence stating the wave's substantive goal.
- **W-Batch** — a list of named artifacts ≤ the batch-size cap from D-2.
- **W-Verification-Cadence** — the named pre-merge, mid-wave, and end-wave
  verification steps from D-4.
- **W-Evidence-Ledger** — a single markdown ledger at
  `docs/waves/<W-Name>/EVIDENCE.md` enumerating per-artifact verification
  artifacts.
- **W-Multispectrum-Verdict** — a signed reviewer-agent verdict per facet at
  `evidence/waves/<W-Name>/multispectrum.signed.json`.
- **W-Promotion** — the named criterion under which the wave is promoted
  from `in-flight` to `landed` (see ADR-0327 promotion gates).

Waves are sequential within a lane (no two in-flight waves overlap on the
same governance lane). Waves are concurrent across lanes provided the
authority-chain invariants of ADR-0145 hold (no cross-lane forced
coupling).

## Consequences

Adopting the wave as the named unit of delivery means every batch of work carries the W-Name, W-Ambition, W-Batch, evidence ledger, multispectrum verdict, and promotion criterion defined above, with waves running sequentially within a lane and concurrently across lanes; the detailed mechanics, SLO implications, and migration path below enumerate the operational consequences of that sequencing discipline.

## Detailed Mechanics

### D-1 Wave definition schema

`/specs/wave-sequencing-schema.json` defines the JSON shape of a wave
descriptor. Required fields:

- `wave_name` (string, kebab-case, date-suffixed).
- `wave_ambition` (string, ≤140 chars; mirrors the W-Ambition single
  sentence).
- `wave_lane` (string; references the canonical governance lane name).
- `wave_predecessors` (array<string>; names of waves whose successful
  completion gates this wave's start).
- `wave_artifacts` (array<object>; each object names the artifact path and
  declares its authority tier per ADR-0322).
- `wave_substance_density_class` (enum: `dense`, `medium`, `sparse`; chosen
  per D-2 calibration).
- `wave_planned_start_at` (RFC 3339).
- `wave_planned_landed_at` (RFC 3339; must be within the per-class window
  from D-2).
- `wave_evidence_ledger_path` (string; convention enforces
  `docs/waves/<wave_name>/EVIDENCE.md`).
- `wave_multispectrum_verdict_path` (string).
- `wave_promotion_criterion` (string; references the ADR-0327 named
  criterion).

A wave descriptor lives at `docs/waves/<wave_name>/DESCRIPTOR.json` and is
authored as the first artifact of the wave.

### D-2 Batch-size calibration

The batch-size cap is a function of substance density and the rolling
in-flight pipeline load:

| Density class | Authority tier of artifacts | Cap per wave | Wave window |
|---------------|-----------------------------|--------------|-------------|
| dense         | Tier-1 (≥800 lines)         | 3            | 5 days      |
| dense         | Tier-2 (≥500 lines)         | 6            | 5 days      |
| medium        | Tier-3 (≥250 lines)         | 12           | 4 days      |
| sparse        | Tier-4 (≥120 lines)         | 24           | 3 days      |

Mixed-tier waves use the most restrictive cap across the tiers present
(e.g. a wave with one Tier-1 plus four Tier-2 artifacts is dense, cap=3+6
slots used as 1+4=5 ≤ 9 but the cap is still constrained by Tier-1's slot
count of 3 minus the one consumed = 2 remaining Tier-1 slots; net cap is
the per-tier cap minus per-tier consumption).

The cap may be reduced (never increased) by pipeline backpressure: if the
in-flight PR count in the wave's lane exceeds 8 during wave authoring, the
remaining cap is multiplied by 0.5 (rounded down) for the wave window.

`oya-governance-wave-batch-size-cap` enforces these caps. A wave descriptor
that exceeds the cap fails the lane and the wave cannot be started.

### D-3 Wave predecessor graph

Waves form a DAG. Each wave names its predecessors and may not start until
each predecessor reaches the `landed` state. The DAG is computed from the
descriptor files at `docs/waves/*/DESCRIPTOR.json` and serialised to
`docs/waves/WAVE_GRAPH.json` by the W1 scaffold wave's tooling. Cycles in
the DAG are detected by `oya-governance-wave-sequencing` and BLOCK the
descriptor PR.

Cross-lane predecessors are allowed (a wave in lane A may depend on a wave
in lane B) but cross-lane predecessors must declare the contract dependency
explicitly in `wave_cross_lane_contract_dependencies` (array<string> of
contract IDs); the contracts must exist in `/specs/contracts/` before the
wave starts.

### D-4 Verification cadence

Per-wave verification has three named touchpoints:

- **Wave-open verification**: at descriptor PR merge, the lane verifies that
  the descriptor schema is valid, the predecessor graph is acyclic, and the
  batch-size cap is satisfied.
- **Mid-wave checkpoint**: at the 50% planned-landed-at midpoint, the wave
  must have ≥30% of its artifacts in either `Proposed` or merged state;
  failure to clear 30% triggers either a wave-scope reduction or a wave
  re-plan (D-7 procedure).
- **Wave-close verification**: at the wave's `landed` transition, every
  artifact must clear ADR-0322's substance bar and the multispectrum
  reviewer-agent verdict must be signed and present at the expected path.

`oya-governance-wave-verification-cadence` enforces all three.

### D-5 Wave evidence ledger

`docs/waves/<wave_name>/EVIDENCE.md` is the canonical evidence ledger for
the wave. Required sections:

- `## Artifacts` — bullet list of artifact paths and current statuses.
- `## Substance-Bar Verdicts` — per-artifact link to the substance-bar
  evaluation record.
- `## Multispectrum Verdicts` — per-facet link to the reviewer-agent
  signature artifact.
- `## Cedar Fragments Introduced` — bullet list of new Cedar fragment paths.
- `## Audit Event Classes Introduced` — bullet list per ADR-0263.
- `## Cross-Wave Dependencies Honoured` — citation to each named predecessor
  with the evidence that its contracts are still satisfied.
- `## Postmortem Pointer` — link to any postmortem if the wave deviated
  from its plan; empty if no deviation.

The ledger is authored progressively as wave artifacts land; the file's
last commit timestamp must precede the wave's `landed` transition.

`oya-governance-wave-evidence-ledger` verifies presence, sections, and
non-emptiness.

### D-6 Concurrent vs sequential waves

Two waves may be concurrent only if all of the following hold:

- They live in different governance lanes.
- Neither wave's artifact set touches a microservice owned by the other
  wave's lane.
- The cross-lane contract dependencies of each wave are declared and
  honoured.
- The combined in-flight PR count across the two waves does not exceed 16
  (a cross-wave global cap, distinct from the per-lane cap of 12).

A wave that violates any of these conditions must be sequenced after the
conflicting wave's `landed` transition.

### D-7 Wave re-plan procedure

If the mid-wave checkpoint flags a wave as off-plan, the council-foundry
authors a re-plan commit that may:

- Reduce the wave's batch size by removing artifacts (the removed artifacts
  become the seed of a new descendant wave).
- Extend the wave's `planned_landed_at` by ≤50% of the original window
  (extensions beyond 50% require a wave-replacement: close the current
  wave with `aborted` status and open a successor wave that inherits the
  artifacts).
- Reduce the wave's substance-density class (e.g. demote dense to medium)
  only if the artifacts' authority tiers actually permit the demotion
  (a Tier-1 ADR cannot ride in a medium-density wave).

Re-plans are recorded in the EVIDENCE.md ledger and on the audit chain via
`governance.wave.replan.recorded` events.

### D-8 Concurrency limit per agent fleet

The wave-batch-size cap interacts with the agent fleet ceiling
(documented in `project_wave_3_g_state_2026_05_21.md` as 11 agents in
flight: 3 Claude + 8 codex). The doctrine adds:

- No wave is permitted with more parallel agents assigned than the
  per-class cap above.
- Per-class agent assignments: dense Tier-1 = 1 author per artifact;
  dense Tier-2 = up to 2 authors per artifact (one primary, one paired);
  medium and sparse = up to 3 authors per artifact for parallel
  drafting of independent sub-sections.

The agent-fleet manager (per the omc-teams substrate referenced in
`oh-my-claudecode:omc-teams`) enforces the per-wave agent budget by
refusing to dispatch new agents when the per-wave assignment exceeds the
table.

### D-9 Pipeline backpressure respect

`oya-governance-wave-pipeline-backpressure-respect` queries the lane's
in-flight PR count via the GitHub API and enforces the 0.5 multiplier
described in D-2. The crate caches lane state for 60 s to avoid rate-limit
exhaustion. When backpressure forces a cap reduction, the crate emits
`governance.wave.backpressure.cap_reduced` with fields
`(wave_name, original_cap, new_cap, lane_in_flight_count)`.

### D-10 Migration of in-flight waves W1..W4

In-flight waves at the time of this ADR's landing are mapped:

- **W1 — substance-bar scaffold wave**: classified `dense Tier-1`, cap=3
  (one ADR per wave instance; this very ADR plus ADR-0322 and ADR-0324
  form a three-artifact dense wave landing as a unit; the remaining
  three ADRs 0325..0327 form a successor wave).
- **W2 — codex-erp-ip-w2 remediation**: classified `medium Tier-3`, cap=12;
  required full rewrite per ADR-0324.
- **W3-G — keystone doctrine cluster**: retroactively split into three
  sub-waves W3-G-α (ADRs 0297-0306), W3-G-β (ADRs 0307-0314), W3-G-γ
  (ADRs 0315-0321); each sub-wave is dense Tier-1.
- **W4 — wave doctrine + completion criteria**: this ADR plus ADR-0327.

Migration commits add `WAVE_MAPPING.md` entries under `docs/waves/` for
each retroactively-classified wave.

### D-11 Wave naming convention and lifecycle states

Wave names follow `w<wave-major>-<wave-minor-or-letter>-<short-slug>-<YYYY-MM-DD>`
(e.g. `w3-g-keystone-doctrine-2026-05-20`, `w4-substance-bar-2026-05-20`).
The naming protects against name collisions when multiple waves land in
the same wave-major (e.g. W3-α vs W3-β). The lifecycle states a wave
passes through:

- `proposed` — descriptor authored but not yet validated.
- `open` — descriptor validated; predecessors satisfied; artifacts may
  be authored.
- `awaiting_predecessor` — descriptor validated but at least one
  predecessor is not yet `landed`.
- `in-flight` — at least one artifact has been authored or merged.
- `awaiting_midpoint_checkpoint` — midpoint date reached; checkpoint
  pending.
- `off-plan` — midpoint checkpoint detected off-plan; awaiting re-plan
  or escalation.
- `awaiting_multispectrum_signature` — all artifacts merged; signatures
  pending.
- `landed` — multispectrum verdict signed; evidence ledger finalised.
- `aborted` — wave terminated early; successor declared.

Each transition emits an audit event from the catalog in D-7 above.

### D-12 Coordination with the foundry pipeline

The Foundry pipeline (per `feedback_foundry_pipeline_canonical.md`) is
the canonical entry path for wave PRs. The wave doctrine binds to the
pipeline as follows:

- Each wave PR carries the wave descriptor's `wave_name` in a
  PR-metadata field; the pipeline's admission gate (per ADR-0111
  projected-state mechanism) consults the wave descriptor to determine
  whether the PR is in-scope for the current wave.
- The merge-queue uses the wave's cap to refuse admission when the
  cap is exhausted; the cap exhaustion emits the
  `governance.wave.batch_cap.exceeded` event.
- The reviewer-agent dispatch (per the omc-teams substrate) consults
  the wave descriptor's reviewer-agent pool when assigning per-facet
  reviewers.
- The wave's `landed` transition is the trigger that allows downstream
  waves (per the predecessor graph) to enter `open`.

### D-13 Wave-level evidence retention

The wave evidence ledger and the per-artifact substance-bar and
multispectrum verdicts are retained for ≥365 days under
`docs/waves/<wave_name>/` and `evidence/`. Retention enables:

- Forensic re-evaluation of any wave's claims.
- Reconstruction of the wave's predecessor graph at the wave's landed
  timestamp.
- Audit of the reviewer-agent pool used for each wave (for fairness
  monitoring per the spectrum-lens-subagents doctrine).
- Quarterly council-foundry review of wave cadence and SLO performance.

### D-14 Multi-tenant wave coordination

Where a wave touches artifacts owned by multiple tenants (per ADR-0244),
the wave descriptor enumerates the tenant set and per-tenant ownership.
The multispectrum-cadence reviewer-agent pool must include at least one
reviewer agent that is authorised under each owning tenant's Cedar
fragments. This composes with the conglomerate-tenant-hierarchy doctrine
(ADR-0313): a wave that touches a parent tenant's artifacts plus its
child tenant's artifacts must satisfy both tenants' authority chains
before promotion.

## Cedar Policy Hooks

```cedar
// Fragment: cedar/wave-sequencing/wave-may-open.cedar
permit (
  principal in Group::"oyatie.governance.wave_authors",
  action == Wave::"open",
  resource is Wave
) when {
  context.descriptor_valid == true &&
  context.predecessor_graph_acyclic == true &&
  context.batch_size_within_cap == true
};
```

```cedar
// Fragment: cedar/wave-sequencing/wave-may-land.cedar
permit (
  principal in Group::"oyatie.governance.wave_promoters",
  action == Wave::"land",
  resource is Wave
) when {
  context.all_artifacts_substance_bar_passed == true &&
  context.multispectrum_verdict_signed == true &&
  context.evidence_ledger_present == true
};
```

```cedar
// Fragment: cedar/wave-sequencing/wave-may-abort.cedar
permit (
  principal in Group::"oyatie.governance.wave_owners",
  action == Wave::"abort",
  resource is Wave
) when {
  context.successor_wave_declared == true &&
  context.postmortem_ledger_present == true
};
```

```cedar
// Fragment: cedar/wave-sequencing/no-concurrent-same-lane.cedar
forbid (
  principal,
  action == Wave::"open",
  resource is Wave
) when {
  context.lane_has_in_flight_wave == true
};
```

```cedar
// Fragment: cedar/wave-sequencing/agent-fleet-budget.cedar
forbid (
  principal == Service::"oyatie.foundry.dispatcher",
  action == Agent::"dispatch",
  resource is Agent
) when {
  context.wave_agent_count_post_dispatch > context.wave_agent_cap
};
```

## Audit Event Classes Emitted

| Class                                            | Severity | Source crate                                |
|--------------------------------------------------|----------|---------------------------------------------|
| governance.wave.opened                           | INFO     | oya-governance-wave-sequencing              |
| governance.wave.midpoint.checkpoint              | INFO     | oya-governance-wave-verification-cadence    |
| governance.wave.midpoint.off_plan                | WARN     | oya-governance-wave-verification-cadence    |
| governance.wave.landed                           | INFO     | oya-governance-wave-sequencing              |
| governance.wave.aborted                          | WARN     | oya-governance-wave-sequencing              |
| governance.wave.replan.recorded                  | INFO     | oya-governance-wave-sequencing              |
| governance.wave.batch_cap.exceeded               | BLOCKER  | oya-governance-wave-batch-size-cap          |
| governance.wave.backpressure.cap_reduced         | INFO     | oya-governance-wave-pipeline-backpressure-respect |
| governance.wave.evidence_ledger.missing          | BLOCKER  | oya-governance-wave-evidence-ledger         |
| governance.wave.predecessor_graph.cycle          | BLOCKER  | oya-governance-wave-sequencing              |
| governance.wave.multispectrum.verdict_missing    | BLOCKER  | oya-governance-wave-sequencing              |
| governance.wave.cross_lane.contract_missing      | BLOCKER  | oya-governance-wave-sequencing              |
| governance.wave.agent_fleet.over_budget          | BLOCKER  | oya-governance-wave-sequencing              |

## SLO Implications

`microservices/governance/wave-sequencing/slos/wave.openslo.yaml`:

- `wave_descriptor_validation_p95`: ≤ 5 s.
- `wave_midpoint_checkpoint_p95`: ≤ 30 s (queries lane PR state, substance
  bar verdicts, evidence ledger state).
- `wave_close_verification_p95`: ≤ 120 s.
- `wave_in_flight_density_p95`: ≤ 12 PRs per lane per the cap from
  `feedback_pipeline_clog_gotchas_2026_05_17.md`.
- `wave_off_plan_rate`: ≤ 15% of waves trigger mid-wave off-plan events;
  breach triggers a council-foundry review of cap calibration.
- `wave_abort_rate`: ≤ 5% of waves require abort; breach triggers a
  doctrine-review postmortem.

## Migration Path / Phased Rollout

- **Phase 0 (T-0, this ADR Proposed)**: descriptors authored for in-flight
  waves W1..W4 per D-10.
- **Phase 1 (T+3 days)**: `oya-governance-wave-batch-size-cap` lane lands
  in shadow mode.
- **Phase 2 (T+7 days)**: lane upgraded to BLOCKER for new wave openings.
- **Phase 3 (T+14 days)**: mid-wave checkpoint automation lives; off-plan
  events emit to audit chain.
- **Phase 4 (T+21 days)**: full evidence-ledger and multispectrum-verdict
  gating active; this ADR eligible for promotion per ADR-0327.

## Failure Modes + Recovery

### F-1: Wave exceeds cap due to late-discovered scope

Mid-wave a critical artifact is discovered that pushes the wave above its
cap. Recovery: invoke the D-7 re-plan, either deferring the new artifact to
a successor wave or aborting the wave with a successor declared.

### F-2: Cross-lane predecessor never lands

A predecessor wave in another lane stalls. Recovery: the dependent wave
remains in `awaiting_predecessor` state; if the wait exceeds 14 days, the
council-foundry authors a re-plan that either removes the dependency
(via contract evolution) or accepts the wait (and updates SLOs).

### F-3: Backpressure cap reduction starves a wave

Pipeline backpressure cuts the wave's remaining cap to zero. Recovery: the
wave's `planned_landed_at` extends automatically by 50%; if a second
backpressure event triggers another reduction, the wave aborts and is
restarted with a fresh descriptor that explicitly inherits the artifacts
already landed.

### F-4: Multispectrum reviewer-agent unavailable at wave close

The signing reviewer agent is offline at wave-close time. Recovery: the
wave enters `awaiting_multispectrum_signature` state; a backup reviewer
agent (declared in the wave descriptor) may sign within 48 hours; if the
48-hour window elapses, the wave is rolled back to `in-flight` and a
new mid-wave checkpoint is run.

### F-5: Evidence ledger malformed

The EVIDENCE.md ledger fails the structure check. Recovery: the wave
cannot transition to `landed` until the ledger is fixed; an audit event
of class `governance.wave.evidence_ledger.malformed` is emitted.

### F-6: Concurrent wave starvation

Two waves in adjacent lanes saturate the cross-wave global cap of 16
in-flight PRs and neither makes progress. Recovery: council-foundry
identifies the contending waves and triages by SLO impact and customer
priority; one wave receives a temporary cap-reduction to free
in-flight slots for the other; an `governance.wave.concurrent_starvation`
warning is emitted.

### F-7: Predecessor wave landed but its contracts regress

A predecessor wave lands successfully but a subsequent commit causes a
contract regression. Recovery: the dependent wave's contract dependency
check fails on its next admission attempt; the dependent wave enters
`awaiting_predecessor` until the regression is repaired; an
`governance.wave.contract_regression.detected` event is emitted with
the offending commit SHA.

### F-8: Wave descriptor renamed mid-flight

An operator renames a wave descriptor file mid-flight, breaking the
predecessor graph. Recovery: the wave-sequencing crate detects the
broken reference (the predecessor name no longer resolves), emits
`governance.wave.predecessor_graph.broken_reference`, and BLOCKs all
operations on the affected wave until the operator either restores the
original name or updates all references.

## Appendix A — Worked example: a dense Tier-1 wave landing

The W4-substance-bar-2026-05-20 wave (this very wave) is a worked
example. The wave's descriptor:

- `wave_name`: `w4-substance-bar-2026-05-20`.
- `wave_ambition`: "Codify substance bar, wave sequencing, anti-script,
  pricing anchors, residency, and promotion gates."
- `wave_lane`: `oya-governance-doctrine`.
- `wave_predecessors`: `[w3-g-keystone-doctrine-cluster]` (which
  delivered ADRs 0297-0321).
- `wave_artifacts`: 6 ADRs (0322-0327), all Tier-1.
- `wave_substance_density_class`: `dense`.
- `wave_planned_start_at`: 2026-05-20T00:00:00Z.
- `wave_planned_landed_at`: 2026-05-25T00:00:00Z (5-day window).

The wave exceeds the dense Tier-1 cap of 3 because it carries 6 ADRs;
the descriptor splits the wave into two sub-waves
`w4-substance-bar-α` (ADRs 0322-0324) and `w4-substance-bar-β` (ADRs
0325-0327) per D-10. Each sub-wave carries cap=3 and lands
sequentially with the α sub-wave gating β. Each sub-wave's evidence
ledger is finalised before its successor starts.

## Appendix B — SLO targets vs realised performance (forward-looking)

The doctrine's SLO targets are forward-looking; realised performance
will be measured against them once the lane is live. The W1 baseline
expectation:

- 90% of wave-open verifications complete in ≤4 s.
- Mid-wave checkpoint coverage 100% (every wave that reaches midpoint
  is checked).
- Wave-close verification false-positive rate ≤1% (measured over the
  trailing 30 days).
- Pipeline backpressure cap-reduction occurs in ≤10% of waves.

The council-foundry reviews these realised numbers monthly during the
first 90 days of the lane's operation; baseline expectations are
adjusted if they prove unreachable or trivially achievable.

## Appendix C — Cross-walk to the Foundry pipeline state machine

The wave doctrine sits above the per-PR state machine of ADR-0110 and
the merge-queue admission gate of ADR-0111. The cross-walk:

| Wave state                       | Foundry pipeline state                  |
|----------------------------------|-----------------------------------------|
| proposed                         | (no pipeline activity)                  |
| open                             | descriptor PR merged                    |
| awaiting_predecessor             | (no admissions for wave artifacts)      |
| in-flight                        | wave artifact PRs admitted              |
| awaiting_midpoint_checkpoint     | midpoint cron tick pending              |
| off-plan                         | re-plan PR awaited                      |
| awaiting_multispectrum_signature | all artifact PRs merged, verdicts open  |
| landed                           | wave promotion event emitted            |
| aborted                          | abort event emitted; successor declared |

A wave artifact PR's admission consults the wave's state; PRs are
refused admission when the wave is in `awaiting_predecessor` or
`aborted` state. The admission refusal carries a structured reason so
the author can correct the situation.

## Appendix D — Wave descriptor minimum example

```json
{
  "wave_name": "w4-substance-bar-alpha-2026-05-20",
  "wave_ambition": "Author ADR-0322 substance bar + ADR-0323 wave sequencing + ADR-0324 anti-script as a dense Tier-1 trio.",
  "wave_lane": "oya-governance-doctrine",
  "wave_predecessors": ["w3-g-keystone-doctrine-cluster-2026-05-20"],
  "wave_artifacts": [
    {"path": "docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md",
     "authority_tier": 1},
    {"path": "docs/decisions/ADR-0323-multi-wave-sequencing-doctrine.md",
     "authority_tier": 1},
    {"path": "docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md",
     "authority_tier": 1}
  ],
  "wave_substance_density_class": "dense",
  "wave_planned_start_at": "2026-05-20T00:00:00Z",
  "wave_planned_landed_at": "2026-05-25T00:00:00Z",
  "wave_evidence_ledger_path": "docs/waves/w4-substance-bar-alpha-2026-05-20/EVIDENCE.md",
  "wave_multispectrum_verdict_path": "evidence/waves/w4-substance-bar-alpha-2026-05-20/multispectrum.signed.json",
  "wave_promotion_criterion": "all_members_accepted_per_adr_0327",
  "wave_reviewer_agent_pool": [
    "reviewer-correctness-r03",
    "reviewer-security-r07",
    "reviewer-privacy-r05",
    "reviewer-substance-r12",
    "reviewer-performance-r11",
    "reviewer-accessibility-r02",
    "reviewer-operability-r09",
    "reviewer-observability-r08",
    "reviewer-cost-r04",
    "reviewer-naming-a01",
    "reviewer-documentation-a02",
    "reviewer-structure-a03",
    "reviewer-architecture-a04",
    "reviewer-dependency-a05",
    "reviewer-schema-a06",
    "reviewer-algorithm-a07"
  ],
  "wave_cross_lane_contract_dependencies": []
}
```

This example demonstrates a dense Tier-1 wave with the canonical 16
reviewer-agent pool (one per facet of multispectrum review v2.4.0).

## Verification

Named CI checks:

- `oya-governance-wave-sequencing/descriptor`
- `oya-governance-wave-sequencing/predecessor-graph`
- `oya-governance-wave-batch-size-cap`
- `oya-governance-wave-verification-cadence/midpoint`
- `oya-governance-wave-verification-cadence/close`
- `oya-governance-wave-evidence-ledger`
- `oya-governance-wave-pipeline-backpressure-respect`

Named crates:

- `oya-governance-wave-sequencing`
- `oya-governance-wave-batch-size-cap`
- `oya-governance-wave-verification-cadence`
- `oya-governance-wave-evidence-ledger`
- `oya-governance-wave-pipeline-backpressure-respect`

Verification fixtures: `tests/governance/wave-sequencing/` including a
synthetic five-wave DAG, a backpressure-triggered cap reduction scenario,
and a wave-replan happy path.

## Cross-References

### Other ADRs

- ADR-0091 (multispectrum review) — wave-cadence binding.
- ADR-0110 (changeset state machine) — wave is a higher-order state on
  top of the per-PR state machine.
- ADR-0111 (merge-queue projected state) — wave admission consumes
  projected-state queries.
- ADR-0112 (webhook-driven Foundry invocation) — wave events fire via
  webhook substrate.
- ADR-0113 (VCS orchestrator end-to-end) — wave operations bind to the
  VCS orchestrator's primitives.
- ADR-0130 (observability SLO-gated promotion) — wave SLOs live under
  the substrate.
- ADR-0131 (per-microservice flat layout) — wave-sequencing crate layout.
- ADR-0132 (suite dissolution) — wave-scoped suite ban honoured.
- ADR-0145 (inter-microservice communication reform) — direct gRPC
  invariants honoured across waves.
- ADR-0242..ADR-0255 (keystone bundle) — wave doctrine composes with
  keystone shapes (tenant, Cedar, MLS, packs, K8s, HLC, Intelligence).
- ADR-0263 (audit-event registry) — wave events land in the registry.
- ADR-0316 (capability tier) — capability-tier-wave coordination.
- ADR-0321 (B2B leader coverage) — wave-driven leader-coverage roll-out.
- ADR-0322 (substance bar) — substance bar applies per wave artifact.
- ADR-0324 (anti-script anti-template) — wave authoring must respect
  the anti-script doctrine.
- ADR-0327 (wave-3 completion criteria) — promotion gates consume waves.

### Standards

- `docs/standards/documentation-rigor.md` §3.2 density schedule.
- `docs/standards/multispectrum-review-v2.4.0.md` reviewer cadence.

### Microservices

- `microservices/governance/wave-sequencing/` — the substrate
  microservice that hosts the wave descriptors and verifies cadence.
- `microservices/foundry/dispatcher/` — agent-fleet dispatcher that
  honours D-8 budget.
- `microservices/audit-chain/` — event sink.
- `microservices/observability/` — SLO substrate.

### Journeys

- `journeys/foundry/jou-2026-05-20-open-a-wave/` — author-facing journey.
- `journeys/foundry/jou-2026-05-20-close-a-wave/` — promoter-facing journey.

### Specs

- `/specs/wave-sequencing-schema.json`
- `/specs/master-plan-sequencing.json` (updated to point at wave concept).

### Feedback notes consumed

- `feedback_go_with_original_ambition_2026_05_20.md`
- `feedback_docs_substance_not_scaffold_2026_05_20.md`
- `feedback_pipeline_clog_gotchas_2026_05_17.md`
- `feedback_automate_everything.md`
- `feedback_consensus_debate_spectrum_lens_subagents.md`
- `feedback_milestone_phase_hierarchy.md`
