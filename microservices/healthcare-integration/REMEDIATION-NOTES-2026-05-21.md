---
doc_class: Remediation-Notes
microservice: healthcare-integration
remediation_date: 2026-05-21
remediation_wave: Wave 15M-A (Healthcare Domain Decomposition Authorization)
remediation_owner: council-architecture + axis-healthcare-integration
governing_adr: ADR-0332-healthcare-domain-decomposition.md
companion_plan: /Users/jasonlee/oyatie/.omc/plans/healthcare-decomposition-plan-2026-05-21.md
related_adrs:
  - ADR-0131 (per-microservice flat layout)
  - ADR-0132 (no-grouping forward policy)
  - ADR-0138 (six-path deprecation pattern)
  - ADR-0145 (inter-microservice communication reform)
  - ADR-0244 (tenant primitive)
  - ADR-0251 (compliance pack + cell certification levels)
  - ADR-0263 (audit emission)
  - ADR-0328 (substance bar canonical sequence)
  - ADR-0332 (healthcare domain decomposition — governing this remediation)
predecessor_audits:
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/coherence-audit-2026-05-20.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/feature-parity-matrix-2026-05-20.md
halt_condition: clean
---

# Remediation Notes — healthcare-integration Scope Narrowing

## §1 Purpose

This document records the scope-narrowing remediation required of the
existing `microservices/healthcare-integration/` µservice as a
consequence of ADR-0332 (Healthcare Domain Decomposition). It tells
later authoring waves and remediation agents exactly which scope the
µservice retains, which capabilities migrate OUT to the new domain
µservices, and where the substantive content currently in this folder
lands after the migration.

The remediation is NOT executed in Wave 15M-A. Wave 15M-A authors the
three foundational deliverables (ADR-0332, the Wave 15M plan, and this
notes file). The actual scope-narrow happens in Wave 15M-B alongside
the new µservice scaffolds. This document is the canonical reference
for that 15M-B work.

## §2 Scope-narrowing summary

### §2.1 Before this remediation (Wave-3-I authoring as of 2026-05-20)

The healthcare-integration µservice was authored under Wave-3-I as the
single owner of:

- 215 features across 14 domains (per the coherence audit at
  `coherence-audit-2026-05-20.md`).
- 5 bounded contexts: `patient-record`, `fhir-resource`, `hl7-message`,
  `referral`, `clinical-consent`.
- 6 capabilities: `fhir-read`, `hl7-route`, `break-glass-authorize`,
  `consent-sync`, `ehr-provenance-seal`, `patient-match-review`.
- 30 IPs (IP-001..IP-030).
- 5 named bounded-context-class clinical concerns merged with the
  integration-substrate concern.

That spread covered patient records (system-of-record), lab/imaging
workflow, ED triage, pharmacy ePrescribe, continuous vital-sign
telemetry, clinical decision-support, care plans + population health,
AND the FHIR/HL7v2/DICOM broker — under one PRD, one manifest, one
SLO bundle.

This violates ADR-0131 (per-microservice flat layout single-concern
doctrine) and ADR-0132 (no-grouping forward policy). Section §3.4 of the
2026-05-20 coherence audit ("Canonical Direction Alignment") records
the verdict `REVISE` with two P0 findings.

### §2.2 After this remediation (Wave 15M-B target state)

The healthcare-integration µservice narrows to FHIR/HL7v2/DICOM
integration substrate concern only. It becomes a broker. It does not
own clinical state of record.

Bounded contexts after narrow: 3 broker-scoped contexts:

1. `fhir-broker` — FHIR R4 + R5 inbound/outbound, Bundle transaction,
   Subscription, Implementation Guide validation
2. `hl7v2-broker` — HL7v2.3..v2.7+ inbound/outbound, MLLP transport,
   ACK/NAK generation, segment validation
3. `dicom-broker` — DIMSE (C-STORE / C-FIND / C-MOVE) + DICOMweb
   (QIDO-RS / STOW-RS / WADO-RS), modality worklist, MPPS

Capabilities after narrow (broker-scoped):

- `fhir-read` — RETAINED (broker concern)
- `hl7-route` — RETAINED (broker concern)
- `ehr-provenance-seal` — RETAINED (integration egress concern)
- `consent-sync` — narrowed to `consent-segmentation-at-broker-
  boundary` (canonical consent state lives in consent-graph µservice;
  the broker only enforces segmentation at boundary)
- `break-glass-authorize` — narrowed to `break-glass-relay` (the
  canonical break-glass policy lives in the domain µservice — emr /
  emergency / pharmacy; the broker relays the justification metadata)
- `patient-match-review` — narrowed to `mpi-substrate` (MPI substrate
  is integration concern; canonical patient identity is emr's
  concern)

