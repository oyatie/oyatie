---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-tenancy
microservice: tenancy
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M02b-substrate-ready
bominal_source:
  - ADR-0018  # tenancy + RLS posture (JWT tenant_id + Postgres RLS)
  - ADR-0011  # isolation-compatible operating model
  - ADR-0019  # runtime target metadata model
  - ADR-0117  # cloud-native infrastructure (OCI A1 → OKE stages)
  - ADR-0009  # cell architecture
doc_status: published
---

# PRD-tenancy: Tenancy shared substrate

---

## Purpose

Tenancy is a shared substrate µservice (always-on; underpin every other
product) that enforces multi-tenant isolation throughout the oyatie ecosystem.
It provides: tenant lifecycle management (create/activate/suspend/delete),
JWT `tenant_id` claim issuance, Postgres RLS policy generation and enforcement,
per-tenant cell assignment, and the data-boundary primitives consumed by every
other µservice.

Every µservice in the flat catalog depends on Tenancy. No µservice holds its
own tenant-isolation logic; they delegate entirely to Tenancy substrate
primitives.

Inherits from Bominal ADR-0018 (tenancy + RLS posture) 1:1. The `platform`
naming in Bominal is translated to `shared` per oyatie glossary
(`feedback_glossary_shared_not_platform.md`). Crate prefix: `tenancy-*`
(not `shared-tenancy-*` — BNF v4.1 flat; `tenancy` is the µservice name).

---

## Tenant Value

Tenancy is internal substrate; the "tenant" here means every other µservice
and every oyatie customer organization.

- **Zero-leakage isolation**: Postgres RLS policies generated per-tenant at
  provisioning time; no cross-tenant row ever returned regardless of query
  author.
- **Fast onboarding**: tenant activation in ≤5 min (ADR-0118 target) via
  self-serve API; schema migration + RLS policy + cell assignment automated.
- **Cell assignment**: new tenant assigned to the least-loaded cell in their
  jurisdiction region; blast-radius bounded per ADR-0009.
- **Suspend/delete with data retention**: tenant suspend preserves data per
  jurisdiction retention policy; delete triggers compliant data-erasure
  workflow.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Application µservice | create a new tenant record with jurisdiction, plan tier, and cell assignment | the tenant is isolated from provisioning moment | `tenant-lifecycle` | Must |
| FR-02 | Any µservice | call `tenancy-kernel` to validate `tenant_id` from JWT | requests are rejected at the infrastructure layer before business logic | `tenant-validation` | Must |
| FR-03 | Application µservice | activate a tenant (schema migration + RLS policy install) in ≤5 min | self-serve SaaS onboarding SLA met (ADR-0118) | `tenant-lifecycle` | Must |
| FR-04 | Platform operator | suspend a tenant (block new requests; preserve data) | non-paying or policy-violating tenants isolated without data loss | `tenant-lifecycle` | Must |
| FR-05 | Platform operator | delete a tenant (trigger compliant erasure; retain audit chain) | GDPR right-to-erasure + data-boundary compliance | `tenant-lifecycle` | Must |
| FR-06 | Any µservice infrastructure layer | obtain the Postgres RLS SET LOCAL statement for the current request's `tenant_id` | row-level isolation is automatic; no per-query filter needed | `rls` | Must |
| FR-07 | Cell orchestrator | query tenant → cell assignment for routing | load balancer routes tenant requests to the correct cell | `cell-assignment` | Must |

---

## Non-Functional Requirements

### Performance
- P99 tenant_id validation (hot path; called on every request): ≤5 ms (in-memory
  cache; Valkey-backed TTL 60 s).
- P99 tenant activation (schema migration + RLS install): ≤5 min (ADR-0118).
- P99 cell assignment lookup: ≤2 ms (read from Valkey).

### Security
- JWT `tenant_id` claim is the ONLY trust input; Tenancy never trusts
  caller-supplied tenant identity beyond the signed JWT.
- Postgres RLS `SET LOCAL app.current_tenant_id = $1` on every connection
  checkout; `FORCE ROW LEVEL SECURITY` on all tenant-bound tables.
- Tenant deletion: erasure workflow sealed in audit chain; erasure certificate
  generated for compliance evidence.
- Cedar policy: platform operators only can create/delete tenants; tenant
  admins can activate/suspend own tenant.

### Audit + Compliance
- Every tenant lifecycle event Ed25519-sealed per ADR-0028.
- Tenant data residency: `jurisdiction_code` pinned at creation; immutable.
- GDPR Article 17 (right to erasure): deletion workflow generates erasure
  certificate; audit chain proves completeness.
- Jurisdiction overlay per ADR-0127 + ADR-0140 (retired per ADR-0145).

### Availability + SLO
- 99.99% monthly (highest bar; Tenancy failure = all products fail for affected
  tenants).
- RTO ≤10 s; RPO ≤1 s (synchronous replication for tenant metadata).

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `lifecycle` | `tenancy-lifecycle-{domain,application,infrastructure,rest}` | Tenant CRUD; activation; suspension; deletion; erasure | `Tenant`, `TenantStatus` |
| `rls` | `tenancy-rls-{domain,application,infrastructure}` | RLS policy generation; SET LOCAL helper; Postgres adapter | `RlsPolicy` |
| `cell-assignment` | `tenancy-cell-assignment-{domain,application,infrastructure}` | Cell routing; load balancing; cell-health queries | `CellAssignment` |
| `kernel` | `tenancy-kernel` | Shared port-traits + `TenantId` value type consumed by ALL µservices | `TenantId`, `TenantContext` |

