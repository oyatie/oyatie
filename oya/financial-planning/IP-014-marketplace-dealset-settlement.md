---
doc_class: IP
ip_id: IP-014
microservice: financial-planning
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0294
  - ADR-0296
  - ADR-0297
  - ADR-0314
  - ADR-0321
journey_ref: J-CFO-FP-DEALSET-SETTLEMENT
tenant_class: paid_core
status: draft
date: 2026-05-20
owner_team: finance-planning-platform
---

# IP-014 Financial Planning marketplace-dealset-settlement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-014-marketplace-dealset-settlement.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- marketplace-dealset-settlement-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- marketplace-dealset-settlement-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- marketplace-dealset-settlement-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- marketplace-dealset-settlement-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- marketplace-dealset-settlement-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- marketplace-dealset-settlement-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- marketplace-dealset-settlement-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- marketplace-dealset-settlement-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- marketplace-dealset-settlement-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- marketplace-dealset-settlement-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- marketplace-dealset-settlement-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-014 settles marketplace DealSets for financial-planning templates, model packs, connectors, and benchmark content.
- Planning teams buy accelerators such as workforce planning, sales forecasting, board reporting, and consolidation templates.
- The settlement layer must distinguish free benchmark migration helpers from paid vendor-compatible model packs.
- Anaplan, Workday Adaptive Planning, Oracle EPM Cloud, OneStream, Vena, Pigment, Planful, IBM Planning Analytics, Board, and Jedox compatibility packs can carry royalties, entitlements, and usage meters.
- Settlement is triggered by install, activation, execution, export, and renewal events.
- Financial-planning never becomes the marketplace ledger of record; it emits metered settlement facts to marketplace and billing.
- Cedar ensures a tenant can only activate DealSets it purchased or is trialing.
- Audit-chain receives settlement evidence because model-pack economics can affect customer contracts.
- Pack activation must not bypass IP-012 edge checks or IP-011 audit events.
- The core invariant is that planning behavior and commercial settlement use the same DealSet id.

## Data Model Deltas
```sql
CREATE TYPE fp_dealset_event_kind AS ENUM ('install','activate','execute','export','renew','revoke');

CREATE TABLE fp_marketplace_dealset_binding (
  binding_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  dealset_id UUID NOT NULL,
  planning_model_id UUID,
  source_vendor TEXT NOT NULL,
  capability_slug TEXT NOT NULL,
  entitlement_ref TEXT NOT NULL,
  settlement_currency CHAR(3) NOT NULL,
  active_from TIMESTAMPTZ NOT NULL,
  active_until TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, dealset_id, planning_model_id)
);

CREATE TABLE fp_dealset_settlement_event (
  settlement_event_id UUID PRIMARY KEY,
  binding_id UUID NOT NULL REFERENCES fp_marketplace_dealset_binding(binding_id),
  event_kind fp_dealset_event_kind NOT NULL,
  usage_units NUMERIC(18,6) NOT NULL DEFAULT 0,
  unit_kind TEXT NOT NULL,
  amount_minor BIGINT NOT NULL DEFAULT 0,
  adr0263_class_name TEXT NOT NULL DEFAULT 'ADR0263_EXPORT_ATTESTATION',
  emitted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```rust
pub enum DealSetEventKind {
    Install,
    Activate,
    Execute,
    Export,
    Renew,
    Revoke,
}

