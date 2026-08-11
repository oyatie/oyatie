---
doc_status: published
id: ADR-0714
title: "ADR-0710 Accept/Reject is blocked on D-8 workload-boundary evidence"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: [ADR-0701, ADR-0710, ADR-0338, ADR-0379]
milestone: F1
deliverables:
  - id: ADR-0714-D1
    description: "Founder dependency edge: ADR-0710 MUST NOT be Accepted until D-8's conjunction evidence is answered (tenant-vs-tenant sharing AND tenant-vs-operator Tier-0 surfaces). Until then live admission law remains ADR-0701's carried ADR-0379/0338 gist (Kubewarden default admission substrate; Kyverno historical)."
    exit_criteria: "D-8 evidence packet published answering both boundary questions; founder then Accepts or Rejects ADR-0710 explicitly. Accepting ADR-0710 without that packet is forbidden by this ADR's Accept."
    verified_by: "oya-ci-required"
  - id: ADR-0714-D2
    description: "While ADR-0710 stays Proposed, runtime-tier admission remains an isolation control (not merely placement hygiene). Tier remap proposals (ADR-0713) raise the bar rather than assume ADR-0710 law."
    exit_criteria: "No encode PR treats ADR-0710 as Accepted substrate; citation-closure / live-resolution rules continue to rank Proposed as non-authority."
    verified_by: "oya-ci-required"
---
# ADR-0714: ADR-0710 Accept/Reject is blocked on D-8 workload-boundary evidence

## Status

**Proposed.** Deliberately not Accepted: this ADR records a founder dependency edge whose
satisfaction is **D-8 evidence for ADR-0710**, not a new admission design. It carries **no
implement authority** while Proposed, and it does **not** Accept ADR-0710 by implication.

Discovery input (not law): Round-2 synthesis in the local planning artifact
the Round-2 node forever-shape Discovery plan (local artifact id e6ec1a68) (founder F1(d)).

## Context

ADR-0710 is already a gated Proposed apex: its Status section states it is deliberately not
Accepted because clause **D-8** (tenant isolation comes from topology, not admission policy)
awaits workload-boundary evidence. D-8 requires a **conjunction** of two answers:

1. **Tenant-vs-tenant** — can two tenants share a node, hypervisor, or physical host?
2. **Tenant-vs-operator** — does any Tier-0 workload run on a node whose kernel, kubelet
   credentials, node identity, or CSI/secret material belong to the operator?

Control-plane separation answers neither. ADR-0710 already records that (1) answers yes under
today's hosted-default shared-substrate topology, so D-8 fails safe now.

Round-1 / Round-2 Discovery correctly forbids treating ADR-0710 as "direction binding" or live
law. Live admission substrate law remains the **ADR-0701 carried gist** of ADR-0379 / ADR-0338
(Kubewarden default; Kyverno historical) until ADR-0710 is explicitly Accepted or Rejected.

This ADR exists so agents cannot "assume 0710" when encoding tier remaps, Rust admission paths,
or Kyverno removals.

## Decision (proposed)

### D-1 — Accept/Reject gate for ADR-0710

On Accept of **this** ADR:

1. ADR-0710's Accept path is **blocked** until a D-8 evidence packet answers both boundary
   questions with measured topology facts (not aspirations).
2. Founder then **Accepts or Rejects ADR-0710** as an explicit act.
3. Until that act, **live law** for admission substrate remains ADR-0701's carried
   ADR-0379/0338 gist.

Rejecting **this** ADR means the dependency edge is refused as written — it does **not**
silently Accept ADR-0710.

### D-2 — Isolation-control posture while 0710 is Proposed

While ADR-0710 remains Proposed:

- Runtime-tier admission remains an **isolation control**, not merely "placement hygiene inside
  a trust domain" (that reframing is exactly what D-8 would authorize after evidence).
- ADR-0713 (tier rename) and related encode work MUST treat the higher bar as live: wrong
  RuntimeClass can breach isolation, not just hygiene.
- No PR may delete Kyverno/Kubewarden defaults solely by citing ADR-0710 while it is Proposed.

## Consequences

- Positive: prevents premature "VAP is law" encode; keeps D-8 honest; clear founder edge.
- Negative: admission migration stays blocked on evidence; dual Discovery/live tension continues
  until D-8 lands.
- Operational: reviewers refuse 0710-as-Accepted citations on authority surfaces (already gated).

## Rejected alternatives (proposed framing)

| Option | Why not |
|---|---|
| Treat ADR-0710 as direction-binding while Proposed | Violates gated-Proposed policy; invents law |
| Accept ADR-0710 without D-8 | Asserts unverified isolation posture |
| Collapse this edge into ADR-0713 | Different decision; tier rename ≠ admission Accept |

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept ADR-0714** | Dependency edge is law: 0710 Accept blocked on D-8 packet; live 0701 gist until then |
| **Reject ADR-0714** | Edge text refused; ADR-0710's own Status gate still applies independently |
| **(Later) Accept ADR-0710** | Only after D-8 evidence; not implied by Accepting ADR-0714 |
| **(Later) Reject ADR-0710** | Explicit; live 0701 gist continues |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`. Citing ADR-0710 as Accepted substrate remains forbidden until ADR-0710's own
frontmatter says Accepted.
