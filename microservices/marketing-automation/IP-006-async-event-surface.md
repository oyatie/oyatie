---
doc_class: ImplementationPlan
ip_id: IP-006-async-event-surface
microservice: marketing-automation
bounded_contexts: [journey, segment, suppression, attribution, consent-audience, deliverability]
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0314, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-006: Async Event Surface

## A. Problem

The stamped version said "binds segment-sync" six times without naming the event contract that makes Marketing Automation operational. This IP closes the actual gap: `contracts/asyncapi-v1.yaml` currently exposes a single generic `marketing-automation.events.v1` action-accepted message, while `src/adapter/asyncapi.rs` names concrete channels for journey launch, suppression, segment sync, consent changes, and deliverability signals. HubSpot Workflow, Marketo Smart Campaign, and Mailchimp Customer Journey parity requires first-class event semantics, not one catch-all payload.

## B. Approach

Promote the Rust channel registry in `src/adapter/asyncapi.rs` into a versioned AsyncAPI contract. The contract must publish accepted command events from `MarketingAutomationEvent` and subscribe to consent and deliverability inputs that gate downstream sends. The event envelope carries `tenant_id`, `principal_id`, `request_id`, `idempotency_key`, `tenant_class`, `data_residency_pack`, `audit_event_class`, and HLC `event_time`. The transport profile remains HTTP/3-first with ECH/PQC references from ADR-0253-amendment.

## C. Deliverables

| Artifact | Change |
|---|---|
| `contracts/asyncapi-v1.yaml` | Split the generic `ActionAccepted` into `JourneyLaunchAccepted`, `SuppressionApplied`, `SegmentSyncRequested`, `AttributionRollupQueued`, `ConsentExportQueued`, `MarketplaceAudienceLicenseHeld`, `ConsentChanged`, and `DeliverabilitySignal`. |
| `src/adapter/asyncapi.rs` | Keep `MarketingAutomationAsyncApiHandler::channels()` as the canonical in-code channel list and add fixture validation for every AsyncAPI message name. |
| `src/usecase/mod.rs` | Ensure `event_for()` emits the same event type names as the AsyncAPI message registry. |
| `tests/integration.rs` | Replace ignored `asyncapi_journey_event_fixture_round_trips` with fixture coverage for one publish channel and one subscribe channel. |
| `dashboards/local-audit-completeness.json` | Add per-channel audit completeness panels keyed by message name. |

## D. Implementation

1. Expand `contracts/asyncapi-v1.yaml` channels to match every channel currently returned by `MarketingAutomationAsyncApiHandler::channels()`.
2. Add subscribed inputs for `marketing-automation.consent.changed.v1` and `marketing-automation.deliverability.signal.v1` with payload fields used by `MarketingAutomationCommand::EnforceSuppression`.
3. Bind each published event to the `MarketingAutomationEvent` enum in `src/domain/mod.rs` so contract drift is caught at compile-time fixture generation.
4. Update `src/usecase/mod.rs` `event_type()` strings to use the contract names or add a mapping test if wire names stay kebab-case.
5. Add test fixtures under the existing `tests/integration.rs` scaffold that validate `validate_channels()` and JSON examples from the AsyncAPI file.
6. Emit audit-chain dimensions for `tenant_class`, `source`, and `data_residency_pack`, because PRD §C.8 makes demo_trial to paid conversion non-destructive.
7. Document replay semantics for event consumers in `backfill-replay.md` without inventing a separate worker path.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app asyncapi`
- `cargo test -p oya-marketing-automation-campaign-journey-app adapter_registry_contains_three_contract_surfaces`
- `cargo run -p oya-dev-cli -- gate validate openapi-contract-binding --microservice marketing-automation --contract asyncapi`
- Manual evidence: no channel in `src/adapter/asyncapi.rs` is absent from `contracts/asyncapi-v1.yaml`.

## F. Evidence

- Local source: `src/adapter/asyncapi.rs` channel registry.
- Local source: `src/usecase/mod.rs` `MarketingAutomationEvent` publication path.
- Local contract: `contracts/asyncapi-v1.yaml`.
- Doctrine: ADR-0324 forbids stamped IP bodies; ADR-0328 elevates HubSpot/Marketo/Mailchimp Big-8 substance.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Workflow enrollment, email tracking, list membership, and subscription changes become named events instead of a generic action-accepted record. |
| Adobe Marketo Engage | Smart Campaign and Bulk Activity style event streams map to `SegmentSyncRequested` and `JourneyLaunchAccepted`. |
| Mailchimp | Customer Journey and audience changes gain tenant-scoped webhook-like events with audit-chain proof. |

## H. Local Traceability

- Command source: `MarketingAutomationCommand::LaunchJourney`.
- Command source: `MarketingAutomationCommand::EnforceSuppression`.
- Command source: `MarketingAutomationCommand::SyncSegment`.
- Command source: `MarketingAutomationCommand::RollupAttribution`.
- Command source: `MarketingAutomationCommand::ExportConsent`.
- Command source: `MarketingAutomationCommand::LicenseMarketplaceAudience`.
- Published event: `JourneyLaunchAccepted`.
- Published event: `SuppressionApplied`.
- Published event: `SegmentSyncRequested`.
- Published event: `AttributionRollupQueued`.
- Published event: `ConsentExportQueued`.
- Published event: `MarketplaceAudienceLicenseHeld`.
- Subscribed event: `ConsentChanged`.
- Subscribed event: `DeliverabilitySignal`.
- Test seam: `asyncapi_journey_event_fixture_round_trips`.
- Dashboard seam: `local-audit-completeness`.
- Runbook seam: `local-webhook-provider-degradation`.
- Failure state: event published without prior audit append is a blocker.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`, `openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-006-async-event-surface.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-006-async-event-surface.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
