---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-emergency
microservice: emergency
title: Emergency Department Information System (ED-IS)
status: scaffold-authored-2026-05-21
date: 2026-05-21
owner_team: axis-emergency + council-clinical + council-product
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0248
  - ADR-0251
  - ADR-0253
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

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# PRD — Emergency Department Information System (ED-IS)

Microservice slug: `emergency`
Owner role: emergency-medicine-platform-engineer (single-owner end-to-end per ADR-0131 + µservice ownership memory)
Authority: ADR-0332 (in flight) | ADR-0328 | ADR-0131 | ADR-0132 | ADR-0251
Status: scaffold-authored 2026-05-21

---

## 0. Purpose

The Emergency Department Information System (ED-IS) is the operational nervous system of a hospital's Emergency Department (ED). It owns every workflow from the moment a patient arrives — by ambulance, walk-in, transfer, or as a mass-casualty victim — through triage, room placement, evaluation, treatment, and final disposition (admit, transfer, discharge, AMA, expired). It is the µservice that the ED charge nurse, attending physician, resident, scribe, registration clerk, and EMS partner live in for the duration of a shift.

ED-IS exists as a distinct µservice from the general-purpose EMR because emergency medicine has a fundamentally different shape:

- **Tempo**: Median patient on the tracking board for under 4 hours, not 4 days. Decisions land in seconds, not rounds.
- **Acuity**: Triage acuity (ESI 1-5) re-shapes the entire UI and notification routing per patient.
- **Concurrency**: A single attending may carry 8-15 patients simultaneously across phases.
- **Protocols**: Trauma / Stroke / STEMI / Sepsis alerts override normal flow, mute the tracking board, and lock in time-stamped bundle adherence.
- **Mass-casualty contingency**: The system must instantly pivot from normal individual-patient mode to MCI mode with START/SALT triage and surge bed assignment.
- **Boarding**: When inpatient beds fill, admitted patients pile up in the ED — a public-policy and quality crisis that demands its own first-class telemetry.
- **Door-to-X metrics**: Door-to-doctor, door-to-CT, door-to-needle, door-to-balloon, door-to-disposition — every minute is regulatorily and reputationally tracked.

ED-IS is therefore not a "view" on the EMR. It is a peer µservice that publishes encounters into the EMR (`emr` µservice) via FHIR, consumes patient lookup and prior-history reads, and projects ED-specific KPIs and registry feeds outward. It is the canonical home of every ED-shaped business rule.

---

## 1. Counterpart Landscape (Top-3 + Secondary)

| Rank | Product | Vendor | Anchor strength | Where ED-IS aims to exceed |
|------|---------|--------|------------------|----------------------------|
| 1 | T-System EDIS | Hyland | Complaint-driven structured templates; the de facto template library for ED documentation in mid-market US EDs. | Bring template authoring under a first-class tenant overlay (KR-pack / EU-pack) and version every template with attestable provenance via `audit-chain`. |
| 2 | Wellsoft EDIS | Wellsoft (Medsphere) | Pure-play EDIS; lean clinical workflow; loved by community hospitals. | Wellsoft is monolithic on-prem-first; ED-IS gives the same focus but with cell-tier deployability per ADR-0248 and OpenSLO-gated promotion. |
| 3 | Cerner FirstNet (Oracle Health) | Oracle Health | Deep tracking board + EMS integration + tight Millennium coupling. | FirstNet is locked to Oracle's EMR; ED-IS interoperates with any FHIR-conformant EMR (including the oyatie `emr` µservice or third-party Epic/Cerner) and is BYOK on every external credential. |
| Sec | Epic ASAP | Epic Systems | Best-in-class triage and tracking when paired with Epic Hyperspace. | ED-IS publishes FHIR R4B Encounter/EpisodeOfCare so it can drop into an Epic-anchored shop as an adjunct without ripping. |
| Sec | Picis CareSuite | Picis | OR / ICU / ED suite; strong critical-care orientation. | ED-IS deliberately does not own OR/ICU (those live in distinct µservices); single-concern + flat per ADR-0131 + ADR-0132. |
| Sec | Medhost EDIS | Medhost | Community-hospital EDIS with tight RCM tie-in. | ED-IS hands off to `cloud-billing-tax` and `crm` via async events, never co-locating billing. |
| Sec | TeleTracking ED Tracker | TeleTracking | Hospital-wide flow visibility + bed control. | ED-IS exposes `ed.boarding.threshold` and `ed.bed.requested` events so TeleTracking-style flow systems can subscribe without polling. |

A union of capabilities across all seven is captured in `competitor-parity-matrix.md` (≥100 capabilities).

---

## 2. Bounded Contexts

ED-IS is a single µservice but is structured into 17 bounded contexts, each owning a coherent slice of the ED problem space. The contexts are deliberately granular so that each maps to one aggregate root, one event prefix, one Cedar policy file, one OpenSLO file, and a clearly-bounded set of FHIR resources.

### 2.1 Triage (`triage.*`)

**Mission.** Convert an arriving patient into an ESI-classified acuity record within the regulator-prescribed door-to-triage window, then re-triage on demand as acuity drifts.

**Aggregate.** `TriageEncounter` rooted on `encounter_id`.

**Key rules.**
- ESI-1 ("emergent — immediately life-threatening") triggers immediate room placement and resus protocol; bed must be locked within 0 minutes of triage close.
- ESI-2 ("urgent — high-risk situation, severe pain/distress") drops a wait-time threshold marker into the tracking board.
- ESI-3/4/5 follow standard wait-time pipelines.
- Triage cannot complete without: vitals snapshot (HR, RR, BP, SpO2, temperature, pain score), chief complaint, and a typed acuity decision.
- Re-triage events are first-class — a patient can be moved up or down without retroactively rewriting prior triage records.

**Inputs.** EMS prehospital report (if ambulance), walk-in registration packet, prior-encounter lookup from `emr`.
**Outputs.** `ed.triage.completed` and `ed.triage.reassessed` events; FHIR `Observation` (triage acuity) projected to `emr`.

### 2.2 Tracking Board (`trackingboard.*`)

