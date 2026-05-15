---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P16-records
impl_plan_id: IP-001-records-kernel-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P02-ontology/IP-001
  reason: OntologyFhirAdapter calls ObjectStore + LinkStore ports from oya-ontology-entity-kernel
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
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-001-records-kernel-scaffold: Scaffold Records FHIR + Released-View Kernel/Domain/Application/Adapter/REST/App — FHIR R5 Ports + Ontology Bridge + DDL

## Intent

Scaffolds all 9 records crates across 2 BCs (records-fhir, records-released-view), declares
the 9 FHIR R5 resource types supported in M02, implements `FhirResourceStore` via an Ontology
bridge adapter (FHIR resources stored as Ontology objects with PHI DataClass), and implements
the released-view projection. After this IP merges, M04 Healthcare executors can scaffold
`oya-medical-*` crates with FhirResourceStore as the persistence port — no re-derivation needed.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 9 records workspace members |
| `crates/oya-records-fhir-kernel/Cargo.toml` | create | Zero framework deps; serde + uuid + chrono |
| `crates/oya-records-fhir-kernel/src/lib.rs` | create | pub mod types; pub mod ports; pub mod errors |
| `crates/oya-records-fhir-kernel/src/types.rs` | create | FhirResourceType (9 variants), FhirResource, FhirId, FhirBundle, FhirSearchParams, PersonId |
| `crates/oya-records-fhir-kernel/src/ports.rs` | create | FhirResourceStore + FhirSearchPort — sealed |
| `crates/oya-records-fhir-kernel/src/errors.rs` | create | RecordsError enum |
| `crates/oya-records-fhir-domain/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-records-fhir-domain/src/lib.rs` | create | FhirValidator: validate_resource(); canonical_json(); fhir_r5_profile_check() |
| `crates/oya-records-fhir-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-records-fhir-application/src/lib.rs` | create | GetFhirResourceUseCase, PutFhirResourceUseCase, SearchFhirResourceUseCase |
| `crates/oya-records-fhir-adapter/Cargo.toml` | create | Depends on application + domain + kernel + oya-ontology-entity-kernel + oya-data-boundary-engine-kernel |
| `crates/oya-records-fhir-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-records-fhir-adapter/src/ontology_fhir_adapter.rs` | create | OntologyFhirAdapter: impl FhirResourceStore; writes via ObjectStore; declares PHI DataClass via DubEvaluator |
| `crates/oya-records-fhir-rest/Cargo.toml` | create | axum; depends on application + kernel |
| `crates/oya-records-fhir-rest/src/lib.rs` | create | FHIR REST handlers: GET /fhir/R5/{type}/{id}, PUT /fhir/R5/{type}/{id}, GET /fhir/R5/{type}?{params} |
| `crates/oya-records-fhir-app/Cargo.toml` | create | Composition root |
| `crates/oya-records-fhir-app/src/main.rs` | create | DI assembly |
| `crates/oya-records-released-view-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-records-released-view-kernel/src/lib.rs` | create | ReleasedViewPort + ReleaseDecisionStore ports; ReleasePolicy + ReleaseDecision types |
| `crates/oya-records-released-view-application/Cargo.toml` | create | Depends on released-view-kernel + fhir-kernel |
| `crates/oya-records-released-view-application/src/lib.rs` | create | GetReleasedViewUseCase; PatientBundleUseCase |
| `crates/oya-records-released-view-adapter/Cargo.toml` | create | Depends on released-view-application + kernel + sqlx |
| `crates/oya-records-released-view-adapter/src/lib.rs` | create | PgReleaseDecisionAdapter: impl ReleaseDecisionStore |
| `contracts/records.openapi.yaml` | create | FHIR R5 REST API: GET/PUT/SEARCH resources; released-view endpoints |
| `migrations/records/V001__records_schema.sql` | create | records.release_decisions table (see Code Shape) |
| `docs/standards/bounded-contexts.md` | update | Register records-fhir + records-released-view BCs |

---

## Crate Naming

```
NAME: oya-records-fhir-kernel
JUSTIFICATION:
- microservice = records: shared clinical data plane; Bominal ADR-0016; ADR-0056 v4.1
- bc-tokens = fhir: FHIR R5 resource types BC; separate from released-view BC
- layer = kernel: FhirResourceStore sealed port; 9 FhirResourceType variants; PHI class
- exemptions claimed: none

NAME: oya-records-released-view-kernel
JUSTIFICATION:
- microservice = records, bc-tokens = released-view: patient-safe projection BC
- layer = kernel: ReleasedViewPort + ReleaseDecisionStore sealed ports
- exemptions claimed: none
```

---

## Code Shape

### `crates/oya-records-fhir-kernel/src/types.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type FhirId = Uuid;
pub type PersonId = Uuid;  // identity.Person reference

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FhirResourceType {
    Encounter,
    Observation,
    Medication,
    MedicationRequest,
    Condition,
    Procedure,
    DiagnosticReport,
    AllergyIntolerance,
    Immunization,
}

