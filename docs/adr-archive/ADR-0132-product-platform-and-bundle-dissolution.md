---
id: ADR-0132
status: Superseded
planning_impact: true
deciders: council-architecture, council-engineering, axis-foundry, axis-workspace, ops-sre-reliability
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-0705]
amended_by: [ADR-0245, ADR-0249, ADR-0316, ADR-0324, ADR-0334, ADR-0347, ADR-0362]
related: [ADR-0056, ADR-0105, ADR-0110, ADR-0115, ADR-0119, ADR-0122, ADR-0127, ADR-0139, ADR-0131, ADR-0135]
related_specs: [/specs/per-microservice-flat-layout.json]
session_context:
  authored: 2026-05-17
  parallel_session_caveat: "Parallel session merges before this ADR. ADR numbers 0125–0129 are claimed by parallel work (OP-11 audit, super-app, hyperscaler patterns, hyperscaler invariants, plan-schema). This ADR was renumbered 0126→0132 on 2026-05-17 to clear the collision. Legacy communications µservice topology is delegated to ADR-0135 (originally drafted as ADR-0126 in the oyatie 2026-05-17 session, renumbered 2026-05-18 to avoid collision with dev's ADR-0126 Employment classification); this ADR carries only the universal forward-policy."
bominal_source: "override — Bominal's bundled product groupings (legacy suite wrappers, dual-context wrappers, vertical groupings) are explicitly overridden by oyatie's flat-catalog doctrine per feedback_flat_product_catalog.md"
purpose: Forward-policy — no new bundle or vertical-grouping µservices anywhere in oyatie. Customer-facing language names concrete services and tenant/RBAC packaging; architecture still ships every new µservice as a flat single-concern µservice consistent with AWS / Google / Microsoft / Stripe per-service practice and ADR-0131 (per-microservice flat layout).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0132: No-grouping forward-policy (universal flat catalog)

## Status

Accepted — 2026-05-17. **Amended 2026-05-25 by ADR-0362**: the grandfather
clause (existing grouping wrappers "out of scope … remain as authored") is **superseded** by later flat-only governance, and the naming policy now retires product/module/service grouping wrappers in favor of concrete service, lib, kernel, application, and infrastructure names. The `no-grouping` lane this ADR specified is implemented for real as the `no-grouping` gate.

## Context

User directive 2026-05-17 established the flat microservice doctrine: grouping wrappers must not become architecture boundaries, and new work should ship as clean-architecture microservices.

Existing grouping wrappers in the repo (Tenant/RBAC packaging, workspace productivity composition, Tenant RBAC, Foundry, Workflow, Cloud) are tombstone or composition artifacts, not permission to create new grouping µservices. This ADR establishes the forward-policy preventing new grouping formations.

Legacy communications grouping dissolution into concrete services is owned by ADR-0135 (historical filename retains the old label); active architecture resolves the user-facing communication surfaces to messenger, community, and mail, without a grouping service. This ADR cross-references that decision but does not re-author it.

Industry precedent: AWS / Google / Microsoft / Oracle / Stripe ship per-surface services with independent operational ownership, not bundled architecture services. Customer-facing packaging must resolve to concrete services, tenant/RBAC entitlements, libs, kernels, applications, or infrastructure components that ship, scale, deploy, and SLO independently.

## Decision

Effective immediately and going forward, every new µservice in oyatie ships as a flat single-concern µservice under `microservices/<ms>/` per ADR-0131. The following patterns are **prohibited**:

- Creating a new `microservices/<bundle>/` folder that contains more than one user-facing concern (e.g., `microservices/connector/`, `microservices/workspace/`, `microservices/healthcare/`, `microservices/fintech/`).
- Authoring a new collection-spec wrapper that binds multiple flat µservices under a legacy suite-style path or name (legacy `/specs/products/<bundle>/suite.json` path retired 2026-05-18 per the specs/products → specs/microservices flatten).
- Authoring a new PRD or phase-spec that declares its scope as a multi-concern architecture bundle. Public/composition naming must resolve to concrete services, tenancy/RBAC entitlements, libs, kernels, applications, or infrastructure components; it must not become a deployable grouping µservice boundary.
- Authoring a new µservice whose name is an industry / vertical / sector / domain term (`healthcare`, `fintech`, `grc`, `ats`, `procurement`, `medical`, `financial-services`, etc.) when that µservice would contain more than one concern.

The following patterns are **mandatory**:

