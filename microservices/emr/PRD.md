---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-emr
microservice: emr
title: Electronic Medical Record (EMR/EHR) Product Requirements
status: wave-15m-b-authored-2026-05-21
date: 2026-05-21
owner_team: axis-emr + council-clinical + council-product
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0251
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0332
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/emr/ARCHITECTURE.md
  - microservices/emr/README.md
  - microservices/emr/manifest.json
  - microservices/emr/competitor-parity-matrix.md
  - microservices/emr/decisions/ADR-MS-001-bounded-contexts.md
  - microservices/emr/decisions/ADR-MS-002-fhir-r5-default.md
  - microservices/emr/decisions/ADR-MS-003-mobile-first-portal.md
counterparts:
  primary:
    - Epic Systems (Epic Hyperspace + EpicCare + MyChart + Beaker + Willow + Cupid + Stork)
    - Oracle Health Cerner (Cerner Millennium + PowerChart + CommunityWorks + HealtheLife)
    - athenahealth (athenaClinicals + athenaCommunicator + athenaCollector + Population Health)
  secondary:
    - Allscripts Veradigm (Sunrise EHR + Paragon + TouchWorks + Practice Fusion)
    - Meditech (Meditech Expanse + Patient Connect + Web Ambulatory)
    - eClinicalWorks (eClinicalWorks V11 + healow + eCW Scribe)
    - NextGen Healthcare (NextGen Enterprise + NextGen Mobile + NextGen Office)
    - Greenway Health (Intergy + Prime Suite)
planned_enforcement_ref: oya-governance-emr-doc-suite
---

# PRD-emr: Electronic Medical Record (EMR/EHR)

## A. Problem

### A.1 Why EMR Is A Foundational Healthcare µservice

EMR — the clinical Record-Of-Truth — is the single non-negotiable substrate of any healthcare platform. Every other healthcare µservice in the oyatie portfolio (diagnostics, pharmacy, emergency, clinical-decision-support, care-management, telehealth, healthcare-integration) reads from or writes to EMR. A platform that ships diagnostics-without-EMR or pharmacy-without-EMR has shipped point tools, not a healthcare platform.

Epic dominates the US inpatient and large-IDN (Integrated Delivery Network) market because Epic IS the EMR — everything else (Epic Beaker labs, Epic Willow pharmacy, Epic Cupid cardiology, Epic Stork OB, Epic Radiant imaging) is a domain module wrapped around the Epic chart. Oracle Health Cerner sustains a comparable market with Cerner Millennium for the same reason. athenahealth disrupted the ambulatory market by shipping cloud-first EMR (athenaClinicals) packaged inseparably with revenue-cycle-management (athenaCollector) — the EMR is again the spine.

oyatie's healthcare offering MUST therefore make EMR the spine. The EMR µservice carries the chart, the orders, the notes, the audit trail, the consent gate, the patient portal session, and the FHIR/HL7 surface that makes external interoperability possible. If EMR is shallow, every adjacent healthcare µservice degrades to a stub.

### A.2 Bounded Context Decomposition — Why 15

Healthcare clinical workflows decompose into the following 15 bounded contexts. The decomposition is dictated by independent persistence shape, independent retention policy, independent access-control gradient, and independent regulatory anchor citation:

| BC | Persistence | Retention | Cedar Gradient | Regulatory Anchor |
|---|---|---|---|---|
| **patient** | Postgres + Citus shard on tenant_id + patient_id; canonical demographics + MPI link | Tenant-lifetime + 7y post-discharge | physician-can-read-cohort; patient-can-read-own | HIPAA §164.514 (de-identification thresholds) |
| **encounter** | Postgres + temporal table; admission–discharge state machine | 7y post-encounter-close; legal-hold-supersedes | care-team-can-read-active; closed-encounter-read-with-purpose | HIPAA §164.514; CMS §482.24 Conditions of Participation |
| **problem** | Postgres relational + SNOMED CT + ICD-10 link | Patient-lifetime; never hard-deleted | physician-can-amend; nurse-can-read | Meaningful Use Stage 3 Problem List |
| **medication** | Postgres relational + RxNorm + NDC link; reconciliation state | Patient-lifetime; never hard-deleted | physician-can-prescribe; pharmacist-can-verify; nurse-can-administer | DEA Schedule rules; HIPAA; FDA REMS |
| **allergy** | Postgres relational + RxNorm + UNII + SNOMED CT | Patient-lifetime; never hard-deleted | clinician-can-write; tombstone-only-deletion | Joint Commission NPSG.03.06.01 |
| **vital** | TimescaleDB hypertable on (tenant, patient, observed_at) | 7y; legal-hold-supersedes | nurse-can-write; device-can-stream | Joint Commission documentation standards |
| **note** | Postgres + WORM audit table for signed notes; CommonMark + LOINC-coded sections | 7y from author signature; legal-hold-supersedes | author-can-amend-pre-sign; co-signer-can-attest | CMS E/M documentation guidelines 2024 |
| **order** | Postgres + saga state machine; CPOE | 7y; legal-hold-supersedes | physician-can-enter; nurse-can-verify; verbal-order-readback | TJC NPSG.02.03.01 + ASHP CPOE |
| **result** | Postgres + LOINC link; subscriber model for results-review | 7y; legal-hold-supersedes | ordering-clinician-can-review; care-team-can-read | CLIA + CAP + JCAHO |
| **care-team** | Postgres relational; effective-date ranges | Per-encounter or patient-relationship-lifetime | care-team-member-can-read; admin-can-write | HIPAA §164.502 minimum necessary |
| **order-set** | Postgres; protocols + bundles | Catalog-lifetime; versioned | physician-can-author; admin-can-publish | ASHP order-set best practices |
| **documentation** | Postgres; templates + smart-text + dot-phrases | Catalog-lifetime; versioned | clinician-can-author; admin-can-publish | CMS E/M documentation 2024 |
| **billing-code** | Postgres; CPT + HCPCS + ICD-10-CM + ICD-10-PCS + Modifier | 7y from claim adjudication | coder-can-finalize; physician-can-attest | AMA CPT + CMS HCPCS + WHO ICD-10 |
| **patient-education** | Postgres + content registry; multi-language | Catalog-lifetime | patient-can-read-own; clinician-can-assign | Patient Bill of Rights + Health Literacy |
| **portal-session** | Valkey + Postgres event log; auth + scope | 90d session log; 7y audit | patient-can-access-own; proxy-can-access-with-grant | HIPAA §164.524 patient right of access |

Each BC ships with its own ports (kernel), domain types, use-case orchestration, and adapter trio (REST + AsyncAPI + gRPC) per ADR-0105 13-layer enum and ADR-0145 inter-microservice communication discipline. Crate naming follows BNF v4.1: `oya-emr-<bc>-<layer>` (e.g., `oya-emr-patient-kernel`, `oya-emr-encounter-domain`, `oya-emr-order-application`).

### A.3 Market Context — Why Now

The US EMR market in 2026 is consolidated to three players (Epic ~38% market share by hospital, Oracle Health Cerner ~25%, athenahealth ~12% in ambulatory) plus a long tail of regional and specialty EHRs. The combined US EMR + ambulatory EHR market is $42B annually (2025 figures, KLAS Research + HIMSS Analytics). Globally the EMR market reaches $75B with Asia-Pacific (KR, JP, AU, SG) and EU growing fastest.

Yet three structural problems persist that an oyatie-native EMR can solve at a structural level:

1. **Vendor lock-in.** Epic's chart data is exportable only via Epic-proprietary tooling (Epic Care Everywhere, Epic Healthy Planet) or via slow FHIR R4 endpoints throttled to per-resource pull. Migration off Epic averages 18-36 months and $100M-$500M for a large IDN. oyatie EMR ships a `/fhir/$export` Bulk Data endpoint at FHIR R5 plus a tenant-data-portability runbook — making both inbound AND outbound migration a deterministic workflow.

2. **Closed clinical-decision-support.** Epic's BPAs (Best Practice Advisories) live inside Epic; Cerner's Discern alerts live inside Cerner. Third-party CDS hooks (CDS Hooks 2.0) are awkwardly grafted on. oyatie EMR is built atop the oyatie clinical-decision-support µservice (a separate µservice per ADR-0132 single-concern); CDS Hooks 2.0 is the native invocation pattern.

3. **No tenant-class native multi-segment.** Epic ships separate products for academic IDNs (Epic Hyperspace), large community IDNs (Epic Community Connect), ambulatory clinics (Epic Spring), and FQHCs (Epic Spring + Epic for Community Connect). Cerner ships Millennium for large IDNs and CommunityWorks for community hospitals. oyatie EMR uses ONE canonical base and tenant-class overlays (per `feedback_canonical_base_localization`) — one code path, per-class behavior.

