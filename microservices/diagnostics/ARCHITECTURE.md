---
doc_class: ARCHITECTURE
microservice: diagnostics
title: Diagnostics Architecture - Lab + Pathology
status: wave-15m-reconciled
date: 2026-05-21
related:
  - PRD.md
  - ../imaging/PRD.md
  - ../../docs/decisions/ADR-0332-healthcare-domain-decomposition.md
  - ../../docs/decisions/ADR-0132-no-suite-policy.md
---

# Diagnostics Architecture - Lab + Pathology

## 1. Scope

Diagnostics is a flat Oyatie microservice for lab and pathology diagnostic evidence. It implements the laboratory information system and anatomic pathology workflow surface: orders, specimens, results, critical-result escalation, reference ranges, reflex testing, sign-out, delivery, turn-around-time, and quality control.

Imaging is not part of this architecture. `microservices/imaging/` owns imaging orders, PACS/VNA, DICOM object custody, DICOMweb/DIMSE, radiologist worklists, read reports, dose monitoring, image AI, and FHIR `ImagingStudy`. This file retains imaging only as a cross-service dependency for lab/pathology result correlation.

## 2. Layer Shape

Diagnostics follows ADR-0131 flat layout and ADR-0132 single-concern discipline.

```text
api          -> REST/OpenAPI, AsyncAPI, proto/gRPC adapters
usecase      -> lab/pathology workflows and external envelope binding
domain       -> LabOrder, LabResult, PathologyCase, Specimen, CriticalResultTicket,
                ReferenceRangeSet, ReflexRule, TatClock, ResultAuthorization,
                ResultInterpretation, ResultDelivery, QualityControlRun
storage      -> tenant-scoped relational/event storage
policy       -> Cedar default-deny gates
outbox       -> domain event publication and downstream delivery retries
observability-> SLO metrics, audit events, and compliance evidence
```

There is no diagnostics DICOM sublayer, PACS adapter, VNA bucket, radiology worklist, or DICOM conformance surface.

## 3. Bounded Context Clusters

| Cluster | Contexts | Storage posture |
| --- | --- | --- |
| Laboratory | lab-order, lab-result, reference-range, reflex-test, quality-control | Tenant-scoped relational/event shards; terminology-indexed reads. |
| Pathology | pathology-case, specimen, result-authorization, result-interpretation | Tenant-scoped relational/event shards; sign-out evidence retained. |
| Operations | critical-result, turn-around-time, result-delivery | Outbox-driven event delivery and operational projections. |

## 4. Data Stores

- Relational command store: tenant/cell partitioned lab orders, pathology cases, specimens, and authorization state.
- Event store: immutable lab/pathology domain events with correlation IDs and causation IDs.
- Read models: FHIR R5 `Observation` and lab/pathology `DiagnosticReport` projections.
- Audit evidence store: critical-result acknowledgement, electronic signatures, QC exceptions, and policy decisions.

No DICOM object, image blob, PACS index, WADO cache, or radiology report store exists in diagnostics.

## 5. External Interfaces

| Interface | Purpose |
| --- | --- |
| REST/OpenAPI | Lab/pathology operational commands and FHIR read projections. |
| AsyncAPI | Lab/pathology domain events and image-correlation request events. |
| Proto/gRPC | Internal lab/pathology commands, read models, and workflow control. |
| Cedar | Default-deny authorization for lab/pathology actions. |
| OpenSLO | Service objectives for ingestion, report issuance, critical notification, pathology sign-out, reflex/reference-range, TAT, and policy decisions. |

## 6. Cross-Service Dependencies

| Service | Direction | Contract |
| --- | --- | --- |
| emr | bidirectional | Lab/pathology order intake and final report delivery. |
| healthcare-integration | inbound | HL7v2/FHIR broker payloads for external lab/pathology connectivity. |
| imaging | bidirectional | Diagnostics emits image-correlation requests; imaging returns report/study references only. |
| cloud-billing | outbound | Lab/pathology charge capture and billing facts. |
| clinical-decision-support | outbound | Lab/pathology facts for recommendation evaluation. |
| care-management | outbound | Critical result follow-up and care-plan trigger facts. |
| ontology | outbound | Lab/pathology observations and diagnostic terminology projections. |

Diagnostics never accepts raw image objects or radiology report ownership from imaging.

## 7. Security and Compliance

Cedar policies enforce:

- lab-result release by authorized lab roles;
- pathology sign-out by authorized pathologists;
- ordering-provider minimum-necessary read access;
- HIPAA default deny for cross-tenant and purpose-of-use violations.

Compliance packs are HIPAA, CLIA, CAP, ISO 15189, GxP, KR IVD, and EU IVDR. Imaging-specific ACR, DICOM conformance, IHE Radiology, mammography, and PACS/VNA packs are owned by `microservices/imaging/`.

## 8. SLO Inventory

Diagnostics owns these SLOs:

- `hl7v2-ingest-success.openslo.yaml`
- `diagnostic-report-issuance-latency.openslo.yaml`
- `critical-result-notify-latency.openslo.yaml`
- `pathology-sign-out-to-delivery.openslo.yaml`
- `reflex-evaluation-latency.openslo.yaml`
- `reference-range-lookup-latency.openslo.yaml`
- `tat-dashboard-refresh.openslo.yaml`
- `policy-decision-latency.openslo.yaml`

Imaging SLOs are removed from diagnostics and live under the imaging microservice.

## 9. Failure Modes

| Failure | Response |
| --- | --- |
| HL7/FHIR intake backlog | Back-pressure healthcare-integration, preserve raw envelope, retry idempotently. |
| Reference-range ambiguity | Hold release, require lab director review, emit audit event. |
| Reflex-rule conflict | Suppress auto-order, require manual adjudication, preserve rule trace. |
| Critical-result acknowledgement timeout | Escalate by tenant policy and retain closed-loop evidence. |
| Pathology sign-out conflict | Block release until signature chain is corrected. |
| Imaging correlation unavailable | Release lab/pathology result when clinically allowed and attach pending correlation status; imaging remains owner of image report. |

## 10. Migration Boundary

The earlier diagnostics bundle included imaging contexts concurrently with the dedicated imaging service. Wave 15M-RECONCILE removes that bundle. Migration rules:

- `ImagingOrder`, `ImagingResult`, `DICOMStudy`, PACS, VNA, DICOMweb, DIMSE, FHIR `ImagingStudy`, radiology structured reporting, and radiologist workflow belong to `microservices/imaging/`.
- Diagnostics retains lab/pathology orders, specimens, results, cases, reference ranges, reflex tests, TAT, quality control, and critical-result workflows.
- Healthcare-integration remains the external FHIR/HL7v2/DICOM broker substrate and does not own domain workflows.

## 11. Completion Criteria

The architecture is complete when diagnostics files contain no owned imaging/DICOM/PACS/RIS surface, ADR-0332 names imaging as a separate healthcare microservice, and validation confirms remaining imaging mentions are supersession or handoff references only.
