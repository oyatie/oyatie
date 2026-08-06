---
id: ADR-0332
title: Healthcare Domain Decomposition — Eight New Domain Microservices + Integration-Substrate Narrowing
status: Superseded
date: 2026-05-21
deciders: council-architecture, council-product, council-clinical, council-privacy, council-security, axis-healthcare-integration, ops-compliance, ops-sre-reliability
owner: council-architecture
supersedes: []
amends:
  - microservices/healthcare-integration/manifest.json (scope narrowed to integration substrate only)
  - microservices/healthcare-integration/PRD.md (de-scoped to FHIR/HL7v2/DICOM broker concern)
  - microservices/diagnostics/PRD.md (de-scoped to lab + pathology only)
  - microservices/imaging/PRD.md (promoted as imaging authority)
superseded_by: [ADR-0701]
related:
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0316-capability-tier-doctrine.md
  - ADR-0321-b2b-saas-industry-leader-universe.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0327-wave-3-completion-criteria-and-promotion-gates.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/per-microservice-flat-layout.json
  - /specs/cell-certification-level-matrix.json
  - /specs/compliance-pack-schema.json
related_memory:
  - feedback_microservice_ownership_coherence
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_substrate_vs_product_layering
  - feedback_canonical_base_localization
  - feedback_amazon_shape_cellular_architecture
  - feedback_build_ahead_of_certification
doc_class: Architecture-Decision-Record
authority_tier: 1
line_floor: 600
keystone_position: healthcare-decomposition-2026-05-21
phase_assignment: Phase 4 (B2B / Industry-Leader Long Tail)
big_8_priority: Healthcare cluster (post-HR, post-CRM, post-ITSM)
enforcement_status: advisory-until-eight-microservices-scaffold-lands
enforced_by:
  - cloud-ci/Rust gate packet per-microservice-layout
  - cloud-ci/Rust gate packet no-grouping
  - cloud-ci/Rust gate packet microservice-coherence-audit
  - cloud-ci/Rust gate packet healthcare-domain-decomposition (NEW lane added by this ADR)
  - cloud-ci/Rust gate packet hipaa-pack-coverage-per-healthcare-microservice
  - cloud-ci/Rust gate packet cross-microservice-handoff-coherence
purpose: >
  Decompose the existing healthcare-integration microservice (215 features
  across 14 domains) into eight new domain-scoped single-concern microservices
  (emr, diagnostics, imaging, emergency, pharmacy, patient-monitoring,
  clinical-decision-support, care-management) and narrow the existing healthcare-
  integration microservice to FHIR/HL7v2/DICOM integration substrate only.
  Each new microservice ships at hyperscaler-grade per ADR-0131 flat layout,
  ADR-0132 no-grouping forward policy, and ADR-0251 HIPAA pack mandatory for
  paid tenants. Each microservice has its own PRD, ARCHITECTURE, contracts,
  SLOs, Cedar policies, runbooks, IaC, IPs, and bespoke counterpart top-3
  parity rooted in named industry-leader software platforms.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0332: Healthcare Domain Decomposition

## Status

Accepted — 2026-05-21.

Enforcement status is `advisory-until-eight-microservices-scaffold-lands`. The
doctrine is authoritative for future authoring waves the moment this ADR lands;
the BLOCKER promotion happens once the eight new microservice folders exist
under `microservices/` with the minimum-viable anchor set (PRD, ARCHITECTURE,
manifest, compliance, contracts skeleton, SLO skeleton, Cedar skeleton).

This ADR is companioned by the Wave-15M plan at
`.omc/plans/healthcare-decomposition-plan-2026-05-21.md` and by the scope-
narrowing remediation notes at
`microservices/healthcare-integration/REMEDIATION-NOTES-2026-05-21.md`. The
three artifacts together form the canonical record of the decomposition.

## A. Context

### A.1 Named pressure: one µservice carrying fourteen domains

The microservice `microservices/healthcare-integration/` was authored under
Wave-3-I as a single µservice covering an extraordinary spread of clinical
surface. The Wave-4-rolling coherence audit at
`microservices/healthcare-integration/coherence-audit-2026-05-20.md` and the
companion feature-parity matrix at
`microservices/healthcare-integration/feature-parity-matrix-2026-05-20.md`
together enumerate 215 features across 14 domains:

- Domain A — FHIR API surface (29 features)
- Domain B — HL7 v2/v3 (29 features)
- Domain C — DICOM imaging (25 features)
- Domain D — EHR connectivity per major system (23 features)
- Domain E — Lab / imaging / pharmacy network integrations (12 features)
- Domain F — Patient matching / MPI (10 features)
- Domain G — Clinical data normalization (12 features)
- Domain H — IHE profile support (15 features)
- Domain I — Terminology services (18 features)
- Domain J — Longitudinal record assembly (11 features)
- Domain K — Consent management (14 features)
- Domain L — HIPAA pack readiness (32 features)
- Domain M — Compliance and sovereignty (12 features)
- Domain N — Operational surface (17 features)

That spread does not match the ADR-0131 + ADR-0132 single-concern doctrine.

ADR-0131 mandates one µservice equals one bounded context, one folder, one PRD,
one set of SLOs, one set of Cedar fragments, one IaC bundle, one release
pointer. ADR-0132 forbids new platform/bundle/vertical formations that contain
more than one user-facing concern.

A µservice that ships ePrescribe (NCPDP SCRIPT to Surescripts), C-STORE/C-FIND/
C-MOVE DICOM imaging, FHIR R5 longitudinal record assembly, MPI deterministic +
probabilistic patient matching, break-glass justification review, HL7v2 ADT/
ORM/ORU routing, ED triage protocols, ICU vital-sign telemetry, and clinical
decision-support evidence-based pathways under one PRD is by definition a
suite. That is the failure mode this decomposition resolves.

### A.2 Named pressure: industry-leader counterpart space is plural

Looking at the modern healthcare software market (2024–2026), the major
industry-leader platforms are organised by clinical domain, not as monolithic
"healthcare" platforms:

- Electronic Medical Records (EMR/EHR): Epic, Oracle Health (formerly Cerner),
  athenahealth, Allscripts/Veradigm, MEDITECH, eClinicalWorks, NextGen
  Healthcare. These platforms are full-platform EMRs themselves (vendor brands);
  oyatie's `emr` µservice projects an EMR substrate that competes shape-for-
  shape with their EMR-core surfaces.
- Laboratory and Pathology Diagnostics: Sunquest, Clinisys, Oracle Health
  PathNet, Epic Beaker + Beaker AP, Roche Navify, LabCorp Diamond,
  Quest Lab Connect.
- Imaging / PACS / VNA: GE Centricity, Philips IntelliSpace, Sectra PACS+VNA,
  Epic Radiant, Agfa HealthCare Enterprise Imaging, Change Healthcare
  Stratus Imaging.
- Emergency Department: T-System (now Hyland T-System), Wellsoft EDIS, Epic
  ASAP, Cerner FirstNet, Picis CareSuite ED, Allscripts EDIS.
- Pharmacy Management: Cerner Pharmacy Manager, Epic Willow Inpatient and
  Willow Ambulatory, McKesson EnterpriseRx, BD Pyxis (automated dispensing
  cabinet integration), Omnicell (XT, IV, Performance Center).
- Patient Monitoring: Philips PIC iX (PIIC iX) and IntelliVue, GE CARESCAPE,
  Mindray BeneVision, Masimo Patient SafetyNet, Welch Allyn Connex,
  Spacelabs Xhibit.
- Clinical Decision Support: UpToDate (Wolters Kluwer), Wolters Kluwer
  Lexicomp, IBM Micromedex (Merative), Epic Best Practice Advisories (BPA),
  Cerner Discern, ClinicalKey (Elsevier), DynaMed.
- Care Management / Population Health: Salesforce Health Cloud, Epic Healthy
  Planet, Innovaccer, Arcadia, Optum Care Coordination, HMS Carenet.
- Healthcare Integration: Redox, Mirth (NextGen Connect), Health
  Gorilla, Lyniate Rhapsody (formerly Corepoint), Iguana / iNTERFACEWARE,
  InterSystems HealthShare, AWS HealthLake, Google Cloud Healthcare API.

Each of these clusters is a distinct competitive market with distinct buyers,
distinct procurement cycles, distinct SLO targets, distinct compliance shapes,
distinct vocabulary, and distinct integration counterparts. Bundling all of
them under one `healthcare-integration` µservice forces oyatie to ship a vendor
suite, not a substrate.

### A.3 Named pressure: ADR-0250 build-ahead-of-certification

ADR-0250 mandates building certified shape day one, never retrofitting
compliance. HIPAA is the obvious compliance anchor for every clinical surface,
but each domain has its own additional regulatory shape:

- EMR — HIPAA, ONC Cures Act API conditions (FHIR R4 mandate under
  45 CFR §170.315), 21st Century Cures Act information-blocking rules,
  HITECH, state-level licensure requirements.
- Diagnostics — CLIA (Clinical Laboratory Improvement Amendments,
  42 CFR §493), CAP accreditation requirements, FDA in-vitro diagnostic (IVD)
  regulations, ISO 15189, GxP evidence retention.
- Imaging — DICOM PS3 standards, IHE Radiology profiles, ACR accreditation,
  MQSA where mammography applies, FDA SaMD for image AI, and dose-monitoring
  evidence where required.
- Emergency — EMTALA (Emergency Medical Treatment and Labor Act,
  42 USC §1395dd), state EMS protocols, NEMSIS (National EMS Information
  System) v3.5 reporting.
- Pharmacy — DEA EPCS (Electronic Prescriptions for Controlled Substances,
  21 CFR §1311), NCPDP SCRIPT 2017+, state PDMP (Prescription Drug
  Monitoring Program) integration, 340B Drug Pricing Program rules, USP 797/
  800 compounding.
- Patient Monitoring — FDA 21 CFR §820 quality system (for medical-device
  data system integration), IEC 60601-1 (medical-electrical safety),
  IEEE 11073 medical-device communication standards, FDA SaMD (Software
  as a Medical Device) classification.
- Clinical Decision Support — FDA SaMD classification, FDA CDS guidance
  (Sept 2022 finalised), EU MDR for clinical decision-support software,
  EU AI Act high-risk classification for clinical AI.
- Care Management — HIPAA, CMS Star Ratings (Medicare Advantage),
  MIPS/MACRA quality reporting, state Medicaid care-management rules,
  HEDIS reporting.
- Healthcare Integration (substrate-narrowed) — HIPAA, 21st Century Cures
  Act information-blocking, ONC §170.315(g)(10) Standardised API,
  TEFCA framework.

Each domain carries a distinct compliance shape, and per ADR-0250 each must
be authored with its compliance pack day one. This is impossible to do
substantively under one bundled PRD; per-domain authoring is the only path.

### A.4 Named pressure: clinical workflow distinctness

