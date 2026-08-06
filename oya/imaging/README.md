# Imaging Microservice

`status: wave-15m-g-authored-2026-05-21`
`split_authority: ADR-0132 + user directive 2026-05-21`
`top_3_anchors: GE Centricity Universal Viewer + PACS-IW; Philips IntelliSpace PACS + Enterprise Imaging; Sectra PACS + VNA`

---

## Overview

The Imaging microservice is Oyatie's hyperscaler-grade medical imaging platform. Single-concern per ADR-0132 — covers the full PACS / VNA / DICOM substrate / radiologist workflow / AI image analysis / enterprise imaging surface.

This µservice was split out of the bundled `diagnostics` µservice (which historically combined lab + imaging + pathology) per ADR-0132's no-grouping doctrine. The concurrently-authored diagnostics µservice retains a bundled scope; **this imaging µservice's authority SUPERSEDES the diagnostics imaging-portions**. Reconciliation is queued for Wave 15M follow-up.

Top-3 industry counterparts the µservice is designed to displace:

1. **GE Healthcare Centricity Universal Viewer + Centricity PACS-IW + Centricity Enterprise Archive** — legacy dominance from deep modality integration (GE makes CT, MRI, US, MG, NM, PET hardware) and installed-base lock-in.
2. **Philips IntelliSpace PACS + Enterprise Imaging (Carestream Vue PACS lineage)** — strong cardiology; non-radiology enterprise imaging strength; post-acquisition product fragmentation as weakness.
3. **Sectra PACS + Sectra VNA** — reading-room workflow leader (especially for high-volume CT and breast); KLAS repeat-customer leader for radiology.

Secondary anchors: Agfa Enterprise Imaging, Visage 7, Merge PACS (IBM), Fujifilm Synapse, Siemens syngo.via, Change Healthcare Stratus.

---

## What This µservice Owns

End-to-end medical imaging:

- **DICOM substrate** — DIMSE (C-STORE / C-FIND / C-MOVE / C-GET / MWL / MPPS / N-CREATE / N-SET / N-ACTION / N-EVENT-REPORT / N-GET / SR) + DICOMweb (WADO-RS / QIDO-RS / STOW-RS / UPS-RS).
- **PACS** — Picture Archiving and Communication System with study/series/instance indexing and worklist surface.
- **VNA** — Vendor-Neutral Archive with cross-VNA federation (XDS-I.b / XCA-I) and legacy-vendor migration (GE EA, Philips ISyntax-VNA, Sectra VNA, Fujifilm Synapse VNA, Agfa Impax, Merge VNA).
- **Per-modality acquisition** — CT (incl. dual-energy / spectral / cardiac-gated / 4D), MRI (T1/T2/FLAIR/DWI/perfusion/spectroscopy/fMRI), X-ray (DR/CR), ultrasound (B-mode / color Doppler / spectral / M-mode / contrast-enhanced), mammography (2D + DBT + diagnostic + screening), nuclear medicine (planar + SPECT), PET (PET/CT + PET/MRI), fluoroscopy / angiography / interventional radiology, DEXA, OCT, echo, cath, EP, ophthalmology fundus / visual field, dermatology clinical photo, pathology whole-slide imaging, dental panoramic / CBCT, surgical intraoperative video.
- **Radiologist workflow** — dynamic prioritized worklist; hanging protocols (per modality + per body-part + per radiologist); structured reporting (BI-RADS, LI-RADS, PI-RADS, TI-RADS, Lung-RADS, O-RADS, CAD-RADS, Bone-RADS, NI-RADS, ACR templates, RadLex); voice recognition (Nuance Dragon Medical, M*Modal Catalyst, in-house Whisper-medical); critical results closed-loop with cascading escalation; peer review (RadPeer-style); RadPeer integration.
- **AI image analysis** — vendor-neutral CADe/CADx fan-out covering ≥15 vendors (Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys, Caption Health, RapidAI, Subtle Medical, Imagia, Behold.ai, ScreenPoint). Indications: lesion detection / characterization, lung nodule, breast lesion, brain hemorrhage, stroke LVO triage, pulmonary embolism, bone fracture, cardiac quantification.
- **3D / Advanced visualization** — MPR (multi-planar reformation), MIP (maximum intensity projection), curved-MPR vascular, volume rendering, server-side rendering with WebGL fallback.
- **Annotation + measurement + comparison** — linear / area / angle / volumetric measurements as DICOM SR-TID-1500; ROI statistics; DICOM Presentation State; prior auto-fetch + side-by-side compare.
- **Dose tracking** — NEMA DICOM Dose Structured Report parsing; per-protocol deviation alerts; aggregate dashboards; EU EURATOM 2013/59 + US CMS QPP MIPS export.
- **Modality interoperability** — vendor-quirk handling for GE / Siemens / Philips / Canon (Toshiba) / Hologic / Hitachi / Mindray.
- **IHE Radiology profiles** — XDS-I.b, XCA-I, IRWF.b, SWF.b, REM, AIW-I, PIR, PDQ, PIX, ATNA, Consistent Time, EUA, PWP.
- **Enterprise imaging beyond radiology** — cardiology (echo, cath, EP), ophthalmology (OCT, fundus, visual field), dermatology, pathology WSI (may split later), dental, surgical video.
- **Mammography tracking** — screening recall, BI-RADS audit (PPV / cancer detection rate / sensitivity / specificity), MQSA retention, FDA 21 CFR 900 conformance.
- **Nuclear medicine workflow** — SUV, Tumor/Background, Deauville (lymphoma), PERCIST (response), hybrid PET/CT and PET/MRI co-registration.
- **Interventional radiology** — pre-procedure imaging review, procedure documentation, hybrid OR planning, fluoroscopy dose tracking.
- **RIS integration** — order receipt from `emr`, scheduling, billing handoff to `cloud-billing`, report distribution back to `emr`.
- **Patient portal sharing** — consent-graph-gated DICOMweb token issuance, plain-language layperson summary, secure share link, FHIR ImagingStudy export.

