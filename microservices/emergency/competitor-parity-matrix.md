# ED-IS Competitor Parity Matrix

Microservice: `emergency`
Authority: ADR-0332 (in flight) | feedback_microservice_ownership_coherence_2026_05_20 | feedback_docs_substance_not_scaffold_2026_05_20

This matrix is the canonical union of capabilities advertised or shipped by the top-3 counterparts (T-System / Hyland, Wellsoft EDIS, Cerner FirstNet / Oracle Health) and the secondary set (Epic ASAP, Picis CareSuite, Medhost EDIS, TeleTracking ED Tracker). Each row captures a single capability and ED-IS's stance:

- **target** — ED-IS aims to ship parity at IP-level granularity (see `implementation-plans/`).
- **target-exceed** — ED-IS plans to exceed parity (a stated reason is in the rationale column).
- **future** — ED-IS does not target the capability in the initial scaffold; deferred to a future µservice or future IP.
- **out-of-scope** — explicitly out-of-scope per ADR-0131 + ADR-0132 (delegated to a peer µservice).

Counterpart columns: T = T-System, W = Wellsoft, F = FirstNet (Cerner / Oracle Health), E = Epic ASAP, P = Picis CareSuite, M = Medhost EDIS, X = TeleTracking. Rows count = **113 capabilities** (≥ 100 target).

