# Imaging Microservice — Architecture

`microservice: imaging`
`layer_enum_source: ADR-0105 (13-layer)`
`status: wave-15m-g-authored-2026-05-21`
`split_authority: ADR-0132 (no-grouping policy) + user directive 2026-05-21`

---

## 0. Executive Architecture Summary

The Imaging µservice is a single-concern medical-imaging platform, split from the bundled `diagnostics` µservice per ADR-0132. It owns the DICOM substrate (DIMSE + DICOMweb), the PACS / VNA store, all per-modality acquisition state machines, the radiologist reading-room workflow, AI image analysis vendor-neutral fan-out, 3D / MPR / MIP rendering, image annotation / measurement, prior comparison, radiation-dose tracking, all IHE Radiology profiles, FHIR ImagingStudy / DiagnosticReport[imaging] surfaces, mammography tracking, nuclear-medicine PET quantification, interventional radiology workflow, RIS integration, and patient-portal sharing.

Architecture style: hexagonal (ports + adapters) per the canonical 12-layer (ADR-0105 13-layer) enum + inward-only dependency flow. The DICOM substrate, PACS index, VNA blob store, AI marketplace adapter, RIS integration, and FHIR/DICOM dual emission are all ports. Modality acquisition workers, AI vendor adapters, and storage backends are adapters.

Deployment: cell-aware per ADR-0248 (Amazon cellular shape) with sovereign-cell mandatory for paid PHI. Pack-based compliance overlay per ADR-0251. Tenant-scoped per ADR-0244. Cedar-gated per ADR-0243.

Persistence: DICOM pixel data → `cloud-storage` blob substrate (S3-compatible) with erasure coding and 13-nine durability. PACS index + worklist + structured-report metadata → `cloud-data` relational substrate (PostgreSQL-compatible) with cell-local primary and cross-AZ replication. Audit-chain entries → `audit-chain` µservice (tamper-evident).

Communication: direct gRPC inter-µservice per ADR-0145 (Workflow+Ontology adapter retired). HTTP/3 + QUIC default per ADR-0253. AsyncAPI events on the substrate event mesh.

---

## 1. ADR-0105 13-Layer Enum Mapping

Each crate is named `oya-imaging-<bounded-context>-<layer>`.

### 1.1 Layer 1: api

External-facing API surface. Crates:

- `oya-imaging-dicomweb-api` — WADO-RS, QIDO-RS, STOW-RS, UPS-RS REST endpoints.
- `oya-imaging-fhir-api` — FHIR R5/R4 ImagingStudy, ImagingSelection, DiagnosticReport[imaging], DocumentReference[CDA].
- `oya-imaging-worklist-api` — Radiologist worklist REST.
- `oya-imaging-portal-api` — Patient portal DICOMweb token issuance.
- `oya-imaging-dimse-api` — DIMSE association listener (C-STORE/C-FIND/C-MOVE/C-GET/MWL/MPPS).

### 1.2 Layer 2: rest

REST presentation + request/response codecs. Crates:

- `oya-imaging-dicomweb-rest` — WADO-RS multipart encoding, QIDO-RS query-string parsing.
- `oya-imaging-fhir-rest` — FHIR JSON + XML codec.
- `oya-imaging-worklist-rest` — Radiologist worklist JSON.
- `oya-imaging-admin-rest` — PACS admin endpoints.

### 1.3 Layer 3: application

Use-case orchestration. Crates:

- `oya-imaging-acquisition-app` — orchestrate MWL → MPPS → C-STORE → tech-QC → forward-to-worklist.
- `oya-imaging-reading-app` — orchestrate worklist load → hanging protocol → prior fetch → AI overlay → structured-report.
- `oya-imaging-critical-result-app` — orchestrate critical-result closed-loop.
- `oya-imaging-peer-review-app` — orchestrate blinded peer review.
- `oya-imaging-mammography-tracking-app` — orchestrate screening recall + BI-RADS audit.
- `oya-imaging-dose-tracking-app` — orchestrate RDSR parse + dose-deviation alert.
- `oya-imaging-ai-marketplace-app` — orchestrate AI vendor dispatch + drift detection.
- `oya-imaging-vna-federation-app` — orchestrate XDS-I.b / XCA-I federation.
- `oya-imaging-patient-portal-app` — orchestrate consent-gated patient access.

### 1.4 Layer 4: usecase

Single-step use-cases. Crates per bounded context (24 × ~5 use-cases each). Examples:

- `oya-imaging-pacs-usecase` — query-study, retrieve-instance, store-instance.
- `oya-imaging-hanging-protocol-usecase` — match-protocol, apply-protocol.
- `oya-imaging-structured-report-usecase` — start-report, save-report, sign-report.

### 1.5 Layer 5: domain

Pure-domain types + business rules. Crates:

- `oya-imaging-dicom-domain` — Study/Series/Instance/AE-Title/Transfer-Syntax types.
- `oya-imaging-rad-workflow-domain` — Worklist/Priority/HangingProtocol types.
- `oya-imaging-report-domain` — SR / FHIR DiagnosticReport types.
- `oya-imaging-dose-domain` — RDSR / DLP / CTDI types.
- `oya-imaging-ai-domain` — CADe/CADx finding / vendor / model-version types.
- `oya-imaging-mammography-domain` — BI-RADS / MQSA-audit types.
- `oya-imaging-nuclear-domain` — SUV / Deauville / PERCIST types.
- `oya-imaging-ihe-domain` — XDS-I.b / XCA-I / IRWF.b actor types.

### 1.6 Layer 6: kernel

Cross-cutting kernel + ports. Crates:

- `oya-imaging-kernel` — central error model, tenant context propagation, Cedar evaluation ports, audit-chain emission ports, cloud-storage ports.

### 1.7 Layer 7: adapter

