---
doc_class: IP
ip_id: IP-012-abuse-defence-edge-waf
microservice: data-warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253-amendment
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0294
  - ADR-0296
  - ADR-0297
  - ADR-0314
  - ADR-0321
journey_ref: J-DW-012-abuse-defence-edge-waf
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-012 Data Warehouse abuse-defence-edge-waf

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-012-abuse-defence-edge-waf.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- abuse-defence-edge-waf-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- abuse-defence-edge-waf-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- abuse-defence-edge-waf-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- abuse-defence-edge-waf-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- abuse-defence-edge-waf-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- abuse-defence-edge-waf-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- abuse-defence-edge-waf-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- abuse-defence-edge-waf-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- abuse-defence-edge-waf-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- abuse-defence-edge-waf-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- abuse-defence-edge-waf-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-012 defines the edge WAF and abuse-defence posture for data-warehouse REST and event replay endpoints.
- Snowflake parity includes network policies and login anomaly awareness, but Oyatie enforces at tenant edge.
- BigQuery parity includes quota and API abuse controls, but Oyatie adds tenant-aware Cedar context.
- Redshift parity includes API throttles, but Oyatie binds throttle decisions to workload pool and tenant.
- Databricks SQL parity includes workspace IP lists, but Oyatie binds edge checks to service routes.
- Synapse Analytics parity includes firewall and private endpoints, but Oyatie adds API-layer signal scoring.
- Firebolt parity includes account limits, but Oyatie adds request-shape and replay abuse detection.
- ClickHouse Cloud parity includes IP filters and quotas, but Oyatie adds tenant event replay guardrails.
- Vertica parity includes connection limits, but Oyatie applies HTTP and gRPC edge protection.
- Teradata Vantage parity includes workload throttles, but Oyatie detects spoofed query-band context.
- Yellowbrick parity includes queue and connection controls, but Oyatie adds edge abuse decisions.
- Emergency-services traffic is challenge-free only under IP-013 and remains audited.
- Abuse defence never rewrites tenant policy; it contributes risk context and enforced edge decisions.

## Data Model Deltas
```sql
CREATE TABLE dw_abuse_signal (
  signal_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  principal_id uuid,
  route_slug text NOT NULL,
  signal_family text NOT NULL,
  risk_score integer NOT NULL CHECK (risk_score BETWEEN 0 AND 100),
  source_ip inet,
  user_agent_hash text,
  decision text NOT NULL CHECK (decision IN ('allow','challenge','rate_limit','block','bypass')),
  audit_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE dw_edge_waf_rule_binding (
  rule_binding_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  route_slug text NOT NULL,
  rule_family text NOT NULL,
  threshold integer NOT NULL,
  action text NOT NULL,
  emergency_services_exempt boolean NOT NULL DEFAULT false
);
```
```rust
pub struct AbuseSignal {
    pub signal_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub principal_id: Option<uuid::Uuid>,
    pub route_slug: String,
    pub signal_family: String,
    pub risk_score: u8,
    pub decision: AbuseDecision,
    pub audit_id: uuid::Uuid,
}
pub enum AbuseDecision { Allow, Challenge, RateLimit, Block, Bypass }
pub struct EdgeWafRuleBinding {
    pub route_slug: String,
    pub rule_family: String,
    pub threshold: u8,
    pub action: AbuseDecision,
    pub emergency_services_exempt: bool,
}
```

## API Endpoints
- REST `POST /v1/data-warehouse/edge/abuse:score` scores a request envelope before route dispatch.
```json
{"tenant_id":"018f-tenant","route_slug":"queries.run","source_ip":"203.0.113.10","user_agent_hash":"sha256:ua","request_shape_hash":"sha256:req"}
```
- REST `GET /v1/data-warehouse/edge/abuse-signals` lists tenant-visible signals.
```json
{"tenant_id":"018f-tenant","route_slug":"events.replay","decision":"rate_limit","since":"2026-05-20T00:00:00Z"}
```
- gRPC `ScoreWarehouseEdgeRequest(ScoreWarehouseEdgeRequestRequest) returns (ScoreWarehouseEdgeRequestResponse)`.
```json
{"tenantId":"018f-tenant","routeSlug":"governed-shares.create","principalId":"018f-principal","sourceIp":"203.0.113.10"}
```
- AsyncAPI channel `data-warehouse.abuse.decision.v1`.
```json
{"tenant_id":"018f-tenant","signal_id":"018f-signal","decision":"challenge","risk_score":72,"audit_event_class":"WarehouseAbuseDefenceDecisionRecorded"}
```

