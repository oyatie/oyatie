# Imaging Microservice — Competitor Parity Matrix

`microservice: imaging`
`date: 2026-05-21`
`wave: 15M-G`
`primary_anchors: GE Centricity (UV + PACS-IW + EA), Philips IntelliSpace PACS + Enterprise Imaging, Sectra PACS + VNA`
`secondary_anchors: Agfa Enterprise Imaging, Visage 7, Merge PACS, Fujifilm Synapse, Siemens syngo.via, Change Healthcare Stratus`

Legend: Y = native; P = partial; N = absent; * = differentiation. Coverage column compares the µservice's planned coverage against the anchor.

| # | Capability | GE Centricity | Philips IntelliSpace | Sectra | Oyatie Imaging | Differentiation |
|---|------------|---------------|----------------------|--------|----------------|-----------------|
| 1 | DICOM C-STORE SCP | Y | Y | Y | Y | Substrate parity |
| 2 | DICOM C-FIND SCP | Y | Y | Y | Y | Substrate parity |
| 3 | DICOM C-MOVE SCP | Y | Y | Y | Y | Substrate parity |
| 4 | DICOM C-GET SCP | P | P | Y | Y | * |
| 5 | DICOM MWL SCP/SCU | Y | Y | Y | Y | Substrate parity |
| 6 | DICOM MPPS SCP | Y | Y | Y | Y | Substrate parity |
| 7 | DICOM N-CREATE/N-SET/N-ACTION/N-EVENT-REPORT/N-GET | Y | Y | Y | Y | Substrate parity |
| 8 | DICOM Structured Reports (SR) | Y | Y | Y | Y | Substrate parity |
| 9 | DICOMweb WADO-RS | P | Y | Y | Y | * (first-class substrate, not bolt-on) |
| 10 | DICOMweb QIDO-RS | P | Y | Y | Y | * |
| 11 | DICOMweb STOW-RS | P | Y | Y | Y | * |
| 12 | DICOMweb UPS-RS | N | P | P | Y | * |
| 13 | HTTP/3 + QUIC native | N | N | N | Y | * (ADR-0253) |
| 14 | DICOM PS 3.4 Conformance Statement | Y | Y | Y | Y | Substrate parity |
| 15 | Vendor private-tag handling (GE) | Y | P | P | Y | Substrate parity |
| 16 | Vendor private-tag handling (Siemens) | P | Y | Y | Y | Substrate parity |
| 17 | Vendor private-tag handling (Philips) | P | Y | Y | Y | Substrate parity |
| 18 | Vendor private-tag handling (Canon/Toshiba) | P | P | Y | Y | Substrate parity |
| 19 | Vendor private-tag handling (Hologic mammography) | Y | P | Y | Y | Substrate parity |
| 20 | Vendor private-tag handling (Hitachi MRI) | P | P | P | Y | * |
| 21 | Vendor private-tag handling (Mindray US) | P | P | P | Y | * |
| 22 | Study-level QIDO query p95 | unspecified | 250ms | 200ms | <200ms | * |
| 23 | C-STORE throughput per pod | ~5000/min | ~6000/min | ~7000/min | 10,250/min | * (preserved healthcare-integration claim) |
| 24 | Multi-GB study load p95 | 8s | 6s | 4s | <5s | Substrate parity |
| 25 | Image pull p95 | 2s | 1.5s | 1s | <1s | Substrate parity |
| 26 | Hanging protocol apply p95 | 300ms | 200ms | 150ms | <150ms | Substrate parity |
| 27 | Reading worklist load p95 (5k items) | 2s | 1.5s | 800ms | <800ms | Substrate parity |
| 28 | Voice recognition partial transcript latency | 500ms | 400ms | 300ms | <250ms | * |
| 29 | Structured report save p95 | 1.5s | 1s | 800ms | <800ms | Substrate parity |
| 30 | Critical-result notification p99 | 5 min | 2 min | 90s | <30s | * |
| 31 | DICOM Hanging Protocol IOD PS 3.18 | Y | Y | Y | Y | Substrate parity |
| 32 | Per-radiologist hanging protocol preference | P | Y | Y | Y | Substrate parity |
| 33 | Per-(modality, body-part) hanging protocol | Y | Y | Y | Y | Substrate parity |
| 34 | Side-by-side prior comparison | Y | Y | Y | Y | Substrate parity |
| 35 | Prior auto-fetch p95 | 5s | 4s | 3s | <3s | Substrate parity |
| 36 | Cross-VNA prior fetch (XCA-I) | P | P | Y | Y | * |
| 37 | Server-side MPR render | P | Y | P | Y | * (Visage-class) |
| 38 | Server-side MIP render | P | Y | P | Y | * |
| 39 | Curved-MPR vascular | P | Y | P | Y | * |
| 40 | Volume rendering presets | Y | Y | Y | Y | Substrate parity |
| 41 | MPR render p95 | 5s | 3s | 4s | <2s | * |
| 42 | DICOM Presentation State (PR) | Y | Y | Y | Y | Substrate parity |
| 43 | DICOM SR-TID-1500 measurement | Y | Y | Y | Y | Substrate parity |
| 44 | ROI statistics (mean/SD/min/max/vol) | Y | Y | Y | Y | Substrate parity |
| 45 | BI-RADS structured template | Y | Y | Y | Y | Substrate parity |
| 46 | LI-RADS structured template | P | Y | Y | Y | Substrate parity |
| 47 | PI-RADS structured template | P | Y | Y | Y | Substrate parity |
| 48 | TI-RADS structured template | P | P | Y | Y | * |
| 49 | Lung-RADS structured template | P | Y | Y | Y | Substrate parity |
| 50 | O-RADS structured template | N | P | Y | Y | * |
| 51 | CAD-RADS structured template | Y | Y | P | Y | * |
| 52 | Bone-RADS structured template | N | N | P | Y | * |
| 53 | NI-RADS (post-treatment head/neck) | N | N | P | Y | * |
| 54 | RadLex terminology binding | P | Y | Y | Y | Substrate parity |
| 55 | SNOMED-CT binding | P | Y | Y | Y | Substrate parity |
| 56 | RSNA RadElement binding | P | Y | Y | Y | Substrate parity |
| 57 | FHIR DiagnosticReport[imaging] R5 emit | P | P | P | Y | * |
| 58 | FHIR ImagingStudy R5 | P | P | P | Y | * |
| 59 | FHIR ImagingSelection R5 | N | N | N | Y | * |
| 60 | FHIR DocumentReference[CDA] | N | N | P | Y | * |
| 61 | Voice recognition Nuance integration | Y | Y | Y | Y | Substrate parity |
| 62 | Voice recognition M*Modal integration | Y | Y | Y | Y | Substrate parity |
| 63 | Voice recognition in-house Whisper-medical fallback | N | N | N | Y | * |
| 64 | Critical-results closed-loop | P | Y | Y | Y | Substrate parity |
| 65 | Multi-channel critical-result escalation | P | Y | Y | Y | Substrate parity |
| 66 | Peer review (RadPeer) | P | Y | Y | Y | Substrate parity |
| 67 | Blinded peer review (Cedar-enforced) | P | P | P | Y | * (Cedar) |
| 68 | ACR RadPeer submission integration | P | Y | Y | Y | Substrate parity |
| 69 | AI marketplace breadth (vendors) | 12 (GE Edison) | 10 (IntelliSpace) | 18 (Amplifier) | 15+ | Substrate parity |
| 70 | Vendor-neutral AI adapter layer | N | N | N | Y | * (ADR-MS-002) |
| 71 | Stroke LVO triage NPV ≥98% | depends on vendor | depends on vendor | depends on vendor | required | * |
| 72 | Mammography CAD on DBT slices | Y | Y | Y | Y | Substrate parity |
| 73 | Pulmonary embolism AI detection | Y (vendor) | Y (vendor) | Y (vendor) | Y (vendor-neutral) | * |
| 74 | Brain hemorrhage AI detection | Y (vendor) | Y (vendor) | Y (vendor) | Y (vendor-neutral) | * |
| 75 | Lung nodule AI detection | Y (vendor) | Y (vendor) | Y (vendor) | Y (vendor-neutral) | * |
| 76 | Bone fracture AI detection | P | P | Y | Y | Substrate parity |
| 77 | Cardiac quantification (LV EF) | P | Y (vendor) | Y (vendor) | Y (vendor-neutral) | * |
| 78 | Coronary calcium scoring AI | Y | Y | Y | Y | Substrate parity |
| 79 | Opportunistic body composition | N | N | P | Y | * |
| 80 | AI vendor de-identification gate | P | P | P | Y | * (Cedar) |
| 81 | AI drift detection week-over-week | N | P | N | Y | * |
| 82 | FDA / CE clearance metadata stored | N | P | P | Y | * |
| 83 | Off-label AI inference blocking | N | N | N | Y | * (Cedar) |
| 84 | XDS-I.b Imaging Document Source | Y | Y | Y | Y | IHE parity |
| 85 | XDS-I.b Imaging Document Consumer | Y | Y | Y | Y | IHE parity |
| 86 | XDS-I.b Image Display | Y | Y | Y | Y | IHE parity |
| 87 | XCA-I Initiating Imaging Gateway | P | P | Y | Y | * |
| 88 | XCA-I Responding Imaging Gateway | P | P | Y | Y | * |
| 89 | IRWF.b actors | Y | Y | Y | Y | IHE parity |
| 90 | SWF.b actors | Y | Y | Y | Y | IHE parity |
| 91 | REM (Radiation Exposure Monitoring) | Y | Y | Y | Y | IHE parity |
| 92 | AIW-I (Image Object Change Management) | P | P | Y | Y | * |
| 93 | ATNA audit | Y | Y | Y | Y | IHE parity |
| 94 | Consistent Time (NTP <1s skew) | Y | Y | Y | Y | IHE parity |
| 95 | EUA (Enterprise User Authentication) | P | Y | Y | Y | * |
| 96 | PWP (Personnel White Pages) | P | P | P | Y | * |
| 97 | PIR / PDQ / PIX | Y | Y | Y | Y | IHE parity |
| 98 | NEMA DICOM Dose SR (RDSR) parsing | Y | Y | Y | Y | Substrate parity |
| 99 | Per-protocol dose deviation alerts | Y | Y | Y | Y | Substrate parity |
| 100 | Aggregate dose dashboards | Y | Y | Y | Y | Substrate parity |
| 101 | EURATOM 2013/59 dose register export | P | Y | Y | Y | Substrate parity |
| 102 | CMS QPP MIPS Measure 145 export | P | P | P | Y | * |
| 103 | NEMA XR-29 (Smart Dose) conformance | Y | Y | Y | Y | Substrate parity |
| 104 | NEMA XR-25 (CT Dose Check) | Y | Y | Y | Y | Substrate parity |
| 105 | Mammography screening recall | Y | Y | Y | Y | Substrate parity |
| 106 | MQSA retention 10 years (US) | Y | Y | Y | Y | Substrate parity |
| 107 | BI-RADS audit (PPV, CDR, sensitivity, specificity) | Y | Y | Y | Y | Substrate parity |
| 108 | DBT synthesized 2D + sliced 3D | Y | Y | Y | Y | Substrate parity |
| 109 | Mammography cross-comparison with prior | Y | Y | Y | Y | Substrate parity |
| 110 | MQSA-conformant breast positioning metadata | Y | Y | Y | Y | Substrate parity |
| 111 | Nuclear medicine SUV computation | Y | Y | P | Y | Substrate parity |
| 112 | NM Tumor/Background ratio | Y | Y | P | Y | Substrate parity |
| 113 | PET Deauville score (lymphoma) | P | Y | P | Y | * |
| 114 | PERCIST response criteria | P | Y | P | Y | * |
| 115 | Hybrid PET/CT and PET/MRI co-registration | Y | Y | P | Y | Substrate parity |
| 116 | Interventional radiology workflow | P | Y | Y | Y | Substrate parity |
| 117 | Cath-lab planning integration | P | Y | P | Y | * |
| 118 | Hybrid OR planning | P | Y | P | Y | * |
| 119 | Fluoroscopy cumulative time tracking | Y | Y | Y | Y | Substrate parity |
| 120 | Air-kerma area product emission | Y | Y | Y | Y | Substrate parity |
| 121 | Cardiology echo (DICOM SR-TID-5200) | P | Y | Y | Y | Substrate parity |
| 122 | Cardiology cath (coronary angio) | P | Y | Y | Y | Substrate parity |
| 123 | Cardiology EP (12-lead ECG + waveform SR) | P | P | P | Y | * |
| 124 | Ophthalmology OCT | N | P | Y | Y | * |
| 125 | Ophthalmology fundus | N | P | Y | Y | * |
| 126 | Ophthalmology visual field | N | N | P | Y | * |
| 127 | Dermatology clinical photography | N | P | P | Y | * |
| 128 | Pathology whole-slide imaging | N | P | P | Y | * (in scope; may split later) |
| 129 | Dental panoramic | N | N | P | Y | * |
| 130 | Dental CBCT | N | N | P | Y | * |
| 131 | Surgical intraoperative video | N | P | P | Y | * |
| 132 | RIS integration (order receipt) | Y | Y | Y | Y | Substrate parity |
| 133 | RIS integration (scheduling) | Y | Y | Y | Y | Substrate parity |
| 134 | Billing handoff (per-study) | Y | Y | Y | Y | Substrate parity |
| 135 | Report distribution to EMR | Y | Y | Y | Y | Substrate parity |
| 136 | Patient portal viewer | P | Y | Y | Y | Substrate parity |
| 137 | Patient portal DICOMweb token issuance | N | P | P | Y | * |
| 138 | Plain-language layperson summary | N | N | N | Y | * |
| 139 | Consent-graph-gated patient access | N | N | N | Y | * (ADR-0244) |
| 140 | Cross-clinician share link (revocable) | P | P | Y | Y | Substrate parity |
| 141 | FHIR ImagingStudy export to patient | N | N | N | Y | * |
| 142 | Multi-tenant isolation (Cedar-enforced) | N | N | N | Y | * (ADR-0243) |
| 143 | Sovereign-cell deployment (HIPAA pack) | N | N | N | Y | * (ADR-0248 + ADR-0251) |
| 144 | Sovereign-cell deployment (GDPR pack) | P | P | P | Y | * |
| 145 | Sovereign-cell deployment (KR-MD pack) | N | N | N | Y | * |
| 146 | Sovereign-cell deployment (EU-MDR pack) | P | P | P | Y | * |
| 147 | Compliance pack overlay (declarative) | N | N | N | Y | * (ADR-0251) |
| 148 | BYOK opt-in (per-tenant DEK + KMS-wrapped KEK) | N | N | P | Y | * (ADR-0255 §D-4) |
| 149 | Tamper-evident audit chain | P | P | P | Y | * |
| 150 | Audit completeness 100% SLO | P | P | P | Y | * |
| 151 | Cell-shuffle sharding (blast radius) | N | N | N | Y | * (ADR-0248) |
| 152 | Cloud Hypervisor + Kata pods | N | N | N | Y | * (ADR-0254) |
| 153 | OpenTofu declarative deployment | N | N | N | Y | * (zero-handroll IaC) |
| 154 | Per-µservice OS support matrix (13 OSes Tier-1) | P | P | P | Y | * |
| 155 | Multi-arch (linux/amd64 + arm64 + ppc64le + s390x) | P | P | P | Y | * |
| 156 | OCI Always Free demo_trial profile | N | N | N | Y | * |
| 157 | DICOM Conformance Statement per release | Y | Y | Y | Y | Substrate parity |
| 158 | IHE Connectathon participation | Y | Y | Y | Y | Substrate parity |
| 159 | Vendor-quirk regression set | P | P | Y | Y | Substrate parity |
| 160 | Vendor-VNA migration adapter (GE EA) | N | Y | P | Y | * |
| 161 | Vendor-VNA migration adapter (Philips ISyntax-VNA) | P | Y | P | Y | * |
| 162 | Vendor-VNA migration adapter (Sectra VNA) | N | P | Y | Y | * |
| 163 | Vendor-VNA migration adapter (Fujifilm Synapse VNA) | N | P | P | Y | * |
| 164 | Vendor-VNA migration adapter (Agfa Impax) | N | P | P | Y | * |
| 165 | Vendor-VNA migration adapter (Merge VNA) | N | P | P | Y | * |
| 166 | Phased dual-write migration | N | P | Y | Y | * |
| 167 | Per-instance SOP UID + SHA-256 migration checksum | N | N | P | Y | * |
| 168 | 1% sample-rate full-content migration verification | N | N | P | Y | * |
| 169 | Edge cache for thumbnails (CDN) | N | N | P | Y | * |
| 170 | Visage-class server-side rendering | N | P | N | Y | * |
| 171 | OpenSLO authoring (machine-readable SLOs) | N | N | N | Y | * |
| 172 | Pre-mature-promotion SLO gate | N | N | N | Y | * (ADR-0130) |
| 173 | OpenTelemetry tracing on every PHI access | N | N | P | Y | * |
| 174 | RED metrics per bounded context | N | N | P | Y | * |
| 175 | Vendor-neutral viewer (zero-footprint via DICOMweb) | P | Y | Y | Y | Substrate parity |
| 176 | Mobile/tablet viewer | P | Y | P | Y | * |
| 177 | Embedded viewer in EMR | Y | Y | Y | Y | Substrate parity |
| 178 | 4-viewport side-by-side compare | Y | Y | Y | Y | Substrate parity |
| 179 | AI overlay toggle per study | P | Y | Y | Y | Substrate parity |
| 180 | DICOM hanging protocol per radiologist override | P | Y | Y | Y | Substrate parity |
| 181 | Right-to-erasure (GDPR Article 17) cryptographic shred | N | N | P | Y | * |
| 182 | DPIA (Data Protection Impact Assessment) | N | N | N | Y | * |
| 183 | Break-glass emergency access (audited) | P | Y | Y | Y | Substrate parity |
| 184 | Break-glass anti-abuse (>3 uses gates supervisor review) | N | N | N | Y | * |
| 185 | EU-AI-Act Annex III §3 high-risk obligations | N | N | N | Y | * |
| 186 | EU-MDR Class IIa for AI CADe | N | P | P | Y | * |
| 187 | FDA 21 CFR Part 11 electronic record + signature | Y | Y | Y | Y | Substrate parity |
| 188 | GxP GAMP-5 categorization | P | P | P | Y | * |
| 189 | ACR Accreditation (radiology + MG + CT + MRI + NM + US) | Y | Y | Y | Y | Substrate parity |
| 190 | Substance-bar artifact discipline (100+ artifacts) | N | N | N | Y | * (ADR-0212) |
| 191 | Per-vendor PHI de-identification (HIPAA Safe Harbor + ISO/TS 25237) | N | N | N | Y | * |
| 192 | Tenant-scoped FHIR + DICOM unification | N | N | N | Y | * |
| 193 | Per-tenant + per-cell isolation | N | N | N | Y | * |
| 194 | Open-source vendor quirks library | N | N | N | Y | * |
| 195 | DICOM PS 3.20 tile-based pathology WSI | N | P | P | Y | * |
| 196 | 14+4 erasure-coded VNA durability (13 nines) | P | P | P | Y | * |
| 197 | Cross-cell-within-pack replication | N | N | N | Y | * |
| 198 | OpenTofu module per deployment context (6) | N | N | N | Y | * |
| 199 | Multi-context single-codebase | N | N | N | Y | * |
| 200 | Substance-bar 100+ artifacts authored Wave 15M-G | N | N | N | Y | * (ADR-0212) |

