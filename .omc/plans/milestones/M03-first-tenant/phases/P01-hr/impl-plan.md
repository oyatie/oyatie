---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-hr
impl_plan_id: IP-P01-hr-full-scaffold
status: pending
owner: council-enterprise
blocked_by: []
acceptance_lanes:
  - cargo-check
  - cargo-build
  - cargo-clippy
  - cargo-nextest
  - cargo-deny
  - lean-a1
  - lean-a2
  - lean-a3
  - lean-a4
  - ontology-type-registry
  - workflow-event-registry
  - audit-chain
  - jurisdiction-overlay
  - k6-smoke
---

# IP-P01-hr-full-scaffold: HR µservice — DDL, domain entities, port traits, adapters, REST API, Cedar policies, Workflow events, Ontology types, load tests

## Intent

Scaffolds the complete `oya-hr-*` µservice: Postgres DDL with Citus distribution + RLS + outbox tables for all 4 BCs; Rust kernel port traits; domain entities (Employee, Employment with 8-class enum, Department/Team/CostCenter, OffboardingRequest); adapter implementations (PostgresEmployeeRepository, OntologyEmployeeWriter); REST API (`/v1/employees`, `/v1/organizations`); Cedar policy pack; Protobuf event schemas + Kafka topic names; OpenAPI contract; K6 load tests. Establishes KR jurisdiction overlay and ADR-0126 8-class `EmploymentClassification` as the single canonical enum.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-hr-employee-kernel/Cargo.toml` | create | `[package]` + deps: `async-trait`, `serde`, `uuid` |
| `crates/oya-hr-employee-kernel/src/lib.rs` | create | `pub mod ports; pub mod types;` re-exports |
| `crates/oya-hr-employee-kernel/src/types.rs` | create | `EmployeeId(Uuid)`, `PersonId(Uuid)`, `TenantId(Uuid)` newtype wrappers |
| `crates/oya-hr-employee-kernel/src/ports.rs` | create | `EmployeeRepository`, `PersonProfileStore` sealed port traits |
| `crates/oya-hr-employment-kernel/Cargo.toml` | create | `[package]` + deps |
| `crates/oya-hr-employment-kernel/src/lib.rs` | create | module declarations |
| `crates/oya-hr-employment-kernel/src/types.rs` | create | `EmploymentId(Uuid)`, `EmploymentClassification` 8-class enum |
| `crates/oya-hr-employment-kernel/src/ports.rs` | create | `EmploymentRepository` sealed port trait |
| `crates/oya-hr-organization-kernel/Cargo.toml` | create | `[package]` + deps |
| `crates/oya-hr-organization-kernel/src/lib.rs` | create | module declarations |
| `crates/oya-hr-organization-kernel/src/types.rs` | create | `OrganizationId(Uuid)`, `DepartmentId(Uuid)`, `CostCenterId(Uuid)` |
| `crates/oya-hr-organization-kernel/src/ports.rs` | create | `OrgRepository` sealed port trait |
| `crates/oya-hr-employee-domain/Cargo.toml` | create | deps: `oya-hr-employee-kernel` |
| `crates/oya-hr-employee-domain/src/lib.rs` | create | `pub mod employee; pub mod person_profile;` |
| `crates/oya-hr-employee-domain/src/employee.rs` | create | `Employee` aggregate + hire/terminate invariants |
| `crates/oya-hr-employee-domain/src/person_profile.rs` | create | `PersonProfile` value object (KR personal info fields) |
| `crates/oya-hr-employment-domain/Cargo.toml` | create | deps: `oya-hr-employment-kernel` |
| `crates/oya-hr-employment-domain/src/lib.rs` | create | `pub mod employment; pub mod classification;` |
| `crates/oya-hr-employment-domain/src/employment.rs` | create | `Employment` aggregate + effective-dated history |
| `crates/oya-hr-employment-domain/src/classification.rs` | create | `EmploymentClassification` impl: `income_tax_stream()`, `statutory_leave_eligible()`, `week_52_cap_applies()`, `as_korean()` |
| `crates/oya-hr-organization-domain/Cargo.toml` | create | deps: `oya-hr-organization-kernel` |
| `crates/oya-hr-organization-domain/src/lib.rs` | create | `pub mod department; pub mod cost_center;` |
| `crates/oya-hr-organization-domain/src/department.rs` | create | `Department` aggregate + parent hierarchy |
| `crates/oya-hr-organization-domain/src/cost_center.rs` | create | `CostCenter` value object |
| `crates/oya-hr-offboarding-domain/Cargo.toml` | create | deps: `oya-hr-employee-kernel`, `oya-hr-employment-kernel` |
| `crates/oya-hr-offboarding-domain/src/lib.rs` | create | `pub mod offboarding_request;` |
| `crates/oya-hr-offboarding-domain/src/offboarding_request.rs` | create | `OffboardingRequest` aggregate + severance trigger logic |
| `crates/oya-hr-employee-application/Cargo.toml` | create | deps: kernel + domain + `oya-workflow-engine-kernel` + `oya-ontology-entity-kernel` |
| `crates/oya-hr-employee-application/src/lib.rs` | create | `pub mod hire_employee; pub mod update_classification; pub mod bulk_import;` |
| `crates/oya-hr-employee-application/src/hire_employee.rs` | create | `HireEmployeeUseCase` — validates, calls `EmployeeRepository::save`, emits `EmployeeHired` via `EventBus` port |
| `crates/oya-hr-employee-application/src/update_classification.rs` | create | `UpdateClassificationUseCase` — validates 8-class enum, emits `EmploymentClassChanged` |
| `crates/oya-hr-employee-application/src/bulk_import.rs` | create | `BulkImportUseCase` — streaming; 10k records in ≤60s |
| `crates/oya-hr-employment-application/Cargo.toml` | create | deps: employment kernel + domain + workflow kernel |
| `crates/oya-hr-employment-application/src/lib.rs` | create | `pub mod offboarding_trigger;` |
| `crates/oya-hr-offboarding-application/Cargo.toml` | create | deps: offboarding domain + workflow kernel |
| `crates/oya-hr-offboarding-application/src/lib.rs` | create | `pub mod initiate_offboarding; pub mod finalize_offboarding;` |
| `crates/oya-hr-employee-adapter/Cargo.toml` | create | deps: employee kernel/domain/application + `sqlx` + `oya-ontology-entity-kernel` |
| `crates/oya-hr-employee-adapter/src/lib.rs` | create | `pub mod postgres_employee_repository; pub mod ontology_employee_writer;` |
| `crates/oya-hr-employee-adapter/src/postgres_employee_repository.rs` | create | `PostgresEmployeeRepository` implements `EmployeeRepository` |
| `crates/oya-hr-employee-adapter/src/ontology_employee_writer.rs` | create | `OntologyEmployeeWriter` writes `Employee` Object Type via `ObjectStore` port |
| `crates/oya-hr-employment-adapter/Cargo.toml` | create | deps: employment kernel + `sqlx` |
| `crates/oya-hr-employment-adapter/src/postgres_employment_repository.rs` | create | `PostgresEmploymentRepository` implements `EmploymentRepository` |
| `crates/oya-hr-organization-adapter/Cargo.toml` | create | deps: organization kernel + `sqlx` |
| `crates/oya-hr-organization-adapter/src/postgres_org_repository.rs` | create | `PostgresOrgRepository` implements `OrgRepository` |
| `crates/oya-hr-employee-rest/Cargo.toml` | create | deps: employee application + `axum` + `serde_json` |
| `crates/oya-hr-employee-rest/src/lib.rs` | create | `pub mod routes;` |
| `crates/oya-hr-employee-rest/src/routes.rs` | create | `/v1/employees` CRUD handlers |
| `crates/oya-hr-organization-rest/Cargo.toml` | create | deps: organization application + `axum` |
| `crates/oya-hr-organization-rest/src/routes.rs` | create | `/v1/organizations` CRUD handlers |
| `crates/oya-hr-app/Cargo.toml` | create | deps: all hr crates; `tokio`, `axum` |
| `crates/oya-hr-app/src/main.rs` | create | DI assembly + Axum server startup |
| `migrations/hr/001_hr_schema.sql` | create | Full DDL (see below) |
| `contracts/hr.openapi.yaml` | create | OpenAPI 3.1 spec for `/v1/employees`, `/v1/organizations`, `/v1/employments` |
| `proto/hr/events.proto` | create | Protobuf event schemas (see below) |
| `policies/hr/hr.cedar` | create | Cedar policy pack (see below) |
| `tests/load/smoke-hr-employee-read.js` | create | k6 smoke test: p99 ≤50ms at 1k RPS |
| `tests/load/smoke-hr-employee-write.js` | create | k6 smoke test: p99 ≤150ms at 500 RPS |
| `Cargo.toml` | update | Add all `oya-hr-*` crates to `[workspace.members]` |
| `docs/standards/bounded-contexts.md` | update | Register employee/employment/organization/offboarding BCs |

---

## Crate Naming

(All justifications provided in phase-spec.md §Scope; reproduced here for executor.)

```
NAME: oya-hr-employee-kernel
JUSTIFICATION:
- microservice = hr: Human Resources µservice; registered; ADR-0056 v4.1
- bc-tokens = employee: employee BC owns Employee + PersonProfile + EmployeeRepository port; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure EmployeeId/PersonId value types + EmployeeRepository/PersonProfileStore port declarations; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none
```

(Full justifications for all crates in phase-spec.md.)

---

## Code Shape

### `crates/oya-hr-employee-kernel/src/ports.rs`

```rust
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// Employee persistence port — implemented in oya-hr-employee-adapter
#[async_trait::async_trait]
pub trait EmployeeRepository: Send + Sync + sealed::Sealed {
    async fn find_by_id(&self, tenant: &TenantId, id: &EmployeeId)
        -> Result<Option<Employee>, HrError>;
    async fn save(&self, tenant: &TenantId, employee: &Employee)
        -> Result<(), HrError>;
    async fn find_active(&self, tenant: &TenantId)
        -> Result<Vec<Employee>, HrError>;
}

