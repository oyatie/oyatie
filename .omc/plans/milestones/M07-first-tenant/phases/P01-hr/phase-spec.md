---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P01-hr
status: Proposed
acceptance_lanes: []
entry_gate: 'M02b-substrate-schema-foundation complete; oya-ontology-entity-kernel
  ships;

  oya-workflow-engine-kernel ships; Postgres + Citus cell provisioned;

  `cargo check --workspace` clean on M02 substrate crates.

  '
exit_gate: "All IP acceptance gates green; `cargo nextest run -p oya-hr-*` 0 failures;\n\
  `oya gate validate lean-a2 --ms hr` exits 0; `oya gate validate ontology-type-registry\
  \ --ms hr`\nexits 0; `oya gate validate audit-chain --ms hr` exits 0;\nk6 smoke\
  \ p99 employee read \u226450 ms at 1k RPS;\ngrit done called on all P01 symbols;\
  \ ICM phase-handoff row emitted.\n"
depends_on:
- milestone: M02
  phase: P22-substrate-ready
  reason: Ontology + Workflow + Citus substrate must exist before HR domain can register
    entity types and emit events.
parallel_wave: 1
owner_team: council-enterprise
purpose: "Delivers the `oya-hr-*` µservice: the authoritative source for `Employee`, `Employment`, and `Organization` entities within a tenant."
---
# P01-hr: HR µservice — Employee lifecycle, KR compliance, Ontology entity registration

## Purpose

Delivers the `oya-hr-*` µservice: the authoritative source for `Employee`,
`Employment`, and `Organization` entities within a tenant. Establishes Korean
Labor Standards Act compliance ground-truth (classifications, record retention,
4대보험 enrollment fields) and registers `Employee` / `Employment` / `Department`
Object Types in the Ontology so downstream µservices (Payroll, Connect, Accounting)
can query without calling HR directly.

Advances Master Plan principle §3 (Workflow + Ontology as the adapter layer) by
emitting `EmployeeHired`, `EmploymentClassChanged`, `OffboardingInitiated`,
`EmployeeTerminated` events via the Workflow event bus rather than peer-to-peer
HTTP calls.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `hr` | `employee` | `oya-hr-employee-{kernel,domain,application,adapter,rest}` |
| `hr` | `employment` | `oya-hr-employment-{kernel,domain,application,adapter}` |
| `hr` | `organization` | `oya-hr-organization-{kernel,domain,application,adapter,rest}` |
| `hr` | `offboarding` | `oya-hr-offboarding-{domain,application}` |
| `hr` | `app` | `oya-hr-app` |

Naming justifications:

```
NAME: oya-hr-employee-kernel
JUSTIFICATION:
- microservice = hr: Human Resources µservice; registered in [workspace.metadata.oya.microservices]; ADR-0056 v4.1 flat BNF
- bc-tokens = employee: hr has multiple BCs (employee/employment/organization/offboarding); employee BC owns Employee entity + PersonProfile + EmployeeRepository port-trait; ADR-0056 v4.1 BC-optionality rule
- layer = kernel: pure EmployeeId/PersonId value types + EmployeeRepository/PersonProfileStore port-trait declarations; zero logic; zero I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-employee-domain
JUSTIFICATION:
- microservice = hr; bc-tokens = employee; layer = domain: Employee aggregate + invariants + hire/terminate/classify use-case logic calling through EmployeeRepository port; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-employee-application
JUSTIFICATION:
- microservice = hr; bc-tokens = employee; layer = application: HireEmployeeUseCase, UpdateClassificationUseCase, BulkImportUseCase orchestrating domain + emitting Workflow events via EventBus port; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-employee-adapter
JUSTIFICATION:
- microservice = hr; bc-tokens = employee; layer = adapter: PostgresEmployeeRepository (implements EmployeeRepository port) + OntologyEmployeeWriter (writes Employee Object Type); ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-employee-rest
JUSTIFICATION:
- microservice = hr; bc-tokens = employee; layer = rest: Axum HTTP handlers for /employees CRUD; maps HTTP ↔ application commands; no business logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-employment-kernel
JUSTIFICATION:
- microservice = hr; bc-tokens = employment: employment BC owns Employment aggregate + EmploymentClassification 8-class enum (ADR-0126) + EmploymentRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-organization-kernel
JUSTIFICATION:
- microservice = hr; bc-tokens = organization: organization BC owns Department/Team/CostCenter entities + OrgRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-offboarding-domain
JUSTIFICATION:
- microservice = hr; bc-tokens = offboarding: offboarding BC owns OffboardingRequest entity + severance trigger logic; single binary at domain layer → BC token required for multi-BC ms; ADR-0056 v4.1 BC-optionality
- layer = domain: business logic only; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-hr-app
JUSTIFICATION:
- microservice = hr; bc-tokens: OMITTED — composition-root binary assembles all BCs; ADR-0056 §"BC optionality: omit for single composition-root"
- layer = app: main.rs + DI wiring; ADR-0056 §"Layer semantics"
- exemptions: none
```

### Out-of-scope

