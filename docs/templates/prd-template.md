---
doc_class: Template
template_id: TPL-PRD
status: Accepted
date: 2026-05-13
purpose: |
  Canonical PRD shape for every oyatie µservice. Implementation-ready: carries
  functional requirements, non-functional requirements, BC list under BNF v4.1,
  Workflow + Ontology integration points, acceptance criteria, and Bominal
  inheritance citations. An autonomous executor filling this template can scaffold
  the µservice without escalation.
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/INDEX.md
  - docs/templates/microservice-template.md
  - docs/standards/bounded-contexts.md
adrs_cited:
  - ADR-0056  # BNF v4.1
  - ADR-0106  # Ontology architecture
  - ADR-0103  # Workflow hexagonal
doc_status: published
---

```yaml
# Required frontmatter for every PRD file
---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-<µservice-name>    # e.g. PRD-hr, PRD-payroll, PRD-ontology
microservice: <µservice-name>  # kebab; registered in [workspace.metadata.oyatie.microservices]
status: Draft | Review | Accepted | Superseded
sales_segment: Healthcare | Enterprise | FinTech | Social | shared-substrate
# NOTE: sales_segment is GTM/marketing segmentation ONLY — not architectural grouping.
# Every µservice is flat in the catalog. A tenant enables any subset à-la-carte.
tier: B2B | B2C | internal
milestone_first_ship: M0X-<slug>
bominal_source:
  - ADR-####  # Bominal ADR(s) this PRD inherits from (1:1 translation)
---
```

# PRD-<µservice-name>: <µservice display name>

---

## Purpose

One to two paragraphs. What this µservice does. Which tenant problem it solves.
Where it sits in the flat µservice catalog (per `feedback_flat_product_catalog.md`).

State the Bominal inheritance baseline: "This µservice inherits from Bominal
ADR-#### (<title>), translated to oyatie glossary." Then note any oyatie overrides.

---

## Tenant Value

What the tenant gets from enabling this µservice. Frame as outcomes, not features.

- **Outcome 1**: <tenant-facing benefit>
- **Outcome 2**: <tenant-facing benefit>
- **Outcome 3**: <tenant-facing benefit>

---

## Functional Requirements

User-story style. Each story maps to at least one BC (§ Bounded Contexts) and
one Workflow event or Ontology write.

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | `<actor>` | `<action>` | `<outcome>` | `<bc-name>` | Must \| Should \| Could |

---

## Non-Functional Requirements

### Performance
- `<metric>`: `<target>` (e.g., "P99 API latency < 200 ms at 1000 RPS")

### Security
- JWT `tenant_id` claim enforced on every request (per Bominal ADR-0018 RLS posture).
- Cedar policy enforcement for cross-tenant data access (per ADR-0140 (retired per ADR-0145)).
- Per-tenant Postgres RLS; no cross-tenant query possible.

### Audit + Compliance
- All mutations emit an audit-chain event (Merkle/Ed25519 per Bominal ADR-0028).
- Jurisdiction overlay applied per tenant's `jurisdiction_code` (per ADR-0127).
- Regulatory corpus locked in `bominal-law/corpus.lock` (per ADR-0128 + ADR-0190).

### Availability + SLO
- Availability target: `<X>%` (e.g., "99.9% monthly")
- RTO: `<duration>`; RPO: `<duration>`

### Data residency
- Tenant data pinned to tenant's assigned OCI region (per ADR-0117 cloud-native
  infrastructure).

---

## Bounded Contexts

BCs under BNF v4.1: `oyatie-<µservice>[-<bc-tokens>]-<layer>`.
Register each BC in `docs/standards/bounded-contexts.md`.

| BC name (kebab) | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `<bc-name>` | `oyatie-<ms>-<bc>-{domain,application,infrastructure,rest,grpc,...}` | <one-line> | `<Entity1>`, `<Entity2>` |

Naming justification block for EACH BC's crate family (mandatory per
`feedback_naming_justification.md`):

```
NAME: oyatie-<µservice>-<bc>-<layer>
JUSTIFICATION:
- microservice = <kebab-token(s)>: <rationale; ADR-0056 v4.1 flat BNF>
- bc-tokens = <kebab-token(s)>: <rationale; ADR-0056 v4.1 BC-optionality rule>
- layer = <layer>: <ADR-0056 §"Layer semantics">
- exemptions claimed: <none | cite exception>
```

Layer mapping per BC (per `feedback_clean_architecture_requirements.md` §1):

| BC | kernel | domain | application | adapter | presentation |
|---|---|---|---|---|---|
| `<bc-name>` | `oyatie-<ms>-<bc>-kernel` | `oyatie-<ms>-<bc>-domain` | `oyatie-<ms>-<bc>-application` | `oyatie-<ms>-<bc>-adapter` | `oyatie-<ms>-<bc>-{rest,grpc}` |

Port traits declared in kernel for each BC (zero business logic; zero I/O):

