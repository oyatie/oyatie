---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-pharmacy
microservice: pharmacy
title: Pharmacy Microservice Product Requirements
status: wave-15m-e-authored-2026-05-21
date: 2026-05-21
owner_team: axis-pharmacy
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0244
  - ADR-0251
  - ADR-0328
  - ADR-0332
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
---

# Pharmacy Microservice — Product Requirements Document (PRD)

- **Microservice slug**: `pharmacy`
- **Wave**: 15M-E (Healthcare Pharmacy Author Wave)
- **Authority ADR**: ADR-0332 (per-microservice pharmacy substrate authorization)
- **Layout authority**: ADR-0131 (per-microservice flat layout)
- **Suite policy**: ADR-0132 (no-suite microservices; single-concern flat)
- **Inter-microservice substrate**: ADR-0145 direct gRPC + 3 invariants (no Workflow+Ontology forced adapter)
- **Architecture overlay**: ADR-0328 (multispectrum review v2.4.0 + Big 8 + foundry absorption)
- **Compliance overlays**: ADR-0251 (compliance-pack primitive) — applicable packs: `hipaa`, `dea-controlled-substance`, `gdpr`, `pci-dss`, `eu-ai-act`, `lgpd`, `cn-pipl-2021`, `kr-pipa`, `state-board-of-pharmacy`, `dscsa`, `usp-797`, `usp-800`, `340b`, `ncpdp-script`, `surescripts`
- **Owner team**: `axis-pharmacy`
- **Criticality tier**: T0 (patient-safety + DEA-regulated controlled-substance plane)
- **Top-3 industry counterparts**: Oracle Health (Cerner) Pharmacy Manager / Epic Willow / BD Pyxis
- **Secondary counterparts**: McKesson EnterpriseRx, Omnicell, Talyst, Parata Mini, Surescripts, NCPDP SCRIPT 2017-071+, GS1 DSCSA, Symphony Health Solutions, Walgreens Boots Alliance Pharmacy Cloud
- **Sole owner agent**: pharmacy authoring agent (this document)
- **Last reviewed**: 2026-05-21

---

## §1 Mission, Boundaries, Anti-Goals

### §1.1 Mission

Author the canonical pharmacy clinical and operational substrate for the Oyatie platform: from the moment a prescription is written by a prescriber to the moment a medication is administered to a patient at the bedside (or dispensed at the retail counter), to the moment its lot serial number is settled against DSCSA traceability and the dispense is adjudicated against payer or 340B. Every step is HIPAA-covered, DEA-compliant where the substance is controlled, and verifiable against state board of pharmacy auditor requirements.

The pharmacy microservice owns:

1. **Medication catalog** — NDC, RxNorm, GPI, GCN, ATC, SNOMED CT, ICD-10, plus brand/generic substitution rules, route, form, strength, package, RxCUI semantic axes, and drug knowledge base ingestion (FDB MedKnowledge / Multum / First Databank / Wolters Kluwer Medi-Span; vendor-neutral adapter contract).
2. **Formulary management** — preferred, non-preferred, restricted, prior-authorization-required, criteria-of-use; per-tenant and per-cell formulary overlay; therapeutic interchange; auto-substitution policy; formulary review committee workflow.
3. **ePrescribe** — Surescripts and NCPDP SCRIPT 2017-071 (and later) round-trip orchestrator: NewRx, RxRenewal, RxChange, CancelRx, RxFill, MedHistoryRequest, MedHistoryResponse, OrderStatus, REMS-NewRx, EPCS for Schedule II–V controlled substances.
4. **Drug interactions** — DDI (drug-drug), DAI (drug-allergy/intolerance), DCI (drug-condition), DPI (drug-pregnancy/lactation), DDxI (drug-diagnosis), DLI (drug-lab), DFI (drug-food), DDoseI (drug-dose-range), with severity stratification (contraindicated/severe/major/moderate/minor/informational) and clinical decision support evidence binding.
5. **Allergy and intolerance cross-check** — patient allergy list ingestion from `emr` microservice, structured allergen normalization (RxNorm ingredient + UNII + SNOMED CT substance), cross-allergy class detection (penicillin → cephalosporin class), severity-aware override path with clinical justification capture.
6. **Dose range checking (DRC)** — weight-based, body-surface-area, renal (eGFR/CrCl adjusted), hepatic (Child-Pugh adjusted), age-based (neonatal/pediatric/geriatric), single-dose-max, daily-dose-max, lifetime-cumulative-max (e.g., anthracyclines), with pharmacist-override path.
7. **Duplicate therapy detection** — same active ingredient (RxCUI), same therapeutic class (ATC L4), same generic-brand pair, overlapping prescription windows, and same-day re-issue patterns.
8. **Pharmacist verification + order-entry verification** — single-pharmacist vs. dual-pharmacist verification per controlled-substance schedule; tall-man-lettering rendered output; barcode-verified product selection; verification audit ledger.
9. **Compounding workflow + sterile compounding** — non-sterile compounding (USP 795), sterile compounding (USP 797), hazardous drug compounding (USP 800), beyond-use-date (BUD) calculation, master formulation record, compounding record, environmental monitoring evidence binding.
10. **Inventory management + medication par levels** — per-location par/min/max, real-time on-hand, dispensing decrement, receiving and put-away, perpetual-inventory reconciliation, lot tracking, expiration date stratification, recall sequestration.
11. **Auto-dispensing cabinet integration** — Pyxis MedStation 4000/ES, Omnicell XT, Carousel (Talyst), AcuDose, MedDispense; vendor-neutral cabinet adapter contract; override events; cabinet-discrepancy reconciliation.
12. **Barcode medication administration (BCMA)** — five-rights verification (right patient, right drug, right dose, right route, right time), barcode scan capture from nurse handheld, override-with-justification, MAR (medication administration record) write-back to `emr`.
13. **IV admixture + IV smart pump integration** — IV order to admixture compound, smart pump drug library push (Alaris, Plum 360, Hospira/ICU Medical), DERS (dose error reduction system) hard/soft limit programming, pump auto-program via QR/barcode handshake.
14. **Controlled substance management** — DEA Form 222 ordering, perpetual inventory (CII–CV), witness-of-waste, two-person count, discrepancy reporting, EPCS-compliant ePrescribing, DEA inspection-ready reporting.
15. **Pharmacy reimbursement** — 340B eligibility evaluation and split-billing handoff, PBM (pharmacy benefit manager) NCPDP D.0 claims, payer adjudication, contract-pricing application, copay calculation, hand-off to `cloud-billing` and `cloud-billing-tax` for posting.
16. **Pharmacy operations** — order queue, prep queue, verification queue, delivery queue, pharmacist workload balancing, retrospective and prospective drug-utilization review (DUR).
17. **Pharmacist clinical interventions** — clinical intervention capture, intervention outcome tracking, billing-eligible MTM intervention codes, intervention dashboards.
18. **Discharge medication reconciliation** — admission/transfer/discharge med rec, pre-admission med list reconciliation, discharge summary integration with `emr`.
19. **Outpatient pharmacy** — retail counter dispensing, drive-through, specialty pharmacy (limited-distribution drugs, REMS), mail-order, refill request handling, will-call expiration.
20. **Medication therapy management (MTM)** — comprehensive medication review (CMR), targeted medication review (TMR), medication action plan (MAP), personal medication list (PML), MTM billing codes (CPT 99605–99607).
21. **DSCSA serialization + traceability** — GS1 SGTIN-198 product identifier, lot, expiration, serial; T1/T2/T3 transaction (TI/TH/TS) exchange; saleable returns verification; suspect-product investigation; saleable-return verification (SRV).

### §1.2 Out of scope (anti-goals)