- HRIS bulk import SFTP integration — deferred to M03/P06 (Application onboarding) pending ADR-XXXX on protocol choice.
- Employment contract PDF generation (Typst) — deferred to M03/P06; Typst template owner TBD per PRD open question #2.
- SCIM 2.0 provisioning sync — deferred to M04 per PRD-application open question #2.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full HR µservice scaffold: DDL, kernel traits, domain entities, adapters, REST API, Cedar policies, Workflow events, Ontology types, load tests | pending | council-enterprise |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                           # exit 0
cargo build -p oya-hr-app --all-features                         # exit 0
cargo clippy -p oya-hr-employee-domain -p oya-hr-employment-domain -p oya-hr-organization-domain -- -D warnings  # exit 0
cargo nextest run -p oya-hr-employee-domain                      # exit 0; includes test_employment_class_validation
cargo nextest run -p oya-hr-employment-domain                    # exit 0
cargo nextest run -p oya-hr-organization-domain                  # exit 0
cargo nextest run -p oya-hr-offboarding-domain                   # exit 0
cargo deny check                                                 # exit 0
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms hr            # LEAN-A2: no imports from payroll/connect/accounting
oya gate validate lean-a1 --ms hr            # LEAN-A1: layer ordering enforced
oya gate validate port-location --ms hr      # port traits in kernel
oya gate validate layer-correctness --ms hr  # declared layer matches code shape
oya gate validate shardability --ms hr       # tenant_id partition key on all tables
```

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --ms hr   # EmployeeHired/EmploymentClassChanged/OffboardingInitiated/EmployeeTerminated registered
oya gate validate ontology-type-registry --ms hr    # Employee/Employment/Department Object Types registered
oya gate validate jurisdiction-overlay --ms hr      # jurisdiction_code=KR on all records
oya gate validate audit-chain --ms hr               # Ed25519 seal latency ≤1s per (tenant_id, period)
```

### Performance gate

```bash
# k6 smoke: p99 employee read ≤50 ms at 1k RPS
k6 run tests/load/smoke-hr-employee-read.js --env BASE_URL=http://localhost:8081
# Pass: http_req_duration{p(99)}<50; error rate <0.1%
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-hr-employee-kernel` | `kernel` | Yes — `EmployeeRepository`, `PersonProfileStore` | N/A |
| `oya-hr-employee-domain` | `domain` | N/A — calls through ports | N/A |
| `oya-hr-employee-application` | `application` | N/A | N/A |
| `oya-hr-employee-adapter` | `adapter` | N/A | Yes — `PostgresEmployeeRepository`, `OntologyEmployeeWriter` |
| `oya-hr-employee-rest` | `rest` | N/A | No direct adapter import |
| `oya-hr-employment-kernel` | `kernel` | Yes — `EmploymentRepository` | N/A |
| `oya-hr-employment-domain` | `domain` | N/A | N/A |
| `oya-hr-employment-application` | `application` | N/A | N/A |
| `oya-hr-employment-adapter` | `adapter` | N/A | Yes — `PostgresEmploymentRepository` |
| `oya-hr-organization-kernel` | `kernel` | Yes — `OrgRepository` | N/A |
| `oya-hr-organization-domain` | `domain` | N/A | N/A |
| `oya-hr-organization-adapter` | `adapter` | N/A | Yes — `PostgresOrgRepository` |
| `oya-hr-organization-rest` | `rest` | N/A | No direct adapter import |
| `oya-hr-offboarding-domain` | `domain` | N/A | N/A |
| `oya-hr-offboarding-application` | `application` | N/A | N/A |
| `oya-hr-app` | `app` | N/A | Unrestricted inward (DI wiring only) |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `employee` | `hr` | pending |
| `employment` | `hr` | pending |
| `organization` | `hr` | pending |
| `offboarding` | `hr` | pending |

---

## Grit Claim Symbols

```
crates/oya-hr-employee-kernel/src/ports.rs::EmployeeRepository
crates/oya-hr-employee-kernel/src/ports.rs::PersonProfileStore
crates/oya-hr-employment-kernel/src/ports.rs::EmploymentRepository
crates/oya-hr-employment-domain/src/employment_classification.rs::EmploymentClassification
crates/oya-hr-organization-kernel/src/ports.rs::OrgRepository
crates/oya-hr-offboarding-domain/src/offboarding_request.rs::OffboardingRequest
contracts/hr.openapi.yaml::createEmployee
contracts/hr.openapi.yaml::updateEmploymentClassification
docs/standards/bounded-contexts.md::hr.employee
```

TTL: `--ttl 3600` per IP; re-claim if exceeding.
Fallback: ICM topic `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
# At phase start
icm store \
  -t context-oyatie \
  -c "Phase P01-hr started; milestone M03-first-paying-tenant; scope: hr µservice (employee/employment/organization/offboarding BCs); entry gate met: M02 substrate ships" \
  -i high \
  -k "M03,P01,phase-start,hr"

# At phase complete
icm store \
  -t context-oyatie \
  -c "Phase P01-hr complete; HR µservice shipped: 4 BCs, 8-class EmploymentClassification, KR jurisdiction overlay, Ontology Employee/Employment/Department types, Workflow events EmployeeHired/EmploymentClassChanged/OffboardingInitiated/EmployeeTerminated; grit symbols released; lanes green; next phase: P02-payroll" \
  -i high \
  -k "M03,P01,phase-complete,hr"
```

---

## References

- PRD: `docs/prds/hr.md`
- Bominal ADRs inherited: ADR-0125 (domain naming), ADR-0126 (employment classification), ADR-0132 (data pillars), ADR-0018 (tenancy RLS), ADR-0028 (audit chain)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_naming_justification.md`, `feedback_workflow_objectgraph_adapter_layer.md`