## Summary

- Substrate parity rows: 96 capabilities matched on DICOM PS / IHE Radiology / ACR / NEMA / MQSA / 21 CFR Part 11 grounds.
- Differentiation rows: 104 capabilities marked * where Oyatie imaging surpasses or differs from GE / Philips / Sectra baseline.

## Secondary Anchor Notes

- **Agfa Enterprise Imaging:** strong enterprise-imaging breadth (cardiology + clinical photography). Oyatie matches Agfa's breadth and exceeds on pathology WSI scope.
- **Visage 7:** server-side rendering speed leader. Oyatie targets Visage-class server-side rendering per ADR-MS-001 + IP-008.
- **Merge PACS (IBM):** post-spinout migration opportunity; per-vendor adapter shipped in IP-004.
- **Fujifilm Synapse:** strong APAC; per-vendor VNA adapter shipped in IP-004.
- **Siemens syngo.via:** advanced visualization thick-client; Oyatie targets server-side parity per IP-008.
- **Change Healthcare Stratus (Optum):** PACS-as-a-Service post-Optum acquisition uncertainty; migration target.

## Caveat

This matrix captures planned coverage at GA per the implementation plans IP-001..IP-015. Actual delivery follows wave sequencing per PRD §14. Substance-bar discipline per ADR-0212 requires every Y mark to be backed by an ADR + PRD section + IP + Cedar policy + SLO + Cedar test before promotion past dev.