Top-3 counterpart set: PRESERVED from Wave-3-I — `[Redox, Mirth
Connect, Health Gorilla]`. The Wave 4-rolling audit identified these
as the canonical integration counterparts and recorded the prior
counterpart set (Epic / Cerner / Allscripts / Veeva — EHR vendors)
as a P1 finding (F-PARITY-COUNTERPART-MISMATCH). The post-narrow
µservice carries the corrected counterpart set forward.

## §3 Capabilities migrated OUT (which 14 domains went where)

The 14 domains enumerated in the 2026-05-20 audit and the 215 features
in the feature-parity matrix migrate per the ADR-0332 §H table.
Summary per domain:

### §3.1 Domain A — FHIR API surface (29 features)

| Feature subset | Migrates to |
|---|---|
| Broker-side FHIR R4/R5 server + Bundle transaction + Subscription substrate (A1..A14) | RETAINED in healthcare-integration as `fhir-broker` |
| US Core 6.1.0 (R4) Patient resource authoring (A15) | emr |
| US Core 7.0.0 (R5) Patient resource authoring (A16) | emr |
| International Patient Summary (IPS-UV) IG authoring (A17) | emr |
| Da Vinci PAS / DTR / CRD authoring (A18..A20) | care-management + clinical-decision-support (joint) |
| CARIN BB authoring (A21) | care-management |
| FHIR Provider Directory authoring (A22) | emr + cloud-iam (joint) |
| FHIR consent segmentation (A23) | consent-graph + healthcare-integration (broker boundary only) |
| FHIR Patient $match operation (A24) | emr (canonical match) + healthcare-integration (MPI substrate) |
| FHIR DocumentReference + Composition (A25) | emr |
| C-CDA conversion (A26) | emr + healthcare-integration (broker conversion) |
| CDS Hooks 2.0 (A27) | clinical-decision-support |
| FHIR Capabilities Statement (A28) | RETAINED in healthcare-integration (broker-side) |
| $reindex + $expunge admin operations (A29) | RETAINED in healthcare-integration |

### §3.2 Domain B — HL7 v2/v3 (29 features)

| Feature subset | Migrates to |
|---|---|
| HL7v2.3..v2.7+ routing substrate (B1..B7) | RETAINED in healthcare-integration as `hl7v2-broker` |
| HL7v2.8 / v2.9 ballot declaration (B8..B9) | RETAINED |
| ADT (Admission / Discharge / Transfer) semantic content (B10) | emr (encounter lifecycle); broker routing RETAINED |
| ORM (Order Message) semantic content (B11) | diagnostics (lab/imaging order); broker routing RETAINED |
| ORU (Observation Result) semantic content (B12) | diagnostics (lab result); broker routing RETAINED |
| MDM (Medical Document Management) semantic content (B13) | emr (clinical note); broker routing RETAINED |
| SIU (Scheduling) semantic content (B14) | emr (encounter scheduling); broker routing RETAINED |
| BAR (Billing Account) semantic content (B15) | cloud-billing; broker routing RETAINED |
| DFT (Detailed Financial Transaction) semantic content (B16) | cloud-billing; broker routing RETAINED |
| VXU (Unsolicited Vaccination Update) semantic content (B17) | emr (immunization); broker routing RETAINED |
| RDS (Pharmacy Encoded Order) semantic content (B18) | pharmacy; broker routing RETAINED |
| PPR (Patient Problem) semantic content (B19) | emr (problem list); broker routing RETAINED |
| QBP / RSP (Query By Parameter / Response) (B20) | emr + healthcare-integration (PDQ broker) |
| ACK / NAK response generation (B21) | RETAINED |
| MLLP transport (B22) | RETAINED |
| HL7 batch (BHS / BTS) (B23) | RETAINED |
| HL7v3 CDA + RIM messages (B24) | emr (clinical document); broker routing RETAINED |
| HL7v2 segment-level validation (B25) | RETAINED (broker validation) |
| HL7v2 → FHIR mapping (B26) | RETAINED (broker concern) |
| NCPDP SCRIPT 2017 (B27) | pharmacy |
| X12 270/271 eligibility (B28) | cloud-billing (payer integration) |
| X12 837 / 835 claims + remittance (B29) | cloud-billing |

### §3.3 Domain C — DICOM imaging (25 features)

| Feature subset | Migrates to |
|---|---|
| DICOM C-STORE / C-FIND / C-MOVE / C-GET (C1..C4) | RETAINED in healthcare-integration as `dicom-broker` |
| DICOMweb QIDO-RS / STOW-RS / WADO-RS / WADO-URI (C5..C8) | RETAINED |
| DICOM SOP classes (C9..C18) | RETAINED (storage); diagnostics displays / manages |
| DICOM Modality Worklist (MWL) (C19) | RETAINED + diagnostics (consumer) |
| DICOM Modality Performed Procedure Step (MPPS) (C20) | RETAINED + diagnostics |
| DICOM TLS (BCP 195) (C21) | RETAINED |
| DICOM PS3.15 §B.1.1 secure transport (C22) | RETAINED |
| DICOM de-identification (PS3.15 §E) (C23) | RETAINED + diagnostics (de-id workflow) |
| DICOMweb auth (OAuth 2.0 + SMART) (C24) | RETAINED + identity µservice |
| DICOM AI inference workflow (C25) | clinical-decision-support (AI inference); RETAINED (broker storage) |

