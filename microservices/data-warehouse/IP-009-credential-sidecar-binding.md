---
doc_class: IP
ip_id: IP-009-credential-sidecar-binding
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
journey_ref: J-DW-009-credential-sidecar-binding
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-009 Data Warehouse credential-sidecar-binding

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-009-credential-sidecar-binding.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- credential-sidecar-binding-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- credential-sidecar-binding-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- credential-sidecar-binding-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- credential-sidecar-binding-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- credential-sidecar-binding-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- credential-sidecar-binding-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- credential-sidecar-binding-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- credential-sidecar-binding-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- credential-sidecar-binding-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- credential-sidecar-binding-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- credential-sidecar-binding-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-009 binds vendor credentials to the sidecar model so warehouse adapters never hold raw long-lived secrets.
- Snowflake private key, OAuth, and external browser flows become short-lived sidecar leases.
- BigQuery service account, workload identity, and OAuth tokens become scoped sidecar leases.
- Redshift IAM auth, Secrets Manager references, and database credentials become scoped sidecar leases.
- Databricks SQL PATs, OAuth tokens, and workspace credentials become scoped sidecar leases.
- Synapse Analytics managed identity, SQL credentials, and linked-service secrets become scoped sidecar leases.
- Firebolt service account secrets and engine credentials become scoped sidecar leases.
- ClickHouse Cloud API keys, user passwords, and TLS client certs become scoped sidecar leases.
- Vertica database credentials and TLS materials become scoped sidecar leases.
- Teradata Vantage wallet entries, database credentials, and LDAP bind references become scoped sidecar leases.
- Yellowbrick database credentials and admin API tokens become scoped sidecar leases.
- The warehouse service requests a credential capability, not a secret value.
- Sidecar leases are bound to tenant, vendor, operation, workload pool, and audit event.

## Data Model Deltas
```sql
CREATE TABLE dw_credential_binding (
  credential_binding_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  source_vendor text NOT NULL,
  credential_purpose text NOT NULL,
  sidecar_secret_ref text NOT NULL,
  allowed_operations text[] NOT NULL,
  rotation_interval_seconds integer NOT NULL,
  last_rotated_at timestamptz,
  status text NOT NULL CHECK (status IN ('active','rotating','revoked'))
);
CREATE TABLE dw_credential_lease (
  lease_id uuid PRIMARY KEY,
  credential_binding_id uuid NOT NULL REFERENCES dw_credential_binding(credential_binding_id),
  tenant_id uuid NOT NULL,
  operation_slug text NOT NULL,
  expires_at timestamptz NOT NULL,
  audit_id uuid NOT NULL,
  issued_to_call_id uuid
);
```
```rust
pub struct CredentialBinding {
    pub credential_binding_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub source_vendor: String,
    pub credential_purpose: String,
    pub sidecar_secret_ref: String,
    pub allowed_operations: Vec<String>,
    pub rotation_interval_seconds: u32,
    pub status: CredentialBindingStatus,
}
pub enum CredentialBindingStatus { Active, Rotating, Revoked }
pub struct CredentialLease {
    pub lease_id: uuid::Uuid,
    pub operation_slug: String,
    pub expires_at: time::OffsetDateTime,
    pub audit_id: uuid::Uuid,
}
```

## API Endpoints
- REST `POST /v1/data-warehouse/credential-bindings` registers a vendor credential reference.
```json
{"tenant_id":"018f-tenant","source_vendor":"databricks_sql","credential_purpose":"statement-execution","sidecar_secret_ref":"openbao://dw/prod/dbsql","allowed_operations":["warehouse-query-run"]}
```
- REST `POST /v1/data-warehouse/credential-bindings/{binding_id}:rotate` starts rotation.
```json
{"tenant_id":"018f-tenant","reason":"scheduled-rotation","replacement_ref":"openbao://dw/prod/dbsql-v2"}
```
- gRPC `AcquireWarehouseCredentialLease(AcquireWarehouseCredentialLeaseRequest) returns (AcquireWarehouseCredentialLeaseResponse)`.
```json
{"tenantId":"018f-tenant","sourceVendor":"SNOWFLAKE","operationSlug":"warehouse-query-run","workloadPoolId":"018f-pool"}
```
- AsyncAPI channel `data-warehouse.credential.lease.issued.v1`.
```json
{"tenant_id":"018f-tenant","lease_id":"018f-lease","source_vendor":"snowflake","expires_at":"2026-05-20T12:05:00Z","audit_event_class":"WarehouseCredentialLeaseIssued"}
```

