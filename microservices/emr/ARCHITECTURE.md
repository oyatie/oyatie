---
doc_class: Architecture
microservice: emr
title: EMR Architecture — 12-layer mapping + cross-µservice handoffs
date: 2026-05-21
status: wave-15m-b-authored-2026-05-21
owner_team: axis-emr + council-architecture
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0244
  - ADR-0251
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0332
binding_doctrines:
  - hexagonal-clean-architecture (12-layer canonical enum per ADR-0105)
  - port-in-kernel
  - inward-only-dependency-flow
  - workflow-plus-ontology-adapter-layer (RETIRED per ADR-0145 — see §3)
  - direct-grpc-inter-microservice (ADR-0145 §D-3)
  - cedar-as-universal-gate (ADR-0243)
  - tenant-as-universal-scoping-primitive (ADR-0244)
  - amazon-shape-cellular-architecture (ADR-0248)
---

# EMR Architecture

## 1. Scope

This document specifies the runtime architecture of the EMR microservice — code organization, layer mapping, inter-µservice handoffs, durable state, and data-flow envelopes. The PRD (`microservices/emr/PRD.md`) specifies WHY and WHAT; this document specifies HOW.

EMR is a B2B-healthcare-provider tenant-class product µservice (per ADR-0132 + ADR-0244 §D-7) covering 15 bounded contexts (BCs). Each BC is a layered hexagonal stack rooted at a kernel crate, with domain, use-case, application-composition, and adapter trio (REST + AsyncAPI + gRPC) above it per ADR-0105's 13-layer enum.

## 2. The 12-layer enum mapped onto EMR

ADR-0105 enumerates 13 canonical layers; EMR materializes them per BC as follows. Layer ordering proceeds from innermost ("clean architecture core") outward to "the world".

### Layer 0 — `kernel` (per BC)

The kernel is a `no_std`-eligible crate containing PORTS (trait definitions), VALUE TYPES (newtypes wrapping primitives), and INVARIANT VALIDATION (e.g., a `Dob` newtype refuses dates in the future). Crate convention: `oya-emr-<bc>-kernel`. Examples:

```
oya-emr-patient-kernel
oya-emr-encounter-kernel
oya-emr-problem-kernel
oya-emr-medication-kernel
oya-emr-allergy-kernel
oya-emr-vital-kernel
oya-emr-note-kernel
oya-emr-order-kernel
oya-emr-result-kernel
oya-emr-care-team-kernel
oya-emr-order-set-kernel
oya-emr-documentation-kernel
oya-emr-billing-code-kernel
oya-emr-patient-education-kernel
oya-emr-portal-session-kernel
```

The kernel exposes ports such as `PatientRepository`, `EncounterRepository`, `OrderDispatcher`, `AuditEmitter`, `CedarPolicyEvaluator`, `ClinicalDecisionSupportClient`, etc. NO IO. NO ASYNC.

### Layer 1 — `domain`

Per-BC domain logic. Aggregates + entities + value-objects + domain services. Pure functions over kernel types; no IO; no async. Implements business invariants (e.g., "a signed note may not be edited; only amended").

```
oya-emr-patient-domain
oya-emr-encounter-domain
oya-emr-problem-domain
oya-emr-medication-domain
... (one per BC)
```

### Layer 2 — `usecase`

Orchestration of domain operations into use cases. Each use case is a `(Context, Input) -> Result<Output, UseCaseError>` shape. Use cases inject ports as trait-bound generics; they call domain operations, dispatch port calls, and assemble outputs.

```
oya-emr-patient-usecase
oya-emr-encounter-usecase
oya-emr-problem-usecase
... (one per BC)
```

Example use cases:

- `CreatePatient` (patient BC)
- `MergePatient` (patient BC)
- `StartEncounter` (encounter BC)
- `DischargeEncounter` (encounter BC)
- `PrescribeMedication` (medication BC)
- `EnterOrderSet` (order BC)
- `SignNote` (note BC)
- `RecordVital` (vital BC)
- `ReviewResult` (result BC)
- `GrantProxyAccess` (portal-session BC)