impl FhirResourceType {
    pub fn ontology_type(&self) -> String {
        format!("records.{:?}", self)
    }
    /// All records resources are PHI per ADR-0008
    pub fn data_class(&self) -> &'static str { "Phi" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirResource {
    pub id: FhirId,
    pub resource_type: FhirResourceType,
    pub tenant_id: TenantId,
    pub subject_person_id: PersonId,   // always linked to a Person (person-pillar)
    pub payload: serde_json::Value,    // FHIR R5 JSON per spec
    pub version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FhirSearchParams {
    pub subject: Option<PersonId>,
    pub date_from: Option<chrono::NaiveDate>,
    pub date_to: Option<chrono::NaiveDate>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirBundle {
    pub resource_type: String,  // "Bundle"
    pub bundle_type: String,    // "searchset"
    pub total: u32,
    pub entries: Vec<FhirBundleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirBundleEntry {
    pub full_url: String,
    pub resource: FhirResource,
}
```

### `migrations/records/V001__records_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS records;

-- Release decisions: provider marks which FHIR resources are released to patient
CREATE TABLE records.release_decisions (
    decision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    resource_type text NOT NULL,
    resource_id uuid NOT NULL,      -- FK to ontology.objects(object_id) at app layer
    subject_person_id uuid NOT NULL,
    released bool NOT NULL DEFAULT false,
    released_by uuid NULL,          -- provider user_id
    released_at timestamptz NULL,
    revoked_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE records.release_decisions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON records.release_decisions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_release_decisions_subject ON records.release_decisions
    (tenant_id, subject_person_id, resource_type) WHERE released = true AND revoked_at IS NULL;
COMMENT ON TABLE records.release_decisions IS 'distribution_column:tenant_id';

-- FHIR resource metadata index (lightweight; actual payload in ontology.objects)
CREATE TABLE records.fhir_index (
    index_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    resource_type text NOT NULL,
    resource_id uuid NOT NULL UNIQUE,  -- same as ontology.objects.object_id
    subject_person_id uuid NOT NULL,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE records.fhir_index FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON records.fhir_index
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_fhir_index_subject ON records.fhir_index
    (tenant_id, subject_person_id, resource_type);
COMMENT ON TABLE records.fhir_index IS 'distribution_column:tenant_id';
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                                          # exit 0
cargo build --workspace --all-features                                          # exit 0
cargo clippy --workspace --all-features -- -D warnings                          # exit 0
cargo nextest run --workspace --all-features                                    # exit 0
cargo nextest run -p oya-records-fhir-adapter --test phi_class_declaration      # exit 0
cargo nextest run -p oya-records-released-view-adapter --test released_view_boundary  # exit 0
cargo nextest run -p oya-records-fhir-adapter --test isolation_records          # exit 0
cargo deny check                                                                # exit 0
oya gate validate lean-a1 --phase P16-records
oya gate validate lean-a2 --phase P16-records
oya gate validate shardability --phase P16-records
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_fhir_resource_type_ontology_type` | FhirResourceType::ontology_type() returns "records.Encounter" etc. |
| `test_fhir_resource_type_data_class` | All 9 types return "Phi" |
| `test_fhir_bundle_construction` | FhirBundle wraps entries; total matches entries.len() |
| `test_release_decision_revoke` | revoked_at set → released = false in query |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_ontology_fhir_put_get` | OntologyFhirAdapter.put() writes Ontology object; .get() returns same |
| `integration_phi_class_declaration` | Every put() calls DubEvaluator.evaluate() with DataClass::Phi |
| `integration_released_view_boundary` | Unreleased resource not returned by ReleasedViewPort.get_released() |
| `integration_isolation_records` | Tenant A cannot read tenant B FHIR resources |
| `integration_fhir_search_by_subject` | search() with subject PersonId returns only that person's resources |

---

## Load Test

| Scenario | Target | Pass criterion |
|---|---|---|
| Get FHIR resource | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |
| Put FHIR resource | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |
| Search (10 resources) | p99 ≤100ms at 1k RPS | `http_req_duration{p(99)}<100` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-architecture \
  --intent "IP-001-records-kernel-scaffold: FHIR R5 kernel + Ontology bridge" \
  --ttl 3600 \
  crates/oya-records-fhir-kernel/src/lib.rs::FhirResourceStore \
  crates/oya-records-fhir-kernel/src/lib.rs::FhirResourceType \
  crates/oya-records-released-view-kernel/src/lib.rs::ReleasedViewPort \
  migrations/records/V001__records_schema.sql::records.release_decisions
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-records-kernel-scaffold merged; FHIR R5 kernel live; 9 resource types; Ontology bridge for PHI storage; released-view boundary; M04 Healthcare can scaffold; next: IP-002-records-ontology-adapter" \
  -i high \
  -k "M02,P16,IP-001,records"
```

---

## Halt Conditions

1. `OntologyFhirAdapter` bypasses DUB evaluation on PHI write — escalate; regulatory requirement.
2. Released-view leaks provider-authoritative fields to patient surface — escalate.
3. LEAN-A2: records adapter importing a product crate (medical, pharmacy) — escalate.

---

## Next IP Pointer

`IP-002-records-ontology-adapter.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0016 (clinical canonical), ADR-0106 (Ontology), ADR-0008 (DUB/PHI), ADR-0056 (BNF v4.1)