- **Patient charting at-large** — owned by `emr`. Pharmacy only writes back MAR + dispense + intervention.
- **Lab result interpretation** — owned by `diagnostics`. Pharmacy reads lab values via FHIR.
- **Encounter scheduling** — owned by `application` calendar primitives.
- **Payer enrollment** — owned by `crm` and benefits substrates. Pharmacy consumes coverage decisions.
- **Generic billing ledger** — owned by `cloud-billing` and `cloud-billing-tax`. Pharmacy emits dispense events and pricing decisions only.
- **Pharmacy-staff scheduling** — owned by HRIS overlay. Pharmacy consumes shift identity via `identity`.
- **General supply-chain procurement** — owned by `warehouse` and `global-trade`. Pharmacy specializes only in pharmaceutical product flow.
- **Therapeutic guidelines authoring** — pharmacy ingests from licensed knowledge bases (FDB/Multum/Medi-Span) via a vendor-neutral adapter. Pharmacy does not author clinical knowledge.

### §1.3 Why this is its own microservice (per ADR-0131 + ADR-0132)

Pharmacy is single-concern (medication lifecycle from order to administration to settlement) and cannot be folded into `emr` because:

1. **Regulatory surface**: DEA Schedule II EPCS, DSCSA traceability, and state board of pharmacy registration requirements are non-overlapping with EMR HIPAA + Meaningful Use.
2. **Operational tempo**: pharmacy operates a sub-second hot path (BCMA scan, DDI check, dispense decrement) at higher RPS than EMR encounter writes; a separate cell tier and SLO budget are required.
3. **Vendor adapter surface**: Surescripts/NCPDP, Pyxis/Omnicell, IV smart pumps, FDB/Multum represent a distinct integration matrix from EMR's HL7/FHIR exchange plane.
4. **Failure domain**: pharmacy MUST remain available when EMR is degraded (a degraded EMR cannot block administration of an already-ordered medication); decoupling is patient-safety-mandatory.
5. **Auditor surface**: state inspectors audit pharmacy ledgers independently of EMR; a per-microservice audit chain is regulatorily clean.

---

## §2 Bounded contexts (20)

Each bounded context owns its own kernel/domain/usecase/api/sdk/rest/adapter/worker layers under `microservices/pharmacy/src/<context>/<layer>/`. Crate naming follows `oya-pharmacy-<context>-<layer>` per ADR-0056 BNF + ADR-0105 13-layer enum.

### §2.1 MedicationCatalog

**Owns**: the canonical medication identity graph (RxNorm + NDC + GPI + ATC + UNII), ingestion of drug knowledge base vendor packages (FDB / Multum / Medi-Span / WHO ATC), brand-generic linking, route/form/strength axes, package configurations, RxCUI semantic axis maintenance.

**Aggregates**:
- `Medication` (RxCUI root, brand/generic synonyms)
- `Ingredient` (UNII, salt vs. base, parent compound)
- `Package` (NDC11, NDC10, GTIN-14, container sizes, child resistant flag)
- `DrugKnowledgePackage` (vendor + version + checksum)

**Key invariants**:
- Every dispensed medication MUST resolve to a known RxCUI; no free-text medication may pass verification.
- Knowledge package versioning MUST support deterministic A/B switching for safety-rollback.
- NDC11 normalization MUST be canonical (5-4-2 with leading zero pad) before any cross-reference.

### §2.2 Formulary

**Owns**: formulary status (preferred/non-preferred/restricted/non-formulary), criteria-of-use, prior-authorization workflow, therapeutic interchange rules, formulary committee (P&T) workflow.

**Aggregates**:
- `FormularyEntry` (medication, status, tier, criteria, effective dates)
- `TherapeuticInterchangeRule` (source-RxCUI → target-RxCUI, conditions)
- `PriorAuthCriterion` (clinical condition predicates, lab thresholds, age limits)
- `PTCommitteeReview` (proposal, vote, minutes, effective date)

**Key invariants**:
- A medication cannot be dispensed against a "non-formulary" entry without an active override case with reason code and approver identity.
- Therapeutic interchange MUST be transparent in the audit chain.

### §2.3 ePrescribe

**Owns**: NCPDP SCRIPT 2017-071+ message orchestration, Surescripts directory lookup, EPCS signing flow for controlled substances, RxFill event handling, transmission failure retry.

**Message types handled**: NewRx, RxRenewal, RxRenewalResponse, RxChange, RxChangeResponse, CancelRx, CancelRxResponse, RxFill, RxHistoryRequest, RxHistoryResponse, REMSInitiationRequest, REMSInitiationResponse, MedHistory, Status, Error, Verify.

**Aggregates**:
- `EPrescription` (prescriber, patient, medication, sig, quantity, refills, EPCS flag, REMS flag)
- `EPCSAttestation` (DEA number, two-factor evidence, signed envelope, KMS key ref)
- `TransmissionEnvelope` (Surescripts envelope, ack, error, retry state)

**Key invariants**:
- EPCS-flagged messages MUST be signed under the prescriber's individual DEA-bound KMS key (via `cloud-kms`); platform-shared keys are forbidden.
- All Schedule II prescriptions MUST be EPCS or paper-only; no plain electronic for CII.

### §2.4 DrugInteraction

**Owns**: DDI/DAI/DCI/DPI/DDxI/DLI/DFI/DDoseI engine, severity stratification, evidence document linkage, suppression rules per-tenant.

**Sub-engines**:
- DDI (drug-drug)
- DAI (drug-allergy)
- DCI (drug-condition)
- DPI (drug-pregnancy/lactation)
- DDxI (drug-diagnosis)
- DLI (drug-lab)
- DFI (drug-food)
- DDoseI (drug-dose-range)

**Key invariants**:
- Every interaction alert MUST carry an evidence link to the source monograph in the knowledge package.
- Tenant-level suppression of severity-bands BELOW `severe` is allowed; suppression of `severe` and `contraindicated` requires a Cedar override and is audit-sealed.

### §2.5 AllergyCheck

**Owns**: patient allergy list mirroring (read-side from `emr`), allergen normalization to RxNorm ingredient + UNII + SNOMED CT, cross-allergy class derivation, severity-aware allergy match output, allergy override capture.

**Aggregates**:
- `AllergyRecord` (patient, allergen, reaction, severity, onset, source)
- `AllergyCheckResult` (medication, matched allergen, class match, severity, override id if applicable)

**Key invariants**:
- A "match" classification (cross-class) MUST be reported alongside any exact-ingredient match; clinicians choose to ignore class-level matches with justification.
- Override events MUST capture: matched allergen, override reason code, attesting clinician, two-step confirmation if severity ≥ severe.

### §2.6 DoseCheck

**Owns**: DRC engine — weight, BSA, renal (eGFR CKD-EPI, CrCl Cockcroft-Gault), hepatic Child-Pugh, age-band, single/daily/cumulative limits.

**Aggregates**:
- `DosingRule` (medication, indication, route, weight band, organ-function band, single-dose-max, daily-dose-max, lifetime-max)
- `DoseCheckResult` (rule id, value, band, exceed flag, exceed magnitude)

**Key invariants**:
- Pediatric dosing MUST use weight-based calculation when published; defaulting to adult dosing for a pediatric patient is a hard fail.
- Renal dosing MUST use the eGFR/CrCl value that is freshest within the patient's chart; stale (>72h) values trigger a warning band.

### §2.7 Verification

**Owns**: pharmacist verification of orders before dispensing, single-pharmacist vs. dual-pharmacist mode per controlled-substance schedule, tall-man-lettering output, override path.

**Aggregates**:
- `VerificationTicket` (order ref, pharmacist id, decision, timestamp, alerts dismissed)
- `DualVerification` (primary pharmacist, witness pharmacist, both signatures)