### Layer 3 — `application`

Per-BC composition root. Wires concrete adapter implementations into the use-case generics. Reads configuration. Constructs the use-case object graph. NO business logic.

```
oya-emr-patient-application
oya-emr-encounter-application
... (one per BC)
```

### Layer 4 — `api` (REST/HTTP shape per BC)

Per-BC REST handler crate. Translates HTTP requests to use-case inputs; renders use-case outputs back to JSON (FHIR R5 by default; FHIR R4 on Accept-Version). Owns OpenAPI 3.2.0 emission.

```
oya-emr-patient-api
oya-emr-encounter-api
... (one per BC)
```

### Layer 5 — `rest` (collation of REST surfaces; HTTP server crate)

Top-level REST server crate that mounts every BC's api crate under unified routing. Single `oya-emr-rest` crate exists; not per-BC. Owns auth middleware (tenant resolution, principal resolution, Cedar evaluation, trace-context propagation).

### Layer 6 — `events` (AsyncAPI shape per BC)

Per-BC AsyncAPI publisher + consumer crate. Serializes domain events to canonical Kafka topics; deserializes inbound events from peer µservices.

```
oya-emr-patient-events
oya-emr-encounter-events
... (one per BC)
```

### Layer 7 — `grpc` (proto3 shape per BC)

Per-BC gRPC handler crate. Implements proto3 services for inter-µservice synchronous calls. Translates protobuf messages to use-case inputs; renders use-case outputs back to protobuf.

```
oya-emr-patient-grpc
oya-emr-encounter-grpc
... (one per BC)
```

### Layer 8 — `adapter-postgres` (per BC; Citus-shard adapter for relational persistence)

Per-BC Postgres adapter. Implements the `<BC>Repository` ports from kernel against Citus-sharded Postgres. Schema management via `sqlx::migrate!`. Tenant-shard-key on every table.

```
oya-emr-patient-adapter-postgres
oya-emr-encounter-adapter-postgres
... (one per BC)
```

### Layer 8b — `adapter-timescale` (vital BC only)

Vital signs hypertable on TimescaleDB. Implements `VitalRepository` against time-series storage.

```
oya-emr-vital-adapter-timescale
```

### Layer 8c — `adapter-valkey` (portal-session BC only)

Portal session adapter on Valkey. Implements `PortalSessionRepository` against Valkey for hot session state; periodic flush to Postgres event log.

```
oya-emr-portal-session-adapter-valkey
```

### Layer 9 — `adapter-clients` (out-of-process peer µservice clients)

Per-peer-µservice client crate. Implements ports defined in kernel for "call peer µservice X" with retries, circuit breaker, deadline propagation. ADR-0145 §D-3 declares direct-gRPC as the canonical inter-µservice transport (no forced Workflow+Ontology adapter; the Workflow+Ontology-as-mandatory-bridge rule was RETIRED per ADR-0145).

```
oya-emr-adapter-client-pharmacy
oya-emr-adapter-client-diagnostics
oya-emr-adapter-client-clinical-decision-support
oya-emr-adapter-client-care-management
oya-emr-adapter-client-healthcare-integration
oya-emr-adapter-client-audit-chain
oya-emr-adapter-client-policy-engine
oya-emr-adapter-client-consent-graph
oya-emr-adapter-client-workflow-engine
oya-emr-adapter-client-cloud-billing
oya-emr-adapter-client-cloud-iam
oya-emr-adapter-client-cloud-kms
oya-emr-adapter-client-cloud-storage
oya-emr-adapter-client-observability
```

### Layer 10 — `worker` (background workers)

Workers for asynchronous workflows: BCMA scan ingestion, vital-stream ingestion, results-receive consumer, audit-emission worker, FHIR Bulk Export worker, legal-hold-application worker.