The 2024-2026 regulatory wave compounds the pressure:

- **HHS ONC Cures Act Final Rule (2020 + 2024 enhancements):** Information blocking penalties + USCDI v4 + FHIR R4 mandatory.
- **HHS HTI-1 Final Rule (2024):** Predictive Decision Support Intervention (DSI) transparency requirements for any AI in clinical workflow.
- **CMS Final Rule on Patient Access (CMS-0057-F, 2024):** Prior authorization API + Provider Access API by 2027-01-01.
- **TEFCA (Trusted Exchange Framework and Common Agreement) Phase 2 (2024):** QHIN (Qualified Health Information Network) participation requires FHIR R4 read + write.
- **State PMPs (Prescription Drug Monitoring Programs):** All 50 states require PDMP query at controlled-substance prescribe time.
- **EU EHDS (European Health Data Space Regulation, 2024 / phased 2025-2029):** Cross-border EU EHR interoperability mandate.

An oyatie-native EMR built on the oyatie compliance-pack primitive (ADR-0251) can ship USCDI v4 + HTI-1 DSI transparency + EHDS data sharing as pack-driven defaults rather than per-vendor add-ons.

### A.4 What This µService IS NOT

To prevent scope creep, EMR explicitly does NOT own:

- **HL7 v2 wire-protocol I/O.** Owned by `healthcare-integration` µservice; EMR consumes parsed FHIR resources via internal gRPC.
- **Lab/imaging instrument I/O.** Owned by `diagnostics` µservice; EMR receives DiagnosticReport + Observation resources.
- **ePrescribing wire-protocol (NCPDP SCRIPT, NCPDP Telecom).** Owned by `pharmacy` µservice; EMR sends MedicationRequest, pharmacy returns MedicationDispense.
- **Insurance eligibility / claims (X12 270/271, 837, 835).** Owned by `cloud-billing` µservice.
- **Clinical decision support rules engine.** Owned by `clinical-decision-support` µservice; EMR invokes CDS Hooks 2.0.
- **Care plan + care-coordination across encounters.** Owned by `care-management` µservice; EMR exposes encounter-level data.
- **Patient demographics master record + duplicate resolution algorithm.** Owned indirectly via the `mpi` slice of `healthcare-integration`; EMR consumes the resolved canonical Patient resource.

This decomposition is mandated by ADR-0132 (single-concern, no suites). EMR is the clinical-chart-of-truth; everything else is a peer µservice.

## B. Target Users

### B.1 Primary Personas

**Persona 1 — Dr. Sarah Kim, Hospitalist (mid-career, urban academic medical center).**
- Sees 12-18 admitted patients per shift; reviews 6-10 new admissions overnight.
- Uses chart-open hundreds of times per shift; latency tax of >2 seconds per chart-open destroys productivity.
- Documents 8-12 H&Ps + 18-25 progress notes per shift.
- Demands: chart-open ≤ 800ms p99; note auto-save every 30s; voice dictation; smart-phrase library.
- Comparison: Epic Hyperspace + Dragon Medical One is her current baseline; she expects oyatie EMR to match or exceed.

**Persona 2 — Maria Lopez, RN, Med-Surg Floor.**
- Cares for 4-5 patients per shift; documents q4h vitals + I/O + assessments.
- Verifies and administers 35-50 medications per shift via barcode (BCMA).
- Demands: vital entry ≤ 300ms; med-administration scan-to-confirm ≤ 500ms; allergy-conflict alert at order-verify time.
- Comparison: Epic Rover + Cerner CareAware are her mobile baselines.

**Persona 3 — David Park, MA (Medical Assistant), Family Medicine Clinic.**
- Rooms 22-28 patients per day in a primary-care setting.
- Captures vitals + chief complaint + medication reconciliation in a 5-7 minute pre-visit window.
- Demands: tablet/iPad-first UX; voice-to-text vital entry; medication-reconciliation pre-population from refill history.
- Comparison: athenaClinicals + Practice Fusion are common in this segment.

**Persona 4 — Margaret Chen, Patient (chronic-condition adult, multi-comorbid).**
- Manages T2DM + HTN + HLD + early CKD; sees 3-4 specialists.
- Uses patient portal to message care team, refill meds, view results, schedule, pay bills.
- Demands: mobile app with biometric login; FHIR-grade allergy + med list export to share with new specialists; ability to grant proxy access to her adult daughter.
- Comparison: Epic MyChart is her current baseline; she has hated every alternative (Cerner HealtheLife in particular).

**Persona 5 — James Thompson, Caregiver-Proxy (adult son of 78yo dad with dementia).**
- Holds healthcare power-of-attorney for his father.
- Needs delegated patient-portal access without his father's password.
- Demands: proxy-grant workflow (per HIPAA §164.502(g) + state-specific personal-representative rules); ability to view dad's chart, message clinicians, manage appointments.

### B.2 Secondary Personas

**Persona 6 — Jennifer Davis, Health Information Management (HIM) Director.**
- Oversees medical-records department; manages release-of-information (ROI), audit log access, and breach response.
- Demands: HIPAA-grade audit log; configurable retention; chart-correction workflow (per Joint Commission); legal-hold tooling.

**Persona 7 — Dr. Anita Patel, CMIO (Chief Medical Information Officer), Mid-Size IDN.**
- Owns clinical informatics governance; manages order-set library, smart-phrase library, BPA library, documentation templates.
- Demands: governed authoring + versioning + retirement of clinical-content artifacts; A/B testing of order-sets; ability to retire entire order-set bundles with retroactive audit trail.

**Persona 8 — Robert Anderson, Compliance Officer.**
- Reports to CIO + General Counsel; owns HIPAA Privacy Rule + Security Rule + 42 CFR Part 2 + state-law compliance.
- Demands: per-employee access-history report; break-glass-justification audit trail; mock breach-notification drill tooling.

**Persona 9 — Hyo-Jin Park, Healthcare System CTO (regulated KR-private-hospital network).**
- Operates in KR jurisdiction under KR 의료법 + KR-PIPA + 건강보험심사평가원 reporting.
- Demands: KR-residency cell certification; 의료법 §22 medical-record-retention 10y; 건강보험 reimbursement-claim code generation.

**Persona 10 — Hana Tanaka, Hospital CISO.**
- Owns security posture for hospital IT.
- Demands: zero-trust per-clinician-per-chart access enforced at the policy engine; mTLS for all internal service-to-service traffic; KMS-backed encryption with per-tenant key envelope; SOC 2 Type II + HITRUST CSF readiness.

## C. User Stories

### C.1 Inpatient Clinical Workflow

- **US-INP-001 (Sarah, hospitalist).** As a hospitalist, I want to open any active inpatient's chart in ≤ 800ms p99 so that my list-driven rounding tempo is not broken.
  Acceptance: chart-open p99 ≤ 800ms; chart-open p50 ≤ 200ms; chart-open uses CDN-cached snapshot when stable + live deltas overlaid; SLO `chart-open-latency.openslo.yaml` BLOCKER-gated.

- **US-INP-002 (Sarah, hospitalist).** As a hospitalist, I want to enter a CPOE order set for sepsis bundle in ≤ 60 seconds so that I can comply with CMS SEP-1 within the 3-hour window.
  Acceptance: bundle-entry uses pre-authored order-set with checkbox parameterization; pharmacy + diagnostics + nursing-order all dispatched atomically; SLO `order-entry-latency.openslo.yaml` p99 ≤ 200ms per order; bundle commit p99 ≤ 1 second.

- **US-INP-003 (Sarah, hospitalist).** As a hospitalist, I want to dictate a daily progress note via voice + smart-phrase macros and have it auto-saved every 30 seconds so that I do not lose work on session interruption.
  Acceptance: every keystroke saved to a local-first CRDT buffer; server commit p99 ≤ 250ms; voice-to-text via on-device + tenant-cloud fallback; smart-phrase library tenant-scoped.

- **US-INP-004 (Maria, RN).** As a med-surg RN, I want barcode med-administration (BCMA) to verify "right patient + right med + right dose + right route + right time" in ≤ 500ms so that the floor workflow is not bottlenecked.
  Acceptance: barcode scan dispatched to local cache; verification result p99 ≤ 500ms; allergy + drug-interaction + due-time conflict surfaced inline; audit emits `MedicationAdministered` with full evidence.

- **US-INP-005 (Maria, RN).** As an RN, I want to enter q4h vital signs from a tablet at the bedside while the patient watches so that I do not break the conversation by walking back to a workstation.
  Acceptance: tablet UX optimized for one-thumb entry; vital-entry p99 ≤ 300ms write; offline-resilient with conflict-free merge on reconnect.

### C.2 Ambulatory + Family Medicine Workflow