---

## What This µservice Refuses (Explicit Out-of-Scope)

- Clinical decision support outside imaging-modality scope — belongs to `emr` + `diagnostics`.
- Lab orders / specimen tracking — belongs to `diagnostics` lab portion.
- Provider directory / credentialing — belongs to `identity` + `cloud-iam`.
- Claims adjudication / RCM — belongs to `cloud-billing` + future RCM µservice.
- Genomics / molecular pathology beyond visual WSI — belongs to future `pathology` µservice when split.
- Patient scheduling outside imaging context — belongs to `calendar` + `emr`.

---

## Tenant-Class Behavior

Per ADR-0330 + ADR-0331:

- **demo_trial** — synthetic studies only; no PHI; 500-study cap; 7-modality cap; 100 AI inferences/day; voice + critical-results closed-loop + cross-VNA federation + patient portal disabled.
- **paid** — PHI-grade clinical use; sovereign-cell mandatory; HIPAA / GDPR / KR-MD / EU-MDR packs as applicable; per-seat + per-usage billing meters emitted.

---

## Deployment Contexts (6)

1. **aws-guest** — AI inference compute fan-out (GPU-bound), with on-prem PACS reachable via VPN.
2. **oci-guest** — OCI Always Free for demo_trial; Oracle-aligned hospital systems for paid.
3. **on-prem** — most common deployment for hospital PACS in customer-controlled data centers.
4. **colo** — co-located hardware adjacent to modalities for sub-second image-pull SLO.
5. **oyatie-cloud-provider** — hosted radiology service bureau / teleradiology night-hawks.
6. **sovereign-cell** — mandatory for paid PHI with Cedar-enforced isolation per HIPAA / GDPR / KR-MD / EU-MDR packs.

OpenTofu modules per context in `iac/<context>/`. No hand-rolled deployments per `feedback_zero_handroll_opentofu_only_2026_05_20`.

---

## OS Support Matrix (Tier-1, 13 OSes)

Per `feedback_os_support_matrix_2026_05_20`. See `supported-oses.json` for the canonical list and CI-lane mapping. Tier-1: Talos Linux, RHEL 9, Oracle Linux 9, SUSE Linux Enterprise 15, Ubuntu 22.04 LTS, Ubuntu 24.04 LTS, Debian 12, Rocky Linux 9, AlmaLinux 9, CentOS Stream 9, Amazon Linux 2023, Flatcar Stable, Photon OS 5. Tier-2 archs: linux/ppc64le, linux/s390x. Developer workstations: darwin/arm64 (Apple Silicon M5+) only.