**Mission.** Provide a real-time visual board of every patient currently in the ED, segmented by bed/zone, acuity, status, elapsed time, and pending tasks. The board is the single most consulted surface in the ED.

**Aggregate.** `BoardLane` (one per ED zone) plus `BoardCell` (one per bed slot).

**Key rules.**
- Every patient transition produces a board update within 250 ms p99.
- Acuity color-coding is overlay-pack-driven (default scheme is ESI-canonical; KR pack overrides).
- Board cells can be in states: `empty`, `awaiting-room`, `roomed`, `in-evaluation`, `awaiting-result`, `awaiting-disposition`, `boarding`, `cleaning`, `out-of-service`.
- Charge nurse can swap bed assignments; the swap publishes `ed.bed.reassigned` carrying both prior and new bed IDs and the actor identity.

**Outputs.** SSE / WebSocket fanout to ED dashboards; CRDT-backed offline tolerance on the device side.

### 2.3 Protocol (`protocol.*`)

**Mission.** Execute fixed-bundle, time-locked clinical protocols — Trauma Alert, Stroke Alert, STEMI Alert, Sepsis Alert, and any tenant-configured local alert (e.g., Pediatric Mass Transfusion Protocol).

**Aggregate.** `ProtocolActivation`.

**Key rules.**
- Activation timestamp is immutable and audit-stamped via `audit-chain`.
- Each protocol has a fixed checklist with target windows (e.g., STEMI door-to-balloon ≤ 90 min, Stroke door-to-CT ≤ 25 min, Sepsis bundle within 3 hours).
- Missed window emits `ed.protocol.window.breached`.
- Protocols can be deactivated (false-alert) only by an attending or charge nurse with documented reason.
- Tenant packs can add protocols but cannot remove or weaken canonical-base protocols (canonical-base neutrality per ADR-0064).

### 2.4 MCI (`mci.*`)

**Mission.** Switch the entire ED-IS instance into Mass-Casualty Incident mode, enabling START (Simple Triage and Rapid Treatment) and SALT (Sort, Assess, Lifesaving Interventions, Treatment/Transport) triage, surge bed grids, and shadow registration.

**Aggregate.** `MCIActivation` (incident-level) and `MCIPatient` (per-victim).

**Key rules.**
- Activation requires an attending or designated emergency manager identity scope.
- On activation, the tracking board acquires an MCI overlay; non-MCI patients remain visible but de-prioritized.
- Patients can be triaged before identity is known — `MCIPatient` accepts a tag-number identifier that later reconciles to `Patient`.
- START categories: `immediate`, `delayed`, `minor`, `expectant`, `deceased`.
- SALT categories: `immediate`, `expectant`, `delayed`, `minimal`, `dead`.
- Deactivation requires explicit closure with after-action report metadata.

### 2.5 EMS Handoff (`emshandoff.*`)

**Mission.** Receive a prehospital report from EMS while the ambulance is en route, accept the bedside handoff on arrival, and lock the handoff record into the encounter.

**Aggregate.** `EMSHandoff`.

**Key rules.**
- Prehospital report can arrive via NEMSIS v3.5 over FHIR `Encounter[prehospital]`, AsyncAPI message, or signed manual entry.
- Handoff closure requires both EMS provider attestation and receiving nurse acknowledgement.
- Handoff record carries the run number, unit ID, ETA, mechanism of injury / chief complaint, vitals trend, and field interventions.
- BYOK applies if EMS uplink is via a third-party CAD vendor.

### 2.6 Registration (`registration.*`)

**Mission.** Get a patient into the system fast — even when identity is unknown. Three modes: quick-reg (minimal fields), ambulance pre-arrival (skeleton from EMS), walk-in (full intake).

**Aggregate.** `Registration`.

**Key rules.**
- Quick-reg requires only a placeholder name (e.g., "Trauma-7-Doe"), an estimated age band, and a sex marker. Full demographics may follow.
- Pre-arrival quick-reg can be projected from the EMS handoff record without re-entry.
- Identity reconciliation is a separate workflow that merges quick-reg into the canonical `Patient` once IDs land.
- Registration cannot block triage. If registration is incomplete, triage proceeds with the quick-reg placeholder.

### 2.7 Order Entry (`orderentry.*`)

**Mission.** Rapid CPOE (Computerized Provider Order Entry) tuned for the ED tempo, with protocol-driven order sets that drop a pre-bundled cluster of orders in one action.

**Aggregate.** `OrderSet` (template) and `Order` (instance).

**Key rules.**
- Standing-order sets per protocol (e.g., Sepsis Bundle: lactate, blood cultures x2, broad-spectrum antibiotic, 30 mL/kg crystalloid).
- Verbal-order bridge: an order may be entered as "verbal, attending Dr X" with an automatic 24-hour countersign requirement; missed countersign emits `ed.order.unverified.window-exceeded`.
- Order routing publishes to downstream µservices (`pharmacy`, `lab`, `imaging`) over AsyncAPI rather than direct synchronous calls (per ADR-0145 direct gRPC + invariants).

### 2.8 Documentation (`documentation.*`)

**Mission.** Template-driven ED clinical documentation that supports rapid charting at bedside and across multiple concurrent patients.

**Aggregate.** `EDNote`.

**Key rules.**
- Note templates are versioned and pack-overlayable (US-anchor, EU-anchor, KR-pack).
- Voice-to-text ingestion is optional and uses the `intelligence` µservice with BYOK-capable LLM credentials.
- Notes can be signed-as-final, amended, or rescinded. Rescissions require attestable reason.
- Note rendering for export embeds an attestable signature chain.

### 2.9 Disposition (`disposition.*`)

**Mission.** Drive every patient to a final outcome: admit, transfer, discharge, AMA (against medical advice), or expired.

**Aggregate.** `Disposition`.

**Key rules.**
- Disposition can only be set by an attending or a delegated authorized provider.
- Admit disposition opens a downstream encounter request in the `emr` µservice and may move the patient into `Boarding` until the inpatient bed lands.
- Transfer disposition requires EMTALA-compliant transfer documentation (CoP per `feedback_compliance_pack_primitive`).
- AMA requires acknowledgement, signed AMA form, and a discharge instruction packet.
- Expired disposition triggers `ed.expired.notify` to chaplaincy / decedent-affairs workflows.