- **US-AMB-006 (David, MA).** As a medical assistant in a family-medicine clinic, I want to room a patient in ≤ 5 minutes including vital capture, CC capture, med-reconciliation, and PMH update so that the physician's 15-minute slot starts on time.
  Acceptance: pre-visit packet pre-populated from refill-history + prior-visit notes; vital capture via Bluetooth-paired devices; med-reconciliation suggests "still taking?" prompts per medication; persistence of rooming-complete event triggers physician notification.

- **US-AMB-007 (Family-medicine physician).** As a primary-care physician, I want a "single-screen patient summary" showing problem list + active meds + allergies + last vitals + recent encounters + open orders + due preventive-care items so that I can start any patient encounter in ≤ 30 seconds of orientation.
  Acceptance: summary-view assembled from one tenant-scoped query; p99 ≤ 500ms; preventive-care items pulled from Population-Health rules; care-gap surface integrated.

- **US-AMB-008 (Family-medicine physician).** As a primary-care physician, I want to e-prescribe a controlled substance (Schedule II) with EPCS (Electronic Prescribing of Controlled Substances) 2FA + DEA-compliant audit + PDMP query in ≤ 30 seconds so that the patient does not wait at the pharmacy.
  Acceptance: prescription request travels to `pharmacy` µservice; PDMP query travels in parallel; EPCS 2FA via passkey (per ADR-0188 + ADR-0244 §D-7); DEA Schedule II compliance gated by Cedar policy.

### C.3 Emergency Department Workflow

- **US-ED-009 (ED physician).** As an ED physician, I want to register a patient on a "John Doe" trauma name and merge them into the canonical Patient record post-identification without losing any orders entered under the trauma name so that life-saving care never waits on demographic resolution.
  Acceptance: patient-merge workflow preserves all orders, results, notes; merge emits `PatientMerged` event; pre-merge identifier remains queryable for legal/audit.

- **US-ED-010 (ED nurse).** As an ED triage nurse, I want to apply ESI (Emergency Severity Index) 1-5 and have the patient routed to the appropriate ED bed with an open chart in ≤ 90 seconds so that the door-to-provider time is not driven by EMR friction.
  Acceptance: ESI capture + bed-assignment + chart-open atomically dispatched; downstream emergency µservice handles bed-assignment; chart-open optimistic.

### C.4 Patient + Caregiver Portal

- **US-PORT-011 (Margaret, patient).** As a chronic-condition patient, I want to view my full chart (problem list, med list, allergy list, immunization record, vital trends, last 24 months of encounter summaries, all my results) from a mobile app authenticated by Face ID + passkey so that I have full transparency into my own data per HIPAA §164.524.
  Acceptance: mobile-first UX; FHIR R5 read endpoint; latency p99 ≤ 1.5 seconds for full-chart-view; biometric auth via passkey (per ADR-0188).

- **US-PORT-012 (Margaret, patient).** As a patient, I want to message my care team and receive a response within 2 business days, attach photos to messages, and have the conversation thread preserved in my chart so that asynchronous communication with my doctors is durable.
  Acceptance: portal messaging persisted as Communication FHIR R5 resource; care-team-routing per role; attachment up to 50MB with virus + PII-redaction scans.

- **US-PORT-013 (James, caregiver-proxy).** As an adult child holding healthcare-power-of-attorney for my father, I want to be granted proxy access to my father's chart by completing a verified-identity workflow + uploading the POA document + having my father attest (when capable) or having a state-specific personal-representative pathway when not capable.
  Acceptance: proxy-grant saga via Workflow Engine; HIPAA §164.502(g) compliance; state-pack overlay handles state-specific personal-representative variation; audit emits `ProxyGrantInitiated`, `ProxyGrantApproved`, `ProxyAccessExercised`.

### C.5 HIM + Compliance Workflow

- **US-HIM-014 (Jennifer, HIM director).** As a HIM director, I want to generate a per-employee chart-access audit report covering any user × any patient × any timeframe so that I can respond to HIPAA §164.528 accounting-of-disclosures requests.
  Acceptance: audit-query API; tenant-scoped; per-patient-row export to CSV; integration with `audit-chain` µservice for tamper-evident sealing.

- **US-HIM-015 (Jennifer, HIM director).** As a HIM director, I want to apply a legal hold to a patient's chart that suspends retention-based deletion until the hold is lifted, with full audit of the hold lifecycle.
  Acceptance: legal-hold flag at patient + encounter level; hold supersedes retention; lift-hold requires legal-counsel attestation; integration with cloud-billing for charge-capture freeze.

- **US-HIM-016 (Robert, compliance officer).** As a compliance officer, I want every break-glass invocation (clinician override of standard access controls in emergency) to require a justification + emit a mandatory-review event that auto-routes to a Privacy Officer for retrospective review within 24 hours.
  Acceptance: break-glass UI requires structured justification (clinical-emergency / patient-relationship-emergency / IT-emergency); audit emits `BreakGlassInvoked`; downstream workflow opens a review case; SLO `break-glass-review-latency.openslo.yaml` 24h.

### C.6 Clinical Content Governance

- **US-CMIO-017 (Anita, CMIO).** As a CMIO, I want to author + version + publish + retire order sets with full audit so that clinical content governance is durable and reversible.
  Acceptance: order-set CRUD with versioning; A/B-eligible flag; deprecation flow with grandfather period; audit emits per-lifecycle event.

- **US-CMIO-018 (Anita, CMIO).** As a CMIO, I want to A/B-test two versions of a sepsis order set on tenant-scoped cohorts and observe outcome metrics (door-to-antibiotics time, 30-day mortality, ICU LOS) so that I can iterate on clinical content with evidence.
  Acceptance: integration with `analytics` µservice for outcome surface; A/B assignment per encounter; analytics emission per arm.

### C.7 Tenant-Class-Driven Variations

- **US-TENANT-019 (KR private hospital).** As a KR-jurisdiction tenant, I want chart retention to default to 10 years per KR 의료법 §22 (rather than 7 per HIPAA) and reimbursement-claim codes to default to HIRA (Health Insurance Review & Assessment) format.
  Acceptance: KR-MEDICAL-LAW-2024 pack overlays change retention rule; HIRA code map active.

- **US-TENANT-020 (US-based Veteran Affairs partner hospital).** As a VA-adjacent tenant operating on a VA-data-sharing agreement, I want VistA-compatible export endpoints + DoD IL5-aligned cell certification.
  Acceptance: deployment to `il5`-cert cell; VistA-FHIR-bridge endpoint; DOD-IL5 pack overlay.

(25 personas × ~30 user stories = additional stories itemized in companion sections; the above are the canonical-spine subset.)

## D. Functional Requirements

### D.1 patient (BC #1) — Demographics + MPI Linkage

- **FR-PAT-001:** `patient.create` shall accept name (given, family, middle, suffix, prefix), dob, gender, sex-at-birth, gender-identity, sexual-orientation, race (OMB 2024 categories), ethnicity, preferred-language, marital-status, religion, address (with address-history), phone (with type), email, SSN-optional (HIPAA-deidentified by default), MRN (medical record number, tenant-scoped), and emit `PatientCreated` event with full evidence envelope.
- **FR-PAT-002:** `patient.merge` shall accept two patient_id values, resolve all references (orders, results, notes, encounters, billing, etc.) to the surviving id, retain a tombstone for the retired id, and emit `PatientMerged` event. Merge is reversible within a configurable window (default 30 days, HIM-extendable to 365).
- **FR-PAT-003:** `patient.unmerge` shall reverse a merge within the configured window, restore the retired id, and re-route references back. Audit emits `PatientUnmerged`.
- **FR-PAT-004:** `patient.read` shall return FHIR R5 Patient resource by default; FHIR R4 on Accept-Version header; HL7 v2.5.1 ADT^A04 on legacy-tenant pack overlay.
- **FR-PAT-005:** `patient.search` shall implement FHIR R5 `_search` semantics including phonetic-match (Soundex + Double Metaphone), DOB-range, gender, address, identifier (MRN + SSN-tokenized), and federated cross-tenant search via the MPI substrate.
- **FR-PAT-006:** `patient.deidentify` shall produce a HIPAA Safe Harbor compliant deidentified projection (18 identifier types removed per §164.514(b)(2)) for research-and-analytics tenants per consent.

### D.2 encounter (BC #2) — Visit Lifecycle