## Cedar Policy Hooks
- principal: `EdgeGateway::"regional-edge"` or `ServicePrincipal::"api-gateway"`.
- action: `Action::"dataWarehouse::ScoreEdgeRequest"` and `Action::"dataWarehouse::ReadAbuseSignals"`.
- resource: `WarehouseEdgeRoute::"tenant_id/route_slug"`.
- context: `tenant_id`, `route_slug`, `source_ip`, `risk_score`, `audience_type`, `emergency_services_asserted`, `audit_event_class`.
- permit scoring for gateway principals only.
- deny challenge or block for valid emergency-services bypass context; defer to IP-013 audit path.
- deny read of abuse signals unless tenant-admin or security purpose is present.
- deny route dispatch when decision is `block`.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake network policy | `WarehouseEdgeRule` | `allowed_ip_list` -> `allow_source_set`, `blocked_ip_list` -> `deny_source_set` |
| BigQuery quota project | `WarehouseEdgeRule` | `quota_project` -> `tenant_quota_scope`, `limit` -> `threshold` |
| Redshift API throttle | `WarehouseEdgeRule` | `api_name` -> `route_slug`, `rate` -> `threshold` |
| Databricks IP access list | `WarehouseEdgeRule` | `list_type` -> `rule_family`, `ip_addresses` -> `source_set` |
| Synapse firewall rule | `WarehouseEdgeRule` | `start_ip` -> `source_range_start`, `end_ip` -> `source_range_end` |
| Firebolt account limit | `WarehouseEdgeRule` | `account` -> `tenant_alias`, `limit` -> `threshold` |
| ClickHouse Cloud quota | `WarehouseEdgeRule` | `quota_key` -> `rule_family`, `duration` -> `window_seconds` |
| Vertica connection limit | `WarehouseEdgeRule` | `resource_pool` -> `route_scope`, `max_connections` -> `threshold` |
| Teradata throttle rule | `WarehouseEdgeRule` | `query_band` -> `context_key`, `limit` -> `threshold` |
| Yellowbrick queue limit | `WarehouseEdgeRule` | `resource_group` -> `route_scope`, `queue_depth` -> `threshold` |

## Workflow Steps
- node `collect_edge_features`: source, route, tenant, principal, request shape, and rate counters.
- node `evaluate_abuse_rules`: compute risk score and candidate decision.
- branch `emergency_services_asserted`: hand off to IP-013 validation before challenge.
- branch `decision_block`: stop request before application dispatch.
- node `attach_risk_context`: pass risk score to Cedar for downstream operation.
- node `record_abuse_signal`: persist signal and audit id.
- node `emit_tenant_signal`: publish tenant-visible decision event.
- node `update_rate_window`: advance counters only after decision is sealed.

## Audit Events
- `WarehouseAbuseDefenceDecisionRecorded`: signal persisted with decision.
- `AbuseDefenceBotBlocked`: bot signature blocked.
- `AbuseDefenceSpoofDetected`: spoofed source or identity signal detected.
- `AbuseDefenceChallengeIssued`: challenge required before dispatch.
- `AbuseDefenceRateLimitHit`: route rate limit applied.
- `AbuseDefenceEmergencyServiceBypass`: challenge bypassed for verified emergency-service context.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 edge score latency | 4 ms |
| p95 edge score latency | 18 ms |
| p99 edge score latency | 45 ms |
| throughput | 60,000 scored requests/sec per edge cell |
| availability | 99.99% for edge scoring path |

## Failure Modes + Recovery
- Scoring engine unavailable: fail open for authenticated read-only routes, fail closed for writes, and emit degraded event.
- False positive challenge spike: lower challenge threshold only through audited tenant override.
- Emergency-service spoof attempt: hand to IP-013, block if assertion invalid, and emit forgery event.
- Rate counter store unavailable: apply conservative static limits and mark telemetry degraded.
- Tenant signal query abuse: rate-limit signal reads separately from warehouse operations.
- Vendor migration metadata missing: use route-level default rules until vendor-specific mapping lands.

## Migration Notes
- Snowflake network policy imports seed allow/deny source sets but do not bypass Cedar.
- BigQuery quotas map to route and tenant thresholds.
- Redshift API throttles map to route family limits.
- Databricks IP access lists map to edge source sets.
- Synapse firewall rules map to regional source range policies.
- Firebolt limits map to tenant and engine route thresholds.
- ClickHouse Cloud quotas map to route window counters.
- Vertica connection limits map to workload route controls.
- Teradata throttle rules map query-band spoof detection.
- Yellowbrick queue limits map to resource-group route thresholds.

## Cross-Microservice Handoffs
- API Gateway calls scoring before REST route dispatch.
- Policy-engine receives risk score and abuse context in Cedar context.
- Security receives spoof, bot, credential, and challenge events.
- Audit-chain seals all decisions and emergency bypasses.
- Observability receives latency, rate, false-positive, and block metrics.
- Tenant Admin UI receives tenant-visible abuse signal summaries.
- IP-013 emergency bypass validates protected no-challenge paths.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-012-abuse-defence-edge-waf.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-012-abuse-defence-edge-waf.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
