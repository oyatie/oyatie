---
id: ADR-0059
status: Superseded
doc_status: published
superseded_by: [ADR-0145]
---

# ADR-0059: Workflow + Ontology = ecosystem adapter layer

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0006, ADR-0035, ADR-0055, ADR-0058, ADR-0060, ADR-0062

---

## Context

In a flat catalog of independent microservices, cross-product integration must have a canonical channel. Without one, microservices develop direct imports of each other, creating hidden coupling that defeats independent deployability and horizontal scalability.

User instruction 2026-05-13: "adapters exist through workflow and object graph which is the key to our ecosystem." Combined with: "Workflow is the 'adapter' or glue that draws relationship between and within and object-graph is the actual information layer."

This is the inversion-of-control principle at the ecosystem scale.

**Naming justification:** "adapter layer" is clean-architecture vocabulary (Uncle Bob ch. 22); Workflow and Ontology are the two registered µservice names that together form this plane.

---

## Decision

**All adapters in the oyatie ecosystem exist through Workflow and Ontology.** Together they are the canonical adapter/integration surface. **Microservices never call each other directly.** Any cross-product integration flows through one of these two primitives:

- **Workflow** — the **action/orchestration adapter**. Cross-product or intra-product action flows: state machines, DAGs, approvals, escalations, SLA timers, automation, handoffs. Microservices publish typed events; Workflow routes them; consuming microservices subscribe.
- **Ontology** — the **information/data adapter** (= Palantir Ontology equivalent). Cross-product or intra-product data sharing: typed Object Types + Link Types + Action Types + Functions, with audit-chain provenance, RLS-enforced tenant isolation, jurisdiction overlays, pillar property (org/person), property tiers, DUB enforcement.

### Architecture rules

1. **No direct cross-microservice imports.** A microservice crate (e.g., `oya-medical-*`) MUST NOT import another microservice crate (e.g., `oya-pharmacy-*`) at any layer. `oya-check-architecture` (LEAN-A2) enforces this.

2. **All cross-microservice integration goes through Workflow OR Ontology:**
   - **Action/event flow** → Workflow. Microservice A emits a typed event; Workflow routes it (state machine / DAG / approval / SLA timer); Microservice B subscribes and reacts.
   - **Data sharing** → Ontology. Microservice A writes an Object/Link/Action; Microservice B reads via Ontology queries or Functions. No direct DB access across microservices.

3. **Intra-microservice orchestration may also use Workflow** for transitions, approvals, automation — Workflow is not exclusively cross-microservice.

4. **Intra-microservice data may also live in Ontology** for consistency, audit-chain, RLS.

5. **Workflow's adapter role** maps to clean-arch adapter layer: ports declared in `oya-workflow-kernel`; implementations in `oya-workflow-adapter` (transition-engine, state-store, event-bus bridge); workers in `oya-workflow-worker`.

6. **Ontology's adapter role** maps to clean-arch data-access adapter: typed-entity layer + provenance + RLS. Microservice domain code uses `oya-ontology-entity-kernel` ports; concrete adapters (`oya-ontology-entity-adapter`) implement.

### Canonical crate layout

```
Workflow µservice:
  oya-workflow-kernel              — port traits (WorkflowEngine, StateStore, EventBus)
  oya-workflow-domain              — state machine + DAG + escalation business logic
  oya-workflow-application         — use-case orchestrators (start, transition, escalate)
  oya-workflow-adapter             — transition-engine impl, state-store impl, event-bus bridge
  oya-workflow-worker              — background SLA timer / automation runners
  oya-workflow-rest                — Workflow HTTP API
  oya-workflow-grpc                — Workflow gRPC API
  oya-workflow-app                 — composition-root binary

Ontology µservice (per ADR-0055):
  oya-ontology-entity-kernel       — typed entity types + port traits
  oya-ontology-entity-domain       — entity business logic + invariants
  oya-ontology-entity-adapter      — Postgres + RLS impl
  oya-ontology-link-kernel
  oya-ontology-link-domain
  oya-ontology-action-kernel
  oya-ontology-action-domain
  oya-ontology-function-kernel
  oya-ontology-agent-gateway-kernel  — LLM tool-call ingress (Bominal ADR-0107)
  oya-ontology-agent-gateway-adapter
  oya-ontology-audit-chain-adapter   — chains to oya-audit-chain-kernel
  oya-ontology-pillar-kernel         — org-pillar + person-pillar (Bominal ADR-0132)
  oya-ontology-pillar-domain
  oya-ontology-rest
  oya-ontology-grpc
  oya-ontology-app
```