**Key invariants**:
- Verification MUST occur after all CDS (DDI/DAI/DCI/DPI/DRC) gates run; dispensing without verification is forbidden.
- Schedule II controlled substances MUST be dual-verified; if single-pharmacy-staff configuration, the dispense MUST be queued until a second pharmacist signs.

### §2.8 Compounding

**Owns**: non-sterile (USP 795), sterile (USP 797), hazardous-drug (USP 800), master formulation record, compounding record, BUD calculation, environmental monitoring evidence binding.

**Aggregates**:
- `MasterFormulationRecord` (ingredients, instructions, equipment, USP class)
- `CompoundingRecord` (master ref, batch id, lot, BUD, ingredients used with lot, environmental log)
- `EnvironmentalMonitoring` (ISO class, particle counts, viable counts, cleaning log, gowning log)

**Key invariants**:
- USP 800 hazardous-drug compounding MUST occur in a negative-pressure ISO 7 buffer room; pharmacy MAY refuse compounding orders if substrate cell does not assert this capability tag.
- BUD MUST be computed via the USP <797> 2024 default table or an in-pharmacy stability study reference; default-table fallback MUST be explicit.

### §2.9 Inventory

**Owns**: per-location par/min/max, on-hand, receiving, put-away, perpetual reconciliation, lot tracking, expiration tiering, recall sequestration.

**Aggregates**:
- `InventoryLot` (medication, lot, expiry, location, quantity, status)
- `ParLevel` (location, medication, par, min, max, refill trigger)
- `RecallNotice` (medication, lot range, severity, scope, sequestration status)

**Key invariants**:
- Recall sequestration MUST block dispense of lots in the recall window; override requires Cedar grant + audit.
- Expiry < 14 days MUST trigger an alert; expiry < 7 days MUST stratify the lot to a "use-first" bucket.

### §2.10 AutoDispensing

**Owns**: integration with automated dispensing cabinets (Pyxis, Omnicell, Carousel, AcuDose, MedDispense); override-event ingestion; cabinet-discrepancy reconciliation.

**Aggregates**:
- `Cabinet` (vendor, location, asset id, slot map)
- `CabinetTransaction` (cabinet, user, medication, qty, override flag, witness)
- `CabinetDiscrepancy` (expected vs. actual, status, resolution)

**Key invariants**:
- Vendor adapters MUST conform to a single contract surface — switching cabinet vendor MUST require zero changes outside `microservices/pharmacy/src/auto-dispensing/adapter-*`.
- Override events MUST be captured even if the cabinet is operating offline; cabinet replay on reconnect.

### §2.11 BCMA

**Owns**: barcode medication administration scan ingestion, five-rights verification, override capture, MAR write-back.

**Aggregates**:
- `AdministrationEvent` (scan id, patient, medication, dose, route, time, nurse, alerts dismissed)
- `FiveRightsResult` (right-patient, right-drug, right-dose, right-route, right-time, all bool)
- `MAR` (admission, medication, scheduled doses, administered doses)

**Key invariants**:
- A scan that fails any one of the five rights MUST hard-block administration; nurse override requires reason code, attestation, and pharmacist callback within 5 minutes.
- Late documentation > 2h after administration MUST be flagged "late doc".

### §2.12 IVAdmixture

**Owns**: IV order to admixture, smart pump drug library push, DERS hard/soft limit programming, pump auto-program via QR/barcode.

**Aggregates**:
- `IVAdmixtureOrder` (base, diluent, additives, rate, duration, route)
- `PumpDrugLibraryEntry` (medication, concentration, rate-range, hard-limits, soft-limits)
- `PumpAutoProgram` (admixture, target pump, programmed values, ack)

**Key invariants**:
- Hard limits MUST never be bypassed by auto-program; soft limit overrides require nurse reason code.
- Concentration mismatch between pharmacy preparation and pump library MUST hard-block.

### §2.13 ControlledSubstance

**Owns**: DEA Form 222 ordering, perpetual inventory CII–CV, witness-of-waste, two-person count, discrepancy reporting, EPCS, DEA inspection report.

**Aggregates**:
- `Form222Order` (supplier, ordered items, signature, audit)
- `CIIInventory` (lot, qty, location, witness-required, last-count)
- `WitnessedWaste` (medication, qty, reason, primary, witness, time)
- `DEAInspectionReport` (period, drugs, transactions, discrepancies, narrative)

**Key invariants**:
- Every CII transaction (receive, dispense, return, waste) MUST have a digital witness signature.
- Perpetual inventory MUST be reconciled to actual count at frequencies prescribed by 21 CFR §1304.

### §2.14 Reimbursement

**Owns**: 340B eligibility evaluation and split-billing handoff, PBM NCPDP D.0 claims, payer adjudication, contract pricing, copay calculation, handoff to `cloud-billing`.

**Aggregates**:
- `Claim` (NCPDP D.0 envelope, recipient, dispense, prices, response)
- `B340Determination` (eligibility, mixed-use disposition, replenishment lot)
- `CopayResult` (insurance plan, member share, plan share, accumulator updates)

**Key invariants**:
- 340B mixed-use determinations MUST be auditable to the level of patient encounter + provider eligibility evidence.
- PBM rejections MUST be classified by NCPDP reject code and routed to pharmacist resolution queue.

### §2.15 Operations

**Owns**: order queue, prep queue, verification queue, delivery queue, pharmacist workload balancing, prospective/retrospective DUR.

**Aggregates**:
- `Workflow` (queue, items, owner, sla)
- `WorkloadMetric` (pharmacist, throughput, dwell, error rate)
- `DURFinding` (retrospective scan, pattern, severity)

### §2.16 Interventions

**Owns**: pharmacist clinical intervention capture, intervention outcome tracking, MTM intervention codes, intervention dashboards.

**Aggregates**:
- `ClinicalIntervention` (medication, problem, intervention, outcome, billable flag)
- `InterventionOutcome` (intervention, follow-up, success)

### §2.17 MedRec

**Owns**: admission/transfer/discharge medication reconciliation, pre-admission med list reconciliation, discharge summary integration with `emr`.

**Aggregates**:
- `ReconciliationSession` (encounter, source list, target list, decisions)
- `ReconciliationDiff` (medication, action: keep/discontinue/modify/add, rationale)

### §2.18 OutpatientRX

**Owns**: retail counter dispensing, drive-through, specialty pharmacy (limited-distribution drugs, REMS), mail-order, refill request handling, will-call expiration.

**Aggregates**:
- `RetailOrder` (script, status, will-call window)
- `SpecialtyEnrollment` (limited-distribution drug, hub-program, patient consent)
- `MailOrderShipment` (order, carrier, tracking, temperature monitor)

### §2.19 MTM

**Owns**: comprehensive medication review (CMR), targeted medication review (TMR), medication action plan (MAP), personal medication list (PML), MTM billing codes.

**Aggregates**:
- `MTMSession` (patient, type, summary, action plan)
- `MedicationActionPlan` (problems, interventions, follow-ups)
- `PersonalMedicationList` (patient-facing PML)

### §2.20 DSCSA

**Owns**: GS1 SGTIN-198 product identifier, lot, expiration, serial; T1/T2/T3 (TI/TH/TS) transaction exchange; saleable returns verification; suspect-product investigation.

**Aggregates**:
- `T3Transaction` (TI + TH + TS bundle)
- `SerialProduct` (SGTIN-198 GS1, lot, expiry, dispensation chain)
- `SaleableReturn` (serial, original transaction, verification result)
- `SuspectProductCase` (serial, suspicion type, investigation, disposition)

---

## §3 Functional requirements (FR)