Outbound adapters. Crates:

- `oya-imaging-adapter-cloud-storage` — DICOM pixel-data blob persistence.
- `oya-imaging-adapter-cloud-data` — PACS index relational persistence.
- `oya-imaging-adapter-audit-chain` — PHI access audit emission.
- `oya-imaging-adapter-cloud-iam` — Cedar policy evaluation.
- `oya-imaging-adapter-cloud-kms` — BYOK envelope encryption.
- `oya-imaging-adapter-cloud-billing` — per-seat + per-usage meter emission.
- `oya-imaging-adapter-workflow-engine` — critical-result escalation workflow.
- `oya-imaging-adapter-emr` — order receipt + report distribution.
- `oya-imaging-adapter-consent-graph` — patient consent + share-link consent.
- `oya-imaging-adapter-ai-vendor-aidoc` — Aidoc CADe/CADx adapter.
- `oya-imaging-adapter-ai-vendor-vizai` — Viz.ai LVO triage adapter.
- `oya-imaging-adapter-ai-vendor-cleerly` — Cleerly coronary adapter.
- `oya-imaging-adapter-ai-vendor-radai` — Rad AI summary adapter.
- `oya-imaging-adapter-ai-vendor-annalise` — Annalise.ai CXR adapter.
- `oya-imaging-adapter-ai-vendor-lunit` — Lunit mammography adapter.
- `oya-imaging-adapter-ai-vendor-qure` — Qure.ai brain/CXR adapter.
- `oya-imaging-adapter-ai-vendor-zebra` — Zebra Medical adapter.
- `oya-imaging-adapter-ai-vendor-arterys` — Arterys cardiac adapter.
- `oya-imaging-adapter-ai-vendor-caption` — Caption Health POCUS adapter.
- `oya-imaging-adapter-ai-vendor-rapidai` — RapidAI stroke adapter.
- `oya-imaging-adapter-ai-vendor-subtle` — Subtle Medical denoise adapter.
- `oya-imaging-adapter-ai-vendor-imagia` — Imagia adapter.
- `oya-imaging-adapter-ai-vendor-behold` — Behold.ai CXR adapter.
- `oya-imaging-adapter-ai-vendor-screenpoint` — ScreenPoint mammography adapter.
- `oya-imaging-adapter-voice-nuance` — Nuance Dragon Medical adapter.
- `oya-imaging-adapter-voice-mmodal` — M*Modal Catalyst adapter.
- `oya-imaging-adapter-voice-whisper-medical` — In-house Whisper-medical adapter.
- `oya-imaging-adapter-ihe-xds-i-b` — XDS-I.b cross-enterprise adapter.
- `oya-imaging-adapter-ihe-xca-i` — XCA-I cross-community adapter.

### 1.8 Layer 8: worker

Async workers. Crates per modality + per cross-cutting workflow. Examples:

- `oya-imaging-acquisition-ct-worker` — CT acquisition state machine.
- `oya-imaging-acquisition-mri-worker` — MRI acquisition state machine.
- `oya-imaging-acquisition-xr-worker` — X-ray acquisition state machine.
- `oya-imaging-acquisition-us-worker` — Ultrasound acquisition state machine.
- `oya-imaging-acquisition-mg-worker` — Mammography acquisition state machine.
- `oya-imaging-acquisition-nm-worker` — Nuclear medicine acquisition state machine.
- `oya-imaging-acquisition-pt-worker` — PET acquisition state machine.
- `oya-imaging-acquisition-rf-worker` — Fluoroscopy acquisition state machine.
- `oya-imaging-acquisition-xa-worker` — Angiography acquisition state machine.
- `oya-imaging-acquisition-ir-worker` — Interventional acquisition state machine.
- `oya-imaging-acquisition-bmd-worker` — DEXA acquisition state machine.
- `oya-imaging-acquisition-oct-worker` — OCT acquisition state machine.
- `oya-imaging-acquisition-echo-worker` — Echo acquisition state machine.
- `oya-imaging-acquisition-cath-worker` — Cath acquisition state machine.
- `oya-imaging-acquisition-derm-worker` — Dermatology clinical-photo worker.
- `oya-imaging-acquisition-path-wsi-worker` — Pathology WSI worker.
- `oya-imaging-acquisition-dental-worker` — Dental worker.
- `oya-imaging-acquisition-surgical-worker` — Surgical-video worker.
- `oya-imaging-vna-federation-worker` — Cross-VNA pull/push.
- `oya-imaging-prior-fetch-worker` — Auto-fetch priors.
- `oya-imaging-ai-dispatch-worker` — AI vendor dispatch + drift detection.
- `oya-imaging-mpr-render-worker` — Server-side MPR.
- `oya-imaging-mip-render-worker` — Server-side MIP.
- `oya-imaging-critical-result-escalation-worker` — Closed-loop escalation.
- `oya-imaging-mammography-recall-worker` — Screening recall queue.
- `oya-imaging-dose-tracking-worker` — RDSR aggregate compute.
- `oya-imaging-deidentification-worker` — PHI strip before AI vendor egress.
- `oya-imaging-layperson-summary-worker` — Plain-language summary for patient portal.

### 1.9 Layer 9: governance

Policy + Cedar evaluation. Crates:

- `oya-imaging-governance-cedar` — Cedar policy bundle for imaging.
- `oya-imaging-governance-hipaa` — HIPAA pack overlay.
- `oya-imaging-governance-mqsa` — MQSA pack overlay.
- `oya-imaging-governance-eu-mdr` — EU-MDR pack overlay.
- `oya-imaging-governance-kr-md` — KR-Medical-Devices pack overlay.

### 1.10 Layer 10: infra-binding

OpenTofu + Helm + Kubernetes manifests. Per-deployment-context binding (covered in §6 + `iac/`).

### 1.11 Layer 11: observability

