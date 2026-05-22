---
doc_class: IP
ip_id: IP-024
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-024: Threat Model Control Map

## Context
- DW24-CTX-01: This IP maps data-warehouse threats to concrete controls, owners, evidence, and audit events.
- DW24-CTX-02: Snowflake account takeover patterns map to local credential sidecar and Cedar controls.
- DW24-CTX-03: BigQuery project sprawl patterns map to tenant boundary and catalog registration controls.
- DW24-CTX-04: Redshift public snapshot exposure maps to export and share policy controls.
- DW24-CTX-05: Databricks SQL token leakage maps to sidecar credential and SDK generation controls.
- DW24-CTX-06: Synapse Analytics linked-service overreach maps to migration adapter isolation controls.
- DW24-CTX-07: Firebolt engine over-permission maps to workload pool and budget controls.
- DW24-CTX-08: ClickHouse Cloud row-policy bypass maps to Cedar row-scope controls.
- DW24-CTX-09: Vertica role inheritance drift maps to DPIA and policy evidence controls.
- DW24-CTX-10: Teradata Vantage account sharing maps to principal alias and audit controls.
- DW24-CTX-11: Yellowbrick cluster network exposure maps to capacity and endpoint controls.
- DW24-CTX-12: Threat controls must be tied to implementation IPs, not loose prose.
- DW24-CTX-13: Every control has preventive, detective, and recovery evidence where applicable.
- DW24-CTX-14: Control failures block SLO promotion and audit closeout.
- DW24-CTX-15: Vendor threat names are translated into Oyatie threat taxonomy.

## Data Model Deltas
- DW24-DDL-01: Add threat control map table.
```sql
CREATE TABLE warehouse_threat_control_maps (
    control_map_id UUID PRIMARY KEY,
    tenant_id UUID,
    threat_id TEXT NOT NULL,
    threat_name TEXT NOT NULL,
    control_id TEXT NOT NULL,
    control_name TEXT NOT NULL,
    control_type TEXT NOT NULL CHECK (control_type IN ('preventive','detective','corrective','compensating')),
    mapped_ip_id TEXT NOT NULL,
    owner_team TEXT NOT NULL,
    evidence_ref TEXT NOT NULL,
    control_status TEXT NOT NULL CHECK (control_status IN ('draft','active','failed','retired')),
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_threat_control_map_threat_idx ON warehouse_threat_control_maps(threat_id, control_status);
```
- DW24-DDL-02: Add control test evidence table.
```sql
CREATE TABLE warehouse_threat_control_test_results (
    test_result_id UUID PRIMARY KEY,
    control_map_id UUID NOT NULL REFERENCES warehouse_threat_control_maps(control_map_id) ON DELETE CASCADE,
    test_name TEXT NOT NULL,
    test_kind TEXT NOT NULL CHECK (test_kind IN ('unit','integration','chaos','policy','manual_review')),
    test_status TEXT NOT NULL CHECK (test_status IN ('passed','failed','blocked')),
    evidence_hash BYTEA NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    remediation_ref TEXT
);
CREATE INDEX wh_threat_control_tests_map_idx ON warehouse_threat_control_test_results(control_map_id, observed_at DESC);
```
- DW24-RUST-01: Threat control map type.
```rust
pub struct WarehouseThreatControlMap {
    pub control_map_id: ThreatControlMapId,
    pub tenant_id: Option<TenantId>,
    pub threat_id: ThreatId,
    pub threat_name: ThreatName,
    pub control_id: ControlId,
    pub control_name: ControlName,
    pub control_type: ControlType,
    pub mapped_ip_id: ImplementationPlanId,
    pub owner_team: OwnerTeam,
    pub evidence_ref: EvidenceRef,
    pub control_status: ControlStatus,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW24-RUST-02: Control test result type.
```rust
pub struct WarehouseThreatControlTestResult {
    pub test_result_id: ControlTestResultId,
    pub control_map_id: ThreatControlMapId,
    pub test_name: TestName,
    pub test_kind: ControlTestKind,
    pub test_status: TestStatus,
    pub evidence_hash: Sha256Digest,
    pub observed_at: DateTime<Utc>,
    pub remediation_ref: Option<RemediationRef>,
}
```
- DW24-RUST-03: `ControlStatus::Active` requires at least one passing evidence result.
- DW24-RUST-04: `ControlType::Compensating` requires an expiry or replacement control.
- DW24-RUST-05: Tenant-null controls apply globally and can be narrowed by tenant-specific maps.

## API Endpoints
- DW24-API-01: REST control map upsert.
```http
POST /v1/data-warehouse/threat-controls
Idempotency-Key: wh-threat-control-024
Content-Type: application/json