### §3.4 Domain D — EHR connectivity per major system (23 features)

| Feature subset | Migrates to |
|---|---|
| Epic connector (D1..D3, including Care Everywhere XCA) | RETAINED in healthcare-integration as `ehr-connector` |
| Oracle Cerner Millennium connector (D4..D5) | RETAINED |
| Allscripts Veradigm Sunrise connector (D6..D7) | RETAINED |
| AthenaHealth connector (D8) | RETAINED |
| eClinicalWorks connector (D9..D10) | RETAINED |
| MEDITECH Expanse connector (D11) | RETAINED |
| NextGen Healthcare connector (D12) | RETAINED |
| Greenway / Practice Fusion / DrChrono connectors (D13..D15) | RETAINED |
| Veeva Vault CRM (life sciences) connector (D16) | RETAINED |
| Salesforce Health Cloud connector (D17) | RETAINED |
| Microsoft Dynamics 365 Healthcare connector (D18) | RETAINED |
| Surescripts (D19) | pharmacy (canonical ePrescribe); RETAINED (broker leg) |
| CommonWell Health Alliance (TEFCA) (D20) | RETAINED |
| CareQuality (TEFCA) (D21) | RETAINED |
| eHealth Exchange (D22) | RETAINED |
| DirectTrust (Direct messaging) (D23) | RETAINED |

### §3.5 Domain E — Lab / imaging / pharmacy network integrations (12 features)

| Feature subset | Migrates to |
|---|---|
| LabCorp / Quest / regional reference labs (E1..E3) | diagnostics (canonical); RETAINED (broker leg + marketplace DealSet) |
| Hospital lab information systems (E4) | diagnostics; RETAINED (broker leg) |
| Imaging centers (RadNet / Akumin / Solis) (E5) | diagnostics + healthcare-integration (joint) |
| Pathology (Path AI / Proscia / Sectra) (E6) | diagnostics |
| Cardiology (Philips IntelliSpace) (E7) | diagnostics + patient-monitoring (joint) |
| Pharmacy benefit managers (PBMs) (E8) | pharmacy |
| Surescripts (E9) | pharmacy + RETAINED (broker leg) |
| DEA EPCS (E10) | pharmacy |
| Immunization Information Systems (E11) | emr |
| Public health reporting (E12) | future `public-health` µservice (Open Question §J of ADR-0332) |

### §3.6 Domain F — Patient matching / MPI (10 features)

| Feature subset | Migrates to |
|---|---|
| Deterministic + probabilistic matching (F1..F2) | RETAINED in healthcare-integration as MPI substrate; emr authors the canonical patient identity model |
| Tunable thresholds (F3) | RETAINED (MPI substrate) |
| Match adjudication review queue (F4) | emr (canonical adjudication on patient record); broker-level review queue RETAINED |
| FHIR Patient $match (F5) | emr + RETAINED |
| PIX / PDQ (F6) | RETAINED (IHE substrate) |
| MPI duplicate detection (F7) | RETAINED |
| Match audit trail (F8) | RETAINED + emr (canonical) |
| Cross-tenant match prevention (F9) | RETAINED (Cedar gate at broker) |
| Match scoring per cell certification level (F10) | RETAINED |

### §3.7 Domain G — Clinical data normalization (12 features)

| Feature subset | Migrates to |
|---|---|
| HL7v2 → FHIR R4/R5 mapping (G1..G2) | RETAINED in healthcare-integration |
| C-CDA → FHIR mapping (G3) | RETAINED |
| Vendor-specific code translation (G4) | RETAINED |
| Unit normalization (UCUM) (G5) | RETAINED |
| Reference-range normalization (G6) | diagnostics (canonical reference ranges); RETAINED (broker-side translation) |
| Result interpretation flag normalization (G7) | diagnostics; RETAINED |
| Allergy normalization (RxNorm + SNOMED) (G8) | emr (canonical allergy); RETAINED |
| Problem list normalization (G9) | emr (canonical problem); RETAINED |
| Medication normalization (G10) | pharmacy (canonical med list); RETAINED |
| Lab normalization (LOINC) (G11) | diagnostics; RETAINED |
| Procedure normalization (CPT + HCPCS) (G12) | emr / diagnostics / cloud-billing; RETAINED |

### §3.8 Domain H — IHE profile support (15 features)

| Feature subset | Migrates to |
|---|---|
| IHE PIX / PDQ / PDQm / PIXm (H1..H4) | RETAINED in healthcare-integration |
| IHE XDS.b / XDR / XCA / MHD (H5..H8) | RETAINED |
| IHE ATNA (H9) | RETAINED + audit-chain µservice (canonical) |
| IHE CT (Consistent Time) (H10) | RETAINED |
| IHE BPPC / APPC (H11..H12) | consent-graph (canonical); RETAINED (broker-side enforcement) |
| IHE SeR (H13) | emr (clinical document) + RETAINED |
| IHE LAB-3 (H14) | diagnostics + RETAINED |
| IHE RAD-TF (H15) | diagnostics + RETAINED |