/// Person profile store port — implemented in oya-hr-employee-adapter
#[async_trait::async_trait]
pub trait PersonProfileStore: Send + Sync + sealed::Sealed {
    async fn find_by_id(&self, tenant: &TenantId, id: &PersonId)
        -> Result<Option<PersonProfile>, HrError>;
    async fn save(&self, tenant: &TenantId, profile: &PersonProfile)
        -> Result<(), HrError>;
}
```

### `crates/oya-hr-employment-kernel/src/types.rs`

```rust
/// Canonical 8-class employment classification (Bominal ADR-0126)
/// Statute: 대한민국.노동.근로기준법 + 파견법 + 기간제법
/// corpus_sha: <read from corpus.lock>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "employment_classification", rename_all = "snake_case")]
pub enum EmploymentClassification {
    /// 정규직 — open-ended; 근로소득; 4대보험 all; 연차 full; 52시간제 yes
    Regular,
    /// 계약직 — fixed-term; auto-converts to Regular after 2y (기간제법)
    FixedTerm,
    /// 단시간근로자 — part-time; pro-rata leave; ≥15h/wk for severance
    PartTime,
    /// 파견 — dispatched; employed by agency; 2y cap (파견법 §6)
    Dispatched,
    /// 도급 — subcontracted; host must not direct day-to-day
    Subcontracted,
    /// 프리랜서 — independent contractor; 사업소득; no 4대보험 (employer)
    Freelance,
    /// 인턴 — time-bounded; 기타소득 OR 근로소득 per subkind
    Intern,
    /// 임원 — corporate officer; registered on 법인등기부; severance per 임원퇴직금규정
    Officer,
}