OTel exporters, SLO definitions, dashboards. Crates:

- `oya-imaging-observability-otel` — OTel tracing/metrics/logs.
- `oya-imaging-observability-slo` — SLO definitions (mirror of `slos/` content as code).

### 1.12 Layer 12: policy

Cedar policy authoring + test surfaces. Crates:

- `oya-imaging-policy-authoring` — Cedar policy DSL helpers.
- `oya-imaging-policy-test` — Cedar policy unit-test harness.

### 1.13 Layer 13: evidence

Evidence emission for compliance / audit / SLO-gated promotion. Crates:

- `oya-imaging-evidence-emitter` — emit promotion-ready evidence bundles.
- `oya-imaging-evidence-conformance` — DICOM PS3.4 + IHE conformance evidence.

---

## 2. Hexagonal Ports & Adapters

### 2.1 Inbound Ports

| Port | Surface | Bounded Context |
|------|---------|-----------------|
| DIMSE listener | C-STORE/C-FIND/C-MOVE/C-GET/MWL/MPPS over DICOM upper-layer protocol | DICOMSubstrate |
| DICOMweb HTTP/3 | WADO-RS/QIDO-RS/STOW-RS/UPS-RS over HTTP/3+QUIC | DICOMweb |
| FHIR HTTP/3 | FHIR R5/R4 resources | RISIntegration / PatientPortalSharing |
| Worklist gRPC | Radiologist worklist queries | RadiologistWorklist |
| Reading-room gRPC | Hanging-protocol + structured-report streaming | HangingProtocol / StructuredReport |
| Voice WebSocket | Voice-recognition partial-transcript streaming | VoiceRecognition |
| AI vendor webhook | AI vendor result callback | AIImageAnalysis |
| Peer-review gRPC | Peer review queue + submission | PeerReview |
| Patient portal HTTP/3 | DICOMweb token + plain-language summary | PatientPortalSharing |
| Admin gRPC | PACS / VNA admin + modality config | (admin) |
| EMR order gRPC | Order receipt from `emr` | RISIntegration |
| Dose dashboard HTTP/3 | Dose tracking export | DoseTracking |
| Mammography tracking HTTP/3 | Screening recall + BI-RADS audit | MammographyTracking |

### 2.2 Outbound Ports

| Port | Adapter target | Bounded Context |
|------|----------------|-----------------|
| BlobStore | `cloud-storage` (S3-compatible) | VNA |
| RelationalStore | `cloud-data` (PostgreSQL-compatible) | PACS / RadiologistWorklist / StructuredReport |
| EncryptionKey | `cloud-kms` (BYOK envelope encryption) | (all PHI persistence) |
| CedarEval | `cloud-iam` (Cedar policy evaluation) | (all read/write paths) |
| AuditEmit | `audit-chain` (tamper-evident PHI audit) | (all PHI access) |
| BillingMeter | `cloud-billing` (per-seat + per-usage meters) | (all paid-tenant usage) |
| WorkflowEnqueue | `workflow-engine` (critical-result + recall) | CriticalResults / MammographyTracking |
| EMROrder | `emr` (order receipt + report distribution) | RISIntegration |
| ConsentCheck | `consent-graph` (patient consent + share-link) | PatientPortalSharing |
| CompliancePack | `compliance` (pack overlay enforcement) | (all PHI surfaces) |
| ObservabilityEmit | `observability` (OTel + SLO) | (all surfaces) |
| AIVendorDispatch | per-AI-vendor adapter (Aidoc/Viz.ai/Cleerly/...) | AIImageAnalysis |
| VoiceVendorDispatch | per-voice-vendor adapter (Nuance/M*Modal/Whisper-medical) | VoiceRecognition |
| LegacyVNAImport | per-vendor VNA adapter (GE-EA/Philips-ISyntax-VNA/Sectra-VNA/Fujifilm-Synapse-VNA/Agfa-Impax-VNA/Merge-VNA) | VNA |
| IHEXDSIB | XDS-I.b cross-enterprise adapter | IHEProfiles |
| IHEXCAI | XCA-I cross-community adapter | IHEProfiles |

---

## 3. DICOM Substrate Architecture

### 3.1 DIMSE Layer

The DIMSE upper-layer protocol listener accepts TCP connections on the configured AE Title + port. Per association:

1. Accept connection, negotiate presentation context (transfer syntax, SOP class).
2. Authenticate via Cedar pre-association policy (tenant-bound AE Title pairing).
3. For each PDU:
   - C-STORE: parse DIMSE command + data set, validate against SOP class, write pixel data to `cloud-storage` blob, write metadata to `cloud-data` index, emit `imaging.study.received` async event, emit audit-chain entry.
   - C-FIND: parse query, translate to QIDO-RS internal query, stream matches as DIMSE C-FIND-RSP.
   - C-MOVE: parse query, translate to internal retrieve, push to destination AE Title via outbound C-STORE-SCU.
   - MWL C-FIND: query worklist scheduled-procedure table.
   - MPPS N-CREATE/N-SET: persist MPPS state changes + emit `imaging.study.acquired` event when COMPLETED.
4. Sustain ≥10,250 instances/min per pod (preserving healthcare-integration claim).

### 3.2 DICOMweb Layer

DICOMweb is the primary substrate for HTTP/3+QUIC native clients (modern zero-footprint viewers).

- **WADO-RS** (retrieve):
  - `/studies/{study}` — multipart/related instance retrieval.
  - `/studies/{study}/series/{series}/instances/{instance}` — single instance.
  - `/studies/{study}/series/{series}/instances/{instance}/frames/{frame}` — single frame.
  - `/studies/{study}/series/{series}/instances/{instance}/metadata` — JSON metadata.
  - `/studies/{study}/series/{series}/instances/{instance}/rendered` — rendered JPEG/PNG.