### 2.10 Boarding (`boarding.*`)

**Mission.** Track and surface admitted patients still physically held in the ED awaiting an inpatient bed — the largest contributor to ED throughput collapse.

**Aggregate.** `BoardingHold`.

**Key rules.**
- A patient enters Boarding the moment Disposition=Admit fires but no inpatient bed is yet assigned.
- Boarding clock starts at disposition time. Threshold alerts at 2h, 4h, 8h, 12h, 24h.
- Boarding patients still appear on the tracking board but are visually distinct.
- `ed.boarding.threshold` event fans out to operations dashboards.

### 2.11 LWBS (`lwbs.*`)

**Mission.** Identify patients who Left Without Being Seen (LWBS) and patients who left After Being Triaged but Before Being Seen (LBTC), and record outreach if applicable.

**Aggregate.** `LWBSRecord`.

**Key rules.**
- Auto-detected when a triaged patient has been on the board for over the LWBS threshold (default 4h) with no provider encounter.
- LWBS event opens an outreach workflow into `crm`/`contact-center` so a callback can be made.
- Quality team subscribes to `ed.lwbs.recorded` for case review.

### 2.12 Metrics (`metrics.*`)

**Mission.** Project the canonical ED KPIs continuously.

**Aggregate.** `MetricSnapshot` (rolling window).

**Key rules.**
- Door-to-doctor: time from arrival timestamp to first provider order or first provider documentation event.
- Door-to-CT: time from arrival to first CT order completion.
- Door-to-needle (stroke): triage to thrombolytic administration.
- Door-to-balloon (STEMI): arrival to PCI device deployment.
- Door-to-disposition: arrival to disposition close.
- ED length-of-stay (LOS): arrival to physical departure.
- LWBS rate: LWBS encounters / total arrivals.
- Boarding median, boarding ≥4h count, boarding ≥24h count.

Metrics are published as OpenTelemetry signals via `observability` and projected to `data-warehouse`.

### 2.13 Bed Control (`bedcontrol.*`)

**Mission.** Own the ED's bed grid: rooms, hall slots, fast-track chairs, resus bays, decon bay, peds zone, behavioral-health zone, isolation rooms.

**Aggregate.** `BedGrid` + `BedSlot`.

**Key rules.**
- Bed states: available, occupied, hold, cleaning, out-of-service.
- Bed mode flags: airborne-isolation-capable, contact-isolation-capable, behavioral-safe, peds-only, resus.
- Bed turnover targets are configurable per ED.

### 2.14 Communication (`communication.*`)

**Mission.** Multi-disciplinary message board, paging, and structured handoff communication (e.g., I-PASS) within the ED.

**Aggregate.** `CommThread`.

**Key rules.**
- Threads attach to an encounter, a bed, a protocol, or a shift.
- Messages route via the `messenger` µservice (MLS RFC 9420 E2EE per `feedback_mls_rfc_9420_e2ee_personal_messenger`).
- Shift change publishes an attestable `ed.shift.handoff` event.

### 2.15 Room Assignment (`roomassignment.*`)

**Mission.** Decide which bed a patient goes into, given their acuity, isolation needs, behavioral risk, and current grid state.

**Aggregate.** `RoomAssignmentDecision`.

**Key rules.**
- Default is rule-based: ESI-1 → first resus bay; ESI-2 → next monitored bed; ESI-3 → standard; ESI-4/5 → fast-track if available.
- AI-assisted recommender is opt-in and runs under `intelligence` µservice with `provider_credential_mode` honored.
- Charge nurse override is always permitted and recorded.

### 2.16 Trauma Registry (`traumaregistry.*`)

**Mission.** Build an ACS Trauma Quality Improvement Program (TQIP) / National Trauma Data Bank (NTDB) — compliant feed for any encounter that meets trauma registry inclusion criteria.

**Aggregate.** `TraumaRegistryRecord`.

**Key rules.**
- Inclusion criteria are configurable per facility (default: trauma alert activation OR ICD-10 injury code).
- Record includes mechanism, anatomic injury (AIS coded), ISS calculation, transport mode, prehospital times, ED times, disposition, and outcome.
- Periodic export job projects to a TQIP-conformant file; export is signed and attested via `audit-chain`.

### 2.17 Disaster Response (`disasterresponse.*`)

**Mission.** Activate the facility's Incident Command Structure (ICS / HICS), broadcast facility status, and orchestrate cross-µservice surge response.

**Aggregate.** `ICSActivation` + `FacilityStatus`.

**Key rules.**
- Facility status states: green / yellow / red / black, plus an MCI mode flag.
- Activation publishes `ed.disaster.activated` consumed by `incident-management`, `ops-dashboard-control-center`, and downstream supply chain / staffing µservices.
- Drill mode is a first-class flag; drill activations never alter production metrics but write a drill-attestable record.

---

## 3. Functional Requirements

This section enumerates the user-observable functional behaviors. Each FR is testable.

### 3.1 Triage FRs

- FR-TRG-001 — User can complete a triage in under 90 seconds for a routine ESI-3 patient given vitals already captured.
- FR-TRG-002 — Triage form blocks completion until acuity, vitals, and chief complaint are all present.
- FR-TRG-003 — A re-triage is recorded as a new TriageEncounter row keyed by sequence number, never overwriting the prior.
- FR-TRG-004 — On ESI-1, the system surfaces an immediate room recommendation and dispatches a paging event to the resus team within 1 second.
- FR-TRG-005 — Pediatric triage uses a PEWS-augmented overlay if the patient is under 18.
- FR-TRG-006 — Pain score is recorded on a 0-10 numeric or FLACC scale for non-verbal pediatric patients.
- FR-TRG-007 — Triage UI surfaces prior 6 months of ED visits with one-tap drill-in to prior chief complaints.

### 3.2 Tracking Board FRs

- FR-TB-001 — Board renders all current encounters within 1.5 s on cold load, on a 50-bed ED.
- FR-TB-002 — A patient state change reflects on every connected dashboard within 500 ms p95.
- FR-TB-003 — Board supports zone filters (Adult-Main / Adult-Fast-Track / Peds / Behavioral / Resus / Hall).
- FR-TB-004 — Board cells display: bed number, patient placeholder name, ESI badge, elapsed time, pending tasks badge, isolation badge.
- FR-TB-005 — Charge nurse may swap bed assignments by drag-and-drop; the swap is atomic.
- FR-TB-006 — Board shows a "wait list" segment for un-roomed patients.