Hospital and ambulatory clinical workflows split cleanly by domain:

- An EMR's bounded contexts are patient record, encounter, problem list,
  allergy, immunization, vitals, charting, and clinical note.
- A diagnostics platform's bounded contexts are lab order, lab result,
  pathology case, specimen, reference range, reflex testing, critical result,
  and result delivery.
- An imaging platform's bounded contexts are imaging order, study, series,
  instance, PACS index, VNA object custody, radiologist worklist, read report,
  hanging protocol, prior comparison, dose tracking, and image AI.
- An ED workflow's bounded contexts are triage acuity (Emergency Severity
  Index ESI-1..ESI-5), trauma activation, mass-casualty incident triage,
  EMS handoff, ED registration, ED tracking board, disposition.
- A pharmacy workflow's bounded contexts are medication order, drug
  interaction check, formulary check, dispense, barcode medication
  administration (BCMA), pharmacy intervention, controlled-substance
  custody chain.
- A patient-monitoring workflow's bounded contexts are vital-sign
  acquisition (HR, BP, SpO2, RR, T), continuous waveform capture (ECG,
  EEG, EMG), alarm, alarm fatigue management, remote patient monitoring
  (RPM), ICU/CCU integration.
- A clinical-decision-support workflow's bounded contexts are clinical
  pathway, drug interaction alert, dose check, allergy alert, evidence-
  based recommendation, BPA (Best Practice Advisory), order set.
- A care-management workflow's bounded contexts are care plan, care
  transition, care team assignment, care coordination message,
  population stratification, risk-score adjustment, intervention.

These are not interchangeable contexts. Folding them under one µservice
forces a single state machine, a single bounded context, a single audit
event class, and a single SLO budget across surfaces that have distinct
throughput envelopes, distinct failure modes, distinct alarm fatigue rules,
and distinct break-glass semantics.

### A.5 Authoritative anchors

Anchor 1: ADR-0131 (per-microservice flat layout). The one-µservice-one-
folder doctrine. Source of the universal artifact-layout pattern.

Anchor 2: ADR-0132 (no-grouping forward-policy). Forbids new bundle / industry /
vertical µservices that contain more than one user-facing concern.

Anchor 3: ADR-0251 (compliance pack + cell certification levels). Source of
HIPAA pack lifecycle, BAA template requirement, breach notification workflow,
PHI data class, and cell certification level (`hipaa-certified`) matrix.

Anchor 4: ADR-0250 (build-ahead-of-certification). Mandates certified shape
day one for every microservice that handles regulated data.

Anchor 5: ADR-0328 §D-1 (5-phase canonical build sequence). Healthcare
belongs to Phase 4 (B2B/industry-leader long-tail), after the Big 8 core
enterprise cluster but inside the Phase 4 sub-sequence.

Anchor 6: ADR-0316 (capability-tier doctrine). Each new µservice carries its
own capability-tier matrix (Bronze / Silver / Gold / Platinum) projected as
UX surface, NOT as directory split.

Anchor 7: ADR-0244 (tenant primitive). Every clinical µservice is tenant-
scoped; PHI cannot cross tenants.

Anchor 8: ADR-0263 (audit emission contract). Every clinical state
transition emits to the audit-chain with a class binding.

Anchor 9: existing healthcare-integration coherence audit at
`microservices/healthcare-integration/coherence-audit-2026-05-20.md` which
documents the 215-feature spread that triggered this decomposition.

### A.6 What this ADR is not

This ADR is not a code change. It is a structural decision that authorises
the eight new microservice folders and narrows the existing healthcare-
integration folder. It does not author the eight µservices' content; that
authoring is governed by the Wave-15M plan companioned with this ADR.

This ADR does not retire the existing healthcare-integration µservice. The
existing folder remains; its scope narrows to integration substrate only
(FHIR/HL7v2/DICOM broker concern with Redox / Mirth / Health
Gorilla counterpart shape preserved). The capabilities that previously
lived under the bundled µservice migrate to the new domain-specific
µservices per §H.

This ADR does not pre-empt earlier-phase work. Healthcare is Phase 4 per
ADR-0328 §D-1.93; the eight µservices are authored after Phase 0–3 + Big 8
core enterprise cluster (HR / Payroll / Accounting / CRM / ITSM / ERP-
core / SCM / customer-data-platform) reaches the substance bar.

## B. Decision

### B.1 Decision statement

Effective immediately, the existing `microservices/healthcare-integration/`
microservice is narrowed in scope to the FHIR/HL7v2/DICOM integration
substrate concern only. Its top-3 counterpart set (Redox / Mirth /
Health Gorilla) is preserved; all domain-specific clinical capabilities
migrate to the eight new microservices below.

Eight new single-concern domain microservices are authorised under
`microservices/`. Each is a flat per-µservice folder conforming to ADR-0131,
each carries one bounded-context cluster, each carries its own PRD,
ARCHITECTURE, manifest, compliance, contracts, SLOs, Cedar policies,
runbooks, IaC, IPs, capabilities, and catalog rows. Each carries the HIPAA
pack as mandatory for paid tenants per ADR-0251.

| # | New µservice slug | Concern | Top-3 industry counterparts |
|---|---|---|---|
| 1 | `emr` | Electronic medical records — patient record, encounter, problems, allergies, immunizations, vitals, charting, clinical note | Epic; Oracle Health (Cerner Millennium); athenahealth |
| 2 | `diagnostics` | Lab + pathology orders, specimens, results, reference ranges, reflex testing, critical results, and report delivery | Sunquest / Clinisys LIS; Oracle Health PathNet; Epic Beaker + Beaker AP |
| 3 | `imaging` | Imaging orders, PACS/VNA, DICOM object custody, radiologist workflow, read reports, dose tracking, and image AI | GE Centricity; Philips IntelliSpace; Sectra PACS+VNA |
| 4 | `emergency` | ED triage, trauma protocols, mass-casualty, EMS handoff, ED registration, ED tracking board, disposition | T-System (Hyland); Wellsoft EDIS; Epic ASAP |
| 5 | `pharmacy` | Medication management, ePrescribe, drug interactions, formulary, dispensing, BCMA scanning, controlled-substance custody | Cerner Pharmacy Manager (Oracle Health); Epic Willow; BD Pyxis |
| 6 | `patient-monitoring` | Vital signs telemetry, continuous waveforms, alarms, alarm fatigue, remote patient monitoring, ICU/CCU integration | Philips PIC iX (PIIC iX); GE CARESCAPE; Mindray BeneVision |
| 7 | `clinical-decision-support` | Clinical pathways, drug interaction alerts, evidence-based recommendations, BPAs, order sets, dose checks | UpToDate (Wolters Kluwer); Wolters Kluwer Lexicomp; IBM Micromedex (Merative) |
| 8 | `care-management` | Care plans, care transitions, population health, care coordination, risk stratification, intervention tracking | Salesforce Health Cloud; Epic Healthy Planet; Innovaccer |

The existing `healthcare-integration` µservice continues with its preserved
top-3 (Redox; Mirth Connect; Health Gorilla) and narrows to FHIR/HL7v2/DICOM
integration substrate only — the broker concern.

Total healthcare-domain microservices after this ADR: 9 (eight new + one
narrowed existing).

### B.2 What this decision does not do

This decision does not create a `microservices/healthcare/` parent folder
or a `microservices/healthcare-suite/` wrapper. ADR-0132 forbids that.

This decision does not move features between domain µservices without an
authoring IP. The Wave-15M plan owns the per-µservice IP roster.

This decision does not change the existing healthcare-integration runbooks,
contracts, or Cedar policies. It narrows the scope; cleanup of out-of-scope
artifacts is staged in the remediation notes.

This decision does not author the eight µservices in this PR. The PR that
lands this ADR ships exactly three files: this ADR, the Wave-15M plan, and
the healthcare-integration remediation notes.

This decision does not pre-empt the Big 8 sequence (HR / Payroll /
Accounting / CRM / ITSM / ERP-core / SCM / customer-data-platform). The
eight healthcare µservices are Phase 4 long-tail per ADR-0328 §D-1.93 and
are scheduled after Big 8 reaches the substance bar.

### B.3 Decision drivers

Driver 1: ADR-0131 + ADR-0132 single-concern doctrine forbids the existing
14-domain bundle.

Driver 2: each domain's industry-leader counterpart cluster is distinct.

Driver 3: each domain carries a distinct compliance shape beyond HIPAA.

Driver 4: each domain has a distinct SLO envelope (e.g., patient-monitoring
alarm latency is sub-second; ED disposition has 4-hour boarding KPI; care
management batch jobs run nightly).

Driver 5: cross-µservice handoffs (emr → pharmacy for ePrescribe;
diagnostics → emr for lab result; care-management → cloud-iam for caregiver
assignment) are naturally expressed by inter-µservice gRPC + Workflow events
under ADR-0145.

Driver 6: HIPAA pack inheritance applies uniformly per ADR-0251; per-
µservice pack adoption keeps the audit trail clear.

Driver 7: failure isolation. A bug in patient-monitoring alarm processing
should not degrade ED triage workflow. Independent µservices, independent
PDBs, independent HPAs, independent error budgets.

Driver 8: independent scaling. Patient-monitoring scales on monitored-bed
count; pharmacy scales on prescription-rate; care-management scales on
member-list size; emergency scales on visit-rate; emr scales on
charting-rate. One µservice cannot scale on seven unrelated dimensions
simultaneously.

## C. Per-µservice Scope Definition

### C.1 `microservices/emr/` — Electronic Medical Records

Concern: the canonical patient longitudinal record. The system of record
for patient demographics, encounters, problems, allergies, immunizations,
vital signs, clinical notes, advance directives, social history, and
family history.

Bounded contexts:
- `patient` — demographic record, identifier set, MRN, master patient
  index linkage to integration µservice
- `encounter` — inpatient stay, outpatient visit, ED visit, telehealth
  visit, with admission / discharge / transfer events
- `problem-list` — active and resolved problems, SNOMED-CT coded
- `allergy-intolerance` — patient allergies and intolerances, RxNorm +
  SNOMED-CT coded
- `immunization` — vaccinations administered, CVX coded, with VXU export
- `vitals` — point-in-time vital signs (separate from continuous
  telemetry, which lives in patient-monitoring)
- `clinical-note` — H&P, progress note, discharge summary, consult note,
  with SmartText / dot-phrase support
- `advance-directive` — DNR, MOLST/POLST, healthcare proxy
- `social-history` — substance use, occupation, education, sexual
  orientation, gender identity
- `family-history` — relatives' conditions

Top-3 industry counterparts:

**Epic Systems Corporation (Verona, WI).** Founded 1979. Holds the
largest US acute-care EMR market share (≈37% of US hospital beds per
KLAS 2024). Modules: Hyperspace (Hyperdrive on web), Chronicles
hierarchical database, EpicCare Ambulatory, EpicCare Inpatient, Stork
(L&D), Beacon (oncology), Willow (pharmacy), ASAP (ED), Lumens (mobile),
MyChart (patient portal), Cosmos (research), Garden Plot (small group),
App Orchard (SMART on FHIR marketplace, with FHIR R4 surface). Public
FHIR R4 API per ONC §170.315(g)(10). FHIR endpoints documented at
`https://fhir.epic.com/`. KLAS Best in KLAS: large hospital + integrated
delivery network (IDN) ranked #1 for 14+ consecutive years.

**Oracle Health (formerly Cerner Corporation, acquired by Oracle 2022).**
Holds ≈25% US acute-care EMR market share. Cerner Millennium (Code App
+ Bedrock + Millennium FHIR R4 API), PowerChart (clinician UI),
FirstNet (ED), CareAware (device-connectivity / RTLS), PharmNet
(pharmacy), Code (provider mobile). Federal contract: VA EHR
modernization (~$10B). Public FHIR R4 surface per ONC §170.315(g)(10).
Public docs at `https://fhir.cerner.com/`.

**athenahealth (Watertown, MA).** Founded 1997. ≈8% US ambulatory market
share; cloud-native SaaS EMR. athenaOne suite: athenaClinicals (EMR),
athenaCollector (RCM), athenaCommunicator (patient engagement),
athenaCoordinator (referral mgmt). Public FHIR R4 API at
`https://fhir.platform.athenahealth.com/`. Acquired by Veritas Capital
+ Evergreen Coast Capital 2022; ≈140k providers, 110M+ patient records.

Compliance shape:
- HIPAA-2024 pack (mandatory for paid; ADR-0251)
- ONC §170.315(g)(10) Standardized API for Patient and Population
  Services (FHIR R4 + US Core IG)
- 21st Century Cures Act information-blocking (45 CFR §171)
- HITECH meaningful use / promoting interoperability
- State-level licensure (varies by state; Texas Medical Practice Act,
  CA HSC, etc.)
- 42 CFR Part 2 (substance abuse confidentiality for SUD-related notes)
- 45 CFR §164.508 (PHI authorization)

SLO envelope:
- FHIR Patient $read p99 < 200ms
- FHIR Encounter search p99 < 400ms
- Clinical note write durability: 11 9s
- ADT event processing p99 < 1s
- Audit emission lag p99 < 5s

Cross-µservice handoffs:
- emr → pharmacy: ePrescribe (Workflow event `emr.medication.prescribed`
  consumed by pharmacy)
- emr → diagnostics: lab/pathology order (Workflow event
  `emr.order.created` consumed by diagnostics)
- emr → imaging: imaging order (Workflow event
  `emr.imaging-order.created` consumed by imaging)
- emr → emergency: ED visit start (Workflow event
  `emr.encounter.ed-arrival` consumed by emergency)
- emr → care-management: condition flag (Workflow event
  `emr.problem.added` consumed by care-management)
- emr ← patient-monitoring: vital-sign rollup (Workflow event
  `patient-monitoring.vitals.point-in-time` consumed by emr)
- emr ← clinical-decision-support: BPA trigger (Workflow event
  `cds.recommendation` consumed by emr UI)
- emr ↔ healthcare-integration: FHIR/HL7v2 ingress/egress
- emr → cloud-iam: caregiver assignment + role binding
- emr → consent-graph: patient consent state
- emr → audit-chain: every PHI access, mutation, and disclosure

### C.2 `microservices/diagnostics/` — Lab + Pathology

Concern: canonical lab and pathology diagnostic-evidence workflow.
Diagnostics owns lab/pathology order intake, specimen tracking, lab
analysis, result computation, reference-range application, reflex
testing, pathology case management, critical-result escalation, and
result-to-EMR delivery.

Bounded contexts:
- `lab-order` — single-test or panel order, with specimen-source code,
  ordering provider, ordering location, priority (STAT, routine, ASAP),
  and collection instructions
- `lab-result` — numeric result, qualitative result, microbiology
  isolate, antibiogram, with LOINC code, units (UCUM), reference range,
  interpretation flag (H/L/A/HH/LL/AA/N), result status (preliminary,
  final, corrected)
- `pathology-case` — surgical pathology case, frozen section, cytology,
  with synoptic reporting (CAP cancer protocols)
- `specimen` — collection, receipt, accessioning, aliquots, rejection,
  and chain-of-custody evidence
- `critical-result` — critical value or urgent pathology finding
  notification, acknowledgement, timeout, and escalation evidence
- `reference-range` — population-specific reference ranges (age, sex,
  race, gestational age, dialysis status, method, unit)
- `reflex-test` — deterministic reflex rule evaluation and follow-up lab
  order request
- `result-authorization` — lab director and pathologist authorization,
  delegation, and electronic-signature evidence
- `result-interpretation` — structured/narrative lab and pathology
  interpretation without owning image artifacts
- `result-delivery` — push-to-EMR, push-to-portal, fax, secure email,
  and downstream delivery evidence
- `turn-around-time` — TAT clocks, breach alerts, and dashboard
  projections
- `quality-control` — analyzer QC, calibrator lots, rule failures, and
  corrective actions

Top-3 industry counterparts:

**Sunquest / Clinisys LIS.** Dominant laboratory information system for
hospital labs, including order entry, result workflow, blood bank,
anatomic pathology, and HL7v2 ORM/ORU patterns.

**Oracle Health PathNet / Cerner Diagnostics Labs.** PathNet and
Millennium Labs cover laboratory and pathology workflows integrated with
the Oracle Health EMR estate.

**Epic Beaker + Beaker AP.** Epic's lab and anatomic pathology modules,
tightly integrated with Epic clinical workflows and pathology sign-out.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- CLIA (Clinical Laboratory Improvement Amendments, 42 CFR §493) —
  laboratory operations
- CAP accreditation (College of American Pathologists) — synoptic
  reporting, proficiency testing
- FDA in-vitro diagnostic (IVD) regulations (21 CFR §809) for IVD-
  reported results
- ISO 15189 laboratory quality competence
- 42 CFR Part 2 where drug-of-abuse panels involve SUD context

SLO envelope:
- Order placement to LIS arrival p99 < 30s
- Result final-publish to EMR p99 < 60s
- Critical-value notification p99 < 5min
- Pathology sign-out to delivery p99 < 60s
- Reference-range lookup p99 < 25ms

Cross-µservice handoffs:
- diagnostics <- emr: lab/pathology order (Workflow event
  `emr.order.created`)
- diagnostics -> emr: lab/pathology result delivery (Workflow event
  `diagnostics.result.finalised`)
- diagnostics -> imaging: image-correlation request for a lab/pathology
  result (Workflow event `diagnostics.lab-result.image-correlation-
  requested`)
- diagnostics <- imaging: imaging report/study reference for correlation
  only (Workflow event `imaging.report.correlation-available`)
- diagnostics -> emergency: STAT lab result for ED patient (Workflow
  event `diagnostics.result.stat`, ED-priority routing)
- diagnostics -> clinical-decision-support: critical-value rule trigger
  (Workflow event `diagnostics.result.critical`)
- diagnostics -> cloud-billing: charge capture for lab/pathology
- diagnostics <-> healthcare-integration: FHIR DiagnosticReport export +
  HL7v2 ORU import
- diagnostics <-> pharmacy: TDM (therapeutic drug monitoring) result-to-
  dose-adjustment loop

### C.3 `microservices/emergency/` — Emergency Department Workflow

Concern: the canonical ED workflow. Triage acuity assignment, trauma
activation, mass-casualty incident triage, EMS pre-arrival handoff,
ED registration, ED tracking board, ED disposition (admit, discharge,
transfer, AMA, LWBS, left-without-treatment), and ED throughput
optimization.

Bounded contexts:
- `triage` — Emergency Severity Index (ESI-1 critical, ESI-2 emergent,
  ESI-3 urgent, ESI-4 less urgent, ESI-5 non-urgent), Manchester Triage
  System (MTS), Canadian Triage and Acuity Scale (CTAS), Australasian
  Triage Scale (ATS), KR KTAS (Korean Triage and Acuity Scale)
- `trauma-activation` — trauma team activation criteria, trauma level
  (level 1 / level 2 / level 3), trauma bay assignment
- `mass-casualty` — START (Simple Triage and Rapid Treatment) tags,
  SALT (Sort, Assess, Lifesaving interventions, Treatment/Transport)
  triage, surge capacity
- `ems-handoff` — EMS pre-arrival notification (radio + electronic),
  NEMSIS v3.5 inbound message, EMS run report attachment
- `ed-registration` — quick-reg vs full-reg, MSE (medical screening
  exam) flag for EMTALA, insurance capture
- `tracking-board` — real-time bed status, patient location, assigned
  nurse, assigned MD, pending tasks, length-of-stay clock, boarding
  flag
- `disposition` — admit (with bed request), discharge (with home
  instructions), transfer (with EMTALA-compliant accepting facility),
  AMA (against medical advice), LWBS (left without being seen),
  elopement
- `boarding` — ED-to-inpatient bed-wait tracking, hallway-bed flag,
  CMS quality measure tracking (ED-1, ED-2)

Top-3 industry counterparts:

**T-System (acquired by Hyland Software 2024, now Hyland T-System
Healthcare).** Founded 1996. ED-specific EHR + ED tracking board.
T-Sheets (template-based ED documentation), T-Server (ED tracking),
T-Risk (high-risk patient screening). Used at ~40% of US ED visits.
Now under Hyland's healthcare BU.

**Wellsoft EDIS.** Wellsoft Corporation. ED Information System
(EDIS) for ED-specific workflow. Wellsoft EDIS Real Time Order Entry,
Wellsoft Tracking. Strong in mid-size community hospitals. Acquired by
Medsphere in 2021.

**Epic ASAP.** Epic's ED-specific module within the Epic suite. ASAP
includes tracking board, MSE workflow, EMTALA compliance, EMS pre-
arrival, trauma activation, and ESI triage. Tightly integrated with
EpicCare Inpatient (admission) and Willow (ED medication
administration). Used by Epic-base IDNs.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- EMTALA (Emergency Medical Treatment and Labor Act, 42 USC §1395dd)
  — medical screening exam, stabilisation, transfer requirements;
  Cedar fragment must enforce MSE before disposition
- NEMSIS v3.5 (National EMS Information System) — EMS data import +
  export
- State EMS protocols (varies by state)
- CMS quality measures: ED-1 (median time from ED arrival to ED
  departure for admitted patients), ED-2 (median time from admit
  decision to ED departure), OP-18 (median time from ED arrival to
  ED departure for discharged patients), OP-22 (left-without-being-
  seen percentage), STK-OP-1 (stroke door-to-IV alteplase ≤ 60min)
- Joint Commission ED-specific standards
- 42 CFR Part 2 (SUD-related ED visits)

