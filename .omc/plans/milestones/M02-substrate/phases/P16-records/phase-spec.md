---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P16-records
status: Proposed
acceptance_lanes: []
entry_gate: 'M02/P02-ontology complete; oya-ontology-kernel ships with ObjectStore
  + ActionStore

  port traits; cargo check clean; grit done on all P02 symbols; ICM phase-handoff
  emitted.

  '
exit_gate: 'All P16 impl-plan acceptance gates green; FHIR R5 canonical resource types
  declared

  in kernel (Encounter, Observation, Medication, Condition, Procedure, DiagnosticReport,

  AllergyIntolerance, Immunization); FhirResourceStore + ReleasedViewPort ports sealed;

  2 BCs registered (records-fhir, records-released-view); all crates pass cargo check/

  build/clippy/nextest/deny; oya gate validate lean-a1/a2/a3/a4 exit 0; grit done
  on

  all P16 symbols; ICM phase-complete row emitted.

  '
depends_on:
- milestone: M02
  phase: P02-ontology
  reason: Records substrate stores FHIR R5 resources as Ontology objects (object_type
    = 'records.Encounter' etc.); FhirResourceStore writes through ObjectStore port;
    linkage between clinical resources uses LinkStore.
owner_team: council-architecture
purpose: Auto-backfilled purpose for phase-spec.md
---
# P16-records: Records Substrate — FHIR R5 Canonical + Released-View Boundary (Healthcare Kernel + Ports for M04+)

## Purpose

Delivers the records substrate kernel and ports that the Healthcare expansion (M04+) will
build on. Per Bominal ADR-0016, `records` is the shared clinical data plane — it owns the
common clinical schema and PHI access infrastructure; `oya-medical` (M04+) is the canonical
product authority. This phase ships the kernel and adapter so M04 executors can start
scaffolding immediately without re-deriving FHIR R5 fundamentals.

FHIR R5 is the canonical clinical record format per Bominal ADR-0016. Records are stored as
typed Ontology objects (`object_type = 'records.Encounter'` etc.) so they inherit full
Ontology provenance, RLS, audit-chain, and DUB enforcement without duplicating those
mechanisms. The released-view boundary governs which FHIR resources are exposed to the
patient surface vs the provider-authoritative record.

Advances Master Plan principles: Healthcare expansion readiness (M04+ ships immediately
with kernel available); Ontology as information layer (FHIR resources = Ontology objects,
no separate storage schema); DUB enforcement (PHI class declared on all records objects).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `records` | `fhir` | `crates/oya-records-fhir-kernel/` | `oya-records-fhir-kernel` |
| `records` | `fhir` | `crates/oya-records-fhir-domain/` | `oya-records-fhir-domain` |
| `records` | `fhir` | `crates/oya-records-fhir-application/` | `oya-records-fhir-application` |
| `records` | `fhir` | `crates/oya-records-fhir-adapter/` | `oya-records-fhir-adapter` |
| `records` | `fhir` | `crates/oya-records-fhir-rest/` | `oya-records-fhir-rest` |
| `records` | `fhir` | `crates/oya-records-fhir-app/` | `oya-records-fhir-app` |
| `records` | `released-view` | `crates/oya-records-released-view-kernel/` | `oya-records-released-view-kernel` |
| `records` | `released-view` | `crates/oya-records-released-view-application/` | `oya-records-released-view-application` |
| `records` | `released-view` | `crates/oya-records-released-view-adapter/` | `oya-records-released-view-adapter` |
| `records` | all | `contracts/records.openapi.yaml` | — |
| `records` | all | `migrations/records/V001__records_schema.sql` | — |

Naming justification:

```
NAME: oya-records-fhir-kernel
JUSTIFICATION:
- microservice = records: shared clinical data plane µservice; Bominal ADR-0016;
  ADR-0056 v4.1
- bc-tokens = fhir: FHIR R5 canonical resource BC; separate from released-view BC;
  includes Encounter/Observation/Medication/Condition/Procedure resource types
- layer = kernel: FhirResourceStore port; FhirResource + ResourceType + FhirId types;
  PHI DataClass declaration; ZERO I/O
- exemptions claimed: none

NAME: oya-records-released-view-kernel
JUSTIFICATION:
- microservice = records, bc-tokens = released-view: the patient-safe projection BC;
  distinct from the provider-authoritative FHIR BC per ADR-0016 §"released view"
- layer = kernel: ReleasedViewPort + ReleasePolicy types
- exemptions claimed: none
```

### Out-of-scope