### 3.3 Protocol FRs

- FR-PRT-001 — Trauma Alert activation requires attending or designated trauma-team-leader scope.
- FR-PRT-002 — Stroke Alert activation drops a fixed time-coded checklist with target windows.
- FR-PRT-003 — STEMI Alert publishes a cardiology paging event immediately.
- FR-PRT-004 — Sepsis Alert auto-suggests when SIRS-or-qSOFA threshold criteria are detected from vitals (alert, not auto-activate).
- FR-PRT-005 — Protocol deactivation (false-alert) requires reason text ≥10 chars.
- FR-PRT-006 — Tenant packs may add new protocols but cannot delete canonical-base protocols.

### 3.4 MCI FRs

- FR-MCI-001 — Activation requires an attending or facility-emergency-manager identity scope.
- FR-MCI-002 — On activation, the tracking board acquires a visible MCI overlay banner.
- FR-MCI-003 — START and SALT triage cards are available as the primary triage modality in MCI mode.
- FR-MCI-004 — Quick-reg in MCI mode accepts a tag number (numeric) as primary identifier with later identity reconciliation.
- FR-MCI-005 — Deactivation requires explicit closure with after-action report flag.
- FR-MCI-006 — Mock-MCI / Drill mode produces a parallel ledger but does not touch production metrics.

### 3.5 EMS Handoff FRs

- FR-EMS-001 — Prehospital report can arrive via NEMSIS-conformant FHIR Encounter or AsyncAPI message.
- FR-EMS-002 — Pre-arrival report is visible on the tracking board with an ETA countdown.
- FR-EMS-003 — On arrival, EMS-side and ED-side both acknowledge handoff in a co-signed record.
- FR-EMS-004 — Run number, unit ID, mechanism, prehospital vitals trend, field interventions are all captured.

### 3.6 Registration FRs

- FR-REG-001 — Quick-reg completes in under 30 seconds with only a placeholder name, age band, sex.
- FR-REG-002 — Identity reconciliation merges quick-reg into canonical Patient without losing the placeholder history.
- FR-REG-003 — Pre-arrival skeleton from EMS Handoff projects automatically into Registration.
- FR-REG-004 — Walk-in registration captures insurance fields, but registration never blocks triage.

### 3.7 Order Entry FRs

- FR-OE-001 — Provider can drop a pre-bundled protocol order set in a single confirmed click.
- FR-OE-002 — Verbal-order entry requires automatic 24-hour countersign tracking.
- FR-OE-003 — Drug-interaction and dose-range checks fire at order entry, sourced from `intelligence` clinical decision support.
- FR-OE-004 — Order routing fans out asynchronously to pharmacy / lab / imaging consumers.
- FR-OE-005 — Order status (acknowledged / in-progress / completed / cancelled) reflects on the tracking board within 1s of upstream event.

### 3.8 Documentation FRs

- FR-DOC-001 — Provider can start a note from a chief-complaint-driven template in one tap.
- FR-DOC-002 — Voice-to-text capture is opt-in and writes alongside structured fields.
- FR-DOC-003 — Note signing produces an attestable record routed to `audit-chain`.
- FR-DOC-004 — Amendments and rescissions are auditable and reason-required.

### 3.9 Disposition FRs

- FR-DIS-001 — Only attending or delegated authorized provider can finalize disposition.
- FR-DIS-002 — Admit disposition opens an inpatient request to the `emr` µservice.
- FR-DIS-003 — Transfer disposition enforces EMTALA-compliant documentation (sending physician note, receiving facility acceptance, mode of transport, condition stable check).
- FR-DIS-004 — AMA flow presents a structured AMA form with attestation.
- FR-DIS-005 — Expired flow triggers chaplaincy / decedent-affairs notification chain.

### 3.10 Boarding FRs

- FR-BRD-001 — Patient enters Boarding when Disposition=Admit fires with no inpatient bed yet.
- FR-BRD-002 — Boarding clock starts at disposition close.
- FR-BRD-003 — Threshold alerts fire at 2h, 4h, 8h, 12h, 24h via `ed.boarding.threshold`.
- FR-BRD-004 — Boarding patients remain on the tracking board with a distinct visual lane.

### 3.11 LWBS FRs

- FR-LWBS-001 — Triaged patient idle on the board past the configured LWBS window is flagged.
- FR-LWBS-002 — Flagging triggers an outreach record into `crm`.
- FR-LWBS-003 — Quality team consumes `ed.lwbs.recorded` for case review.

### 3.12 Metrics FRs

- FR-MET-001 — Door-to-doctor, door-to-CT, door-to-needle, door-to-balloon, door-to-disposition, LOS, LWBS rate, boarding ≥4h count are all published continuously.
- FR-MET-002 — Metrics are projected to OpenTelemetry signals.
- FR-MET-003 — Metrics are also projected to `data-warehouse` for retrospective analytics.

### 3.13 Bed Control FRs

- FR-BC-001 — Bed grid is authoritative; no other µservice may hold a parallel ED bed state.
- FR-BC-002 — Bed status changes propagate to tracking board within 500 ms.
- FR-BC-003 — Out-of-service bed reason is recorded.

### 3.14 Communication FRs

- FR-COM-001 — Threads attach to encounter, bed, protocol, or shift entity.
- FR-COM-002 — Messages are E2EE via `messenger` MLS.
- FR-COM-003 — Shift change handoff produces an attestable record.

### 3.15 Room Assignment FRs

- FR-RA-001 — Rule-based assignment is the default.
- FR-RA-002 — AI-assisted recommender is opt-in per tenant pack.
- FR-RA-003 — Charge-nurse override is always permitted and logged.

### 3.16 Trauma Registry FRs

- FR-TR-001 — Inclusion criteria are configurable.
- FR-TR-002 — Record carries mechanism, AIS, ISS, prehospital times, ED times, disposition, outcome.
- FR-TR-003 — Periodic TQIP-conformant export is signed.