| Port trait | Kernel crate | Implemented in |
|---|---|---|
| `<RepositoryTrait>` | `oyatie-<ms>-<bc>-kernel` | `oyatie-<ms>-<bc>-adapter` |
| `<ServiceTrait>` | `oyatie-<ms>-<bc>-kernel` | `oyatie-<ms>-<bc>-adapter` |

Cross-product rule: this µservice MUST NOT import any other product µservice
crate at any layer. All cross-product flows go through Workflow (events) or
Ontology (entity reads/writes). LEAN-A2 CI lane enforces.

CI lanes that must green:
- `buck2 test <pipeline-lean-a1-target>` — dependency-direction
- `buck2 test <pipeline-lean-a2-target>` — cross-product-refusal
- `buck2 test <pipeline-port-location-target>` — ports in kernel
- `buck2 test <pipeline-layer-correctness-target>` — layer enum match

---

## Integration via Workflow + Ontology

All cross-µservice integration flows through Workflow (action/orchestration
adapter) or Ontology (information adapter). Direct µservice-to-µservice calls
are prohibited (LEAN-A2 enforcement). Per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`.

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG name |
|---|---|---|---|
| `<EventType>` | `<trigger condition>` | `<µservice>` | `<sm-name>` |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action taken |
|---|---|---|---|
| `<EventType>` | `<µservice>` | `<bc-name>` | `<action>` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `<ObjectType>` | `<LinkType>` (or `-`) | `<bc-name>` | Ed25519 audit event emitted |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `<ObjectType>` | `<bc-name>` | `filter(tenant_id).where(<predicate>)` |

---

## Competitive Benchmark

Industry leaders this µservice targets parity with. Cite primary-source
research (competitor docs, public APIs, third-party reviews). Required before
µservice graduates from Proof-Ladder L4 → L5 (per `feedback_quality_performance_scalability_bar.md`).

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| `<competitor>` | `<product>` | `<feature list>` | `<URL / doc ref>` |

Key parity gaps to close (ordered by priority):

1. `<gap>` — target: `<spec>`
2. `<gap>` — target: `<spec>`

---

## Performance Targets

Concrete targets testable via load tests (k6 / locust / vegeta) in impl plans.
Per `feedback_quality_performance_scalability_bar.md` §"Performance bar":

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Read API latency | `<ms>` | `≤200ms` | `<ms>` | Ontology Function reads ≤50ms per ADR-0107 |
| Write API latency | `<ms>` | `≤200ms` | `<ms>` | Action Types |
| Throughput per cell | — | `10k+ req/s` | — | Sharding to 100k+ aggregate |
| Event propagation lag | — | `<1s` | — | Outbox → consumer per Bominal ADR-0028 |
| Audit chain seal latency | — | `<1s` | — | Per (tenant, period) per ADR-0028 |
| Tenant onboarding | — | `≤5 min` | — | Self-serve SaaS per ADR-0118 |

Error budget:
- Monthly error budget: `<X>%` (e.g., 0.1% = 4.4 h/month)
- SLO burn-rate alarm: `<N>x` (e.g., 5x burn rate triggers page)
- Error budget policy: `<link to runbook>`

---

## Horizontal Scalability

Declares scalability posture per Bominal ADR-0019 runtime catalog +
ADR-0009 cell architecture. Mandatory per `feedback_quality_performance_scalability_bar.md`.

**State strategy** (per ADR-0019 enum):
`stateless | postgres | object-storage | persistent-volume | mixed`

Rationale: `<one sentence>`

**Active-active compatibility**: `stateless-compatible | single-writer-compatible`

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max QPS | `<N>` | `<N>` | CPU > 70% or queue depth > `<N>` |
| Max concurrent users | `<N>` | `<N>` | Memory > 80% |
| Max storage per tenant | `<N>` GB | `<N>` TB | Storage > 80% |

Scale-out policy:
- Kubernetes HPA: scale on CPU `>70%`; min `<N>` replicas; max `<N>` replicas
- Kubernetes VPA: memory `<target>`
- Pre-warmed pool: `<N>` standby pods; cold-start budget `≤500ms` (ADR-0020)

Cross-region story:
- M03 launch: single KR region (OCI ap-seoul-1); residency locked per ADR-0117
- Post-M03: `<expansion plan>` — deferred, documented in `docs/ROADMAP.md`
- Cross-region replication required if domain is: Medical | Payments | Connect-Pro mail

Sharding:
- Postgres + Citus for tenant-bound state; `tenant_id` partition key enforced
- ClickHouse replicas for analytics/audit queries
- `check-shardability-cli` CI lane verifies partition key presence (M02 substrate phase)

---

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | `<measurable criterion>` | `<command or test name>` |

---

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | `<question>` | `<team-id>` | `ADR-####` or `YYYY-MM-DD` |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-#### | `<title>` | inherited |
| oyatie ADR-#### | `<title>` | Oyatie-specific |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
| ADR-0018 | Tenancy RLS posture | security baseline |
