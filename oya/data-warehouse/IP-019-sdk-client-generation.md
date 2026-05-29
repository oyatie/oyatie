---
doc_class: IP
ip_id: IP-019
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-2
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-019: SDK Client Generation

## Context
- DW19-CTX-01: This IP publishes typed data-warehouse clients without exposing vendor SDK semantics to application teams.
- DW19-CTX-02: Snowflake connector examples become migration snippets, not runtime dependencies.
- DW19-CTX-03: BigQuery client patterns become local REST, gRPC, and AsyncAPI examples.
- DW19-CTX-04: Redshift JDBC assumptions are replaced by Oyatie query and catalog commands.
- DW19-CTX-05: Databricks SQL statement APIs are represented through local query submission types.
- DW19-CTX-06: Synapse Analytics T-SQL client concerns are isolated to migration adapters.
- DW19-CTX-07: Firebolt SDK usage maps to workload-pool and query-client examples.
- DW19-CTX-08: ClickHouse Cloud HTTP client examples map to local export and query APIs.
- DW19-CTX-09: Vertica client examples map to replay and catalog projection snippets.
- DW19-CTX-10: Teradata Vantage client usage maps to migration import examples.
- DW19-CTX-11: Yellowbrick PostgreSQL-wire examples map to local API-first clients.
- DW19-CTX-12: Generated clients must carry idempotency, tenant id, policy decision, and audit event fields.
- DW19-CTX-13: Client generation includes Rust, TypeScript, Kotlin, Swift, and Python shapes.
- DW19-CTX-14: No generated SDK may bypass Cedar or capacity admission.
- DW19-CTX-15: SDK artifacts are cataloged with compatibility and deprecation metadata.

## Data Model Deltas
- DW19-DDL-01: Add SDK generation manifest table.
```sql
CREATE TABLE warehouse_sdk_generation_manifests (
    manifest_id UUID PRIMARY KEY,
    tenant_id UUID,
    api_surface TEXT NOT NULL CHECK (api_surface IN ('rest','grpc','asyncapi')),
    language TEXT NOT NULL CHECK (language IN ('rust','typescript','kotlin','swift','python')),
    contract_digest BYTEA NOT NULL,
    package_name TEXT NOT NULL,
    semver TEXT NOT NULL,
    generated_artifact_ref TEXT NOT NULL,
    compatibility_status TEXT NOT NULL CHECK (compatibility_status IN ('current','deprecated','blocked')),
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX wh_sdk_manifest_unique_idx ON warehouse_sdk_generation_manifests(api_surface, language, package_name, semver);
```
- DW19-DDL-02: Add SDK vendor example mapping table.
```sql
CREATE TABLE warehouse_sdk_vendor_example_mappings (
    mapping_id UUID PRIMARY KEY,
    manifest_id UUID NOT NULL REFERENCES warehouse_sdk_generation_manifests(manifest_id) ON DELETE CASCADE,
    vendor_source TEXT NOT NULL,
    vendor_example_ref TEXT NOT NULL,
    oyatie_example_ref TEXT NOT NULL,
    replaced_concept TEXT NOT NULL,
    policy_note TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_sdk_vendor_mapping_manifest_idx ON warehouse_sdk_vendor_example_mappings(manifest_id, vendor_source);
```
- DW19-RUST-01: SDK manifest type.
```rust
pub struct WarehouseSdkGenerationManifest {
    pub manifest_id: SdkManifestId,
    pub tenant_id: Option<TenantId>,
    pub api_surface: ApiSurface,
    pub language: SdkLanguage,
    pub contract_digest: Sha256Digest,
    pub package_name: PackageName,
    pub semver: Semver,
    pub generated_artifact_ref: ArtifactRef,
    pub compatibility_status: CompatibilityStatus,
    pub audit_event_id: AuditEventId,
}
```
- DW19-RUST-02: SDK command envelope included in every language.
```rust
pub struct WarehouseCommandEnvelope<T> {
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub idempotency_key: IdempotencyKey,
    pub policy_context: PolicyContext,
    pub payload: T,
}
```
- DW19-RUST-03: Generated clients expose `policy_decision_id` in every mutating response.
- DW19-RUST-04: Generated clients expose `audit_event_id` where ADR-0263 events are emitted.
- DW19-RUST-05: Vendor mappings are docs metadata only and never runtime dispatch keys.