SLO envelope:
- ESI triage assignment to patient-record write p99 < 5s
- Trauma activation alert to trauma team page p99 < 30s
- EMS pre-arrival receipt to tracking board p99 < 15s
- ED tracking board refresh latency p99 < 2s
- ED disposition write to bed-management broadcast p99 < 10s

Cross-µservice handoffs:
- emergency → emr: ED encounter start/end (Workflow event
  `emergency.ed-arrival` / `emergency.ed-departure`)
- emergency ← emr: patient history pull (synchronous gRPC, emr-side
  FHIR Patient $everything for ED context)
- emergency ↔ diagnostics: STAT order priority routing
- emergency ↔ pharmacy: ED medication admin with BCMA
- emergency → patient-monitoring: ED-bed vital-sign continuous
  monitoring activation
- emergency → cloud-billing: ED charge capture
- emergency ↔ healthcare-integration: HL7v2 ADT A04 (ED registration)
  import + ADT A03 (ED discharge) export
- emergency → clinical-decision-support: triage-based protocol trigger
  (sepsis bundle, stroke alert, STEMI alert)

### C.4 `microservices/pharmacy/` — Pharmacy Management + ePrescribe

Concern: the canonical pharmacy workflow. Medication order processing,
drug-drug interaction check, allergy check, dose-range check, formulary
check, ePrescribe to retail/specialty/mail-order pharmacies, in-house
dispensing, BCMA (barcode medication administration), pharmacy
intervention documentation, controlled-substance custody chain (DEA
EPCS), and 340B drug pricing program tracking.

Bounded contexts:
- `medication-order` — drug, route, dose, frequency, duration,
  indication, with RxNorm + NDC coding
- `interaction-check` — drug-drug, drug-allergy, drug-food, drug-lab,
  drug-condition, drug-gene (pharmacogenomic) checks; severity level
  (contraindicated, severe, moderate, minor)
- `formulary-check` — payer formulary status, prior auth requirement,
  step therapy, quantity limit
- `eprescribe` — Surescripts-routed prescription via NCPDP SCRIPT 2017+,
  with prescriber DEA validation for controlled substances
- `dispensing` — in-house pharmacy dispense, robot dispense (BD
  Pyxis / Omnicell), unit-dose dispense
- `bcma` — barcode medication administration with patient wristband
  scan + medication barcode scan + nurse credential check
- `intervention` — pharmacist clinical intervention (dose adjustment,
  therapy substitution, monitoring recommendation)
- `controlled-substance` — DEA EPCS 21 CFR §1311 e-prescribing with
  two-factor identity proofing, biometric or token, PDMP query
- `compounding` — sterile (USP 797) and hazardous (USP 800) compounded
  preparations with chain-of-custody
- `340b` — 340B Drug Pricing Program eligibility tracking + audit trail

Top-3 industry counterparts:

**Cerner Pharmacy Manager (Oracle Health Pharmacy after 2022).**
Cerner Millennium pharmacy module. Includes PharmNet (inpatient
pharmacy), RxStation (automated dispensing cabinet integration),
SurgiNet (perioperative medication). Integrates with BD Pyxis. Used by
Cerner-base IDNs.

**Epic Willow (Willow Inpatient + Willow Ambulatory).** Epic's
pharmacy module. Willow Inpatient covers inpatient order entry,
verification, dispensing, BCMA, IV workflow, sterile compounding;
Willow Ambulatory covers retail pharmacy workflow (Epic-affiliated
retail). Tight integration with Surescripts for ePrescribe.

**BD Pyxis (Becton Dickinson).** Automated dispensing cabinet (ADC)
substrate. Pyxis MedStation ES, Pyxis Logistics, Pyxis CIISafe (anti-
diversion). Biometric (fingerprint) cabinet access. PII / PHI exposure
on cabinet UI. Strong DEA controlled-substance custody chain. Industry
co-leader in ADC with Omnicell.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- DEA EPCS 21 CFR §1311 — Electronic Prescriptions for Controlled
  Substances; two-factor identity proofing, FIPS 140-2 cryptographic
  module, biometric or hard-token at signing
- NCPDP SCRIPT 2017+ standard for ePrescribe
- State PDMP (Prescription Drug Monitoring Program) integration —
  varies by state (NY I-STOP, CA CURES, TX PMP-AWARE, KY KASPER)
- 340B Drug Pricing Program (42 USC §256b) — covered-entity tracking
- USP 797 sterile compounding standards
- USP 800 hazardous compounding standards
- FDA Drug Supply Chain Security Act (DSCSA, 21 USC §360eee) —
  serialised drug tracking
- ISMP (Institute for Safe Medication Practices) safe-practice
  guidelines

SLO envelope:
- Order to pharmacist verify p99 < 5min
- Verify to dispense ready p99 < 30min (in-house)
- ePrescribe to Surescripts ack p99 < 10s
- BCMA scan to confirm p99 < 1s
- Interaction check evaluation p99 < 100ms (per drug pair)
- PDMP query p99 < 5s

Cross-µservice handoffs:
- pharmacy ← emr: medication order (Workflow event
  `emr.medication.prescribed`)
- pharmacy → emr: medication-administration-record (MAR) write
  (Workflow event `pharmacy.mar.administered`)
- pharmacy → cloud-billing: drug charge capture
- pharmacy ↔ diagnostics: TDM result-to-dose-adjustment loop
- pharmacy ↔ clinical-decision-support: drug-interaction-alert
  cooperative; pharmacy is the source of truth for active medications,
  CDS is the policy engine for interaction rules
- pharmacy ↔ healthcare-integration: NCPDP SCRIPT outbound to
  Surescripts, RxHistory inbound
- pharmacy → audit-chain: every controlled-substance event (DEA EPCS
  21 CFR §1311 requires retention of audit trail for 2 years past last
  use)

### C.5 `microservices/patient-monitoring/` — Vital Sign Telemetry + Alarms + RPM

Concern: the canonical patient-monitoring workflow. Continuous vital-
sign acquisition from bedside monitors, central-station aggregation,
alarm processing with alarm-fatigue reduction, remote patient
monitoring (RPM) from home devices, ICU/CCU integration, waveform
storage (ECG, EEG, EMG, capnography, arterial line), and clinical
decision-support input for early-warning scores (NEWS2, MEWS, qSOFA,
PEWS for pediatrics).

Bounded contexts:
- `vital-sign` — point-in-time (HR, BP, SpO2, RR, T) with timestamp
  and source device; distinct from emr's point-in-time vitals which
  may be manually entered
- `waveform` — continuous ECG (12-lead, 5-lead, 3-lead), invasive
  arterial pressure, central venous pressure, intracranial pressure,
  capnography, EEG, EMG; with sample rate, channel count, baseline,
  derived heart-rate / respiratory-rate
- `alarm` — alarm origin (device, central station, derived from
  early-warning score), severity (crisis, warning, advisory),
  alarm-fatigue rule (deferral, escalation, silencing window), nurse
  acknowledgement
- `alarm-fatigue` — alarm-management policy (per-unit thresholds,
  alarm deferral, alarm escalation chain), conformance with TJC NPSG
  06.01.01 (alarm management)
- `rpm` — remote patient monitoring from home devices (continuous
  glucose monitor, blood-pressure cuff, weight scale, pulse oximeter,
  smart inhaler, Apple Watch / Fitbit / Garmin / Oura / Whoop)
- `icu-bundle` — ICU-specific telemetry bundle (continuous lactate,
  continuous cardiac output, mixed-venous SaO2)
- `early-warning-score` — NEWS2 (UK National Early Warning Score 2),
  MEWS (Modified Early Warning Score), qSOFA (quick Sepsis-related
  Organ Failure Assessment), PEWS (Pediatric Early Warning Score),
  computed from vital-sign + waveform streams

Top-3 industry counterparts:

**Philips PIC iX (PIIC iX — Patient Information Center iX).** Philips
Healthcare. ICU/CCU central monitoring station. Aggregates Philips
IntelliVue patient monitors (MX-series bedside, MP-series transport),
Avalon fetal monitoring, Performance Manager. Industry leader in
acute-care central monitoring. Integrates with Philips IntelliSpace
Critical Care and Anesthesia (ICCA). Strong waveform fidelity (250Hz +
ECG).

**GE CARESCAPE.** GE HealthCare. CARESCAPE Central Station, CARESCAPE
B450/B650/B850 bedside monitors, CARESCAPE Telemetry. Central
monitoring + bedside integration. Strong telemetry-network architecture
for step-down units.

**Mindray BeneVision.** Mindray (Shenzhen). BeneVision N-series bedside,
BeneVision Central Monitoring System. Emerging global #3 by share,
rapidly gaining at mid-market hospitals. HL7v2 ORU-R30 vital-sign
export, IHE PCD-01 (Point-of-Care Device) profile-conformant.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- FDA 21 CFR §820 quality system (Medical Device Data System per
  21 CFR §880.6310)
- IEC 60601-1 (medical-electrical safety, general)
- IEC 60601-2-49 (multifunction patient monitor safety)
- IEEE 11073 medical-device communication standards (PHD personal
  health device, PoCD point-of-care device)
- IHE PCD (Patient Care Device) profile family
- TJC NPSG 06.01.01 (alarm management) — alarm-fatigue policy
  conformance
- FDA SaMD (Software as a Medical Device) classification depending on
  derived-score authority (NEWS2 alone is informational; if it auto-
  triggers RRT, it may rise to Class II)

SLO envelope:
- Alarm signal to nurse station notification p99 < 1s
- Alarm signal to nurse phone (in-app) p99 < 3s
- Waveform write durability: 11 9s (continuous waveform; loss is a
  safety event)
- Vital-sign export to emr p99 < 2s (per ORU-R30 cycle)
- Early-warning-score recomputation cycle ≤ 1 minute
- Central station UI refresh p99 < 500ms

Cross-µservice handoffs:
- patient-monitoring → emr: vital-sign and rolled-up score (Workflow
  event `patient-monitoring.vitals.point-in-time` and
  `patient-monitoring.score.computed`)
- patient-monitoring → clinical-decision-support: early-warning-score
  alert (Workflow event `patient-monitoring.ews.alert`)
- patient-monitoring → emergency: ED-bed continuous monitoring stream
- patient-monitoring → care-management: RPM data stream for at-home
  enrollee
- patient-monitoring ↔ healthcare-integration: HL7v2 ORU-R30 (real-
  time vital sign), FHIR Observation R5 (continuous) when ingressing
  external RPM device
- patient-monitoring → audit-chain: alarm acknowledgement chain
- patient-monitoring → cloud-billing: RPM CPT 99453/99454/99457/99458
  charge capture

### C.6 `microservices/clinical-decision-support/` — CDS + BPAs + Pathways

