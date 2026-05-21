---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-imaging
microservice: imaging
title: Imaging Microservice Product Requirements
status: wave-15m-g-authored-2026-05-21
date: 2026-05-21
owner_team: axis-imaging + council-clinical + council-product
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0248
  - ADR-0251
  - ADR-0253
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

# Imaging Microservice — Product Requirements Document (PRD)

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

<!--
COMPLETION REPORT (Wave 15M-G — 2026-05-21):
- Owner: SOLE-OWNER imaging-µservice authoring agent (Wave 15M-G).
- Authority: split from diagnostics per ADR-0132 single-concern doctrine + user directive 2026-05-21.
- Scope: full medical imaging platform (DICOM substrate, PACS, VNA, radiologist workflow, AI image analysis, enterprise imaging beyond radiology).
- Top-3 anchors: GE Centricity / Philips IntelliSpace / Sectra PACS+VNA.
- Substance bar: ADR-0212 100+ artifact floor; this PRD ≥800 lines, ARCHITECTURE ≥600 lines, README ≥300 lines.
- Supersedes: imaging portions of the concurrently-authored diagnostics µservice (bundled lab+imaging+pathology). Reconciliation in Wave 15M follow-up.
- No commits. No scripting. No stamping. Writes restricted to microservices/imaging/*.
- Artifacts authored this wave: manifest.json, supported-oses.json, PRD.md (this doc), ARCHITECTURE.md, README.md, 4 contracts (openapi.yaml, asyncapi.yaml, proto/imaging.proto, proto/imaging-events.proto), 12 OpenSLOs, 8 Cedar policies, 6 IaC contexts, 4 ADR-MS decisions, 15 IPs, competitor-parity-matrix.md, REMEDIATION-NOTES-2026-05-21.md.
-->

`microservice: imaging`
`status: wave-15m-g-authored-2026-05-21`
`tier: product / b2b-leader-operational-concern / medical-device-software`
`tenant_class_eligibility: ["demo_trial", "paid"]`
`split_authority: ADR-0132 (no-suite policy) + user directive 2026-05-21`
`top_3_anchors: GE Centricity Universal Viewer; Philips IntelliSpace PACS; Sectra PACS + VNA`

---

## 0. Executive Summary

The Imaging microservice is Oyatie's hyperscaler-grade enterprise imaging platform. It provides a hospital, imaging-center, teleradiology service, and integrated-delivery-network with a single, unified surface for:

1. A standards-conformant DICOM substrate (DIMSE, DICOMweb, IHE profiles) capable of sustaining the existing 10,250-instance/min C-STORE throughput already claimed in `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md`.
2. A Picture Archiving and Communication System (PACS) covering acquisition through reading-room consumption.
3. A Vendor-Neutral Archive (VNA) with cross-VNA federation and migration paths from legacy GE / Philips / Sectra / Fujifilm / Agfa / Merge installs.
4. The full radiologist workflow surface: dynamic worklist, hanging protocols, structured reporting, voice recognition, critical results closed-loop, peer review, and RadPeer integration.
5. A vendor-neutral AI image analysis marketplace fan-out covering CADe (computer-aided detection), CADx (characterization), triage workflows, and quantification (LV EF, SUV, BIRADS audit).
6. 3D reconstruction, multi-planar reformation (MPR), maximum intensity projection (MIP), image annotation, prior comparison, and radiation dose tracking.
7. Modality interoperability via DICOM C-STORE/C-FIND/C-MOVE/C-GET, Modality Worklist (MWL), MPPS, and the full IHE Radiology profile portfolio (XDS-I.b, XCA-I, IRWF.b, SWF.b, REM, AIW-I).
8. Enterprise imaging beyond radiology — cardiology (echo, cath, EP), ophthalmology (OCT, fundus, visual field), dermatology clinical photography, pathology whole-slide imaging (with the caveat that pathology may split further later), dental, and surgical intraoperative video.
9. Radiology Information System (RIS) integration, order-entry handoff from `emr`, mammography tracking/recall workflows, BIRADS audit, nuclear medicine PET quantification, and interventional radiology cath-lab planning.
10. Patient portal image sharing with consent-graph enforcement.

The product is designed to displace GE Centricity Universal Viewer + PACS-IW + Enterprise Archive, Philips IntelliSpace PACS + Enterprise Imaging (Carestream Vue lineage), and Sectra PACS + VNA — not by feature-checkbox parity, but by delivering a unified DICOM + FHIR + IHE substrate that the legacy vendors approximate with bolted-together acquired products. Secondary anchors (Agfa Enterprise Imaging, Visage 7, Merge PACS, Fujifilm Synapse, Siemens syngo.via, Change Healthcare Stratus) inform the secondary capability matrix.

This µservice owns ONE concern (medical imaging end-to-end) per ADR-0132's no-suite policy. The concurrently-authored `diagnostics` µservice has a bundled scope (lab + imaging + pathology) that violates ADR-0132; **this imaging µservice's authority SUPERSEDES the imaging portions of diagnostics**. Reconciliation is queued for Wave 15M follow-up.

---

## 1. Vision

> Make every imaging study acquirable, archivable, retrievable, readable, analyzable, comparable, billable, and shareable through one Cedar-gated, tenant-scoped, compliance-pack-overlaid surface that beats GE, Philips, and Sectra on (a) substrate openness, (b) AI marketplace neutrality, (c) DICOMweb-first architecture, and (d) cell-aware sovereign deployment.

### 1.1 North-Star Outcomes

1. **Time-to-first-image (technologist acquires → reader sees diagnostic quality):** p95 < 30 seconds from C-STORE completion to worklist entry visible.
2. **Reading-room productivity:** ≥15% increase in studies-read-per-shift vs. GE Centricity / Sectra benchmarks (Sectra averages ~120 CT studies / 8h shift for high-volume reading rooms; target ≥138).
3. **AI-augmented triage:** Stroke LVO detection NPV ≥98%, time-to-radiologist-notification p95 < 90 seconds from acquisition.
4. **VNA durability:** 13 nines on PHI pixel data (erasure-coded across ≥3 cells per pack policy).
5. **Sovereign-cell isolation:** zero cross-cell PHI egress under HIPAA / GDPR / KR-Medical-Devices / EU-MDR packs, enforced by Cedar + audit-chain.
6. **AI marketplace neutrality:** ≥15 third-party AI vendors (Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys, Caption Health, RapidAI, Subtle Medical, Imagia, Behold.ai, ScreenPoint) integrated via a single CADe/CADx adapter layer.

### 1.2 Anti-Vision (What This µservice Refuses)

- We do not own clinical decision support beyond the imaging-modality scope (CDS for non-imaging belongs to `emr` / `diagnostics`).
- We do not own non-imaging lab orders or specimen tracking (belongs to `diagnostics` lab portion).
- We do not own clinical genomics or molecular pathology beyond the visual whole-slide imaging substrate (belongs to a future `pathology` µservice when split).
- We do not own provider directory / credentialing (belongs to `identity` + `cloud-iam`).
- We do not own claims adjudication or revenue-cycle management (belongs to `cloud-billing` + a future RCM µservice).

---

## 2. Personas

### 2.1 Radiologist (Reader)

- Reads 80–200 studies per 8-hour shift depending on modality.
- Needs: dynamic worklist with priority (stat / routine / preliminary / addendum), hanging protocols per modality+body-part, structured reporting with BI-RADS/LI-RADS/PI-RADS/TI-RADS templates, voice recognition (Nuance Dragon Medical / M*Modal), critical-results closed-loop, peer review queue, prior auto-fetch, side-by-side compare, MPR/MIP, AI overlay (toggle on/off per study).
- Pain in legacy systems: hanging protocol mis-application after viewer upgrade, voice dictation drop-outs, critical-result phone-tag, slow prior fetch, AI overlay false positives drowning out workflow.

### 2.2 Radiologic Technologist (Tech)

- Operates the modality, acquires studies, performs MPPS (Modality Performed Procedure Step), routes to PACS, manages worklist.
- Needs: MWL pull, MPPS push, on-modality QC, dose tracking emission, exam barcode integration with patient identity, structured-data acquisition (CT dose report, MRI sequence metadata).
- Pain in legacy systems: MWL stale or missing, MPPS not closing properly, exam routing errors, dose data not emitting cleanly.

### 2.3 Referring / Ordering Physician

- Orders imaging via `emr` → handoff to `imaging`; consumes the signed report; views images via embedded zero-footprint viewer.
- Needs: clear order status, embedded image viewer with measurement, structured report parsed back into EMR, critical-results inbox.
- Pain in legacy systems: image quality degraded in zero-footprint viewers, structured report not parseable back into EMR discrete fields.

### 2.4 IT / PACS Administrator

- Manages PACS/VNA installation, modality on-boarding, AI vendor configuration, audit, DR, capacity, dose-monitoring substrate.
- Needs: declarative modality config, audit completeness ≥100%, capacity-based admission control, AI vendor health dashboard, dose-monitoring NEMA-DICOM dose-report substrate.
- Pain in legacy systems: opaque admin surfaces, per-vendor config UIs, no declarative IaC for PACS.

### 2.5 Patient

- Views their own studies through a patient portal; downloads original DICOM for second opinion; shares with referring clinician.
- Needs: consent-gated access, plain-language layperson summary, secure share link.
- Pain in legacy systems: image-CD downloads in 2026, no DICOMweb patient access, no FHIR ImagingStudy export.

### 2.6 Referring Clinician (External, Non-Tenant)

- Views shared studies via cross-enterprise share link (XCA-I or DICOMweb token).
- Needs: zero-footprint viewer, single-link access, time-boxed token.

### 2.7 Peer Reviewer (Internal QA)

- Performs RadPeer-style double-read on a 5% sample plus targeted high-risk subsets.
- Needs: blinded view of the primary read, structured discordance reporting, RadPeer score aggregation, no influence on the primary read until peer-review submit.

### 2.8 Dose Compliance Officer

- Tracks aggregate radiation dose across CT, fluoroscopy, interventional radiology, mammography per ACR / Image Wisely / NEMA XR-29 substrate; reports to regulators (US: CMS QPP MIPS, EU: EURATOM 2013/59).
- Needs: aggregate dose dashboards, per-protocol dose deviation alerts, regulatory export.

### 2.9 Mammography Coordinator

- Tracks MQSA mammography quality, screening recall workflow, BI-RADS audit, MQSA-mandated retention.
- Needs: screening recall queue, BI-RADS audit reports, MQSA-conformant retention, FDA-21-CFR-Part-11 audit.

### 2.10 AI Operations Engineer

- Manages the AI vendor marketplace integration: which vendors are enabled per tenant, per modality, per body-part, per indication; tracks AI vendor performance, drift, and CE/FDA-cleared use.
- Needs: per-vendor health, FDA/CE clearance metadata, drift dashboards, vendor model versioning, override workflow when AI vendor degrades.

---

## 3. Market Context

### 3.1 Top-3 Anchors

**GE Healthcare Centricity Universal Viewer + Centricity PACS-IW + Centricity Enterprise Archive**

GE's legacy dominance comes from deep modality integration (GE manufactures CT, MRI, US, mammography, NM, PET hardware) and an installed-base lock-in. The Universal Viewer is a thick-client / zero-footprint hybrid. The Enterprise Archive is GE's VNA offering. Weaknesses: aging viewer architecture, fragmented module integration (separately licensed cardiology, mammography, oncology modules), AI marketplace is GE-centric (Edison) rather than vendor-neutral, no first-class DICOMweb, sovereign-cell deployment is bespoke per customer.

**Philips IntelliSpace PACS + Enterprise Imaging (Carestream Vue PACS lineage)**

Philips acquired Carestream Health's Healthcare IT business (~2019) to fold Vue PACS into IntelliSpace. The product is strong in cardiology (Philips also owns echo/cath modality hardware). Enterprise Imaging extends scope to non-radiology images (clinical photography, dermatology, ophthalmology). Weaknesses: post-acquisition product fragmentation, Vue and IntelliSpace coexisting, dependency on Philips IntelliSpace Portal for advanced 3D/MPR (separate license), AI ecosystem (IntelliSpace AI Workflow Suite) is improving but not yet vendor-neutral at the Stripe-of-AI bar.

**Sectra PACS + Sectra Vendor Neutral Archive**

Sectra (Swedish) leads in reading-room workflow ergonomics, particularly for high-volume CT and breast imaging. The VNA is a separate, well-engineered product. Sectra wins repeat customer satisfaction (KLAS) for radiology. Weaknesses: smaller global footprint than GE/Philips, narrower enterprise-imaging coverage outside radiology, smaller AI ecosystem (Sectra Amplifier marketplace is growing but limited).

### 3.2 Secondary Anchors

- **Agfa Enterprise Imaging:** unified viewer for radiology + cardiology + clinical photography; strong in Europe.
- **Visage 7:** server-side rendering streaming viewer; speed leader for very large studies (>4GB CT).
- **Merge PACS (IBM):** IBM Watson Health lineage; struggling post-Francisco Partners spin-out; opportunity for migration.
- **Fujifilm Synapse:** strong in Asia-Pacific; integrated with Fujifilm modalities; Synapse 3D / Synapse VNA.
- **Siemens syngo.via:** thick-client advanced-visualization suite; strong cardiac and oncology read protocols; tightly coupled to Siemens modalities.
- **Change Healthcare Stratus (now Optum):** PACS-as-a-Service; cloud-native pioneer; opportunity for customer migration post-Optum acquisition uncertainty.

### 3.3 Differentiation Theses

1. **DICOMweb-first**: PACS substrate is DICOMweb-native (WADO-RS/QIDO-RS/STOW-RS); DIMSE is bridged but not the primary substrate. This is the modern architecture Visage proves works at scale.
2. **AI marketplace vendor-neutral**: AI vendors integrate via a single CADe/CADx adapter. No "Edison-only" or "IntelliSpace-AI-Workflow-only" lock-in.
3. **Cell-aware sovereign deployment**: HIPAA, GDPR, KR-Medical-Devices, EU-MDR packs map to sovereign cells with Cedar-enforced isolation. No legacy vendor offers this primitive.
4. **Tenant-scoped FHIR + DICOM unification**: ImagingStudy / ImagingSelection / DiagnosticReport[imaging] / DocumentReference[CDA] are first-class alongside DICOM. Order-to-report data flows in FHIR; pixel data flows in DICOM.
5. **Enterprise-imaging native**: cardiology, ophthalmology, dermatology, dental, surgical video, and pathology WSI are all in scope from day one (not a bolt-on like Agfa or Philips).
6. **Substance-bar artifact discipline**: 100+ artifact substance bar per ADR-0212 — every claim has an ADR + PRD section + IP + Cedar policy + SLO + Cedar test + parity-matrix row.

---

## 4. Bounded Contexts (24)

Each bounded context maps to a slice of the µservice. Layer mapping follows ADR-0105 13-layer enum.

### 4.1 DICOMSubstrate

Provides the core DICOM PS3 conformance: DIMSE associations (C-STORE/C-FIND/C-MOVE/C-GET/N-CREATE/N-SET/N-ACTION/N-EVENT-REPORT/N-GET), DICOMweb (WADO-RS/QIDO-RS/STOW-RS/UPS-RS), Modality Worklist (MWL), Modality Performed Procedure Step (MPPS), Storage Commitment, Print Management (mostly retired but retained for veterinary + dental).

### 4.2 PACS

The Picture Archiving and Communication System view of the substrate: study tables, series tables, instance tables, indexed by Study Instance UID / Series Instance UID / SOP Instance UID; query/retrieve surfaces optimized for radiologist workflow.

### 4.3 VNA

Vendor-Neutral Archive substrate: cross-vendor study storage, federation with external VNAs (legacy GE EA / Philips ISyntax-VNA / Sectra VNA via XDS-I.b or proprietary import), deduplication, deep-archive tiering.

### 4.4 ImageAcquisition

Per-modality acquisition workers covering all modalities listed in `manifest.json#modalities_supported`. Each modality has its own acquisition state machine: schedule → patient-arrive → MWL pull → acquire → MPPS in-progress → acquire-complete → MPPS-complete → C-STORE → tech-QC → forward-to-radiologist-worklist.

### 4.5 RadiologistWorklist

Dynamic worklist for radiologists. Sortable, filterable, prioritizable. Implements: stat queue, body-part queue, modality queue, sub-specialty routing, on-call rotation, teleradiology night-hawks, escalation timers.

### 4.6 HangingProtocol

DICOM Hanging Protocols + Oyatie-extension layer. Per-modality + per-body-part + per-radiologist preference. Applies side-by-side comparison with priors, viewport layout, default window/level, default series order, default annotations visibility.

### 4.7 StructuredReport

DICOM Structured Reports (SR) + FHIR DiagnosticReport[imaging] dual emission. Templates: BI-RADS (mammography), LI-RADS (liver), PI-RADS (prostate), TI-RADS (thyroid), Lung-RADS, O-RADS (ovary), CAD-RADS (coronary), Bone-RADS, NI-RADS (head-neck post-treatment), and the broader ACR template library. RadLex terminology + SNOMED-CT + RSNA RadElement integration.

### 4.8 VoiceRecognition

Voice-recognition integration with Nuance Dragon Medical One, M*Modal Catalyst, and a forward-compatible in-house Whisper-medical fine-tune. Streaming partial transcripts, structured-field auto-fill, voice commands (next-image, prior-compare, AI-toggle).

### 4.9 CriticalResults

Closed-loop critical-results communication. Triggered by structured-report tagging (e.g., "critical: pulmonary embolism present"). Cascades through escalation: ordering clinician → covering clinician → charge nurse → on-call attending → patient safety officer. Confirmation required at each step within an escalation timer. Audit-chain entry for every step.

### 4.10 PeerReview

RadPeer-style internal peer review. 5% random sample + targeted high-risk subsets (post-procedure, transfer-in, post-AI-disagreement). Blinded view of primary read. Discordance scoring per ACR RadPeer 3-point scale. Aggregate score per radiologist. ACR submission integration.

### 4.11 AIImageAnalysis

Vendor-neutral CADe/CADx integration. Vendor adapter layer abstracts over each AI vendor's API. Vendors: Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys, Caption Health, RapidAI, Subtle Medical, Imagia, Behold.ai, ScreenPoint, and others. Indications: lesion detection/characterization, lung nodule, breast lesion (mammography + DBT), brain hemorrhage, stroke LVO triage, pulmonary embolism, bone fracture, cardiac quantification (LV EF, LA volume), spine fracture, coronary calcium scoring, opportunistic body composition.

### 4.12 3DReconstruction

Volume-rendering, MPR (multi-planar reformation), MIP (maximum intensity projection), curved-MPR for vascular trees, surface-rendering. Server-side rendering (Visage-class architecture) with client-side WebGL fallback.

### 4.13 ImageAnnotation

DICOM Presentation State (PR), DICOM SR-TID-1500 measurement, Oyatie-native overlay layer for ROI statistics (mean, SD, min, max, volume), linear / angle / area measurements.

### 4.14 PriorComparison

Auto-fetch of prior studies. Side-by-side viewport. Auto-registration where modality+body-part match. Cross-VNA federated lookup if priors live in legacy VNA.

### 4.15 DoseTracking

NEMA DICOM Dose Structured Report (RDSR) consumption. Per-protocol dose deviation alerts. Aggregate dose dashboards per patient / per modality / per protocol / per technologist. Regulatory export (US: CMS QPP MIPS measure; EU: EURATOM 2013/59 reporting).

### 4.16 ModalityInterop

Per-modality conformance statement + per-vendor quirk handling (GE Advantage Workstation tags / Siemens private tags / Philips private tags / Toshiba/Canon private tags). MWL/MPPS protocol bridges.

### 4.17 IHEProfiles

Implementation of IHE Radiology profiles: XDS-I.b (cross-enterprise imaging document share), XCA-I (cross-community access for imaging), IRWF.b (imaging workflow), SWF.b (scheduled workflow), REM (radiation exposure monitoring), AIW-I (imaging-object change management), PIR, PDQ, PIX, ATNA, Consistent Time, EUA, PWP.

### 4.18 DICOMweb

WADO-RS (retrieve), QIDO-RS (query), STOW-RS (store), UPS-RS (worklist/workflow). REST-first DICOM substrate. Token-gated, tenant-scoped, Cedar-enforced.

### 4.19 EnterpriseImaging

Non-radiology imaging: cardiology (echo, cath, EP), ophthalmology (OCT, fundus, visual field), dermatology clinical photography, pathology whole-slide imaging, dental panoramic/CBCT, surgical intraoperative video.

### 4.20 MammographyTracking

Mammography-specific workflow: screening recall, diagnostic follow-up, BI-RADS audit (positive predictive value, cancer detection rate, recall rate, sensitivity, specificity), MQSA-conformant retention (US: 5 years minimum, 10 if abnormal), letter generation.

### 4.21 NuclearMedicine

NM/PET-specific workflow: SUV (standardized uptake value), Tumor/Background ratio, Deauville score (lymphoma), PERCIST response criteria, quantitative dynamic PET. Hybrid PET/CT and PET/MRI co-registration.

### 4.22 InterventionalRadiology

IR/cath-lab workflow: pre-procedure imaging review, procedure documentation, hybrid OR planning, fluoroscopy dose tracking.

### 4.23 RISIntegration

Radiology Information System integration: order receipt from `emr`, scheduling, billing handoff to `cloud-billing`, report distribution back to `emr`.

### 4.24 PatientPortalSharing

Patient-portal access: consent-graph-gated DICOMweb token issuance, plain-language layperson summary generation, secure share link to external referring clinicians, FHIR ImagingStudy export.

---

## 5. Functional Requirements

### 5.1 DICOM Substrate (FR-DICOM-*)

| ID | Requirement |
|----|-------------|
| FR-DICOM-001 | The µservice MUST act as DICOM C-STORE SCP for all modalities listed in `manifest.json#modalities_supported`. |
| FR-DICOM-002 | The µservice MUST sustain ≥10,250 instances/min C-STORE throughput per pod (preserving the healthcare-integration claim). |
| FR-DICOM-003 | The µservice MUST act as DICOM C-FIND SCP at the patient, study, series, and instance level. |
| FR-DICOM-004 | The µservice MUST act as DICOM C-MOVE SCP supporting destination AE Title routing. |
| FR-DICOM-005 | The µservice MUST act as DICOM C-GET SCP. |
| FR-DICOM-006 | The µservice MUST act as DICOM MWL SCP and SCU. |
| FR-DICOM-007 | The µservice MUST act as DICOM MPPS SCP. |
| FR-DICOM-008 | The µservice MUST support DICOM N-CREATE / N-SET / N-ACTION / N-EVENT-REPORT / N-GET for storage commitment and instance availability notification. |
| FR-DICOM-009 | The µservice MUST emit DICOM Structured Reports (SR) and accept incoming SR via C-STORE. |
| FR-DICOM-010 | The µservice MUST implement DICOMweb WADO-RS for instance, frame, bulk-data, and metadata retrieval. |
| FR-DICOM-011 | The µservice MUST implement DICOMweb QIDO-RS for patient, study, series, instance query. |
| FR-DICOM-012 | The µservice MUST implement DICOMweb STOW-RS for store. |
| FR-DICOM-013 | The µservice MUST implement DICOMweb UPS-RS for unified procedure step. |
| FR-DICOM-014 | The µservice MUST publish a DICOM Conformance Statement (PS 3.4) per release. |
| FR-DICOM-015 | The µservice MUST preserve DICOM private tags from all major vendors (GE / Siemens / Philips / Toshiba-Canon / Hitachi / Hologic). |

### 5.2 PACS / VNA (FR-PACS-*)

| ID | Requirement |
|----|-------------|
| FR-PACS-001 | Studies MUST be indexed by Study Instance UID, Series Instance UID, SOP Instance UID, Accession Number, and Patient ID. |
| FR-PACS-002 | Queries MUST execute at p95 < 200ms for study-level QIDO-RS with up to 10 filter parameters. |
| FR-PACS-003 | The VNA MUST support cross-VNA federation via XDS-I.b and XCA-I. |
| FR-PACS-004 | The VNA MUST support legacy-VNA import from GE EA, Philips ISyntax-VNA, Sectra VNA, Fujifilm Synapse VNA, Agfa Impax VNA, Merge VNA. |
| FR-PACS-005 | The VNA MUST deduplicate by SOP Instance UID + transfer syntax. |
| FR-PACS-006 | The VNA MUST tier to deep-archive cold storage after configurable age (default: 2 years untouched). |
| FR-PACS-007 | The VNA MUST support study-level deletion for GDPR right-to-erasure with cryptographic shred. |

### 5.3 Modality Acquisition (FR-ACQ-*)

| ID | Requirement |
|----|-------------|
| FR-ACQ-001 | Each supported modality MUST have a dedicated acquisition state machine. |
| FR-ACQ-002 | MWL pull MUST occur on tech-station modality-arrive event with p95 < 500ms. |
| FR-ACQ-003 | MPPS MUST emit IN_PROGRESS, DISCONTINUED, COMPLETED states with audit-chain entries. |
| FR-ACQ-004 | Mammography acquisition MUST include MQSA-conformant breast positioning metadata. |
| FR-ACQ-005 | CT acquisition MUST emit RDSR dose-structured-report by default per NEMA XR-29. |
| FR-ACQ-006 | Fluoroscopy and IR MUST emit cumulative fluoroscopy time and air-kerma area product. |

### 5.4 Radiologist Workflow (FR-RAD-*)

| ID | Requirement |
|----|-------------|
| FR-RAD-001 | Worklist MUST sort by priority (STAT > preliminary > routine > addendum) with stable secondary sort by acquisition time. |
| FR-RAD-002 | Worklist MUST filter by modality, body-part, sub-specialty, on-call group. |
| FR-RAD-003 | Worklist load MUST execute at p95 < 800ms for up to 5000 items. |
| FR-RAD-004 | Hanging protocols MUST apply at p95 < 150ms per study load. |
| FR-RAD-005 | Hanging protocols MUST be expressible in DICOM PS 3.18 Hanging Protocol format + Oyatie-extension YAML. |
| FR-RAD-006 | Structured-report templates MUST cover BI-RADS, LI-RADS, PI-RADS, TI-RADS, Lung-RADS, O-RADS, CAD-RADS, Bone-RADS, NI-RADS. |
| FR-RAD-007 | Voice-recognition partial transcripts MUST stream at p95 < 250ms latency. |
| FR-RAD-008 | Structured-report save MUST execute at p95 < 800ms. |
| FR-RAD-009 | Reports MUST emit both DICOM-SR and FHIR DiagnosticReport[imaging]. |
| FR-RAD-010 | Critical-results notification MUST reach the ordering clinician at p99 < 30 seconds with closed-loop confirmation required within an escalation timer. |
| FR-RAD-011 | Peer review MUST blind the primary read; the reviewer MUST NOT see the primary report until the peer review is submitted. |
| FR-RAD-012 | Prior studies MUST auto-fetch at p95 < 3 seconds. |
| FR-RAD-013 | MPR rendering MUST execute at p95 < 2 seconds. |
| FR-RAD-014 | The viewer MUST support side-by-side comparison up to 4 viewports without performance degradation. |
| FR-RAD-015 | AI overlays MUST be toggle-able per study without re-fetch. |

### 5.5 AI Image Analysis (FR-AI-*)

| ID | Requirement |
|----|-------------|
| FR-AI-001 | The vendor-neutral CADe/CADx adapter MUST support ≥15 third-party AI vendors at GA. |
| FR-AI-002 | AI inference dispatch MUST execute at p95 < 500ms (round-trip to vendor + back). |
| FR-AI-003 | Per-vendor health metrics (uptime, p95, p99, error rate) MUST be visible to AI Ops Engineer. |
| FR-AI-004 | FDA/CE clearance metadata MUST be stored per vendor model version. |
| FR-AI-005 | Drift detection MUST raise an alert when per-vendor positive-predictive-value drops >10% week-over-week. |
| FR-AI-006 | Stroke LVO detection MUST achieve NPV ≥98% in the validation cohort. |
| FR-AI-007 | Mammography CAD MUST conform to FDA-cleared use indications (no off-label inference). |
| FR-AI-008 | AI overlays MUST be de-identifiable: pixel data sent to vendor MUST be PHI-stripped per HIPAA Safe Harbor + ISO/TS 25237. |

### 5.6 3D / Advanced Visualization (FR-3D-*)

| ID | Requirement |
|----|-------------|
| FR-3D-001 | MPR MUST support axial/coronal/sagittal/oblique reformations. |
| FR-3D-002 | MIP MUST support thick-slab MIP with configurable slab thickness. |
| FR-3D-003 | Curved-MPR MUST support vascular tree centerline extraction. |
| FR-3D-004 | Volume rendering MUST support presets (lung, bone, soft-tissue, vascular, cardiac). |
| FR-3D-005 | Server-side rendering MUST be the default; client-side WebGL is fallback. |

### 5.7 Annotation, Measurement, Comparison (FR-MEAS-*)

| ID | Requirement |
|----|-------------|
| FR-MEAS-001 | Linear, area, angle, volumetric measurements MUST persist as DICOM SR-TID-1500. |
| FR-MEAS-002 | ROI statistics (mean, SD, min, max, volume) MUST compute server-side. |
| FR-MEAS-003 | DICOM Presentation State (PR) MUST persist viewer-state per study per radiologist. |

### 5.8 Dose Tracking (FR-DOSE-*)

| ID | Requirement |
|----|-------------|
| FR-DOSE-001 | RDSR MUST be parsed on C-STORE-completed event. |
| FR-DOSE-002 | Per-protocol dose deviation alerts MUST fire when DLP > protocol-target × 1.25. |
| FR-DOSE-003 | Aggregate dose dashboards MUST report at patient / modality / protocol / technologist granularity. |
| FR-DOSE-004 | EU EURATOM 2013/59 dose register export MUST be supported. |
| FR-DOSE-005 | US CMS QPP MIPS Measure 145 (Radiology: Exposure Time Reported for Procedures Using Fluoroscopy) MUST be supported. |

### 5.9 IHE Profile Conformance (FR-IHE-*)

| ID | Requirement |
|----|-------------|
| FR-IHE-001 | XDS-I.b actor roles: Imaging Document Source, Imaging Document Consumer, Image Display. |
| FR-IHE-002 | XCA-I actor roles: Initiating Imaging Gateway, Responding Imaging Gateway. |
| FR-IHE-003 | IRWF.b actor roles: Image Manager, Image Archive, Image Display. |
| FR-IHE-004 | SWF.b actor roles: ADT Patient Registration, Order Placer, Order Filler, DSS/Order Filler, Image Manager, Image Archive. |
| FR-IHE-005 | REM profile actor roles: Acquisition Modality, RDSR Repository, Dose Information Reporter, Dose Information Consumer. |
| FR-IHE-006 | AIW-I actor roles: Image Manager / Archive, Imaging Object Change Management. |
| FR-IHE-007 | ATNA audit emission MUST cover every PHI access. |
| FR-IHE-008 | Consistent Time MUST be enforced via NTP with maximum skew 1 second. |

### 5.10 Enterprise Imaging Beyond Radiology (FR-EI-*)

| ID | Requirement |
|----|-------------|
| FR-EI-001 | Cardiology: echo (DICOM SR for adult echo per ASE), cath (DICOM coronary angio), EP (12-lead ECG via DICOM ECG SOP class + waveform SR). |
| FR-EI-002 | Ophthalmology: OCT (DICOM Ophthalmic Tomography SOP), fundus (DICOM Ophthalmic Photography SOP), visual field (DICOM Ophthalmic Visual Field SOP). |
| FR-EI-003 | Dermatology: clinical photo (DICOM Visible Light Photography SOP). |
| FR-EI-004 | Pathology: whole-slide imaging via DICOM Pathology / DICOM VL Whole Slide Microscopy IOD. |
| FR-EI-005 | Dental: panoramic + CBCT (DICOM Dental panoramic SOP). |
| FR-EI-006 | Surgical: intraoperative video (DICOM Video Endoscopic SOP). |

### 5.11 Mammography (FR-MAMMO-*)

| ID | Requirement |
|----|-------------|
| FR-MAMMO-001 | Screening recall workflow MUST track recall rate per radiologist with MQSA audit. |
| FR-MAMMO-002 | BI-RADS audit MUST compute positive-predictive-value, cancer detection rate, sensitivity, specificity. |
| FR-MAMMO-003 | DBT (digital breast tomosynthesis) MUST display synthesized 2D + sliced 3D. |
| FR-MAMMO-004 | MQSA retention MUST default to 10 years (US). |
| FR-MAMMO-005 | Mammography CAD MUST run on synthesized 2D + DBT slices. |

### 5.12 Patient Portal Sharing (FR-PORTAL-*)

| ID | Requirement |
|----|-------------|
| FR-PORTAL-001 | Patient access MUST require consent-graph affirmative consent. |
| FR-PORTAL-002 | DICOMweb tokens MUST be time-boxed and revocable. |
| FR-PORTAL-003 | Plain-language layperson summary MUST be generated from the structured report. |
| FR-PORTAL-004 | FHIR ImagingStudy export MUST be supported. |
| FR-PORTAL-005 | Cross-clinician share links MUST be revocable. |

---

## 6. Non-Functional Requirements

### 6.1 Performance

| Target | Value | Source |
|--------|-------|--------|
| DICOM C-STORE throughput | ≥10,250 instances/min/pod | Preserved from `healthcare-integration` |
| Image-pull p95 | < 1 second | This PRD §1.1 |
| Multi-GB study load p95 | < 5 seconds | This PRD §1.1 |
| Hanging-protocol apply p95 | < 150ms | This PRD §5.4 FR-RAD-004 |
| AI inference dispatch p95 | < 500ms | This PRD §5.5 FR-AI-002 |
| Critical-result notification p99 | < 30 seconds | This PRD §5.4 FR-RAD-010 |
| MPR render p95 | < 2 seconds | This PRD §5.4 FR-RAD-013 |
| Prior auto-fetch p95 | < 3 seconds | This PRD §5.4 FR-RAD-012 |
| Voice-recognition partial transcript p95 | < 250ms | This PRD §5.4 FR-RAD-007 |
| Structured-report save p95 | < 800ms | This PRD §5.4 FR-RAD-008 |
| QIDO-RS study query p95 | < 200ms | This PRD §5.2 FR-PACS-002 |
| Worklist load p95 (5k items) | < 800ms | This PRD §5.4 FR-RAD-003 |
| MWL pull p95 | < 500ms | This PRD §5.3 FR-ACQ-002 |

### 6.2 Availability

| Surface | SLO | Window |
|---------|-----|--------|
| C-STORE SCP | 99.99% | 30d rolling |
| DICOMweb WADO-RS/QIDO-RS | 99.99% | 30d rolling |
| MWL SCP | 99.95% | 30d rolling |
| Reading worklist | 99.95% | 30d rolling |
| AI marketplace dispatch | 99.9% | 30d rolling |
| Patient portal | 99.5% | 30d rolling |
| Peer review | 99.0% | 30d rolling |

### 6.3 Durability

| Asset class | Durability | Replication |
|-------------|------------|-------------|
| DICOM pixel data | 13 nines | Erasure-coded ≥3 cells per pack policy |
| DICOM SR + FHIR DiagnosticReport | 13 nines | Same |
| Audit-chain PHI access records | 13 nines | Cross-cell replicated; tamper-evident hash chain |
| Worklist state | 11 nines | Cross-AZ replicated |

### 6.4 Security & Privacy

- HIPAA Security Rule: technical safeguards 164.312 (access control, audit controls, integrity, person/entity authentication, transmission security).
- HIPAA Privacy Rule: minimum-necessary 164.502(b).
- GDPR Article 9 special-category data (health) + Article 32 (security of processing).
- KR-Medical-Devices Act (의료기기법) cybersecurity guidance.
- EU-MDR Annex I §17 cybersecurity essential requirements.
- All PHI access Cedar-authorized + audit-chain emitted.
- AI vendor PHI de-identification per HIPAA Safe Harbor + ISO/TS 25237 BEFORE vendor egress.
- BYOK per ADR-0255 §D-4 for tenants requiring customer-managed encryption keys.

### 6.5 Compliance

| Pack | Coverage |
|------|----------|
| HIPAA-2024 | Privacy + Security + Breach Notification + Omnibus |
| GDPR | Articles 9, 17, 20, 32, 33, 34, 35 |
| SOC-2 | Trust Service Criteria CC1..CC9 + PI |
| ISO-27001 | A.5..A.18 Annex A controls |
| EU-AI-Act | Article 6 high-risk + Annex III §3 medical-device-AI |
| EU-MDR | Annex I + Article 10 + Article 52 |
| KR-Medical-Devices | 의료기기법 + 의료기기 사이버보안 가이드라인 |
| GxP | GAMP-5 categorization + Part 11 audit |
| FDA-21-CFR-Part-11 | Electronic records + electronic signatures |
| ACR-Accreditation | Radiology, Mammography, CT, MRI, NM, US accreditation programs |
| MQSA-Mammography | Quality + retention + audit |
| NEMA-DICOM-PS3-conformance | Full PS 3.1..3.18 conformance |
| IHE-Radiology | All listed profiles in §5.9 |

### 6.6 Scalability

Horizontal scalability per `feedback_quality_performance_scalability_bar`. Per-pod throughput targets MUST hold under 10× pod count scale-out. No single-writer per-tenant bottlenecks except where mandated by HIPAA serial-write audit requirements.

### 6.7 Observability

OpenTelemetry traces on every DICOM association, DICOMweb request, AI inference call, voice-recognition partial-transcript stream. RED metrics (rate, errors, duration) per bounded context. Service-level objective definitions in OpenSLO format at `slos/`.

### 6.8 DR Posture (ADR-0343)

- Target: RTO 300s and RPO 0s for DICOM ingest, VNA index, DICOMweb retrieval, structured reporting, and AI triage dispatch, matching `manifest.json` `dr.rto_p99_seconds=300` and `dr.rpo_p99_seconds=0`.
- Compliance floors: HIPAA-2024 floors at 3600s/300s with multi-region required; EU-AI-ACT-2024-HIGH-RISK floors at 1800s/300s with multi-region required for medical-device AI; SOC2-T2 floors at 14400s/900s; ISO27001-2022 floors at 14400s/3600s; KR-PIPA sensitive-PI floor at 7200s/600s. The effective imaging target remains 300s/0s with active-active regulated storage and critical-result routing.
- failover_runbook: `microservices/imaging/runbooks/imaging-vna-failover.md`.
- multi_region_active_active: true for VNA metadata, DICOMweb read path, critical-result events, and AI triage queues; bulk pixel rehydration may follow pack-specific recovery ordering.
- Why: radiology can keep acquiring, finding, reading, and escalating studies through a regional failure without losing PHI pixel custody.

### 6.9 Capacity Model (ADR-0340)

- Per-tenant baseline: 2.5 vCPU, 4096 MiB RAM, 10240 GB object storage for VNA pixel data, 10 Postgres connections, 2 Valkey connections, and 20 outbound HTTP connections, matching `manifest.json` `capacity_model`.
- Scaling dimension: `per_request` for DICOM instances, DICOMweb calls, AI dispatches, and structured-report saves.
- Cell placement class: Tier-2. Imaging pairs `pod_runtime_tier=1` with PHI pixel-data and radiology data-plane isolation without claiming tenant-code execution.
- Autoscaling boundaries: min 3 pods per tenant cell, max 60 pods per tenant cell before study-volume, modality-cluster, or VNA tier split review.
- Why: the service must preserve DICOM C-STORE, DICOMweb, VNA read, radiology worklist, and AI dispatch load while isolating large pixel-storage bursts per tenant and cell.

### 6.10 Sustainability + Cost Attribution (ADR-0344)

- Emission envelope: every imaging audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, modality, capability, provider, cell, storage tier, and compliance-pack dimensions.
- Provider-routing affected by carbon: no for critical-result reads, emergent imaging, AI triage, or EU-AI high-risk medical-device workflows; yes for non-urgent 3D reconstruction, prior prefetch, lifecycle compaction, and cold-tier VNA rebalancing.
- Tenant cost transparency: `finops-portal` exposes imaging cost by study, modality, pixel-storage tier, AI-vendor dispatch, DICOMweb transfer, and lifecycle policy.
- Why: CSRD, SB-253, and SEC climate-disclosure evidence must account for high-storage imaging workloads without delaying emergent reads or regulated AI triage.

### 6.11 API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet across `Oyatie-Api-Version`, `/v/YYYY-MM-DD/...` URL prefixes, and proto3 `api_version_date` fields.
- SDK semver model: major.minor.patch for DICOMweb, FHIR ImagingStudy, AI marketplace, portal image-sharing, and VNA migration clients.
- Support window: last 3 public versions supported for at least 180 days.
- Per-tenant pinning: yes for PACS, VNA, modality, DICOMweb, AI-vendor, and patient-portal integrations.
- Internal-mesh exemption: yes; direct gRPC and event handoffs to EMR, diagnostics, healthcare-integration, audit-chain, and cloud-billing preserve ADR-0145.

---

## 7. Tenant-Class Behavior Matrix

Per ADR-0329 + ADR-0330 + ADR-0331.

| Capability | demo_trial | paid (sovereign-cell) |
|------------|-----------|-----------------------|
| PHI ingestion | Forbidden — synthetic studies only | Required for clinical use |
| Modality count cap | 7 modalities, 500 studies total | Unlimited |
| AI inference / day | 100 inferences | Unlimited per tenant pack |
| Voice recognition | Disabled | Enabled (Nuance / M*Modal / in-house) |
| Critical-results closed-loop | Disabled (demo mode only) | Enabled (mandatory for clinical) |
| Audit retention | 30 days | Pack-defined: HIPAA 6 years; MQSA 10; GDPR per-request |
| Peer review | Disabled | Enabled |
| Cross-VNA federation | Disabled | Enabled |
| Patient portal sharing | Disabled (demo only) | Enabled (consent-graph gated) |
| BYOK | Disabled | Per ADR-0255 §D-4 |
| Sovereign-cell isolation | N/A | Mandatory |
| Billing meters emitted | None | per_seat + per_usage |

---

## 8. Deployment Contexts (6)

Per the `feedback_multi_context_provider_agnostic_2026_05_20` memory.

### 8.1 aws-guest

For AI inference compute fan-out (GPU-bound). PACS-substrate runs on-prem; aws-guest is reachable via VPN. OpenTofu module: `iac/aws-guest/`.

### 8.2 oci-guest

For demo_trial tenants on OCI Always Free + paid tenants choosing OCI for Oracle-aligned health systems. Exploits 2× Ampere A1 ARM 4 OCPU + 24GB RAM tier. OpenTofu module: `iac/oci-guest/`.

### 8.3 on-prem

Most common for hospital PACS. Customer-controlled hardware in their data center. OpenTofu module: `iac/on-prem/`.

### 8.4 colo

Co-located hardware adjacent to modalities for sub-second image-pull SLO. Common for very-large hospital systems. OpenTofu module: `iac/colo/`.

### 8.5 oyatie-cloud-provider

Hosted radiology service bureau / teleradiology night-hawk pattern. OpenTofu module: `iac/oyatie-cloud-provider/`.

### 8.6 sovereign-cell

Mandatory for paid PHI. Cell-aware deployment with Cedar isolation. OpenTofu module: `iac/sovereign-cell/`.

---

## 9. Top-3 Counterpart Capability Anchoring

For each of the top-3 counterparts, the µservice MUST publish a parity matrix row covering:

1. Capability name
2. GE Centricity coverage (yes/no/partial)
3. Philips IntelliSpace coverage
4. Sectra coverage
5. Oyatie imaging coverage
6. Differentiation (substrate / openness / cell / AI marketplace / DICOMweb-first)

See `competitor-parity-matrix.md` for ≥150 rows.

---

## 10. Cross-µservice Dependencies

| µservice | Why depended-upon |
|----------|-------------------|
| cloud-iam | Cedar gateway for every read/write/admin call |
| identity | Radiologist / technologist / clinician identity |
| audit-chain | PHI access audit (HIPAA 164.312(b)) |
| cloud-storage | DICOM pixel data blob durability (13 nines) |
| observability | OpenTelemetry + SLO emission |
| cloud-billing | Per-seat + per-usage meter emission |
| emr | Order receipt + report distribution |
| diagnostics | Lab-result-image correlation (after reconciliation) |
| consent-graph | Patient consent + share-link consent |
| compliance | Pack overlay (HIPAA, MQSA, EU-MDR, KR-MD) |
| workflow-engine | Critical-results escalation workflow |
| cloud-kms | BYOK + at-rest envelope encryption |
| cloud-data | Worklist + structured-report relational state |
| api-gateway | Edge surface for DICOMweb + FHIR |

---

## 11. Out-of-Scope (Explicit Refusal)

- Clinical decision support outside imaging-modality scope (belongs to `emr` + `diagnostics`).
- Lab orders, specimen tracking (belongs to `diagnostics` lab portion).
- Provider directory / credentialing (belongs to `identity` + `cloud-iam`).
- Claims adjudication / revenue-cycle management (belongs to `cloud-billing` + future RCM µservice).
- Genomics / molecular pathology beyond visual WSI (belongs to future `pathology` µservice when split).
- Patient scheduling outside imaging context (belongs to `calendar` + `emr`).

---

## 12. Success Metrics

| Metric | Target | Window |
|--------|--------|--------|
| New-hospital onboarding time | ≤6 weeks | per onboarding |
| Modality on-boarding time | ≤2 hours | per modality |
| Reading-room productivity vs. GE/Sectra | +15% | per shift |
| AI vendor marketplace breadth | ≥15 vendors GA | annual |
| Audit completeness | 100% | rolling |
| SLO breach incidents per year | ≤2 per surface | annual |
| Compliance pack certification time | ≤90 days | per pack |

---

## 13. Versioning & Compatibility

- DICOM Conformance Statement per release tag.
- DICOMweb OpenAPI + AsyncAPI + proto semver.
- FHIR profile versioning per US-Core, UK-Core, KR-Core, IPA.
- Breaking changes require ADR + sunset notice ≥180 days + dual-emit deprecation per `feedback_no_silent_regression`.

---

## 14. Roadmap (Wave Sequencing)

Per ADR-0328 §D-15..D-20 substance-bar discipline + Big-8 priority.

| Wave | Scope |
|------|-------|
| 15M-G (this wave) | µservice scaffold, PRD, ARCHITECTURE, contracts, SLOs, policies, IaC, ADRs, IPs, parity matrix, REMEDIATION |
| 15M follow-up | Reconciliation with diagnostics imaging-portion supersession |
| 16-imaging-substrate | DICOM substrate kernel + adapters (IP-001..IP-005) |
| 17-imaging-pacs | PACS query / store / retrieve (IP-006..IP-009) |
| 18-imaging-rad-workflow | Worklist + hanging protocols + structured reporting (IP-010..IP-012) |
| 19-imaging-ai | AI marketplace + CADe/CADx adapter (IP-013) |
| 20-imaging-enterprise | Enterprise imaging beyond radiology (IP-014) |
| 21-imaging-mammography-dose | Mammography + dose tracking + IHE REM (IP-015) |

---

## 15. References

- `microservices/healthcare-integration/manifest.json` (preserved 10,250 inst/min claim)
- `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md`
- `docs/decisions/ADR-0131-per-microservice-flat-layout.md`
- `docs/decisions/ADR-0132-no-suite-policy.md`
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`
- `docs/decisions/ADR-0251-compliance-pack-primitive.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
- `docs/decisions/ADR-0329-tier-retirement.md`
- `docs/decisions/ADR-0330-tenant-class.md`
- `docs/decisions/ADR-0331-per-microservice-adoption.md`
- `docs/decisions/ADR-0332-healthcare-domain-decomposition.md`
- `docs/decisions/ADR-0212-substance-bar.md`
- DICOM PS 3.1..3.20 (NEMA)
- IHE Radiology Technical Framework (Vol 1 + Vol 2 + Vol 3 + Vol 4 supplements)
- HL7 FHIR R5 + R4 ImagingStudy + ImagingSelection + DiagnosticReport[imaging] + DocumentReference[CDA]
- ACR Practice Parameters + Technical Standards
- MQSA 21 CFR 900
- EURATOM 2013/59
- NEMA XR-29 (CT Smart Dose) + XR-25 (CT Dose Check)
- FDA 21 CFR Part 11
- ISO/TS 25237 (Pseudonymization in Healthcare)
- HIPAA Safe Harbor (45 CFR 164.514(b)(2))

---

## 16. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Legacy VNA migration data integrity | Med | High | Per-vendor adapter with checksum + sample-verified validation lane |
| AI vendor model drift | High | Med | Drift detector per FR-AI-005 + on-call AI-Ops |
| Voice recognition vendor lock-in | Med | Low | Whisper-medical fallback + multi-vendor abstraction |
| Modality private-tag breakage on vendor upgrade | High | Low | Vendor-quirk regression suite per modality |
| Sovereign-cell capacity exhaustion | Low | High | Per-cell capacity admission control + cross-pack capacity dashboards |
| HIPAA breach via AI vendor egress | Low | Critical | Pixel-PHI de-identification gate per FR-AI-008 + Cedar policy `ai-model-can-read-deidentified.cedar` |
| Critical-result delivery failure | Low | Critical | Multi-channel escalation + audit-chain + workflow-engine retry |
| MQSA retention violation | Low | High | MQSA-retention enforcement gate via compliance pack |
| Cross-VNA federation auth failure | Med | Med | XCA-I conformance suite + IHE Connectathon participation |
| Multi-GB study load latency under load | Med | Med | Visage-class server-side rendering + edge CDN for thumbnails |

---

## 17. Glossary

| Term | Definition |
|------|------------|
| PACS | Picture Archiving and Communication System |
| VNA | Vendor-Neutral Archive |
| DICOM | Digital Imaging and Communications in Medicine (NEMA PS 3.1..3.20) |
| DICOMweb | RESTful DICOM (WADO-RS/QIDO-RS/STOW-RS/UPS-RS) |
| MWL | Modality Worklist (DICOM C-FIND-SCP for scheduled procedures) |
| MPPS | Modality Performed Procedure Step |
| RDSR | Radiation Dose Structured Report (DICOM SR-TID-10001..10013) |
| SR | Structured Report |
| SOP | Service-Object Pair (DICOM instance) |
| AE Title | Application Entity Title (DICOM endpoint name) |
| C-STORE | DICOM Composite Store service |
| C-FIND | DICOM Composite Find service |
| C-MOVE | DICOM Composite Move service |
| BI-RADS | Breast Imaging Reporting and Data System (ACR) |
| LI-RADS | Liver Imaging Reporting and Data System (ACR) |
| PI-RADS | Prostate Imaging Reporting and Data System (ACR) |
| TI-RADS | Thyroid Imaging Reporting and Data System (ACR) |
| Lung-RADS | Lung CT Screening Reporting and Data System (ACR) |
| O-RADS | Ovary / Adnexa Imaging Reporting and Data System (ACR) |
| CAD-RADS | Coronary Artery Disease Reporting and Data System (SCCT/ACR) |
| MQSA | Mammography Quality Standards Act (US, FDA 21 CFR 900) |
| IHE | Integrating the Healthcare Enterprise |
| XDS-I.b | IHE Cross-Enterprise Document Sharing for Imaging (b version) |
| XCA-I | IHE Cross-Community Access for Imaging |
| IRWF.b | IHE Imaging Workflow (b version) |
| SWF.b | IHE Scheduled Workflow (b version) |
| REM | IHE Radiation Exposure Monitoring |
| AIW-I | IHE Image Object Change Management |
| ATNA | IHE Audit Trail and Node Authentication |
| RadPeer | ACR peer review program |
| CADe / CADx | Computer-Aided Detection / Computer-Aided Characterization |
| LVO | Large Vessel Occlusion (stroke) |
| SUV | Standardized Uptake Value (PET) |
| Deauville | Lymphoma PET response 5-point scale |
| PERCIST | PET Response Criteria in Solid Tumors |
| DBT | Digital Breast Tomosynthesis |
| OCT | Optical Coherence Tomography |
| WSI | Whole-Slide Imaging (digital pathology) |
| RIS | Radiology Information System |
| EHR / EMR | Electronic Health / Medical Record |
| FHIR | HL7 Fast Healthcare Interoperability Resources |
| Cedar | AWS Cedar policy language (ADR-0243 universal gate) |
| Pack | Compliance / localization pack per ADR-0251 |
| Cell | Cell-based isolation unit per ADR-0248 |

---

## 18. Open Questions / Future Splits

- **Pathology WSI split:** as digital pathology matures, the WSI substrate (massive image sizes 10–100+ GB per slide) may warrant its own `pathology` µservice per ADR-0132 single-concern doctrine. Currently in scope.
- **Cardiology echo / cath split:** if cardiology imaging acquires substantial cardiology-specific workflow (TAVR planning, structural-heart imaging), it may warrant a `cardiology-imaging` µservice. Currently in scope.
- **Teleradiology service bureau split:** if the night-hawk / teleradiology marketplace surface acquires substantial RFP/billing surface, it may warrant a `teleradiology-marketplace` µservice. Currently in scope.

---

## 19. Sign-Off Criteria (Wave 15M-G)

| Artifact | Status |
|----------|--------|
| manifest.json | Authored |
| supported-oses.json | Authored |
| PRD.md (this document) | Authored ≥800 lines |
| ARCHITECTURE.md | Authored ≥600 lines |
| README.md | Authored ≥300 lines |
| contracts/openapi.yaml | Authored |
| contracts/asyncapi.yaml | Authored |
| contracts/proto/imaging.proto | Authored |
| slos/ (≥12 OpenSLO files) | Authored |
| policies/ (≥8 Cedar files) | Authored |
| iac/ (6 contexts) | Authored |
| decisions/ADR-MS-001..004 | Authored |
| implementation-plans/IP-001..015 | Authored |
| competitor-parity-matrix.md (≥150 rows) | Authored |
| REMEDIATION-NOTES-2026-05-21.md | Authored |
| Substance-bar artifact count | ≥100 per ADR-0212 |

---

## 20. Per-Modality SOP Class Coverage Detail

The µservice's DICOM Conformance Statement covers the following SOP classes per modality. This list is the authoritative scope ceiling for Wave 15M-G; Wave 16+ may add SOP classes for newer modalities (cryo-EM whole-organ imaging, photon-counting CT, ultra-high-field 7T MRI clinical, theranostic isotopes).

### 20.1 CT SOP Classes

- CT Image Storage (1.2.840.10008.5.1.4.1.1.2)
- Enhanced CT Image Storage (1.2.840.10008.5.1.4.1.1.2.1)
- Legacy Converted Enhanced CT Image Storage (1.2.840.10008.5.1.4.1.1.2.2)
- CT Performed Procedure Protocol Storage (1.2.840.10008.5.1.4.1.1.200.2)
- Radiation Dose SR Storage (1.2.840.10008.5.1.4.1.1.88.67)
- Encapsulated PDF Storage for dose-summary (1.2.840.10008.5.1.4.1.1.104.1)
- 4D-CT cardiac-gated multi-frame.

### 20.2 MRI SOP Classes

- MR Image Storage (1.2.840.10008.5.1.4.1.1.4)
- Enhanced MR Image Storage (1.2.840.10008.5.1.4.1.1.4.1)
- Legacy Converted Enhanced MR (1.2.840.10008.5.1.4.1.1.4.4)
- MR Spectroscopy Storage (1.2.840.10008.5.1.4.1.1.4.2)
- Enhanced MR Color Image Storage (1.2.840.10008.5.1.4.1.1.4.3)

### 20.3 X-ray, Mammography, Fluoroscopy

- Digital X-Ray Image Storage – For Presentation (1.2.840.10008.5.1.4.1.1.1.1)
- Digital X-Ray Image Storage – For Processing (1.2.840.10008.5.1.4.1.1.1.1.1)
- Digital Mammography X-Ray Image Storage – For Presentation (1.2.840.10008.5.1.4.1.1.1.2)
- Digital Mammography X-Ray Image Storage – For Processing (1.2.840.10008.5.1.4.1.1.1.2.1)
- Breast Tomosynthesis Image Storage (1.2.840.10008.5.1.4.1.1.13.1.3)
- Breast Projection X-Ray Image Storage – For Presentation (1.2.840.10008.5.1.4.1.1.13.1.4)
- Breast Projection X-Ray Image Storage – For Processing (1.2.840.10008.5.1.4.1.1.13.1.5)
- X-Ray Angiographic Image Storage (1.2.840.10008.5.1.4.1.1.12.1)
- Enhanced XA Image Storage (1.2.840.10008.5.1.4.1.1.12.1.1)
- X-Ray Radiofluoroscopic Image Storage (1.2.840.10008.5.1.4.1.1.12.2)
- Enhanced XRF Image Storage (1.2.840.10008.5.1.4.1.1.12.2.1)
- X-Ray 3D Angiographic Image Storage (1.2.840.10008.5.1.4.1.1.13.1.1)
- X-Ray Radiation Dose SR Storage (1.2.840.10008.5.1.4.1.1.88.67)

### 20.4 Ultrasound

- Ultrasound Image Storage – Retired (1.2.840.10008.5.1.4.1.1.6)
- Ultrasound Image Storage (1.2.840.10008.5.1.4.1.1.6.1)
- Enhanced US Volume Storage (1.2.840.10008.5.1.4.1.1.6.2)
- Ultrasound Multi-frame Image Storage – Retired (1.2.840.10008.5.1.4.1.1.3)
- Ultrasound Multi-frame Image Storage (1.2.840.10008.5.1.4.1.1.3.1)

### 20.5 Nuclear Medicine, PET

- Nuclear Medicine Image Storage – Retired (1.2.840.10008.5.1.4.1.1.5)
- Nuclear Medicine Image Storage (1.2.840.10008.5.1.4.1.1.20)
- Positron Emission Tomography Image Storage (1.2.840.10008.5.1.4.1.1.128)
- Enhanced PET Image Storage (1.2.840.10008.5.1.4.1.1.130)
- Legacy Converted Enhanced PET (1.2.840.10008.5.1.4.1.1.128.1)
- PET-CT and PET-MR co-registered storage (vendor-specific transfer syntaxes accepted).

### 20.6 Enterprise (Non-Radiology) SOP Classes

- Ophthalmic Photography 8-Bit Image Storage (1.2.840.10008.5.1.4.1.1.77.1.5.1)
- Ophthalmic Tomography Image Storage (1.2.840.10008.5.1.4.1.1.77.1.5.4)
- Ophthalmic Visual Field Static Perimetry Measurements Storage
- Visible Light Endoscopic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.1)
- Video Endoscopic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.1.1)
- Visible Light Microscopic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.2)
- Video Microscopic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.2.1)
- Visible Light Photographic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.4)
- Video Photographic Image Storage (1.2.840.10008.5.1.4.1.1.77.1.4.1)
- VL Whole Slide Microscopy Image Storage (1.2.840.10008.5.1.4.1.1.77.1.6)
- Dermoscopic Photography Image Storage
- 12-lead ECG Waveform Storage (1.2.840.10008.5.1.4.1.1.9.1.1)
- General ECG Waveform Storage (1.2.840.10008.5.1.4.1.1.9.1.2)
- Ambulatory ECG Waveform Storage (1.2.840.10008.5.1.4.1.1.9.1.3)
- Hemodynamic Waveform Storage (1.2.840.10008.5.1.4.1.1.9.2.1)
- Cardiac Electrophysiology Waveform Storage (1.2.840.10008.5.1.4.1.1.9.3.1)

### 20.7 Structured Report SOP Classes

- Basic Text SR Storage (1.2.840.10008.5.1.4.1.1.88.11)
- Enhanced SR Storage (1.2.840.10008.5.1.4.1.1.88.22)
- Comprehensive SR Storage (1.2.840.10008.5.1.4.1.1.88.33)
- Comprehensive 3D SR Storage (1.2.840.10008.5.1.4.1.1.88.34)
- Procedure Log Storage
- Mammography CAD SR Storage (1.2.840.10008.5.1.4.1.1.88.50)
- Chest CAD SR Storage (1.2.840.10008.5.1.4.1.1.88.65)
- X-Ray Radiation Dose SR Storage
- Radiopharmaceutical Radiation Dose SR Storage (1.2.840.10008.5.1.4.1.1.88.68)
- Colon CAD SR Storage (1.2.840.10008.5.1.4.1.1.88.69)
- Implantation Plan SR Storage (1.2.840.10008.5.1.4.1.1.88.70)
- Acquisition Context SR Storage (1.2.840.10008.5.1.4.1.1.88.71)
- Simplified Adult Echo SR Storage (1.2.840.10008.5.1.4.1.1.88.72)
- Patient Radiation Dose SR Storage (1.2.840.10008.5.1.4.1.1.88.73)
- Planned Imaging Agent Administration SR Storage (1.2.840.10008.5.1.4.1.1.88.74)
- Performed Imaging Agent Administration SR Storage (1.2.840.10008.5.1.4.1.1.88.75)

### 20.8 Other Storage SOP Classes

- Encapsulated PDF Storage (1.2.840.10008.5.1.4.1.1.104.1)
- Encapsulated CDA Storage (1.2.840.10008.5.1.4.1.1.104.2)
- Encapsulated STL Storage (1.2.840.10008.5.1.4.1.1.104.3)
- Encapsulated OBJ Storage (1.2.840.10008.5.1.4.1.1.104.4)
- Encapsulated MTL Storage (1.2.840.10008.5.1.4.1.1.104.5)

---

## 21. Tenant-Class Transition Policy

Per ADR-0330 + ADR-0331, tenant_class transitions are auditable. Transitions supported:

- `demo_trial → paid` — upgrade path with no PHI carry-over (synthetic-only data discarded; new paid tenant onboarding starts fresh sovereign-cell binding).
- `paid → archived` — wind-down path; retention honored per pack policy until expiry; cryptographic shred on archive completion.

Forbidden transitions:

- `paid → demo_trial` (would orphan PHI).
- `archived → paid` (must reinstantiate).

Audit-chain entries are emitted on every transition.

---

## 22. Modality Onboarding Workflow

A new modality is onboarded into the µservice by:

1. Modality config registered via `POST /admin/modalities` (OpenAPI surface).
2. AE-title pairing established with Cedar policy `technologist-can-acquire.cedar`.
3. Vendor-quirks profile selected from `oya-imaging-modality-vendor-quirks` library.
4. MWL filter rules added to per-modality config.
5. Dose-emission flag enabled (default true for CT / fluoro / IR / mammography).
6. Acquisition state machine validated against IHE SWF.b actor.
7. Smoke test: synthetic study round-trip through C-STORE → PACS index → worklist surface.

p95 modality onboarding time: ≤ 2 hours per PRD §12.

---

## 23. Disaster Recovery Drills

Per `feedback_quality_performance_scalability_bar`, DR drills are mandatory:

- **Monthly cell-level failover drill** — primary cell goes offline; failover cell within same pack takes traffic; RTO ≤ 5 minutes asserted.
- **Quarterly cross-cell-within-pack rebuild drill** — full cell rebuild from cross-cell replica; data integrity asserted via per-instance SHA-256 checksum.
- **Annual pack-policy violation drill** — attempt cross-pack failover (e.g., EU pack → US pack) and assert Cedar denial.
- **Semi-annual key-rotation drill** — KMS-wrapped DEK rotation under load; per-tenant audit-chain entries emitted.

Each drill emits a published evidence bundle to `evidence/dr-drill-<date>.json`.

---

## 24. Backward Compatibility Strategy

DICOM C-STORE association compatibility:

- Support all ACR-published implicit + explicit transfer syntaxes back to 2000.
- Negotiate down to least-common transfer syntax with private-tag preservation.
- Vendor-specific quirks bridged by `oya-imaging-modality-vendor-quirks`.

DICOMweb compatibility:

- WADO-RS GA and DICOMweb v2 (draft) parsed.
- QIDO-RS spec PS 3.18 conformance with Oyatie-extension query parameters opt-in.

FHIR R4 + R5 dual:

- Default R5 emit; R4 adapter on request via Accept header.

gRPC API stability:

- proto3 reserved-field discipline; no breaking changes within minor version per `feedback_no_silent_regression`.

---

## 25. Acknowledgements

This PRD is authored as the sole-owner artifact for the imaging µservice per ADR-0132 + user directive 2026-05-21. It supersedes the imaging portions of the concurrently-authored diagnostics µservice, with reconciliation queued for Wave 15M follow-up. All performance leadership claims preserved from `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md`. All compliance pack obligations rooted in ADR-0251. All cross-µservice contracts defined per ADR-0245 substrate-vs-product layering.
