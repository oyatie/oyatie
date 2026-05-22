---
doc_class: ImplementationPlan
ip_id: IP-007-grpc-internal-surface
microservice: marketing-automation
bounded_contexts: [journey, segment, suppression, attribution]
related_adrs: [ADR-0105, ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-007: gRPC Internal Surface

## A. Problem

Marketing Automation has a Rust gRPC adapter (`src/adapter/grpc.rs`) and a proto file (`contracts/marketing-automation-v1.proto`), but they disagree. The Rust adapter exposes `LaunchJourney`, `SyncSegment`, `EnforceSuppression`, and `ReadJourneyState` on `CampaignJourneyService`; the proto exposes one generic `InvokeAction` method on `MarketingAutomationService`. That mismatch blocks internal workflow-engine and worker callers from relying on a typed surface.

## B. Approach

Replace the generic proto action with a typed internal control-plane surface that mirrors `MarketingAutomationCommand`. gRPC is internal only: north-south tenants use REST, event consumers use AsyncAPI, and workflow-engine / worker code uses proto. Messages must carry `tenant_id`, `principal_id`, `request_id`, `idempotency_key`, and bounded context identifiers (`CampaignJourneyId`, `SegmentId`, `ConsentLedgerRef`) compatible with `src/domain/mod.rs`.

## C. Deliverables

| Artifact | Change |
|---|---|
| `contracts/marketing-automation-v1.proto` | Define `CampaignJourneyService` with `LaunchJourney`, `SyncSegment`, `EnforceSuppression`, `RollupAttribution`, `ExportConsent`, `LicenseMarketplaceAudience`, and `ReadJourneyState`. |
| `src/adapter/grpc.rs` | Align `GrpcMethod` registry with the proto service and add missing methods currently present in `MarketingAutomationCommand`. |
| `src/domain/mod.rs` | Reuse `CampaignJourneyId`, `SegmentId`, and `ConsentLedgerRef` wire fields without adding duplicate identifier types. |
| `tests/integration.rs` | Turn the ignored proto round-trip test into a fixture test for `LaunchJourneyRequest` and `CommandReceipt`. |

## D. Implementation

1. Rename proto service to `CampaignJourneyService` or update `src/adapter/grpc.rs`; do not keep two service names.
2. Add request messages matching each command variant in `MarketingAutomationCommand`.
3. Add `CommandReceipt` with `accepted`, `tenant_id`, `capability`, and `audit_event_type` to match `src/usecase/mod.rs`.
4. Add `JourneyStateView` for `ReadJourneyState`, using `JourneyState` values from `src/domain/mod.rs`.
5. Make `MarketingAutomationGrpcHandler::methods()` enumerate every proto method.
6. Add a test that fails if proto method names and Rust registry method names diverge.
7. Keep gRPC transport internal and mTLS-bound; public OAuth/private-app SDK callers stay on OpenAPI.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app grpc`
- `cargo test -p oya-marketing-automation-campaign-journey-app public_surface_names_required_handlers`
- `buf lint microservices/marketing-automation/contracts/marketing-automation-v1.proto` or the repo-equivalent proto lint lane.
- Contract review confirms workflow-engine can call journey launch without stringly `action_id`.

## F. Evidence

- Local source: `src/adapter/grpc.rs` method registry.
- Local source: `src/domain/mod.rs` command and journey state types.
- Local contract: `contracts/marketing-automation-v1.proto`.
- Local test: `tests/integration.rs` ignored proto fixture placeholder.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Workflow APIs are explicit operations, not generic invoke strings. |
| Adobe Marketo Engage | Smart Campaign and Lead Database calls map to typed service methods. |
| Mailchimp | Customer Journey internals can be invoked by workers while tenant-facing APIs remain REST/webhook shaped. |

## H. Local Traceability

- Proto source: `contracts/marketing-automation-v1.proto`.
- Rust registry: `MarketingAutomationGrpcHandler::methods()`.
- Service target: `CampaignJourneyService`.
- Method target: `LaunchJourney`.
- Method target: `SyncSegment`.
- Method target: `EnforceSuppression`.
- Method target: `RollupAttribution`.
- Method target: `ExportConsent`.
- Method target: `LicenseMarketplaceAudience`.
- Method target: `ReadJourneyState`.
- Domain identifier: `CampaignJourneyId`.
- Domain identifier: `SegmentId`.
- Domain identifier: `ConsentLedgerRef`.
- Receipt target: `CommandReceipt`.
- State target: `JourneyState`.
- Test seam: `grpc_launch_journey_fixture_round_trips`.
- Consumer: `workflow-engine`.
- Failure state: generic `InvokeAction` remains the anti-pattern to remove.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-007-grpc-internal-surface.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-007-grpc-internal-surface.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