Concern: the canonical clinical-decision-support workflow. Clinical
pathways (sepsis bundle, stroke alert, STEMI alert, VTE prophylaxis,
catheter-associated UTI prevention, central-line bloodstream-infection
prevention), drug-interaction alerts (delegating to pharmacy as source
of truth for active meds), evidence-based recommendations from
external knowledge bases (UpToDate, Lexicomp, Micromedex), BPAs (Best
Practice Advisories) at point of order, dose checks, allergy alerts,
and order sets.

Bounded contexts:
- `clinical-pathway` — multi-step protocol (sepsis 1-hour bundle, NIH
  stroke protocol, AHA STEMI protocol, VTE prophylaxis assessment)
- `interaction-alert` — drug-drug, drug-allergy, drug-condition (rule
  evaluation; medication-list state lives in pharmacy)
- `evidence-recommendation` — recommendation from UpToDate / Lexicomp /
  Micromedex integration via CDS Hooks 2.0 + FHIR Clinical Reasoning
  IG
- `bpa` — Best Practice Advisory at point of order or point of charting
- `dose-check` — renal dose adjustment, hepatic dose adjustment,
  pediatric dose adjustment, gestational dose adjustment, geriatric
  dose adjustment
- `allergy-alert` — drug-allergy cross-reactivity rule
- `order-set` — bundled order template (admission order set, post-op
  order set, anti-coagulation order set)
- `cds-hook` — implements CDS Hooks 2.0 service for patient-view, order-
  select, order-sign, encounter-start, encounter-discharge,
  appointment-book

Top-3 industry counterparts:

**UpToDate (Wolters Kluwer Health).** Evidence-based clinical decision
support content. Founded 1992. Subscribed by ~80% of US academic
medical centers. UpToDate Anywhere mobile, UpToDate Lexidrug, UpToDate
Pathways. EHR integration via REST API + CDS Hooks (in beta as of
2025). Content authored by ~7000 physician editors.

**Wolters Kluwer Lexicomp (Lexidrug).** Drug-information content + drug
interaction engine + dose-checking engine. Lexicomp Online + Lexidrug
Suite. Sold as content service to EMRs (Epic, Cerner, athenahealth
embed Lexicomp content). IBM Watson Health acquired (briefly), then
spun back to Merative.

**IBM Micromedex (now Merative Micromedex after Merative acquired IBM
Watson Health 2022).** Drug-information content + interaction engine.
Micromedex DrugDex, Micromedex DrugReax (interactions), Micromedex
Neofax (pediatric). Sold as content service to EMRs. Strong in pediatric
+ specialty dose-checking.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- FDA SaMD classification — Software as a Medical Device for
  decision-support algorithms; FDA CDS guidance (September 2022
  finalised) — exempts CDS that displays evidence and lets clinician
  decide, regulates CDS that auto-triggers action
- EU MDR (Medical Device Regulation 2017/745) — clinical decision
  support is a medical device in EU; CE marking required
- EU AI Act 2024/1689 — clinical AI is high-risk Annex III; conformity
  assessment + human oversight required
- HL7 CDS Hooks 2.0 specification conformance
- HL7 Clinical Reasoning IG (FHIR-native)
- CDS rule provenance + content versioning (per ADR-0247
  self-modification doctrine, CDS rules are first-class versioned
  bundles)

SLO envelope:
- BPA evaluation at order-sign p99 < 200ms
- Interaction check p99 < 100ms per drug pair
- Pathway-step evaluation p99 < 500ms
- Evidence-recommendation pull from UpToDate / Lexicomp p99 < 2s
- Order-set materialisation p99 < 500ms

Cross-µservice handoffs:
- clinical-decision-support ← emr: patient context + active problem
  list + active meds (synchronous gRPC on every CDS evaluation)
