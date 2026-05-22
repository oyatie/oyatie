---
id: ADR-EMR-MS-001
status: Accepted
deciders: axis-emr, council-clinical, council-architecture
date: 2026-05-21
microservice: emr
purpose: Lock the 15 bounded-context decomposition of the EMR µservice as the canonical boundary used for crate naming, persistence schema, REST routing, and audit emission.
related:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0251
  - ADR-EMR-MS-002
  - ADR-EMR-MS-003
---

# ADR-EMR-MS-001: EMR bounded-context decomposition

## Status

Accepted — 2026-05-21 (Wave 15M-B authoring).

## Context

EMR — the clinical Record-Of-Truth — covers a broad swath of clinical-workflow surface area. Without a deliberate bounded-context decomposition, the µservice quickly degrades into a god-object with monolithic data shapes, monolithic Cedar policy, and monolithic audit emission.

The Epic-Cerner-athena reference codebases consistently slice clinical-chart functionality into the following families:

- **Demographic + identity** (Patient, MPI)
- **Encounter lifecycle** (Admit-Discharge-Transfer)
- **Clinical lists** (Problem, Medication, Allergy, Immunization, Family History)
- **Time-series** (Vital signs)
- **Authored content** (Notes / Documentation)
- **Order entry** (CPOE)
- **Results** (Lab + Imaging review)
- **Care team** (Assignment)
- **Clinical content governance** (Order sets, documentation templates, smart-phrases)
- **Billing capture** (CPT/ICD/HCPCS)
- **Patient-facing** (Education, Portal)

oyatie EMR adopts this convention with explicit naming + scope decisions. ADR-0132 (no suites) forbids bundling these BCs into product wrappers; ADR-0131 (per-microservice flat layout) requires a per-BC crate trio.

## Decision

EMR is decomposed into the following **15 bounded contexts**, each owning persistence, port surface, REST/AsyncAPI/gRPC adapters, Cedar policy fragments, and audit emission patterns. Names are kebab-case BNF v4.1 conformant. Crate-stem pattern: `oya-emr-<bc>-<layer>` per ADR-0105's 13-layer enum.

| # | BC | Persistence | Retention | Regulatory anchor |
|---|---|---|---|---|
| 1 | `patient` | Postgres+Citus, tenant-shard | 7y post-discharge (10y KR-PIPA pack) | HIPAA §164.514 |
| 2 | `encounter` | Postgres+temporal | 7y post-close | CMS §482.24 |
| 3 | `problem` | Postgres relational | Patient-lifetime | MU Stage 3 |
| 4 | `medication` | Postgres relational + saga | Patient-lifetime | DEA Schedule rules |
| 5 | `allergy` | Postgres relational | Patient-lifetime | TJC NPSG.03.06.01 |
| 6 | `vital` | TimescaleDB hypertable | 7y | TJC documentation |
| 7 | `note` | Postgres + WORM signed | 7y from signature | CMS E/M 2024 |
| 8 | `order` | Postgres + saga | 7y | TJC NPSG.02.03.01 |
| 9 | `result` | Postgres + LOINC link | 7y | CLIA / CAP / JCAHO |
| 10 | `care-team` | Postgres relational | Per encounter | HIPAA §164.502 |
| 11 | `order-set` | Postgres versioned catalog | Catalog-lifetime | ASHP order-set |
| 12 | `documentation` | Postgres versioned catalog | Catalog-lifetime | CMS E/M 2024 |
| 13 | `billing-code` | Postgres relational | 7y from claim | AMA CPT |
| 14 | `patient-education` | Postgres + content registry | Catalog-lifetime | Patient Bill of Rights |
| 15 | `portal-session` | Valkey + Postgres event log | 90d session log; 7y audit | HIPAA §164.524 |

## Rejected alternatives

- **Single monolithic BC ("chart").** Rejected — monolithic chart bloats persistence, monolithic Cedar policy, monolithic audit emission. Industry leaders all decompose at this granularity.
- **Folding `result` into `order`.** Rejected — orders and results have independent retention (results may outlive an order's saga-completion), independent provenance, independent consent gradient.
- **Folding `note` into `documentation`.** Rejected — `note` is the live authoring artifact for a specific patient × encounter; `documentation` is the catalog of templates + smart-phrases.
- **Folding `care-team` into `encounter`.** Rejected — care-team relationships span encounters (chronic disease management, primary-care continuity).
- **Folding `patient-education` into `documentation`.** Rejected — `patient-education` is patient-facing content with multi-language requirements; `documentation` is clinician-facing templates.
- **Folding `portal-session` into `cloud-iam`.** Rejected — portal-session embodies patient-specific authorization (proxy grants, 42 CFR Part 2 consent, segmented record access) that is EMR-scoped, not generic-auth-scoped.
- **Adding `anesthesia-record` as the 16th BC.** Deferred — anesthesia is sufficiently distinct workflow that it may earn its own µservice; ADR pending council-clinical input.

## Consequences

- 15 BCs × (kernel, domain, usecase, application, api, events, grpc) = ~105 Rust crates for the core hexagonal stack, plus adapters (postgres × 14, timescale × 1, valkey × 1) + peer-client adapters + workers + the top-level `oya-emr-app` composition root.
- Audit emission segments by BC (e.g., `emr.patient.viewed.v1`, `emr.order.entered.v1`, ...).
- Cedar policy organizes per-BC at `microservices/emr/policies/` plus the cross-cutting `audit-everything.cedar` + `hipaa-deny-default.cedar`.
- Each BC has its own retention rule (the `manifest.json#wave_15m_b_substance_floor` substance bar verifies that retention per BC is declared).
- Cross-BC dependencies inside EMR follow inward-only flow: e.g., `medication` references `patient` (identity) but `patient` does not import `medication` (avoid kernel-cycle).

## Verification

- `oya gate validate per-microservice-layout --microservice emr` exits 0.
- BC enumeration in `manifest.json#bounded_contexts` matches this ADR.
- Crate-naming validator confirms `oya-emr-<bc>-<layer>` for every crate where `<bc>` is in the enumeration.

## References

- ADR-0105 13-layer canonical enum.
- ADR-0131 per-microservice flat layout.
- ADR-0132 no suites + single-concern.
- Epic Hyperspace bounded-context heuristics (KLAS).
- Cerner Millennium product-line decomposition.
- athenaClinicals + athenaCollector bundling pattern.
- HL7 FHIR R5 ResourceTypes (Patient, Encounter, Condition, MedicationRequest, AllergyIntolerance, Observation, DocumentReference, ServiceRequest, DiagnosticReport, CarePlan, CareTeam, Composition).
