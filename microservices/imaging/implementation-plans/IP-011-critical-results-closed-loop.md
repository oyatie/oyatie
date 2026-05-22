# IP-011 — Critical-results closed-loop communication

`scope: oya-imaging-critical-result-app + workflow-engine integration`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0105 + ADR-0244`

## Objective

Notify ordering clinician within p99 < 30s of critical-finding signing. Cascade escalation through covering clinician → charge nurse → on-call attending → patient safety officer. Confirmation required at each step within timer.

## Scope

1. Trigger on structured-report critical-finding code tagging.
2. workflow-engine state machine.
3. Channel adapters: comms-push, comms-sms, comms-voice, in-portal, fallback fax.
4. Escalation timer per criticality (I: 5min; II: 15min; III: 1h).
5. Audit-chain emission per step.
6. Closed-loop confirmation requires explicit user action.

## Acceptance criteria

- p99 < 30s to ordering clinician notification (FR-RAD-010).
- Audit-chain entry per step.
- Confirmation gating: timer expiry without confirmation cascades.
- Five-step escalation cascade verified end-to-end.

## Dependencies

- IP-009.
- `workflow-engine` µservice.
- `comms-*` µservices.

## Risks

- Phone-tag (most-common legacy failure); mitigate with multi-channel + structured-confirmation.

## Estimated effort

- 6–10 person-weeks.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/imaging/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-011-critical-results-closed-loop.md:9` - Notify ordering clinician within p99 < 30s of critical-finding signing. Cascade escalation through covering clinician → charge nurse → on-call attending → patient safe...; `microservices/imaging/implementation-plans/IP-011-critical-results-closed-loop.md:22` - - p99 < 30s to ordering clinician notification (FR-RAD-010)..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-011-critical-results-closed-loop.md:17` - 5. Audit-chain emission per step..