impl EmploymentClassification {
    pub fn as_korean(&self) -> &'static str {
        match self {
            Self::Regular => "정규직",
            Self::FixedTerm => "계약직",
            Self::PartTime => "단시간근로자",
            Self::Dispatched => "파견",
            Self::Subcontracted => "도급",
            Self::Freelance => "프리랜서",
            Self::Intern => "인턴",
            Self::Officer => "임원",
        }
    }

    /// Income tax withholding stream (소득세법 시행령 §20/21)
    pub fn income_tax_stream(&self) -> IncomeTaxStream {
        match self {
            Self::Freelance => IncomeTaxStream::Business,  // 사업소득 3.3%
            Self::Intern => IncomeTaxStream::Other,        // 기타소득 (stipend subkind)
            _ => IncomeTaxStream::Employment,              // 근로소득
        }
    }

    /// 연차 accrual eligibility (근로기준법 §60)
    pub fn statutory_leave_eligible(&self) -> bool {
        !matches!(self, Self::Freelance | Self::Officer)
    }

    /// 52-hour weekly cap applicability
    pub fn week_52_cap_applies(&self) -> bool {
        matches!(self, Self::Regular | Self::FixedTerm | Self::PartTime
            | Self::Dispatched | Self::Subcontracted)
    }
}

