---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-hr
microservice: hr
status: Accepted
sales_segment: Enterprise
tier: B2B
milestone_first_ship: M03-first-paying-tenant
bominal_source:
  - ADR-0125  # domain naming canon (Employee/Employment/Person distinctions)
  - ADR-0126  # employment classification (8 classes)
  - ADR-0132  # data ownership pillars (org-pillar / person-pillar)
  - ADR-0018  # tenancy RLS posture
doc_status: published
---

# PRD-hr: HR µservice

---

## Purpose

The HR µservice manages the employee lifecycle for tenant organizations: hiring
onboarding, employment classification, organizational structure, and offboarding.
It is the authoritative source for the `Employee`, `Employment`, and
`Organization` entities within a tenant.

Inherits from Bominal ADR-0125 (domain naming canon: Tenant / Organization /
User / Person / Employee / Employment distinctions) and ADR-0126 (8-class
employment classification) 1:1 with oyatie glossary translation. Workflow and
Ontology integration follows `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`.

---

## Tenant Value

- **Unified employee record**: single source of truth for every employee across
  all enabled µservices (Payroll, Accounting, Connect-Pro, ATS).
- **KR compliance**: Korean Labor Standards Act classifications, mandatory
  reporting fields, hiredate / termination record retention.
- **Org chart + role graph**: hierarchical organization structure queryable via
  Ontology; downstream µservices read org relationships without calling HR directly.
- **Audit trail**: every employment mutation cryptographically sealed
  (Merkle/Ed25519); admissible for labor dispute resolution.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | HR admin | create an employee record with personal info, employment type (per ADR-0126 8 classes), and hire date | I have a single authoritative record | `employee` | Must |
| FR-02 | HR admin | update employment classification (full-time / part-time / contract / dispatch / daily / intern / apprentice / executive) | records reflect actual work arrangement for tax and compliance | `employment` | Must |
| FR-03 | HR admin | define organizational units (department, team, cost-center) and assign employees to them | Payroll and Accounting can roll up costs by org unit | `organization` | Must |
| FR-04 | HR admin | initiate offboarding workflow with final-day, severance calculation trigger, and benefit cessation | Payroll processes final paycheck; Connect-Pro revokes access | `offboarding` | Must |
| FR-05 | Employee | view my own employment record and payslips (read-only via Ontology) | I can verify my employment data without contacting HR | `employee` | Must |
| FR-06 | HR admin | bulk-import employees via CSV or HRIS integration | migration from legacy system completes in one operation | `employee` | Should |
| FR-07 | Auditor | export full audit chain for any employee record, timestamped and signed | labor authority investigations are satisfied without manual reconstruction | `audit` | Must |

---

## Non-Functional Requirements

### Performance
- P99 employee read API: ≤50 ms (Ontology Function read per ADR-0107).
- P99 employee write API: ≤200 ms.
- Bulk import: 10,000 records in ≤60 s.

### Security
- JWT `tenant_id` claim enforced on every request (ADR-0018).
- Cedar policy: employees can read own record only; HR admins scoped to their org.
- Person-pillar data (personal info) isolated from org-pillar data (employment terms)
  per Bominal ADR-0132; cross-pillar joins require explicit Cedar policy grant.

### Audit + Compliance
- Every employment mutation emits Ed25519 audit event sealed per (tenant_id, period)
  per ADR-0028; seal latency ≤1 s.
- Korean Labor Standards Act §17 (employment contract), §42 (record retention
  3 years): records immutable after seal; deletion requires explicit regulatory
  override workflow.
- Jurisdiction overlay `KR` applied per ADR-0127.

### Availability + SLO
- Availability: 99.9% monthly.
- RTO: ≤30 s per-cell; RPO: ≤5 s (outbox + cross-region replication readiness).

### Data residency
- M03: KR region only (OCI ap-seoul-1); `jurisdiction_code = KR`.

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `employee` | `hr-employee-{domain,application,infrastructure,rest}` | Employee entity lifecycle; personal info; employment terms | `Employee`, `PersonProfile` |
| `employment` | `hr-employment-{domain,application,infrastructure}` | Employment classification; contract terms; ADR-0126 8 classes | `Employment`, `EmploymentClass` |
| `organization` | `hr-organization-{domain,application,infrastructure,rest}` | Org units; hierarchy; cost-center assignment | `Department`, `Team`, `CostCenter` |
| `offboarding` | `hr-offboarding-{domain,application}` | Offboarding workflow trigger; severance; access revocation | `OffboardingRequest` |