Both `workflow` and `ontology` are registered in `[workspace.metadata.oya.microservices]`.

### Ecosystem diagram

```
Microservice A    Microservice B    Microservice C
     │                 │                 │
     │ typed event     │ typed event     │
     └────────────────►│────────────────►│
              Workflow (action/orchestration adapter)
     │                 │                 │
     │ entity write/   │ entity read/    │
     │ link/action     │ query           │
     └────────────────►│────────────────►│
              Ontology (information adapter; Palantir-Ontology equiv)
     │                 │                 │
     (supporting substrates: tenancy, identity, audit-chain, eventing,
      secrets, observability, kms, policy, search, vector, ...)
```

---

## Consequences

### Quality / Performance / Scalability (per ADR-0062)

- **Workflow p99 target:** ≤200ms for any state transition (inherited from Bominal ADR-0107 Action Type target).
- **Ontology p99 target:** ≤50ms for read-only Functions; ≤200ms for Action Types (per `[[feedback-quality-performance-scalability-bar]]`).
- **Throughput:** both Workflow and Ontology must horizontally scale to 10k+ req/sec per cell baseline via stateless `application`/`rest`/`grpc` layers + sharded Postgres adapters.
- Event-driven design via outbox → Kafka KRaft (Bominal ADR-0116 inherited): sub-second event lag Workflow → subscriber.

**Clean architecture lanes enforcing the adapter layer rules:**

| Lane | What it enforces |
|---|---|
| `oya-shared-architecture-check-cli -- cross-product-refusal` (LEAN-A2) | Refuses any direct cross-microservice import; all cross-product integration MUST go through `oya-workflow-*` or `oya-ontology-*` |
| `oya-shared-architecture-check-cli -- port-location` | Port traits in `oya-workflow-kernel` / `oya-ontology-entity-kernel` (not in domain); impls in `oya-workflow-adapter` / `oya-ontology-entity-adapter` |
| `oya-shared-architecture-check-cli -- dependency-direction` | Inward-only flow enforced for all Workflow + Ontology crates |
| `oya-check-statelessness-cli` | `oya-workflow-application`, `oya-workflow-rest`, `oya-workflow-grpc`, `oya-ontology-rest`, `oya-ontology-grpc` have zero module-level mutable state |
| `oya-check-shardability-cli` | `oya-ontology-entity-adapter` declares `tenant_id` partition key + RLS; `oya-workflow-adapter` declares `tenant_id` on state tables |

Sealed port traits per Bominal ADR-0101 (inherited): `oya-workflow-kernel` port traits use `#[doc(hidden)] mod sealed` pattern.
Hexagonal microservice standard per Bominal ADR-0101/ADR-0103 (inherited).
Per `[[feedback-clean-architecture-requirements]]` §3 (port-location), §4 (cross-product rule), §5 (hexagonal standard).

### Positive

- Cross-microservice integration is mechanical, audited, and consent-gated.
- No microservice knows another's internals; independent deployability preserved.
- Ontology RLS + audit chain guarantee cross-product data sharing is tenant-isolated.
- Workflow state machines give cross-product orchestration a single observable trace.

### Negative

- All cross-microservice action flows route through Workflow; Workflow becomes a hot path — mitigated by horizontal scaling + stateless layers.
- Initial ergonomic cost: teams must model cross-product flows as events/subscriptions rather than direct calls.

---

## Related

- ADR-0001 (cohesion thesis — Workflow + Ontology are the integration plane)
- ADR-0006 (Ontology typed-entity layer)
- ADR-0035 (Workflow engine — state-machine + DAG hybrid)
- ADR-0055 (Ontology renamed to Ontology)
- ADR-0056 (BNF v4.1 — workflow and ontology are registered µservice names)
- ADR-0058 (Flat microservice catalog — no direct cross-microservice calls)
- ADR-0060 (Bominal-inheritance — Workflow is shared, not Corporate-owned; override #1 and #10)
- ADR-0062 (Quality/Performance/Scalability bar — p99 targets for Workflow + Ontology)
- `[[feedback-workflow-objectgraph-adapter-layer]]` — THE load-bearing architectural rule
- `[[feedback-workflow-is-shared]]` — Workflow placement in shared/*
- Bominal ADR-0103 (Workflow hexagonal, inherited)
- Bominal ADR-0106 (Ontology = Ontology in oyatie glossary)
- Bominal ADR-0107 (Ontology agent gateway, inherited)