```
oya-emr-worker-bcma-ingest
oya-emr-worker-vital-stream
oya-emr-worker-results-consumer
oya-emr-worker-audit-emitter
oya-emr-worker-bulk-export
oya-emr-worker-legal-hold
oya-emr-worker-deidentify-projection
```

### Layer 11 — `app` (top-level service binary + composition root)

The `oya-emr-app` crate is the single binary deployed in a cell. It composes the REST server, gRPC server, AsyncAPI producer/consumer, workers, healthcheck, metrics endpoint, and configuration loader.

```
oya-emr-app
```

### Layer 12 — `governance` (ci-fitness checks)

Per the ADR-0131 + ADR-0132 governance pattern, the EMR-specific governance checks are owned by the central `governance` µservice (per ADR-0131 §migration §IP-M01-MIGR-014). EMR contributes lane definitions:

```
oya-governance-check-emr-bc-naming-conformance
oya-governance-check-emr-fhir-r5-default
oya-governance-check-emr-cedar-coverage
oya-governance-check-emr-audit-emission-coverage
oya-governance-check-emr-bcma-required
```

These crates live under `microservices/governance/src/crates/` per ADR-0131; EMR owns only the lane definitions referenced from `microservices/emr/governance/`.

## 3. Inter-µservice handoffs

EMR is centrally positioned — most healthcare µservices either dispatch to EMR or receive from EMR. Per ADR-0145, the canonical inter-µservice transport is direct gRPC (HTTP/3 + QUIC underneath per ADR-0253). The Workflow+Ontology-as-mandatory-adapter rule is RETIRED. Workflow + Ontology remain available as adapter patterns for cases requiring durable orchestration or projection.

### 3.1 → diagnostics (lab + imaging order dispatch)

When a clinician enters a lab or imaging order via CPOE (`order.enter_lab` / `order.enter_imaging`), the EMR `order` BC dispatches a `DispatchOrder` gRPC call to `diagnostics`:

```
proto3 service: diagnostics.OrderIngest
rpc DispatchOrder(DispatchOrderRequest) returns (DispatchOrderResponse)

DispatchOrderRequest carries:
  - tenant_id
  - patient_id
  - encounter_id
  - order_id (issued by EMR)
  - order_type (LAB | IMAGING)
  - LOINC | SNOMED code
  - specimen_source | modality
  - clinical_question
  - icd10_indication
  - priority
  - ordering_clinician_id
  - audit_envelope (idempotency_key, principal, purpose, data_class, pack_overlay)
```

`diagnostics` returns an accepted-order receipt. EMR persists the dispatch with state `DISPATCHED_TO_DIAGNOSTICS`. Subsequent results arrive via AsyncAPI on the `diagnostics.result.recorded.v1` channel; the `oya-emr-worker-results-consumer` worker materializes them under the `result` BC.

### 3.2 → pharmacy (ePrescribing + MAR)

`medication.prescribe` dispatches `DispatchPrescription` gRPC to `pharmacy`:

```
proto3 service: pharmacy.PrescriptionIngest
rpc DispatchPrescription(PrescriptionRequest) returns (PrescriptionResponse)
```

For Schedule II controlled substances, `EPCS-2FA-Attestation` is in the request envelope (proof-of-2FA + DEA-registrant attestation). The pharmacy µservice handles NCPDP SCRIPT transmission. MAR (Medication Administration Record) events return via AsyncAPI `pharmacy.medication.administered.v1`; the BCMA scan workflow handled by `oya-emr-worker-bcma-ingest`.

### 3.3 → clinical-decision-support (CDS Hooks 2.0)

At the following hook points, EMR invokes CDS:

- `patient-view` (chart-open)
- `medication-prescribe` (every prescribe-time)
- `order-sign` (every order-set commit)
- `order-select` (CPOE select)
- `appointment-book` (scheduling-time)
- `encounter-discharge` (discharge-medication-review)

