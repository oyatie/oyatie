---
doc_status: published
id: ADR-0715
title: "F1 Admission package: ADR-0710 Accept/Reject blocked on D-8; explicit Reject timebox allowed"
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
related: [ADR-0701, ADR-0710, ADR-0338, ADR-0379, ADR-0714]
milestone: F1
masterplan_work_item: MPV2-0056
deliverables:
  - id: ADR-0715-D1
    description: "F1 Admission package / founder dependency edge: ADR-0710 MUST NOT be Accepted until D-8's conjunction evidence is answered (tenant-vs-tenant sharing AND tenant-vs-operator Tier-0 surfaces). F1(d) MAY alternatively resolve as explicit Reject of ADR-0710 when hosted topology self-fails D-8, under a dated timebox trigger. Until Accept or Reject, live admission law remains ADR-0701's carried ADR-0379/0338 gist."
    exit_criteria: "Either (1) D-8 evidence packet published answering both boundary questions and founder Accepts or Rejects ADR-0710 explicitly, OR (2) timebox fires and founder records explicit Reject of ADR-0710 because D-8 self-fails in hosted topology. Accepting ADR-0710 without the packet is forbidden."
    verified_by: "oya-ci-required"
  - id: ADR-0715-D2
    description: "While ADR-0710 stays Proposed, runtime-tier admission remains an isolation control (not merely placement hygiene). Tier remap proposals (ADR-0714) raise the bar rather than assume ADR-0710 law."
    exit_criteria: "No encode PR treats ADR-0710 as Accepted substrate; citation-closure / live-resolution rules continue to rank Proposed as non-authority."
    verified_by: "oya-ci-required"
---
# ADR-0715: F1 Admission package — ADR-0710 Accept/Reject blocked on D-8

## Status

**Proposed.** Deliberately not Accepted: this ADR **is the F1 Admission package** (F1(d)). It
records a founder dependency edge whose satisfaction is **D-8 evidence for ADR-0710**, and it
explicitly permits F1(d) to resolve as **explicit Reject** of ADR-0710 when hosted topology
self-fails D-8, under a **timebox trigger**. It carries **no implement authority** while
Proposed, and it does **not** Accept ADR-0710 by implication.

Gate (outcome-determining, not parking): either the D-8 packet lands and founder Accepts or
Rejects ADR-0710, or the timebox fires and founder records explicit Reject. Both close the
edge.

Live masterplan anchor (planning only, not implement authority):
[`/specs/masterplan.json#masterplan_v2.work_items[MPV2-0056]`](../../specs/masterplan.json)
(F1(d) Admission package). Local Discovery artifact `e6ec1a68` is provenance only.

## Context

ADR-0710 is already a gated Proposed apex: its Status section states it is deliberately not
Accepted because clause **D-8** (tenant isolation comes from topology, not admission policy)
awaits workload-boundary evidence. D-8 requires a **conjunction** of two answers:

1. **Tenant-vs-tenant** — can two tenants share a node, hypervisor, or physical host?
2. **Tenant-vs-operator** — does any Tier-0 workload run on a node whose kernel, kubelet
   credentials, node identity, or CSI/secret material belong to the operator?

Control-plane separation answers neither. ADR-0710 already records that (1) answers yes under
today's hosted-default shared-substrate topology, so **D-8 fails safe now** in that topology.

This package exists so agents cannot "assume 0710" when encoding tier remaps, Rust admission
paths, or Kyverno removals — and so F1(d) has a durable masterplan work item and an explicit
Reject path when evidence will not reverse the hosted self-fail.

Bominal inheritance: no Bominal equivalent — oyatie override for admission Accept gating.

## Decision

### D-1 — Accept/Reject gate for ADR-0710 (F1 Admission package)

On Accept of **this** ADR:

1. ADR-0710's Accept path is **blocked** until a D-8 evidence packet answers both boundary
   questions with measured topology facts (not aspirations).
