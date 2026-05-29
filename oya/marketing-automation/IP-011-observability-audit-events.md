---
doc_class: ImplementationPlan
ip_id: IP-011-observability-audit-events
microservice: marketing-automation
bounded_contexts: [journey, segment, suppression, attribution, deliverability, webhook-subscription]
related_adrs: [ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-observability
tenant_class_aware: true
---

# IP-011: Observability and Audit Events

## A. Problem

The service has `AuditPort`, `EventPort`, SLO YAMLs, and dashboards, but the stamped IP never named the audit evidence Marketing Automation must produce. HubSpot Audit Log, Marketo activity logs, and Mailchimp campaign reports all expose operator-visible history. Oyatie needs stronger evidence: every journey admission, suppression denial, attribution rollup, deliverability pause, and marketplace audience-license hold must seal audit-chain events and expose SLO burn.

## B. Approach

Bind `src/usecase/mod.rs` `AuditPort::append()` to a per-event audit catalog and make dashboards consume the same event names as AsyncAPI and OpenSLO files. The observability surface uses `dashboards/local-audit-completeness.json`, `dashboards/local-slo-burn.json`, `dashboards/slo-and-error-budget.json`, and SLO files under `slos/` for latency, replay, audit emission lag, and policy-decision latency.

## C. Deliverables

| Artifact | Change |
|---|---|
| `src/usecase/mod.rs` | Ensure every `MarketingAutomationEvent` is appended before event publication. |
| `slos/audit-emission-lag.openslo.yaml` | Track accepted command to audit-chain seal latency. |
| `slos/policy-decision-latency.openslo.yaml` | Track Cedar evaluation latency for campaign/journey actions. |
| `dashboards/local-audit-completeness.json` | Add per-event completeness panels and missing-audit alert. |
| `incident-response.md` | Add audit-chain emission degradation response. |

## D. Implementation

1. Create the audit event catalog from `MarketingAutomationEvent`: journey launch accepted, suppression applied, segment sync requested, attribution rollup queued, consent export queued, marketplace audience license held.
2. Add dimensions required by PRD and manifest: `tenant_id`, `principal_id`, `tenant_class`, `data_residency_pack`, `capability`, `request_id`, and `idempotency_key_hash`.
3. Add SLO thresholds: audit emission p95 under 250 ms for command acceptance, policy decision p99 under the existing `policy-decision-latency` target.
4. Wire dashboard panels to the exact event names from `src/usecase/mod.rs` and `contracts/asyncapi-v1.yaml`.
5. Add alert path for audit append failure: command must fail before event publication, because audit-less action acceptance is a false claim.
6. Add integration test with a fake `AuditPort` that records append-before-publish ordering.
7. Document forensic query examples for marketer-facing actions in `incident-response.md`.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app audit`
- `cargo run -p oya-dev-cli -- gate validate audit-emission --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate slo-catalog --microservice marketing-automation`
- Manual evidence: no event in `MarketingAutomationEvent` lacks a dashboard or SLO reference.

## F. Evidence

- Local source: `src/usecase/mod.rs` `AuditPort` and `event_for()`.
- Local dashboards: `dashboards/local-audit-completeness.json`, `dashboards/slo-and-error-budget.json`.
- Local SLOs: `slos/audit-emission-lag.openslo.yaml`, `slos/policy-decision-latency.openslo.yaml`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Audit Log-like operator history becomes audit-chain sealed. |
| Adobe Marketo Engage | Activity-log style events receive SLO and forensic dimensions. |
| Mailchimp | Campaign and audience actions expose evidence beyond report summaries. |

## H. Local Traceability

- Audit port: `AuditPort::append`.
- Event port: `EventPort::publish`.
- Ordering invariant: append before publish.
- Event source: `event_for()`.
- Event source: `event_type()`.
- SLO file: `slos/audit-emission-lag.openslo.yaml`.
- SLO file: `slos/policy-decision-latency.openslo.yaml`.
- Dashboard: `dashboards/local-audit-completeness.json`.
- Dashboard: `dashboards/local-slo-burn.json`.
- Dashboard: `dashboards/slo-and-error-budget.json`.
- Incident doc: `incident-response.md`.
- Audit dimension: `tenant_id`.
- Audit dimension: `principal_id`.
- Audit dimension: `tenant_class`.
- Audit dimension: `data_residency_pack`.
- Audit dimension: `capability`.
- Audit dimension: `request_id`.
- Failure state: action accepted without audit event is a blocker.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-011-observability-audit-events.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-011-observability-audit-events.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-011-observability-audit-events.md` matched [`attribution`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-011-observability-audit-events.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