## API Endpoints
- DW19-API-01: REST query client example.
```http
POST /v1/data-warehouse/queries
Idempotency-Key: wh-sdk-query-019
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","principal_id":"01JUSER019","sql_template_id":"rev_by_region_v3","parameters":{"region":"KR"},"workload_pool_id":"01JPOOL019"}
```
- DW19-API-02: REST SDK manifest endpoint.
```http
POST /v1/data-warehouse/sdk/manifests:generate
Content-Type: application/json

{"api_surface":"rest","language":"typescript","package_name":"@oyatie/data-warehouse","semver":"0.19.0","contract_digest":"sha256:019"}
```
- DW19-API-03: gRPC query call.
```proto
rpc SubmitWarehouseQuery(SubmitWarehouseQueryRequest) returns (SubmitWarehouseQueryResponse);
message SubmitWarehouseQueryRequest {
  string tenant_id = 1;
  string principal_id = 2;
  string idempotency_key = 3;
  string sql_template_id = 4;
  map<string,string> parameters = 5;
}
```
- DW19-API-04: AsyncAPI client event sample.
```yaml
warehouse.query.completed.v1:
  payload:
    query_id: 01JQUERY019
    sdk_language: typescript
    policy_decision_id: 01JPOLICY019
    audit_event_class: WarehouseQueryCompleted
```
- DW19-API-05: REST SDK generation rejects contract digests not present in catalog.
- DW19-API-06: gRPC generated clients include timeout, retry, and idempotency interceptors.
- DW19-API-07: AsyncAPI generated consumers require tenant-scoped subscription filters.

## Cedar Policy Hooks
- DW19-CEDAR-01: principal = `Oyatie::Principal::"sdk_publisher:{publisher_id}"`.
- DW19-CEDAR-02: action = `Oyatie::Action::"warehouse_sdk_generate"`.
- DW19-CEDAR-03: resource = `Oyatie::WarehouseApiContract::"{contract_digest}"`.
- DW19-CEDAR-04: context.language must be in approved SDK language set.
- DW19-CEDAR-05: context.contract_digest must match catalog current digest.
- DW19-CEDAR-06: context.includes_idempotency must be true for mutating APIs.
- DW19-CEDAR-07: context.includes_policy_decision must be true for all protected calls.
- DW19-CEDAR-08: context.includes_audit_event must be true for ADR-0263 eventing calls.
- DW19-CEDAR-09: context.vendor_dependency_count must equal 0.
- DW19-CEDAR-10: deny if generated package would expose raw vendor credentials.

## Ontology Projection
- DW19-ONTO-01: Snowflake `Connection` sample -> `WarehouseClientConfig.endpoint`.
- DW19-ONTO-02: BigQuery `JobConfig` sample -> `WarehouseQueryRequest.execution_options`.
- DW19-ONTO-03: Redshift JDBC URL -> `WarehouseClientConfig.api_base_url`.
- DW19-ONTO-04: Databricks SQL `StatementExecution` -> `WarehouseQuerySubmission`.
- DW19-ONTO-05: Synapse `SqlConnection` -> `WarehouseMigrationAdapterConfig`.
- DW19-ONTO-06: Firebolt `EngineClient` -> `WarehouseWorkloadPoolClient`.
- DW19-ONTO-07: ClickHouse Cloud HTTP query -> `WarehouseQueryRequest`.
- DW19-ONTO-08: Vertica `ConnectionProperties` -> `WarehouseMigrationAdapterConfig`.
- DW19-ONTO-09: Teradata Vantage connection profile -> `WarehouseMigrationSourceConfig`.
- DW19-ONTO-10: Yellowbrick PostgreSQL-wire profile -> `WarehouseMigrationSourceConfig`.
- DW19-ONTO-11: Vendor credential field -> `CredentialSidecarReference`.
- DW19-ONTO-12: Vendor retry option -> `OyatieRetryPolicy`.

## Workflow Steps
- DW19-WF-01: Node `LoadContracts` reads OpenAPI, proto, and AsyncAPI digests from catalog.
- DW19-WF-02: Node `EvaluateGenerationPolicy` runs Cedar on language and contract.
- DW19-WF-03: Branch `DigestNotCurrent` blocks package generation.
- DW19-WF-04: Node `GenerateRustClient` emits strongly typed Rust crate.
- DW19-WF-05: Node `GenerateTypeScriptClient` emits browser and node package.
- DW19-WF-06: Node `GenerateKotlinClient` emits JVM and Android artifacts.
- DW19-WF-07: Node `GenerateSwiftClient` emits Apple platform package.
- DW19-WF-08: Node `GeneratePythonClient` emits typed async package.
- DW19-WF-09: Node `InjectInterceptors` adds idempotency, tracing, and policy metadata.
- DW19-WF-10: Node `MapVendorExamples` writes migration example mapping rows.
- DW19-WF-11: Node `PublishArtifacts` pushes signed packages to internal registry.
- DW19-WF-12: Node `EmitAudit` emits SDK generation events.