- clinical-decision-support ← pharmacy: active medication state
  (medication list is pharmacy's truth; CDS reads it)
- clinical-decision-support ← diagnostics: latest lab values for dose
  checks (e.g., creatinine for renal dose adjustment)
- clinical-decision-support ← patient-monitoring: early-warning-score
  feed (e.g., NEWS2 ≥ 7 triggers RRT pathway)
- clinical-decision-support → emr: BPA payload (Workflow event
  `cds.recommendation` consumed by emr UI overlay)
- clinical-decision-support → audit-chain: every BPA fire + clinician
  override (override-reason captured per ADR-0263)

### C.7 `microservices/care-management/` — Care Plans + Coordination + Population Health

Concern: the canonical care-management workflow. Care plan authoring,
care transitions (hospital-to-home, hospital-to-SNF, hospital-to-
rehab), care team assignment, care coordination messaging, population
stratification (risk score, high-utiliser, gap-in-care), intervention
tracking, longitudinal outreach campaign, and quality-measure
reporting (HEDIS, MIPS, CMS Star Ratings).

Bounded contexts:
- `care-plan` — multi-disciplinary care plan with problems, goals,
  interventions, expected outcomes, target dates; FHIR CarePlan R5
  canonical
- `care-transition` — hospital discharge to home / SNF / rehab / hospice
  with medication reconciliation, follow-up appointment, transitional
  care management (TCM) CPT 99495/99496 billing
- `care-team` — care team members with role (PCP, care manager, social
  worker, dietitian, pharmacist, behavioural health, community health
  worker), with FHIR CareTeam R5
- `coordination-message` — secure messaging between care-team members
  (distinct from clinician-to-clinician chat; this is care-plan-
  bound)
- `population-stratification` — risk-score computation (HCC, ACG, CMS-
  HCC, LACE, CCM), gap-in-care detection (overdue colonoscopy,
  overdue A1c, overdue mammogram), HEDIS measure attribution
- `intervention` — outreach event (call, text, email, mailed letter,
  community health worker visit) with outcome tracking
- `outreach-campaign` — multi-touch campaign (e.g., diabetes A1c
  follow-up sequence) with consent + opt-out tracking
- `quality-measure` — HEDIS, MIPS, CMS Star Ratings, ACO measures
  (Medicare Shared Savings Program), with attribution + numerator/
  denominator computation

Top-3 industry counterparts:

**Salesforce Health Cloud.** Salesforce. Patient-relationship management
+ care coordination platform built on Salesforce Lightning. Acquired
ClickSoftware (field service) 2019. Native FHIR R4 surface. Used by
provider organisations + health plans for care management. Integrates
with EHRs via FHIR R4 + HL7v2.

**Epic Healthy Planet.** Epic's population-health module. Includes
care management, risk stratification (Epic-built), HEDIS measure
attribution, quality reporting. Tight integration with EpicCare
Ambulatory + MyChart. Used by Epic-base ACOs (Accountable Care
Organisations) for MSSP reporting.

**Innovaccer.** Founded 2014, San Francisco. Population-health platform
with FHIR-native longitudinal record, risk stratification, care
management workflow, value-based-care contract management. Used by
provider organisations + ACOs + Medicare Advantage plans. Hedge against
EHR-specific population-health tools (works across Epic + Cerner +
athenahealth + Allscripts).

Compliance shape:
- HIPAA-2024 pack (mandatory)
- CMS Star Ratings (Medicare Advantage Part C + Part D Star Ratings)
- MIPS/MACRA (Merit-based Incentive Payment System; Quality / Cost /
  Improvement Activities / Promoting Interoperability categories)
- HEDIS (Healthcare Effectiveness Data and Information Set,
  NCQA-defined)
- State Medicaid care-management rules (varies)
- MSSP (Medicare Shared Savings Program) ACO measures
- 21st Century Cures Act information-blocking
- 42 CFR Part 2 (SUD-related care plans)
- TCPA (Telephone Consumer Protection Act) — outreach calls/SMS to
  patient mobile

SLO envelope:
- Care plan write p99 < 500ms
- Population stratification batch (1M members) ≤ 4 hours nightly
- Gap-in-care detection per member p99 < 500ms
- Outreach campaign send (10k members) ≤ 30 minutes
- HEDIS measure attribution recomputation ≤ 24 hours

Cross-µservice handoffs:
- care-management ← emr: problem / condition flag (Workflow event
  `emr.problem.added`)
- care-management ← diagnostics: gap-in-care detection (e.g., A1c
  overdue; lab catalogue from diagnostics)
- care-management ← pharmacy: medication adherence signal
- care-management → emr: care plan attached to patient
- care-management → cloud-iam: care-team role assignment (caregiver
  binding to patient via Cedar policy)
- care-management → comms-email + messenger: outreach campaign
  delivery
- care-management ↔ healthcare-integration: FHIR CarePlan exchange
  with payer (Da Vinci PCDE — Patient Cost Disclosure Exchange and
  PDex — Payer Data Exchange) for value-based-care contracts
- care-management → audit-chain: every consent-bound outreach event

### C.8 `microservices/imaging/` — Imaging / PACS / VNA

Concern: canonical imaging workflow and image object custody. Imaging owns
imaging orders, acquisition coordination, PACS/VNA indexing, DICOM study/
series/instance custody, DICOMweb/DIMSE domain behavior, radiologist
worklists, read reports, structured reporting, prior comparison, hanging
protocols, dose tracking, image AI, and FHIR `ImagingStudy` projections.

Bounded contexts:
- `imaging-order` — modality, body site, laterality, indication,
  scheduled time, priority, acquisition routing, and accession number
- `imaging-study` — study, series, instance, accession, study lifecycle,
  and FHIR `ImagingStudy` projection
- `pacs-index` — queryable image metadata, worklist lookup, and study
  discovery
- `vna-object` — vendor-neutral archive custody, retention, replication,
  and object integrity
- `radiologist-worklist` — assignment, priority, subspecialty, TAT clock,
  and interruption handling
- `read-report` — draft, preliminary, final, addendum, amended, critical
  result, and EMR delivery
- `hanging-protocol` — reader/viewer layout and prior-comparison protocol
- `dose-monitoring` — modality dose event capture and regulatory evidence
- `image-ai` — model invocation, de-identification, result capture, and
  human review

Top-3 industry counterparts:

**GE Centricity.** GE HealthCare imaging portfolio covering Centricity
Universal Viewer / imaging workflow surfaces, enterprise viewer posture,
and radiology/cardiology imaging informatics.

**Philips IntelliSpace.** Enterprise imaging, PACS, advanced visualization,
and clinical workflow portfolio used in hospital radiology estates.

**Sectra PACS+VNA.** Enterprise imaging platform combining PACS, VNA,
viewer, workflow, and cross-specialty imaging object custody.

Compliance shape:
- HIPAA-2024 pack (mandatory)
- DICOM PS3 conformance and TLS profiles
- IHE Radiology profiles
- ACR accreditation evidence
- MQSA where mammography applies
- FDA SaMD where image AI participates in clinical workflow

SLO envelope:
- DICOM receive acknowledgement p99 < 200ms per instance
- DICOMweb study retrieval p99 < 300ms per study
- Worklist load p99 < 1s
- Read-report final-publish to EMR p99 < 60s
- Prior fetch p99 < 3s for local-cell prior studies

Cross-µservice handoffs:
- imaging <- emr: imaging order (Workflow event
  `emr.imaging-order.created`)
- imaging -> emr: final read report and FHIR `ImagingStudy` reference
  (Workflow event `imaging.report.finalised`)
- imaging <- diagnostics: image-correlation request for lab/pathology
  result context (Workflow event `diagnostics.lab-result.image-
  correlation-requested`)
- imaging -> diagnostics: imaging report/study reference for correlation
  (Workflow event `imaging.report.correlation-available`)
- imaging -> emergency: critical imaging finding for ED patient
  (Workflow event `imaging.result.critical`)
- imaging -> clinical-decision-support: imaging finding or AI result for
  evidence evaluation (Workflow event `imaging.finding.available`)
- imaging -> cloud-billing: imaging charge capture
- imaging <-> healthcare-integration: brokered DICOMweb/DIMSE/FHIR
  ingress/egress without moving image ownership into the broker
- imaging -> audit-chain: every PHI/image access, report mutation, and
  disclosure event

### C.9 `microservices/healthcare-integration/` — Integration Substrate (Scope Narrowed)

Concern (narrowed): the canonical clinical-integration substrate.
FHIR/HL7v2/DICOM brokering between oyatie and external EHR systems +
clinical-data networks. The integration µservice does not own clinical
state; it ingests, transforms, routes, and emits.

Bounded contexts (narrowed from 14 domains to integration concern):
- `fhir-broker` — FHIR R4 + R5 inbound + outbound, IG validation,
  Bundle transaction, Subscription
- `hl7v2-broker` — HL7v2.3–v2.7+ inbound + outbound, MLLP transport,
  ACK/NAK generation, segment validation
- `dicom-broker` — DIMSE (C-STORE / C-FIND / C-MOVE) + DICOMweb (QIDO-
  RS / STOW-RS / WADO-RS), DICOM modality worklist (MWL), DICOM MPPS
- `mpi-substrate` — Master patient index identifier cross-reference
  between external EHR identities and oyatie patient identifiers
  (deterministic + Fellegi-Sunter probabilistic). The MPI matches
  external identities to oyatie's emr-owned canonical patient; the
  canonical patient record lives in `emr`
- `consent-segmentation` — purpose-of-use enforcement at the broker
  boundary; PHI segmentation tag at egress
- `provenance-seal` — cryptographic seal on outbound clinical bundle
  for EHR audit purposes (Ed25519 signature per ADR-0030)
- `break-glass-broker` — emergency-access bypass at the broker
  boundary; the actual break-glass policy lives in the domain
  µservice (emr / emergency / pharmacy), the broker only relays the
  justification metadata
- `ehr-connector` — vendor-specific connector adapters (Epic App
  Orchard, Cerner Code App, athenahealth athenaOne API, etc.)

Top-3 counterparts (PRESERVED from existing audit):

**Redox.** San Francisco, 2014. Single-API integration-as-a-service
broker. Public docs: `https://docs.redoxengine.com/`. Redox Data Models
(vendor-neutral intermediate). Redox Cloud + Hub. 600+ EHR systems
connected.

**Mirth / NextGen Connect.** Open-source HL7v2 integration
engine. Channel-based message routing with JavaScript/Java
transformers. Multi-channel listeners (MLLP, TCP, HTTP, FTP/SFTP,
JMS). De-facto open-source baseline.

**Health Gorilla.** Sunnyvale, 2014. Clinical-data network + FHIR-native
longitudinal-record platform. Direct connections to LabCorp + Quest +
600+ regional labs + 90% US imaging.

Compliance shape (substrate-narrowed):
- HIPAA-2024 pack (mandatory)
- 21st Century Cures Act information-blocking (45 CFR §171)
- ONC §170.315(g)(10) Standardized API
- TEFCA (Trusted Exchange Framework and Common Agreement)
- IHE ATNA (Audit Trail and Node Authentication)
- DICOM PS3.15 §B.1.1 secure transport profile
- DICOM TLS BCP 195

SLO envelope (substrate-narrowed):
- HL7v2 ingress to ack p99 < 1s
- FHIR R5 read p99 < 200ms
- DICOM C-STORE p99 < 200ms per instance
- DICOMweb STOW-RS p99 < 300ms per study
- MPI match query p99 < 500ms

What was REMOVED from healthcare-integration scope (migrating per §H):
- Patient record system-of-record concern → `emr`
- Lab/pathology workflow concern → `diagnostics`
- Imaging/PACS/VNA/radiology workflow concern → `imaging`
- ED workflow concern → `emergency`
- Pharmacy workflow concern → `pharmacy`
- Continuous monitoring + alarms concern → `patient-monitoring`
- Clinical decision-support pathways concern → `clinical-decision-support`
- Care plans + population health concern → `care-management`

What REMAINS in healthcare-integration scope:
- FHIR/HL7v2/DICOM broker
- Vendor EHR connector adapters
- MPI identifier cross-reference (substrate-level; canonical patient
  identity belongs to emr)
- Consent segmentation at broker boundary
- Provenance seal on outbound bundles
- IHE profile substrate (XDS.b, XDR, XCA, MHD)
- Terminology service substrate (SNOMED / LOINC / RxNorm / ICD-10 /
  CPT — bring-your-own-terminology pack handling)

## D. Cross-µservice Handoff Matrix

The nine healthcare-domain microservices coordinate through Workflow
events + gRPC + Ontology Object Type references per ADR-0145. The
handoff matrix below names the canonical inter-µservice contracts. All
handoffs are tenant-scoped per ADR-0244 and Cedar-gated per ADR-0243.

| From | To | Trigger | Mechanism | Workflow event class | Cedar fragment |
|---|---|---|---|---|---|
| emr | pharmacy | medication ordered | gRPC + Workflow | `emr.medication.prescribed` | pharmacy.medication-order.accept |
| emr | diagnostics | lab/pathology order | gRPC + Workflow | `emr.order.created` | diagnostics.order.accept |
| emr | imaging | imaging order | gRPC + Workflow | `emr.imaging-order.created` | imaging.order.accept |
| emr | emergency | ED encounter start | Workflow event | `emr.encounter.ed-arrival` | emergency.encounter.accept |
| emr | care-management | problem added | Workflow event | `emr.problem.added` | care-management.problem.observe |
| emr | clinical-decision-support | (CDS evaluation request) | gRPC (sync) | n/a (sync call) | cds.evaluate |
| diagnostics | emr | lab/pathology result finalised | gRPC + Workflow | `diagnostics.result.finalised` | emr.result.accept |
| imaging | emr | imaging read report finalised | gRPC + Workflow | `imaging.report.finalised` | emr.imaging-report.accept |
| diagnostics | imaging | image-correlated lab/pathology result context | Workflow event | `diagnostics.lab-result.image-correlation-requested` | imaging.correlation.accept |
| imaging | diagnostics | imaging report/study correlation available | Workflow event | `imaging.report.correlation-available` | diagnostics.correlation.accept |
| diagnostics | emergency | STAT result for ED patient | Workflow event | `diagnostics.result.stat` | emergency.result.accept |
| diagnostics | clinical-decision-support | critical-value rule | Workflow event | `diagnostics.result.critical` | cds.evaluate-critical |
| diagnostics | pharmacy | TDM result for dose adjust | Workflow event | `diagnostics.tdm.delivered` | pharmacy.tdm.evaluate |
| diagnostics | cloud-billing | lab/pathology charge | Workflow event | `diagnostics.charge.captured` | cloud-billing.charge.accept |
| imaging | cloud-billing | imaging charge | Workflow event | `imaging.charge.captured` | cloud-billing.charge.accept |
| imaging | emergency | critical imaging finding for ED patient | Workflow event | `imaging.result.critical` | emergency.imaging-result.accept |
| imaging | clinical-decision-support | imaging finding / AI result | Workflow event | `imaging.finding.available` | cds.imaging-finding.evaluate |
| emergency | emr | ED departure / disposition | Workflow event | `emergency.ed-departure` | emr.encounter.close |
| emergency | pharmacy | ED medication admin | Workflow event | `emergency.medication.administered` | pharmacy.administration.accept |
| emergency | patient-monitoring | ED-bed monitoring activate | gRPC | n/a (sync) | patient-monitoring.bed.activate |
| emergency | clinical-decision-support | triage protocol trigger | gRPC (sync) | `emergency.triage.protocol-trigger` | cds.protocol-evaluate |
| emergency | cloud-billing | ED visit charge | Workflow event | `emergency.charge.captured` | cloud-billing.charge.accept |
| pharmacy | emr | MAR (medication administration record) | Workflow event | `pharmacy.mar.administered` | emr.mar.accept |
| pharmacy | cloud-billing | drug charge | Workflow event | `pharmacy.charge.captured` | cloud-billing.charge.accept |
| pharmacy | clinical-decision-support | active medication state push | Workflow event | `pharmacy.med-list.changed` | cds.med-list.observe |
| pharmacy | healthcare-integration | NCPDP SCRIPT outbound | gRPC | n/a | healthcare-integration.eprescribe.send |
| patient-monitoring | emr | vital-sign + early-warning-score | Workflow event | `patient-monitoring.vitals.point-in-time` + `patient-monitoring.score.computed` | emr.vitals.accept |
| patient-monitoring | clinical-decision-support | early-warning-score alert | Workflow event | `patient-monitoring.ews.alert` | cds.ews-evaluate |
| patient-monitoring | emergency | ED-bed continuous stream | gRPC (stream) | n/a | emergency.bed-stream.observe |
| patient-monitoring | care-management | RPM data stream | Workflow event | `patient-monitoring.rpm.delivered` | care-management.rpm.observe |
| patient-monitoring | cloud-billing | RPM CPT charge | Workflow event | `patient-monitoring.charge.captured` | cloud-billing.charge.accept |
| clinical-decision-support | emr | BPA recommendation | Workflow event | `cds.recommendation` | emr.bpa.accept |
| care-management | emr | care plan attached | Workflow event | `care-management.care-plan.attached` | emr.care-plan.accept |
| care-management | cloud-iam | care-team role assignment | gRPC | n/a | cloud-iam.caregiver-role.bind |
| care-management | comms-email | outreach delivery | Workflow event | `care-management.outreach.email` | comms-email.send.accept |
| care-management | messenger | outreach delivery | Workflow event | `care-management.outreach.message` | messenger.send.accept |
| All nine | audit-chain | every state transition | Workflow event | (per ADR-0263 audit-event class) | audit-chain.event.accept |
| All nine | consent-graph | consent verification | gRPC (sync) | n/a | consent-graph.verify |
| All nine | healthcare-integration | external EHR / HL7v2 / DICOM I/O | gRPC | n/a | (per channel) |

Cross-µservice anti-pattern (forbidden): direct database access between
healthcare-domain µservices. Patient record state lives in emr;
diagnostics MUST NOT read emr's patient table directly. Cross-µservice
data flows only through Workflow events + gRPC + Ontology Object Type
reads.

## E. HIPAA Pack Inheritance

Per ADR-0251 §D-3, the HIPAA-2024 pack is mandatory for any paid tenant
on any healthcare-domain microservice. Every microservice in the
nine-µservice cluster activates the HIPAA pack uniformly:

Cell certification level requirement (per ADR-0251 §D-4):
- Demo_trial tenant on healthcare-domain µservice: synthetic data only;
  HIPAA pack optional but PHI is forbidden in demo cells
- Paid tenant on healthcare-domain µservice: cell MUST be
  `hipaa-certified` (a cell certification level enumerated in
  `/specs/cell-certification-level-matrix.json`); HIPAA-2024 pack
  installed at tenant pack-install time

Per-µservice HIPAA pack inheritance:

| µservice | HIPAA-mandated controls (above baseline) |
|---|---|
| emr | PHI access audit (§164.312(b)); minimum-necessary (§164.502(b)); patient access right (§164.524) |
| diagnostics | PHI delivery audit; CLIA + CAP overlay; specimen chain-of-custody |
| imaging | Image access audit; DICOM transport controls; PACS/VNA object custody; ACR/MQSA evidence where applicable |
| emergency | EMTALA conformance audit (§42 USC 1395dd); MSE-before-disposition Cedar gate |
| pharmacy | DEA EPCS 21 CFR §1311 two-factor identity + biometric/token at signing; PDMP query audit; controlled-substance custody chain (2-year audit retention) |
| patient-monitoring | FDA 21 CFR §820 MDDS; IEC 60601-1 safety; alarm-management TJC NPSG 06.01.01 |
| clinical-decision-support | FDA SaMD classification; CDS provenance + content version; clinician-override audit |
| care-management | TCPA (telephone consumer protection) outreach consent; MSSP / Star / MIPS / HEDIS attribution audit |
| healthcare-integration | broker audit (IHE ATNA); cross-tenant routing Cedar gate; DICOM TLS BCP 195 |

Each µservice's `compliance.md` must include a §"HIPAA pack mapping"
that traces HIPAA Security Rule controls (§164.308 admin / §164.310
physical / §164.312 technical safeguards) to the µservice's Cedar
fragments, OpenSLO definitions, audit-event classes, runbooks, and
threat-model entries.

## F. Tenant-class Behaviour

Per ADR-0244 + ADR-0251 §D-3, tenant_class gating applies uniformly:

| Tenant class | Cell certification level | Synthetic data | PHI allowed | HIPAA pack | BAA required |
|---|---|---|---|---|---|
| demo_trial | general | yes | NO | optional | no |
| paid (US) | hipaa-certified | yes | YES | mandatory | YES — oyatie acts as Business Associate |
| paid (EU healthcare) | eu-sovereign + hipaa-certified-overlay-on-EU | yes | YES | mandatory | DPA (GDPR Article 28) instead of BAA |
| paid (KR healthcare) | kr-sovereign + 의료법-overlay | yes | YES | mandatory + KR 의료법 pack | KR 위탁계약 (entrustment agreement) |
| paid (federal — VA / DoD) | fedramp-high or il5 | yes | YES | mandatory + FedRAMP overlay | BAA + interconnection security agreement |

Cedar fragment requirements (per µservice):

```cedar
// Forbid PHI in demo_trial tenant — uniform across all nine µservices
forbid (
  principal,
  action,
  resource
) when {
  resource.tenant.class == "demo_trial" &&
  resource.data_class == "phi" &&
  resource.consent != "synthetic"
};
```

```cedar
// Forbid paid healthcare action without HIPAA-certified cell
forbid (
  principal,
  action,
  resource
) when {
  resource.tenant.class == "paid" &&
  resource.tenant.audience_type == "tenant-b2b-healthcare" &&
  !resource.cell.certifications.contains("hipaa-certified")
};
```

Pack-install workflow per ADR-0251 §D-2 records tenant_class + cell
certification level at install time; the audit-event class
`CompliancePackInstalled` captures both.

## G. Implementation Timeline — Wave 15M

This ADR's authoring is Wave 15M (post-Wave-15 keystone-bundle land,
post-Wave-15 Big 8 substance-bar reach). The decomposition rolls out
in sub-waves:

- **Wave 15M-A (this ADR + plan + remediation notes)** — 2026-05-21.
  Three files: this ADR, the Wave-15M plan, the healthcare-integration
  remediation notes. Zero authoring of the eight new µservices.

- **Wave 15M-B (eight µservice scaffolds + healthcare-integration
  scope narrow)** — 2026-05-22..2026-05-24. Eight new
  `microservices/<emr|diagnostics|imaging|emergency|pharmacy|patient-monitoring|
  clinical-decision-support|care-management>/` folders created with
  minimum-viable anchor set: manifest.json, PRD.md, ARCHITECTURE.md,
  compliance.md skeleton, contracts/ skeleton, slos/ skeleton,
  policies/ skeleton. healthcare-integration manifest + PRD scope-
  narrowed per remediation notes. Total: 8 × 6-anchor + 1 narrow =
  ~56 file ops.

- **Wave 15M-C (per-µservice IP roster)** — 2026-05-25..2026-05-31.
  Each new µservice gets 30 IPs (per ADR-0131 IP roster shape):
  tenant-scope-kernel, cedar-default-deny, ontology-projection,
  workflow-template-library, rest-contract-surface, async-event-
  surface, grpc-internal-surface, policy-eval-library-binding,
  credential-sidecar-binding, multi-region-cell-layout, observability-
  audit-events, abuse-defence-edge-waf, emergency-services-bypass (or
  domain-specific equivalent), marketplace-dealset-settlement, data-
  residency-pack-overlays, backfill-replay-worker, cost-budget-
  enforcer, capacity-admission-control, sdk-client-generation,
  catalog-layer-registration, slo-gated-promotion, chaos-drill-pack,
  dpia-evidence-packet, threat-model-control-map, audit-findings-
  closeout, and ~5 domain-specific IPs each. Total: 8 × 30 = 240 IPs.

- **Wave 15M-D (capability-tier matrices)** — 2026-06-01..2026-06-03.
  Each new µservice gets a `capability-tiers/tier-matrix.md` (Bronze /
  Silver / Gold / Platinum) projection.

- **Wave 15M-E (cross-µservice handoff IP slices)** — 2026-06-04..
  2026-06-07. Per-handoff IP slice authoring (matrix in §D above).

- **Wave 15M-F (HIPAA pack scaffold under microservices/governance/
  packs/HIPAA-2024/v1/)** — 2026-06-08..2026-06-14. BAA template,
  breach-notification workflow, training acknowledgement workflow,
  Cedar default-deny PHI-in-demo_trial fragment, FIPS mode IaC.

- **Wave 15M-G (per-domain compliance pack scaffolds)** — 2026-06-15..
  2026-06-21. Per-domain regulatory packs: ONC §170.315(g)(10) for
  emr, CLIA + CAP for diagnostics, DICOM/IHE/ACR/MQSA for imaging,
  EMTALA for emergency, DEA EPCS for pharmacy, FDA SaMD for clinical-
  decision-support, CMS Star + HEDIS for care-management.

Parallelism strategy: per ADR-0328 §D-14 batch ceiling of 8 codex
agents, Wave 15M-B dispatches the eight new µservice scaffolds in the
first batch and sequences the healthcare-integration scope narrow after
that batch clears.
Wave 15M-C must serialise by µservice (one agent owns one µservice's
30-IP roster end-to-end per `feedback_microservice_ownership_
coherence_2026_05_20`).

## H. Migration Path — Existing healthcare-integration → New µservices

Substantive content from the existing healthcare-integration µservice
migrates to the new µservices per the mapping below. Migration is
artifact-by-artifact and IP-tracked per the Wave 15M-B plan; no
content is dropped, no content is duplicated.

| Existing artifact | Migrates to | Reason |
|---|---|---|
| Bounded-context `patient-record` (PRD + ARCHITECTURE) | emr | system-of-record concern |
| Bounded-context `fhir-resource` (PRD + ARCHITECTURE) | RETAINED in healthcare-integration as `fhir-broker` | broker is integration concern |
| Bounded-context `hl7-message` | RETAINED in healthcare-integration as `hl7v2-broker` | broker is integration concern |
| Bounded-context `referral` | care-management (referral is care-coordination workflow) | care-coordination concern |
| Bounded-context `clinical-consent` | split: consent state model → consent-graph (already exists); broker-level consent segmentation → RETAINED in healthcare-integration | consent state belongs to consent-graph µservice; broker only enforces segmentation at boundary |
| Capability `fhir-read` | RETAINED in healthcare-integration | broker concern |
| Capability `hl7-route` | RETAINED in healthcare-integration | broker concern |
| Capability `break-glass-authorize` | emr (canonical break-glass on patient record); broker-level relay RETAINED in healthcare-integration | the policy belongs to the domain; the broker only relays |
| Capability `consent-sync` | consent-graph (canonical); broker boundary segmentation RETAINED | consent state lives in consent-graph |
| Capability `ehr-provenance-seal` | RETAINED in healthcare-integration | seal is at integration egress |
| Capability `patient-match-review` | emr (canonical patient is emr-owned); MPI substrate RETAINED in healthcare-integration | MPI substrate is integration concern; canonical patient is emr concern |
| IP-026 hl7-ack-route-custody | RETAINED in healthcare-integration | broker concern |
| IP-027 fhir-consent-segmentation | RETAINED in healthcare-integration | broker boundary concern |
| IP-028 break-glass-justification-review | RETAINED in healthcare-integration as broker-relay; emr authors a parallel IP for the canonical break-glass on the patient record | broker relays; emr policies |
| IP-029 mpi-patient-match-adjudication | RETAINED in healthcare-integration (MPI substrate); emr authors a parallel IP for canonical patient-match-review | MPI substrate is integration concern |
| IP-030 clinical-provenance-seal-export | RETAINED in healthcare-integration | seal at integration egress |
| Capability tier matrix (HL7v2 versions, FHIR R5, DICOM SOP classes, IHE profiles, terminology services, retention) | SPLIT: integration-scoped tiers → RETAINED; domain-scoped tiers (vendor EHR coverage, lab connectivity, MPI match tiers) → applicable new µservices | integration vs domain split |
| FHIR Implementation Guides (US Core 6.1.0 + 7.0.0, IPS-UV, Da Vinci IGs, CARIN BB, Provider Directory) | SPLIT: broker-side conformance → RETAINED; domain-side authoring of conformant resources → applicable new µservices (emr authors US Core Patient; diagnostics authors US Core Observation and lab/pathology DiagnosticReport; imaging authors ImagingStudy/read-report references; pharmacy authors US Core MedicationRequest; care-management authors US Core CarePlan + CareTeam) | broker vs authoring |
| HL7v2 message types (ADT / ORM / ORU / MDM / SIU / BAR / DFT) | SPLIT: routing in broker → RETAINED; semantic content in domain (ADT → emr; ORM/ORU → diagnostics; MDM → emr; SIU → emr; BAR/DFT → cloud-billing) | routing vs semantics |
| DICOM SOP classes (CR / CT / MR / NM / PT / US / XA / RF / SC / MG + secondary) | SPLIT: protocol brokering retained in healthcare-integration; imaging consumes, stores, displays, and manages image studies; diagnostics receives references only for lab/pathology correlation | broker vs imaging domain ownership |
| Terminology services (SNOMED / LOINC / RxNorm / ICD-10 / CPT) | RETAINED in healthcare-integration as substrate | substrate concern; domain µservices consume |
| Performance-leadership claims (FHIR throughput, HL7 throughput, DICOM throughput) | RETAINED in healthcare-integration's `benchmarks/` for broker-side; new µservices author their own benchmarks for domain-specific throughput (e.g., patient-monitoring waveform throughput; pharmacy ePrescribe rate) | broker vs domain |
| Compliance.md HIPAA Security Rule mapping | SPLIT: integration-substrate-scoped → RETAINED; domain-scoped → applicable new µservices | universal split |
| compliance.md KR 의료법 mapping | SPLIT per regulation surface: data movement → RETAINED; PHI access → applicable new µservices |  |