- **FR-ENC-007:** `encounter.start` shall accept type (inpatient-admission, ED-visit, outpatient-office, telehealth-video, telehealth-audio-only, observation, ambulatory-surgery, urgent-care, virtual-second-opinion), care-team (initial assignment), location (bed, room, telehealth-room), chief-complaint, admit-source, admit-priority, and emit `EncounterStarted`.
- **FR-ENC-008:** `encounter.transfer` shall move an encounter between locations (e.g., ED → med-surg → ICU), preserving all in-flight orders and notes, and emit `EncounterTransferred`.
- **FR-ENC-009:** `encounter.discharge` shall close the encounter with discharge-disposition, discharge-summary, discharge-instructions, discharge-medication-list, follow-up appointment, and emit `EncounterDischarged`. Trigger downstream care-management episode-of-care creation.
- **FR-ENC-010:** `encounter.reopen` shall reverse discharge within a configurable window (default 24h) for documentation correction, requiring physician attestation.

### D.3 problem (BC #3) — Problem List

- **FR-PROB-011:** `problem.add` shall accept SNOMED CT concept code (preferred) + ICD-10-CM code (cross-walk), onset date, abatement date, status (active, recurrence, relapse, inactive, remission, resolved), severity (mild, moderate, severe), and emit `ProblemAdded`.
- **FR-PROB-012:** `problem.resolve` shall transition status to resolved with abatement date.
- **FR-PROB-013:** `problem.amend` shall update a problem with author + timestamp + reason; original retained as immutable history per CMS amendment rules.
- **FR-PROB-014:** `problem.search` shall query by SNOMED, ICD-10, onset-date-range, status, severity.

### D.4 medication (BC #4) — Active Medication List + Reconciliation

- **FR-MED-015:** `medication.prescribe` shall accept RxNorm RXCUI + NDC (when known) + dose + strength + route + frequency + duration + refill count + indication (problem link) + DEA Schedule designation + EPCS attestation (for Schedule II); dispatch to `pharmacy` µservice; emit `MedicationPrescribed`.
- **FR-MED-016:** `medication.reconcile` shall present a UI for entering "medications-taken-as-reported" from the patient, allow accept/modify/discontinue for each, and emit `MedicationReconciled`.
- **FR-MED-017:** `medication.discontinue` shall mark a medication discontinued with reason (resolved, ineffective, adverse-effect, completed-course, patient-preference, drug-interaction), notify pharmacy, and emit `MedicationDiscontinued`.
- **FR-MED-018:** `medication.refill` shall queue a refill via pharmacy with care-team attestation.
- **FR-MED-019:** `medication.interaction_check` shall invoke `clinical-decision-support` CDS Hooks 2.0 at every prescribe-time with severity-class drug-drug + drug-allergy + drug-condition checks.
- **FR-MED-020:** `medication.controlled_substance.pdmp_query` shall query the state Prescription Drug Monitoring Program at every Schedule II + Schedule III + Schedule IV prescribe time.

### D.5 allergy (BC #5)

- **FR-ALG-021:** `allergy.record` shall accept allergen (RxNorm RXCUI for drugs; UNII for substances; SNOMED for foods/environmental), reaction (SNOMED CT), severity (mild, moderate, severe, life-threatening), criticality (low, high, unable-to-assess), verification-status (unconfirmed, confirmed, refuted, entered-in-error), and emit `AllergyRecorded`.
- **FR-ALG-022:** `allergy.refute` shall mark an allergy refuted with clinician attestation, retain immutable history; allergy-checks at prescribe time SHALL NOT use refuted allergies.
- **FR-ALG-023:** `allergy.search` shall query by allergen, reaction, severity.

### D.6 vital (BC #6)

- **FR-VIT-024:** `vital.record` shall accept code (LOINC; e.g., 8867-4 heart rate, 8480-6 systolic BP, 8462-4 diastolic BP, 8310-5 body temperature, 9279-1 respiratory rate, 2710-2 oxygen saturation, 29463-7 body weight, 8302-2 body height, 39156-5 BMI, 56115-9 pain score), value, unit, device-link (when available), observed-at, observer (clinician id), and emit `VitalRecorded`.
- **FR-VIT-025:** `vital.stream` shall accept high-frequency device streams (e.g., 1Hz pulse-ox, 1-second-EKG) via WebSocket + flatbuffer, persisting to TimescaleDB with downsampling rule.
- **FR-VIT-026:** `vital.trend` shall return time-series for any vital code over any range with downsampled aggregation tiers.

### D.7 note (BC #7) — Clinical Documentation

- **FR-NOTE-027:** `note.draft` shall create a Composition resource (FHIR R5) with sections (e.g., SOAP: subjective, objective, assessment, plan; or H&P: HPI, ROS, PE, A&P; or Procedure: indication, technique, findings, impression, plan; or Discharge: hospital course, discharge meds, follow-up).
- **FR-NOTE-028:** `note.autosave` shall persist every keystroke-delta with CRDT-conflict-free merge; the auto-saved buffer is encrypted at rest with the tenant-KEK.
- **FR-NOTE-029:** `note.sign` shall transition a note from draft to signed, emit `NoteSigned`, freeze the note for editing (only amendments allowed thereafter).
- **FR-NOTE-030:** `note.amend` shall append an amendment block (per CMS guideline §30.3.4) with author + timestamp + reason; original retained.
- **FR-NOTE-031:** `note.cosign` shall route a note for co-signature (e.g., resident → attending; PA → supervising physician); emit `NoteCoSigned`.
- **FR-NOTE-032:** `note.dictate` shall accept voice input via the configured voice-to-text adapter (oyatie-native speech-to-text or vendor-BYOK like Nuance Dragon Medical One).
- **FR-NOTE-033:** `note.template` shall apply a documentation template with smart-phrases + dot-phrases (.cvexam → cardiovascular exam template); per-tenant template library.

### D.8 order (BC #8) — CPOE

- **FR-ORD-034:** `order.enter_medication` shall be implemented as an alias to `medication.prescribe` (preserves CPOE audit semantics).
- **FR-ORD-035:** `order.enter_lab` shall accept LOINC test code + specimen-source + collection-priority + clinical-question + ICD-10 indication; dispatch to `diagnostics` µservice; emit `OrderEntered`.
- **FR-ORD-036:** `order.enter_imaging` shall accept SNOMED procedure code + modality + clinical-question + ICD-10 indication; dispatch to `diagnostics` µservice.
- **FR-ORD-037:** `order.enter_consult` shall accept target-specialty + question + priority; route to consulting physician's queue.
- **FR-ORD-038:** `order.enter_diet` shall accept diet-order (NPO, regular, cardiac, renal, dysphagia-mech-soft, etc.) with start/stop times.
- **FR-ORD-039:** `order.enter_activity` shall accept activity-order (bedrest, BRP, OOB-to-chair, ambulate-with-assist, ad-lib) with start/stop times.
- **FR-ORD-040:** `order.enter_nursing` shall accept nursing-order (vital-frequency, I/O monitoring, telemetry, fall-precautions, isolation type).
- **FR-ORD-041:** `order.enter_set` shall accept order-set-id with parameter overrides, expand to individual orders atomically.
- **FR-ORD-042:** `order.verify` shall transition orders requiring nurse verification (verbal-order-readback, etc.) to verified.
- **FR-ORD-043:** `order.cancel` shall cancel an order with reason; emit `OrderCanceled`.

### D.9 result (BC #9)

- **FR-RES-044:** `result.receive` shall receive DiagnosticReport + Observation from `diagnostics` µservice; route to ordering-clinician's queue; emit `ResultReceived`.
- **FR-RES-045:** `result.review` shall mark a result reviewed by ordering-clinician with optional comment; emit `ResultReviewed`.
- **FR-RES-046:** `result.acknowledge` shall close-out a critical-value result with required acknowledgment (per Joint Commission NPSG.02.03.01); emit `ResultAcknowledged`.
- **FR-RES-047:** `result.subscribe` shall enable provider-subscription to results for a patient panel; downstream notifications.

### D.10 care-team (BC #10)

- **FR-CT-048:** `care_team.assign` shall add a clinician to a patient's care team with role (attending, hospitalist, consultant, primary-nurse, social-worker, case-manager, dietician, RT, PT, OT, ST, chaplain), effective-date, optional end-date.
- **FR-CT-049:** `care_team.discharge` shall remove a clinician on care-transition.
- **FR-CT-050:** `care_team.read` shall return the active care team for any patient × any time-point.

### D.11 order-set (BC #11)

- **FR-OS-051:** `order_set.author` shall create a new order set with title, indication, contained-orders, parameterization.
- **FR-OS-052:** `order_set.publish` shall transition a draft to published, making it available in CPOE.
- **FR-OS-053:** `order_set.deprecate` shall flag an order set as deprecated; existing references continue to work, but new uses warn.
- **FR-OS-054:** `order_set.retire` shall retire an order set; new uses blocked.

### D.12 documentation (BC #12)

- **FR-DOC-055:** `documentation.template.author` shall create a documentation template (e.g., H&P, discharge-summary, procedure-note, consult-note).
- **FR-DOC-056:** `documentation.smart_phrase.author` shall create a tenant-scoped smart phrase (.cvexam, .pulmexam, .dispnote).
- **FR-DOC-057:** `documentation.dot_phrase.expand` shall expand a dot-phrase at note-authoring time.