{"threat_id":"WH-T-ACCOUNT-TAKEOVER","threat_name":"warehouse account takeover","control_id":"WH-C-CEDAR-SIDECAR-001","control_name":"Cedar plus credential sidecar gate","control_type":"preventive","mapped_ip_id":"IP-009","owner_team":"axis-data-platform","evidence_ref":"evidence://warehouse/threat/WH-C-CEDAR-SIDECAR-001"}
```
- DW24-API-02: REST test result append.
```http
POST /v1/data-warehouse/threat-controls/{control_map_id}/test-results
Content-Type: application/json

{"test_name":"cedar-denies-cross-tenant-query","test_kind":"policy","test_status":"passed","evidence_hash":"sha256:024","observed_at":"2026-05-20T21:00:00Z"}
```
- DW24-API-03: gRPC control map.
```proto
rpc UpsertWarehouseThreatControl(UpsertWarehouseThreatControlRequest) returns (UpsertWarehouseThreatControlResponse);
message UpsertWarehouseThreatControlRequest {
  string threat_id = 1;
  string control_id = 2;
  string control_type = 3;
  string mapped_ip_id = 4;
  string evidence_ref = 5;
}
```
- DW24-API-04: AsyncAPI event.
```yaml
warehouse.threat_control.failed.v1:
  payload:
    control_map_id: 01JWH24CONTROL
    threat_id: WH-T-ROW-POLICY-BYPASS
    control_id: WH-C-CEDAR-ROW-SCOPE
    audit_event_class: WarehouseThreatControlFailed