- **QIDO-RS** (query):
  - `/studies?{filter}` — study-level query.
  - `/studies/{study}/series?{filter}` — series-level query.
  - `/studies/{study}/series/{series}/instances?{filter}` — instance-level query.
- **STOW-RS** (store):
  - `POST /studies` — multipart/related store.
  - `POST /studies/{study}` — explicit study-level store.
- **UPS-RS** (workflow):
  - `/workitems` — create/query/update workitems for radiologist worklist.

All endpoints Cedar-gated, tenant-scoped, audit-emitted. HTTP/3 + QUIC default per ADR-0253.

### 3.3 PACS Index

PostgreSQL-compatible relational store (via `cloud-data`). Tables:

- `study` (study_instance_uid, tenant_id, patient_id, accession_number, study_date, modality, body_part, study_description, referring_physician, ...).
- `series` (series_instance_uid, study_instance_uid, modality, series_number, ...).
- `instance` (sop_instance_uid, series_instance_uid, sop_class_uid, transfer_syntax_uid, blob_storage_uri, blob_size, ...).
- `mpps` (mpps_uid, study_instance_uid, status, start_time, end_time, performed_protocol_codes, ...).
- `mwl_workitem` (workitem_uid, scheduled_procedure_step_id, modality, scheduled_date, patient_id, ...).
- `presentation_state` (pr_uid, instance_uid, radiologist_id, layout, window_level, annotations, ...).
- `structured_report` (sr_uid, instance_uid, template_id, status, signing_radiologist_id, content, ...).

Cell-local primary. Cross-AZ replication. Tenant-id partition key on every table (ADR-0244).

### 3.4 VNA Blob Store

`cloud-storage` S3-compatible. Object key: `<tenant>/<study_uid>/<series_uid>/<instance_uid>.dcm`. Erasure-coded 14+4 across cells per pack. 13-nine durability. Per-tenant KMS envelope per ADR-0255 §D-4 BYOK opt-in.

VNA federation outbound: XDS-I.b Imaging Document Consumer + XCA-I Initiating Imaging Gateway. Inbound: Imaging Document Source + Responding Imaging Gateway. Legacy import: per-vendor adapter (GE EA via SOAP + Philips ISyntax-VNA via REST + Sectra VNA via XDS-I.b + Fujifilm Synapse VNA via DICOM C-MOVE + Agfa Impax via REST + Merge VNA via SOAP).

---

## 4. Modality Acquisition Architecture

### 4.1 Per-Modality State Machine

Each modality has a Rust state machine in `oya-imaging-acquisition-<modality>-worker`. Universal state transitions:

```
SCHEDULED -> ARRIVED -> MWL_PULLED -> ACQUIRING (MPPS:IN_PROGRESS)
            -> ACQUIRE_COMPLETE (MPPS:COMPLETED) -> C_STORED -> TECH_QC -> FORWARDED_TO_WORKLIST
```

Modality-specific deltas:

- CT: emit RDSR on ACQUIRE_COMPLETE.
- MRI: emit per-sequence metadata.
- Mammography: emit breast positioning metadata (MQSA) on ACQUIRE_COMPLETE.
- Fluoroscopy / IR: emit cumulative fluoroscopy time + air-kerma area product.
- Ultrasound: live preview frames during ACQUIRING.
- NM/PET: emit SUV calibration metadata.
- Pathology WSI: handle very-large slide sizes (10–100+ GB) via tiled JPEG2000 multi-resolution.

### 4.2 MWL / MPPS

Modality Worklist (C-FIND-MWL-SCP):

- Source: `mwl_workitem` table populated by `emr` order handoff.
- Query: per-modality, per-AE-title, scheduled-date range, accession.

MPPS (N-CREATE/N-SET):

- IN_PROGRESS: modality begins acquisition.
- DISCONTINUED: modality discontinues (technologist abort).
- COMPLETED: acquisition complete; triggers C-STORE + dose-emission.

### 4.3 Modality Vendor Quirks

A `oya-imaging-modality-vendor-quirks` library handles known private-tag patterns:

- GE Advantage Workstation private tags.
- Siemens syngo private tags (esp. cardiac gating).
- Philips IntelliSpace private tags.
- Canon (formerly Toshiba) private tags.
- Hologic Selenia (mammography DBT) private tags.
- Hitachi MRI private tags.
- Mindray ultrasound private tags.

Each vendor's quirks are encapsulated in a regression-test set per modality.

---

## 5. Radiologist Workflow Architecture

### 5.1 Worklist

`oya-imaging-worklist-app` reads from the `study` + `mpps` tables filtered to FORWARDED_TO_WORKLIST status. Sort/filter parameters:

- Priority (STAT > preliminary > routine > addendum).
- Modality.
- Body-part.
- Sub-specialty group (neuro / msk / chest / breast / abd-pelvis / cardio / peds).
- Sub-specialty routing rules per radiologist credential (e.g., MQSA-certified for mammography reads).
- Stable secondary sort: acquisition time ascending.

Worklist load SLO: p95 < 800ms for ≤5000 items (FR-RAD-003).

### 5.2 Hanging Protocols

Hanging-protocol engine matches on (modality, body-part, study-description, prior-availability, radiologist-id). Match algorithm:

1. Exact radiologist + modality + body-part match (highest priority).
2. Radiologist + modality wildcard body-part.
3. Tenant default per (modality, body-part).
4. Built-in fallback per modality.

Protocol expression: DICOM PS 3.18 Hanging Protocol IOD OR Oyatie-extension YAML for richer features (e.g., per-radiologist viewport-count preference, custom annotation visibility).

Apply SLO: p95 < 150ms (FR-RAD-004).

### 5.3 Structured Report