- One concern, one µservice, one `microservices/<ms>/` folder, one PRD.
- Cross-µservice composition flows through Workflow events and Ontology reads/writes, per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`.
- Categorization (e.g., `DomainTag = {Agentic, Dev, Business, Healthcare, SupplyChain, Delivery}` inside Workflow Studio) stays as metadata, NOT as a directory split.

Existing grouping wrappers (Tenant/RBAC packaging, workspace productivity composition, Tenant RBAC, Foundry, Workflow, Cloud) are not new architecture permission. Their refactor — if any — is owned by their respective teams under separate ADRs.

## Rejected alternatives

- **Dissolve all existing grouping wrappers in this ADR** — rejected for the original 2026-05-17 scope; refactor of existing grouping µservices is scheduled-for-distinct-tracked-work to per-team decisions.
- **Retain retired grouping labels as brand-layer concepts** — rejected by the 2026-05-28 naming update. Customer-facing language must name concrete services or tenant/RBAC packaging; retired grouping labels remain only in historical references and legacy path examples.
- **Allow new bundle µservices when the bundle has a single SLO target** — rejected. A single SLO across multiple user-facing concerns means one concern's regression breaks the others' SLO; that is the failure mode this ADR prevents.

## Consequences

### Positive

- New µservices land in a uniform shape; no bundle/grouping drift.
- Per-concern SLO + release pointer (per ADR-0139) is the universal pattern, not the exception.
- **Independent scaling.** Each concern scales on its own dimension without dragging unrelated concerns along: mail scales on mailbox-count + inbound-message-rate; calendar scales on event-write-rate; messenger scales on persistent-connection-count + message-rate; slo-engine scales on µservice-count × evaluator-cadence; ehr (future) scales on patient-count + FHIR-write-rate; payments (future) scales on transaction-rate + fraud-eval-rate. A bundled communications or workspace µservice would force these to scale together — wasting capacity on under-utilised surfaces while under-provisioning the bottleneck. Clean-architecture + flat-catalog enables horizontal-scaling efficiency at hyperscaler-grade. Per-pod HPA, per-µservice Mimir cardinality budgets, per-µservice cost budgets all flow from this.
- Future migrations of existing grouping wrappers have a clear target shape (this ADR's flat-µservice mandate); their per-team refactor ADRs reference this one as the destination.
- Forward-policy is enforceable as a CI lane.

### Negative

- The existing grouping µservices remain inconsistent with the forward-policy. Reviewers reading old grouping PRDs may be confused about which doctrine applies; the resolution is: composition naming must resolve to concrete services and tenancy/RBAC packaging, while ADR-0132 flat-µservice forward-policy applies to new µservices.

### Migration cost

This ADR adds zero migration cost: existing grouping µservices are explicitly out of scope (per 2026-05-17 user directive "we won't worry about suites yet"). The cost is exclusively forward-prevention (one new BLOCKER CI lane). Per-µservice refactor costs, if and when scheduled by their owning team, follow the per-µservice migration tooling at `/specs/microservice-migration-tooling.json` and the cost classes in ADR-0131 §"Migration cost quantification".

### Operational

- **New CI lane: `oya-governance-no-grouping`** (BLOCKER on `dev`). Refuses:
  - New `microservices/<name>/` folder whose name matches a banned bundle/industry/vertical pattern AND which contains more than one user-facing concern.
  - New collection-wrapper spec files using legacy suite-style names or paths (legacy `/specs/products/<name>/suite.json` path retired 2026-05-18 per the specs/products → specs/microservices flatten).
  - New PRDs whose `microservice` frontmatter value matches a banned-pattern term and whose scope covers >1 concern.
  Pattern detail in `/specs/per-microservice-flat-layout.json` §`validator_rules`.
- Existing grouping wrapper files (`/specs/tenant-rbac-packaging.json`, `/specs/microservices/tenant-rbac.json`, ADR-0029, etc. — relocated 2026-05-18 from legacy grouping paths) are explicitly exempted by allowlist in the lane until their owning team refactors them.

## Clean Architecture Impact

This ADR is structural / policy; layer assignments, dependency direction, and port-location rules are unchanged. ADR-0131 (per-microservice flat layout) remains the authority for folder shape; this ADR is its sibling that forbids new grouping-shaped µservices.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none |
| `per-microservice-layout` (per ADR-0131) | Reinforced | this ADR's `no-grouping` lane is a sibling enforcement |
| `no-grouping` (NEW) | New BLOCKER on dev | refuses grouping-shaped new µservices |

## Verification

- `cargo run -p oya-dev-cli -- gate validate no-grouping` — exit 0; no banned bundle names introduced by new µservices.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout` — exit 0 (ADR-0131 sibling lane).
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.

## References

- ADR-0056: BNF v4.1 naming.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0135 (originally drafted as ADR-0126 in the oyatie 2026-05-17 session; renumbered 2026-05-18 to avoid collision with dev's ADR-0126 Employment classification): Legacy communications dissolution; active architecture resolves to messenger, community, and mail services, and this ADR cross-references but does not re-author it.
- ADR-0127 (parallel session 2026-05-17): Portfolio hyperscaler pattern enforcement.
- ADR-0139: Agentic SLO-gated promotion (per-microservice release pointers depend on this ADR's flat-µservice mandate).
- ADR-0131: Per-microservice flat layout (the layout authority; this ADR is its enforcement sibling for grouping-prevention).
- `feedback_flat_product_catalog.md`: "Everything is shared; flat product catalog."
- 2026-05-28 naming update: retired grouping labels are replaced by concrete service, lib, kernel, application, infrastructure, and tenancy/RBAC packaging names; architecture grouping remains forbidden.
- Industry: AWS / Google / Microsoft / Oracle / Stripe per-service shipping precedent.