/// Extra discriminant fields on Employment aggregate
pub struct EmploymentDiscriminants {
    /// set iff Dispatched — FK to dispatching organization
    pub dispatching_organization_id: Option<uuid::Uuid>,
    /// set iff Subcontracted — FK to contracting organization
    pub contracting_organization_id: Option<uuid::Uuid>,
    /// set iff Intern
    pub intern_subkind: Option<InternSubkind>,
    /// set iff Officer
    pub officer_register_kind: Option<OfficerRegisterKind>,
    /// required for PartTime; informative for others
    pub weekly_hours_committed: Option<rust_decimal::Decimal>,
}
```

### `crates/oya-hr-employee-domain/src/employee.rs`

```rust
/// Employee aggregate — Person × Organization × active Employment
/// Invariants:
/// - hire_date must be ≤ today
/// - organization_id must be within same tenant
/// - employment_classification must be a valid EmploymentClassification variant
pub struct Employee {
    pub id: EmployeeId,
    pub tenant_id: TenantId,
    pub person_id: PersonId,
    pub organization_id: OrganizationId,
    pub hire_date: chrono::NaiveDate,
    pub termination_date: Option<chrono::NaiveDate>,
    pub status: EmployeeStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub enum EmployeeStatus { Active, Terminated, OnLeave }

impl Employee {
    pub fn hire(
        tenant_id: TenantId,
        person_id: PersonId,
        organization_id: OrganizationId,
        hire_date: chrono::NaiveDate,
    ) -> Result<Self, HrError> { /* validates hire_date ≤ today */ }