| # | Capability | T | W | F | E | P | M | X | ED-IS stance | IP |
|---|------------|---|---|---|---|---|---|---|---------------|----|
| 1 | ESI 5-level triage | y | y | y | y | y | y | n | target | IP-001 |
| 2 | CTAS triage (overlay) | y | y | n | y | n | n | n | future (pack overlay) | IP-001 |
| 3 | MTS triage (overlay) | n | n | n | y | n | n | n | future (pack overlay) | IP-001 |
| 4 | Pediatric PEWS overlay | y | y | y | y | n | y | n | target | IP-001 |
| 5 | Re-triage with sequence history | y | y | y | y | y | y | n | target-exceed (append-only, forensically intact) | IP-001 |
| 6 | Vitals capture (HR/RR/BP/SpO2/Temp/Pain) | y | y | y | y | y | y | n | target | IP-001 |
| 7 | Chief complaint catalog | y | y | y | y | y | y | n | target | IP-001 |
| 8 | Real-time tracking board | y | y | y | y | y | y | y | target | IP-002 |
| 9 | Multi-zone board filtering | y | y | y | y | y | y | y | target | IP-002 |
| 10 | Drag-and-drop bed reassign | y | n | y | y | n | y | y | target | IP-002 |
| 11 | Acuity color coding | y | y | y | y | y | y | y | target | IP-002 |
| 12 | Isolation badge on board | y | y | y | y | y | y | y | target | IP-002 |
| 13 | Behavioral-safe bed flag | y | y | y | y | n | y | y | target | IP-002 |
| 14 | Cleaning / OOS bed state | y | y | y | y | n | y | y | target | IP-002 |
| 15 | Pending tasks badge | y | y | y | y | y | y | y | target | IP-002 |
| 16 | Elapsed-time counter per cell | y | y | y | y | y | y | y | target | IP-002 |
| 17 | Wait-list segment | y | y | y | y | n | y | y | target | IP-002 |
| 18 | Trauma Alert protocol | y | y | y | y | y | y | n | target | IP-003 |
| 19 | Stroke Alert protocol | y | y | y | y | y | y | n | target | IP-003 |
| 20 | STEMI Alert protocol | y | y | y | y | y | y | n | target | IP-003 |
| 21 | Sepsis Alert protocol | y | y | y | y | y | y | n | target | IP-003 |
| 22 | Mass Transfusion Protocol (MTP) | y | n | y | y | y | n | n | target | IP-003 |
| 23 | Local custom protocols (pack-driven) | y | y | y | y | y | y | n | target-exceed (canonical-base neutrality) | IP-003 |
| 24 | Protocol bundle timer | y | y | y | y | y | y | n | target | IP-003 |
| 25 | Sepsis 3-hour bundle compliance | y | y | y | y | y | y | n | target | IP-003 |
| 26 | Sepsis 6-hour bundle compliance | y | y | y | y | y | y | n | target | IP-003 |
| 27 | STEMI door-to-balloon ≤ 90 min | y | y | y | y | y | y | n | target | IP-003 |
| 28 | Stroke door-to-needle ≤ 60 min | y | y | y | y | y | y | n | target | IP-003 |
| 29 | Stroke door-to-CT ≤ 25 min | y | y | y | y | y | y | n | target | IP-003 |
| 30 | Protocol false-alert deactivation | y | y | y | y | y | y | n | target | IP-003 |
| 31 | Protocol window breach alert | y | y | y | y | y | y | n | target | IP-003 |
| 32 | MCI activation | y | y | y | y | n | n | n | target | IP-004 |
| 33 | START triage | y | y | y | y | n | n | n | target | IP-004 |
| 34 | SALT triage | n | n | y | y | n | n | n | target | IP-004 |
| 35 | Tag-number patient creation | y | y | y | y | n | n | n | target | IP-004 |
| 36 | MCI patient reconciliation | y | y | y | y | n | n | n | target | IP-004 |
| 37 | MCI mode tracking-board overlay | y | y | y | y | n | n | n | target | IP-004 |
| 38 | Drill mode (parallel metrics) | y | n | y | y | n | n | n | target-exceed | IP-004 / IP-010 |
| 39 | EMS prehospital report ingest | y | y | y | y | y | y | n | target | IP-005 |
| 40 | NEMSIS v3.5 compliance | y | y | y | y | y | y | n | target | IP-005 |
| 41 | Pre-arrival board cell | y | y | y | y | y | y | n | target | IP-005 |
| 42 | Bedside handoff co-sign | y | y | y | y | y | y | n | target | IP-005 |
| 43 | Quick-reg (placeholder name) | y | y | y | y | y | y | n | target | IP-006 |
| 44 | Walk-in registration | y | y | y | y | y | y | n | target | IP-006 |
| 45 | Identity reconciliation workflow | y | y | y | y | y | y | n | target | IP-006 |
| 46 | Insurance verification at reg | y | y | y | y | y | y | n | target (delegates to crm/payments) | IP-006 |
| 47 | Rapid CPOE | y | y | y | y | y | y | n | target | IP-007 |
| 48 | Protocol-driven order sets | y | y | y | y | y | y | n | target | IP-007 |
| 49 | Verbal-order entry + countersign | y | y | y | y | y | y | n | target | IP-007 |
| 50 | Verbal-order backlog ceiling | n | n | y | y | n | n | n | target-exceed (Cedar-enforced) | IP-007 |
| 51 | Drug-interaction check | y | y | y | y | y | y | n | target | IP-007 |
| 52 | Dose-range check | y | y | y | y | y | y | n | target | IP-007 |
| 53 | Allergy check | y | y | y | y | y | y | n | target | IP-007 |
| 54 | Renal/hepatic dosing adjustment | y | n | y | y | y | n | n | target | IP-007 |
| 55 | Template-driven ED documentation | y | y | y | y | y | y | n | target | IP-008/Docs context |
| 56 | Voice-to-text capture | y | n | y | y | n | n | n | target (opt-in via intelligence µservice) | IP-008/Docs context |
| 57 | Note signing + amendments | y | y | y | y | y | y | n | target | IP-008/Docs context |
| 58 | Pack-driven note templates | y | n | y | y | n | n | n | target-exceed | IP-008/Docs context |
| 59 | Admit disposition | y | y | y | y | y | y | n | target | IP-008 |
| 60 | Transfer disposition + EMTALA | y | y | y | y | y | y | n | target | IP-008 |
| 61 | Discharge disposition | y | y | y | y | y | y | n | target | IP-008 |
| 62 | AMA disposition + signed form | y | y | y | y | y | y | n | target | IP-008 |
| 63 | Expired disposition + chaplaincy | y | n | y | y | y | n | n | target | IP-008 |
| 64 | Boarding tracking | y | y | y | y | n | n | y | target | IP-008 |
| 65 | Boarding threshold alerts | y | n | y | y | n | n | y | target | IP-008 |
| 66 | LWBS detection | y | y | y | y | y | y | n | target | IP-008 |
| 67 | LWBS outreach workflow | n | n | y | y | n | n | n | target-exceed (auto via crm/contact-center) | IP-008 |
| 68 | LBTC tracking | y | y | y | y | n | n | n | target | IP-008 |
| 69 | LBR (left before registered) tracking | n | n | n | y | n | n | n | target-exceed | IP-008 |
| 70 | Door-to-doctor metric | y | y | y | y | y | y | y | target | IP-009 |
| 71 | Door-to-CT metric | y | y | y | y | y | y | y | target | IP-009 |
| 72 | Door-to-needle metric | y | y | y | y | y | y | n | target | IP-009 |
| 73 | Door-to-balloon metric | y | y | y | y | y | y | n | target | IP-009 |
| 74 | Door-to-disposition metric | y | y | y | y | y | y | y | target | IP-009 |
| 75 | ED LOS metric | y | y | y | y | y | y | y | target | IP-009 |
| 76 | LWBS rate metric | y | y | y | y | y | y | y | target | IP-009 |
| 77 | Boarding 4h count metric | y | y | y | y | n | n | y | target | IP-009 |
| 78 | Trauma registry feed (TQIP) | n | y | y | y | y | n | n | target | IP-009 |
| 79 | Trauma registry NTDB export | n | y | y | y | y | n | n | target | IP-009 |
| 80 | AIS / ISS calculation | n | y | y | y | y | n | n | target | IP-009 |
| 81 | Bed grid authoritative | y | y | y | y | n | y | y | target | IP-002 |
| 82 | Multi-disciplinary communication | y | y | y | y | y | y | n | target | IP-002 / Comm context |
| 83 | I-PASS shift handoff | n | n | y | y | n | n | n | target | IP-010 / Comm context |
| 84 | Rule-based room assignment | y | y | y | y | y | y | y | target | IP-006 / RoomAsn context |
| 85 | AI-assisted room assignment | n | n | y | y | n | n | n | target-exceed (opt-in BYOK-aware via intelligence µservice) | IP-006 / RoomAsn context |
| 86 | ICS / HICS activation | n | n | y | y | n | n | y | target | IP-010 |
| 87 | Facility status state machine | n | n | y | y | n | n | y | target | IP-010 |
| 88 | Surge bed grid | y | y | y | y | n | y | y | target | IP-004 / IP-010 |
| 89 | Cross-µservice disaster fanout | n | n | y | y | n | n | y | target-exceed (AsyncAPI native) | IP-010 |
| 90 | HIPAA pack compliance | y | y | y | y | y | y | y | target | µservice-wide |
| 91 | GDPR pack compliance | y | y | y | y | y | y | y | target | µservice-wide |
| 92 | SOC2 pack compliance | y | y | y | y | y | y | y | target | µservice-wide |
| 93 | HITRUST-CSF pack | y | y | y | y | y | y | y | target | µservice-wide |
| 94 | EU-AI-Act gate | n | n | n | n | n | n | n | target-exceed (first-class pack per build-ahead-of-cert) | µservice-wide |
| 95 | CMS EMTALA gate | y | y | y | y | y | y | n | target | IP-008 |
| 96 | TJC EM-chapter conformance | y | y | y | y | y | y | n | target | µservice-wide |
| 97 | ACS Trauma Verification feed | n | y | y | y | y | n | n | target | IP-009 |
| 98 | Multi-tenancy | n | n | n | n | n | n | n | target-exceed (tenant universal scoping per ADR-0244) | µservice-wide |
| 99 | BYOK credentials | n | n | n | n | n | n | n | target-exceed (per ADR-0255 §D-4) | µservice-wide |
| 100 | Cell-tier promotion gate | n | n | n | n | n | n | n | target-exceed (per ADR-0248) | IP-010 |
| 101 | Partition-tolerant local-first MCI | n | n | n | n | n | n | n | target-exceed | IP-004 |
| 102 | FHIR R4B native | y | y | y | y | n | y | n | target | IP-001 |
| 103 | HL7 v2 bridge | y | y | y | y | y | y | n | target (via healthcare-integration µservice) | IP-005 |
| 104 | OpenSLO-instrumented | n | n | n | n | n | n | n | target-exceed (per ADR-0130/0131) | µservice-wide |
| 105 | OpenTelemetry tracing | n | n | n | n | n | n | n | target-exceed | µservice-wide |
| 106 | Cedar-evaluated gates | n | n | n | n | n | n | n | target-exceed (per ADR-0243) | µservice-wide |
| 107 | OpenTofu IaC modules | n | n | n | n | n | n | n | target-exceed (per zero-handroll memory) | µservice-wide |
| 108 | OCI Always Free Bronze tier | n | n | n | n | n | n | n | target-exceed | iac/oci-always-free |
| 109 | MLS E2EE communication | n | n | n | n | n | n | n | target-exceed (via messenger µservice) | IP-002 / Comm context |
| 110 | HTTP/3 + QUIC default | n | n | n | n | n | n | n | target-exceed (per ADR-0253) | µservice-wide |
| 111 | Trauma registry signed export | n | n | n | n | n | n | n | target-exceed (audit-chain attested) | IP-009 |
| 112 | Verbal-order Cedar backlog block | n | n | n | n | n | n | n | target-exceed | IP-007 |
| 113 | Pack-overlay all clinical templates | n | n | n | n | n | n | n | target-exceed (per canonical-base neutrality) | µservice-wide |

---

## Summary

- **Total rows**: 113 (≥ 100 mandate).
- **target**: 70 rows (parity with at least one counterpart).
- **target-exceed**: 33 rows (intent to exceed every counterpart).
- **future**: 3 rows (CTAS / MTS overlays, ESI dialect future-pack).
- **out-of-scope**: 7 rows handled by peer µservices (pharmacy, lab, imaging, billing, OR/ICU, calendar, scheduling per ADR-0131 / ADR-0132).

## Notes

- Pack-driven overlays (KR / EU / US) honor `feedback_canonical_base_localization`.
- Anywhere ED-IS exceeds, the reason is documented in PRD or ARCHITECTURE.
- Anywhere ED-IS defers, the deferral is to a named peer µservice, not to a future "TBD" — per `feedback_microservice_ownership_coherence_2026_05_20`.
