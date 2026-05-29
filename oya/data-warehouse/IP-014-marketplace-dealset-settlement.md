---
doc_class: IP
ip_id: IP-014
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-014: Marketplace DealSet Settlement

## Context
- DW14-CTX-01: This IP makes data-warehouse queries and governed shares billable through local DealSet settlement, not vendor metering.
- DW14-CTX-02: Snowflake private listings map to Oyatie `WarehouseDealSetSettlement` records with tenant-owned price proof.
- DW14-CTX-03: BigQuery Analytics Hub subscriptions map to the same settlement ledger and lose project-global billing assumptions.
- DW14-CTX-04: Redshift Data Sharing subscriptions map to dataset grants with explicit capacity and egress cost splits.
- DW14-CTX-05: Databricks SQL Marketplace shares map to audit-sealed governed shares with workflow approval.
- DW14-CTX-06: Synapse Analytics linked services are treated as import provenance, not as settlement authority.
- DW14-CTX-07: Firebolt share consumption becomes query-credit debits inside the tenant cost envelope.
- DW14-CTX-08: ClickHouse Cloud database sharing maps to cell-local catalog entries and tenant policy hooks.
- DW14-CTX-09: Vertica Eon consumption maps to workload-pool reservations with renewable settlement windows.
- DW14-CTX-10: Teradata Vantage data product billing maps to DealSet line items with data-class evidence.
- DW14-CTX-11: Yellowbrick capacity slices map to reserved pool reservations and no external invoice truth.
- DW14-CTX-12: The settlement path must support buyer, seller, platform, and marketplace fee participants.
- DW14-CTX-13: Settlement output feeds finance, audit-chain, policy, and ontology without direct peer service calls.
- DW14-CTX-14: DealSet settlement is denied when query provenance lacks catalog lineage or policy evaluation.
- DW14-CTX-15: This IP is the market-facing closure point for vendor displacement in data-warehouse.