### FR-1 Medication catalog ingestion
- FR-1.1 The system SHALL ingest FDB MedKnowledge, Multum Lexicon, and Medi-Span knowledge packages via vendor-neutral adapters.
- FR-1.2 The system SHALL normalize NDC10 → NDC11 with leading-zero packing per HRSA spec.
- FR-1.3 The system SHALL maintain RxNorm RxCUI as the canonical identifier and re-link on monthly RxNorm release.
- FR-1.4 The system SHALL maintain ATC L4 classification for therapeutic-class lookups.
- FR-1.5 The system SHALL support A/B switching of knowledge package version per tenant for safety rollback.

### FR-2 Formulary
- FR-2.1 The system SHALL classify every medication as preferred / non-preferred / restricted / non-formulary per tenant.
- FR-2.2 The system SHALL support per-cell formulary overlays.
- FR-2.3 The system SHALL run the P&T committee workflow with proposal, vote, minutes, and effective-date scheduling.
- FR-2.4 The system SHALL support therapeutic interchange with explicit clinician opt-out.
- FR-2.5 The system SHALL allow restricted medications to be dispensed only when criteria-of-use predicates evaluate true.

### FR-3 ePrescribe
- FR-3.1 The system SHALL implement NCPDP SCRIPT 2017-071 (and the next ratified version when released) for all message types listed in §2.3.
- FR-3.2 The system SHALL connect to Surescripts production endpoints over mTLS with rotating client certificates from `cloud-secrets`.
- FR-3.3 The system SHALL implement EPCS signing for Schedule II controlled substances using DEA-bound KMS keys.
- FR-3.4 The system SHALL track REMS requirements at the medication level and gate prescribing accordingly.
- FR-3.5 The system SHALL reconcile Surescripts MedHistoryResponse with the patient's local medication history.

### FR-4 Drug interactions
- FR-4.1 The system SHALL evaluate DDI, DAI, DCI, DPI, DDxI, DLI, DFI, DDoseI on every order entry.
- FR-4.2 Every alert SHALL carry an evidence link to the source monograph.
- FR-4.3 Tenant suppression of severity bands below `severe` SHALL be allowed; suppression of `severe`/`contraindicated` SHALL require Cedar grant.
- FR-4.4 The system SHALL surface a structured override path with reason code, attesting clinician, and two-step for high severity.
- FR-4.5 The system SHALL emit interaction-evaluated and interaction-overridden audit events.

### FR-5 Allergy check
- FR-5.1 Allergy check SHALL match exact-ingredient (RxNorm) AND cross-allergy class.
- FR-5.2 The system SHALL render override path with reason code and attestation; two-step confirmation when severity ≥ severe.
- FR-5.3 Allergy ingestion from `emr` SHALL be incremental and idempotent.

### FR-6 Dose range checking
- FR-6.1 The system SHALL evaluate weight-based, BSA-based, renal-adjusted, hepatic-adjusted, age-banded dosing.
- FR-6.2 Pediatric patients SHALL never default to adult dosing.
- FR-6.3 Renal dosing SHALL use the freshest eGFR/CrCl within 72h; stale values SHALL flag a warning.

### FR-7 Pharmacist verification
- FR-7.1 Pharmacist verification SHALL be mandatory before dispensing.
- FR-7.2 Schedule II controlled substances SHALL be dual-verified.
- FR-7.3 Verification UI SHALL render tall-man-lettering for the medication name.

### FR-8 Compounding
- FR-8.1 The system SHALL support USP 795 / 797 / 800 with appropriate environmental monitoring evidence.
- FR-8.2 BUD SHALL be computed per the USP table or an in-pharmacy stability reference.
- FR-8.3 USP 800 hazardous-drug compounding SHALL be refused on cells without an ISO 7 negative-pressure capability tag.

### FR-9 Inventory
- FR-9.1 The system SHALL track lot, expiry, and location for every unit.
- FR-9.2 Recall sequestration SHALL block dispense of in-window lots.
- FR-9.3 Expiry < 7 days SHALL stratify to a use-first bucket.

### FR-10 Auto-dispensing cabinets
- FR-10.1 The system SHALL support Pyxis, Omnicell, Carousel, AcuDose, MedDispense via vendor-neutral adapters.
- FR-10.2 Override events SHALL be captured even when the cabinet is offline; replay on reconnect.

### FR-11 BCMA
- FR-11.1 The system SHALL evaluate five-rights on every scan.
- FR-11.2 Any failed right SHALL hard-block administration unless overridden with reason and pharmacist callback within 5 minutes.

### FR-12 IV admixture
- FR-12.1 The system SHALL transform IV orders to compound preparations.
- FR-12.2 Smart pump drug library SHALL be pushed via vendor adapter; hard limits non-bypassable.

### FR-13 Controlled substances
- FR-13.1 The system SHALL maintain perpetual inventory for CII–CV.
- FR-13.2 Every CII transaction SHALL be witness-signed.
- FR-13.3 The system SHALL produce DEA inspection-ready reports on demand.

### FR-14 Reimbursement
- FR-14.1 The system SHALL evaluate 340B eligibility per HRSA criteria.
- FR-14.2 The system SHALL submit NCPDP D.0 claims to PBM connections.
- FR-14.3 Claim responses SHALL be reconciled to the local dispense record.

### FR-15 Operations
- FR-15.1 The system SHALL queue orders, prep, verify, deliver with per-step SLAs.
- FR-15.2 The system SHALL balance workload across pharmacists.

### FR-16 Interventions
- FR-16.1 The system SHALL capture clinical interventions with billable-code mapping.
- FR-16.2 The system SHALL track outcomes longitudinally.

### FR-17 Medication reconciliation
- FR-17.1 The system SHALL run admission, transfer, discharge med rec sessions.
- FR-17.2 Diffs SHALL be persisted and signed by the reconciling clinician.

### FR-18 Outpatient pharmacy
- FR-18.1 The system SHALL handle retail counter, drive-through, mail-order, and specialty workflows.
- FR-18.2 Specialty pharmacy SHALL track limited-distribution-drug enrollment, REMS, and hub programs.

### FR-19 MTM
- FR-19.1 The system SHALL conduct CMR/TMR with structured findings.
- FR-19.2 The system SHALL emit MTM billing codes (CPT 99605–99607).

### FR-20 DSCSA
- FR-20.1 The system SHALL maintain SGTIN-198 + lot + expiry + serial for every saleable unit.
- FR-20.2 The system SHALL exchange T1/T2/T3 with upstream wholesalers (GS1 EPCIS-aligned).
- FR-20.3 Saleable returns SHALL be verified before re-stock.
- FR-20.4 Suspect-product investigations SHALL be tracked to disposition.

---

## §4 Non-functional requirements (NFR)

### §4.1 Availability
- NFR-A1: dispense path availability ≥ 99.99% (target; SLO oya-pharmacy-dispense-availability).
- NFR-A2: BCMA scan path availability ≥ 99.99%.
- NFR-A3: ePrescribe outbound transmission availability ≥ 99.9% (Surescripts is dependency tier; degraded mode queues).
- NFR-A4: Catalog read availability ≥ 99.95%.

### §4.2 Latency
- NFR-L1: DDI/DAI/DCI/DPI evaluation p99 ≤ 200 ms.
- NFR-L2: BCMA scan p99 ≤ 100 ms.
- NFR-L3: ePrescribe round-trip p95 ≤ 5 s (excluding pharmacy network return).
- NFR-L4: Dispense cycle (verification to label-print) p99 ≤ 2 s.
- NFR-L5: 340B eligibility evaluation p99 ≤ 50 ms.

### §4.3 Throughput
- NFR-T1: 10,000 BCMA scans / minute per cell.
- NFR-T2: 1,000 dispense events / minute per cell.
- NFR-T3: 50,000 catalog reads / minute per cell.