---

## Performance Targets (Preserved + New)

- DICOM C-STORE: ≥10,250 instances/min sustained per pod (preserved from `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md`).
- Image pull p95: < 1 second.
- Multi-GB study load p95: < 5 seconds.
- Hanging-protocol apply p95: < 150ms.
- AI inference dispatch p95: < 500ms.
- Critical-result notification p99: < 30 seconds.
- MPR render p95: < 2 seconds.
- Prior auto-fetch p95: < 3 seconds.
- Voice recognition partial transcript p95: < 250ms.
- Structured-report save p95: < 800ms.
- VNA durability: 13 nines.
- PHI audit completeness: 100%.

See `slos/` for canonical OpenSLO definitions and `PRD.md` §6.1 for the full performance table.

---

## Compliance Packs

HIPAA-2024, GDPR, SOC-2, ISO-27001, EU-AI-Act, EU-MDR, KR-Medical-Devices, GxP, FDA 21 CFR Part 11, ACR Accreditation, MQSA, NEMA DICOM PS3 Conformance, IHE Radiology.

See `PRD.md` §6.5 and `ARCHITECTURE.md` §9 for pack architecture.

---

## Cross-µservice Dependencies

`cloud-iam` (Cedar gateway), `identity` (radiologist + tech + clinician identity), `audit-chain` (PHI access audit), `cloud-storage` (DICOM pixel blob), `observability` (OTel + SLO), `cloud-billing` (per-seat + per-usage), `emr` (order + report), `diagnostics` (lab-result correlation, post-reconciliation), `consent-graph` (patient + share-link consent), `compliance` (pack overlay), `workflow-engine` (critical-result + recall), `cloud-kms` (BYOK), `cloud-data` (relational state), `api-gateway` (edge surface).

---

## Architecture Highlights

- **DICOMweb-first** — primary substrate is DICOMweb over HTTP/3 + QUIC per ADR-0253. DIMSE is bridged but not the modern path.
- **Vendor-neutral AI marketplace** — single `AiVendorPort` adapter trait abstracts over ≥15 vendors. No "Edison-only" or "IntelliSpace-AI-Workflow-only" lock-in.
- **Cell-aware sovereign deployment** — HIPAA / GDPR / KR-MD / EU-MDR packs map to sovereign cells with Cedar-enforced isolation. No legacy vendor offers this primitive.
- **FHIR + DICOM dual emission** — ImagingStudy / ImagingSelection / DiagnosticReport[imaging] / DocumentReference[CDA] alongside DICOM. Order-to-report flows in FHIR; pixel data flows in DICOM.
- **Enterprise-imaging native** — cardiology, ophthalmology, dermatology, dental, surgical, pathology WSI in scope from day one (not a bolt-on).
- **Hexagonal ports + adapters** — per ADR-0105 13-layer enum, inward-only dependency flow.
- **Shuffle-sharded cells** — per ADR-0248 Amazon cellular shape. Each tenant maps to a shuffled subset of pods, limiting blast radius.
- **Cloud Hypervisor + Kata Containers** — per ADR-0254 VM-grade isolation for PHI workloads.
- **Substance-bar discipline** — every claim has an ADR + PRD section + IP + Cedar policy + SLO + Cedar test + parity-matrix row per ADR-0212.

---

## Artifact Map