### 3.17 Disaster Response FRs

- FR-DR-001 — Facility status state machine: green / yellow / red / black, with MCI flag.
- FR-DR-002 — Activation publishes downstream events for surge coordination.
- FR-DR-003 — Drill mode is first-class.

---

## 4. Non-Functional Requirements

### 4.1 Performance

- p99 board state change to dashboard delivery ≤ 500 ms.
- p95 order entry round-trip ≤ 400 ms.
- p95 triage save ≤ 600 ms.
- p99 metrics projection lag ≤ 5 s.
- Cold dashboard load for 50-bed ED ≤ 1.5 s.

### 4.2 Availability

- ED-IS targets four nines (99.99%) at the API surface inside the cell.
- Cell-tier promotion gates per ADR-0248 cellular architecture (Tier 0-4).
- Read-only degraded mode is the failure floor; the tracking board must remain readable even if writes fail.

### 4.3 Scalability

- Horizontally scalable per cell.
- A single cell supports up to 10 EDs (each up to 100 beds).
- Cross-cell isolation per ADR-0248 shuffle sharding.

### 4.4 Security & Compliance

- All PHI at rest encrypted via `cloud-kms` per HIPAA + HITRUST.
- All inter-µservice traffic over HTTP/3 + QUIC (per ADR-0253).
- Per-tenant compliance packs apply HIPAA / GDPR / SOC2 / HITRUST / ISO-27001 / PCI-DSS / EU-AI-Act / CMS-CoP-EMTALA / TJC / ACS-Trauma-Verification (per `feedback_compliance_pack_primitive`).
- Every gate evaluated through Cedar per `feedback_cedar_as_universal_gate`.
- BYOK supported and required for any tenant whose pack mandates it.

### 4.5 Tenancy

- Every row, audit, metric carries tenant context per `feedback_tenant_as_universal_scoping_primitive`.
- `oyatie` itself is a reserved-namespace tenant; the µservice's own observability flows under that reservation per `feedback_oyatie_is_a_tenant_doctrine`.

### 4.6 Observability

- Every aggregate emits an OpenTelemetry span.
- Logs structured (per `observability` µservice contract).
- Trace-id propagation across protocol activations is mandatory.

### 4.7 Disaster Tolerance

- Cells survive single-zone failure with no PHI loss.
- Disaster response mode functions even with `ontology` and `analytics` degraded.

### 4.8 Localization

- Canonical-base is global-neutral.
- KR pack (first), EU pack, US pack overlays as needed per `feedback_canonical_base_localization`.

### 4.9 Accessibility

- ED-IS UI honors WCAG 2.2 AA at minimum.
- High-contrast / large-print mode is first-class for resus environments.

### 4.10 DR Posture (ADR-0343)

- Target: RTO 1800s and RPO 120s for triage, tracking board, protocol activation, disposition, and mass-casualty mode, matching `manifest.json` `dr.rto_p99_seconds=1800` and `dr.rpo_p99_seconds=120`.
- Compliance floors: HIPAA-2024 floors at 3600s/300s with multi-region required; EU-AI-ACT-2024-HIGH-RISK floors at 1800s/300s with multi-region required; SOC2-T2 floors at 14400s/900s; ISO27001-2022 floors at 14400s/3600s; PCI-DSS-L1-v4 floors at 86400s/3600s. The effective ED target remains 1800s/120s with active-active regional cells.
- failover_runbook: `microservices/emergency/runbooks/emergency-board-failover.md`.
- multi_region_active_active: true for triage, board-read, write-capture, MCI mode, and protocol activation.
- Why: ED staff keep the board readable and can continue triage, resus, trauma, stroke, STEMI, sepsis, and MCI workflows while a failed cell recovers.

### 4.11 Capacity Model (ADR-0340)

- Per-tenant baseline: 0.45 vCPU, 1024 MiB RAM, 12 GB operational state storage, 6 Postgres connections, 8 Valkey connections, and 10 outbound HTTP connections, matching `manifest.json` `capacity_model`.
- Scaling dimension: `per_request` for board updates, triage saves, protocol activations, and metric projections.
- Cell placement class: Tier-3. Emergency is a first-party product application aligned to `pod_runtime_tier=2`, even though it handles urgent PHI and EMTALA-sensitive audit records.
- Autoscaling boundaries: min 3 pods per tenant cell, max 30 pods per tenant cell before ED-zone and tenant-cell split review.
- Why: this serves request-heavy ED board refreshes, triage mutations, bed changes, and alert acknowledgements without misclassifying the service as a substrate control plane.

### 4.12 Sustainability + Cost Attribution (ADR-0344)

- Emission envelope: every ED audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside tenant, capability, provider, cell, and compliance-pack axes.
- Provider-routing affected by carbon: no for triage, tracking board, trauma/stroke/STEMI/sepsis protocols, MCI mode, or HIPAA emergency-mode traffic; yes for retrospective ED metrics and non-urgent data-warehouse projections.
- Tenant cost transparency: `finops-portal` exposes ED cost rows by board, triage, protocol, registry feed, and metric projection, including the data-warehouse path governed by ADR-0337.
- Why: CSRD, SB-253, and SEC climate-disclosure evidence is needed for ED operations without changing routing for life-safety workflows.

### 4.13 API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet across `Oyatie-Api-Version`, `/v/YYYY-MM-DD/...` URL prefixes, and proto3 `api_version_date` fields.
- SDK semver model: major.minor.patch for ED board, EMS handoff, registry, and FHIR-facing clients.
- Support window: last 3 public versions supported for at least 180 days.
- Per-tenant pinning: yes for EMS partners, hospital-command centers, and registry feeds.
- Internal-mesh exemption: yes; direct gRPC handoffs to EMR, healthcare-integration, audit-chain, and observability preserve ADR-0145.

---

## 5. Domain Model (Aggregates + Events)

### 5.1 Aggregates