## Data Model Deltas
- DW14-DDL-01: Add settlement header table.
```sql
CREATE TABLE warehouse_dealset_settlements (
    settlement_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    dealset_id UUID NOT NULL,
    buyer_principal_id UUID NOT NULL,
    seller_account_id UUID NOT NULL,
    vendor_source TEXT NOT NULL CHECK (vendor_source IN ('snowflake','bigquery','redshift','databricks_sql','synapse_analytics','firebolt','clickhouse_cloud','vertica','teradata_vantage','yellowbrick','oyatie_native')),
    settlement_window tstzrange NOT NULL,
    currency_code CHAR(3) NOT NULL,
    gross_amount_micros BIGINT NOT NULL CHECK (gross_amount_micros >= 0),
    platform_fee_micros BIGINT NOT NULL CHECK (platform_fee_micros >= 0),
    seller_net_micros BIGINT NOT NULL CHECK (seller_net_micros >= 0),
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_dealset_settlements_tenant_window_idx ON warehouse_dealset_settlements USING gist (tenant_id, settlement_window);
```
- DW14-DDL-02: Add line items for query, share, and egress charges.
```sql
CREATE TABLE warehouse_dealset_settlement_lines (
    line_id UUID PRIMARY KEY,
    settlement_id UUID NOT NULL REFERENCES warehouse_dealset_settlements(settlement_id) ON DELETE CASCADE,
    source_object_ref TEXT NOT NULL,
    charge_kind TEXT NOT NULL CHECK (charge_kind IN ('query_credit','share_subscription','egress_gib','reservation_hour','platform_fee','refund')),
    quantity NUMERIC(28,9) NOT NULL,
    unit_price_micros BIGINT NOT NULL,
    amount_micros BIGINT NOT NULL,
    lineage_hash BYTEA NOT NULL,
    ontology_projection_id UUID NOT NULL
);
CREATE INDEX wh_dealset_lines_settlement_idx ON warehouse_dealset_settlement_lines(settlement_id, charge_kind);
```
- DW14-RUST-01: Domain type for the settlement header.
```rust
pub struct WarehouseDealSetSettlement {
    pub settlement_id: SettlementId,
    pub tenant_id: TenantId,
    pub dealset_id: DealSetId,
    pub buyer_principal_id: PrincipalId,
    pub seller_account_id: AccountId,
    pub vendor_source: WarehouseVendorSource,
    pub settlement_window: TimeWindow,
    pub currency_code: IsoCurrency,
    pub gross_amount_micros: MoneyMicros,
    pub platform_fee_micros: MoneyMicros,
    pub seller_net_micros: MoneyMicros,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW14-RUST-02: Settlement line type keeps vendor object identity separate from Oyatie identity.
```rust
pub struct WarehouseDealSetSettlementLine {
    pub line_id: SettlementLineId,
    pub settlement_id: SettlementId,
    pub source_object_ref: VendorObjectRef,
    pub charge_kind: WarehouseChargeKind,
    pub quantity: Decimal,
    pub unit_price_micros: MoneyMicros,
    pub amount_micros: MoneyMicros,
    pub lineage_hash: LineageHash,
    pub ontology_projection_id: OntologyProjectionId,
}
```
- DW14-RUST-03: `WarehouseVendorSource` is closed over the ten named vendors plus `OyatieNative`.
- DW14-RUST-04: `MoneyMicros` rejects negative amounts before persistence.
- DW14-RUST-05: `LineageHash` is computed over dataset version, query id, policy decision, and share grant.

## API Endpoints
- DW14-API-01: REST preview endpoint.
```http
POST /v1/data-warehouse/dealsets/{dealset_id}/settlements:preview
Idempotency-Key: wh-settle-preview-2026-05-20-001
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","window_start":"2026-05-01T00:00:00Z","window_end":"2026-05-31T23:59:59Z","vendor_sources":["snowflake","bigquery","redshift"],"include_refunds":true}
```
- DW14-API-02: REST commit endpoint.
```http
POST /v1/data-warehouse/dealsets/{dealset_id}/settlements
Idempotency-Key: wh-settle-commit-2026-05-20-001
Content-Type: application/json

{"preview_token":"prv_warehouse_settle_014","expected_gross_amount_micros":972550000,"policy_context":{"purpose":"marketplace_settlement","jurisdiction":"KR"}}
```
- DW14-API-03: gRPC command.
```proto
rpc CommitWarehouseDealSetSettlement(CommitWarehouseDealSetSettlementRequest) returns (CommitWarehouseDealSetSettlementResponse);
message CommitWarehouseDealSetSettlementRequest {
  string tenant_id = 1;
  string dealset_id = 2;
  string preview_token = 3;
  int64 expected_gross_amount_micros = 4;
  repeated string vendor_sources = 5;
}
```
- DW14-API-04: AsyncAPI event.
```yaml
warehouse.dealset.settlement.committed.v1:
  payload:
    settlement_id: 01JWH14SETTLE
    tenant_id: 018f8d8f-6fd1-7c28-bd2c-91c4045a0401
    dealset_id: 01JDEALSET014
    gross_amount_micros: 972550000
    emitted_audit_class: WarehouseDealSetSettlementCommitted
