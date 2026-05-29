# IP-001 — Triage Engine Core

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-MS-001 | ADR-0332 (in flight) | ADR-0131
Sequence: 1 / 10

---

## Scope

Stand up the canonical ESI v4 triage engine with re-triage, pediatric PEWS overlay, FHIR Observation projection, and Cedar policy hookup. This IP delivers the smallest unit of functioning ED-IS: a triage-capable µservice that can ingest an arriving patient, capture vitals + chief complaint + acuity, persist a `TriageEncounter`, project the Observation to `emr`, and publish `ed.triage.completed`.

## Out-of-scope

- Room-recommendation rule engine (deferred to IP-002).
- Order entry (IP-007).
- Tracking-board fanout (IP-002).
- Protocol activation (IP-003).

## Deliverables

- `src/crates/emergency-triage/` — domain crate with `TriageEncounter` aggregate.
- `src/crates/emergency-domain/` — shared aggregate types.
- `src/crates/emergency-application/` — `complete_triage`, `reassess_triage` use-cases.
- `src/crates/emergency-infrastructure/` — Postgres repo + FHIR Observation projection adapter.
- `src/crates/emergency-api-rest/` — `POST /Encounter/{id}/$ed-triage`.
- `src/crates/emergency-api-async/` — publish `ed.triage.completed`, `ed.triage.reassessed`.
- Cedar policy hookup via `emergency-policy`.
- 12 OpenSLO objects pre-populated in `slos/`; this IP wires up `triage-latency.openslo.yaml`.
- Tests:
  - Unit tests on ESI rules (≥ 40 cases covering each ESI level).
  - Integration test on save + projection + event publish.
  - Contract test against `contracts/openapi.yaml`.

## Sequencing

1. Domain crate with `TriageEncounter` + `Vitals` + `Acuity` types.
2. ESI v4 decision tree as a pure Rust function (testable in isolation).
3. PEWS overlay function with age-band branching.
4. Postgres migration for `triage_encounter` table.
5. Repo + projection wiring.
6. REST endpoint with input validation.
7. AsyncAPI publisher.
8. Cedar policy hookup at the REST gate.
9. OpenSLO instrumented at the application layer.
10. Tests + golden contract assertions.

## Acceptance Criteria

- `cargo test -p emergency-triage` passes 100%.
- `cargo test -p emergency-application -- triage` passes.
- Contract test against `openapi.yaml` operation `$ed-triage` passes.
- `triage-latency.openslo.yaml` is observable via the `observability` µservice.
- p95 save wall-clock ≤ 600 ms in load test with 100 concurrent triages.
- `ed.triage.completed` event payload validates against `asyncapi.yaml`.

## Risks

- ESI v4 algorithm rater drift — mitigation: 40+ golden test cases pin the canonical behavior.
- LOINC code drift — mitigation: pull terminology snapshot from `healthcare-integration`.
- Postgres write contention under burst — mitigation: connection pool + batched advisory locks.

## Migration Appendix

For a tenant migrating from T-System / Wellsoft / FirstNet:

1. Export prior triage records via the source vendor's HL7 ADT / ORM feed.
2. Land into `healthcare-integration` µservice.
3. Project into `emergency-triage` via a one-shot importer adapter (vendor-specific mapping).
4. Re-key on `(encounter_id, sequence=0)` for the historical record.
5. Verify on a sample by replaying ESI re-evaluation.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/emergency/implementation-plans/IP-001-triage-engine.md:34` - - Contract test against `contracts/openapi.yaml`.; `microservices/emergency/implementation-plans/IP-001-triage-engine.md:53` - - Contract test against `openapi.yaml` operation `$ed-triage` passes..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-001-triage-engine.md:30` - - 12 OpenSLO objects pre-populated in `slos/`; this IP wires up `triage-latency.openslo.yaml`.; `microservices/emergency/implementation-plans/IP-001-triage-engine.md:46` - 9. OpenSLO instrumented at the application layer..
