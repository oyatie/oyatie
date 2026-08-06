---
id: ADR-0180
status: Superseded
deciders: council-architecture, axis-observability, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0706]
related: [ADR-0042, ADR-0093, ADR-0128, ADR-0139, ADR-0160]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
renumber_note: "Originally allocated ADR-0174 in PR #143 Fix-L round 2; renumbered to ADR-0180 after a multi-stage rebump because ADR-0174-0177 were concurrently allocated by Fix-J / Fix-K agents."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0180 — SLO composition + inheritance arithmetic

## Status

Accepted (2026-05-18). Authored as part of PR #143 Fix-L anti-hyperscaler pattern audit round 2.

## Context

ADR-0139 (agentic SLO-gated promotion) requires every µservice to declare SLOs in `microservices/<ms>/slos/*.openslo.yaml` before promotion past dev. ADR-0139 does NOT declare how child-µservice SLOs aggregate into parent-product SLOs.

Without composition arithmetic, two failure modes occur silently:

1. **Aggregate impossibility** — a parent product claims 99.95% availability while one of its blocking child µservices claims 99.5%. The parent target is unreachable; the operator finds out only after the parent SLO is breached in production.
2. **Hidden drift** — a child µservice degrades from 99.9% to 99.8% (still inside its own SLO if the SLO drifted). The parent product silently breaches because nothing computes the composed budget.

Hyperscaler precedents:

- **Google SRE Workbook** — "Embedded SLO Hierarchy"; serial-call composition `parent ≤ product_of(children)`; parallel-call composition uses min/max patterns.
- **AWS Service Quotas** — hierarchical limits with explicit parent → child derivation.
- **Datadog SLO composition** — explicit SLO objects compose; UI surfaces the math.
- **Stripe internal SLO orchestrator** — per-API SLOs roll up to product SLOs with declared composition.

## Decision

Oyatie declares **SLO composition arithmetic** as a first-class manifest concern. Every parent product (Workflow Studio, Foundry, Super-App, etc.) declares its composition rule; every blocking child µservice's SLO is verified to satisfy the parent's budget.

### Composition rules

1. **Serial composition (call chain).** Parent SLO availability ≤ `product(children_in_chain.availability)`.
2. **Parallel composition (any-of).** Parent SLO availability ≤ `1 - product(1 - child.availability)`.
3. **Critical-path composition (must-all-succeed).** Parent SLO availability ≤ `min(children.availability)` (the weakest link).
4. **Latency composition.** Parent P99 ≤ `sum(serial-children.P99)` + `max(parallel-children.P99)`.
5. **Error budget composition.** Parent error budget ≤ `sum(children's error budget consumed share, weighted by call ratio)`.

### Per-product `slos/composition.openslo.yaml`

Every parent product declares:

```yaml
apiVersion: openslo/v1
kind: SLOComposition
metadata:
  name: workflow-studio-availability
spec:
  parent_slo:
    objective: 99.95
    metric: availability
  composition_kind: critical_path        # or serial | parallel
  children:
    - microservice: workflow-engine
      slo_ref: workflow-engine/availability
      call_ratio: 1.0
      criticality: blocking
    - microservice: ontology
      slo_ref: ontology/availability
      call_ratio: 0.6
      criticality: blocking
    - microservice: governance
      slo_ref: governance/availability
      call_ratio: 1.0
      criticality: blocking
```

### Gate enforcement

`oya-check-slo-composition-feasibility` lane validates:

- Every parent product ships exactly one `composition.openslo.yaml`.
- Every blocking child SLO satisfies the composed budget given the declared composition_kind.
- An impossible parent (e.g., parent 99.95% with a blocking child at 99.5%) blocks promotion.
- The gate runs DEFERRED initially; STRICT mode lands after the manifest backfill ships.

### Auto-rollback integration

Where ADR-0160 progressive-delivery-flagger surfaces a child SLO regression, the parent-product composition gate computes whether the parent's budget is breached. If breached, Flagger receives the auto-rollback signal at the parent's promotion frontier (not just the child's).

## Alternatives considered

### A. Leave SLO composition implicit (status quo)
- **Pros:** zero new artifact.
- **Cons:** silent aggregate impossibility; silent drift; matches no hyperscaler practice.
- **Rejected.**

### B. Composition arithmetic only at promotion gate (no per-product file)
- **Pros:** less authoring surface.
- **Cons:** loses the design-time check; promotion-time-only check is too late; the composition needs to be a contract.
- **Rejected.**

### C. Per-product composition file (accepted)
- **Pros:** design-time contract; gate validates feasibility; operator UI surfaces the math; auto-rollback integrates.
- **Cons:** authoring cost per parent product (~5-10 files).
- **Accepted.**

### D. Tool-specific composition (e.g., Datadog SLO composition)
- **Pros:** managed surface.
- **Cons:** vendor-specific; violates ADR-0121 portability invariant.
- **Rejected.**

## Consequences

### Positive

1. **Design-time feasibility check** — impossible parent SLOs are caught at gate time, not at production breach time.
2. **Composition becomes a contract** — child µservice SLO drift surfaces against the parent budget.
3. **Auto-rollback gains a parent-level signal** — Flagger rollback triggers at the parent's promotion frontier when child drift breaches the composed budget.
4. **OpenSLO-compatible** — composition.openslo.yaml uses the OpenSLO v1 schema with the `SLOComposition` kind extension; portable to any OpenSLO-aware tool.
5. **Google SRE Workbook alignment** — explicit composition is the SRE Workbook recommendation.

### Negative

1. **Composition authoring cost** — per parent product file (~5-10 files for oyatie's product catalog).
2. **Composition arithmetic discipline** — engineers must distinguish serial vs parallel vs critical-path correctly.
3. **`SLOComposition` kind is an OpenSLO extension** — OpenSLO upstream is considering a `Composite` kind; oyatie's local kind aligns with the proposal but may rename when upstream lands.

### Operational

1. Every parent product (per `specs/products/*.json`) ships `slos/composition.openslo.yaml`.
2. `oya-check-slo-composition-feasibility` gate authored; DEFERRED mode initially.
3. Per-µservice `slos/*.openslo.yaml` cross-references the parent-product composition by `composition_ref`.
4. ADR-0139 promotion gate consumes the composition feasibility check before allowing promotion.
5. ADR-0160 Flagger config consumes the composition rollback signal at the parent's promotion frontier.

## References

- Google SRE Workbook — "Embedded SLO Hierarchy" + "Composing SLIs."
- OpenSLO v1 specification — https://openslo.com
- Datadog SLO composition guide.
- Stripe public engineering — per-API SLO rollup pattern.
- ADR-0042 observability stack (OTel + in-house UI).
- ADR-0093 latency-budget-reporter-rename.
- ADR-0128 hyperscaler-architecture-invariants.
- ADR-0139 agentic SLO-gated promotion.
- ADR-0160 progressive-delivery-flagger.
