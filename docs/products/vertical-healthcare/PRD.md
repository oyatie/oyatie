# Oyatie — Product PRD: Vertical Healthcare

> **Status:** preview
> **Owning team:** [`teams/vertical-healthcare/CHARTER.md`](../../teams/vertical-healthcare/CHARTER.md)
> **Owning axis:** vertical-healthcare (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-healthcare-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Healthcare is a FHIR R4-native, HL7 v2-interoperable clinical operations platform for ambulatory, hospital, and multi-site health systems worldwide. It owns the canonical entity model for Patient, Encounter, Observation, Condition, MedicationRequest, and DiagnosticReport — aligned to FHIR R4 resources — with per-region extensions supplied by regional packs (KR: KrPatientId, KrInsurancePayer-NHIS; US: NPI, MBI, SNOMED-CT bindings; EU: EHR-eHealth-DSI). DICOM imaging references and NCPDP pharmacy data flows are declared as first-class seams. The product exists within Oyatie's ecosystem — and only within it — because the combination of a single tenancy boundary enforcing PHI isolation, a Foundry agent runtime operating clinical workflows under the autonomy ceiling with mandatory evidence emission, and a privacy program that structurally blocks PHI from the search and ads axes (not by policy text but by data-class annotation enforced at compile time and runtime) is the regulatory moat that no vertical-only EHR can replicate.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Clinic Administrator | Patient scheduling, encounter management, billing workflow, regulatory reporting dashboard | Per-seat subscription (clinical tier) |
| Clinician (physician, NP, PA) | Clinical documentation, SOAP note authoring (Foundry-assisted), order entry, medication reconciliation, problem list | Per-clinician seat |
| Nurse / Medical Assistant | Vital sign capture, medication administration, patient queue management | Per-seat (nursing tier) |
| Health IT / Tenant Builder | FHIR R4 API access, HL7 v2 interface engine config, DICOM WADO-RS access, capability workflow authoring | Builder seat |
| Pharmacist | NCPDP ePrescribing queue, formulary check, medication fill history | Per-seat (pharmacy tier) |
| Medical Coder / Biller | ICD-10 / CPT / DRG coding assist (Foundry), claims queue, remittance reconciliation | Per-seat (billing tier) |
| Public Health Regulator / Auditor (MFDS, FDA, EMA) | Evidence portal, clinical audit trail, consent audit trail, break-glass log | Cost of doing business |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Patient demographics, encounter creation, SOAP note authoring (Foundry-assisted), vital signs capture, HL7 v2 ADT ingestion, FHIR R4 Patient/Encounter/Observation resources, basic medication list, KR NHIS insurance claims submission | FHIR R4 REST API, HL7 v2 MLLP listener, Web UI (ambulatory) |
| Vertical-Stable | Full FHIR R4 resource set (Condition, MedicationRequest, DiagnosticReport, AllergyIntolerance, Immunization, Procedure), DICOM WADO-RS viewer integration, NCPDP ePrescribing (US Surescripts), ICD-10 / CPT / KR-EDI coding assist via Foundry, claims/remittance (KR-EDI 보건의료, US X12 837P/835), lab result ingestion (LIS bridge), KR 의무기록 compliance, consent management FHIR Consent resource | FHIR R4 API stable, HL7 v2 interface engine, DICOM WADO-RS, Webhook console |
| Public-GA | Longitudinal patient timeline, care gap analysis (Foundry analytics agent), population health dashboard, FHIR Bulk Data export ($export), referral management, prior authorization (FHIR DA Vinci CRD/DTR), global clinical terminology server (SNOMED-CT, LOINC, RxNorm, KR 진료행위코드, JP 傷病名) | Public FHIR R4 API (SMART on FHIR), bulk export, analytics API |
| Region-Fan-Out | Per-regional-pack clinical coding, insurance payer adapters, national patient ID schemes | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- Clinical decision-making AI that replaces clinician judgment — Foundry provides *assistance* under autonomy ceiling; the clinician is always the decision authority
- Insurance underwriting or actuarial modeling
- Hospital bed management / surgical scheduling at ICU depth (deferred to Stable+ roadmap)
- Consumer health / wearable data ingestion at device-protocol depth (FHIR DeviceObservation is in-scope; BLE/ANT+ device pairing is not)
- Pharmaceutical R&D / clinical trial management (separate vertical evaluation pending)
- Advertising targeting using any PHI or FHIR-touching record — **always and permanently blocked** (PRIVACY-PROGRAM §2.2.3 healthcare override)
- Cross-tenant sharing of de-identified data without explicit council approval and DPIA sign-off

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Healthcare. Flat-crates target prefix: `crates/oya-vertical-healthcare-*`.

The healthcare vertical owns the clinical entity model (FHIR-aligned), HL7 v2 interface engine, DICOM reference seam, NCPDP pharmacy seam, and clinical terminology server. Cross-axis contracts: `oya-platform-tenant-kernel` (tenancy, `ad_targetable_blocked` forced), `oya-platform-audit-chain-kernel` (break-glass, consent, access audit), `oya-foundry-api` (clinical Foundry capabilities under autonomy ceiling T1 max), `oya-platform-regulatory-kernel` (MFDS/FDA/EMA/PMDA packs), `oya-platform-dsr-kernel` (HIPAA patient access request / KR-의무기록 patient right).

### 4.2 Layered Structure

```
crates/oya-vertical-healthcare-kernel-clinical/     — Patient, Encounter, Observation, Condition, MedicationRequest, DiagnosticReport; FHIR-aligned; no I/O
crates/oya-vertical-healthcare-kernel-terminology/  — CodeSystem, ValueSet, ConceptMap entities; SNOMED/LOINC/RxNorm/KR-EDI bindings
crates/oya-vertical-healthcare-kernel-imaging/      — ImagingStudy, Series, Instance reference entities; DICOM seam trait declarations
crates/oya-vertical-healthcare-kernel-pharmacy/     — MedicationRequest, MedicationDispense, MedicationAdministration; NCPDP seam trait
crates/oya-vertical-healthcare-kernel-claims/       — Claim, ClaimResponse, CoverageEligibility entities; per-region coding seam
crates/oya-vertical-healthcare-domain-clinical/     — Clinical use cases: admit, discharge, document-encounter, order, reconcile-medications
crates/oya-vertical-healthcare-domain-consent/      — Patient consent management; FHIR Consent resource; DSR bridge
crates/oya-vertical-healthcare-domain-claims/       — Claims use cases: create-claim, adjudicate, reconcile-remittance
crates/oya-vertical-healthcare-app-clinical/        — Clinical saga orchestration, Foundry capability delegation (SOAP-note assist, coding assist)
crates/oya-vertical-healthcare-app-claims/          — Claims submission saga (per-payer regional pack)
crates/oya-vertical-healthcare-adapter-db/          — Postgres adapters (PHI schema, per-patient sharding)
crates/oya-vertical-healthcare-adapter-hl7v2/       — HL7 v2 MLLP listener + parser adapter
crates/oya-vertical-healthcare-adapter-dicom/       — DICOM WADO-RS / STOW-RS adapter seam
crates/oya-vertical-healthcare-adapter-ncpdp/       — NCPDP SCRIPT ePrescribing adapter
crates/oya-vertical-healthcare-adapter-payer/       — Per-region payer API adapters (NHIS KR, CMS US, etc.)
crates/oya-vertical-healthcare-api-fhir/            — Inbound FHIR R4 REST server (SMART on FHIR token validation)
crates/oya-vertical-healthcare-api-hl7v2/           — Inbound HL7 v2 MLLP / HTTP gateway
crates/oya-vertical-healthcare-worker-events/       — Kafka consumers (encounter-created, lab-result-received, etc.)
crates/oya-vertical-healthcare-runtime/             — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| FHIR R4 REST API | `contracts/healthcare-fhir-r4.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| SMART on FHIR token endpoint | `contracts/healthcare-smart.openapi.yaml` | Control | 99.9% / p95 < 200ms |
| HL7 v2 MLLP gateway | `contracts/healthcare-hl7v2.yaml` | Data | 99.9% / p95 < 500ms |
| DICOM WADO-RS / STOW-RS | `contracts/healthcare-dicom.yaml` | Data | 99.5% / p95 < 2s (image retrieval) |
| FHIR $export (Bulk Data) | `contracts/healthcare-bulk.yaml` | Data | async; < 1 hour for 100K resources |
| Webhook events (patient-admitted, lab-received) | `contracts/healthcare-webhooks.yaml` | Data | at-least-once, ≤ 60s |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `ClinicalDocumentIndexable` | `SearchIndexable` (tenant-private only) | Search axis (encounter summaries, note titles — never PHI full-text cross-tenant) |
| `PatientConsentSource` | `ConsentBroker` trait | DSR kernel, Audit chain |
| `ClinicalTerminologyResolver` | `TerminologyProvider` trait | Coding assist Foundry capability, Claims domain |
| `PatientIdentitySync` | `IdentitySync` | Platform identity (patient portal access) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel (with `ad_targetable_blocked` forced) | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis + privacy review |
| `Identity / Cedar policy` | SaaS platform | `oya-platform-identity-kernel` | Cross-axis + security |
| `Capability invocation` (autonomy ceiling T1 max for clinical) | Foundry | `oya-foundry-api` | Foundry + healthcare review |
| `Audit-chain event` (break-glass mandatory) | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `RegulatoryPack` seam | Platform regulatory | `oya-platform-regulatory-kernel` | Regulatory + healthcare review |
| `DSR cascade` | Privacy | `oya-platform-dsr-kernel` | Privacy + healthcare review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-healthcare-kernel-clinical
// ALL healthcare PHI fields carry data_class: PHI
// Tenant-class override: ad_targetable_blocked = true (forced, cannot be raised)
// plane: data

/// FHIR R4 Patient resource aligned
pub struct Patient {
    pub id: PatientId,                             // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                       // data_class: INTERNAL_ONLY
    pub region: RegionCode,                        // data_class: INTERNAL_ONLY
    pub schema_version: u32,
    pub fhir_id: FhirId,                          // data_class: INTERNAL_ONLY (FHIR logical id)
    pub mrn: MedicalRecordNumber,                  // data_class: PHI
    pub national_id: Option<NationalPatientId>,    // data_class: PHI (KR: RRN; US: MBI; EU: eIDAS national)
    pub local_patient_id: Option<LocalPatientId>,  // data_class: PHI (regional pack extension point)
    pub name: HumanName,                           // data_class: PHI
    pub birth_date: Option<NaiveDate>,             // data_class: PHI
    pub gender: AdministrativeGender,              // data_class: PHI (FHIR AdministrativeGender enum)
    pub deceased: DeceasedInfo,                    // data_class: PHI
    pub address: Vec<Address>,                     // data_class: PHI
    pub telecom: Vec<ContactPoint>,                // data_class: PHI
    pub marital_status: Option<CodeableConcept>,   // data_class: PHI
    pub language: Vec<Communication>,             // data_class: PHI
    pub insurance: Vec<PatientInsurance>,          // data_class: PHI (payer/member-id/group)
    pub consent_ref: Option<ConsentId>,            // data_class: INTERNAL_ONLY
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct HumanName {
    pub family: String,
    pub given: Vec<String>,
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
    pub period: Option<Period>,
}

pub enum AdministrativeGender { Male, Female, Other, Unknown }
pub struct DeceasedInfo { pub is_deceased: bool, pub deceased_date_time: Option<DateTime<Utc>> }

/// FHIR R4 Encounter resource aligned
/// plane: data
pub struct Encounter {
    pub id: EncounterId,
    pub tenant_id: TenantId,
    pub patient_id: PatientId,                     // data_class: PHI
    pub region: RegionCode,
    pub schema_version: u32,
    pub fhir_id: FhirId,
    pub status: EncounterStatus,                   // data_class: PHI
    pub class: Coding,                             // data_class: PHI (AMB, IMP, EMER, etc.)
    pub type_codes: Vec<CodeableConcept>,          // data_class: PHI (encounter type)
    pub service_type: Option<CodeableConcept>,     // data_class: PHI
    pub priority: Option<CodeableConcept>,         // data_class: PHI
    pub subject_ref: PatientId,                    // data_class: PHI
    pub participant: Vec<EncounterParticipant>,    // data_class: PHI (clinician + role)
    pub period: Option<Period>,                    // data_class: PHI
    pub reason_code: Vec<CodeableConcept>,         // data_class: PHI (ICD-10 / SNOMED reason)
    pub diagnosis: Vec<EncounterDiagnosis>,        // data_class: PHI
    pub location: Vec<EncounterLocation>,          // data_class: PHI
    pub service_provider: Option<OrganizationRef>, // data_class: INTERNAL_ONLY
    pub foundry_run_id: Option<FoundryRunId>,      // data_class: INTERNAL_ONLY (SOAP-note assist)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum EncounterStatus {
    Planned, Arrived, Triaged, InProgress,
    OnLeave, Finished, Cancelled, Unknown,
}

/// FHIR R4 Observation resource aligned
/// plane: data
pub struct Observation {
    pub id: ObservationId,
    pub tenant_id: TenantId,
    pub patient_id: PatientId,                     // data_class: PHI
    pub encounter_id: Option<EncounterId>,         // data_class: PHI
    pub region: RegionCode,
    pub schema_version: u32,
    pub fhir_id: FhirId,
    pub status: ObservationStatus,                 // data_class: PHI
    pub category: Vec<CodeableConcept>,            // data_class: PHI
    pub code: CodeableConcept,                     // data_class: PHI (LOINC / SNOMED / KR-EDI code)
    pub subject_ref: PatientId,                    // data_class: PHI
    pub effective: ObservationEffective,           // data_class: PHI
    pub value: ObservationValue,                   // data_class: PHI (quantity, codeable, string, etc.)
    pub interpretation: Vec<CodeableConcept>,      // data_class: PHI
    pub note: Vec<Annotation>,                     // data_class: PHI
    pub component: Vec<ObservationComponent>,      // data_class: PHI (for multi-component obs like BP)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ObservationStatus { Registered, Preliminary, Final, Amended, Cancelled, Unknown }
pub enum ObservationValue {
    Quantity(Quantity),
    CodeableConcept(CodeableConcept),
    StringValue(String),
    Boolean(bool),
    Range(Range),
    Ratio(Ratio),
    SampledData(SampledData),
}

/// FHIR R4 Condition resource aligned
/// plane: data
pub struct Condition {
    pub id: ConditionId,
    pub tenant_id: TenantId,
    pub patient_id: PatientId,                     // data_class: PHI
    pub region: RegionCode,
    pub schema_version: u32,
    pub fhir_id: FhirId,
    pub clinical_status: CodeableConcept,          // data_class: PHI (active, recurrence, relapse, etc.)
    pub verification_status: CodeableConcept,      // data_class: PHI (confirmed, provisional, etc.)
    pub category: Vec<CodeableConcept>,            // data_class: PHI (problem-list-item, encounter-diagnosis)
    pub severity: Option<CodeableConcept>,         // data_class: PHI
    pub code: CodeableConcept,                     // data_class: PHI (ICD-10 / SNOMED-CT / KCD-8 KR)
    pub body_site: Vec<CodeableConcept>,           // data_class: PHI
    pub subject_ref: PatientId,                    // data_class: PHI
    pub encounter_ref: Option<EncounterId>,        // data_class: PHI
    pub onset: Option<ConditionOnset>,             // data_class: PHI
    pub abatement: Option<ConditionAbatement>,     // data_class: PHI
    pub note: Vec<Annotation>,                     // data_class: PHI
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// FHIR R4 MedicationRequest resource aligned
/// plane: data
pub struct MedicationRequest {
    pub id: MedicationRequestId,
    pub tenant_id: TenantId,
    pub patient_id: PatientId,                     // data_class: PHI
    pub region: RegionCode,
    pub schema_version: u32,
    pub fhir_id: FhirId,
    pub status: MedicationRequestStatus,           // data_class: PHI
    pub intent: MedicationRequestIntent,           // data_class: PHI
    pub medication: MedicationRef,                 // data_class: PHI (CodeableConcept or Reference)
    pub subject_ref: PatientId,                    // data_class: PHI
    pub encounter_ref: Option<EncounterId>,        // data_class: PHI
    pub requester_ref: PractitionerRef,            // data_class: PHI (prescribing clinician)
    pub dosage_instruction: Vec<Dosage>,           // data_class: PHI
    pub dispense_request: Option<DispenseRequest>, // data_class: PHI
    pub substitution: Option<MedSubstitution>,     // data_class: PHI
    pub ncpdp_script_ref: Option<NcpdpRef>,        // data_class: INTERNAL_ONLY (ePrescribing seam)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum MedicationRequestStatus {
    Active, OnHold, Cancelled, Completed, EnteredInError, Stopped, Draft, Unknown
}
pub enum MedicationRequestIntent {
    Proposal, Plan, Order, OriginalOrder, ReflexOrder, FillerOrder, InstanceOrder, Option
}
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `PatientAggregate` | `Patient` | Demographics, insurance, consent; per-patient consistency (all changes to Patient root are within this aggregate) |
| `EncounterAggregate` | `Encounter` + inline `EncounterDiagnosis[]` | All documentation associated with one clinical encounter; Observations, Conditions, MedicationRequests are separate aggregates with encounter-ref |
| `ClinicalDocumentAggregate` | `DiagnosticReport` or `ClinicalNote` | A clinician-authored document (SOAP note, discharge summary); immutable after sign-off |
| `MedicationListAggregate` | `MedicationRequest[]` for one patient | Active medication list reconciliation; consistency: no two active orders for same drug-class without override |
| `ClaimAggregate` | `Claim` + `ClaimItem[]` | One billing claim per encounter or episode; claim lines are part of the aggregate |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Patient | Postgres (PHI schema, per-patient shard) | `patient_id` (shuffle sharding across node pool) | Per-patient row with tenant isolation | Streaming replication × 3 (PHI criticality) | Per-regulatory: KR 10 years (의무기록법), US HIPAA 6 years, EU GDPR 10 years minimum |
| Encounter + ClinicalDocument | Postgres (per-patient shard) | `patient_id` | Same shard as Patient for locality | Streaming replication × 3 | Same as Patient |
| Observation | TimescaleDB (per-tenant hypertable) | `(tenant_id, patient_id, effective_time)` | Hypertable time partition | Streaming replication × 3 | 10+ years (clinical observation retention) |
| MedicationRequest | Postgres (per-patient shard) | `patient_id` | Per-patient | Streaming replication × 3 | 10 years |
| DICOM references | Postgres metadata + Object Store (DICOM blobs) | `(tenant_id, study_id)` | Per-study shard | Object store geo-replication (per residency class) | 7-10 years (KR 의료법 §22) |
| Claims | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 7 years (KR 국민건강보험법) |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `PatientAdmitted` | `healthcare.patient.admitted` | `contracts/events/healthcare-clinical.json` | Encounter aggregate, Audit chain, Notification | 30 days | `encounter_id` |
| `EncounterClosed` | `healthcare.encounter.closed` | `contracts/events/healthcare-clinical.json` | Claims aggregate, Analytics projection, Audit chain | 30 days | `encounter_id` |
| `LabResultReceived` | `healthcare.observation.lab_result` | `contracts/events/healthcare-clinical.json` | Encounter (auto-link), Clinician notification, Audit chain | 30 days | `observation_id` |
| `MedicationPrescribed` | `healthcare.medication.prescribed` | `contracts/events/healthcare-medication.json` | NCPDP ePrescribing seam, Audit chain | 30 days | `medication_request_id` |
| `ClaimSubmitted` | `healthcare.claim.submitted` | `contracts/events/healthcare-claims.json` | Remittance reconciliation, Audit chain | 90 days | `claim_id` |
| `PatientConsentRevoked` | `healthcare.consent.revoked` | `contracts/events/healthcare-consent.json` | DSR cascade, Search index cascade delete, Audit chain | 365 days | `(patient_id, consent_id)` |
| `BreakGlassAccess` | `healthcare.break_glass` | `contracts/events/healthcare-audit.json` | Audit chain (mandatory), Compliance dashboard | 7 years | `(accessor_id, patient_id, timestamp)` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `Encounter.type_codes` description | tenant-private search index (encounter lookup) | `PHI` — tenant-private only; never cross-tenant; never ads | Yes — patient consent revocation triggers cascade |
| `Patient.name` (tokenized, not full-text cross-tenant) | tenant-private patient directory | `PHI` — tenant-private only | Yes |
| `DiagnosticReport.conclusion` (summary only) | tenant-private clinical search | `PHI` — tenant-private only | Yes |
| `ClinicalNote.title` | tenant-private note search | `PHI` — tenant-private only | Yes |

**Structural enforcement:** `PHI` data_class makes every field above structurally impossible to route into the cross-tenant search index or ads axis. The `oya-platform-ads-gate` rejects any PHI-tagged record at the eventing backbone level. No policy-only control.

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| Patient record access (any) | `audit.healthcare.patient_accessed` | `accessor_id`, `patient_id` (pseudonymized), `access_type`, `encounter_id`, `data_classes_touched`, `break_glass: bool` |
| Break-glass access | `audit.healthcare.break_glass` | `accessor_id`, `patient_id`, `reason`, `clinical_justification`, `duration_seconds` |
| Patient consent decision | `audit.healthcare.consent_decision` | `patient_id`, `consent_type`, `decision`, `decision_made_by`, `effective_period` |
| Medication prescribed | `audit.healthcare.medication_prescribed` | `patient_id` (pseudonymized), `prescriber_id`, `drug_class`, `ncpdp_ref` |
| Claim submitted to payer | `audit.healthcare.claim_submitted` | `claim_id`, `payer_id`, `encounter_id`, `regulatory_pack_id`, `amount` |
| DSR cascade deletion | `audit.healthcare.dsr_cascade` | `patient_id`, `dsr_type`, `stores_affected`, `deletion_proof_hash` |
| PHI export | `audit.healthcare.phi_exported` | `patient_id`, `exported_by`, `data_classes`, `export_format`, `dsr_ref`, `destination` |

### 5.7 Schema Migration Policy

- FHIR resource schema changes must maintain backward compatibility (FHIR versioning rules apply).
- HL7 v2 interface upgrades (e.g., v2.5 → v2.8) are additive; transformers are versioned.
- PHI schema migrations require a DPIA review and audit-chain evidence of zero PHI loss on staging.
- Down-migrations for clinical data are prohibited once data has been electronically signed (immutability).

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `patient_id`-based shuffle sharding for PHI isolation; one patient's data never crosses cell boundaries without explicit DR failover |
| Sharding strategy | Per-patient shard for Patient/Encounter/Observation (maximum PHI isolation); per-tenant shard for Claims/Terminology |
| Caching tier | In-memory LRU for ValueSet / terminology lookups (high-read, low-write); Redis for active encounter state; no PHI in Redis without DEK-per-tenant encryption |
| Bulk endpoint contract | `FHIR $export` (Bulk Data export per FHIR R4 spec); `POST /observations/bulk` for LIS result bulk ingest; `POST /claims/bulk` for batch claim submission |
| Pagination | FHIR-standard `Bundle` pagination with `next` link; cursor on `(lastUpdated, id)`; page size max 200 resources; `_since` and `_type` filter params |
| Idempotency | FHIR `If-None-Exist` conditional create for idempotent patient/encounter creation; HL7 v2 `MSH.10` message control ID for dedup; all claims idempotent on `claim_id` |
| Batch dispatch | Foundry `SOAPNoteAssist` and `CodingAssist` capabilities run as batch per encounter queue; NCPDP ePrescribing sent as batch to Surescripts |
| Backpressure | HL7 v2 MLLP consumer implements ACK-NAK flow control; FHIR bulk export uses async job with polling; Kafka consumer lag monitored on `healthcare.observation.lab_result` |
| Hot-path benchmarks | `fhir_patient_read` criterion gate < 50ms; `hl7v2_adt_parse` < 20ms; `observation_insert_batch` < 100ms for 100 observations |
| Agent-driven optimization | Foundry `SOAPNoteAssist` (clinical documentation); Foundry `CodingAssist` (ICD-10/CPT code suggestion); Foundry `CareGapAnalyzer` (population health); all under autonomy ceiling T1 (recommend-only, clinician approves) |
| FinOps unit-economics | Per-active-patient-month metering (primary); per-FHIR-API-call metering for external developers; DICOM storage per-GB-month |
| Build-cache / CI affected-graph | `oya-vertical-healthcare-kernel-clinical` → full rebuild of all healthcare crates; `adapter-hl7v2` → targeted rebuild + HL7 conformance test suite |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Patient national ID validation | `NationalPatientIdValidator` | Yes | `oya-pack-kr` (RRN + NHIS 환자번호), `oya-pack-us` (MBI, NPI), `oya-pack-jp` (マイナンバー for healthcare) |
| Insurance payer adapter | `PayerAdapter` | Yes — per national insurance system | `oya-pack-kr` (NHIS 국민건강보험 + 의원/병원 청구 EDI), `oya-pack-us` (CMS/Medicare + major commercial payers X12 835/837), `oya-pack-jp` (健康保険 レセプト) |
| Clinical coding system binding | `LocalIndustryExtension` (healthcare kernel) | Yes — per country code system | `oya-pack-kr` (KCD-8 상병코드, EDI 진료행위코드), `oya-pack-us` (ICD-10-CM, CPT-4, SNOMED-CT, LOINC, RxNorm), `oya-pack-eu` (ICD-10-CM + national extensions), `oya-pack-jp` (ICD-10 JP edition, 薬価収載 drug codes) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (MFDS 의료기기법, 의무기록법, 의료법), `oya-pack-us` (HIPAA/HITECH, ONC certification, FDA 21 CFR Part 11), `oya-pack-eu` (EMA, EU MDR, GDPR Art 9) |
| ePrescribing adapter | `EprescribingAdapter` | Yes | `oya-pack-us` (NCPDP SCRIPT + Surescripts), `oya-pack-kr` (의약품안전나라 DUR), `oya-pack-eu` (ePrescription FHIR profiles per country) |
| Identity-provider adapter | `IdentityProvider` | Yes (clinician + patient portal) | `oya-pack-kr` (본인확인서비스 for patient portal), `oya-pack-us` (SMART on FHIR / Login.gov) |
| Break-glass audit evidence | `BreakGlassAuditPolicy` | Yes | All onboarded packs (HIPAA minimum-necessary + KR 의료법) |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-healthcare-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # MFDS, 의무기록법, 의료법, 국민건강보험법, PIPA
  - oya-pack-us   # HIPAA/HITECH, ONC, FDA 21 CFR Part 11, CCPA
  - oya-pack-jp   # PMDA, APPI, 健康保険法
  - oya-pack-eu   # EMA, EU MDR, GDPR Art 9, EHR-eHealth-DSI
tenant_class_overrides:
  ad_targetable_blocked: true   # forced; cannot be raised by tenant admin
  search_index_class: PHI       # tenant-private only; cross-tenant blocked
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio`, `axum`, `sqlx`, `serde`, `rustls` | kernel-grade | MIT / Apache-2 | No | Use |
| `fhir-rs` (FHIR R4 types) | maturing | MIT | In-house FHIR types strongly considered; fhir-rs has incomplete R4 coverage | Build in-house `oya-vertical-healthcare-kernel-clinical` with FHIR-aligned structs; do not depend on external FHIR crate for kernel types |
| `hl7-mllp-rs` (HL7 v2 MLLP) | early-stable | MIT | In-house MLLP listener considered | Use for MLLP transport layer; build in-house HL7 v2 parser in `adapter-hl7v2` |
| `dicom-rs` (DICOM parsing) | maturing | MIT | In-house DICOM metadata parser considered | Use `dicom-rs` for DICOM parsing; WADO-RS gateway built in-house in `adapter-dicom` |
| `rust-loinc` / terminology crates | none mature | — | No mature Rust LOINC crate exists | Build in-house `oya-vertical-healthcare-kernel-terminology` backed by NLM LOINC API + embedded value sets |
| TimescaleDB (Postgres extension for Observation time series) | stable | Apache-2 (Timescale OSS) | Pure Postgres partitioning considered | Use TimescaleDB OSS; ADR required (Apache-2 confirmed) |
| SNOMED-CT / LOINC / RxNorm | content licenses | SNOMED International / NLM (free for clinical use) | No code dep; content sets licensed and embedded | License; embed in terminology kernel; KR KCD-8 licensed from HIRA |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Active patient records under management | ≥ 1,000 (design-partner clinic) | ≥ 50,000 | ≥ 500,000 |
| FHIR R4 API conformance score (HL7 Inferno) | ≥ 90% | 100% | 100% |
| HL7 v2 ADT message processing P99 | < 1s | < 500ms | < 200ms |
| FHIR resource round-trip fidelity | 100% (no data loss on create→read) | 100% | 100% |
| Audit-chain emission completeness (PHI access) | 100% | 100% | 100% |
| Break-glass log completeness | 100% | 100% | 100% |
| NHIS claims acceptance rate (KR) | ≥ 95% first-submission | ≥ 99% | ≥ 99.5% |
| Foundry SOAPNoteAssist adoption (% encounters with assist) | ≥ 20% | ≥ 60% | ≥ 80% |
| DSR/patient-access-request fulfillment time | < 30 days (HIPAA) | < 10 days | < 5 days |
| PHI leak to search/ads axis | 0 (hard zero, CI-enforced) | 0 | 0 |
| Cross-axis contract violations | 0 | 0 | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| PHI leak to search or ads axis | Catastrophic | `data_class: PHI` on all patient fields; structurally impossible to route to ads gate or cross-tenant search; compile-time + runtime enforced | Privacy + Architecture |
| Break-glass misuse (unauthorized emergency access) | Critical | Cedar policy enforces T1 autonomy ceiling; every break-glass emits mandatory audit record; automated anomaly detection on frequency | Security + Compliance |
| FHIR resource validation failure (malformed data reaches downstream) | High | FHIR $validate on every create/update; HL7 v2 conformance profile validation at MLLP layer; schema version checked before persistence | Healthcare domain |
| KR NHIS claim rejection rate spike | High | Dual-path claim validation (in-house + KR-pack EDI validator before submission); Foundry CodingAssist trained on HIRA reject reasons | KR pack + Claims domain |
| Patient identity mismatch (wrong patient chart) | Catastrophic | MPI (Master Patient Index) with probabilistic match requiring clinician confirmation; no auto-merge above match threshold 0.9; audit trail for every merge | Patient domain |
| DICOM storage cost explosion (imaging at scale) | Medium | Tiered storage: hot (active studies) → warm (30 days) → cold archive; DICOM series lifecycle policy per tenant; FinOps alert on per-tenant storage growth | Infrastructure + FinOps |
| Foundry clinical agent hallucination (wrong SOAP note content) | High | Autonomy ceiling T1 = recommend-only; clinician must review + approve all Foundry-generated content before persistence; not auto-saved | Foundry + Healthcare domain |
| Regulatory change: KR 의무기록법 amendment (retention period change) | Medium | Regulatory-change watch lane; KR pack versioned; affected patient record TTLs updated in controlled migration | KR pack + Compliance |
| NCPDP / Surescripts API deprecation | Medium | ePrescribing via `EprescribingAdapter` trait; Surescripts is one impl; alternative impl registered in pack without kernel change | US pack + Pharmacy domain |
| mTLS misconfiguration exposing PHI over network | Catastrophic | `rustls` enforced at all service-to-service boundaries; no plaintext PHI over wire; mutual TLS required for MLLP gateway | Security |

---

## 11. Open Questions

- Council decision: DICOM WADO-RS hosted in-house (object store + gateway) vs. partnership with specialized PACS vendor per region?
- FHIR Subscription (R4B WebSocket push) — include in Vertical-Preview or defer to Stable?
- Population health analytics cohort building: PHI aggregation approach — Foundry + DP wrapper, or dedicated analytics aggregate with k-anonymization?
- KR 건강보험 청구 EDI — direct NHIS API or via clearinghouse partnership?
- EU EHR-eHealth-DSI cross-border patient summary — scope for Vertical-Stable or Region-Fan-Out?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| FHIR R4 as canonical entity model (not HL7 v2) | 2026-05-09 | FHIR is the global standard for new clinical data exchange; HL7 v2 supported for legacy interoperability only | — |
| All PHI fields carry `data_class: PHI` at kernel level | 2026-05-09 | Structural enforcement from the type system; no runtime-only policy | PRIVACY-PROGRAM §2.2.1 |
| `ad_targetable_blocked: true` forced for all healthcare tenants | 2026-05-09 | HIPAA + MFDS + 의료법 mandate; PRIVACY-PROGRAM §2.2.3 | PRIVACY-PROGRAM §2.2.3 |
| Per-patient shard for PHI isolation (not per-tenant) | 2026-05-09 | Maximum blast-radius control for PHI; regulatory requirement for individual data isolation | DESIGN.md §9 |
| Foundry autonomy ceiling T1 max for clinical capabilities | 2026-05-09 | Clinician is always the decision authority; agents recommend, humans approve | ADR-0050; ADR-0022 |
| Flat-crates: `crates/oya-vertical-healthcare-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md` — north star, wave sequencing
- `docs/DESIGN.md` §1, §4, §9, §10, §12 — bounded context, sharding, regional pack architecture
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3 — PHI class, healthcare tenant-class override
- `docs/GLOSSARY.md` §5, §7 — data vocabulary, regulatory terms
- ADR-0015, ADR-0003, ADR-0050, ADR-0017

---

## Doc-Catalog Row

```
| `vertical-healthcare` | `vertical-2` | FHIR R4 / HL7 v2 / DICOM / NCPDP clinical platform; PHI-isolated | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