Each invocation is a synchronous gRPC call with a deadline. The CDS service returns Cards (info / suggestion / hard-stop) which the EMR REST/UI surfaces. Hard-stop cards block the orchestration; suggestion cards offer an alternative; info cards inform.

```
proto3 service: clinical_decision_support.CdsHooks
rpc Invoke(InvokeRequest) returns (InvokeResponse)
```

Per CDS Hooks 2.0, each invocation also includes a per-hook-context payload (FHIR resources) so the CDS service can reason about the clinical context.

### 3.4 → emergency (ED-to-inpatient handoff)

When an ED-encounter transitions to inpatient admission, EMR dispatches `HandoffEdEncounter`:

```
proto3 service: emergency.HandoffIngest
rpc HandoffEdEncounter(HandoffRequest) returns (HandoffResponse)
```

The handoff request contains the full ED chart envelope. EMR receives back the inpatient encounter id + bed assignment. The encounter BC then transitions to inpatient status preserving all ED orders and notes.

### 3.5 → care-management (care-plan + episode-of-care)

On `encounter.discharge`, EMR publishes a `care-management.episode.opened.v1` event on AsyncAPI. The care-management µservice opens an episode-of-care, may create a longitudinal care plan, and may schedule follow-up tasks. Care-management can publish back `care-management.care-plan.updated.v1` which EMR consumes for the patient summary view.

### 3.6 → cloud-iam (caregiver + patient auth)

Every PHI-touching request runs through `cloud-iam` to:

- Resolve the principal from the bearer JWT (per ADR-0244 §D-7).
- Verify passkey-based 2FA freshness when policy requires.
- Refresh consent-graph state where applicable.

A typical request: `Bearer JWT` → `cloud-iam.AuthN` → principal + groups → policy-engine.Authorize (Cedar) → EMR use-case.

### 3.7 → policy-engine (Cedar evaluation, every PHI access)

Per ADR-0243, every PHI-touching action is gated through Cedar. EMR materializes principal + action + resource + context and calls:

```
proto3 service: policy_engine.Authorize
rpc Authorize(AuthorizeRequest) returns (AuthorizeResponse)
```

Cedar fragments under `microservices/emr/policies/` plus HIPAA-pack fragments under `microservices/governance/packs/HIPAA-2024/` plus tenant-specific fragments compose at evaluation time per ADR-0243 §D-4 baseline / pack / overlay / tenant layering.

### 3.8 → audit-chain (every state-change + PHI-access)

Every state-change + every PHI-access emits an audit event. EMR uses the audit-chain gRPC client:

```
proto3 service: audit_chain.Emit
rpc Emit(AuditEvent) returns (EmitResponse)
```

Audit events are batched per ADR-0263 §emission-batching and tamper-evident-sealed in the audit-chain µservice.

### 3.9 → cloud-billing (charge capture)

When billing codes are captured (`billing_code.capture`), EMR dispatches them via AsyncAPI to `cloud-billing.charge.captured.v1`. Cloud-billing handles claim generation + 837 submission + 835 reconciliation. Cloud-billing replies with claim-status events that EMR consumes for the patient-financial view.

### 3.10 → healthcare-integration (HL7 v2 + legacy interop)

For tenants with legacy HL7 v2 interfaces (ADT, ORM, ORU, MDM, SIU), EMR consumes the parsed FHIR envelopes from `healthcare-integration`. Outbound, EMR publishes FHIR resources to `healthcare-integration` for external EHR exchange (Carequality / Care Everywhere / TEFCA QHIN).

### 3.11 → consent-graph (consent + 42 CFR Part 2 segmentation)

For tenants with SUD-data-segmentation or special-category-consent, EMR queries the consent-graph at FHIR-read time to filter resources per patient consent. Returns a CONSENT_DENIED result for resources the principal lacks consent for; integrated with ADR-0251 cross-pack traffic rules.

