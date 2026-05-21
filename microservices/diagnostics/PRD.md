---
doc_class: PRD
prd_id: PRD-diagnostics
microservice: diagnostics
title: Diagnostics - Lab + Pathology
status: wave-15m-reconciled
date: 2026-05-21
owner_team: axis-diagnostics + council-product
reconciliation_wave: 15m-reconcile
canonical_sources:
  - ../../docs/decisions/ADR-0332-healthcare-domain-decomposition.md
  - ../../docs/decisions/ADR-0132-no-suite-policy.md
  - ../imaging/PRD.md
  - ../imaging/REMEDIATION-NOTES-2026-05-21.md
top_3_counterparts:
  - Sunquest / Clinisys LIS
  - Oracle Health PathNet / Cerner Diagnostics Labs
  - Epic Beaker + Beaker AP
secondary_counterparts:
  - Roche Navify
  - LabCorp Diamond
  - Quest Lab Connect
  - Clinisys AP
superseded_scope:
  imaging: ../imaging/PRD.md
local_adrs:
  - decisions/ADR-MS-001-lab-vs-imaging-vs-pathology-split.md
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0251
  - ADR-0328
  - ADR-0332
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
---

# PRD-diagnostics: Diagnostics - Lab + Pathology

## A. Purpose

Diagnostics owns laboratory and pathology diagnostic evidence production for Oyatie. It receives lab and pathology orders, manages specimens, produces verified results, handles critical-result escalation, supports reflex testing and reference-range lookup, and delivers final lab/pathology reports to EMR, billing, care-management, and quality consumers.

The dedicated imaging microservice is the sole authority for imaging. `microservices/imaging/PRD.md` supersedes all diagnostic imaging portions formerly bundled here, including `ImagingOrder`, `ImagingResult`, `DICOMStudy`, PACS, VNA, radiologist workflow, DICOMweb, DIMSE, FHIR `ImagingStudy`, imaging `DiagnosticReport`, image annotation, 3D reconstruction, dose tracking, IHE Radiology profiles, and radiology-specific Cedar/SLO/capability artifacts.

## B. Product Boundary

### In Scope

| Capability | Diagnostics responsibility |
| --- | --- |
| Lab ordering | Accept, validate, route, and track lab orders from EMR and integration substrates. |
| Lab resulting | Capture, normalize, verify, amend, cancel, and release lab results. |
| Pathology case management | Register cases, bind specimens, manage gross/microscopic/final diagnosis text, and support sign-out. |
| Specimen chain of custody | Track specimen collection, receipt, accessioning, aliquots, rejection, and custody evidence. |
| Critical results | Detect critical values, escalate, document acknowledgement, and close the loop. |
| Reference ranges | Resolve tenant, patient, method, age, sex, and unit-aware ranges. |
| Reflex testing | Evaluate deterministic reflex rules and generate follow-up lab orders. |
| Result authorization | Enforce lab director/pathologist authorization and electronic signature gates. |
| Result interpretation | Store structured lab and pathology interpretation without owning images. |
| Result delivery | Publish final reports to EMR, cloud-billing, care-management, and analytics consumers. |
| Turn-around-time | Track lab/pathology TAT clocks, breach alerts, and operational dashboards. |
| Quality control | Record QC runs, calibration, method lots, analyzer exceptions, and corrective actions. |

### Out of Scope

Diagnostics does not own imaging orders, imaging acquisition, PACS/VNA storage, DICOM object custody, DICOMweb endpoints, radiologist worklists, radiology structured reporting, image viewing, image AI, dose monitoring, or FHIR `ImagingStudy` projections. Those responsibilities live in `microservices/imaging/`.

Diagnostics may publish or consume cross-service correlation events when a lab/pathology result needs imaging context, but the imaging service remains the system of record for image studies and read reports.

## C. Bounded Contexts