| Path | Purpose |
|------|---------|
| `manifest.json` | µservice manifest |
| `supported-oses.json` | Tier-1 OS support matrix |
| `PRD.md` | Product requirements |
| `ARCHITECTURE.md` | Architecture |
| `README.md` | This file |
| `competitor-parity-matrix.md` | ≥150 capability rows vs GE / Philips / Sectra |
| `REMEDIATION-NOTES-2026-05-21.md` | Split-from-diagnostics + supersession + reconciliation note |
| `contracts/openapi.yaml` | OpenAPI 3.2.0 for DICOMweb + FHIR |
| `contracts/asyncapi.yaml` | AsyncAPI 3.1.0 for imaging events |
| `contracts/proto/imaging.proto` | gRPC proto3 for inter-µservice |
| `contracts/proto/imaging-events.proto` | gRPC proto3 for event envelopes |
| `slos/*.openslo.yaml` | OpenSLO definitions (≥12) |
| `policies/*.cedar` | Cedar policy bundle (≥8) |
| `iac/aws-guest/` | OpenTofu aws-guest module |
| `iac/oci-guest/` | OpenTofu oci-guest module (incl. Always Free for demo) |
| `iac/on-prem/` | OpenTofu on-prem module |
| `iac/colo/` | OpenTofu colo module |
| `iac/oyatie-cloud-provider/` | OpenTofu oyatie-cloud-provider module |
| `iac/sovereign-cell/` | OpenTofu sovereign-cell module |
| `decisions/ADR-MS-001-dicomweb-substrate.md` | DICOMweb-first substrate ADR |
| `decisions/ADR-MS-002-ai-image-analysis-vendor-neutral.md` | Vendor-neutral AI marketplace ADR |
| `decisions/ADR-MS-003-vna-federation.md` | VNA federation + legacy migration ADR |
| `decisions/ADR-MS-004-enterprise-imaging-scope.md` | Enterprise imaging beyond radiology ADR |
| `implementation-plans/IP-001..IP-015` | Substantive implementation plans |

---

## Quickstart (Developer Workstation)

Per `feedback_rust_strict_only_no_python_2026_05_20`, all µservice code is Rust. Developer workstations are darwin/arm64 (Apple Silicon M5+) only.

```
# clone + build (from oyatie repo root)
cargo build -p oya-imaging-kernel
cargo build -p oya-imaging-dicomweb-api
cargo test  -p oya-imaging-kernel
```

Local PACS substrate runs on `oci-guest` Always Free profile or sovereign-cell-localdev profile. See `iac/oci-guest/always-free/README.md` (forthcoming Wave 16-imaging-substrate).

---

## Migration Path from Legacy PACS / VNA

See `ARCHITECTURE.md` §14. Per-vendor adapters cover GE EA, Philips ISyntax-VNA, Sectra VNA, Fujifilm Synapse VNA, Agfa Impax, Merge VNA. Phase 1 dual-write → Phase 2 backfill → Phase 3 read cutover → Phase 4 decommission.

---

## Risks & Mitigations

See `PRD.md` §16.

---

## Glossary

See `PRD.md` §17.

---

## Roadmap

See `PRD.md` §14.

---

## License + Authority

- Authority: ADR-0132 single-concern doctrine + user directive 2026-05-21.
- Substance-bar source: ADR-0212.
- Owner team: axis-imaging + council-clinical.
- Criticality: Tier 0.
- Reconciliation pending: Wave 15M follow-up to retire imaging-portions of bundled `diagnostics` µservice.

---

## Wave 15M-G Completion Note

Authored 2026-05-21 as the sole-owner imaging-µservice artifact. Supersedes the imaging portions of the concurrently-authored `diagnostics` µservice (bundled lab + imaging + pathology). No commits made. No scripting. No stamping (the anti-pattern under review). Writes restricted to `microservices/imaging/*`.

---

## Bounded Context Index (24)

The 24 bounded contexts owned by this µservice are documented in `PRD.md` §4 and mapped to crates in `ARCHITECTURE.md` §1. Quick index:

1. **DICOMSubstrate** — DIMSE + DICOMweb upper-layer protocols.
2. **PACS** — Picture Archiving + Communication System indexing.
3. **VNA** — Vendor-Neutral Archive blob store + federation.
4. **ImageAcquisition** — per-modality acquisition state machines.
5. **RadiologistWorklist** — dynamic prioritized worklist.
6. **HangingProtocol** — DICOM HP IOD + Oyatie extension matching engine.
7. **StructuredReport** — BI-RADS / LI-RADS / PI-RADS / TI-RADS / Lung-RADS / O-RADS / CAD-RADS / Bone-RADS / NI-RADS authoring.
8. **VoiceRecognition** — Nuance / M*Modal / in-house Whisper-medical streaming.
9. **CriticalResults** — closed-loop cascading escalation.
10. **PeerReview** — RadPeer-style blinded review.
11. **AIImageAnalysis** — vendor-neutral CADe/CADx fan-out.
12. **3DReconstruction** — server-side MPR / MIP / volume rendering.
13. **ImageAnnotation** — DICOM PR + SR-TID-1500 measurements.
14. **PriorComparison** — auto-fetch + cross-VNA federated.
15. **DoseTracking** — RDSR + EURATOM + CMS QPP MIPS export.
16. **ModalityInterop** — per-vendor quirk handling.
17. **IHEProfiles** — XDS-I.b / XCA-I / IRWF.b / SWF.b / REM / AIW-I / ATNA.
18. **DICOMweb** — WADO-RS / QIDO-RS / STOW-RS / UPS-RS.
19. **EnterpriseImaging** — cardiology / ophthalmology / dermatology / pathology / dental / surgical.
20. **MammographyTracking** — screening recall + BI-RADS audit + MQSA retention.
21. **NuclearMedicine** — SUV + Deauville + PERCIST.
22. **InterventionalRadiology** — cath-lab planning + hybrid OR.
23. **RISIntegration** — order receipt + report distribution.
24. **PatientPortalSharing** — consent-gated DICOMweb token issuance.