### 3.12 → workflow-engine (durable sagas)

The following EMR workflows are durable sagas in the workflow-engine:

- `proxy-grant-saga` (US-PORT-013): KYC/ID verify → POA upload → consent attest → grant active.
- `break-glass-review-saga` (US-HIM-016): break-glass-fired → privacy-officer assigned → review → ratify or rescind.
- `legal-hold-saga` (US-HIM-015): legal-counsel attestation → hold active → audit notation → retention freeze.
- `patient-merge-saga` (FR-PAT-002): two ids → reference resolution → tombstone → 30-day reversibility window.
- `discharge-medication-review-saga` (US-INP encounter.discharge): outpatient med-list reconciliation → e-prescribing dispatch → patient education delivery.

## 4. Data envelopes + canonical types

### 4.1 Tenant scoping (ADR-0244)

Every persisted row carries:

```
tenant_id            (UUID; tenant primary identifier)
home_cell_id         (UUID; the cell the tenant resides in)
audience_type        (enum per ADR-0244 §D-7)
home_jurisdiction    (ISO 3166-1)
compliance_packs[]   (installed pack ids + versions)
```

Postgres + Citus shards on `tenant_id`. Cross-tenant queries are FORBIDDEN at the DB level (Citus distributed table constraint).

### 4.2 Audit envelope (every state-change)

Every persisted state-change carries:

```
audit_event_id       (UUID)
tenant_id            (foreign key)
principal_id         (UUID; the actor)
principal_type       (clinician | patient | proxy | system | break-glass)
action               (canonical action string; e.g., "emr.medication.prescribe")
resource_type        (e.g., "FHIR.MedicationRequest")
resource_id          (UUID)
purpose_of_use       (TREATMENT | PAYMENT | OPERATIONS | RESEARCH | PUBLIC-HEALTH | LEGAL | BREAK-GLASS)
data_class           (e.g., "phi-protected-health-information")
pack_overlay         (e.g., "HIPAA-2024")
idempotency_key      (caller-supplied UUID)
trace_id             (W3C TraceContext)
emitted_at           (UTC ISO 8601)
event_payload        (canonical JSON envelope)
```

### 4.3 Cedar context

When EMR invokes Cedar (per §3.7), the context block carries:

```
{
  "tenant": { "id": "tenant-acme-health", "jurisdiction": "US-CA" },
  "principal": { "id": "clinician-7821", "role": "physician", "specialty": "hospitalist" },
  "action": "emr.medication.prescribe",
  "resource": { "type": "FHIR.MedicationRequest", "id": "med-req-9921", "patient_id": "patient-5512" },
  "context": {
    "purpose_of_use": "TREATMENT",
    "care_team_member": true,
    "encounter_active": true,
    "encounter_id": "enc-7783",
    "minimum_necessary_assessed": true,
    "dea_schedule": "II",
    "epcs_2fa_passed": true,
    "pdmp_queried": true,
    "consent_state": "GRANTED",
    "session_assurance_level": "AAL3"
  }
}
```

Cedar fragments under `microservices/emr/policies/` resolve permit/deny.

### 4.4 FHIR R5 envelope

REST handlers return FHIR R5 resources by default. Example Patient resource:

```json
{
  "resourceType": "Patient",
  "id": "patient-5512",
  "meta": {
    "versionId": "37",
    "lastUpdated": "2026-05-21T10:30:15.123Z",
    "security": [
      {"system": "http://terminology.hl7.org/CodeSystem/v3-Confidentiality", "code": "R"},
      {"system": "http://terminology.hl7.org/CodeSystem/v3-ActReason", "code": "TREAT"}
    ],
    "tag": [
      {"system": "urn:oyatie:tenant", "code": "tenant-acme-health"},
      {"system": "urn:oyatie:pack", "code": "HIPAA-2024"}
    ]
  },
  "identifier": [...],
  "active": true,
  "name": [...],
  "telecom": [...],
  "gender": "female",
  "birthDate": "1968-03-15",
  "address": [...]
}
```