| Context | Aggregate root | Notes |
| --- | --- | --- |
| `lab-order` | `LabOrder` | ServiceRequest intake, order validation, cancellation, and status. |
| `lab-result` | `LabResult` | Observation generation, result correction, abnormal/critical flagging. |
| `pathology-case` | `PathologyCase` | AP/CP case lifecycle, sign-out, amendments, addenda. |
| `critical-result` | `CriticalResultTicket` | Closed-loop notification and acknowledgement evidence. |
| `reference-range` | `ReferenceRangeSet` | Method, tenant, unit, demographic, and effective-date selection. |
| `reflex-test` | `ReflexRule` | Deterministic reflex evaluation and downstream lab-order request. |
| `turn-around-time` | `TatClock` | Lab/pathology TAT timers, breach projections, dashboard materialization. |
| `specimen` | `Specimen` | Collection, accessioning, chain-of-custody, aliquots, rejection. |
| `result-authorization` | `ResultAuthorization` | Reviewer, signer, delegation, and electronic-signature evidence. |
| `result-interpretation` | `ResultInterpretation` | Narrative and structured lab/pathology interpretation. |
| `result-delivery` | `ResultDelivery` | EMR, billing, care-management, analytics, and patient-facing delivery events. |
| `quality-control` | `QualityControlRun` | Analyzer QC, calibrator lots, rule failures, and corrective actions. |

Removed contexts: `ImagingOrder`, `ImagingResult`, `DICOMStudy`, and imaging-specific variants are superseded by `microservices/imaging/`.

## D. Functional Requirements

1. Accept lab/pathology orders from EMR and healthcare-integration with tenant, patient, ordering-provider, specimen, priority, and code-system validation.
2. Maintain specimen custody from collection through final result delivery, including rejection and recollection flows.
3. Normalize lab results into internal domain events and FHIR R5 `Observation` / lab-pathology `DiagnosticReport` projections.
4. Support pathology case accessioning, specimen binding, text/numeric result entry, case sign-out, amendments, and addenda.
5. Evaluate reference ranges and abnormal flags with tenant-specific method, unit, demographic, and effective-date rules.
6. Evaluate reflex-test rules deterministically and emit follow-up lab-order requests.
7. Detect critical results, trigger closed-loop notifications, enforce acknowledgement deadlines, and retain evidence.
8. Enforce Cedar default-deny authorization for lab-result release, pathology sign-out, ordering-provider read access, and HIPAA minimum-necessary access.
9. Publish final lab/pathology results to EMR, care-management, analytics, ontology, billing, and patient-context consumers.
10. Publish image-correlation requests to imaging when a lab/pathology result needs image-study context; consume only imaging report references returned by imaging.

## E. Cross-Service Handoffs

| Direction | Purpose | Boundary |
| --- | --- | --- |
| `emr -> diagnostics` | Lab/pathology order intake | Diagnostics owns order validation and result lifecycle. |
| `diagnostics -> emr` | Final lab/pathology report delivery | EMR owns chart presentation and longitudinal record. |
| `diagnostics -> imaging` | Image-correlation request for a lab/pathology result | Imaging owns study lookup, image report authority, and image artifacts. |
| `imaging -> diagnostics` | Imaging report reference for correlation | Diagnostics stores references only, not image data or radiology reports. |
| `diagnostics -> cloud-billing` | Lab/pathology charge capture | Billing owns invoice, payer, tax, and remittance. |
| `diagnostics -> clinical-decision-support` | Lab/pathology facts for rule evaluation | CDS owns recommendations and alerts. |
| `diagnostics -> analytics` | De-identified lab/pathology metrics | Analytics owns aggregate reporting. |
| `healthcare-integration -> diagnostics` | HL7v2/FHIR broker intake | Broker owns external protocol translation only. |

## F. Contract Surface

Diagnostics exposes:

- REST/OpenAPI for lab/pathology operational commands and FHIR R5 `Observation` / lab-pathology `DiagnosticReport` reads.
- AsyncAPI for lab/pathology domain events and image-correlation request events.
- Proto3/gRPC for internal lab/pathology commands and read models.
- Cedar policies for lab/pathology authorization gates.
- OpenSLO files for lab/pathology latency, notification, ingestion, reference-range, reflex, dashboard, and policy-decision objectives.