### §3.9 Domain I — Terminology services (18 features)

| Feature subset | Migrates to |
|---|---|
| SNOMED CT / LOINC / ICD-10 / CPT / HCPCS / RxNorm / NDC / UCUM (I1..I10) | RETAINED in healthcare-integration as terminology substrate |
| CVX / MVX (I11..I12) | RETAINED + emr (immunization consumer) |
| FHIR ValueSet $expand / ConceptMap $translate / CodeSystem $lookup / ValueSet $validate-code (I13..I16) | RETAINED |
| Bring-your-own-terminology pack (I17) | RETAINED |
| Terminology pack version pinning per tenant (I18) | RETAINED |

### §3.10 Domain J — Longitudinal record assembly (11 features)

| Feature subset | Migrates to |
|---|---|
| Patient-centric longitudinal record (FHIR $everything) (J1) | emr (canonical longitudinal record assembly) |
| Cross-tenant patient assembly (J2) | emr + consent-graph + RETAINED (broker leg) |
| Longitudinal medication history (J3) | pharmacy + emr (read) |
| Longitudinal problem list (J4) | emr |
| Longitudinal lab history (J5) | diagnostics + emr (read) |
| Longitudinal imaging history (J6) | diagnostics + RETAINED (DICOM broker storage) |
| Document timeline (J7) | emr |
| Encounter timeline (J8) | emr |
| Care plan + goal timeline (J9) | care-management |
| Allergy + intolerance timeline (J10) | emr |
| Provider history (J11) | emr + cloud-iam (joint) |

### §3.11 Domain K — Consent management (14 features)

| Feature subset | Migrates to |
|---|---|
| FHIR R5 Consent resource (K1) | consent-graph (canonical) + RETAINED (broker-side FHIR projection) |
| Consent grant / revoke workflow (K2..K3) | consent-graph |
| Purpose-of-use granularity (K4) | consent-graph + RETAINED (broker-side enforcement) |
| Time-bounded / provider-scoped / care-team-scoped / data-class-scoped consent (K5..K8) | consent-graph |
| Pediatric consent (K9) | consent-graph + emr (consumer) |
| Withdraw-as-easy-as-grant (K10) | consent-graph |
| Audit emission per consent transition (K11) | audit-chain + consent-graph |
| Cross-organization consent propagation (K12) | consent-graph + RETAINED (IHE BPPC/APPC broker leg) |
| Break-glass override of consent (K13) | emr + emergency + pharmacy (canonical; per-domain); RETAINED (broker-side relay) |
| Break-glass justification review (K14) | emr + emergency + pharmacy + RETAINED (broker relay) |

### §3.12 Domain L — HIPAA pack readiness (32 features)

All 32 L-row features migrate to `microservices/governance/packs/
HIPAA-2024/v1/` per ADR-0251 + ADR-0332 §E. Each healthcare-domain
µservice (all 8) inherits the pack uniformly.

| Feature subset | Migrates to |
|---|---|
| BAA execution (L1) | HIPAA-2024 pack |
| PHI data class registration (L2) | HIPAA-2024 pack + data-class-registry per ADR-0099 |
| Encryption at rest / in transit / FIPS 140-2 (L3..L5) | HIPAA-2024 pack iac + all 8 healthcare-domain µservices |
| Audit trail (L6..L7) | audit-chain µservice + HIPAA-2024 pack retention policy |
| Access controls / unique user ID / automatic logoff (L8..L10) | identity µservice + HIPAA-2024 pack |
| Data localization (CONUS) (L11) | cell-certification-level matrix + HIPAA-2024 pack |
| Breach notification (60-day rule) (L12..L13) | HIPAA-2024 pack breach-notification-workflow |
| HHS OCR audit-protocol evidence (L14) | audit-chain + HIPAA-2024 pack |
| HIPAA Security Risk Analysis (L15) | threat-model.md (per µservice) + HIPAA-2024 pack |
| Workforce training (L16) | HIPAA-2024 pack training-acknowledgement-workflow |
| Subcontractor flow-down (L17) | HIPAA-2024 pack BAA cascade |
| Return / destroy of PHI on tenant uninstall (L18) | HIPAA-2024 pack + each domain µservice's offboarding runbook |
| Sanction policy / activity review (L19..L20) | HIPAA-2024 pack |
| Assigned Security Responsibility / Workforce / Contingency / Evaluation (L21..L24) | HIPAA-2024 pack |
| Business associate contracts (L25) | HIPAA-2024 pack |
| Facility / Workstation / Device controls (L26..L28) | out-of-scope-deployer / cloud-provider BAA |
| Audit / Integrity / Authentication / Transmission (L29..L32) | per ADR-0263 audit-chain + identity µservice + ADR-0188 passkey/webauthn + TLS 1.3 + ECH per ADR-0253 |