```
NAME: hr-employee-domain
JUSTIFICATION:
- microservice = hr: Human Resources µservice; flat catalog; registered in [workspace.metadata.oya.microservices]; ADR-0056 v4.1
- bc-tokens = employee: HR has multiple BCs (employee / employment / organization / offboarding); employee BC owns Employee entity + PersonProfile; ADR-0056 v4.1 BC-optionality rule
- layer = domain: pure business logic; Employee entity + invariants + EmployeeRepository port-trait; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `EmployeeHired` | `Employment` record created + active | `payroll`, `connector`, `audit-chain` | `employee-lifecycle-sm` |
| `EmploymentClassChanged` | Classification updated | `payroll` | `payroll-recalc-sm` |
| `OffboardingInitiated` | HR admin triggers offboarding | `payroll`, `connector`, `accounting` | `offboarding-sm` |
| `EmployeeTerminated` | Offboarding complete + final day reached | `payroll`, `connector`, `accounting` | `offboarding-sm` |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `PayrollFinalizedForEmployee` | `payroll` | `offboarding` | Mark offboarding financial step complete |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Employee` | `EmployedBy` → `Organization` | `employee` | Ed25519 on every mutation |
| `Employment` | `HasClassification` → `EmploymentClass` | `employment` | Ed25519 on every mutation |
| `Department` | `PartOf` → `Department` (parent) | `organization` | Ed25519 on every mutation |

### Ontology reads

| Object Type | Read by | Query shape |
|---|---|---|
| `Employee` | `payroll`, `connector`, `accounting` | `filter(tenant_id).where(active=true)` |
| `Department` | `accounting` | `filter(tenant_id).costCenter(id)` |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| 더존비즈온 | iCUBE HR | KR Labor Standards Act fields; 4대보험 EDI integration fields; Korean org-chart | https://www.douzone.com |
| SAP | SuccessFactors HCM | Employment classification depth; org hierarchy; audit trail quality | https://www.sap.com/products/hcm |
| Workday | HCM | Ontology-style entity model; worker object; org graph | https://www.workday.com |
| Rippling | HR Cloud | Onboarding/offboarding automation depth; integration breadth | https://www.rippling.com |

Key parity gaps to close (ordered):
1. KR 4대보험 enrollment/disenrollment fields on `Employment` — target: full EDI-compatible field set per 더존 iCUBE schema.
2. Korean employment contract PDF generation (Typst template) per Labor Standards Act §17.
3. Org-chart multi-level hierarchy with cost-center rollup (Workday/SAP parity).

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Employee read (Ontology Function) | 10 ms | 50 ms | 100 ms | ADR-0107 ≤50ms p99 |
| Employee write | 30 ms | 150 ms | 300 ms | |
| Org chart query (depth ≤5) | 20 ms | 100 ms | 200 ms | |
| Bulk import 10k records | — | 60 s | — | streaming; backpressure |
| Audit chain seal | — | 1 s | — | per (tenant_id, period) ADR-0028 |

Error budget: 0.1% monthly (4.4 h/month). SLO burn-rate alarm: 5× triggers page.

---

## Horizontal Scalability

**State strategy**: `postgres` — tenant-bound employee records in Postgres + Citus;
`tenant_id` partition key on all tables; Postgres RLS enforced.

**Active-active compatibility**: `single-writer-compatible` (employment classification
changes are serialized per employee; single-writer per tenant shard).

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max QPS | 2,000 | 20,000 | CPU > 70% |
| Max employees per tenant | 10,000 | 500,000 | Storage > 80% |
| Max concurrent API users | 500 | 5,000 | Memory > 80% |

Scale-out: Kubernetes HPA on CPU >70%; min 2 replicas; max 20.
Cross-region: M03 KR only; post-M03 expansion documented in `docs/ROADMAP.md`.

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Employee CRUD round-trip; audit event emitted | `cargo nextest run -p hr-employee-domain` |
| AC-02 | ADR-0126 all 8 employment classes accepted; invalid class rejected | unit test `test_employment_class_validation` |
| AC-03 | `EmployeeHired` event routed by Workflow to payroll consumer | integration test `test_employee_hired_workflow` |
| AC-04 | Ontology `Employee` Object Type queryable by payroll µservice | `oya gate validate ontology-type-registry --ms hr` |
| AC-05 | LEAN-A2: no direct imports from payroll/connect/accounting | `oya gate validate lean-a2 --ms hr` exits 0 |
| AC-06 | p99 employee read ≤50 ms at 1k RPS | k6 smoke test; threshold `http_req_duration{p(99)}<50` |
| AC-07 | KR jurisdiction overlay applied; `jurisdiction_code=KR` on all records | `oya gate validate jurisdiction-overlay --ms hr` |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | HRIS integration protocol for bulk import: REST push or SFTP pull? | council-architecture | ADR-#### |
| 2 | Employment contract PDF generation: Typst template owner? | hr-team | M03/P02 |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0125 | Domain naming canon | inherited — Employee/Employment/Person entity distinctions |
| Bominal ADR-0126 | Employment classification | inherited — 8-class enum |
| Bominal ADR-0132 | Data ownership pillars | inherited — org-pillar vs person-pillar |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
