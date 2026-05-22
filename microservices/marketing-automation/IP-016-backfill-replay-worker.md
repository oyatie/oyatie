---
doc_class: ImplementationPlan
ip_id: IP-016-backfill-replay-worker
microservice: marketing-automation
bounded_contexts: [segment, journey, consent-audience, attribution, email-tracking, behavioral-profile]
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-sre-reliability
tenant_class_aware: true
---

# IP-016: Backfill Replay Worker

## A. Problem

Marketing data arrives late from migrations, webhook retries, CRM revenue events, product telemetry, and consent revocations. The stamped IP did not name replay inputs or deterministic outcomes. The actual gap is a worker that can rebuild segment membership, journey decisions, suppression proofs, and attribution rollups without violating consent or pack residency.

## B. Approach

Use `backfill-replay.md` as the operating contract and bind it to `MarketingAutomationCommand` variants plus AsyncAPI event cursors. Replay is deterministic over input event cursor, ontology snapshot, policy bundle version, and HLC window. Replay never bypasses suppression or consent; it recomputes them.

## C. Deliverables

| Artifact | Change |
|---|---|
| `backfill-replay.md` | Add replay modes for segment rebuild, journey decision replay, consent projection, attribution rerun, and tracking import. |
| `src/usecase/mod.rs` | Add worker-oriented command receipt fields for replay id and source cursor when implementation lands. |
| `contracts/asyncapi-v1.yaml` | Add replay request/result events or document replay over existing event stream. |
| `runbooks/local-segment-rebuild-stall.md` | Tie stalled replay to cursor lag and ontology snapshot mismatch. |
| `slos/replay-freshness.openslo.yaml` | Track replay completion freshness by tenant class and data size. |

## D. Implementation

1. Define replay job key: `tenant_id`, `replay_kind`, `source_cursor`, `ontology_snapshot_id`, `policy_bundle_version`, and `requested_by`.
2. Implement idempotent job admission with `IdempotencyKey` from `src/domain/mod.rs`.
3. Recompute segment deltas before journey replay so downstream launch decisions see fresh membership.
4. Re-evaluate consent and suppression for historical sends; never copy old allow decisions blindly.
5. Re-run attribution using explicit model version and CRM revenue event cursor.
6. Emit replay audit events for queued, started, completed, drift-detected, and failed states.
7. Add operator recovery for poison events: quarantine row, skip only with approval, and preserve skipped event id.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app replay`
- `cargo run -p oya-dev-cli -- gate validate replay-freshness --microservice marketing-automation`
- Manual evidence: replay output includes input cursor, policy bundle, ontology snapshot, and deterministic drift flag.

## F. Evidence

- Local docs: `backfill-replay.md`.
- Local runbooks: `runbooks/local-segment-rebuild-stall.md`, `runbooks/local-attribution-rollup-gap.md`.
- Local SLO: `slos/replay-freshness.openslo.yaml`.
- Local source: `src/domain/mod.rs` idempotency and command identifiers.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | List/workflow reprocessing gains deterministic replay evidence. |
| Adobe Marketo Engage | Smart Campaign and activity backfills can be rerun with explicit model versions. |
| Mailchimp | Audience imports and journey history can be reconciled without weakening suppression. |

## H. Local Traceability

- Contract doc: `backfill-replay.md`.
- Domain key: `tenant_id`.
- Domain key: `replay_kind`.
- Domain key: `source_cursor`.
- Domain key: `ontology_snapshot_id`.
- Domain key: `policy_bundle_version`.
- Domain key: `requested_by`.
- Identifier: `IdempotencyKey`.
- Replay mode: segment rebuild.
- Replay mode: journey decision replay.
- Replay mode: consent projection.
- Replay mode: attribution rerun.
- Replay mode: tracking import.
- Runbook: `local-segment-rebuild-stall.md`.
- Runbook: `local-attribution-rollup-gap.md`.
- SLO: `slos/replay-freshness.openslo.yaml`.
- Failure state: replay copies old consent decision without re-evaluation.
- Failure state: skipped poison event lacks approval.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-016-backfill-replay-worker.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-016-backfill-replay-worker.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-016-backfill-replay-worker.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-016-backfill-replay-worker.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