| Aggregate | Root key | Context |
|-----------|----------|---------|
| TriageEncounter | encounter_id + sequence | Triage |
| BoardLane | zone_id | TrackingBoard |
| BoardCell | bed_id | TrackingBoard |
| ProtocolActivation | activation_id | Protocol |
| MCIActivation | incident_id | MCI |
| MCIPatient | tag_number | MCI |
| EMSHandoff | handoff_id | EMSHandoff |
| Registration | registration_id | Registration |
| OrderSet | order_set_id | OrderEntry |
| Order | order_id | OrderEntry |
| EDNote | note_id | Documentation |
| Disposition | disposition_id | Disposition |
| BoardingHold | hold_id | Boarding |
| LWBSRecord | lwbs_id | LWBS |
| MetricSnapshot | snapshot_id | Metrics |
| BedGrid | grid_id | BedControl |
| BedSlot | bed_id | BedControl |
| CommThread | thread_id | Communication |
| RoomAssignmentDecision | decision_id | RoomAssignment |
| TraumaRegistryRecord | trauma_id | TraumaRegistry |
| ICSActivation | ics_id | DisasterResponse |
| FacilityStatus | status_id | DisasterResponse |

### 5.2 Domain Events (AsyncAPI prefix `ed.*`)

| Event | Trigger |
|-------|---------|
| `ed.patient.registered` | Registration close (any mode) |
| `ed.triage.completed` | TriageEncounter finalized |
| `ed.triage.reassessed` | Re-triage finalized |
| `ed.protocol.activated` | ProtocolActivation persisted |
| `ed.protocol.deactivated` | ProtocolActivation closed false-alert |
| `ed.protocol.window.breached` | Bundle timer past target |
| `ed.mci.activated` | MCIActivation start |
| `ed.mci.deactivated` | MCIActivation closure |
| `ed.ems.report.received` | EMS prehospital report ingested |
| `ed.ems.handoff.completed` | Bedside handoff acknowledged |
| `ed.order.placed` | Order accepted |
| `ed.order.signed` | Verbal order countersigned |
| `ed.order.unverified.window-exceeded` | Verbal-order countersign window past |
| `ed.bed.assigned` | RoomAssignmentDecision persisted |
| `ed.bed.reassigned` | Charge-nurse override of room assignment |
| `ed.bed.status.changed` | BedSlot state transition |
| `ed.disposition.set` | Disposition finalized |
| `ed.boarding.threshold` | BoardingHold past threshold |
| `ed.lwbs.recorded` | LWBS auto-flag |
| `ed.metrics.snapshot` | MetricSnapshot publication |
| `ed.shift.handoff` | Provider/nurse shift change |
| `ed.disaster.activated` | ICSActivation start |
| `ed.disaster.deactivated` | ICSActivation closure |
| `ed.trauma.registry.exported` | TQIP export job complete |
| `ed.expired.notify` | Disposition=Expired |

---

## 6. Public APIs

### 6.1 REST (FHIR-anchored)

- POST `/Encounter` — create ED encounter (class=EMER per FHIR).
- POST `/EpisodeOfCare` — open ED episode.
- POST `/Observation` (acuity LOINC code) — triage acuity record.
- POST `/CarePlan` — protocol checklist.
- POST `/Patient` — quick-reg / walk-in / ambulance.
- POST `/Encounter/{id}/$ed-triage` — operation that persists a TriageEncounter and projects FHIR.
- POST `/Encounter/{id}/$ed-disposition` — operation that finalizes disposition.
- POST `/Encounter/{id}/$ed-protocol-activate` — operation that activates a protocol.
- POST `/MCI/$activate` — start MCI mode.
- POST `/MCI/$deactivate` — end MCI mode.
- GET `/ed/board` — current tracking board snapshot (SSE/WebSocket subscribable).
- GET `/ed/metrics` — current metric snapshot.
- POST `/ed/handoff` — EMS prehospital report ingest.

Full schema in `contracts/openapi.yaml`.

### 6.2 AsyncAPI

Every event listed in 5.2 with a normative JSON-schema payload. Full schema in `contracts/asyncapi.yaml`.

### 6.3 gRPC

`emergency.proto` defines stream-friendly RPCs for the tracking board (`BoardSubscribe`), metric subscription (`MetricsSubscribe`), and MCI mode coordination (`MciActivate`, `MciTriageWrite`). Full schema in `contracts/proto/emergency.proto`.

---

## 7. SLO Catalog

ED-IS publishes ≥10 OpenSLO objects in `slos/`. Highlights:

- door-to-doctor.openslo.yaml — median ≤ 30 min, p95 ≤ 60 min.
- door-to-ct.openslo.yaml — Stroke Alert door-to-CT ≤ 25 min.
- door-to-needle.openslo.yaml — Stroke door-to-thrombolytic ≤ 60 min.
- door-to-balloon.openslo.yaml — STEMI door-to-PCI ≤ 90 min.
- door-to-disposition.openslo.yaml — median ≤ 240 min.
- ed-throughput.openslo.yaml — overall LOS p95 ≤ 6 h for treat-and-release.
- triage-latency.openslo.yaml — door-to-triage-complete median ≤ 10 min, p95 ≤ 20 min.
- boarding-burden.openslo.yaml — ≥4h-boarders count daily target.
- lwbs-rate.openslo.yaml — LWBS ≤ 2% (target) / ≤ 4% (ceiling).
- tracking-board-staleness.openslo.yaml — p99 board update lag ≤ 500 ms.
- protocol-bundle-compliance.openslo.yaml — Sepsis 3-hour bundle adherence ≥ 90%.
- api-availability.openslo.yaml — 99.99% rolling 30-day.

---

## 8. Policy Surface (Cedar)

`policies/` holds Cedar policies (per `feedback_cedar_as_universal_gate`). Highlights:

- `charge-nurse-can-reassign-bed.cedar` — only `role.chargeNurse` or `role.attending` may move a patient between beds.
- `registration-can-quick-reg.cedar` — quick-reg requires either an ED registration clerk or an emergency manager.
- `trauma-alert-bypass-rules.cedar` — trauma alert bypasses normal triage queueing while preserving downstream eventing.
- `ed-only-disposition.cedar` — only providers with active ED privileges and an open Encounter[ED] may set disposition.
- `mci-mode-activation.cedar` — MCI activation gated to attending or facility-emergency-manager.
- `ams-disposition.cedar` — AMA disposition requires a signed AMA form artifact + attestation.
- `verbal-order-bridge.cedar` — verbal-order entries require provider claim + 24h countersign window enforcement.
- `byok-credential-mode.cedar` — Cedar gate enforces `provider_credential_mode` per tenant pack.