- Clinical product logic (encounter management, order workflows) — owned by oya-medical (M04+)
- Pharmacy fill state, emergency intake — owned by oya-pharmacy / oya-emergency (M04+)
- FHIR subscription webhooks — deferred to M04+
- SMART on FHIR OAuth scopes — deferred to M04+

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-records-kernel-scaffold.md`](IP-001-records-kernel-scaffold.md) | Scaffold all 9 records crates; FHIR R5 resource types; FhirResourceStore + ReleasedViewPort ports; DDL | pending | `council-architecture` |
| [`IP-002-records-ontology-adapter.md`](IP-002-records-ontology-adapter.md) | FhirResourceAdapter: writes FHIR resources as Ontology objects; declares PHI DataClass | pending | `council-architecture` |
| [`IP-003-records-released-view.md`](IP-003-records-released-view.md) | Released-view projection logic; provider release policy; patient-safe FHIR Bundle generation | pending | `council-architecture` |
| [`IP-004-records-load-tests.md`](IP-004-records-load-tests.md) | k6 load tests; FHIR read p99 ≤50ms; FHIR write p99 ≤200ms | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P16-records
oya gate validate lean-a2 --phase P16-records
oya gate validate lean-a3 --phase P16-records
oya gate validate lean-a4 --phase P16-records
```

### Records-specific gates

```bash
# PHI DataClass declared on all FHIR resource writes
cargo nextest run -p oya-records-fhir-adapter --test phi_class_declaration  # exit 0
# Released-view does not leak provider-authoritative fields
cargo nextest run -p oya-records-released-view-adapter --test released_view_boundary  # exit 0
# Cross-tenant isolation on records
cargo nextest run -p oya-records-fhir-adapter --test isolation_records  # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-records-fhir-kernel` | `kernel` | Yes — FhirResourceStore, FhirSearchPort | N/A |
| `oya-records-fhir-domain` | `domain` | N/A | N/A |
| `oya-records-fhir-application` | `application` | N/A | N/A |
| `oya-records-fhir-adapter` | `adapter` | N/A | Yes — OntologyFhirAdapter (writes via ObjectStore) |
| `oya-records-fhir-rest` | `rest` | N/A | No direct adapter import |
| `oya-records-fhir-app` | `app` | N/A | Unrestricted inward |
| `oya-records-released-view-kernel` | `kernel` | Yes — ReleasedViewPort, ReleaseDecisionStore | N/A |
| `oya-records-released-view-application` | `application` | N/A | N/A |
| `oya-records-released-view-adapter` | `adapter` | N/A | Yes — PgReleaseDecisionAdapter |

### Port traits declared in kernel

```rust
// oya-records-fhir-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// FHIR R5 resource types supported in M02 kernel
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// All FHIR resource types in this kernel are PHI
    pub fn data_class(&self) -> &'static str { "Phi" }
    /// Ontology object_type string
    pub fn ontology_type(&self) -> String {
        format!("records.{:?}", self)
    }
}

#[async_trait::async_trait]
pub trait FhirResourceStore: Send + Sync + sealed::Sealed {
    async fn get(&self, tenant_id: TenantId, resource_type: FhirResourceType, id: FhirId) -> Result<Option<FhirResource>, RecordsError>;
    async fn put(&self, tenant_id: TenantId, resource: FhirResource) -> Result<FhirId, RecordsError>;
    async fn search(&self, tenant_id: TenantId, resource_type: FhirResourceType, params: FhirSearchParams) -> Result<FhirBundle, RecordsError>;
    async fn delete(&self, tenant_id: TenantId, resource_type: FhirResourceType, id: FhirId) -> Result<(), RecordsError>;
}

// oya-records-released-view-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait ReleasedViewPort: Send + Sync + sealed::Sealed {
    /// Returns the patient-safe projection of a FHIR resource; None if not released
    async fn get_released(&self, tenant_id: TenantId, resource_type: FhirResourceType, id: FhirId) -> Result<Option<FhirResource>, RecordsError>;
    /// Returns a patient-safe FHIR Bundle for a given patient (person_id)
    async fn patient_bundle(&self, tenant_id: TenantId, person_id: PersonId, resource_types: &[FhirResourceType]) -> Result<FhirBundle, RecordsError>;
}
```

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P16-records` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P16-records` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P16-records` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P16-records` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P16-records` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `records-fhir` | `records` | pending |
| `records-released-view` | `records` | pending |

---

## Grit Claim Symbols

```
crates/oya-records-fhir-kernel/src/lib.rs::FhirResourceStore
crates/oya-records-fhir-kernel/src/lib.rs::FhirResourceType
crates/oya-records-released-view-kernel/src/lib.rs::ReleasedViewPort
crates/oya-records-fhir-adapter/src/lib.rs::OntologyFhirAdapter
contracts/records.openapi.yaml::getFhirResource
migrations/records/V001__records_schema.sql::records.release_decisions
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P16-records started; FHIR R5 kernel + ports; Healthcare expansion readiness; depends P02-ontology" \
  -i high \
  -k "M02,P16,phase-start,records"

icm store \
  -t context-oyatie \
  -c "Phase P16-records complete; FhirResourceStore + ReleasedViewPort ports live; PHI DataClass declared; M04 Healthcare can scaffold immediately; next: P17-capability-registry" \
  -i high \
  -k "M02,P16,phase-complete,records"
```

---

## References

- Bominal ADRs inherited: ADR-0016 (clinical canonical record), ADR-0106 (Ontology), ADR-0028 (audit-chain)
- oyatie ADRs cited: ADR-0008 (DUB/PHI), ADR-0056 v4.1
- M02-substrate-schema-foundation §6-N (records outlined)