### §4.4 Durability + retention
- NFR-D1: Dispense and administration events SHALL be retained 10 years (federal default); per-pack overlays may extend.
- NFR-D2: Controlled-substance ledger SHALL be retained per 21 CFR §1304 (currently 2 years federal + state-board extensions).
- NFR-D3: DSCSA transaction data SHALL be retained 6 years post-transaction.

### §4.5 Security
- NFR-S1: All medication data MUST be classified PHI; access SHALL be Cedar-gated and break-glass capable.
- NFR-S2: All EPCS keys MUST be HSM-backed; never present in process memory longer than signing operation.
- NFR-S3: All controlled-substance writes SHALL go through dual-control witness signing.
- NFR-S4: All PBM and Surescripts traffic SHALL be mTLS with rotation interval ≤ 90 days.

### §4.6 Compliance
- NFR-C1: HIPAA 45 CFR §164 covered; BAAs propagated to all sub-processors.
- NFR-C2: DEA 21 CFR §1300–§1321 for controlled substances.
- NFR-C3: State board of pharmacy registration evidence per state of operation.
- NFR-C4: USP <795>, <797>, <800> for compounding.
- NFR-C5: DSCSA Title II (FDASIA) for serialization and traceability.
- NFR-C6: 340B HRSA OPAIS reporting.
- NFR-C7: NCPDP SCRIPT 2017-071+ for ePrescribing.
- NFR-C8: 42 CFR Part 2 for substance-use-disorder treatment medications (with re-disclosure controls).
- NFR-C9: GDPR (EU pack), LGPD (BR pack), PIPA (KR pack), PIPL (CN pack) overlays per `cloud-iam` + compliance packs.

### §4.7 Observability
- NFR-O1: All 20 bounded contexts SHALL emit four-golden-signals (RED + USE) via OpenTelemetry.
- NFR-O2: All Cedar grants SHALL emit policy-decision counter.
- NFR-O3: All DDI/DAI overrides SHALL emit override-rate metric segmented by severity.
- NFR-O4: ≥ 10 self-SLOs registered with `observability` substrate.

### §4.8 Scalability
- NFR-S1: Horizontal scale via cells (`microservices/cell`) and shuffle-sharded tenancy.
- NFR-S2: Read replicas for catalog and formulary (eventual consistency ≤ 60 s).
- NFR-S3: Writes go to leader per cell.

### §4.9 Resilience
- NFR-R1: BCMA degraded mode SHALL allow scan capture during a control-plane outage and replay on recovery.
- NFR-R2: Auto-dispensing cabinets SHALL operate in offline mode (last-known-state) with reconciliation on reconnect.
- NFR-R3: Surescripts outbound queue SHALL preserve message order per (prescriber, patient).

### §4.10 DR Posture (ADR-0343)

- Target: RTO 900s and RPO 60s for dispense, BCMA, EPCS, allergy/DDI checks, MAR write-back, controlled-substance ledger, and DSCSA events, matching `manifest.json` `dr.rto_p99_seconds=900` and `dr.rpo_p99_seconds=60`.
- Compliance floors: HIPAA-2024 floors at 3600s/300s with multi-region required; PCI-DSS-L1-v4 floors at 86400s/3600s for payment and PBM-linked paths; SOC2-T2 floors at 14400s/900s; ISO27001-2022 floors at 14400s/3600s; KR-PIPA sensitive-PI floor at 7200s/600s. The effective pharmacy target remains 900s/60s for safety paths.
- failover_runbook: `microservices/pharmacy/runbooks/pharmacy-dispense-failover.md`.
- multi_region_active_active: true for inpatient dispense, BCMA, controlled-substance custody, EPCS signing evidence, and DSCSA traceability.
- Why: medication verification, cabinet override reconciliation, bedside administration, and controlled-substance accountability continue when a region or cell fails.

### §4.11 Capacity Model (ADR-0340)

- Per-tenant baseline: 0.55 vCPU, 1280 MiB RAM, 20 GB medication, inventory, and ledger storage, 8 Postgres connections, 8 Valkey connections, and 12 outbound HTTP connections, matching `manifest.json` `capacity_model`.
- Scaling dimension: `per_workflow_run` for dispense, verification, EPCS, BCMA, DSCSA, and reimbursement workflows.
- Cell placement class: Tier-2. Pharmacy pairs `pod_runtime_tier=1` with regulated tenant medication data-plane isolation without claiming tenant-code execution.
- Autoscaling boundaries: min 2 pods per tenant cell, max 28 pods per tenant cell before pharmacy location, cabinet, or queue partition review.
- Why: this serves workflow-run-driven medication safety and settlement paths without coupling pharmacy safety workflows to EMR availability.

### §4.12 Sustainability + Cost Attribution (ADR-0344)

- Emission envelope: every pharmacy audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider-routing affected by carbon: no for DDI/DAI, dose checks, BCMA, controlled-substance dispense, EPCS, or PCI real-time fraud/payment checks; yes for catalog ingestion, formulary rebuilds, and non-urgent KPI/data-warehouse streams.
- Tenant cost transparency: `finops-portal` exposes pharmacy cost rows by dispense event, catalog ingest, formulary build, Surescripts/PBM path, 340B/DSCSA evidence, and data-warehouse stream.
- Why: CSRD, SB-253, and SEC climate-disclosure obligations apply to medication operations, but safety and controlled-substance workflows cannot be carbon-delayed.

### §4.13 API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet across `Oyatie-Api-Version`, `/v/YYYY-MM-DD/...` URL prefixes, and proto3 `api_version_date` fields.
- SDK semver model: major.minor.patch for prescriber, pharmacy-network, BCMA, cabinet, pump, PBM, and NCPDP/Surescripts clients.
- Support window: last 3 public versions supported for at least 180 days.
- Per-tenant pinning: yes for pharmacy-network, Surescripts, NCPDP, PBM, cabinet, and smart-pump integrations.
- Internal-mesh exemption: yes; direct gRPC handoffs to EMR, diagnostics, cloud-billing, cloud-billing-tax, and audit-chain preserve ADR-0145.

---

## §5 Domain glossary (selected)

- **NDC** — National Drug Code (FDA, 11-digit normalized).
- **RxCUI** — RxNorm concept unique identifier; canonical medication identity.
- **GPI** — Generic Product Identifier; therapeutic class hierarchy.
- **ATC** — Anatomical Therapeutic Chemical (WHO); five-level therapeutic class.
- **EPCS** — Electronic Prescribing of Controlled Substances (DEA).
- **NCPDP SCRIPT** — National Council for Prescription Drug Programs standard for ePrescribing.
- **DSCSA** — Drug Supply Chain Security Act (US Title II FDASIA, 2013).
- **DDI** — drug-drug interaction.
- **DAI** — drug-allergy/intolerance.
- **DCI** — drug-condition (e.g., metformin + acute kidney injury).
- **DPI** — drug-pregnancy/lactation.
- **DDxI** — drug-diagnosis.
- **DLI** — drug-lab.
- **DFI** — drug-food.
- **DDoseI** — drug-dose-range.
- **DRC** — dose range checking.
- **BCMA** — barcode medication administration.
- **MAR** — medication administration record.
- **DERS** — dose error reduction system (IV smart pumps).
- **DUR** — drug utilization review.
- **MTM** — medication therapy management.
- **CMR** — comprehensive medication review.
- **TMR** — targeted medication review.
- **MAP** — medication action plan.
- **PML** — personal medication list.
- **PBM** — pharmacy benefit manager.
- **340B** — HRSA covered-entity drug pricing program.
- **USP 795/797/800** — non-sterile / sterile / hazardous compounding standards.
- **BUD** — beyond-use date.
- **SGTIN-198** — GS1 Serialized Global Trade Item Number, 198-bit, for DSCSA.
- **EPCIS** — Electronic Product Code Information Services (GS1).
- **TI / TH / TS** — DSCSA Transaction Information / History / Statement.
- **MedKnowledge / Multum / Medi-Span** — leading commercial drug knowledge bases.
- **REMS** — Risk Evaluation and Mitigation Strategy (FDA).
- **Tall-man lettering** — case-mixed rendering to disambiguate look-alike drug names (e.g., hydrOXYzine vs. hydrALAZINE).