---

## Service Index (gRPC + REST + Events)

The µservice exposes the following service surfaces. See `contracts/proto/imaging.proto` for gRPC, `contracts/openapi.yaml` for REST + DICOMweb + FHIR, and `contracts/asyncapi.yaml` for events.

- **PacsService** — query/retrieve/store/delete-study (gRPC).
- **ModalityWorklistService** — MWL query + MPPS lifecycle (gRPC).
- **RadWorklistService** — radiologist worklist (gRPC).
- **HangingProtocolService** — match + apply (gRPC).
- **StructuredReportService** — start/save/sign (gRPC).
- **AiMarketplaceService** — dispatch + result + health + vendor list (gRPC).
- **CriticalResultService** — escalation + confirm (gRPC).
- **PeerReviewService** — queue + submit (gRPC).
- **DoseTrackingService** — aggregate + EURATOM export + deviation query (gRPC).
- **MammographyTrackingService** — recall queue + BI-RADS audit (gRPC).
- **PatientPortalService** — issue/revoke token + layperson summary (gRPC).
- **VnaFederationService** — pull from legacy + federated query (gRPC).
- **ModalityAdminService** — list/register/deregister modality (gRPC).
- **DICOMweb endpoints** — WADO-RS / QIDO-RS / STOW-RS / UPS-RS (REST/HTTP/3).
- **FHIR endpoints** — ImagingStudy / ImagingSelection / DiagnosticReport / DocumentReference (REST/HTTP/3).
- **Events** — 15 AsyncAPI channels (study.received, study.acquired, study.read, report.signed, critical-result.communicated, peer-review.completed, ai.inference-dispatched, ai.inference-result, ai.drift-detected, dose.deviation-detected, mammography.recall-issued, vna.federation-pulled, patient-portal.access-granted, mpps.state-changed, hanging-protocol.applied).

---

## Modality Vendor Quirks Coverage

The µservice ships with vendor-quirk profiles for:

- **GE** Advantage Workstation private tags (CT, MR, US, NM, PET).
- **Siemens** syngo cardiac gating + magnetom MRI private tags.
- **Philips** IntelliSpace + Vue private tags.
- **Canon (formerly Toshiba)** Aquilion / Vantage / Aplio private tags.
- **Hologic** Selenia mammography + DBT tomosynthesis private tags.
- **Hitachi** MRI Echelon / Oasis private tags.
- **Mindray** ultrasound private tags.
- **Fujifilm** Synapse + CR/DR private tags.
- **Agfa** CR/DR private tags.

Each vendor profile carries a regression-test corpus of anonymized real-world studies. See `ARCHITECTURE.md` §4.3.

---

## See Also

- `PRD.md` — full product requirements (≥800 lines).
- `ARCHITECTURE.md` — full architecture (≥600 lines).
- `competitor-parity-matrix.md` — 200+ row parity matrix.
- `REMEDIATION-NOTES-2026-05-21.md` — split origin + supersession + reconciliation note.
- `contracts/` — OpenAPI / AsyncAPI / proto.
- `slos/` — 13 OpenSLO definitions.
- `policies/` — 9 Cedar policies.
- `iac/` — 6 OpenTofu modules per deployment context.
- `decisions/` — 4 ADR-MS records.
- `implementation-plans/` — 15 substantive IPs.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