---

## 9. Deployment & IaC

ED-IS ships an OpenTofu (NOT Terraform) module per deployment context (per `feedback_zero_handroll_opentofu_only_2026_05_20`):

- `iac/aws-guest/`
- `iac/oci-guest/`
- `iac/on-prem/`
- `iac/colo/`
- `iac/oyatie-cloud/`

All modules are signed and tenant onboarding completes with a single `tofu apply`.

---

## 10. Compliance Packs

ED-IS supports the canonical pack set with ED-specific overlays:

- **HIPAA** — mandatory for any US deployment. Audit logging, breach notification hooks, BAA-style controls.
- **GDPR** — mandatory for EU. Data subject rights, lawful basis, processor records.
- **SOC2** — mandatory commercial baseline.
- **HITRUST-CSF** — overlays HIPAA with prescriptive controls.
- **ISO-27001** — baseline ISMS attestation.
- **PCI-DSS** — applies because patient self-pay flows may carry card data via `payments`.
- **EU-AI-Act** — applies wherever AI-assisted room assignment or AI documentation is enabled (per `feedback_build_ahead_of_certification`).
- **CMS-CoP-EMTALA** — mandatory for any US ED.
- **TJC-Standards** — Joint Commission ED-specific (PC, EP, EM chapters).
- **ACS-Trauma-Verification** — required for any verified trauma center; the trauma registry feed is conformance-gated.

---

## 11. Cross-Microservice Integration

| Counterpart µservice | Direction | Purpose |
|----------------------|-----------|---------|
| identity | inbound | Provider, nurse, charge nurse, EMS user identity + role scopes |
| audit-chain | outbound | Every protocol, disposition, MCI activation attested |
| consent-graph | inbound | Patient consent for data flows |
| healthcare-integration | bidirectional | Inbound HL7 v2 / FHIR from existing third-party EMR; outbound HL7 ADT / ORM / ORU |
| emr | bidirectional | Open inpatient encounter on admit; read prior history; project ED encounter |
| messenger | outbound | Multi-disciplinary communication, MLS E2EE |
| ontology | outbound | Project ED entities into the canonical ontology |
| observability | outbound | OpenTelemetry + structured logs |
| tenancy | inbound | Tenant context + pack flags |
| compliance | inbound | Pack catalog + per-tenant pack resolution |
| governance | inbound | Policy versions + Cedar binding |
| calendar | inbound | Provider on-call schedules |
| workflow-engine | bidirectional | Protocol checklist execution backbone |
| contact-center | outbound | LWBS outreach |
| notes | outbound | Note storage |
| forms | inbound | AMA form, refusal forms, etc. |
| analytics | outbound | Metric projection |
| data-pipeline | outbound | Event firehose |
| data-warehouse | outbound | Long-term analytics |
| incident-management | outbound | Disaster activations |
| ops-dashboard-control-center | outbound | Boarding + facility status |
| pharmacy (downstream) | outbound | Medication orders |
| crm | outbound | LWBS outreach |
| payments | outbound | Self-pay flows |
| cloud-billing-tax | outbound | Encounter billing handoff |

---

## 12. Implementation Plan Sequencing

The IPs (`implementation-plans/IP-001..IP-010`) sequence is:

1. IP-001 — Triage engine core (ESI 5-level + reassessment).
2. IP-002 — Tracking board projection + SSE/WebSocket fanout.
3. IP-003 — Protocol activation (Trauma / Stroke / STEMI / Sepsis) + bundle timer.
4. IP-004 — MCI mode + START/SALT triage + tag-number reconciliation.
5. IP-005 — EMS handoff (prehospital report ingest + bedside handoff).
6. IP-006 — Registration (quick-reg + pre-arrival + walk-in) + identity reconciliation.
7. IP-007 — Order entry + verbal-order countersign + protocol order sets.
8. IP-008 — Disposition + boarding + LWBS + EMTALA transfer.
9. IP-009 — Metrics projection (door-to-X) + trauma registry feed.
10. IP-010 — Disaster response + drill mode + cell-tier promotion gates.

Each IP has its own brief in `implementation-plans/`.

---

## 13. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Tracking board staleness causes patient missed by team | OpenSLO `tracking-board-staleness` + auto-paging on lag |
| Protocol bundle misses regulatory window silently | OpenSLO `protocol-bundle-compliance` + `ed.protocol.window.breached` event |
| Boarding crisis (admitted patients pile in ED) under-surfaced | First-class Boarding context + `boarding-burden` SLO + ops dashboard fanout |
| MCI mode accidentally activated in production | Drill flag required separately; Cedar `mci-mode-activation` policy gates |
| Verbal-order countersign drift | `ed.order.unverified.window-exceeded` event + dashboard |
| Trauma registry export incompleteness blocks ACS verification | Conformance check before export + signed artifact + `audit-chain` |
| EMS handoff data lost on race | Idempotency keys + retry-safe ingest + co-signed close |
| Quick-reg placeholder collides with real patient | Reconciliation workflow + dedupe by demographic + manual review queue |
| ESI ratings drift across raters | Calibration dashboards + quarterly review |
| AMA disposition without proper attestation | Cedar `ams-disposition.cedar` gate |
| Pediatric patient triaged on adult overlay | Auto-overlay switch on DOB; if DOB unknown, age-band-driven |
| LWBS undercount because patient leaves before triage | LWBS variant `LBR` (Left Before Registered) tracked separately |

---

## 14. Out-of-scope

Per ADR-0131 + ADR-0132 (single-concern, flat, no suites):

- ED-IS does NOT own inpatient orders → those live in `emr`.
- ED-IS does NOT own pharmacy fulfillment → `pharmacy` µservice (separate).
- ED-IS does NOT own lab fulfillment → `lab` µservice (separate).
- ED-IS does NOT own imaging fulfillment → `imaging` µservice (separate).
- ED-IS does NOT own billing → `cloud-billing-tax`.
- ED-IS does NOT own scheduling → `calendar`.
- ED-IS does NOT own OR/ICU → distinct µservices (not in this PRD).