    pub fn terminate(&mut self, termination_date: chrono::NaiveDate) -> Result<(), HrError> {
        /* validates termination_date ≥ hire_date; sets status = Terminated */
    }
}
```

---

## Postgres DDL

### migrations/hr/001_hr_schema.sql

```sql
-- ============================================================
-- HR µservice schema — Citus-ready; RLS; outbox
-- Jurisdiction: KR (OCI ap-seoul-1)
-- ADR-0018 (tenancy RLS), ADR-0028 (audit chain), ADR-0126 (8-class enum)
-- corpus.lock: 대한민국.노동.근로기준법 §17 (record retention 3yr), §42
-- ============================================================

CREATE SCHEMA IF NOT EXISTS hr;

CREATE TYPE hr.employment_classification AS ENUM (
    'regular', 'fixed_term', 'part_time', 'dispatched',
    'subcontracted', 'freelance', 'intern', 'officer'
);

CREATE TYPE hr.intern_subkind AS ENUM ('paid_employee', 'stipend_only');
CREATE TYPE hr.officer_register_kind AS ENUM ('inside', 'outside');
CREATE TYPE hr.employee_status AS ENUM ('active', 'terminated', 'on_leave');

-- Organizations (legal entities inside a tenant; ADR-0125)
CREATE TABLE hr.organizations (
    organization_id     uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    name                text NOT NULL,
    parent_organization_id uuid NULL REFERENCES hr.organizations(organization_id),
    kr_entity_kind      text NOT NULL CHECK (kr_entity_kind IN ('개인사업자','법인','비영리법인','공공기관')),
    tenant_tier         text NOT NULL CHECK (tenant_tier IN ('5인미만','sme','중견','대기업','공공')),
    jurisdiction_code   text NOT NULL DEFAULT 'KR',
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE hr.organizations ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.organizations
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_org_tenant ON hr.organizations (tenant_id);
-- Citus distribution (tenant_id shard key per ADR-0117)
-- SELECT create_distributed_table('hr.organizations', 'tenant_id');

-- Departments (part of org hierarchy)
CREATE TABLE hr.departments (
    department_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    organization_id     uuid NOT NULL REFERENCES hr.organizations(organization_id),
    name                text NOT NULL,
    parent_department_id uuid NULL REFERENCES hr.departments(department_id),
    cost_center_code    text NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE hr.departments ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.departments
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_dept_tenant_org ON hr.departments (tenant_id, organization_id);

-- Persons (human records; ADR-0125 Person ≠ User ≠ Employee)
-- Person-pillar data (ADR-0132); personal info encrypted at column level via KMS DEK
CREATE TABLE hr.persons (
    person_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    -- KR personal info fields (Labor Standards Act §17 required fields)
    full_name_encrypted bytea NOT NULL,         -- AES-256-GCM under tenant DEK
    national_id_encrypted bytea NULL,           -- 주민등록번호 encrypted
    birth_date          date NULL,
    gender              text NULL CHECK (gender IN ('M','F','X')),
    nationality         text NOT NULL DEFAULT 'KR',
    phone_encrypted     bytea NULL,
    address_encrypted   bytea NULL,
    jurisdiction_code   text NOT NULL DEFAULT 'KR',
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE hr.persons ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.persons
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_person_tenant ON hr.persons (tenant_id);

-- Employees (Person × Organization × active Employment; ADR-0125)
CREATE TABLE hr.employees (
    employee_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    person_id           uuid NOT NULL REFERENCES hr.persons(person_id),
    organization_id     uuid NOT NULL REFERENCES hr.organizations(organization_id),
    employee_number     text NULL,              -- tenant-assigned employee number
    hire_date           date NOT NULL,
    termination_date    date NULL,
    status              hr.employee_status NOT NULL DEFAULT 'active',
    jurisdiction_code   text NOT NULL DEFAULT 'KR',
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE hr.employees ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.employees
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_emp_tenant_org ON hr.employees (tenant_id, organization_id) WHERE status = 'active';
CREATE INDEX idx_emp_tenant_person ON hr.employees (tenant_id, person_id);
CREATE UNIQUE INDEX idx_emp_number ON hr.employees (tenant_id, employee_number) WHERE employee_number IS NOT NULL;

-- Employments (effective-dated role snapshots; ADR-0125)
-- ADR-0126: 8-class EmploymentClassification
-- Statute citations per corpus.lock:
--   classification = Dispatched: 대한민국.노동.파견법.제6조 (2yr cap)
--   classification = PartTime: 대한민국.노동.기간제법.제2조 (pro-rata leave)
--   severance: 대한민국.노동.근로기준법.제34조 (퇴직금 1yr/15hr threshold)
CREATE TABLE hr.employments (
    employment_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    employee_id         uuid NOT NULL REFERENCES hr.employees(employee_id),
    classification      hr.employment_classification NOT NULL DEFAULT 'regular',
    -- ADR-0126 discriminant fields
    dispatching_organization_id   uuid NULL,
    contracting_organization_id   uuid NULL,
    intern_subkind      hr.intern_subkind NULL,
    officer_register_kind hr.officer_register_kind NULL,
    weekly_hours_committed numeric(5,2) NULL,
    -- Employment terms
    title               text NOT NULL,
    department_id       uuid NULL REFERENCES hr.departments(department_id),
    manager_employee_id uuid NULL REFERENCES hr.employees(employee_id),
    salary_amount       numeric(15,2) NULL,
    salary_currency     text NOT NULL DEFAULT 'KRW',
    fte_pct             numeric(5,2) NOT NULL DEFAULT 100.0,
    effective_from      date NOT NULL,
    effective_to        date NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT chk_dispatched CHECK (
        classification != 'dispatched' OR dispatching_organization_id IS NOT NULL
    ),
    CONSTRAINT chk_subcontracted CHECK (
        classification != 'subcontracted' OR contracting_organization_id IS NOT NULL
    ),
    CONSTRAINT chk_intern CHECK (
        classification != 'intern' OR intern_subkind IS NOT NULL
    ),
    CONSTRAINT chk_officer CHECK (
        classification != 'officer' OR officer_register_kind IS NOT NULL
    ),
    CONSTRAINT chk_part_time CHECK (
        classification != 'part_time' OR weekly_hours_committed IS NOT NULL
    )
);
ALTER TABLE hr.employments ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.employments
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_emp_hist_employee ON hr.employments (tenant_id, employee_id, effective_from DESC);
CREATE INDEX idx_emp_hist_active ON hr.employments (tenant_id, employee_id)
    WHERE effective_to IS NULL;

-- Offboarding requests
CREATE TABLE hr.offboarding_requests (
    request_id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    employee_id         uuid NOT NULL REFERENCES hr.employees(employee_id),
    final_day           date NOT NULL,
    reason              text NOT NULL,
    severance_trigger   bool NOT NULL DEFAULT true,
    benefit_cessation_date date NULL,
    status              text NOT NULL DEFAULT 'initiated' CHECK (status IN ('initiated','payroll_finalized','completed','cancelled')),
    workflow_run_id     uuid NULL,              -- linked Workflow run
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE hr.offboarding_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hr.offboarding_requests
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Outbox table (per outbox pattern; Bominal ADR-0117)
CREATE TABLE hr.outbox (
    outbox_id           bigserial PRIMARY KEY,
    tenant_id           uuid NOT NULL,
    topic               text NOT NULL,          -- e.g. 'hr.EmployeeHired'
    key                 text NOT NULL,          -- e.g. employee_id
    payload             jsonb NOT NULL,
    published_at        timestamptz NULL,
    created_at          timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_hr_outbox_unpublished ON hr.outbox (created_at)
    WHERE published_at IS NULL;
```

---

## Protobuf Event Schemas

### proto/hr/events.proto

```proto
syntax = "proto3";
package hr.events;

// Kafka topic: hr.EmployeeHired (per-tenant namespace: hr.{tenant_id}.EmployeeHired)
message EmployeeHired {
    string tenant_id = 1;
    string employee_id = 2;
    string person_id = 3;
    string organization_id = 4;
    string hire_date = 5;          // ISO 8601 date
    string classification = 6;     // EmploymentClassification snake_case
    string jurisdiction_code = 7;  // "KR"
    int64  occurred_at_ms = 8;
    string workflow_run_id = 9;    // outbox correlation
}

// Kafka topic: hr.EmploymentClassChanged
message EmploymentClassChanged {
    string tenant_id = 1;
    string employee_id = 2;
    string employment_id = 3;
    string old_classification = 4;
    string new_classification = 5;
    string effective_date = 6;     // ISO 8601 date
    int64  occurred_at_ms = 7;
}

// Kafka topic: hr.OffboardingInitiated
message OffboardingInitiated {
    string tenant_id = 1;
    string employee_id = 2;
    string request_id = 3;
    string final_day = 4;
    bool   severance_trigger = 5;
    int64  occurred_at_ms = 6;
}

// Kafka topic: hr.EmployeeTerminated
message EmployeeTerminated {
    string tenant_id = 1;
    string employee_id = 2;
    string termination_date = 3;
    string reason = 4;
    int64  occurred_at_ms = 5;
}
```

---

## Cedar Policy Pack

### policies/hr/hr.cedar

```cedar
// HR Cedar policy pack — ADR-0132 (pillars), ADR-0018 (tenancy)

entity Tenant;
entity Organization in [Tenant];
entity Employee     in [Organization];
entity Employment   in [Employee];
entity Department   in [Organization];

// Principals
entity HrAdmin   in [Tenant] = { organization_id: String };
entity EmployeeUser in [Employee];
entity Auditor   in [Tenant];

// HR admin can manage employees within their org
permit (
    principal is HrAdmin,
    action in [Action::"CreateEmployee", Action::"UpdateEmployee",
               Action::"UpdateClassification", Action::"InitiateOffboarding"],
    resource is Employee
) when {
    resource.organization_id == principal.organization_id &&
    context.tenant_id == principal.tenant_id
};

// Employee can read own record only (FR-05)
permit (
    principal is EmployeeUser,
    action == Action::"ReadEmployee",
    resource is Employee
) when {
    resource == principal.employee_id
};

// Auditor can export audit chain (FR-07)
permit (
    principal is Auditor,
    action == Action::"ExportAuditChain",
    resource is Employee
) when {
    context.tenant_id == principal.tenant_id
};

// Person-pillar isolation (ADR-0132): org-admin cannot read person-pillar objects
forbid (
    principal is HrAdmin,
    action in [Action::"ReadPersonProfile"],
    resource
) when {
    resource.ownership_pillar == "person" &&
    principal != resource.assigned_hr_admin
};
```

---

## OpenAPI Contract (key endpoints)

```yaml
# contracts/hr.openapi.yaml (excerpt)
openapi: "3.1.0"
info:
  title: HR API
  version: "1.0.0"
paths:
  /v1/employees:
    post:
      operationId: createEmployee
      summary: Hire a new employee
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/HireEmployeeRequest' }
      responses:
        '201': { description: Employee created }
        '400': { description: Validation error (invalid classification, future hire_date) }
        '403': { description: Cedar policy denial }
  /v1/employees/{employee_id}/classification:
    put:
      operationId: updateEmploymentClassification
      summary: Update employment classification with effective date
      parameters:
        - name: employee_id
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/UpdateClassificationRequest' }
      responses:
        '200': { description: Classification updated; EmploymentClassChanged event emitted }
        '409': { description: Effective date conflict }
  /v1/organizations:
    post:
      operationId: createOrganization
    get:
      operationId: listOrganizations
components:
  schemas:
    HireEmployeeRequest:
      type: object
      required: [person_id, organization_id, hire_date, classification]
      properties:
        person_id: { type: string, format: uuid }
        organization_id: { type: string, format: uuid }
        hire_date: { type: string, format: date }
        classification:
          type: string
          enum: [regular, fixed_term, part_time, dispatched, subcontracted, freelance, intern, officer]
        dispatching_organization_id: { type: string, format: uuid }
        contracting_organization_id: { type: string, format: uuid }
        intern_subkind: { type: string, enum: [paid_employee, stipend_only] }
        officer_register_kind: { type: string, enum: [inside, outside] }
        weekly_hours_committed: { type: number }
```

---

## Acceptance Gates

```bash
# 1. Compile
cargo check -p oya-hr-employee-kernel -p oya-hr-employment-kernel -p oya-hr-organization-kernel --all-features  # exit 0
cargo check -p oya-hr-employee-domain -p oya-hr-employment-domain -p oya-hr-organization-domain --all-features  # exit 0
cargo check -p oya-hr-app --all-features  # exit 0

# 2. Build
cargo build -p oya-hr-app --all-features  # exit 0

# 3. Lint
cargo clippy -p oya-hr-employee-domain -p oya-hr-employment-domain -- -D warnings  # exit 0; 0 warnings

# 4. Tests
cargo nextest run -p oya-hr-employee-domain  # exit 0; includes test_employee_hire_invariants
cargo nextest run -p oya-hr-employment-domain --test test_employment_class_validation  # exit 0; all 8 classes valid; invalid rejected
cargo nextest run -p oya-hr-organization-domain  # exit 0
cargo nextest run -p oya-hr-offboarding-domain  # exit 0

# 5. Integration
cargo nextest run --test test_employee_hired_workflow  # exit 0; EmployeeHired event routed to payroll consumer

# 6. Supply chain
cargo deny check  # exit 0

# 7. LEAN checks
oya gate validate lean-a1 --ms hr  # dependency-direction
oya gate validate lean-a2 --ms hr  # cross-product-refusal; no imports from payroll/connect/accounting
oya gate validate port-location --ms hr  # ports in kernel
oya gate validate layer-correctness --ms hr
oya gate validate shardability --ms hr  # tenant_id shard key on all hr.* tables

# 8. Ontology + Workflow
oya gate validate ontology-type-registry --ms hr  # Employee/Employment/Department Object Types registered
oya gate validate workflow-event-registry --ms hr  # EmployeeHired/EmploymentClassChanged/OffboardingInitiated/EmployeeTerminated
oya gate validate jurisdiction-overlay --ms hr    # jurisdiction_code=KR on all records
oya gate validate audit-chain --ms hr             # Ed25519 seal latency ≤1s
```

---

## Test Plan

### Unit tests

| Test | File | Verifies |
|---|---|---|
| `test_employee_hire_invariants` | `oya-hr-employee-domain/src/employee.rs` | hire_date ≤ today; status = Active |
| `test_employment_class_validation` | `oya-hr-employment-domain/src/classification.rs` | All 8 classes accepted; invalid enum value rejected; dispatch requires dispatching_org_id |
| `test_employment_class_income_stream` | same | Regular→근로소득; Freelance→사업소득; Intern(stipend)→기타소득 |
| `test_employment_class_week_52_cap` | same | Regular/FixedTerm/PartTime=true; Freelance/Officer=false |
| `test_department_hierarchy` | `oya-hr-organization-domain/src/department.rs` | Parent-child hierarchy max depth 10; cycle detection |
| `test_offboarding_request_create` | `oya-hr-offboarding-domain/src/offboarding_request.rs` | final_day ≥ today; severance_trigger default true |
| `test_employee_repository_round_trip` | `oya-hr-employee-adapter/src/postgres_employee_repository.rs` | save → find_by_id returns same entity |

### Integration tests

| Test | Verifies |
|---|---|
| `test_employee_hired_workflow` | `HireEmployeeUseCase` → `EmployeeHired` event on Kafka topic `hr.{tenant_id}.EmployeeHired` |
| `test_ontology_employee_registration` | `OntologyEmployeeWriter` writes `Employee` Object Type; queryable via `ObjectStore` |
| `test_bulk_import_10k` | 10k employees imported in ≤60s; backpressure works; 0 errors |
| `test_audit_chain_seal` | Ed25519 seal emitted per (tenant_id, period); latency ≤1s |

### Load test

```javascript
// tests/load/smoke-hr-employee-read.js
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 100,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<50'],   // p99 ≤50ms — PRD target
    http_req_failed: ['rate<0.001'],    // error rate <0.1%
  },
};

export default function () {
  const tenantId = __ENV.TENANT_ID;
  const res = http.get(
    `${__ENV.BASE_URL}/v1/employees`,
    { headers: { Authorization: `Bearer ${__ENV.JWT_TOKEN}`, 'X-Tenant-Id': tenantId } }
  );
  check(res, { 'status 200': (r) => r.status === 200 });
}
```

```javascript
// tests/load/smoke-hr-employee-write.js — p99 write ≤150ms at 500 RPS
export const options = {
  vus: 50,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<150'],
    http_req_failed: ['rate<0.001'],
  },
};
```

---

## Clean Architecture Compliance

### Dependency direction

```
oya-hr-employee-rest
  └→ oya-hr-employee-application
       └→ oya-hr-employee-domain
            └→ oya-hr-employee-kernel
oya-hr-employee-adapter
  └→ oya-hr-employee-application → domain → kernel
     (implements kernel ports)
oya-hr-app (composition root)
  └→ all above layers (unrestricted inward)
```

Forbidden: no layer imports a layer closer to the edge. No product-to-product import.

### Port traits (kernel)

All `EmployeeRepository`, `PersonProfileStore`, `EmploymentRepository`, `OrgRepository`
declared in respective `*-kernel/src/ports.rs` with `sealed::Sealed` marker trait.
Zero business logic. Zero I/O.

### Cross-product integration

- `oya-hr-employee-application` emits Workflow events via `oya-workflow-engine-kernel::EventBus` port (no direct workflow crate dependency beyond kernel).
- `oya-hr-employee-adapter::OntologyEmployeeWriter` writes to Ontology via `oya-ontology-entity-kernel::ObjectStore` port.
- No imports from `oya-payroll-*`, `oya-connect-*`, `oya-accounting-*`.

---

## Grit Symbol-Locks

```bash
grit session start ip-p01-hr-full-scaffold

grit claim \
  --agent ip-p01-hr \
  --intent "P01-hr: scaffold Employee/Employment/Organization/Offboarding BCs with full DDL, kernel ports, domain entities, adapters, REST, Cedar, Protobuf events, load tests" \
  --ttl 3600 \
  crates/oya-hr-employee-kernel/src/ports.rs::EmployeeRepository \
  crates/oya-hr-employee-kernel/src/ports.rs::PersonProfileStore \
  crates/oya-hr-employment-kernel/src/types.rs::EmploymentClassification \
  crates/oya-hr-employment-kernel/src/ports.rs::EmploymentRepository \
  crates/oya-hr-organization-kernel/src/ports.rs::OrgRepository \
  crates/oya-hr-offboarding-domain/src/offboarding_request.rs::OffboardingRequest \
  contracts/hr.openapi.yaml::createEmployee \
  contracts/hr.openapi.yaml::updateEmploymentClassification \
  proto/hr/events.proto::EmployeeHired \
  migrations/hr/001_hr_schema.sql::hr.employees \
  policies/hr/hr.cedar::HrAdmin
```

Release: `grit done --agent ip-p01-hr` after all acceptance gates pass.
Fallback: ICM topic `scaffold-locks-oyatie` if grit has no active locks.

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P01-hr-full-scaffold merged; crates scaffolded: oya-hr-employee-{kernel,domain,application,adapter,rest}, oya-hr-employment-{kernel,domain,application,adapter}, oya-hr-organization-{kernel,domain,application,adapter,rest}, oya-hr-offboarding-{domain,application}, oya-hr-app; DDL: hr schema + 8 tables + outbox; EmploymentClassification 8-class enum (ADR-0126); KR corpus citations; Workflow events: EmployeeHired/EmploymentClassChanged/OffboardingInitiated/EmployeeTerminated; all LEAN lanes green; next: IP-P02-payroll" \
  -i high \
  -k "M03,P01,IP-P01-hr-full-scaffold,hr,EmploymentClassification,ADR-0126"
```

---

## Halt Conditions

1. `cargo check` fails after 3 attempts with the same LEAN-A2 violation — indicates design boundary error; escalate to architect.
2. `EmploymentClassification` enum cannot be mapped to a valid `sqlx::Type` in Postgres — escalate; do not use stringly-typed workaround.
3. Grit claim conflicts with another agent on `crates/oya-hr-employment-kernel/src/types.rs::EmploymentClassification` — escalate; this is the canonical ADR-0126 type.
4. `oya gate validate ontology-type-registry` fails after DDL + adapter changes — indicates Ontology Object Type registration protocol changed; escalate.
5. `test_bulk_import_10k` consistently exceeds 60s after optimization attempts — escalate; streaming backpressure design may need architectural review.

---

## Next IP Pointer

`phases/P02-payroll/impl-plan.md` — Payroll µservice scaffold (blocked on this IP completing).

---

## Cross-References

- Phase spec: `phase-spec.md`
- PRD: `docs/prds/hr.md`
- Bominal ADR-0125 (naming canon), ADR-0126 (8-class enum), ADR-0132 (pillars), ADR-0018 (RLS), ADR-0028 (audit chain)
- ADR-0056 (BNF v4.1), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
