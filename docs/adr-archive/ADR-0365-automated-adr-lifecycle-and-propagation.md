---
id: ADR-0365
status: Superseded
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0364, ADR-0363, ADR-0247]
planning_impact: true
milestone: M-PLANNING-SSOT
depends_on: [ADR-0364]
door: one-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/masterplan.json]
deliverables:
  - id: ADR-0365-D1
    description: "Automated ADR authoring pipeline: best-practice-research -> planning-and-task-breakdown/ralplan/plan -> consensus -> ADR (generative template), wired as a governed workflow."
    exit_criteria: "an ADR can be produced end-to-end by the pipeline; the ADR carries an evidence[] block citing research + a consensus record."
    verified_by: "oya gate validate adr-provenance"
  - id: ADR-0365-D2
    description: "Propagation engine: an accepted ADR regenerates the masterplan + every artifact named in its affected_surfaces (trickle-down)."
    exit_criteria: "`oya gen propagate <ADR>` regenerates masterplan + affected_surfaces; idempotent."
    verified_by: "oya gen propagate --check"
  - id: ADR-0365-D3
    description: "Propagation drift gate: every affected_surfaces artifact is consistent with the ADRs that drive it (committed == regenerated)."
    exit_criteria: "gate fails when an affected doc is hand-edited out of sync with its ADR."
    verified_by: "oya gate validate propagation-drift"
  - id: ADR-0365-D4
    description: "ADR-provenance gate: a planning_impact ADR must cite best-practice-research evidence + a consensus record; no ADR bypasses the pipeline."
    exit_criteria: "gate fails an ADR missing evidence[]/consensus provenance."
    verified_by: "oya gate validate adr-provenance"
  - id: ADR-0365-D5
    description: "Door-classification autonomy gate: `door: two-way` ADRs auto-merge on green; `door: one-way` require founder sign-off (the agent autonomy boundary)."
    exit_criteria: "merge automation honors door; one-way ADR cannot auto-merge."
    verified_by: "oya gate validate decision-door"
  - id: ADR-0365-D6
    description: "COE->gate flywheel: every incident/agent-failure Correction-of-Error must terminate at a missing gate/spec/automation and emit a new gate (never 'human/agent error')."
    exit_criteria: "COE template + gate rejecting a human-error root cause; each COE links a new/updated gate."
    verified_by: "oya gate validate coe-to-gate"
purpose: Make the decision LIFECYCLE itself enforced and automated. An ADR is the output of a governed pipeline (research -> consensus-plan -> ADR) and, once accepted, propagates automatically to every dependent doc including the masterplan. Nothing that can be generated is hand-maintained; nothing that can be enforced is left to discipline.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0365: Automated ADR lifecycle — research → consensus → ADR → auto-propagate

## Status
Accepted — 2026-05-26.

## Context
ADR-0364 made the masterplan a generated projection of the ADR log. The remaining gap: the ADR
*itself* is still hand-authored, and propagation to other docs is manual. The doctrine: **everything
that can be enforced is a gate; everything that can be generated is generated.** A decision should be
*born* from evidence + consensus and *propagate* itself — so the decision log can never silently
diverge from research, from the roadmap, or from the docs it governs. (Synthesizes the 2026-05-26
hyperscaler research: AWS PR-FAQ / one-way-vs-two-way-door / COE-to-correction; Google design-doc→ADR
funnel + generated-docs; the KEP generative model from ADR-0364.)

## Decision

### 1. ADR authoring is an automated pipeline
A decision flows: **`best-practice-research` → `planning-and-task-breakdown` / `ralplan` / `plan`
(multi-perspective consensus) → ADR** in the ADR-0364 generative template. The ADR records its
provenance: an `evidence[]` block (research citations) + a consensus record (planner/architect/critic
verdicts). For a *new* service/lane, a PR-FAQ (does this belong?) precedes the pipeline.

### 2. Acceptance auto-propagates (trickle-down)
On acceptance, `oya gen propagate <ADR>` regenerates the masterplan (ADR-0364) **and every artifact
named in the ADR's `affected_surfaces`** (specs, catalogs, contract registry, doc reference pages).
`affected_surfaces` is the propagation map. Generated artifacts are build output — never hand-edited.

### 3. Everything enforced (the gates)
- **adr-provenance** — a `planning_impact` ADR must cite research evidence + a consensus record; no ADR
  bypasses the pipeline.
- **propagation-drift** — every `affected_surfaces` artifact == its regenerated form (extends the
  ADR-0364 masterplan-drift gate to all dependent docs).
- **decision-door** — `door: two-way` (reversible) ADRs auto-merge on green CI (agent-autonomous);
  `door: one-way` (irreversible) require founder sign-off. This is the explicit agent-autonomy
  boundary (Amazon one-way/two-way-door).
- **coe-to-gate** — every Correction-of-Error must terminate at a missing gate/spec/automation (never
  "human/agent error") and emit a new gate. Agent mistakes become deterministic gates — the
  self-improvement flywheel.

### 4. Self-hosting / dogfood
The pipeline runs on the substrate (git + Jenkins + GitHub, ADR-0363); `oya` owns the gates +
generators. This ADR was authored manually as the bootstrap; subsequent planning_impact ADRs flow
through the pipeline it defines.

## Rejected alternatives
- **Hand-authored ADRs + manual doc updates** — the drift this ADR eliminates.
- **Auto-merge all ADRs** — rejected: irreversible (one-way-door) decisions need a human; hence the
  door-classification gate.
- **Free-form incident notes** — rejected: COEs must convert to gates, or failures recur.

## Consequences
- Positive: decisions are evidence-grounded, consensus-reviewed, and self-propagating; the docs/
  masterplan/contracts cannot drift from the decision log; agent autonomy is bounded + explicit;
  failures compound into gates.
- Negative/cost: build the pipeline wiring + 4 gates + the propagation engine (D1–D6). The pipeline
  must not become heavyweight (lean ADRs; the research/consensus steps are bounded).
- Neutral: this is governance tooling, not a product code change.

## Verification
Per-deliverable `verified_by` above: `oya gate validate adr-provenance | propagation-drift |
decision-door | coe-to-gate` green; `oya gen propagate --check` idempotent; `oya gen masterplan --check`
(ADR-0364) green.

## References
Best-practice research 2026-05-26 (Google + AWS/Meta/MS/Netflix passes, banked in
docs/ideas/hyperscaler-practices-to-adopt.md): AWS Working-Backwards/PR-FAQ, one-way/two-way-door, COE;
Google design-doc→ADR + generated docs; KEP generative model. ADR-0364 (generative template +
masterplan generation), ADR-0363 (substrate), ADR-0247 (self-hosting).