```
NAME: tenancy-kernel
JUSTIFICATION:
- microservice = tenancy: Tenancy shared substrate µservice; flat catalog; ADR-0056 v4.1; no "shared|vertical" bisection — tenancy IS the µservice name
- bc-tokens: OMITTED — kernel crate has a single concept (TenantId value type + TenantContext port-trait); ADR-0056 v4.1 BC-optionality rule
- layer = kernel: shared types + value objects consumed cross-layer; TenantId newtype + TenantContext trait; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `TenantActivated` | Activation complete | `application`, all enabled µservices | `tenant-onboarding-sm` |
| `TenantSuspended` | Platform operator suspends | `application`, all µservices | `tenant-lifecycle-sm` |
| `TenantDeletionCompleted` | Erasure workflow complete | `audit-chain` | `tenant-erasure-sm` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Tenant` | `AssignedToCell` → `Cell` | `cell-assignment` | Ed25519 on assignment |
| `TenantStatus` | `HasStatus` → `Tenant` | `lifecycle` | Ed25519 on every status change |

### Ontology reads

Tenancy is primarily written-to by the substrate; other µservices read `Tenant`
and `TenantStatus` to verify tenant validity before processing requests.

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Auth0 | Auth0 Organizations | Tenant isolation model; organization-level JWT claims; RBAC | https://auth0.com/docs/manage-users/organizations |
| AWS Cognito | Cognito User Pools + Identity Center | Multi-tenant JWT; tenant isolation; per-tenant config | https://docs.aws.amazon.com/cognito |
| Stripe | Stripe (platform model) | Tenant (account) lifecycle; isolated data; platform-level oversight | https://stripe.com/docs/connect |
| Neon | Neon Postgres (serverless branching) | Per-tenant Postgres schema isolation; instant provisioning; RLS | https://neon.tech/docs |

Key parity gaps:
1. **Sub-5-min tenant activation** (Neon/Auth0 parity): Postgres schema + RLS install must be automated; no manual DBA step.
2. **GDPR erasure certificate** (Stripe/Auth0 compliance parity): machine-readable erasure proof for each deleted tenant.
3. **Cell health routing** (AWS internal parity): routing table updated within ≤2 s of cell unhealthy signal.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| TenantId validation (hot path) | 0.5 ms | 5 ms | 10 ms | Valkey cache; cache-miss triggers DB |
| Tenant activation (schema + RLS) | 30 s | 5 min | 10 min | ADR-0118 ≤5 min p99 |
| Cell assignment lookup | 0.2 ms | 2 ms | 5 ms | Valkey read |
| RLS SET LOCAL per connection | 0.1 ms | 1 ms | — | In-process; no network hop |
| Audit chain seal per lifecycle event | — | 1 s | — | ADR-0028 |

Error budget: 0.01% monthly (highest bar). SLO burn-rate alarm: 2×.

---

## Horizontal Scalability

**State strategy**: `postgres` — tenant metadata in Postgres + Citus;
`tenant_id` is the shard key (circular); cell assignment table globally
replicated to all cells via logical replication.

**Active-active compatibility**: `single-writer-compatible` — tenant lifecycle
mutations serialized (one writer per tenant_id shard); reads active-active via
Postgres read replicas + Valkey.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max tenants per cell | 10,000 | 1,000,000 | Shard fill > 80% |
| TenantId validation QPS | 100,000 | 10,000,000 | Valkey memory > 80% |
| Concurrent activations | 10 | 1,000 | Worker pool > 80% |

Scale-out: validation path is stateless (Valkey + read replica); activation
workers HPA on queue depth; cell assignment globally replicated (small table).
Cross-region: M03 KR only; post-M03 global per ADR-0117 stages.

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Tenant activation completes in ≤5 min; RLS policy active post-activation | integration test `test_tenant_activation_e2e` |
| AC-02 | Cross-tenant query returns zero rows under RLS | `cargo nextest run -p tenancy-rls-domain --test rls_isolation` |
| AC-03 | TenantId validation p99 ≤5 ms at 100k RPS (Valkey cache hit) | k6 smoke; `http_req_duration{p(99)}<5` |
| AC-04 | `TenantActivated` event routed by Workflow to all enabled µservices | integration test `test_tenant_activated_workflow` |
| AC-05 | GDPR erasure: all tenant data deleted; erasure certificate generated | `cargo nextest run -p tenancy-lifecycle-domain --test gdpr_erasure` |
| AC-06 | Cell assignment: new tenant routed to least-loaded cell in correct jurisdiction | `cargo nextest run -p tenancy-cell-assignment-domain` |
| AC-07 | LEAN-A2: tenancy-kernel has no upstream µservice imports | `oya gate validate lean-a2 --ms tenancy` exits 0 |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Schema-per-tenant vs shared-schema-RLS: decision needed before M02 scaffolding | council-architecture | ADR-#### |
| 2 | Cell assignment algorithm: consistent hashing or weighted-round-robin? | council-infrastructure | M02/P01 |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0018 | Tenancy + RLS posture | inherited — primary authority |
| Bominal ADR-0011 | Isolation-compatible operating model | inherited |
| Bominal ADR-0019 | Runtime target metadata model | inherited |
| Bominal ADR-0009 | Cell architecture | inherited |
| Bominal ADR-0117 | Cloud-native infra scaling | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