Templates: BI-RADS (mammography), LI-RADS (liver), PI-RADS (prostate), TI-RADS (thyroid), Lung-RADS, O-RADS, CAD-RADS, Bone-RADS, NI-RADS, plus general ACR template library + custom-template upload.

Persistence: DICOM SR (TID-1500 + per-template TID) + FHIR DiagnosticReport[imaging] dual emission. Terminology: RadLex (RID#####), SNOMED-CT, ICD-10, RSNA RadElement.

Save SLO: p95 < 800ms (FR-RAD-008).

### 5.4 Voice Recognition

Streaming voice-recognition partial transcripts. Vendors: Nuance Dragon Medical One, M*Modal Catalyst, in-house Whisper-medical fine-tune. Adapter abstraction in `oya-imaging-adapter-voice-*`.

Latency SLO: partial transcript p95 < 250ms (FR-RAD-007).

### 5.5 Critical Results Closed-Loop

`oya-imaging-critical-result-app` triggers when structured-report tagging includes a critical-finding code (per ACR Critical Results categories I/II/III). Cascade:

1. Notify ordering clinician (push + SMS + voice via comms-* µservices) with confirmation timer (per criticality).
2. On timer expiry without confirmation: notify covering clinician.
3. On timer expiry: notify charge nurse / unit secretary.
4. On timer expiry: notify on-call attending.
5. On final timer: notify patient safety officer + raise incident.

Workflow engine: `workflow-engine` µservice with retry semantics.

Audit: every step emits audit-chain entry. Closed-loop confirmation requires explicit user action (one-click or voice).

p99 < 30s to ordering clinician notification (FR-RAD-010).

### 5.6 Peer Review

`oya-imaging-peer-review-app` selects 5% random sample + targeted high-risk subsets:

- Post-procedure (within 48h after IR procedure).
- Transfer-in studies (post-acceptance read).
- Post-AI-disagreement (AI marketplace flagged + radiologist read differs).
- Pre-litigation (legal flag).

Blinded view: reviewer sees images + study metadata but NOT the primary report until peer-review submit. Discordance scoring per ACR RadPeer 3-point scale (concordant / minor discrepancy / major discrepancy). Aggregate per radiologist + tenant. ACR submission via RadPeer adapter.

### 5.7 Prior Comparison

`oya-imaging-prior-fetch-worker` triggers on worklist-item-open event. Algorithm:

1. Query PACS index for same-patient priors (last 24 months default, configurable).
2. Filter by modality + body-part match (relaxable).
3. Cross-VNA federate via XCA-I if no local match.
4. Stream to viewer.

p95 < 3s (FR-RAD-012).

### 5.8 MPR / MIP

`oya-imaging-mpr-render-worker` performs server-side rendering (Visage-class architecture). Input: volume series (CT, MRI). Output: axial / coronal / sagittal / oblique reformations + thick-slab MIP. Client-side WebGL fallback for low-bandwidth.

p95 < 2s (FR-RAD-013).

### 5.9 AI Overlay

`oya-imaging-ai-marketplace-app` dispatches to per-vendor adapter when:

- Tenant enables vendor + (modality, body-part, indication) match.
- PHI is de-identified per HIPAA Safe Harbor + ISO/TS 25237 BEFORE vendor egress.
- Vendor model version is FDA-cleared / CE-marked.

Overlay rendering: per-finding bounding boxes / heat maps / volumetric ROIs. Toggle per radiologist preference.

Dispatch p95 < 500ms (FR-AI-002).

---

## 6. Deployment Architecture (Cell-Aware)

### 6.1 Cell Topology

Per ADR-0248 (Amazon cellular shape):

- **Tier-0 cells** — sovereign per-region per-pack (HIPAA-US, GDPR-EU, KR-MD, EU-MDR). Mandatory for paid PHI.
- **Tier-1 cells** — regional multi-pack (e.g., US generic, EU generic).
- **Tier-2 cells** — multi-region failover.
- **Tier-3 cells** — demo_trial only.
- **Tier-4 cells** — emergency cold-standby.

Imaging µservice runs in Tier-0 / Tier-1 / Tier-2. Demo_trial may run in Tier-3.

### 6.2 Pod Topology Per Cell

- DIMSE listener pods (StatefulSet for AE-title binding): 3 replicas minimum, scale to 30 per cell at 10× peak.
- DICOMweb HTTP/3 pods (Deployment): 5 replicas minimum, scale to 50 per cell.
- Acquisition workers per modality (StatefulSet): 2 replicas minimum per active modality.
- AI marketplace dispatcher (Deployment): 3 replicas minimum.
- MPR/MIP render workers (Deployment, GPU): 2 replicas minimum, scale on queue depth.
- Voice-recognition adapter (Deployment): 3 replicas minimum.
- Audit-emitter sidecar: per pod.
- Cedar evaluator sidecar: per pod (or shared).

### 6.3 Shuffle Sharding

Per ADR-0248: each tenant maps to a unique shuffled subset of pods within a cell, limiting blast radius. Imaging-specific shuffle key: `sha256(tenant_id || cell_id)[:6]` mapped to a shard slot of 4-of-N pods per surface.

### 6.4 Cloud Hypervisor + Kata

Per ADR-0254: pods run on Cloud Hypervisor + Kata Containers for VM-grade isolation. Required for PHI workloads.

---

## 7. Cross-µservice Handoffs

### 7.1 emr → imaging

Order receipt: `emr` calls `oya-imaging-rest::POST /orders` (or gRPC equivalent) when imaging order is placed. Imaging persists to `mwl_workitem`. MWL becomes available to modality on next C-FIND-MWL.

Report distribution: imaging publishes `imaging.report.signed` event on AsyncAPI mesh. `emr` consumes + parses structured report back into discrete EMR fields.

### 7.2 imaging → diagnostics

Lab-result-image correlation: after Wave 15M reconciliation, imaging will surface a gRPC RPC `GetImagingForLabContext(patient, accession_or_orderset)` for diagnostics to correlate lab results with concurrent imaging. Bidirectional event subscription.

### 7.3 imaging → cloud-storage

Pixel data persistence: `oya-imaging-adapter-cloud-storage` writes `<tenant>/<study_uid>/<series_uid>/<instance_uid>.dcm` blobs with envelope encryption (KMS key from `cloud-kms`).

### 7.4 imaging → cloud-iam

Cedar policy evaluation on every read/write/admin call. Policy bundle published per `oya-imaging-governance-cedar`.

### 7.5 imaging → audit-chain

Every PHI access emits audit-chain entry. HIPAA 164.312(b) audit-control requirement.

### 7.6 imaging → cloud-billing

Per-seat meter: monthly per named radiologist / technologist / referring-clinician. Per-usage meter: study-archived-GB-month + AI-inference-run + voice-minute + DICOM-egress-GB + MWL-query.

### 7.7 imaging → consent-graph

Patient consent check on patient-portal access. Share-link consent for cross-clinician sharing.

### 7.8 imaging → workflow-engine

Critical-result closed-loop escalation. Mammography recall workflow.

### 7.9 imaging → compliance

Pack overlay enforcement. HIPAA / GDPR / KR-MD / EU-MDR / MQSA / GxP packs.

---

## 8. AI Marketplace Architecture

### 8.1 Vendor-Neutral Adapter Layer

Each AI vendor has a `oya-imaging-adapter-ai-vendor-<vendor>` crate implementing the common `AiVendorPort` trait:

```
trait AiVendorPort {
  async fn dispatch(&self, study_de_id: DeIdentifiedStudy, indication: Indication, model_version: ModelVersion) -> Result<AiResult>;
  async fn health(&self) -> Result<AiVendorHealth>;
  async fn list_clearances(&self) -> Result<Vec<FdaCeClearance>>;
}
```

Result types are normalized to a common `AiResult` enum: BoundingBox / HeatMap / VolumetricROI / Quantification / Triage.

### 8.2 De-identification

PHI strip via `oya-imaging-deidentification-worker` BEFORE vendor egress. Implements HIPAA Safe Harbor 18-identifier removal + ISO/TS 25237 pseudonymization. Per-tenant configurable retention of pseudonym mapping for re-identification on result return.

### 8.3 Drift Detection

`oya-imaging-ai-dispatch-worker` tracks per-vendor positive-predictive-value, sensitivity, specificity weekly. Alert when PPV drops >10% week-over-week (FR-AI-005).

### 8.4 FDA / CE Clearance Metadata

Per vendor model version: stored clearance number, cleared indications, cleared modalities. Off-label inference blocked by Cedar policy `ai-model-can-read-deidentified.cedar`.

---

## 9. Compliance Pack Architecture

### 9.1 HIPAA-2024 Pack

- Cedar policy bundle: `policies/hipaa-deny-default.cedar`.
- Audit completeness: 100% per FR-DOSE / FR-RAD / FR-AI / FR-PORTAL paths.
- Break-glass: `policies/break-glass-emergency.cedar`.
- Minimum-necessary: per-role minimum-necessary policies.
- BAA: per BAA template managed in `compliance` µservice.

### 9.2 GDPR Pack

- Right-to-erasure: study-level cryptographic shred via FR-PACS-007.
- Data residency: sovereign-cell EU mandatory.
- DPIA: documented per ADR-0251 pack.

### 9.3 KR-Medical-Devices Pack

- 의료기기법 cybersecurity guidance compliance.
- 식약처 (MFDS) device classification.
- Sovereign-cell KR mandatory.

### 9.4 EU-MDR Pack

- Class IIa designation for AI CADe per Annex VIII Rule 11.
- Annex I §17 cybersecurity essential requirements.
- Article 10 manufacturer obligations.

### 9.5 EU-AI-Act Pack

- Annex III §3 medical-device-AI high-risk classification.
- Article 6 high-risk obligations: risk management, data governance, technical documentation, record-keeping, transparency, human oversight, accuracy/robustness, post-market surveillance.

### 9.6 MQSA Pack (US Mammography)

- 21 CFR 900 retention: 5 years minimum / 10 years if abnormal.
- Audit per FR-MAMMO-002.

### 9.7 GxP Pack

- GAMP-5 categorization Category 4 (configured product) for the modality-specific configurations.
- 21 CFR Part 11 electronic record + electronic signature.

### 9.8 NEMA DICOM PS3 Conformance Pack

- Full PS 3.1..3.20 conformance statement per release.

### 9.9 IHE Radiology Pack

- XDS-I.b / XCA-I / IRWF.b / SWF.b / REM / AIW-I / PIR / PDQ / PIX / ATNA / CT / EUA / PWP.

---

## 10. Disaster Recovery + Continuity

### 10.1 RPO / RTO

- RPO: 0 for PHI pixel data (synchronous cross-AZ replication).
- RTO: 5 minutes for primary cell failure (cell-level failover).
- Cross-cell failover: 1 hour for full pack-policy honoring.

### 10.2 Cell-Level Failover

Failover scope: per-pack-region. Imaging primary cell → failover cell in same pack. No cross-pack failover (e.g., EU pack does not failover to US pack — GDPR violation).

### 10.3 Backup

- DICOM pixel data: erasure-coded 14+4 per cell; cross-AZ replicated; cross-cell within same pack.
- PACS index: streaming WAL backup; cross-AZ replicated.
- Audit-chain: cross-cell replicated; tamper-evident hash chain.

### 10.4 Backfill / Replay

Per-study replay supported via `oya-imaging-vna-federation-worker` re-pull from legacy VNA + re-emit `imaging.study.received` events.

---

## 11. Capacity Model

### 11.1 Per-Cell Capacity Targets

- 50,000 instances/min sustained C-STORE per cell (5 pods × 10,250 inst/min/pod).
- 500,000 active worklist items.
- 50,000 active radiologist seats.
- 100M studies under management per cell.

### 11.2 Scale-out Triggers

- Pod CPU > 70% sustained 5min → HPA scale up.
- C-STORE queue depth > 1000 → scale up.
- AI dispatch queue depth > 500 → scale up.
- MPR render queue depth > 100 → scale up GPU pods.

### 11.3 Admission Control

Per-tenant rate limits at edge (api-gateway). Per-pack quota enforcement. Cell-level admission control via `oya-imaging-kernel` admission-control library.

---

## 12. Observability

### 12.1 OTel Tracing

Every DICOM association, DICOMweb request, AI dispatch, voice-recognition WebSocket frame emits OTel spans. Span attributes: `tenant_id`, `study_instance_uid`, `modality`, `bounded_context`, `cell_id`, `pack_id`.

### 12.2 RED Metrics

Per bounded context: rate (per-second request rate), errors (5xx rate), duration (p50/p95/p99).

### 12.3 SLO Definitions

OpenSLO format at `slos/`. ≥12 SLOs covering all critical surfaces.

### 12.4 Dashboards

- C-STORE throughput per cell per modality.
- DICOMweb latency per endpoint.
- Worklist load latency per radiologist.
- AI vendor health per vendor.
- Critical-results closed-loop completion rate.
- Mammography recall latency.
- Dose-deviation rate per protocol.

---

## 13. Security Architecture

### 13.1 TLS / mTLS

All inter-µservice gRPC: mTLS with per-pod cert. Cert rotation via `cloud-secrets`. PQC migration path per ADR-0253.

### 13.2 DIMSE TLS

DIMSE associations require TLS per IHE ATNA. AE-title pairing pre-authorization.

### 13.3 Encryption at Rest

DICOM pixel-data blobs: envelope-encrypted with per-tenant DEK + KMS-wrapped KEK. BYOK per ADR-0255 §D-4 opt-in.

### 13.4 PHI Egress Controls

AI vendor egress: PHI-stripped pixel data only. Cedar policy `ai-model-can-read-deidentified.cedar` denies if PHI fields present. Patient portal: consent-graph-gated DICOMweb token, time-boxed, revocable. Cross-clinician share: XCA-I federated, consent-graph-checked.

### 13.5 Audit

Every PHI access emits to `audit-chain`. HIPAA 164.312(b). MQSA audit per FDA 21 CFR 900. GDPR Article 30 record of processing activities.

---

## 14. Migration Architecture (Legacy VNA Import)

### 14.1 Per-Vendor Adapters

- GE Enterprise Archive: SOAP `RetrieveStudyService` + DICOM C-MOVE pull.
- Philips ISyntax-VNA (legacy Carestream Vue): REST + C-MOVE.
- Sectra VNA: XDS-I.b federated pull.
- Fujifilm Synapse VNA: DICOM C-MOVE.
- Agfa Impax VNA: REST + C-MOVE.
- Merge VNA (IBM, post-spinout): SOAP + C-MOVE.

### 14.2 Migration Validation

- Checksum verification (SOP Instance UID + pixel SHA-256).
- Sample-rate full-content verification (1% random sample).
- Audit-chain emission for every migrated study.
- Pre-migration baseline + post-migration verification report.

### 14.3 Cutover Strategy

- Phase 1: dual-write new studies to legacy + oyatie VNA.
- Phase 2: backfill historical studies via per-vendor adapter.
- Phase 3: cut reads to oyatie VNA.
- Phase 4: decommission legacy VNA after retention boundary.

---

## 15. Testing Architecture

### 15.1 Unit Tests

Per crate; standard `cargo test` lane.

### 15.2 DICOM Conformance Tests

Per release: DICOM PS 3.4 conformance test set covering every implemented SOP class.

### 15.3 IHE Connectathon

Annual participation in IHE Connectathon (NA + Europe) covering all listed profiles.

### 15.4 Vendor-Quirk Regression Tests

Per modality vendor: regression test corpus with anonymized real-world studies covering known private-tag patterns.

### 15.5 Performance Tests

`benchmarks/` lane: C-STORE throughput, DICOMweb latency, MPR render time, AI dispatch latency.

### 15.6 Compliance Pack Tests

Per pack: policy-decision tests (Cedar) + audit-completeness assertions.

### 15.7 Multi-Tenant Isolation Tests

Cross-tenant PHI leak prevention test corpus: deny-default + positive cases.

---

## 16. Trade-Offs & Decisions

| Decision | Trade-off | Rationale |
|----------|-----------|-----------|
| DICOMweb-first vs DIMSE-first | DIMSE compatibility burden | DICOMweb is modern, HTTP/3-native, easier scale |
| Server-side MPR rendering | Compute cost vs bandwidth | Server-side wins on bandwidth + viewer simplicity per Visage proof-point |
| Vendor-neutral AI vs single-vendor | Adapter complexity vs marketplace breadth | Marketplace breadth is the differentiator |
| Sovereign-cell mandatory for paid | Operational complexity | HIPAA / GDPR / KR-MD / EU-MDR compliance mandates |
| Tenant-class demo_trial / paid only | Loses free-tier breadth | Aligns with ADR-0330 tenant-class enum |
| FHIR + DICOM dual emission | Storage / authoring overhead | EMR integration + modern interoperability |
| Per-modality acquisition workers | Operational footprint | Modality-specific quirks demand isolation |
| HTTP/3 + QUIC default | Legacy client compatibility | Aligns with ADR-0253 default protocol |
| Whisper-medical fallback for voice | Build vs buy | Avoids Nuance / M*Modal sole-source lock-in |

---

## 17. Open Architectural Questions

1. **Pathology WSI separation:** when slide sizes routinely exceed 50GB, pathology may warrant its own µservice with bespoke pyramidal tiling architecture.
2. **Cardiology imaging separation:** if structural-heart / TAVR planning workloads grow, a dedicated `cardiology-imaging` µservice may be warranted.
3. **Teleradiology marketplace:** if the night-hawk / cross-organization read marketplace surface acquires substantial RFP / billing / compliance surface, it may split.
4. **Edge / mobile acquisition:** point-of-care ultrasound (POCUS) on tablet/phone may warrant a separate edge µservice.
5. **AI training data lake:** if training-data curation acquires substantial scope (data-rights, label workflow, augmentation), it may warrant a separate µservice.

---

## 18. Sequence Diagrams (Text Form)

### 18.1 C-STORE → Worklist

```
Modality -> imaging.dimse-api: C-STORE-REQ (Study/Series/Instance)
imaging.dimse-api -> imaging.kernel: parseDIMSE + Cedar.check(modality_AE -> storage)
imaging.kernel -> cloud-iam: evaluate(action: store, principal: modality_AE)
cloud-iam -> imaging.kernel: ALLOW
imaging.kernel -> cloud-storage: PUT pixel-blob (envelope-encrypted)
imaging.kernel -> cloud-data: INSERT instance row
imaging.kernel -> audit-chain: emit PHI store
imaging.kernel -> event-mesh: imaging.study.received
imaging.dimse-api -> Modality: C-STORE-RSP success
event-mesh -> imaging.acquisition-app: imaging.study.received
imaging.acquisition-app -> imaging.workflow: MPPS state transition
imaging.acquisition-app -> imaging.workflow: tech-QC -> FORWARDED_TO_WORKLIST
imaging.workflow -> cloud-data: UPDATE study.status
```

### 18.2 Radiologist Reads + AI Overlay

```
Radiologist -> imaging.worklist-api: GET worklist
imaging.worklist-api -> cloud-data: SELECT worklist items
imaging.worklist-api -> Radiologist: worklist (sorted by priority)
Radiologist -> imaging.dicomweb-api: GET /studies/{uid}
imaging.dicomweb-api -> cloud-storage: GET pixel blobs
imaging.dicomweb-api -> Radiologist: multipart/related instances
Radiologist -> imaging.ai-marketplace-app: enable AI overlay
imaging.ai-marketplace-app -> imaging.deidentification-worker: strip PHI
imaging.deidentification-worker -> ai-vendor-aidoc: POST de-identified pixels
ai-vendor-aidoc -> imaging.ai-marketplace-app: findings (bounding boxes)
imaging.ai-marketplace-app -> Radiologist: AI overlay
Radiologist -> imaging.report-app: dictate structured report
imaging.report-app -> voice-vendor-nuance: stream audio
voice-vendor-nuance -> imaging.report-app: partial transcripts
Radiologist -> imaging.report-app: sign report
imaging.report-app -> cloud-data: INSERT structured-report
imaging.report-app -> event-mesh: imaging.report.signed
event-mesh -> emr: imaging.report.signed
```

### 18.3 Critical Result Closed-Loop

```
imaging.report-app -> imaging.critical-result-app: critical-finding code in SR
imaging.critical-result-app -> workflow-engine: start escalation
workflow-engine -> comms-push: notify ordering clinician
workflow-engine -> wait for confirmation (timer: criticality-defined)
[timer expires no confirmation]
workflow-engine -> comms-sms: notify covering clinician
workflow-engine -> wait
[ordering clinician confirms]
workflow-engine -> imaging.critical-result-app: confirmed
imaging.critical-result-app -> audit-chain: closed-loop completed
```

---

## 19. Configuration Surface

Per-modality config (declarative YAML): AE Title, port, called-AE pairing, accepted SOP classes, transfer syntaxes, MWL filters, MPPS expectations, dose-emission flag, vendor-quirks profile.

Per-AI-vendor config: API endpoint, auth secret ref, enabled (modality, body-part, indication) triples, model versions, FDA/CE clearance metadata, drift-detection thresholds.

Per-hanging-protocol config: per-radiologist + per-(modality, body-part) DICOM HP IOD or Oyatie YAML.

Per-pack config: HIPAA / GDPR / KR-MD / EU-MDR / MQSA / GxP overlay parameters.

---

## 20. Versioning

Crate semver per workspace policy. DICOM Conformance Statement per release. FHIR profile versioning per US-Core, UK-Core, KR-Core, IPA. Cedar policy bundles versioned per pack release.

---

## 21. References

- ADR-0105 (13-layer enum)
- ADR-0131 (per-µservice flat layout)
- ADR-0132 (no-grouping policy — split authority)
- ADR-0145 (direct gRPC inter-µservice)
- ADR-0212 (substance-bar artifact count)
- ADR-0243 (Cedar universal gate)
- ADR-0244 (tenant-as-universal-scoping)
- ADR-0245 (substrate-vs-product layering)
- ADR-0248 (Amazon cellular shape)
- ADR-0251 (compliance pack primitive)
- ADR-0253 (HTTP/3 + QUIC default)
- ADR-0254 (K8s + Cloud Hypervisor + Kata)
- ADR-0255 (BYOK opt-in)
- ADR-0328 (substance-bar canonical sequence)
- ADR-0329 (tier-retirement)
- ADR-0330 (tenant-class)
- ADR-0331 (per-µservice adoption)
- ADR-0332 (healthcare domain decomposition)
- DICOM PS 3.1..3.20 (NEMA)
- IHE Radiology Technical Framework
- HL7 FHIR R5
- NEMA XR-29 + XR-25 (dose check)
- FDA 21 CFR 900 (MQSA)
- EURATOM 2013/59