Diagnostics does not expose DICOMweb, DIMSE, PACS/VNA, FHIR `ImagingStudy`, or radiology report APIs.

## G. Non-Functional Requirements

### G.1 DR Posture (ADR-0343)

- Target: RTO 3600s and RPO 300s for lab orders, specimen custody, critical-result tickets, and final lab/pathology report release, matching `manifest.json` `dr.rto_p99_seconds=3600` and `dr.rpo_p99_seconds=300`.
- Compliance floors: HIPAA-2024 floors at 3600s/300s with multi-region required; SOC2-T2 floors at 14400s/900s; ISO27001-2022 floors at 14400s/3600s; KR-PIPA sensitive-PI floor at 7200s/600s. The effective target remains 3600s/300s with continuous cross-region replication.
- failover_runbook: `microservices/diagnostics/runbooks/diagnostics-result-failover.md`.
- multi_region_active_active: true per manifest, implemented as active-passive cross-region continuous recovery for diagnostics result paths; read-only dashboards may degrade.
- Why: clinicians and lab directors keep receiving verified critical values and custody evidence even if one cell or region is lost.

### G.2 Capacity Model (ADR-0340)

- Per-tenant baseline: 0.2 vCPU, 512 MiB RAM, 8 GB storage, 4 Postgres connections, 2 Valkey connections, and 8 outbound HTTP connections, matching `manifest.json` `capacity_model`.
- Scaling dimension: `per_message` for lab orders, specimen events, result messages, and critical-result notifications.
- Cell placement class: Tier-3. Diagnostics is a first-party healthcare application aligned to `pod_runtime_tier=2`.
- Autoscaling boundaries: min 2 pods per tenant cell, max 24 pods per tenant cell before analyzer-ingest and pathology-signout partition review.
- Why: the model fits message-driven clinical lab traffic without over-reserving EMR-grade chart substrate capacity.

### G.3 Sustainability + Cost Attribution (ADR-0344)

- Emission envelope: each lab/pathology audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside tenant, capability, provider, cell, and compliance-pack dimensions.
- Provider-routing affected by carbon: no for STAT labs, critical values, pathology sign-out, or HIPAA emergency-mode traffic; yes for non-urgent QC dashboards, reference-range rebuilds, and de-identified analytics publication.
- Tenant cost transparency: `finops-portal` exposes diagnostics cost by lab order, pathology case, analyzer integration, critical-result escalation, and de-identified metrics stream.
- Why: clinical labs need CSRD, SB-253, and SEC climate-disclosure evidence without delaying high-acuity results.

### G.4 API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet across `Oyatie-Api-Version`, `/v/YYYY-MM-DD/...` URL prefixes, and proto3 `api_version_date` fields.
- SDK semver model: major.minor.patch for lab/pathology REST, FHIR Observation, DiagnosticReport, and event clients.
- Support window: last 3 public versions supported for at least 180 days.
- Per-tenant pinning: yes for tenant lab integrations, reference-range consumers, and report-delivery endpoints.
- Internal-mesh exemption: yes; internal gRPC handoffs to EMR, pharmacy, clinical-decision-support, and healthcare-integration preserve ADR-0145.

## H. Compliance and Safety

Diagnostics must satisfy HIPAA, CLIA, CAP, ISO 15189, GxP evidence retention, KR IVD, and EU IVDR pack overlays where applicable. Imaging-specific ACR, DICOM conformance, IHE Radiology, mammography, and PACS/VNA evidence packs are owned by `microservices/imaging/`.

## I. Acceptance Criteria

- Diagnostics PRD, architecture, contracts, Cedar policies, SLOs, manifests, and local plans contain no diagnostics-owned imaging contexts.
- The only remaining imaging mentions in diagnostics are explicit supersession notes or cross-service handoff references.
- ADR-0332 lists imaging as the eighth new healthcare microservice and updates the healthcare handoff matrix.
- Diagnostics counterpart parity lists lab and pathology vendors only.
- Reconciliation evidence is recorded in `REMEDIATION-NOTES-2026-05-21.md`.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