## 5. Cellular topology (Amazon-shape per ADR-0248)

EMR is shuffle-sharded across cells per ADR-0248. Per-cell topology:

```
Cell (tier-0 or tier-1; cert-level ∈ {hipaa-certified, hipaa-pci-certified, healthcare-sovereign}):
  Region: us-east-1 (or eu-west-3, ap-northeast-2, etc.)
  AZs: 3
  EMR pod replicas per AZ: 12 (tier-0) / 6 (tier-1)
  Postgres + Citus: 4 coordinators + 12 workers; tenant-sharded
  TimescaleDB: 3-node HA replicaset (per cell)
  Valkey Cluster: 6-node (3 master + 3 replica) for portal-session cache
  Kafka: 5-broker cluster shared with other µservices in cell
```

Tenants are pinned to a primary cell + DR cell pair. DR cell is in a different region (per ADR-0241 + manifest.json rpo_seconds=60).

## 6. Per-tenant data residency + replication

A tenant's PHI is materialized only in cells whose `cell_eligibility.cert_levels` intersect the tenant's `compliance_packs[].minimum_certification_level_set`. EMR enforces this at write time via the policy engine.

Cross-cell replication of PHI is metadata-only by default. Full content replication requires:

- An installed pack on the source tenant whose `cross_tenant_rules.cross_pack_traffic_default ∈ {permitted-with-agreement, case-by-case-cedar-permit}`.
- A signed BAA (business associate agreement) governing the destination cell.
- A Cedar permit per access (no standing access).

## 7. Performance + scaling architecture

### 7.1 Chart-open ≤ 800ms p99

Chart-open assembles Patient + active Problems + active Medications + Allergies + last-N Vitals + last-N Encounters + open Orders + recent Notes. Strategy:

1. **Snapshot tier (Valkey):** Pre-computed chart-summary blob keyed on `(tenant_id, patient_id, version)`. Updated on every state-change via change-data-capture worker. Read: O(1) Valkey GET.
2. **Delta tier (Postgres):** Reads any state-changes after the snapshot version. Typically <50 rows.
3. **Composition (Application layer):** Snapshot + delta merge → FHIR Bundle response.

If snapshot tier miss (cold patient), full read against Postgres + Citus → snapshot tier recomputation backgrounded.

### 7.2 Order entry ≤ 200ms p99

CPOE order entry path:

1. Order validated in `usecase` layer (Cedar; CDS Hooks 2.0; drug-allergy + drug-interaction; dose-range).
2. Order persisted to Postgres (single shard).
3. Dispatch to `pharmacy` / `diagnostics` via async outbox pattern.
4. UI receives confirmation; outbox flush asynchronous.

### 7.3 FHIR read ≤ 150ms p99

FHIR resource reads go through:

1. URL parsing + tenant + principal + Cedar → use-case input.
2. Postgres single-row fetch via shard key.
3. Domain conversion → FHIR R5 envelope.
4. JSON serialization.

Caching: most-requested resources (Patient, recent Encounter) read-through cached in cell-local Valkey with TTL 60s + cache-invalidate on state-change.

## 8. Storage layout

