# IP-003 — Smart-alarm engine

**Status**: drafted
**ADR binding**: ADR-MS-002
**Bounded contexts**: AlarmManagement + SmartAlarm
**Owner**: axis-clinical-realtime
**Estimated effort**: 4 dev-weeks

## Slice 1: Rule DSL parser

- Define grammar per ADR-MS-002 §3.
- Author Rust parser; round-trip tests.

## Slice 2: Validity + persistence primitives

- `lead_confidence` gate.
- N-sample persistence buffer per parameter.

## Slice 3: Compound condition combinator

- Multi-parameter AND/OR combinators.
- Hysteresis on edge transitions.

## Slice 4: Patient-specific + diurnal + trend-gating

- Per-patient threshold overrides.
- Diurnal night-time relaxation.
- Trend-derived re-classification on slow-creep breaches.

## Slice 5: Dedup window

- Rolling 5-min dedup; severity escalation overrides dedup.

## Slice 6: Cedar suppression integration

- Bind to `policies/alarm-suppression-requires-justification.cedar`.
- Fail-CLOSED on Cedar timeout (per ADR-0332).
- Suppression-ledger Postgres-16.

## Slice 7: Escalation chain

- Bedside → charge → on-call → code-blue per severity ladder.
- Configurable per tenant.

## Slice 8: Audit emit

- Every fire/ack/suppress/clear → audit-chain hash + chain.

## Acceptance criteria

- Smart-alarm rule eval p99 ≤ 5 ms per sample (per SLO).
- MIMIC-IV replay: alarm-fire count reduced ≥ 40% vs. dumb-threshold baseline.
- 100% sensitivity for life-threatening events on MIMIC-IV labeled subset.
- Cedar fail-CLOSED verified by chaos test.

## Dependencies

- IP-001 streaming substrate
- ADR-MS-002 accepted ✅
- audit-chain µservice ready
- policy-engine µservice ready

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-003-smart-alarm-engine.md:51` - - Smart-alarm rule eval p99 ≤ 5 ms per sample (per SLO)..