### §3.13 Domain M — Compliance and sovereignty (12 features)

| Feature subset | Migrates to |
|---|---|
| SOC 2 Type II / ISO 27001 (M1..M2) | governance µservice (existing pack registry) |
| HIPAA-2024 pack (M3) | HIPAA-2024 pack |
| EU GDPR Article 9 special-category data (M4) | EU-GDPR pack (existing) + all healthcare-domain µservices |
| KR 의료법 (M5) | KR-MED-LAW-v2024 pack (new per Wave 15M-G) |
| EU MDR (M6) | EU-MDR-v2024 pack (new) |
| FDA 21 CFR Part 11 (M7) | FDA-SaMD pack + GxP (existing) |
| 42 CFR Part 2 (M8) | consent-graph + emr + emergency (joint) |
| TEFCA (M9) | TEFCA-v2024 pack (new per Wave 15M-G) |
| HITRUST CSF (M10) | HIPAA-2024 pack overlay |
| NIST SP 800-66 (M11) | HIPAA-2024 pack controls/ |
| EU AI Act (clinical AI) (M12) | EU-AI-ACT-HEALTHCARE-v2024 pack (new per Wave 15M-G) |

### §3.14 Domain N — Operational surface (17 features)

All 17 N-row features are operational substrate concerns shared with
the wider oyatie substrate. They RETAIN in healthcare-integration as
operational concerns of the broker µservice (multi-tenant isolation,
per-tenant key custody, per-cell SLO, cell-aware data residency,
active-active multi-region, DR RTO/RPO, chaos drilling, capacity
admission control, cost budget enforcement, migration playbook, SDK
gen, tenant onboarding, FAQ, tutorial, threat model, DPIA, catalog
records). Each new healthcare-domain µservice authors its own
equivalent N-domain shape per ADR-0131 §"Canonical folder shape".

## §4 Capabilities PRESERVED in healthcare-integration

After the scope-narrow, the healthcare-integration µservice retains:

### §4.1 Bounded contexts (3)

- `fhir-broker` (renamed from `fhir-resource` to clarify broker concern)
- `hl7v2-broker` (renamed from `hl7-message`)
- `dicom-broker` (new explicit bounded context for DICOM concern that
  was implicit in the 2026-05-20 manifest)

Removed bounded contexts (5 → 3):
- `patient-record` → migrated to emr
- `referral` → migrated to care-management
- `clinical-consent` → migrated to consent-graph (canonical) +
  healthcare-integration (broker-side segmentation only; renamed
  internally to `consent-segmentation-at-broker-boundary`)

### §4.2 Capabilities (6, narrowed)

- `fhir-read` — broker concern (PRESERVED)
- `hl7-route` — broker concern (PRESERVED)
- `ehr-provenance-seal` — integration egress (PRESERVED)
- `consent-segmentation` (narrowed from `consent-sync` — broker
  boundary segmentation only; canonical consent state in
  consent-graph)
- `break-glass-relay` (narrowed from `break-glass-authorize` —
  broker relays the justification metadata; canonical break-glass
  policy in emr / emergency / pharmacy)