### D.13 billing-code (BC #13)

- **FR-BIL-058:** `billing_code.capture` shall capture CPT + HCPCS + ICD-10-CM diagnosis + ICD-10-PCS procedure + modifier; route to `cloud-billing` µservice.
- **FR-BIL-059:** `billing_code.physician_attest` shall require physician attestation of E/M level and procedure codes prior to claim submission.
- **FR-BIL-060:** `billing_code.coder_finalize` shall allow a certified coder to finalize the claim code-set.

### D.14 patient-education (BC #14)

- **FR-PED-061:** `patient_education.assign` shall assign a patient-education item (multi-language; PDF / HTML / video) to a patient with delivery channel (portal, print, SMS).
- **FR-PED-062:** `patient_education.acknowledge` shall record patient acknowledgment of the item (per Joint Commission patient-education-documentation standards).

### D.15 portal-session (BC #15)

- **FR-PORT-063:** `portal_session.login` shall authenticate a patient (or proxy) via passkey (per ADR-0188) + risk-adaptive 2FA where applicable.
- **FR-PORT-064:** `portal_session.proxy_grant` shall execute the proxy-grant saga per US-PORT-013.
- **FR-PORT-065:** `portal_session.fhir_read` shall expose FHIR R5 patient-scoped read (Patient, Observation, MedicationStatement, AllergyIntolerance, Immunization, Condition, Procedure, Encounter, DiagnosticReport, DocumentReference, CarePlan, CareTeam).
- **FR-PORT-066:** `portal_session.bulk_export` shall expose `$export` for full patient-data download (per HIPAA §164.524 right of access).

## E. Non-Functional Requirements

### E.1 Performance

- Chart-open p99 ≤ 800ms (per manifest.json).
- Order entry p99 ≤ 200ms.
- FHIR read p99 ≤ 150ms.
- FHIR write p99 ≤ 300ms.
- Search p99 ≤ 400ms.
- Note save p99 ≤ 250ms.
- Per-cell throughput ≥ 50,000 QPS sustained.
- Per-cell concurrent clinicians ≥ 25,000.
- Per-cell concurrent portal users ≥ 100,000.

### E.2 Availability + DR

- 99.99% availability SLO (4.38 min downtime / month).
- RTO ≤ 15 minutes (per manifest).
- RPO ≤ 60 seconds (per manifest).
- Multi-region active-active for cell-tier-0 deployments.
- Quarterly chaos-engineering drills (cell-kill, region-kill, AZ-kill).

### E.2.1 DR Posture (ADR-0343)

- Target: RTO 900s and RPO 60s for chart, order, medication, allergy, result, and portal-session state, matching `manifest.json` `dr.rto_p99_seconds=900` and `dr.rpo_p99_seconds=60`.
- Compliance floors: HIPAA-2024 floors at 3600s/300s with multi-region required; SOC2-T2 floors at 14400s/900s; ISO27001-2022 floors at 14400s/3600s; KR-PIPA sensitive-PI floor at 7200s/600s. The effective clinical target remains the stricter 900s/60s with multi-region active-active enabled.
- failover_runbook: `microservices/emr/runbooks/emr-cell-failover.md`.
- multi_region_active_active: true for PHI-bearing clinical-record cells.
- Why: a clinician can continue chart review, order entry, allergy review, and patient-portal release through a regional event without tenant-visible PHI loss.

### E.2.2 Capacity Model (ADR-0340)

- Per-tenant baseline: 0.75 vCPU, 1536 MiB RAM, 35 GB storage, 8 Postgres connections, 6 Valkey connections, and 10 outbound HTTP connections, matching `manifest.json` `capacity_model`.
- Scaling dimension: `per_request` for chart, order, medication, portal, and FHIR API pressure.
- Cell placement class: Tier-2. The manifest rationale keeps EMR in the PHI application data plane aligned to `pod_runtime_tier=1` without claiming Tier-0 substrate ownership.
- Autoscaling boundaries: min 2 pods per tenant cell, max 40 pods per tenant cell before cell split review.
- Why: this serves chart-of-truth API traffic and portal bursts while keeping EMR capacity bound to the manifest-declared PHI application tier.

### E.2.3 Sustainability + Cost Attribution (ADR-0344)

- Emission envelope: every PHI read/write audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider-routing affected by carbon: no for live chart, order, medication, allergy, and HIPAA emergency-mode paths; yes only for de-identified research exports and non-urgent bulk portal downloads.
- Tenant cost transparency: `finops-portal` exposes EMR cost center rows by bounded context, FHIR bulk export volume, portal-download volume, and clinical-write hot path.
- Why: CSRD, SB-253, and SEC climate-disclosure evidence needs per-tenant cost and carbon attribution without deferring clinical-care traffic.

### E.2.4 API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet across `Oyatie-Api-Version`, `/v/YYYY-MM-DD/...` URL prefixes, and proto3 `api_version_date` fields.
- SDK semver model: major.minor.patch for EMR FHIR, portal, and bulk-export clients.
- Support window: last 3 public versions supported for at least 180 days.
- Per-tenant pinning: yes for FHIR, patient-portal, and Bulk Data `$export` contracts.
- Internal-mesh exemption: yes; direct gRPC handoffs preserve ADR-0145 for EMR-to-diagnostics, pharmacy, emergency, and healthcare-integration traffic.

### E.3 Security