## Cedar Policy Hooks
- principal: `ServicePrincipal::"data-warehouse-adapter"` or `WorkflowService::"data-warehouse"`.
- action: `Action::"dataWarehouse::AcquireCredentialLease"` and `Action::"dataWarehouse::RotateCredentialBinding"`.
- resource: `WarehouseCredentialBinding::"tenant_id/source_vendor/credential_purpose"`.
- context: `tenant_id`, `source_vendor`, `operation_slug`, `workload_pool_id`, `sidecar_secret_ref`, `audit_event_class`, `rotation_reason`.
- permit requires operation in `allowed_operations`, active binding, and sidecar health.
- deny if caller asks for raw secret material.
- deny if binding status is `rotating` and operation is not read-only.
- deny rotation without security or tenant-admin authority.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake key pair user | `WarehouseCredentialBinding` | `user` -> `credential_subject`, `private_key_path` -> `sidecar_secret_ref` |
| BigQuery service account | `WarehouseCredentialBinding` | `client_email` -> `credential_subject`, `key_id` -> `rotation_marker` |
| Redshift IAM role | `WarehouseCredentialBinding` | `role_arn` -> `credential_subject`, `cluster` -> `resource_scope` |
| Databricks PAT/OAuth client | `WarehouseCredentialBinding` | `workspace_url` -> `resource_scope`, `client_id` -> `credential_subject` |
| Synapse managed identity | `WarehouseCredentialBinding` | `principal_id` -> `credential_subject`, `workspace` -> `resource_scope` |
| Firebolt service account | `WarehouseCredentialBinding` | `account_name` -> `resource_scope`, `client_id` -> `credential_subject` |
| ClickHouse Cloud API key | `WarehouseCredentialBinding` | `key_id` -> `credential_subject`, `service_id` -> `resource_scope` |
| Vertica database user | `WarehouseCredentialBinding` | `username` -> `credential_subject`, `database` -> `resource_scope` |
| Teradata wallet alias | `WarehouseCredentialBinding` | `wallet_alias` -> `credential_subject`, `system` -> `resource_scope` |
| Yellowbrick admin token | `WarehouseCredentialBinding` | `token_id` -> `credential_subject`, `cluster` -> `resource_scope` |

## Workflow Steps
- node `register_binding`: validate sidecar reference and operation allowlist.
- node `verify_sidecar_health`: require signing and secret fetch readiness.
- node `evaluate_lease_policy`: check caller, tenant, vendor, and operation.
- branch `binding_rotating`: allow read-only lease or return retryable denial.
- node `issue_short_lived_lease`: create sidecar lease without returning secret value to domain code.
- node `invoke_adapter_with_lease`: pass lease id to vendor adapter boundary.
- node `revoke_lease`: revoke or let TTL expire after adapter call completes.
- node `seal_credential_audit`: record lease, rotation, and denied access events.

## Audit Events
- `WarehouseCredentialBindingRegistered`: binding created.
- `WarehouseCredentialLeaseIssued`: short-lived lease issued.
- `WarehouseCredentialLeaseDenied`: Cedar or sidecar denial.
- `WarehouseCredentialRotationStarted`: rotation branch started.
- `WarehouseCredentialRotationCompleted`: replacement activated.
- `WarehouseCredentialRawSecretRequestBlocked`: caller attempted raw secret access.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 lease acquisition | 15 ms |
| p95 lease acquisition | 80 ms |
| p99 lease acquisition | 200 ms |
| throughput | 12,000 lease checks/sec per sidecar pool |
| availability | 99.99% for credential lease path |

## Failure Modes + Recovery
- Sidecar unavailable: fail closed for vendor calls and keep workflow node retryable.
- Binding revoked: deny leases, mark dependent workload pools degraded, and notify tenant admin.
- Rotation half-complete: keep previous binding active until replacement validates, then swap atomically.
- Raw secret request detected: block call, emit audit event, and quarantine caller service token.
- Vendor auth failure: revoke lease, mark credential suspect, and launch rotation workflow.
- OpenBao latency spike: use no secret cache for writes; shed non-critical read-only operations first.

## Migration Notes
- Snowflake key pairs move into sidecar references with rotation markers.
- BigQuery JSON keys move to workload identity or sidecar-held service-account refs.
- Redshift database passwords move to IAM or sidecar-held temporary credentials.
- Databricks PATs move to OAuth/client credentials managed through sidecar.
- Synapse secrets move from linked-service literals to managed identity or sidecar refs.
- Firebolt secrets move to service-account lease acquisition.
- ClickHouse Cloud API keys move to sidecar refs and per-operation leases.
- Vertica credentials move out of connection strings into sidecar refs.
- Teradata wallet aliases remain aliases only; sidecar owns material.
- Yellowbrick admin tokens move to scoped and rotating sidecar references.

## Cross-Microservice Handoffs
- Security owns sidecar policy, raw-secret blocking, and rotation posture.
- Identity owns service-principal trust and tenant-admin rotation authority.
- Policy-engine owns lease and rotation Cedar decisions.
- Audit-chain owns lease, denial, and rotation evidence.
- Workflow owns rotation and adapter-call durable branches.
- Observability receives sidecar latency, denial, and rotation metrics.
- Vendor adapters receive only lease ids and never receive raw secrets from domain code.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-009-credential-sidecar-binding.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-009-credential-sidecar-binding.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