2. Founder then **Accepts or Rejects ADR-0710** as an explicit act.
3. **F1(d) MAY resolve as explicit Reject** of ADR-0710 without waiting for an aspirational
   topology change, when the founder records that D-8 **self-fails in hosted topology** (as
   ADR-0710 already notes for tenant-vs-tenant sharing under today's defaults). That Reject
   MUST be an explicit ADR-0710 status flip by founder process — this Proposed ADR does not
   flip it.
4. **Timebox trigger (machine-readable):** `MPV2-0056.decision_timebox.deadline_utc_date`
   is **2026-09-10**. If the D-8 evidence packet is not published by that date (or a successor
   dated founder amendment to that structured field), founder MUST either publish the packet or
   record **explicit Reject** of ADR-0710. The timebox does not silently Accept anything. The
   cross-artifact agreement gate evaluates the timebox schema and, once the UTC date has
   passed with the work item still open and no Accept/Reject evidence attached, fails closed.
5. Until Accept or Reject of ADR-0710, **live law** for admission substrate remains ADR-0701's
   carried ADR-0379/0338 gist (Kubewarden default; Kyverno historical).

Rejecting **this** ADR means the F1 Admission package edge text is refused as written — it does
**not** silently Accept ADR-0710. ADR-0710's own Status gate still applies independently.

### D-2 — Isolation-control posture while 0710 is Proposed

While ADR-0710 remains Proposed:

- Runtime-tier admission remains an **isolation control**, not merely "placement hygiene inside
  a trust domain" (that reframing is exactly what D-8 would authorize after evidence).
- ADR-0714 (tier rename) and related encode work MUST treat the higher bar as live: wrong
  RuntimeClass can breach isolation, not just hygiene.
- No PR may delete Kyverno/Kubewarden defaults solely by citing ADR-0710 while it is Proposed.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | Notes |
|---|---|---|
| D-8 evidence packet | create | Gate for ADR-0710 Accept path |
| `docs/decisions/ADR-0710-*.md` | status flip (founder only) | Explicit Accept or Reject — not this PR |
| `/specs/masterplan.json` `MPV2-0056` | update | Dated timebox trigger field / amendment |

No admission substrate encode from this ADR while Proposed.

### Integration via Workflow + Ontology

Not applicable — founder Accept/Reject gating decision.

### Positive

- Prevents premature "VAP is law" encode; keeps D-8 honest; clear Reject/timebox path for F1(d).

### Negative

- Admission migration stays blocked on evidence or explicit Reject; dual Discovery/live tension
  continues until one closes.

### Operational

- Reviewers refuse 0710-as-Accepted citations on authority surfaces (already gated).
- CI: `oya-ci-required`; no authority-surface citation while Proposed.

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

**Alternative 1 — Treat ADR-0710 as direction-binding while Proposed**
- Pros: faster migration narration.
- Cons: invents law; violates gated-Proposed policy.
- Reason rejected.

**Alternative 2 — Fold this edge into ADR-0710 only / delete F1(d) package**
- Pros: fewer ADRs.
- Cons: F1(d) needs a masterplan-anchored Admission package and explicit Reject/timebox; founder
  Round-4 keeps it as its own draft.
- Reason rejected.

**Alternative 3 — Accept ADR-0710 without D-8**
- Pros: unblocks VAP encode.
- Cons: asserts unverified isolation posture.
- Reason rejected.

**Alternative 4 — Infinite wait without timebox**
- Pros: no awkward Reject.
- Cons: parks F1(d); END-STATE-POLICY forbids ungated parking.
- Reason rejected: timebox + explicit Reject path.

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept ADR-0715** | F1 Admission package is law: 0710 Accept blocked on D-8 packet; explicit Reject + timebox paths authorized; live 0701 gist until 0710 closes |
| **Reject ADR-0715** | Edge text refused; ADR-0710's own Status gate still applies independently |
| **(Later) Accept ADR-0710** | Only after D-8 evidence; not implied by Accepting ADR-0715 |
| **(Later) Reject ADR-0710** | Explicit; including F1(d) hosted self-fail / timebox path; live 0701 gist continues |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`. Citing ADR-0710 as Accepted substrate remains forbidden until ADR-0710's own
frontmatter says Accepted.

## References

- Live masterplan: `MPV2-0056` in `/specs/masterplan.json#masterplan_v2.work_items`
- ADR-0710 (gated Proposed apex on D-8), ADR-0701 / ADR-0379 / ADR-0338 (live admission gist)
- ADR-0714 (isolation rename; higher bar while 0710 Proposed)
- Round-2 Discovery local artifact `e6ec1a68` — provenance only
- PR #1929 Round-4 amend (renumbered from draft ADR-0714; remains F1 Admission package)