Each downstream is a peer; ED-IS publishes events and consumes responses through the canonical inter-µservice contract.

---

## 15. Acceptance Criteria

ED-IS reaches Tier-1 (cell-tier-1) promotion when:

- All 17 bounded contexts have aggregate + event + handler implementations in `src/`.
- All ≥10 OpenSLOs published in `slos/` and observable in `observability`.
- All Cedar policies in `policies/` are evaluated on the gate path.
- OpenAPI / AsyncAPI / proto contracts are versioned + signed.
- OpenTofu modules in all 6 deployment contexts apply cleanly in CI.
- HIPAA + GDPR + SOC2 + EMTALA packs pass conformance.
- Trauma registry export passes a TQIP-conformant sample.
- p99 board state delivery ≤ 500 ms in load test.

---

## 16. Open Questions

- Should the system natively support ED-anchored ultrasound (POCUS) documentation, or defer to `imaging` µservice? (Lean: defer; ED captures the order, `imaging` owns the study.)
- Pediatric severity overlay choice — PEWS vs. SREM vs. local — per pack vs. canonical?
- AI-recommender room assignment governance: who must sign off on a rule change? (Lean: attending plus QI committee, via `governance`.)
- MCI tag-number reconciliation workflow ownership — `emergency` or `identity`? (Lean: `emergency` owns the tag and the reconcile workflow, `identity` resolves once known.)
- Boarding policy threshold (when to start CMS measure) — pack-driven (US vs. EU) vs. canonical default 2h?

These will be resolved in ADR-MS-001..ADR-MS-NN as the IPs land.

---

## 17. Glossary

- **ESI** — Emergency Severity Index 5-level triage scale (1 emergent → 5 minor).
- **CPOE** — Computerized Provider Order Entry.
- **MCI** — Mass-Casualty Incident.
- **START** — Simple Triage and Rapid Treatment.
- **SALT** — Sort, Assess, Lifesaving Interventions, Treatment/Transport.
- **EMTALA** — Emergency Medical Treatment and Labor Act (US federal).
- **LWBS** — Left Without Being Seen.
- **LBTC** — Left Before Treatment Complete.
- **AMA** — Against Medical Advice.
- **PCI** — Percutaneous Coronary Intervention (STEMI definitive treatment).
- **ICS / HICS** — Incident Command Structure / Hospital ICS.
- **NEMSIS** — National EMS Information System.
- **TQIP / NTDB** — Trauma Quality Improvement Program / National Trauma Data Bank.
- **AIS / ISS** — Abbreviated Injury Scale / Injury Severity Score.
- **PEWS** — Pediatric Early Warning Score.
- **qSOFA / SIRS** — Sepsis screening criteria.
- **I-PASS** — Structured handoff communication framework.

---

## 18. Authority Trail

- ADR-0332 (in flight) — Emergency Department Information System µservice.
- ADR-0328 — Substance bar as canonical sequence.
- ADR-0131 — Per-µservice flat layout.
- ADR-0132 — Suite dissolution / no bundle.
- ADR-0251 — Compliance pack primitive.
- ADR-0248 — Amazon-shape cellular architecture.
- ADR-0064 — Canonical-base neutrality (locale packs).
- ADR-0145 — Direct gRPC + 3 invariants for inter-µservice communication.
- ADR-0253 — HTTP/3 + QUIC default protocol.
- `feedback_microservice_ownership_coherence_2026_05_20`
- `feedback_docs_substance_not_scaffold_2026_05_20`
- `feedback_compliance_pack_primitive`
- `feedback_oci_always_free_maximization_2026_05_20`
- `feedback_rust_strict_only_no_python_2026_05_20`
- `feedback_os_support_matrix_2026_05_20`
- `feedback_byok_everywhere_credentials`
- `feedback_intelligence_two_layer_substrate`
- `feedback_cedar_as_universal_gate`
- `feedback_oyatie_is_a_tenant_doctrine`
- `feedback_mls_rfc_9420_e2ee_personal_messenger`

---

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

<!--
COMPLETION REPORT
=================
microservice: emergency
wave: 15M-D
authored_artifacts:
  - PRD.md (this file, 17 bounded contexts, ≥800 lines, substantive)
  - ARCHITECTURE.md (≥600 lines)
  - README.md (≥300 lines)
  - manifest.json
  - contracts/openapi.yaml
  - contracts/asyncapi.yaml
  - contracts/proto/emergency.proto
  - slos/ (12 OpenSLO files; spec mandates ≥10)
  - policies/ (8 Cedar policies)
  - iac/ for 6 deployment_contexts (aws-guest, oci-guest, oci-always-free, on-prem, colo, oyatie-cloud)
  - decisions/ADR-MS-001-triage-engine.md
  - decisions/ADR-MS-002-mass-casualty-mode.md
  - implementation-plans/IP-001..IP-010
  - competitor-parity-matrix.md (≥100 capabilities, T-System + Wellsoft + FirstNet + secondary union)
  - supported-oses.json
counterpart_baseline:
  top_three: ["T-System (Hyland)", "Wellsoft EDIS", "Cerner FirstNet (Oracle Health)"]
  secondary: ["Epic ASAP", "Picis CareSuite", "Medhost EDIS", "TeleTracking ED Tracker"]
canonical_authority:
  - ADR-0332 (in flight)
  - ADR-0328
  - ADR-0131
  - ADR-0132
  - ADR-0251
ownership_model: single-owner-end-to-end
ownership_role: emergency-medicine-platform-engineer
status: scaffold-complete
notes:
  - Single-owner end-to-end per µservice ownership memory.
  - Substantive per docs-of-substance memory; no scaffold-stamping.
  - Rust-strict for service logic; OpenTofu HCL for IaC; Cedar for policy; OpenAPI/AsyncAPI/proto for contracts; OpenSLO for SLOs (per Rust-strict + zero-handroll-OpenTofu + Cedar + OS matrix memories).
  - OCI Always Free deployment_context included per OCI-Always-Free-maximization memory.
  - All six deployment contexts covered per multi-context + provider-agnostic memory.
-->