---

## §6 Inter-microservice dependencies

| Depends on | Via | Reason |
| --- | --- | --- |
| `audit-chain` | `oya-audit-chain-emission-sdk` | Bilateral seal of every controlled-substance event, override, EPCS signature, DSCSA T3. |
| `emr` | gRPC `emr.Patient`, `emr.Encounter`, `emr.Allergy`, `emr.Problem`, `emr.Labs`, `emr.MAR` | Patient identity, allergies, problems, labs (renal, hepatic), MAR write-back. |
| `identity` | OIDC + Cedar principal | Prescriber DEA identity, pharmacist license, nurse identity, EPCS two-factor evidence. |
| `cloud-iam` | Cedar policy gate | Every guarded action authorized; break-glass logged. |
| `cloud-kms` | KMS sign + verify | EPCS DEA-bound keys, audit-chain Merkle keys, PBM mTLS material. |
| `cloud-secrets` | secret fetch | Surescripts mTLS, PBM NCPDP credentials, FDB/Multum API keys. |
| `observability` | OpenTelemetry + OpenSLO | 10+ SLOs + RED/USE. |
| `cloud-billing` | dispense event handoff | Pharmacy emits dispense to billing for charge posting. |
| `cloud-billing-tax` | tax determination | State drug tax (rare; some states tax) + sales tax overlays. |
| `compliance` | pack registration | HIPAA + DEA + state-board pack hooks. |
| `governance` | review-and-audit lane | Policy changes flow through governance pipeline. |
| `intelligence` | LLM substrate (read-only) | MTM patient-letter drafting and PML rendering with redaction. |
| `analytics` | dispense fact + MAR fact streams | Pharmacy KPIs in `data-warehouse`. |
| `community` | patient-facing notifications | Refill ready, will-call expiring (B2C surface). |
| `healthcare-integration` | inbound HL7v2 / FHIR | Outside-system med orders, refill renewals from PMP. |
| `comms-email` | refill reminders | Outpatient refill reminders. |
| `forms` | clinician acknowledgments | REMS attestations, EPCS step-up forms. |
| `consent-graph` | cross-tenant data sharing | Specialty-pharmacy hub program data-sharing agreements. |
| `cell` | cell topology | Pharmacy is T0; runs on dedicated patient-safety cell tier. |
| `tenancy` | tenant resolution | Per-tenant formulary, per-tenant pack. |

---

## §7 Capability tiers (per ADR-0130 + ADR-0328)