## Audit Events
- DW19-AUDIT-01: `WarehouseSdkGenerationRequested` records surface, language, and digest.
- DW19-AUDIT-02: `WarehouseSdkGenerationPolicyDenied` records denied fields.
- DW19-AUDIT-03: `WarehouseSdkArtifactGenerated` records artifact ref and package semver.
- DW19-AUDIT-04: `WarehouseSdkVendorExampleMapped` records vendor source and replacement concept.
- DW19-AUDIT-05: `WarehouseSdkArtifactPublished` records registry and signature digest.
- DW19-AUDIT-06: `WarehouseSdkArtifactDeprecated` records replacement semver.
- DW19-AUDIT-07: `WarehouseSdkContractDigestMismatch` records requested and current digest.

## SLO Targets
- DW19-SLO-01: p50 SDK manifest generation <= 1 second.
- DW19-SLO-02: p95 full language generation <= 12 seconds.
- DW19-SLO-03: p99 package publication <= 45 seconds.
- DW19-SLO-04: throughput >= 20 SDK generation jobs per minute.
- DW19-SLO-05: availability >= 99.9 percent for SDK manifest API.
- DW19-SLO-06: generated client compile pass rate must be 100 percent for current contracts.
- DW19-SLO-07: contract digest mismatch detection <= 1 second.
- DW19-SLO-08: vendor dependency count in generated clients must be 0.

## Failure Modes + Recovery
- DW19-FAIL-01: Contract digest changed during generation; discard artifact and regenerate from current digest.
- DW19-FAIL-02: Language generator fails compile check; mark manifest blocked and keep previous package current.
- DW19-FAIL-03: Registry publication fails; retain signed artifact ref and retry publication idempotently.
- DW19-FAIL-04: Vendor dependency appears in generated lockfile; block publish and emit policy denial.
- DW19-FAIL-05: Example mapping references retired vendor API; publish migration warning and require replacement example.
- DW19-FAIL-06: AsyncAPI consumer lacks tenant filter; fail generation and record `WarehouseSdkGenerationPolicyDenied`.

## Migration Notes
- DW19-MIG-01: Snowflake SDK examples become local query and share examples only.
- DW19-MIG-02: BigQuery SDK examples become local query submission and export examples.
- DW19-MIG-03: Redshift JDBC examples become local REST/gRPC samples.
- DW19-MIG-04: Databricks SQL SDK examples become local statement lifecycle samples.
- DW19-MIG-05: Synapse Analytics client examples stay in migration adapter docs.
- DW19-MIG-06: Firebolt SDK examples become workload pool and query examples.
- DW19-MIG-07: ClickHouse Cloud HTTP examples become local export and query samples.
- DW19-MIG-08: Vertica client examples become replay import samples.
- DW19-MIG-09: Teradata Vantage examples become migration source config samples.
- DW19-MIG-10: Yellowbrick wire protocol examples become migration source config samples.

## Cross-Microservice Handoffs
- DW19-HANDOFF-01: Developer-platform receives signed SDK artifacts and manifest rows.
- DW19-HANDOFF-02: Catalog receives contract digest and compatibility status.
- DW19-HANDOFF-03: Policy receives Cedar generation decision evidence.
- DW19-HANDOFF-04: Audit-chain receives ADR-0263 SDK events.
- DW19-HANDOFF-05: Docs portal receives generated examples and vendor replacement notes.
- DW19-HANDOFF-06: Credential sidecar receives proof no raw vendor credential field is exposed.
- DW19-HANDOFF-07: Workflow receives blocked generation remediation tasks.
- DW19-HANDOFF-08: Tenant-admin receives SDK deprecation notices.
- DW19-HANDOFF-09: Marketplace receives client examples for governed shares.
- DW19-HANDOFF-10: Query planner receives generated client compatibility matrix.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-019-sdk-client-generation.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-019-sdk-client-generation.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