```
Postgres (Citus + Patroni):
  schema: emr
    table: patient                  (tenant-shard)
    table: encounter                (tenant-shard)
    table: problem                  (tenant-shard)
    table: medication               (tenant-shard)
    table: allergy                  (tenant-shard)
    table: note                     (tenant-shard)
    table: note_amendment           (tenant-shard, WORM)
    table: order                    (tenant-shard)
    table: order_set                (tenant-shard)
    table: result                   (tenant-shard)
    table: care_team_member         (tenant-shard)
    table: care_team_assignment     (tenant-shard)
    table: documentation_template   (tenant-shard)
    table: smart_phrase             (tenant-shard)
    table: billing_code             (tenant-shard)
    table: patient_education_item   (tenant-shard)
    table: patient_education_assign (tenant-shard)
    table: portal_session_log       (tenant-shard)

TimescaleDB:
  schema: emr_vital
    hypertable: vital_observation   (chunked on observed_at, tenant-shard)

Valkey Cluster:
  namespace: emr:session:<session_id>
  namespace: emr:chart_snapshot:<tenant>:<patient>
  namespace: emr:cds_cache:<patient>:<hook>

SeaweedFS (cloud-storage):
  bucket: emr-attachments-<tenant_id> (PHI-encrypted with tenant KEK)
    object: <encounter_id>/<note_id>/<attachment_id>
```

## 9. Configuration

Per cell + per tenant configuration, layered:

1. **Base config** (`microservices/emr/config/base.yaml`).
2. **Cell config** (`microservices/emr/config/cell-<cell-id>.yaml`).
3. **Pack-overlay** (from installed `compliance_packs[]`).
4. **Tenant-overlay** (`microservices/emr/config/tenants/<tenant-id>.yaml`).

Config keys include:

- `default_fhir_version` (R5 / R4).
- `chart_open_snapshot_ttl_seconds`.
- `cds_hooks_deadline_ms`.
- `bcma_required` (true / false).
- `epcs_2fa_required_for_schedules` (II / II+III / II+III+IV).
- `pdmp_required_for_schedules` (II / II+III / II+III+IV).
- `audit_emission_batch_size`.
- `audit_emission_batch_window_ms`.

## 10. Observability (per ADR-0263)

### 10.1 Logs

Structured logging (tracing + tracing_subscriber) at INFO default; per-tenant log level overridable via runtime config. PHI scrubbed at log-emit (no PHI fields in logs).

### 10.2 Metrics

Per-BC Prometheus metrics:

```
emr_chart_open_duration_seconds_bucket{tenant,cell,le}
emr_order_entry_duration_seconds_bucket{tenant,cell,bc=order,le}
emr_fhir_read_duration_seconds_bucket{tenant,cell,resource_type,le}
emr_fhir_write_duration_seconds_bucket{tenant,cell,resource_type,le}
emr_note_save_duration_seconds_bucket{tenant,cell,le}
emr_search_duration_seconds_bucket{tenant,cell,bc,le}
emr_cedar_evaluation_duration_seconds_bucket{tenant,cell,outcome,le}
emr_break_glass_invocations_total{tenant,cell,principal_role}
emr_audit_emissions_total{tenant,cell,event_class}
emr_active_clinician_sessions_gauge{tenant,cell}
emr_active_portal_sessions_gauge{tenant,cell}
```

### 10.3 Tracing

W3C TraceContext propagated through every request. Cross-µservice spans linked via b3 headers.

### 10.4 SLOs

Per-SLO files in `microservices/emr/slos/` (per OpenSLO 1.0 schema):

- `chart-open-latency.openslo.yaml`
- `order-entry-latency.openslo.yaml`
- `fhir-read-latency.openslo.yaml`
- `fhir-write-latency.openslo.yaml`
- `search-latency.openslo.yaml`
- `note-save-latency.openslo.yaml`
- `availability.openslo.yaml`
- `audit-emission-lag.openslo.yaml`
- `cedar-evaluation-latency.openslo.yaml`
- `cds-hooks-deadline-compliance.openslo.yaml`
- `break-glass-review-latency.openslo.yaml`

## 11. Security architecture

### 11.1 Authentication

- Clinicians: passkey (per ADR-0188) + AAL3 step-up for break-glass.
- Patients: passkey + biometric on mobile.
- Proxies: passkey + verified-grant audit chain.
- Service-to-service: SPIFFE/SPIRE X.509 SVID + mTLS.