```
- DW24-API-05: REST rejects active status without passing evidence.
- DW24-API-06: gRPC returns `FAILED_PRECONDITION` when mapped IP id is unknown.
- DW24-API-07: Async failure events block SLO promotion through gate subscription.

## Cedar Policy Hooks
- DW24-CEDAR-01: principal = `Oyatie::Principal::"security_engineer:{principal_id}"`.
- DW24-CEDAR-02: action = `Oyatie::Action::"warehouse_threat_control_map_write"`.
- DW24-CEDAR-03: resource = `Oyatie::WarehouseThreatControl::"{control_id}"`.
- DW24-CEDAR-04: context.owner_team must be authorized for the mapped IP id.
- DW24-CEDAR-05: context.evidence_ref must point to immutable evidence.
- DW24-CEDAR-06: context.active_status requires passing evidence.
- DW24-CEDAR-07: context.compensating_control requires expiry.
- DW24-CEDAR-08: context.threat_id must be in approved warehouse threat taxonomy.
- DW24-CEDAR-09: context.audit_event_class must equal map upsert or failure class.
- DW24-CEDAR-10: deny if principal lacks `warehouse.security.control_map.write`.

## Ontology Projection
- DW24-ONTO-01: Snowflake user/role threat -> `WarehouseThreat.account_takeover`.
- DW24-ONTO-02: BigQuery project sprawl threat -> `WarehouseThreat.tenant_boundary_drift`.
- DW24-ONTO-03: Redshift public snapshot threat -> `WarehouseThreat.uncontrolled_export`.
- DW24-ONTO-04: Databricks SQL token leakage threat -> `WarehouseThreat.credential_exposure`.
- DW24-ONTO-05: Synapse linked service overreach -> `WarehouseThreat.adapter_overreach`.
- DW24-ONTO-06: Firebolt engine over-permission -> `WarehouseThreat.capacity_abuse`.
- DW24-ONTO-07: ClickHouse Cloud row policy bypass -> `WarehouseThreat.row_scope_bypass`.
- DW24-ONTO-08: Vertica role inheritance drift -> `WarehouseThreat.policy_drift`.
- DW24-ONTO-09: Teradata account sharing -> `WarehouseThreat.principal_ambiguity`.
- DW24-ONTO-10: Yellowbrick network exposure -> `WarehouseThreat.endpoint_exposure`.
- DW24-ONTO-11: Vendor risk severity -> `WarehouseThreat.risk_level`.
- DW24-ONTO-12: Local control id -> `WarehouseControl.control_id`.

## Workflow Steps
- DW24-WF-01: Node `IngestThreat` receives threat taxonomy row.
- DW24-WF-02: Node `MapControl` links threat to control id and mapped IP.
- DW24-WF-03: Node `ValidateEvidence` confirms immutable evidence hash.
- DW24-WF-04: Node `EvaluatePolicy` runs Cedar for map write.
- DW24-WF-05: Branch `MissingEvidence` keeps control draft.
- DW24-WF-06: Branch `CompensatingControl` requires expiry and replacement.
- DW24-WF-07: Node `ActivateControl` marks control active after passing evidence.
- DW24-WF-08: Node `RunControlTest` appends test result.
- DW24-WF-09: Branch `TestFailed` marks control failed and blocks promotion.
- DW24-WF-10: Node `OpenRemediation` sends task to owner team.
- DW24-WF-11: Node `EmitAudit` emits control map event.
- DW24-WF-12: Node `PublishControlMap` sends summary to compliance and security dashboard.

## Audit Events
- DW24-AUDIT-01: `WarehouseThreatControlMapped` records threat and control ids.
- DW24-AUDIT-02: `WarehouseThreatControlEvidenceAccepted` records evidence hash.
- DW24-AUDIT-03: `WarehouseThreatControlActivated` records active status.
- DW24-AUDIT-04: `WarehouseThreatControlTestPassed` records test result id.
- DW24-AUDIT-05: `WarehouseThreatControlFailed` records failed test and remediation.
- DW24-AUDIT-06: `WarehouseThreatControlCompensatingExpirySet` records expiry.
- DW24-AUDIT-07: `WarehouseThreatControlRetired` records replacement control id.

## SLO Targets
- DW24-SLO-01: p50 control map write <= 50 ms.
- DW24-SLO-02: p95 control map write <= 200 ms.
- DW24-SLO-03: p99 control map write <= 600 ms.
- DW24-SLO-04: throughput >= 300 control map writes per minute.
- DW24-SLO-05: availability >= 99.9 percent for threat control API.
- DW24-SLO-06: failed control propagation to SLO gates <= 5 seconds.
- DW24-SLO-07: active control without passing evidence count must be 0.
- DW24-SLO-08: remediation task creation p95 <= 10 seconds after failure.

## Failure Modes + Recovery
- DW24-FAIL-01: Threat id is unknown; reject write and route taxonomy update request.
- DW24-FAIL-02: Evidence hash cannot be verified; keep control draft and block activation.
- DW24-FAIL-03: Control test fails; mark control failed, block promotion, and open remediation.
- DW24-FAIL-04: Owner team mismatch; Cedar denies update and emits policy denial.
- DW24-FAIL-05: Compensating control expires; automatically fail control and page owner.
- DW24-FAIL-06: Compliance dashboard handoff fails; retry outbox and keep control status authoritative locally.

## Migration Notes
- DW24-MIG-01: Snowflake roles and network policies seed account takeover controls.
- DW24-MIG-02: BigQuery project and dataset IAM seed tenant boundary controls.
- DW24-MIG-03: Redshift snapshot and datashare settings seed export controls.
- DW24-MIG-04: Databricks SQL PAT and service principal usage seed credential controls.
- DW24-MIG-05: Synapse Analytics linked services seed adapter scope controls.
- DW24-MIG-06: Firebolt engine grants seed capacity abuse controls.
- DW24-MIG-07: ClickHouse Cloud row policies seed row-scope controls.
- DW24-MIG-08: Vertica inherited roles seed policy drift controls.
- DW24-MIG-09: Teradata Vantage shared account evidence seeds principal controls.
- DW24-MIG-10: Yellowbrick cluster network settings seed endpoint exposure controls.

## Cross-Microservice Handoffs
- DW24-HANDOFF-01: Security receives active and failed control map rows.
- DW24-HANDOFF-02: Compliance receives control evidence and risk mapping.
- DW24-HANDOFF-03: SLO gate receives failed-control block signals.
- DW24-HANDOFF-04: Audit-chain receives ADR-0263 control events.
- DW24-HANDOFF-05: Workflow receives remediation tasks.
- DW24-HANDOFF-06: Policy receives Cedar map write decisions.
- DW24-HANDOFF-07: Tenant-admin receives applicable tenant control summary.
- DW24-HANDOFF-08: Catalog receives exposure-blocking control status.
- DW24-HANDOFF-09: Credential sidecar receives credential-related threat controls.
- DW24-HANDOFF-10: Observability receives control test pass/fail metrics.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-024-threat-model-control-map.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