```
- DW14-API-05: REST errors use `409 settlement_preview_changed` when any source lineage hash changed.
- DW14-API-06: gRPC errors map failed policy to `PERMISSION_DENIED` with `policy_decision_id`.
- DW14-API-07: Async events carry no raw SQL text, only query hash and catalog object ids.

## Cedar Policy Hooks
- DW14-CEDAR-01: principal = `Oyatie::Principal::"data_platform_operator:{principal_id}"`.
- DW14-CEDAR-02: action = `Oyatie::Action::"warehouse_dealset_settlement_commit"`.
- DW14-CEDAR-03: resource = `Oyatie::WarehouseDealSet::"{dealset_id}"`.
- DW14-CEDAR-04: context.tenant_id must equal resource tenant and every settlement line tenant.
- DW14-CEDAR-05: context.vendor_sources must be subset of approved marketplace migration vendors.
- DW14-CEDAR-06: context.lineage_complete must be true.
- DW14-CEDAR-07: context.currency_code must match DealSet contract currency.
- DW14-CEDAR-08: context.cross_region_egress_disclosed must be true when any line has egress.
- DW14-CEDAR-09: context.audit_event_class must equal `WarehouseDealSetSettlementCommitted`.
- DW14-CEDAR-10: deny when principal lacks `warehouse.settlement.commit` and `marketplace.dealset.settle`.

## Ontology Projection
- DW14-ONTO-01: Snowflake `LISTING_USAGE.EVENT_ID` -> `WarehouseUsageLine.vendor_event_id`.
- DW14-ONTO-02: BigQuery `AnalyticsHubSubscription.name` -> `WarehouseShareGrant.vendor_subscription_ref`.
- DW14-ONTO-03: Redshift `datashare_name` -> `WarehouseShareGrant.source_share_name`.
- DW14-ONTO-04: Databricks SQL `provider_listing_id` -> `WarehouseDealSet.vendor_listing_ref`.
- DW14-ONTO-05: Synapse `linkedServiceName` -> `WarehouseImportProvenance.vendor_connection_ref`.
- DW14-ONTO-06: Firebolt `engine_id` -> `WarehouseWorkloadPool.vendor_capacity_ref`.
- DW14-ONTO-07: ClickHouse Cloud `database_id` -> `WarehouseDataset.vendor_database_ref`.
- DW14-ONTO-08: Vertica `depot_name` -> `WarehouseReservation.vendor_depot_ref`.
- DW14-ONTO-09: Teradata Vantage `database_name` -> `WarehouseDataset.vendor_database_ref`.
- DW14-ONTO-10: Yellowbrick `cluster_id` -> `WarehouseWorkloadPool.vendor_capacity_ref`.
- DW14-ONTO-11: Vendor invoice id -> `WarehouseDealSetSettlement.external_reference` only as evidence, never authority.
- DW14-ONTO-12: Vendor SKU -> `WarehouseChargeKind` through a reviewed mapping table.

## Workflow Steps
- DW14-WF-01: Node `CollectUsage` reads finalized query and share usage for the settlement window.
- DW14-WF-02: Node `ProjectVendorObjects` maps vendor objects into ontology deltas.
- DW14-WF-03: Node `EvaluateSettlementPolicy` runs Cedar with DealSet contract context.
- DW14-WF-04: Branch `PolicyDenied` stops and emits a preview rejection.
- DW14-WF-05: Node `PriceLines` calculates query-credit, subscription, egress, reservation, and refund lines.
- DW14-WF-06: Branch `GrossMismatch` returns 409 and keeps the preview reusable for inspection.
- DW14-WF-07: Node `CommitLedger` inserts header and lines in one transaction.
- DW14-WF-08: Node `EmitAudit` emits `WarehouseDealSetSettlementCommitted`.
- DW14-WF-09: Node `NotifyFinance` hands off net seller lines to finance ledger.
- DW14-WF-10: Node `PublishMarketplaceEvidence` hands off public settlement proof to marketplace.
- DW14-WF-11: Branch `VendorSourceQuarantine` isolates lines from an untrusted migration import.
- DW14-WF-12: Node `CloseWindow` marks the settlement window immutable after audit seal.

## Audit Events
- DW14-AUDIT-01: `WarehouseDealSetSettlementPreviewed` records inputs, source counts, and preview token hash.
- DW14-AUDIT-02: `WarehouseDealSetSettlementPolicyDenied` records Cedar decision and denied context keys.
- DW14-AUDIT-03: `WarehouseDealSetSettlementCommitted` records settlement id, gross/net amounts, and line count.
- DW14-AUDIT-04: `WarehouseDealSetSettlementGrossMismatch` records expected versus recomputed amount.
- DW14-AUDIT-05: `WarehouseDealSetSettlementVendorSourceQuarantined` records source vendor and object ref.
- DW14-AUDIT-06: `WarehouseDealSetSettlementFinanceHandoffQueued` records target finance command id.
- DW14-AUDIT-07: `WarehouseDealSetSettlementMarketplaceEvidencePublished` records evidence bundle hash.

## SLO Targets
- DW14-SLO-01: p50 preview latency <= 180 ms for 10k line windows.
- DW14-SLO-02: p95 preview latency <= 650 ms for 100k line windows.
- DW14-SLO-03: p99 commit latency <= 900 ms for one settlement transaction.
- DW14-SLO-04: throughput >= 40 settlement previews per second per cell.
- DW14-SLO-05: availability >= 99.95 percent for settlement preview and commit APIs.
- DW14-SLO-06: audit emission lag p95 <= 3 seconds.
- DW14-SLO-07: finance handoff lag p95 <= 15 seconds.
- DW14-SLO-08: recomputation drift must remain 0 micros after commit.

## Failure Modes + Recovery
- DW14-FAIL-01: Vendor usage import is incomplete; quarantine source window, emit `WarehouseDealSetSettlementVendorSourceQuarantined`, and retry import.
- DW14-FAIL-02: DealSet contract changed after preview; return 409, invalidate preview token, and require new preview.
- DW14-FAIL-03: Cedar policy denies cross-region egress; split offending lines into denied preview evidence.
- DW14-FAIL-04: Finance handoff queue is unavailable; keep settlement committed and retry idempotently from outbox.
- DW14-FAIL-05: Audit-chain seal fails; transaction rolls back before settlement id becomes visible.
- DW14-FAIL-06: Amount overflow detected; reject command before persistence and page data-platform finance owner.

## Migration Notes
- DW14-MIG-01: Snowflake private listing usage is imported before customer invoices are cut over.
- DW14-MIG-02: BigQuery Analytics Hub rows must normalize project, dataset, and subscription ids separately.
- DW14-MIG-03: Redshift Data Sharing exports need namespace and producer account ids.
- DW14-MIG-04: Databricks SQL Marketplace imports must preserve provider, listing, and consumer workspace ids.
- DW14-MIG-05: Synapse Analytics linked-service metadata is provenance only and cannot set price.
- DW14-MIG-06: Firebolt engine telemetry maps to reservation-hour line items.
- DW14-MIG-07: ClickHouse Cloud database sharing must preserve organization and service ids.
- DW14-MIG-08: Vertica Eon depot usage maps to reservation utilization.
- DW14-MIG-09: Teradata Vantage data product exports must include database and account map evidence.
- DW14-MIG-10: Yellowbrick cluster capacity maps to reserved workload-pool settlement lines.

## Cross-Microservice Handoffs
- DW14-HANDOFF-01: Marketplace receives committed settlement evidence and exposes buyer/seller proof.
- DW14-HANDOFF-02: Finance receives seller net and platform fee lines for ledger posting.
- DW14-HANDOFF-03: Ontology receives vendor-to-Oyatie field deltas for catalog graph updates.
- DW14-HANDOFF-04: Audit-chain receives ADR-0263 event classes and Merkle seal material.
- DW14-HANDOFF-05: Policy receives Cedar decision ids for compliance review.
- DW14-HANDOFF-06: Workflow receives node completion states and retry branches.
- DW14-HANDOFF-07: Notification receives buyer and seller settlement summaries.
- DW14-HANDOFF-08: Data-pipeline receives quarantined source references for replay correction.
- DW14-HANDOFF-09: Cost-budget receives line item totals for budget envelope deltas.
- DW14-HANDOFF-10: Catalog receives immutable dataset share settlement status.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-014-marketplace-dealset-settlement.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-014-marketplace-dealset-settlement.md` matched `cost, emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