- All PHI encrypted at rest with AES-256-GCM + tenant-scoped KEK (BYOK supported).
- All traffic in transit via TLS 1.3 + HTTP/3 + QUIC (per ADR-0253 KS#10).
- mTLS for inter-µservice traffic.
- Cedar policy evaluation on every PHI access (per ADR-0243).
- Break-glass with mandatory retrospective review.
- HIPAA-compliant audit trail (per ADR-0251 + audit-chain µservice).
- Passkey-first auth (per ADR-0188).
- Zero standing access (clinicians authenticate per-shift; access scoped per-encounter).

### E.4 Compliance

- HIPAA pack mandatory (HIPAA-2024).
- HITECH breach-notification ≤ 60 days.
- 42 CFR Part 2 (SUD-extra-sensitive data) supported.
- USCDI v4 + USCDI v5 (when published).
- HTI-1 Decision Support Intervention transparency (every CDS invocation auditable).
- TEFCA QHIN-compatible FHIR R4 surface.
- EU GDPR Article 9 (special category) supported via EU-GDPR pack.
- KR 의료법 §22 (10-year retention) via KR-MEDICAL-LAW-2024 pack.

### E.5 Interoperability

- FHIR R5 as default (per FR-PAT-004 etc.).
- FHIR R4 for compatibility (TEFCA, USCDI, 21st Century Cures).
- HL7 v2 only via `healthcare-integration` (no direct support in EMR).
- CDA / CCD via consolidated CDA R2.1 (per ONC requirements).
- C-CDA Continuity-of-Care Document export.
- DICOM only via `diagnostics` µservice.
- IHE profiles (PIX, XDS, XCA, PCC, QED) supported.

### E.6 Audit + Retention

- Every PHI read/write emits an audit event.
- Audit retention 7 years (HIPAA-2024 default) extensible per pack.
- Legal-hold supersedes retention.
- Per-employee + per-patient audit query.
- Tamper-evident sealing via `audit-chain` µservice.

### E.7 Localization + Internationalization

- UI in en-US default; per-tenant pack adds locales.
- Patient-education content multi-language per assigned locale.
- Date/time per ISO 8601 + tenant locale presentation.
- Address per ISO 19160 + USPS / KR-postal / EU-postal validators.

## F. Tenant Class Behavior Matrix

| tenant_class | retention default | EPCS required | TEFCA-QHIN | KR-HIRA codes | Deployment context default |
|---|---|---|---|---|---|
| b2b-healthcare-provider (US) | 7y | yes (DEA-registrant) | yes (opt-in) | n/a | guest-on-aws / on-prem |
| b2b-healthcare-network (US-IDN) | 7y; many depts 10y | yes | yes | n/a | on-prem + multi-region |
| b2b-academic-medical-center | 7y; many depts 10y | yes | yes | n/a | on-prem (sovereign-cell) |
| b2b-community-clinic | 7y | yes | optional | n/a | oyatie-public-cloud |
| b2b-ambulatory-surgery-center | 7y | yes | optional | n/a | oyatie-public-cloud / guest-on-oci |
| b2b-telehealth-platform | 7y | optional | optional | n/a | oyatie-public-cloud + edge |
| b2b-rural-health-clinic (FQHC) | 7y | optional | optional | n/a | oyatie-public-cloud |
| b2b-federally-qualified-health-center | 7y | optional | yes (HRSA-eligible) | n/a | guest-on-aws |
| b2b-skilled-nursing-facility | 7y | optional | optional | n/a | on-prem |
| b2b-home-health-agency | 7y | optional | optional | n/a | oyatie-public-cloud |
| KR-private-hospital (10y, HIRA reimbursement) | 10y | n/a | n/a | yes | on-prem + KR-residency |
| EU-private-hospital | per EU member state | n/a | n/a | n/a | colo (EU-sovereign cell) |

## G. Cross-µService Handoffs

EMR consumes from and produces to:

- **healthcare-integration:** consumes HL7 v2 + FHIR resources via gRPC bridge; consumes MPI patient-resolution; emits FHIR resources upstream for external EHR exchange (Care Everywhere / Carequality).
- **diagnostics:** dispatches lab + imaging orders; receives DiagnosticReport + Observation; ack critical results.
- **pharmacy:** dispatches MedicationRequest; receives MedicationDispense + MedicationAdministered; coordinates ePrescribing via NCPDP SCRIPT.
- **emergency:** dispatches emergency-encounter creation; receives ED-to-inpatient handoff with full chart preserved.
- **clinical-decision-support:** invokes CDS Hooks 2.0 at prescribe-time, order-time, view-time; receives BPA fire events.
- **care-management:** publishes episode-of-care discharge events; consumes longitudinal care plan updates.
- **cloud-iam:** authenticates clinicians + patients + proxies via passkey + risk-adaptive 2FA.
- **cloud-kms:** wraps per-tenant KEK; envelope-encrypts PHI columns.
- **cloud-storage:** stores blob attachments (images, scans, voice-dictations) under PHI-scoped buckets.
- **audit-chain:** emits all PHI-access + state-change events; tamper-evident sealing.
- **consent-graph:** evaluates patient-consent at FHIR-API-call time; resource-segmentation per 42 CFR Part 2.
- **workflow-engine:** orchestrates discharge-saga, proxy-grant-saga, break-glass-review-saga, legal-hold-lifecycle.
- **cloud-billing:** receives billing-code capture; sends back claim status.
- **observability:** SLO emission per ADR-0263.

## H. Out-Of-Scope

- HL7 v2 wire-protocol (owned by `healthcare-integration`).
- Lab/imaging instrument I/O (owned by `diagnostics`).
- Insurance eligibility 270/271 (owned by `cloud-billing`).
- Patient genome / lab pipeline (separate `genomics` µservice; not in current portfolio).
- Veterinary EMR (separate `vet-emr` µservice if ever scoped).
- Mental health 42 CFR Part 2 segmentation (extension; default supported).
- Long-term care MDS 3.0 (extension; pack-overlay).

## I. Success Metrics

- **Adoption:** 50+ pilot tenants by Q4 2027; 200+ paid tenants by 2028.
- **Latency SLO compliance:** ≥ 99.9% of measurement windows pass.
- **Migration win rate (Epic / Cerner → oyatie EMR):** 5+ flagship migrations by 2028.
- **Clinician satisfaction (KLAS-equivalent net promoter):** ≥ +30 within 12 months of go-live.
- **Patient portal MAU per tenant:** ≥ 35% of active patient panel.
- **Breach incidents:** zero data exfiltration > 500 records HIPAA-reportable in first 24 months.
- **FHIR uptake:** ≥ 100M FHIR API calls/month across portfolio by 2028.

## J. Open Questions

- **OQ-1:** Should EMR ship its own anesthesia-record sub-module or always delegate to a separate `anesthesia` µservice? — Pending council-clinical input.
- **OQ-2:** Should pediatric growth charts (WHO + CDC) ship in `emr.patient-education` content registry or as a tenant-class-overlay-pack? — Pending peds-IDN-pilot scope.
- **OQ-3:** Should the EMR surface a FHIR R6 (future) compatibility layer pre-emptively, or wait for HL7 finalization? — ADR-MS-002 recommends defer until R6 hits FHIR Working Group Ballot.
- **OQ-4:** Tele-ICU continuous monitoring stream — owned by EMR vital BC or by a peer `tele-monitoring` µservice? — Pending tele-icu-pilot scope.

## K. Detailed Tenant Class Examples (substantive walkthroughs)

### K.1 Marcus Chen Memorial Hospital, 600-bed academic medical center

- **Tenant identity:** `tenant-marcus-chen-memorial`
- **Tenant class:** `b2b-academic-medical-center`
- **Cell:** `cell-us-east-1-tier-0-hipaa-sovereign-007`
- **Cert level:** `healthcare-sovereign`
- **Installed packs:** `HIPAA-2024`, `SOC2-T2-2024`, `ISO-27001-2022`, `FERPA-2024` (academic-affiliated residents handling student-records), `FDA-21CFR-PART11-2024` (research arm), `EU-GDPR-2018-baseline` (overseas partnership).
- **EMR shape:** 2,800 clinicians + 18,000 employees; 350,000 active patients; 3,200 inpatient beds across 4 affiliated hospitals; 47 outpatient clinics; full ED + L&D + OR.
- **Cross-µservice handoff exposure:** `diagnostics` (15M lab orders/yr; 12M imaging studies/yr), `pharmacy` (5M outpatient + 2M inpatient meds/yr), `emergency` (180k ED visits/yr), `clinical-decision-support` (~250 CDS invocations per active inpatient per stay), `cloud-billing` (2.1M claims/yr).
- **Performance bar:** chart-open p99 ≤ 800ms across all 4 hospitals; order-entry p99 ≤ 200ms; FHIR read p99 ≤ 150ms.
- **DR:** multi-region active-active in `us-east-1` + `us-west-2`; RPO < 60s tested quarterly.
- **Migration story:** transitioning from Epic Hyperspace (started 2010); 18-month dual-write strangler under `migration-from-epic` pack overlay.

### K.2 Yejin Park Family Medicine (3-physician clinic)

- **Tenant identity:** `tenant-yejin-park-family-medicine`
- **Tenant class:** `b2b-community-clinic`
- **Cell:** `cell-us-west-2-tier-1-hipaa-022`
- **Cert level:** `hipaa-certified`
- **Installed packs:** `HIPAA-2024`, `SOC2-T2-2024`.
- **EMR shape:** 3 physicians + 5 MAs + 2 RNs + 1 office manager; ~5,200 active patients; outpatient-only; 22 daily-visit average.
- **Performance bar:** chart-open p99 ≤ 800ms; tablet-first MA UX.
- **DR:** RPO < 60s in same region; RTO < 15min.
- **Migration story:** transitioning from athenaClinicals; clean-cutover with FHIR Bulk Data import.

### K.3 KR-private-hospital network (Asan Medical Center analogue)

- **Tenant identity:** `tenant-kr-private-hospital-network`
- **Tenant class:** `KR-private-hospital`
- **Cell:** `cell-ap-northeast-2-tier-0-healthcare-sovereign-kr-002`
- **Cert level:** `healthcare-sovereign-kr`
- **Installed packs:** `HIPAA-2024`, `KR-PIPA-2023-amendment`, `KR-MEDICAL-LAW-2024`, `KR-ISMS-P-2024`, `KR-CSAP-v3.1`.
- **EMR shape:** 2,700 beds across 3 hospitals; KR-residency mandatory; 의료법 §22 10-year retention; HIRA reimbursement-code generation; Naver Cloud / KT Cloud substrate.
- **KR-specific behavior:**
  - Retention default = 10 years (overrides HIPAA 7-year default).
  - HIRA code-set captured at billing-code BC instead of CMS HCPCS.
  - Patient-portal default-language = ko-KR; clinician UI also ko-KR.
  - 주민등록번호 (resident registration number) handled per KR-PIPA pseudonymization rule.
- **DR:** all data in KR; cross-border export blocked.

### K.4 Telehealth-Only Startup (oyatie-public-cloud SaaS)

- **Tenant identity:** `tenant-bright-telehealth`
- **Tenant class:** `b2b-telehealth-platform`
- **Cell:** `cell-us-east-1-tier-1-hipaa-013`
- **Cert level:** `hipaa-certified`
- **EMR shape:** 80 employed clinicians; nationwide multi-state licensing; ~15,000 active patients; 100% telehealth; oyatie-public-cloud SaaS.
- **Specifics:** EPCS optional; encounter-type-mix = 95% telehealth-video + 5% telehealth-audio-only.

### K.5 Long-term-care SNF chain (Wave-2+ pack overlay)

- **Tenant identity:** `tenant-evergreen-snf-chain`
- **Tenant class:** `b2b-skilled-nursing-facility`
- **EMR shape:** 22 SNFs; ~3,800 residents.
- **Wave-2 pack overlay required:** MDS 3.0 assessment forms; CMS minimum data set submission.
- **Status:** EMR core supports SNF demographic + encounter; MDS workflow ships in Wave-2.

## L. Detailed Failure Modes + Mitigations

### L.1 Cell-kill (entire cell unavailable)

- **Impact:** all tenants pinned to this cell lose chart access.
- **RPO:** ≤ 60 seconds (cross-region async replication catches up).
- **Mitigation:** DR cell pre-warm; DNS shift in 30s; tenant data restored from cross-region async replica.
- **Drill cadence:** quarterly per ADR-0241.

### L.2 Postgres+Citus coordinator failure

- **Impact:** write traffic halts; reads from worker shards continue (read-only mode).
- **Mitigation:** Patroni-managed coordinator failover; election within 10s; readiness probe surfaces unhealthy state to load-balancer.

### L.3 Cedar evaluator slowdown

- **Impact:** all PHI-touching actions slow proportional to Cedar latency.
- **Mitigation:** per-cell Cedar evaluator cache; circuit-breaker fast-fail to "deny-default" if Cedar > deadline; SLO `cedar-evaluation-latency.openslo.yaml` ≤ 20ms p99.
- **Operations:** alert on burn-rate; auto-restart of Cedar evaluator pods.

### L.4 audit-chain backpressure

- **Impact:** audit emission lag; SLO breach.
- **Mitigation:** per-cell buffered queue; spill-to-disk; reconcile when backpressure clears.
- **Hard-stop:** if audit-chain unreachable > 60s, EMR refuses NEW write actions (read continues per `audit-everything.cedar` exception clause for `health.check` + `metrics.read`).

### L.5 CDS Hooks 2.0 service unavailable

- **Impact:** clinician sees a "CDS unavailable" inline warning at prescribe / order time.
- **Behavior:** orchestration continues; the order proceeds without CDS Cards (the clinician is responsible for the clinical decision).
- **Mitigation:** circuit-breaker; per-tenant fallback policy (default: allow-with-warning; tenant-pack may override to deny).

### L.6 Pharmacy µservice unavailable (e.g., NCPDP SCRIPT downstream blip)

- **Impact:** ePrescribing dispatches queue.
- **Mitigation:** outbox pattern; retries; clinician sees "queued for pharmacy"; manual fallback to phone-Rx supported.

### L.7 Diagnostics µservice slow

- **Impact:** lab/imaging order entry dispatches slow.
- **Mitigation:** outbox + async; order persists locally with `DISPATCHED_PENDING`; clinician sees confirmation.

### L.8 Per-clinician session compromise

- **Impact:** unauthorized PHI access.
- **Mitigation:** passkey hardware-bound; risk-adaptive 2FA step-up; session-anomaly detection (per audit-chain); break-glass-style mandatory review on suspicious session.

### L.9 Mass-data-export attempt (insider risk)

- **Impact:** insider exfiltration.
- **Mitigation:** bulk-export gated by Cedar + audit-emission; tenant-policy ceiling on per-session export volume; alert + auto-suspend on threshold breach.

### L.10 Tenant pack downgrade attempt (HIPAA uninstall while PHI present)

- **Impact:** would violate retention + protection rules.
- **Mitigation:** pack uninstall workflow refuses while PHI exists; requires erase-or-reclassify-all per ADR-0251.

## M. Detailed Acceptance Criteria for Wave 15M-B

This wave authors PRD + ARCHITECTURE + companion artifacts. The acceptance bar:

- M.1 PRD ≥ 800 lines of substantive content (no template stamping; per ADR-0212).
- M.2 ARCHITECTURE ≥ 600 lines; 12-layer enum mapped; all cross-µservice handoffs declared.
- M.3 README ≥ 300 lines.
- M.4 manifest.json includes binding ADRs (0131, 0132, 0244, 0251, 0328, 0329, 0330, 0331, 0332) + tenant_class_eligibility + paid_billing_components + 6 deployment_contexts + supported_oses.
- M.5 OpenAPI 3.2.0 surface ≥ 30 paths covering FHIR R5 resources.
- M.6 AsyncAPI 3.1.0 surface ≥ 20 channels covering clinical events.
- M.7 proto3 surface ≥ 8 services.
- M.8 OpenSLO ≥ 10 SLO files.
- M.9 Cedar policies ≥ 6 covering the principal × action × resource matrix.
- M.10 6 IAC contexts populated with main.tf modules.
- M.11 3 service-scoped ADRs (MS-001, MS-002, MS-003).
- M.12 10 implementation plans (IP-EMR-001..010) with substantive (not stamped) content.
- M.13 Competitor parity matrix ≥ 100 capabilities (UNION Epic + Cerner + athena + secondary).
- M.14 supported-oses.json with 13 Tier-1 OSes.

## N. References

- Epic Hyperspace 2024 reference UX; Epic FHIR R5 docs (https://fhir.epic.com/).
- Oracle Health Cerner Millennium 2024 documentation; HL7 FHIR R4 + R5 normative content.
- athenahealth Architecture overview 2024 (athena Marketplace docs).
- HHS ONC Cures Act Final Rule (45 CFR §170.215 USCDI; §170.315 Certification).
- HHS HTI-1 Final Rule (2024) — DSI transparency requirements.
- CMS-0057-F (2024) Prior Authorization Final Rule.
- TEFCA Common Agreement v2 (2024).
- HL7 FHIR R5 normative content (October 2023 normative ballot + 2024 errata).
- USCDI v4 + USCDI v5-draft.
- Joint Commission NPSG 2024.
- ADR-0131 per-microservice flat layout.
- ADR-0132 single-concern + suite dissolution.
- ADR-0244 tenant-as-universal-scoping-primitive.
- ADR-0251 compliance pack + cell certification levels.
- ADR-0328 substance-bar as canonical sequence + batch discipline.
- ADR-0329 / ADR-0330 / ADR-0331 / ADR-0332 (in flight — healthcare domain decomposition + EMR foundational role).
- ADR-MS-001 EMR bounded contexts.
- ADR-MS-002 FHIR R5 default.
- ADR-MS-003 Mobile-first patient portal.

## O. Glossary

- **BAA** — Business Associate Agreement (HIPAA): a contract between a covered entity and a business associate establishing safeguards for PHI.
- **BCMA** — Barcode Medication Administration: nurse scans patient wristband + medication barcode + their own ID at administration.
- **BC** — Bounded Context (DDD).
- **BPA** — Best Practice Advisory (Epic term): pop-up clinical guidance.
- **CCD** — Continuity of Care Document (CDA R2.1 template).
- **CDS Hooks** — Clinical Decision Support Hooks (HL7-defined CDS invocation pattern).
- **CHD** — Cardholder Data (PCI DSS).
- **C-CDA** — Consolidated Clinical Document Architecture.
- **CMIO** — Chief Medical Information Officer.
- **CPOE** — Computerized Physician Order Entry.
- **CRDT** — Conflict-free Replicated Data Type (for note autosave).
- **DEA** — Drug Enforcement Administration (US).
- **DPIA** — Data Protection Impact Assessment (GDPR Article 35).
- **DPO** — Data Protection Officer.
- **EHR** — Electronic Health Record (patient-facing term; same as EMR in oyatie).
- **EMR** — Electronic Medical Record (clinician-facing term).
- **EPCS** — Electronic Prescribing of Controlled Substances (DEA 21 CFR §1311).
- **ESI** — Emergency Severity Index (1–5).
- **FHIR** — Fast Healthcare Interoperability Resources (HL7).
- **FQHC** — Federally Qualified Health Center.
- **HIM** — Health Information Management.
- **HIRA** — Health Insurance Review & Assessment Service (KR).
- **HITECH** — Health Information Technology for Economic and Clinical Health Act (2009).
- **HTI-1** — Health Data, Technology, and Interoperability Final Rule (2024).
- **IDN** — Integrated Delivery Network.
- **IHE** — Integrating the Healthcare Enterprise (profiles).
- **LOINC** — Logical Observation Identifiers Names and Codes (lab + vitals).
- **MAR** — Medication Administration Record.
- **MPI** — Master Patient Index (cross-EHR patient identity).
- **NCPDP SCRIPT** — National Council for Prescription Drug Programs SCRIPT standard (ePrescribing).
- **NDC** — National Drug Code (FDA-assigned).
- **NPSG** — National Patient Safety Goal (Joint Commission).
- **OASIS** — Outcome and Assessment Information Set (home-health).
- **OCR** — Office for Civil Rights (HHS).
- **ONC** — Office of the National Coordinator for Health Information Technology.
- **PDMP** — Prescription Drug Monitoring Program (state-level).
- **PHI** — Protected Health Information (HIPAA defined).
- **PIPA** — Personal Information Protection Act (KR).
- **POA** — Power of Attorney.
- **QHIN** — Qualified Health Information Network (TEFCA).
- **REMS** — Risk Evaluation and Mitigation Strategy (FDA).
- **ROI** — Release of Information.
- **RxNorm** — National Library of Medicine's normalized drug naming system.
- **SEP-1** — CMS Severe Sepsis and Septic Shock Management Bundle.
- **SNOMED CT** — Systematized Nomenclature of Medicine — Clinical Terms.
- **SUD** — Substance Use Disorder (42 CFR Part 2).
- **TEFCA** — Trusted Exchange Framework and Common Agreement (ONC).
- **TJC** — The Joint Commission.
- **USCDI** — U.S. Core Data for Interoperability (ONC).
- **VistA** — Veterans Health Information Systems and Technology Architecture (VA).
- **의료법** — KR Medical Service Act.

## P. Wave-2+ Roadmap (out-of-scope for Wave 15M-B authoring)

| Wave | Scope |
|---|---|
| Wave 16 | Pediatric growth chart pack (WHO + CDC); OB module (Stork-equivalent) |
| Wave 17 | Anesthesia record (separate `anesthesia` µservice candidate) |
| Wave 18 | OR / surgical scheduling (separate `surgical` µservice candidate) |
| Wave 19 | Specialty modules: Cardiology, Oncology, Behavioral Health (separate µservices) |
| Wave 20 | LTC pack (MDS 3.0); Home Health pack (OASIS) |
| Wave 21 | EU EHDS bridge; Schrems-II-aligned cross-border |
| Wave 22 | FHIR R6 compatibility shim (pending HL7 normative) |

## Q. Risk Register

### Q.1 Regulatory drift

- **R-1 (HHS HTI-2 / HTI-3 publication):** ONC may issue additional certification rules requiring USCDI v5 + Predictive DSI registry expansion. *Mitigation:* DSI transparency hooks ship in EMR audit emission already; USCDI roadmap monitored in Wave-21 backlog.
- **R-2 (Schrems III adequacy decision):** EU-US data transfer adequacy could be invalidated, forcing tenant-data-residency reshuffles. *Mitigation:* EU-sovereign cells are already a deployment option; oyatie can re-pin EU-resident tenants.
- **R-3 (DEA EPCS amendment):** DEA could expand EPCS scope to Schedule V or change attestation strength. *Mitigation:* EPCS policy is parameterized; pack-overlay update lands without code changes.
- **R-4 (KR 의료법 amendment):** retention rules can change; HIRA reimbursement code-set updates seasonally. *Mitigation:* KR pack-overlay-driven; quarterly KR-tenant review.

### Q.2 Operational

- **R-5 (Per-tenant noisy neighbor):** large IDN tenant on shared cell starves smaller tenants. *Mitigation:* per-tenant rate limiting + per-tenant SLO-burn-aware admission control; tier-0 cells reserved for large tenants.
- **R-6 (Postgres+Citus shard imbalance):** uneven tenant size causes hot-shard. *Mitigation:* rebalance routine; large tenants get dedicated worker.
- **R-7 (Kafka backpressure on event volume):** large IDN may emit 100k events/sec at peak. *Mitigation:* per-cell Kafka sized for 5x peak; per-topic compression + batch tuning; outbox-spill-to-disk under back-pressure.

### Q.3 Migration

- **R-8 (Epic Care Everywhere export throttle):** Epic may throttle Bulk Data export; long-running migrations delayed. *Mitigation:* tenant migration runbook anticipates throttle; multi-month window OK; resumable batches.
- **R-9 (Patient ID collision during merge):** inbound MRN collisions cause merge ambiguity. *Mitigation:* MPI substrate adjudicates; 30-day reversibility window per FR-PAT-003.

### Q.4 Security

- **R-10 (Insider mass-exfil):** privileged clinician exfiltrates large patient cohort. *Mitigation:* per-session export ceiling; auto-suspend on threshold; mandatory tenant-policy review.
- **R-11 (Compromised passkey):** stolen mobile device with active patient passkey. *Mitigation:* hardware-bound passkey; revocation via cloud-iam; tenant-DPO breach-notification workflow.
- **R-12 (Cell-level vendor compromise — KMS host):** mitigation: BYOK + multiple-region KEK roots; key-rotation runbook.

## R. Open Items for Council Decision (post-Wave-15M-B)

- Should portal mobile apps be tenant-white-label-only or also offer an "oyatie" branded app for B2C-style direct-to-patient marketing? — Wave-21 council-product.
- Should EMR ship a "research-warehouse projection" worker that de-identifies + ships data to a peer `data-warehouse` µservice automatically (consent-grant-conditional)? — pending consent-graph + data-warehouse PRD-coupling.
- Should the 30-day patient-merge reversibility window be tenant-configurable up to 1 year (HIM-extendable)? — preference for fixed 30d with HIM-extend-to-365d-with-attestation; pending council-clinical.
- Should EMR vital BC also accept consumer wearable streams (Apple Watch, Fitbit) directly, or always via patient-portal upload? — Wave-2+ scope.

## S. Lifecycle commitments

EMR carries an explicit lifecycle posture to its tenants:

- **Backward compatibility:** REST + AsyncAPI + gRPC contracts are versioned (v1, v2, …); a major version bump is the only path to break compatibility, and every break is preceded by a 12-month deprecation window per `feedback_no_silent_regression`.
- **Long-term support:** every EMR major release receives at least 36 months of security + regulatory-update patches.
- **Pack lifecycle:** when a regulatory pack version sunsets (e.g., HIPAA-2024 → HIPAA-2027), EMR runs both packs in parallel for at least 18 months to enable tenant-driven migration.
- **Deprecation:** any EMR endpoint / event / RPC marked deprecated is announced 90 days before removal; downstream consumers are notified per ADR-0263 emission contract.
- **Tenant exit:** any tenant may export their full chart corpus via `/fhir/$export` Bulk Data and decommission their EMR install with a 30-day data-retention countdown per HIPAA-2024 pack uninstall workflow.

## T. Pilot Tenant Plan (Wave 16+ — informational, not in Wave 15M-B scope)

EMR's first three pilot tenants are slated to onboard in Wave 16 (sequence: small clinic → community hospital → academic medical center) so the µservice is exercised across tenant-class diversity from day one. Per Wave 15M-B authoring discipline, these pilots are informational only — actual onboarding is a separate IP series.

## U. Out-of-Wave Considerations

This wave (15M-B) authors the µservice scaffold + contracts + policies + IaC modules. The following are intentionally deferred:

- Rust implementation of all crates (IP-EMR-001..010 sequence).
- Mobile app source (Swift + Kotlin) — owned by frontend µservices.
- Per-tenant pilot onboarding runbooks.
- Production-grade Helm chart values (templates exist; tenant-specific values per IP).
- Multispectrum review evidence packets (owned by the changeset reviewer-agents).

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

<!--
COMPLETION REPORT (Wave 15M-B):
sole_owner: claude-emr-author-wave-15m-b
date: 2026-05-21
deliverables_complete:
  - PRD.md (this file) — 822 lines
  - ARCHITECTURE.md — 612+ lines (companion)
  - README.md — 318+ lines (companion)
  - manifest.json — full schema with binding ADRs 0131 0132 0244 0251 0328 0329 0330 0331 0332
  - contracts/openapi-emr-v1.yaml — FHIR R5 surface
  - contracts/asyncapi-emr-v1.yaml — clinical events
  - contracts/proto/emr.proto — gRPC
  - slos/ — 11 OpenSLO files
  - policies/ — 7 Cedar policies
  - iac/ — 6 deployment contexts populated
  - decisions/ — 3 ADRs (MS-001/002/003)
  - implementation-plans/ — IP-001..IP-010
  - competitor-parity-matrix.md — 100+ capabilities (Epic + Cerner + athenahealth UNION)
  - supported-oses.json — 13 Tier-1 OSes
counterparts_targeted:
  primary: [Epic, Oracle Health Cerner, athenahealth]
  secondary: [Allscripts/Veradigm, Meditech, eClinicalWorks, NextGen]
substance_bar: ADR-0212 compliant — bespoke content per BC; no template stamping
canonical_sources_consulted:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0131 (flat layout)
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0132 (single-concern)
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0244 (tenant scoping)
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0251 (compliance pack)
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328 (substance bar)
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/ (structural reference)
  - /Users/jasonlee/oyatie/microservices/cloud-iam/ (structural reference)
  - /Users/jasonlee/oyatie/microservices/cloud-iac/ (structural reference)
notes_to_orchestrator:
  - ADR-0329/0330/0331/0332 referenced as in-flight; binding once accepted
  - emr is the foundational healthcare µservice; depth bar matters
  - cross-µservice handoffs declared but not implemented (peer µservices author independently)
  - no commits per execution rules
-->
