---
doc_class: Template
template_id: TPL-DD
status: Accepted
date: 2026-05-12
purpose: |
  Google-style design doc. Authored **before** implementation for every non-trivial change. Lives alongside the phase index for phase-level designs; alongside the capability for capability-level designs. Reviewed before any IP claims a symbol.
enforcing_fitness_lane: governance-design-doc-shape  # advisory at draft; lane on lift
owner_team: council-architecture
related:
  - .omc/scratch/hyperscaler-best-practices-2026-05-12.md  # §Domain 1 Google design docs
  - docs/templates/adr-template-v2.md
  - docs/templates/implementation-plan-template.md
  - docs/DESIGN.md
adrs_cited:
  - ADR-0052  # inventory ledger (data model state machines)
  - ADR-0053  # sanctioned primitives (agent path)
length_cap: 200
doc_status: published
---

```yaml
# Required frontmatter
---
doc_class: DesignDoc
template_id: TPL-DD
design_doc_id: DD-NNNN-<slug>
title: "<imperative one-line>"
status: draft | in-review | accepted | implemented | superseded
date: YYYY-MM-DD
owner_team: <team-id>
co_owners: [<team-id>]
reviewers: [<role | agent-id>]
related_adrs: [ADR-####, ...]
related_ips: [IP-NNN-<slug>, ...]
supersedes: [DD-NNNN, ...]
superseded_by: [DD-NNNN, ...]
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# DD-NNNN: <imperative title>

## Problem

One paragraph. What is the user-visible (or operator-visible) pain? Cite the source: customer request, regulator obligation, postmortem action item, capacity projection, etc.

## Goals

Numbered list. Each goal is **measurable**. Bad: "make it faster." Good: "p99 latency < 200ms at 10K RPS sustained for 30 days."

1. <measurable goal>.
2. <measurable goal>.
3. <measurable goal>.

## Non-goals

Numbered list of items explicitly out of scope. **MUST** name them — silent non-goals cause scope drift.

1. <out-of-scope item> — handled by <other doc / future phase>.

## Background

Two-paragraph maximum. Why now? Prior art (internal: prior ADRs, prior design docs; external: industry references). Skip if context is in `docs/DESIGN.md`; cite instead.

## Detailed design

The core of the doc. Recommended structure:

### Component diagram

Inline Mermaid / D2 source — the visualization-as-code principle (Master Plan §2 principle 11). Never hand-drawn raster images.

```mermaid
graph LR
  Client --> Gateway
  Gateway --> Service
  Service --> Store[(Store)]
```

### Data model

Schema definitions, invariants, state-machine transitions. If a domain `State → State` machine, draw it. Cite `data_class` per `docs/PRIVACY-PROGRAM.md §2.2.1`. Inventory entries for migration-class state per ADR-0052.

### API surface

OpenAPI / Protobuf / AsyncAPI excerpt. Provider-neutral (Master Plan §2 principle 4). Provider-specific code lives in `oya-<context>-adapter-<provider>-*` crates only.

### Concurrency / consistency model

Linearizable / sequential / causal / eventual. Quorum if distributed. Conflict resolution if multi-writer.

### Failure modes + recovery

Enumerate failure modes (timeout, partial write, provider down, audit-chain unreachable, tenant boundary breach attempt). For each: detection signal, mitigation, recovery procedure (runbook link).

### Observability

SLI / SLO definitions per `docs/SLO-CATALOG.md`. Audit-chain topic(s) emitted. Trace span names. Metric names. Dashboard reference.

### Security + privacy

- Authn / authz: which Cedar policies; which capability tier; which residency boundary.
- `data_class` allowlist / blocklist.
- Threat model summary or link to `docs/templates/threat-model-template.md`.

### Performance + capacity

- Benchmark methodology (per `docs/QA-TEST-STRATEGY.md`).
- ≥ 2 stress scenarios (CONSTITUTION §Decision principles Do.9).
- Capacity projection (12-month).

## Alternatives considered

**MUST** include ≥ 2 viable alternatives with bounded pros/cons; if only 1 viable option survives, explicit invalidation rationale for the rest.

### Alternative A — <name>
- Pros.
- Cons.
- Reason rejected (or "selected — see Why chosen").

### Alternative B — <name>
- Pros, cons, reason.

## Why chosen

Map the selected design back to: goals (every goal addressed?); Master Plan principles (which inherited?); related ADRs (which decisions enforced?); the alternatives it beats and why.

## Open questions

Numbered list. Each item routes to: a future ADR, an IP follow-up, or an explicit `(advisory)` marker if deferred.

## Cross-references

- ADR(s) authored / amended: `ADR-####`. ADR-0052 (inventory), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim).
- Implementation Plan(s): `IP-NNN-<slug>`.
- Related design docs: `DD-NNNN`.
- Hyperscaler practice inherited: per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §<domain>`.
- Industry references: papers, RFCs, vendor docs.

## Pre-mortem (RECOMMENDED for deliberate consensus mode)

Three failure scenarios that would make this design retrospectively wrong:
1. <scenario> → detection signal → recovery action.
2. <scenario>.
3. <scenario>.