- **T0 (no agency)** — read-only catalog and formulary reads; allergy mirroring; DSCSA pedigree lookups.
- **T1 (low agency)** — DDI/DAI evaluation, DRC evaluation, dose calculator, BCMA verify (decision is human's).
- **T2 (limited agency)** — order queue routing, prep queue assignment, refill request triage, intervention drafting (pharmacist final review).
- **T3 (high agency, narrow)** — therapeutic interchange within formulary, 340B mixed-use classification, automated PBM resubmission within configured rules.
- **T4 (high agency, broad)** — RESERVED. No pharmacy capability runs at T4 without explicit per-tenant pharmacist-in-charge sign-off and a Cedar-gated capability principal.

---

## §8 Data flows

### §8.1 Inpatient order-to-administration

1. Prescriber writes order in `emr`.
2. `emr` emits `medication-request-created`.
3. Pharmacy ingests; runs DDI/DAI/DCI/DPI/DRC/duplicate-therapy.
4. Pharmacist verifies (dual-verify if CII).
5. Pharmacy emits `medication-dispense-prepared`.
6. Auto-dispensing cabinet pulls (Pyxis/Omnicell); BCMA scan at bedside.
7. Nurse scans patient + medication; five-rights validated; administration recorded.
8. Pharmacy emits `medication-administration-recorded`; MAR written back to `emr`.
9. Pharmacy emits `medication-dispense-settled` to `cloud-billing`.
10. Audit chain seals all 9 hops bilaterally.

### §8.2 Outpatient retail dispensing

1. Surescripts NewRx inbound.
2. Pharmacy enrolls patient (if new); resolves payer coverage.
3. PBM NCPDP D.0 claim submitted; response reconciled.
4. Pharmacist verifies; prep queue.
5. Patient picks up at counter; barcode scan; counsel completed.
6. Dispense settled to `cloud-billing`.
7. Refill reminders scheduled in `comms-email`.

### §8.3 EPCS Schedule II flow

1. Prescriber initiates Schedule II Rx in `emr`.
2. Pharmacy gateway requests EPCS sign envelope.
3. `cloud-kms` returns DEA-bound signature; two-factor evidence captured.
4. Surescripts EPCS message transmitted.
5. Pharmacy receives at receiving institution; dual-pharmacist verification.
6. Audit-sealed at every hop.

### §8.4 DSCSA T3 verification on receipt

1. Wholesaler delivers product; serial scans at receiving dock.
2. Each SGTIN-198 verified against received T3.
3. Suspect serials raise SuspectProductCase.
4. Verified serials enter perpetual inventory.

---

## §9 Acceptance criteria (entrance into M16 promotion gate)

- AC-1: All 20 bounded contexts have kernel/domain/usecase/adapter/api/sdk/rest/worker stubs registered in `Cargo.toml`.
- AC-2: All 10+ SLOs published as OpenSLO under `microservices/pharmacy/slos/`.
- AC-3: At least 5 Cedar policy files registered under `microservices/pharmacy/policies/`.
- AC-4: OpenAPI surface covers FHIR Medication, MedicationRequest, MedicationDispense, MedicationAdministration, MedicationStatement, plus pharmacy-specific extensions.
- AC-5: AsyncAPI surface covers `rx.prescribed`, `rx.verified`, `rx.dispensed`, `rx.administered`, `rx.refused`, `rx.alert.ddi`.
- AC-6: Proto definitions cover all internal gRPC contracts.
- AC-7: IaC under all 6 deployment contexts (aws-guest, oci-guest, on-prem, colo, oyatie-cloud, sovereign).
- AC-8: OCI always-free module present for sandbox/demo/trial tenants.
- AC-9: Service-level ADRs ADR-MS-001 and ADR-MS-002 land alongside manifest.
- AC-10: 10 implementation plans (IP-001..IP-010) drafted.
- AC-11: Competitor parity matrix UNION of Cerner + Epic Willow + Pyxis covers ≥ 100 capabilities.
- AC-12: `supported-oses.json` declares Tier-1 OS matrix per `feedback_os_support_matrix_2026_05_20`.
- AC-13: HIPAA + DEA controlled-substance pack hooks present in `manifest.json`.

---

## §10 Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| DEA EPCS audit failure | KMS-only DEA signing, two-factor enforced, audit chain sealed, quarterly DEA-pattern self-audit. |
| 340B mixed-use misclassification (HRSA finding) | Auditable encounter evidence + replenishment lot tagging; quarterly OPAIS-format reporting. |
| Surescripts certificate rotation outage | 90-day rotation with 7-day overlap; cert health check + ops runbook RUN-001. |
| Cabinet vendor lock-in | Vendor-neutral adapter contract; switch-vendor smoke test included in CI. |
| FDB/Multum knowledge package regression | A/B switching per tenant; rollback inside 5 min; smoke tests on every package upgrade. |
| Allergy cross-class false positives | Tunable class-match suppression with audit; per-tenant baseline. |
| BCMA scan false-pass | Five-rights hard block; nurse override requires pharmacist callback within 5 min. |
| DSCSA serial collision | SGTIN-198 deterministic lookup; suspect-product workflow. |
| Compounding BUD miscalculation | USP-table fallback explicit; stability-study reference required for exceptions. |
| PBM rejection loop | NCPDP reject classifier + pharmacist resolution queue with SLA. |

---

## §11 Versioning + compatibility

- OpenAPI under `contracts/openapi/pharmacy.yaml` version `1.0.0` with stable URL pattern `/api/v1/pharmacy/...`.
- AsyncAPI under `contracts/asyncapi/pharmacy-events.yaml` version `1.0.0` with topic prefix `oya.pharmacy.<event>`.
- Proto under `contracts/proto/pharmacy.proto` versioned `oya.pharmacy.v1` package.
- All breaking changes follow `feedback_no_silent_regression`: ADR + version bump + sunset.

---

## §12 References

- ADR-0131 per-microservice flat layout.
- ADR-0132 no-suite microservices.
- ADR-0145 inter-microservice direct-gRPC + 3 invariants.
- ADR-0251 compliance-pack primitive.
- ADR-0328 multispectrum review v2.4.0 overlay.
- ADR-0332 pharmacy substrate authorization (NEW; this microservice).
- 21 CFR §1300–§1321 (DEA controlled substances).
- 45 CFR §164 (HIPAA Security + Privacy).
- USP <795>, <797>, <800>.
- NCPDP SCRIPT 2017-071+.
- DSCSA (Title II FDASIA 2013).
- 42 CFR Part 2.
- 340B Program HRSA OPA.
- HL7 FHIR R5 Medication resources.
- GS1 SGTIN-198 + EPCIS 2.0.

---

## §13 Capability list (parity-driven, ≥ 100)

See `competitor-parity-matrix.md` for the full enumerated capability list and counterpart coverage. The capability index is the source of truth for what pharmacy promises.

---

## §14 Open questions

- OQ-1: REMS-as-a-service exposed to specialty hub partners — capability tier T3 or T4? (default T3 pending PIC sign-off).
- OQ-2: Cross-state pharmacy practice (e.g., a pharmacist in TX dispensing for a CA patient) — license-residency gate handled by `cloud-iam` or pharmacy-local? (default `cloud-iam` enforces residency; pharmacy consumes the decision).
- OQ-3: Cannabis (state-legal, federally Schedule I) — out of scope per default; tenant-pack overlay opens it where state-legal? (out of scope until ADR-MS-003 lands).
- OQ-4: AI-generated PML in MTM — intelligence-substrate-driven; subject to `eu-ai-act` and `intelligence` redaction contract.
- OQ-5: Long-acting injectable (LAI) administration outside the inpatient setting — does it live in `outpatient-rx` or a new bounded context? (default: `outpatient-rx` with specialty pharmacy overlay).
- OQ-6: Investigational drugs (IND) — out of scope for now; if a tenant runs a research pharmacy, a new bounded context `investigational-rx` will be authored under a separate IP.
- OQ-7: Pet/veterinary pharmacy — explicit anti-goal. Veterinary tenants must use a dedicated vertical microservice.
- OQ-8: Nuclear pharmacy (radio-pharmaceuticals) — anti-goal for v1.0; revisit in v2.0 with NRC overlay.

---

## §15 Bounded-context interaction matrix

The 20 bounded contexts do not stand alone — every patient-safety outcome depends on inter-context handoffs. The following matrix is the source of truth for who-calls-whom inside the pharmacy microservice.

| From → To | Synchronous gRPC | Asynchronous event | Notes |
|---|---|---|---|
| ePrescribe → DrugInteraction | `Evaluate(patient, rxcuis, engines=ALL)` | — | Blocking; SLO p99 ≤ 200 ms. |
| ePrescribe → AllergyCheck | `Check(patient, rxcui)` | — | Blocking. |
| ePrescribe → DoseCheck | `Check(...)` | — | Blocking. |
| ePrescribe → Formulary | `Lookup(tenant, cell, rxcui)` | — | Blocking; cached 5 s. |
| ePrescribe → MedicationCatalog | `Resolve(rxcui)` | — | Cached 60 s read-through. |
| ePrescribe → Verification | — | `rx.prescribed` | Verification queue subscribes. |
| Verification → ePrescribe | `EPCSSignEnvelope` (CII–CV) | — | Blocking only if outbound. |
| Verification → ControlledSubstance | `RecordVerification(...)` | `controlled.witness-signed` | Dual-verify for CII. |
| Inventory → Verification | — | `inventory.recall-sequestered` | Verification queue refuses sequestered. |
| AutoDispensing → Inventory | `Decrement(lot, qty)` | `cabinet.discrepancy` if drift > 0 | Transactional. |
| AutoDispensing → BCMA | — | `cabinet.dispensed-to-bed` | BCMA expects scan within window. |
| BCMA → emr (out) | gRPC `emr.MAR.write` | `rx.administered` | Write-back to chart. |
| IVAdmixture → Compounding | `OpenRecord(USP_797 or USP_800)` | — | Compounding workflow attached. |
| IVAdmixture → AutoDispensing | `Decrement` for base + diluent + additives | — | Multi-lot transactional. |
| Reimbursement → MedicationCatalog | `Resolve(ndc)` | — | NDC-driven PBM contract pricing. |
| Reimbursement → cloud-billing (out) | gRPC `cloud-billing.Charge` | `reimbursement.claim-accepted` | Settlement. |
| Operations → all queues | reads queue depth | — | Workload balancer. |
| MedRec → emr (out) | `emr.MedicationStatement.read` | — | Pre-admission med list. |
| MTM → intelligence (out) | `intelligence.Compose(redaction_profile=mtm-pml)` | — | LLM-drafted PML; pharmacist edits. |
| DSCSA → Inventory | `IngestSerial(sgtin, lot)` | — | Serials enter perpetual inventory. |
| DSCSA → emr (out) | `emr.MedicationDispense.attach_sgtin` | — | DSCSA serial attached to dispense record. |

Every async event is sealed in `audit-chain`. Every cross-tenant flow (specialty hub) is also sealed with bilateral cross-pointer per ADR-0214.

---

## §16 Patient-safety guardrails (hard blocks)

The following are HARD BLOCKS: pharmacy MUST refuse the action and the action MUST NOT be retried without an explicit Cedar-gated override.

- HB-1: Verifying without all CDS gates run (DDI/DAI/DCI/DPI/DRC + duplicate therapy).
- HB-2: Dispensing CII without dual-pharmacist verification.
- HB-3: Dispensing without barcode match on selected product.
- HB-4: Pediatric dose defaulting to adult dose without weight-based calculation.
- HB-5: Recall-sequestered lot reaching a patient.
- HB-6: USP 800 compounding on a cell without ISO-7 negative-pressure capability.
- HB-7: BCMA five-rights failure without explicit override + pharmacist callback in 5 min.
- HB-8: EPCS signing without DEA-bound KMS key + two-factor evidence.
- HB-9: Therapeutic interchange across a different ATC L4 class without explicit prescriber order.
- HB-10: Non-formulary dispense without medical-director approval case.
- HB-11: Allergy override at severity ≥ severe without two-step + reason code.
- HB-12: Smart-pump program with concentration mismatch between pharmacy prep and pump library.
- HB-13: Cabinet override on schedule III+ without witness signature.
- HB-14: DSCSA suspect serial reaching dispense queue.
- HB-15: 340B classification with stale eligibility evidence (>24 h).

Each guard publishes a counter under `oya_pharmacy_hard_block_total{kind, tenant}` so an upstream regression in alert flow is immediately visible.

---

## §17 Privacy + minimum-necessary controls

- PHI fields in events MUST be redacted to the level required by the consumer's `intent_class`. The pharmacy event envelope carries `data_classification` per ADR-0244.
- MTM PML drafts via `intelligence` MUST use the `mtm-pml` redaction profile: drop direct identifiers; surface clinical content only.
- Specialty pharmacy hub partners SHALL receive a projected view defined by a `DataSharingAgreement` (consent-graph) — never the raw record.
- Break-glass elevation SHALL be sealed under `oya.pharmacy.access.break-glass` with mandatory 24-hour governance review.
- Patient self-service B2C interactions SHALL never return controlled-substance schedule III+ data through a public channel; specialty handling required.

---

## §18 Disaster recovery + data sovereignty

- **RPO**: dispense + administration events ≤ 60 s (synchronous emission to per-cell Pulsar tier-1 + tier-2 storage).
- **RTO**: per-cell ≤ 5 min via cell-failover; cross-cell ≤ 15 min via shuffle-sharding rebalancing.
- Data sovereignty pin per tenant (e.g., EU, KR, US-only, sovereign realm); cross-region copy forbidden unless explicit `DataSharingAgreement` covers it.
- DR drill cadence: quarterly cell-fail; biannual cross-cell.
- DSCSA T1/T2/T3 history retained 6 years; controlled-substance ledger per 21 CFR §1304 (federal floor 2 years + state overlays).

---

## §19 Failure mode envelope per bounded context

| Bounded context | Primary failure mode | Mitigation | Degraded behavior |
|---|---|---|---|
| MedicationCatalog | Knowledge package corrupt | A/B rollback < 5 min | Read from last good version |
| Formulary | Effective-date drift | HLC + deterministic worker | Fail closed on ambiguous effective date |
| ePrescribe | Surescripts down | Outbound queue per (Rx, patient) | Queue and replay |
| DrugInteraction | Engine slowness | Per-engine fan-out + timeout 200 ms | Return "interaction-eval-degraded" alert |
| AllergyCheck | EMR allergy mirror lag | Use stale; mark `staleness_seconds` | Allow with elevated alert |
| DoseCheck | Lab value stale (>72h) | Flag stale; require fresher | Allow with WARN band |
| Verification | Pharmacist absent for CII dual | Queue with SLA | Hold dispense |
| Compounding | Cell capability missing | Refuse compound | Pharmacist re-routes |
| Inventory | Recall feed missing | Conservative sequestration | Refuse dispense; alert |
| AutoDispensing | Cabinet offline | Cache + reconcile | Local cache hot path |
| BCMA | Endpoint timeout | Local cache | Pharmacist callback in 5 min |
| IVAdmixture | Pump library push fail | Manual pump program with audit | Hard limits unchanged |
| ControlledSubstance | Witness pharmacist absent | Queue with SLA | Hold |
| Reimbursement | PBM endpoint down | Queue claim | Submit at recovery |
| Operations | Queue grow > threshold | HPA + alert | Workload rebalance |
| Interventions | Intervention drop | Idempotent re-queue | Replay |
| MedRec | EMR med list missing | Use last admission list | Mark "pending-emr-merge" |
| OutpatientRX | Specialty hub down | Queue refill | Local will-call extension |
| MTM | Intelligence substrate down | Pharmacist drafts manually | Without LLM draft |
| DSCSA | EPCIS partner down | Queue T3 ingest | Replay |

---

## §20 Sub-microservice deferred-feature list (not in v1.0)

- Cannabis (state-legal) — needs Schedule I federal carve-out work (OQ-3).
- Veterinary pharmacy — separate vertical.
- Nuclear pharmacy — needs NRC overlay.
- Investigational drugs (IND) — separate bounded context.
- 503A/503B outsourced compounding registry sync — deferred to v1.1.
- DoseMe Bayesian dosing optimizer — deferred to v1.1 intelligence-substrate integration.
- Pharmacogenomics (PGx) clinical decision support — deferred to v1.1 (CYP2D6, CYP2C19, TPMT phenotypes need genomics substrate).
- Telepharmacy supervisor remote verification — deferred to v1.1 with `application` calendar integration.
- 503B outsourcing facility receiving handshake — deferred to v1.1.

---

## §21 Quality bar (per `feedback_quality_performance_scalability_bar`)

Pharmacy is held to:

- **Quality** — Stripe/Palantir/Linear-grade APIs (idempotency keys on every write; ETag on every read; documented version pin).
- **Performance** — hyperscaler-grade hot path (p99 < 200 ms for CDS; p99 < 100 ms for BCMA; p95 < 5 s for ePrescribe round-trip).
- **Scalability** — horizontal via cells + shuffle sharding (per ADR-0248); read replicas for catalog/formulary; per-cell leader writes.

All CI lanes enforce these:

- `lean-a10-no-silent-regression` — public-contract drift fails the build.
- `lean-a5-doc-coverage` — every µservice ships full doc suite.
- `pharmacy-os-tier-1` — Tier-1 OS lane blocking.
- `pharmacy-os-tier-2` — Tier-2 soft-gate.
- `pharmacy-cabinet-vendor-swap-smoke` — switch-vendor smoke per IP-009.
- `pharmacy-knowledge-package-ab-smoke` — A/B rollback smoke per IP-001.
- `pharmacy-cedar-policy-compile` — Cedar policies compile.
- `pharmacy-fhir-conformance` — FHIR R5 Medication-family conformance.
- `pharmacy-ncpdp-codec-roundtrip` — NCPDP SCRIPT 2017-071 codec.
- `pharmacy-dscsa-epcis-2.0-conformance` — DSCSA EPCIS 2.0.

---

---

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

<!--
COMPLETION REPORT
microservice: pharmacy
authoring_wave: 15M-E
authoring_date: 2026-05-21
sole_owner: pharmacy authoring agent
authority_adr: ADR-0332
layout_authority: ADR-0131
suite_policy: ADR-0132
bounded_contexts_authored: 20 (MedicationCatalog, Formulary, ePrescribe, DrugInteraction, AllergyCheck, DoseCheck, Verification, Compounding, Inventory, AutoDispensing, BCMA, IVAdmixture, ControlledSubstance, Reimbursement, Operations, Interventions, MedRec, OutpatientRX, MTM, DSCSA)
slos_authored: 12
cedar_policies_authored: 5
implementation_plans_authored: 10
service_level_adrs_authored: 2
contracts: openapi.yaml + asyncapi.yaml + pharmacy.proto
iac_deployment_contexts: 6 (aws-guest, oci-guest, on-prem, colo, oyatie-cloud, sovereign) plus always-free
competitor_parity_matrix: Cerner Pharmacy Manager (Oracle Health) + Epic Willow + BD Pyxis + Omnicell + McKesson EnterpriseRx UNION; ≥100 capabilities enumerated
compliance_pack_hooks: hipaa + dea-controlled-substance + gdpr + pci-dss + eu-ai-act + lgpd + cn-pipl-2021 + kr-pipa + state-board-of-pharmacy + dscsa + usp-797 + usp-800 + 340b + ncpdp-script + surescripts
supported_oses_authored: yes (Tier-1 blocking + Tier-2 soft-gate per feedback_os_support_matrix_2026_05_20)
manifest_authored: yes
prd_line_count_target: ≥800
architecture_line_count_target: ≥600
readme_line_count_target: ≥300
authoring_status: DELIVERABLES_COMPLETE
-->