pub struct FinancialPlanningDealSetBinding {
    pub binding_id: Uuid,
    pub tenant_id: Uuid,
    pub dealset_id: Uuid,
    pub planning_model_id: Option<Uuid>,
    pub source_vendor: PlanningVendor,
    pub capability_slug: String,
    pub entitlement_ref: String,
    pub settlement_currency: CurrencyCode,
}
```

## API Endpoints
- REST `POST /v1/financial-planning/dealsets/{dealset_id}/activate`
```json
{
  "planning_model_id": "fp-model-sales-forecast-fy27",
  "source_vendor": "planful",
  "capability_slug": "rolling-forecast-driver-pack",
  "entitlement_ref": "mkt-entitlement-78421",
  "settlement_currency": "USD"
}
```
- REST `POST /v1/financial-planning/dealsets/{dealset_id}/usage-events` records execute/export units.
- REST `GET /v1/financial-planning/dealsets/bindings/{binding_id}` returns entitlement and active window.
- gRPC `FinancialPlanningDealSet.Activate(ActivateDealSetRequest) returns (DealSetBinding)`.
- gRPC `FinancialPlanningDealSet.RecordUsage(RecordDealSetUsageRequest) returns (SettlementEvent)`.
- AsyncAPI topic `financial-planning.dealset.settlement.v1`.
- AsyncAPI payload includes `dealset_id`, `binding_id`, `usage_units`, `unit_kind`, `amount_minor`, and `adr0263_class_name`.

## Cedar Policy Hooks
```cedar
permit(
  principal,
  action in [
    Oyatie::Action::"FinancialPlanningDealSetActivate",
    Oyatie::Action::"FinancialPlanningDealSetExecute",
    Oyatie::Action::"FinancialPlanningDealSetExport"
  ],
  resource in Oyatie::Resource::"MarketplaceDealSet",
  context
) when {
  principal.tenant_id == resource.tenant_id &&
  context.entitlement.status in ["active", "trial"] &&
  context.dealset_id == resource.dealset_id &&
  context.planning_model_tenant_id == principal.tenant_id &&
  context.settlement_meter_ready == true
};
```

## Ontology Projection
- Anaplan `AppHubApp.id` -> Oyatie `dealset_id`.
- Anaplan `ModelTemplate.workspaceId` -> Oyatie `planning_model_id`.
- Workday Adaptive `SolutionPackage.code` -> Oyatie `capability_slug`.
- Oracle EPM Cloud `MarketplaceListing.sku` -> Oyatie `entitlement_ref`.
- OneStream `SolutionExchangePackage.packageId` -> Oyatie `dealset_id`.
- Vena `TemplateMarketplaceItem.templateId` -> Oyatie `dealset_id`.
- Pigment `TemplateLibraryEntry.id` -> Oyatie `capability_slug`.
- Planful `SolutionHubPackage.id` -> Oyatie `dealset_id`.
- IBM Planning Analytics `AcceleratorAsset.id` -> Oyatie `dealset_id`.
- Board `MarketplaceCapsule.id` -> Oyatie `capability_slug`.
- Jedox `MarketplaceModel.id` -> Oyatie `dealset_id`.

## Workflow Steps
- Node `resolve_entitlement`: asks marketplace for tenant DealSet status.
- Node `bind_planning_model`: links DealSet to model, vendor baseline, and capability slug.
- Branch `entitlement_missing`: deny activation and emit policy event.
- Branch `trial_allowed`: activate with trial expiry and limited export usage.
- Node `activate_pack`: records binding and emits install or activate settlement event.
- Node `execute_pack`: meters template execution, recalculation, or connector transform.
- Node `export_pack_result`: meters board packet, workbook, or external report export.
- Branch `usage_overage`: send billing handoff and keep execution allowed only if entitlement permits.
- Node `publish_settlement`: sends event to marketplace and billing.
- Node `audit_settlement`: emits ADR-0263 export attestation for commercial traceability.

## Audit Events
- `financial_planning.dealset.activated` uses `ADR0263_EXPORT_ATTESTATION`.
- `financial_planning.dealset.execution_metered` uses `ADR0263_MUTATION_EVIDENCE`.
- `financial_planning.dealset.export_metered` uses `ADR0263_EXPORT_ATTESTATION`.
- `financial_planning.dealset.entitlement_denied` uses `ADR0263_POLICY_DECISION`.
- `financial_planning.dealset.usage_overage` uses `ADR0263_POLICY_DECISION`.
- `financial_planning.dealset.revoked` uses `ADR0263_REPLAY_CHECKPOINT`.

## SLO Targets
- p50 entitlement check latency: 20 ms.
- p95 entitlement check latency: 95 ms.
- p99 entitlement check latency: 220 ms.
- Throughput: 5,000 usage events per second per regional cell.
- Availability: 99.99 percent for activation and metering.
- Marketplace settlement publish p95: 500 ms.
- Duplicate settlement rate: below 1 per 10 million usage events.

## Failure Modes + Recovery
- Marketplace entitlement API unavailable: allow existing active bindings, deny new activations, queue verification.
- Duplicate usage event: idempotency key suppresses double billing and emits duplicate observation.
- Currency mismatch: reject activation until marketplace entitlement and tenant billing currency align.
- DealSet revoked while executing: finish current atomic execution, block exports, and mark binding inactive.
- Settlement publish fails: persist event locally, retry with exponential backoff, and block renewal closeout until drained.
- Vendor template id collision: namespace by source vendor and marketplace publisher id.

## Migration Notes
- Anaplan App Hub templates migrate as DealSets with model, module, and list compatibility declarations.
- Workday Adaptive Planning solution packages map to capability slugs and process tracker templates.
- Oracle EPM Cloud marketplace listings map SKU and application type into entitlement refs.
- OneStream Solution Exchange packages need workflow profile and cube compatibility metadata.
- Vena template marketplace items need workbook, workflow, and Office integration settlement units.
- Pigment template library entries map to block, metric, and list starter kits.
- Planful Solution Hub packages map to scenario, template, and reporting packs.
- IBM Planning Analytics accelerators map to TM1 cube and dimension assets.
- Board marketplace capsules map to procedure, layout, and data model assets.
- Jedox marketplace models map to database, cube, and spreadsheet report assets.

## Cross-Microservice Handoffs
- `marketplace` owns entitlement state, publisher contracts, and DealSet catalog.
- `billing` receives amount, currency, and usage-unit settlement facts.
- `audit-chain` seals activation, usage, export, and revocation events.
- `policy-engine` evaluates entitlement-aware Cedar hooks.
- `ontology` maps vendor template objects into Oyatie capability slugs.
- `data-warehouse` receives commercial planning usage facts.
- `workflow-engine` handles approval for paid activation in governed tenants.
- `compliance` receives settlement evidence for customer and publisher disputes.