### 11.2 Authorization

- Cedar at every PHI-touching action (per ADR-0243).
- Default-deny.
- Pack-fragment + tenant-fragment + base-fragment composition.
- Cross-tenant traffic: forbidden by default; per-pack pack-driven permit.

### 11.3 Encryption

- At rest: AES-256-GCM with tenant-scoped KEK (BYOK supported per `feedback_byok_everywhere_credentials`).
- In transit: TLS 1.3 + HTTP/3 + QUIC (per ADR-0253 KS#10).
- Inter-µservice: mTLS over QUIC.
- Audit chain: Ed25519 signing per event.

### 11.4 Key management

- KEKs wrapped by tenant-managed HSM (BYOK) or platform HSM (default).
- Rotation policy per HIPAA-2024 pack default 365 days; tenant-overridable.

## 12. Reliability + DR

### 12.1 RTO/RPO

- RTO ≤ 15 minutes (manifest.json).
- RPO ≤ 60 seconds (manifest.json).
- Achieved via: synchronous-cross-AZ Postgres replicas + asynchronous cross-region replication with conflict-free convergence on tenant-shard.

### 12.2 Backup

- WAL streamed to cell-local SeaweedFS + cross-region archive.
- Logical pg_dump weekly + monthly cold-tier.
- Audit-chain immutable + tamper-evident; archived 7y per HIPAA-2024.

### 12.3 Chaos engineering

- Quarterly cell-kill drill.
- Monthly AZ-failure drill.
- Weekly synthetic-tenant fault injection.

## 13. Migration strategy (Epic / Cerner / athenahealth → oyatie EMR)

For inbound migration:

1. Tenant provisions `migration-from-epic` (or `-cerner`, `-athena`) pack-overlay.
2. Source EHR exposes FHIR R4 Bulk Data endpoint (or proprietary exports for older versions).
3. `oya-emr-worker-bulk-export` ingestion-mode pulls patient cohorts in batches.
4. Per-patient FHIR resources mapped to oyatie EMR domain types (BC-aware).
5. Idempotent re-ingestion supported (FHIR Bundle conditional create).
6. Cutover saga: dual-write window → read-cutover → write-cutover → source-decommission.

## 14. Build + deploy

### 14.1 Build

```
cargo build -p oya-emr-app --release
```

### 14.2 Container

Multi-stage Dockerfile; distroless runtime; per-OS image per `supported_oses[]`.

### 14.3 Deploy

OpenTofu modules per `iac/<context>/` for the 6 deployment contexts. Helm chart deploys app + sidecars (auth-proxy, audit-emitter, otel-collector).

## 15. Future architecture roadmap

- FHIR R6 compatibility shim (ADR-MS-002 directs defer until R6 normative).
- Federated learning surface for population-health analytics (separate µservice; EMR exposes deidentified projection).
- ASTP HTI-2 / HTI-3 rules (when CMS finalizes).
- TEFCA Phase 3 QHIN-to-QHIN bridging.
- WHO ICD-11 cross-walk (currently ICD-10-CM / ICD-10-PCS).

## 16. References

- ADR-0105 13-layer canonical enum.
- ADR-0131 per-microservice flat layout.
- ADR-0132 single-concern + suite dissolution.
- ADR-0145 inter-microservice communication reform (direct gRPC; Workflow+Ontology adapter optional).
- ADR-0188 passkey/WebAuthn as canonical auth.
- ADR-0244 tenant as universal scoping primitive.
- ADR-0248 amazon-shape cellular architecture.
- ADR-0251 compliance pack + cell certification levels.
- ADR-0253 HTTP/3 + QUIC default protocol.
- ADR-0263 observability emission contract.
- HL7 FHIR R5 normative.
- USCDI v4 + draft v5.
- HHS HTI-1 Final Rule 2024.
- TEFCA Common Agreement v2 2024.
- CDS Hooks 2.0 spec.