Migration completion gate per healthcare-integration scope narrow:

1. Manifest scope-narrowed: `bounded_contexts` reduced from 5 to 3
   broker-scoped contexts (fhir-broker, hl7v2-broker, dicom-broker
   substitute the previous 5 list); `coverage_benchmarks` updated to
   `[Redox, Mirth Connect, Health Gorilla]`.
2. PRD scope-narrowed: bounded-context section + FR list + US list
   reduce to broker concern.
3. Bounded-context-named files move to new µservice or to broker
   suffix.
4. The eight new microservice manifest.json files all declare
   `depends_on_microservices: [...healthcare-integration, audit-chain,
   consent-graph, tenancy, identity, cloud-iam, workflow-engine,
   ontology, governance, observability]` per ADR-0145.
5. Workflow event names align with the §D handoff matrix.
6. Per-domain compliance shape declared in each new µservice's
   compliance.md.

Migration does not delete anything in the first sub-wave. The existing
healthcare-integration artifacts that migrate OUT remain on disk as
RETIRED markers + a redirect stub pointing to the new µservice
location, per ADR-0138 six-path deprecation pattern. Deletion of
RETIRED-marker stubs happens only in a successor cleanup wave (Wave
15M-H) after all references update.

## I. Verification

The decomposition is verified through:

1. `cloud-ci/Rust gate packet per-microservice-layout` exits 0 against each new
   µservice folder.
2. `cloud-ci/Rust gate packet no-grouping` exits 0 — the new µservices
   are not bundle/grouping shapes.
3. `cloud-ci/Rust gate packet microservice-coherence-audit` exits 0 against
   each new µservice (the same five-dimension protocol used in the
   existing healthcare-integration audit).
4. The new BLOCKER lane `cloud-ci/Rust gate packet healthcare-domain-
   decomposition` (this ADR adds) enforces:
   - Each healthcare-domain µservice declares HIPAA-2024 pack in
     `manifest.compliance_packs[]`.
   - Each healthcare-domain µservice declares
     `audience_type: tenant-b2b-healthcare`.
   - Each healthcare-domain µservice declares a cell-certification-
     level expectation of `hipaa-certified` for paid tenant class.
   - Cross-µservice handoff Workflow event names match the §D matrix.
5. `cloud-ci/Rust gate packet hipaa-pack-coverage-per-healthcare-microservice`
   exits 0 — every healthcare-domain µservice has BAA template,
   breach-notification workflow ref, PHI data class registration, and
   audit-event class binding.
6. `cloud-ci/Rust gate packet cross-microservice-handoff-coherence` exits 0 —
   handoff event names match the §D matrix; no orphan Workflow event
   names; no missing consumer.
7. `cargo build --workspace` exits 0 once µservice scaffolds land.
8. `cargo nextest run --workspace` exits 0 against the new µservices'
   placeholder test trees.

## J. Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Behavioural health (mental health + SUD-specific) — does it become an eighth domain µservice or live inside emr with 42 CFR Part 2 segmentation? | council-clinical | resolved-by-default: live inside emr with 42 CFR Part 2 Cedar segmentation; if substance bar shows segmentation is insufficient, promote to `behavioural-health` µservice in Wave 16. |
| 2 | Surgical / OR scheduling — does it become a domain µservice or live inside emr? | council-clinical | resolved-by-default: live inside emr (Phase 1); promote to `surgery` µservice if perioperative workflow proves distinct (Wave 16+). |
| 3 | Anesthesia information management system (AIMS) — separate µservice? | council-clinical | resolved-by-default: lives inside patient-monitoring for intra-op vital-sign continuous capture + a `surgery` µservice's anesthesia record; Wave 16+ decision. |
| 4 | Genomics — separate µservice or part of diagnostics? | council-clinical | resolved-by-default: part of diagnostics (Wave 16+ decision); volume + IHE-DRG-DiPATH coverage may justify separation later. |
| 5 | Telehealth video — separate µservice or part of `meet`? | council-product | resolved-by-default: part of `meet` (existing) with healthcare-overlay (Wave 16+ decision). |
| 6 | Public health reporting (state IIS, vital records) — where does it live? | council-clinical | resolved-by-default: VXU export from emr (immunization registry); separate `public-health` µservice considered for Wave 17 if reporting load justifies. |
| 7 | Veterinary medicine — does it overlap with these µservices? | council-product | out-of-scope intentional — veterinary is a separate non-HIPAA market; future `veterinary-medicine` µservice considered post-Phase-4. |

## K. References

- ADR-0105: 13-layer canonical enum (each new µservice's crate suffixes
  follow this).
- ADR-0131: Per-microservice flat layout (each new µservice is a flat
  folder under `microservices/`).
- ADR-0132: No-grouping forward-policy (forbids new bundle/grouping
  µservices; this decomposition COMPLIES by authoring single-concern
  µservices, not a healthcare bundle).
- ADR-0145: Inter-microservice communication reform (direct gRPC + 3
  invariants for cross-µservice handoff).
- ADR-0188: Passkey/webauthn as canonical auth (DEA EPCS 21 CFR §1311
  two-factor requirement maps to this).
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0245: Substrate vs product layering.
- ADR-0247: Self-hosting / self-modification doctrine (CDS rules are
  versioned bundles).
- ADR-0248: Amazon-shape cellular architecture.
- ADR-0250: Build-ahead-of-certification (each µservice ships certified
  shape day one).
- ADR-0251: Compliance pack + cell certification levels (HIPAA pack
  mandatory for paid healthcare tenants).
- ADR-0263: Observability emission contract (every state transition
  emits to audit-chain).
- ADR-0316: Capability tier doctrine (Bronze / Silver / Gold /
  Platinum projected as UX, not directory split).
- ADR-0321: B2B SaaS industry-leader universe (these eight µservices
  cover the healthcare cluster of Phase 4).
- ADR-0322: Substance bar as doctrine and CI enforcement.
- ADR-0327: Wave 3 completion criteria and promotion gates.
- ADR-0328: Substance bar as canonical sequence and batch discipline.
- existing audit:
  `microservices/healthcare-integration/coherence-audit-2026-05-20.md`.
- existing parity matrix:
  `microservices/healthcare-integration/feature-parity-matrix-2026-05-
  20.md`.
- Industry sources cited inline per §C top-3 entries:
  - Epic Systems Corporation — `https://www.epic.com/` and
    `https://fhir.epic.com/` (App Orchard FHIR developer portal).
  - Oracle Health — `https://www.oracle.com/health/` and
    `https://fhir.cerner.com/`.
  - athenahealth — `https://www.athenahealth.com/` and
    `https://docs.athenahealth.com/api/`.
  - Sunquest (Clinisys) — `https://www.clinisys.com/`.
  - Cerner Diagnostics / Oracle Health Labs — `https://www.oracle.com/
    health/laboratory-information-system/`.
  - GE HealthCare Centricity — `https://www.gehealthcare.com/`.
  - T-System (Hyland) — `https://www.hyland.com/en/solutions/industries/
    healthcare/t-system`.
  - Wellsoft — `https://www.wellsoft.com/`.
  - Epic ASAP — Epic suite documentation.
  - Cerner Pharmacy Manager — Oracle Health pharmacy.
  - Epic Willow — Epic suite documentation.
  - BD Pyxis — `https://www.bd.com/en-us/products-and-solutions/
    products/product-families/pyxis-medication-management-system`.
  - Philips PIC iX — `https://www.usa.philips.com/healthcare/product/
    HC865350/intellivue-information-center-ix-piic-ix-patient-
    monitoring-network-software`.
  - GE CARESCAPE — `https://www.gehealthcare.com/products/patient-
    monitoring/carescape-monitors`.
  - Mindray BeneVision — `https://www.mindray.com/en/product-and-
    solutions/product/medical-imaging/benevision-clinical-information-
    system.html`.
  - UpToDate (Wolters Kluwer) — `https://www.wolterskluwer.com/en/
    solutions/uptodate`.
  - Wolters Kluwer Lexicomp — `https://www.wolterskluwer.com/en/
    solutions/lexicomp`.
  - IBM Micromedex / Merative — `https://www.merative.com/clinical-
    decision-support`.
  - Salesforce Health Cloud — `https://www.salesforce.com/health/`.
  - Epic Healthy Planet — Epic suite documentation.
  - Innovaccer — `https://innovaccer.com/`.
  - Redox — `https://www.redoxengine.com/` and
    `https://docs.redoxengine.com/`.
  - Mirth (NextGen Connect) — `https://www.nextgen.com/products-
    and-services/integration-engine`.
  - Health Gorilla — `https://www.healthgorilla.com/`.

<!--
COMPLETION REPORT — WAVE 15M-A
authored_by: claude-opus-4.7
authored_date: 2026-05-21
files_produced:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0332-healthcare-domain-decomposition.md
  - /Users/jasonlee/oyatie/.omc/plans/healthcare-decomposition-plan-2026-05-21.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/REMEDIATION-NOTES-2026-05-21.md
findings:
  - existing healthcare-integration µservice = 215 features × 14 domains in one µservice
  - violation of ADR-0131 single-concern + ADR-0132 no-grouping forward policy
  - decomposition into 8 new domain µservices + scope-narrow of existing healthcare-integration to integration substrate only
  - per-domain top-3 industry counterparts identified via research
  - HIPAA pack mandatory uniformly across all 9 healthcare-domain µservices
  - Phase 4 sub-wave with 8 µservices to author in Wave 15M-B
  - cross-µservice handoff matrix authored covering all inter-µservice events
zero_commits: true
zero_writes_outside_3_paths: true
zero_scripting: true
halt_condition: clean
-->