- `mpi-substrate` (narrowed from `patient-match-review` — MPI
  substrate is integration concern; canonical patient identity is
  emr's concern)

### §4.3 IPs (10 broker-scoped of the original 30)

PRESERVED IPs:
- IP-001 tenant-scope-kernel
- IP-002 cedar-default-deny
- IP-005 rest-contract-surface (broker REST surface)
- IP-006 async-event-surface (broker event surface)
- IP-007 grpc-internal-surface
- IP-011 observability-audit-events
- IP-013 emergency-services-bypass (broker-side bypass relay)
- IP-026 hl7-ack-route-custody (broker concern)
- IP-027 fhir-consent-segmentation (broker boundary concern)
- IP-030 clinical-provenance-seal-export (integration egress)

MIGRATED IPs (move to new µservices in Wave 15M-C):
- IP-028 break-glass-justification-review → emr authors a parallel IP
  for the canonical break-glass on the patient record; broker-relay
  IP RETAINED here
- IP-029 mpi-patient-match-adjudication → emr authors a parallel IP
  for canonical patient-match-review; MPI substrate IP RETAINED here

OPERATIONAL-SUBSTRATE IPs RETAINED (operational concerns of any
flat µservice): IP-003 ontology-projection, IP-004 workflow-template-
library, IP-008 policy-eval-library-binding, IP-009 credential-
sidecar-binding, IP-010 multi-region-cell-layout, IP-012 abuse-
defence-edge-waf, IP-014 marketplace-dealset-settlement, IP-015 data-
residency-pack-overlays, IP-016 backfill-replay-worker, IP-017 cost-
budget-enforcer, IP-018 capacity-admission-control, IP-019 sdk-
client-generation, IP-020 catalog-layer-registration, IP-021 slo-
gated-promotion, IP-022 chaos-drill-pack, IP-023 dpia-evidence-
packet, IP-024 threat-model-control-map, IP-025 audit-findings-
closeout.

Total IPs in narrowed µservice: 10 broker-scoped + 18 operational-
substrate = 28 IPs (down from 30).

### §4.4 Top-3 counterparts (PRESERVED)

- Redox
- Mirth (NextGen Connect)
- Health Gorilla

The Wave 4-rolling audit at `coherence-audit-2026-05-20.md` already
re-anchored this set to integration counterparts (instead of the
Wave-3-I-era EHR-vendor set Epic/Cerner/Allscripts/Veeva). The
post-narrow µservice carries the corrected counterpart set forward
unchanged.

### §4.5 Performance leadership claims (split per-domain)

The Wave-3-I benchmark file
`benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-oyatie.md`
declared throughput leadership across HL7v2 ingest, FHIR R5 read,
DICOM C-STORE, and MPI patient-match. After narrow:

| Benchmark | Migrates to |
|---|---|
| HL7v2 ingest throughput | RETAINED in healthcare-integration (broker concern) |
| FHIR R5 read p99 | RETAINED (broker concern) |
| DICOM C-STORE p99 | RETAINED (broker concern) |
| DICOMweb STOW-RS p99 | RETAINED |
| MPI match query p99 | RETAINED (MPI substrate) |
| Patient record p99 | emr authors its own benchmark |
| Lab order to result p99 | diagnostics authors its own benchmark |
| ED triage assignment p99 | emergency authors its own |
| ePrescribe to Surescripts ack p99 | pharmacy authors its own |
| Alarm signal to nurse notification p99 | patient-monitoring authors its own |
| BPA evaluation p99 | clinical-decision-support authors its own |
| Care plan write p99 | care-management authors its own |

Each new µservice authors its own `benchmarks/<counterparts>.md`
file per Wave 15M-B exit criteria with the µservice's own top-3
counterpart performance baseline.

## §5 Manifest scope-narrow target

The post-narrow `manifest.json` should reflect:

```json
{
  "microservice": "healthcare-integration",
  "title": "Healthcare Integration Substrate",
  "status": "scope-narrowed-per-adr-0332",
  "date": "2026-05-22",
  "audience_type": "tenant-b2b-healthcare",
  "tenant_class_eligibility": ["demo_trial", "paid"],
  "binding_adrs": [
    "ADR-0105",
    "ADR-0131",
    "ADR-0132",
    "ADR-0138",
    "ADR-0145",
    "ADR-0244",
    "ADR-0245",
    "ADR-0250",
    "ADR-0251",
    "ADR-0263",
    "ADR-0316",
    "ADR-0322",
    "ADR-0328",
    "ADR-0332"
  ],
  "coverage_benchmarks": [
    "Redox",
    "Mirth Connect",
    "Health Gorilla"
  ],
  "bounded_contexts": [
    "fhir-broker",
    "hl7v2-broker",
    "dicom-broker"
  ],
  "depends_on_microservices": [
    "audit-chain",
    "consent-graph",
    "workflow-engine",
    "ontology",
    "tenancy",
    "identity",
    "governance",
    "observability",
    "policy-engine",
    "cloud-iam",
    "compliance"
  ],
  "downstream_microservices": [
    "emr",
    "diagnostics",
    "emergency",
    "pharmacy",
    "patient-monitoring",
    "clinical-decision-support",
    "care-management"
  ],
  "compliance_packs": [
    "HIPAA-2024",
    "SOC-2",
    "ISO-27001",
    "GDPR",
    "21CCA-INFO-BLOCKING-v2024",
    "TEFCA-v2024",
    "KR-MED-LAW-v2024"
  ]
}
```

The `tier`, `tier_subtype`, `tier_classification`, `criticality_tier`,
`capability_tiers` fields RETIRE per the Wave-3-I audit finding
F-TIER-RETIRED-NOT-MIGRATED (P0). Capability-tier projection moves
to `capability-tiers/tier-matrix.md` per ADR-0316.

## §6 PRD scope-narrow target

The post-narrow `PRD.md` retains the existing 400-line shape but
narrows §"Bounded Contexts" to the 3 broker contexts and replaces
the 25 user stories + 30 functional requirements with broker-
scoped equivalents:

- 5 personas × 3 bounded contexts = 15 user stories (down from 25)
- 6 verbs × 3 nouns = 18 functional requirements (down from 30)

Each FR carries bespoke acceptance criteria per ADR-0322 substance
bar (the Wave-3-I PRD shape's uniform acceptance criteria is recorded
as finding F-PRD-FR-ACCEPTANCE-CRITERIA-THIN, P2).

The narrowed PRD adds a §"Decomposition" section at the top citing
ADR-0332 and pointing to the new µservices for the 14-domain
clinical surface.

## §7 RETIRED-marker stubs

Per ADR-0138 six-path deprecation pattern, content that migrates OUT
in Wave 15M-B leaves a RETIRED marker at the old path until all
references update (cleanup in Wave 15M-H).

RETIRED markers required:

- `microservices/healthcare-integration/decisions/ADR-HI-001-fhir-
  envelope-consent-sync-and-break-glass-state-machine.md` — the
  state-machine for break-glass migrates to emr's local ADR. RETIRED
  stub here with redirect to `microservices/emr/decisions/ADR-EMR-
  001-break-glass-state-machine.md` (authored in Wave 15M-C).
- Any IP-028 / IP-029 content that becomes broker-relay-scoped retains
  the IP file but adds a §"Scope narrow — Wave 15M-A/B" section noting
  the canonical IP migrates to the named new µservice.

## §8 Cross-reference update inventory

Files that reference the healthcare-integration µservice and that
need updating in Wave 15M-B+:

- All ADRs that cite `healthcare-integration` as the canonical owner
  of a 14-domain capability — recheck citations and re-target to the
  correct new µservice per §3 above.
- `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` — if the
  thesis mentions healthcare as a single µservice, update to reference
  the 8-µservice cluster.
- `/specs/master-plan-sequencing.json` — Phase 4 healthcare entry
  expands from 1 µservice to 8.
- Any Wave 4-rolling audit briefs that referenced healthcare-
  integration as a single coherence-audit target — successor briefs
  must target the 8 µservices individually.

## §9 Verification

This remediation note's correctness is verified by:

1. Each row in §3 §3.1..§3.14 maps to a §H row in ADR-0332.
2. The scope retained in §4 matches the bounded-context, capability,
   and IP set declared in ADR-0332 §C.8.
3. The manifest scope-narrow target in §5 matches the JSON contract
   the Wave 15M-B agent will produce.
4. The PRD scope-narrow target in §6 matches the ADR-0322 substance
   bar.
5. The RETIRED-marker requirement in §7 satisfies ADR-0138 six-path
   deprecation.
6. The cross-reference inventory in §8 is exhaustive (the Wave 15M-B
   agent verifies each reference updates in lockstep).

`oya gate validate cross-ref-validity` should exit 0 after Wave
15M-B lands; until then, this remediation note carries the
authoritative scope-narrow contract.

## §10 Open items for Wave 15M-B execution

| # | Item | Owner | Resolution path |
|---|---|---|---|
| 1 | Should `mpi-substrate` move to a dedicated `mpi` µservice or stay inside healthcare-integration? | council-architecture | Resolved-by-default: stay inside healthcare-integration as MPI substrate; promote to dedicated µservice in Wave 16+ if cross-cell match query volume justifies. |
| 2 | Should `terminology-service` move to a dedicated `terminology` µservice? | council-architecture | Resolved-by-default: stay inside healthcare-integration; promote to dedicated µservice in Wave 16+ if FHIR terminology service operations ($expand / $translate / $lookup / $validate-code) need independent SLO budget. |
| 3 | Should `ehr-connector` (Epic / Cerner / athenahealth / Allscripts / etc. adapters) move to a marketplace-style plugin model? | council-architecture + axis-marketplace | Resolved-by-default: stay inside healthcare-integration as `ehr-connector` bounded context; per-vendor adapters live as vendor-pack overlays per ADR-0064 canonical-base-and-localization-packs. |
| 4 | What is the Wave-15M-B authoring of the broker-side `dicom-broker` bounded context (the 2026-05-20 audit flagged DICOMweb capability YAML missing)? | axis-healthcare-integration | Wave 15M-B-hci-narrow agent authors three new capability YAMLs: `dicomweb-search.yaml`, `dicomweb-store.yaml`, `dicomweb-retrieve.yaml` per the existing audit finding F-DICOMWEB-CAPABILITY-MISSING. |

## §11 Halt

This remediation note halts CLEAN with the scope-narrow contract
fully declared, all 14 domain migrations traced to destination
µservices, the manifest + PRD narrow targets defined, the RETIRED-
marker requirement enumerated, and the cross-reference update
inventory listed for Wave 15M-B execution.

No content is moved or deleted in Wave 15M-A; this note is the
authoring contract for Wave 15M-B agents to consume.

End of remediation notes.
## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O review for `healthcare-integration`.
- IPs rewritten in place: 0.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 30.
- Counterpart anchors were made explicit where the verification regex lacked the service's native benchmark vocabulary.
- Follow-up: none for stamp-shell conversion; future service owners may still improve individual IP depth outside this bucket.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/healthcare-integration/coherence-audit-2026-05-20.md
- microservices/healthcare-integration/IP-020-catalog-layer-registration.md
- microservices/healthcare-integration/decisions/ADR-HI-001-fhir-envelope-consent-sync-and-break-glass-state-machine.md
- microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-adapter-valkey.yaml

Counterpart-fact preservations:
- None.

Files renamed:
- microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-adapter-redis.yaml -> microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-adapter-valkey.yaml
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 30.
- Trigger A matched: 13.
- Trigger B matched: 28.
- Trigger C matched: 22.
- Trigger D matched: 0.
- IPs unmatched: 0.

### IP changes
- `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-002-cedar-default-deny.md` — added DR posture.
- `microservices/healthcare-integration/IP-003-ontology-projection.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-004-workflow-template-library.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-005-rest-contract-surface.md` — added API Versioning, DR posture.
- `microservices/healthcare-integration/IP-006-async-event-surface.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-007-grpc-internal-surface.md` — added API Versioning, DR posture.
- `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-009-credential-sidecar-binding.md` — added DR posture.
- `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-011-observability-audit-events.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-012-abuse-defence-edge-waf.md` — added DR posture.
- `microservices/healthcare-integration/IP-013-emergency-services-bypass.md` — added DR posture.
- `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-016-backfill-replay-worker.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-018-capacity-admission-control.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-019-sdk-client-generation.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-020-catalog-layer-registration.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-021-slo-gated-promotion.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-022-chaos-drill-pack.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-023-dpia-evidence-packet.md` — added DR posture.
- `microservices/healthcare-integration/IP-024-threat-model-control-map.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-025-audit-findings-closeout.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-027-fhir-consent-segmentation.md` — added DR posture, Sustainability emission.
- `microservices/healthcare-integration/IP-028-break-glass-justification-review.md` — added Sustainability emission.
- `microservices/healthcare-integration/IP-029-mpi-patient-match-adjudication.md` — added API Versioning.
- `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md` — added API Versioning, DR posture, Sustainability emission.

### Follow-up
- `microservices/healthcare-integration/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `healthcare-integration`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 1800s, RPO 300s, active-active clinical message ingress and emergency bypass queues, failover_runbook `microservices/healthcare-integration/runbooks/clinical-interoperability-failover.md`.
- ADR: ADR-0343; HIPAA/SOC2/ISO/KR-PIPA floors are satisfied by the integration target.
- Alternative considered: use queue backlog recovery only; rejected because acknowledgement custody and emergency bypass require live failover semantics.
- Cost: requires partner-route replay discipline and duplicated consent/patient-match safety queues.

### Capacity model
- Values: 0.35 vCPU, 768 MiB RAM, 10 GB replay/provenance storage, 5 Postgres connections, 4 Valkey connections, 16 outbound HTTP connections; `per_workflow_run` scaling; Tier-2 placement; 2-36 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: a flat per-partner connector pool; rejected because the manifest declares workflow execution and external endpoint fanout as the capacity driver.
- Cost: adds workflow-run, route-table, queue-shard, and partner-dimension admission controls.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for emergency bypass, HIPAA emergency-mode, patient-match safety queues, and consent conflict resolution.
- ADR: ADR-0344.
- Alternative considered: carbon-route all replay jobs; rejected because some replays are incident recovery or safety evidence recovery.
- Cost: adds per-partner, per-route, per-replay, and per-data-class cost dimensions.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for FHIR, HL7, referral, consent, and partner migration contracts, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: external vendor versioning only; rejected because Oyatie must guarantee tenant pinning across partner migrations.
- Cost: maintains compatibility lanes for partner adapters and replay clients.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.35 vCPU, 768 MiB RAM, 10 GB storage, and per_workflow_run scaling follow FHIR import, HL7 route, consent sync, break-glass, patient match, and provenance workflow templates.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-3 placement because pod_runtime_tier=1 cannot co-vary with Tier-3 even though this is a product-facing interoperability service.
- Cost: Commits Healthcare Integration to workflow replay, external endpoint headroom, and Kata-backed placement for clinical data exchange.

### Block 2: dr
- Values: RTO 1800s, RPO 300s, active-active true, backup substrates postgres_wal_g, valkey_cluster, object_storage_versioned, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected 60s RPO because integration workflows can replay from idempotent queues and sealed evidence without claiming bedside-alarm class recovery.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/clinical-interoperability-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; Healthcare Integration brokers FHIR, HL7, consent, break-glass, provider directory, and provenance workflows across tenant PHI boundaries. It does not execute tenant-customer code, but it is a tenant clinical interoperability data-plane service, so Tier 1 and Tier-2 cell placement are required.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 0 because templates and workflows are first-party governed actions, not arbitrary tenant-customer code execution.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only v1 public contract files are in-tree.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, valkey, cedar, openbao, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected connector-specific SaaS names because CVE ownership belongs to registry-backed substrate dependencies.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/valkey-cluster, oci-guest/tenant-namespace, oci-guest/valkey-cluster, on-prem/tenant-namespace, colo/tenant-namespace, oyatie-as-cloud-provider/shard-cell, oyatie-as-cloud-provider/openbao-bindings.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected legacy provider-specific free-form module paths because the schema requires context plus primitive plus version_pin.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
