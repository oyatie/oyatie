---
doc_class: IP
ip_id: IP-023
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0251, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0321]
journey_ref: J89-uk-aadc-minor-ux-adaptation
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-023: DPIA Evidence Packet

## Context
- DW23-CTX-01: This IP packages data-warehouse privacy impact evidence for regulated datasets and vendor migrations.
- DW23-CTX-02: Snowflake table and share metadata becomes DPIA source inventory evidence.
- DW23-CTX-03: BigQuery dataset policy tags become data-class and purpose evidence.
- DW23-CTX-04: Redshift Lake Formation or IAM mappings become access-control evidence only.
- DW23-CTX-05: Databricks SQL table ACLs become migration evidence for local Cedar policy.
- DW23-CTX-06: Synapse Analytics Purview labels become classification import evidence.
- DW23-CTX-07: Firebolt role grants become access-history evidence.
- DW23-CTX-08: ClickHouse Cloud role and row-policy metadata becomes policy migration evidence.
- DW23-CTX-09: Vertica roles and access policies become local policy-review evidence.
- DW23-CTX-10: Teradata Vantage roles and profiles become historical access evidence.
- DW23-CTX-11: Yellowbrick grants become migration evidence for tenant policy.
- DW23-CTX-12: DPIA packets must include data categories, purpose, retention, residency, access, and audit events.
- DW23-CTX-13: Packets are produced per dataset or share before public catalog activation.
- DW23-CTX-14: Evidence packets are immutable after compliance signoff.
- DW23-CTX-15: DPIA evidence feeds compliance, tenant admin, audit closeout, and SLO promotion.

## Data Model Deltas
- DW23-DDL-01: Add DPIA packet table.
```sql
CREATE TABLE warehouse_dpia_evidence_packets (
    packet_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    catalog_object_id UUID NOT NULL,
    packet_status TEXT NOT NULL CHECK (packet_status IN ('draft','ready_for_review','approved','rejected','superseded')),
    data_categories TEXT[] NOT NULL,
    processing_purposes TEXT[] NOT NULL,
    retention_policy_ref TEXT NOT NULL,
    residency_overlay_id UUID NOT NULL,
    access_policy_ref TEXT NOT NULL,
    evidence_bundle_ref TEXT NOT NULL,
    reviewer_principal_id UUID,
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ
);
CREATE INDEX wh_dpia_packet_object_idx ON warehouse_dpia_evidence_packets(tenant_id, catalog_object_id, packet_status);
```
- DW23-DDL-02: Add DPIA evidence item table.
```sql
CREATE TABLE warehouse_dpia_evidence_items (
    item_id UUID PRIMARY KEY,
    packet_id UUID NOT NULL REFERENCES warehouse_dpia_evidence_packets(packet_id) ON DELETE CASCADE,
    evidence_kind TEXT NOT NULL CHECK (evidence_kind IN ('classification','access_control','residency','retention','lineage','audit_event','vendor_export','risk_mitigation')),
    evidence_ref TEXT NOT NULL,
    evidence_hash BYTEA NOT NULL,
    vendor_source TEXT,
    risk_level TEXT NOT NULL CHECK (risk_level IN ('low','medium','high','critical')),
    accepted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_dpia_items_packet_kind_idx ON warehouse_dpia_evidence_items(packet_id, evidence_kind);
```
- DW23-RUST-01: DPIA packet type.
```rust
pub struct WarehouseDpiaEvidencePacket {
    pub packet_id: DpiaPacketId,
    pub tenant_id: TenantId,
    pub catalog_object_id: CatalogObjectId,
    pub packet_status: DpiaPacketStatus,
    pub data_categories: Vec<DataCategory>,
    pub processing_purposes: Vec<ProcessingPurpose>,
    pub retention_policy_ref: RetentionPolicyRef,
    pub residency_overlay_id: OverlayId,
    pub access_policy_ref: AccessPolicyRef,
    pub evidence_bundle_ref: EvidenceBundleRef,
    pub reviewer_principal_id: Option<PrincipalId>,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW23-RUST-02: DPIA evidence item type.
```rust
pub struct WarehouseDpiaEvidenceItem {
    pub item_id: DpiaEvidenceItemId,
    pub packet_id: DpiaPacketId,
    pub evidence_kind: DpiaEvidenceKind,
    pub evidence_ref: EvidenceRef,
    pub evidence_hash: Sha256Digest,
    pub vendor_source: Option<WarehouseVendorSource>,
    pub risk_level: RiskLevel,
    pub accepted: bool,
}
```
- DW23-RUST-03: `DpiaPacketStatus::Approved` requires accepted evidence for every mandatory kind.
- DW23-RUST-04: Critical-risk evidence requires reviewer principal and mitigation item.
- DW23-RUST-05: Superseded packets remain visible for audit history.

## API Endpoints
- DW23-API-01: REST packet creation.
```http
POST /v1/data-warehouse/dpia/packets
Idempotency-Key: wh-dpia-packet-023
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","catalog_object_id":"01JCATALOG023","data_categories":["employee_compensation","financial_aggregate"],"processing_purposes":["analytics","statutory_reporting"],"retention_policy_ref":"retention:data-warehouse:finance:7y","residency_overlay_id":"01JOVERLAY023","access_policy_ref":"cedar:warehouse:finance-analyst"}
```
- DW23-API-02: REST review endpoint.
```http
POST /v1/data-warehouse/dpia/packets/{packet_id}:review
Content-Type: application/json

{"decision":"approved","review_notes":"mandatory evidence complete; high risk mitigated by residency and purpose controls","evidence_bundle_ref":"evidence://dpia/01JWH23PACKET"}
```
- DW23-API-03: gRPC packet build.
```proto
rpc BuildWarehouseDpiaPacket(BuildWarehouseDpiaPacketRequest) returns (BuildWarehouseDpiaPacketResponse);
message BuildWarehouseDpiaPacketRequest {
  string tenant_id = 1;
  string catalog_object_id = 2;
  repeated string data_categories = 3;
  repeated string processing_purposes = 4;
  string residency_overlay_id = 5;
}
```
- DW23-API-04: AsyncAPI event.
```yaml
warehouse.dpia.packet.approved.v1:
  payload:
    packet_id: 01JWH23PACKET
    catalog_object_id: 01JCATALOG023
    reviewer_principal_id: 01JREVIEWER023
    audit_event_class: WarehouseDpiaPacketApproved
```
- DW23-API-05: REST review rejects `approved` when mandatory evidence is absent.
- DW23-API-06: gRPC returns `FAILED_PRECONDITION` for missing residency overlay.
- DW23-API-07: Async rejection events include missing evidence kinds.

## Cedar Policy Hooks
- DW23-CEDAR-01: principal = `Oyatie::Principal::"privacy_reviewer:{principal_id}"`.
- DW23-CEDAR-02: action = `Oyatie::Action::"warehouse_dpia_packet_review"`.
- DW23-CEDAR-03: resource = `Oyatie::WarehouseDpiaPacket::"{packet_id}"`.
- DW23-CEDAR-04: context.tenant_id must equal packet tenant.
- DW23-CEDAR-05: context.mandatory_evidence_complete must be true for approval.
- DW23-CEDAR-06: context.critical_risk_count must be 0 unless mitigations accepted.
- DW23-CEDAR-07: context.residency_overlay_active must be true.
- DW23-CEDAR-08: context.processing_purpose_allowed must be true.
- DW23-CEDAR-09: context.audit_event_class must equal approval or rejection class.
- DW23-CEDAR-10: deny if principal lacks `privacy.dpia.review`.

## Ontology Projection
- DW23-ONTO-01: Snowflake tag `DATA_CATEGORY` -> `WarehouseDpiaEvidenceItem.classification`.
- DW23-ONTO-02: BigQuery policy tag -> `WarehouseDpiaEvidenceItem.classification`.
- DW23-ONTO-03: Redshift IAM role mapping -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-04: Databricks SQL table ACL -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-05: Synapse Purview classification -> `WarehouseDpiaEvidenceItem.classification`.
- DW23-ONTO-06: Firebolt role grant -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-07: ClickHouse Cloud row policy -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-08: Vertica role grant -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-09: Teradata profile -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-10: Yellowbrick grant -> `WarehouseDpiaEvidenceItem.access_control`.
- DW23-ONTO-11: Vendor retention setting -> `WarehouseDpiaEvidenceItem.retention`.
- DW23-ONTO-12: Vendor audit export -> `WarehouseDpiaEvidenceItem.audit_event`.

## Workflow Steps
- DW23-WF-01: Node `OpenPacket` creates draft packet for catalog object.
- DW23-WF-02: Node `CollectClassification` reads catalog data class and vendor labels.
- DW23-WF-03: Node `CollectAccessControls` gathers Cedar and vendor migration evidence.
- DW23-WF-04: Node `CollectResidency` attaches active residency overlay.
- DW23-WF-05: Node `CollectRetention` attaches retention policy.
- DW23-WF-06: Node `CollectLineage` attaches lineage and source export hashes.
- DW23-WF-07: Branch `MissingMandatoryEvidence` keeps packet draft.
- DW23-WF-08: Node `EvaluateRisk` assigns risk levels per evidence item.
- DW23-WF-09: Branch `CriticalRisk` requires mitigation acceptance.
- DW23-WF-10: Node `ReviewPacket` applies Cedar and reviewer decision.
- DW23-WF-11: Node `SealEvidence` writes immutable evidence bundle.
- DW23-WF-12: Node `ApproveCatalogExposure` notifies catalog and SLO gate.

## Audit Events
- DW23-AUDIT-01: `WarehouseDpiaPacketDrafted` records catalog object id.
- DW23-AUDIT-02: `WarehouseDpiaEvidenceItemAdded` records evidence kind and hash.
- DW23-AUDIT-03: `WarehouseDpiaMandatoryEvidenceMissing` records missing kinds.
- DW23-AUDIT-04: `WarehouseDpiaCriticalRiskRaised` records risk and mitigation need.
- DW23-AUDIT-05: `WarehouseDpiaPacketApproved` records reviewer and bundle ref.
- DW23-AUDIT-06: `WarehouseDpiaPacketRejected` records rejection reason.
- DW23-AUDIT-07: `WarehouseDpiaPacketSuperseded` records replacement packet id.

## SLO Targets
- DW23-SLO-01: p50 packet build <= 250 ms.
- DW23-SLO-02: p95 packet build <= 1,200 ms.
- DW23-SLO-03: p99 packet build <= 3,000 ms.
- DW23-SLO-04: throughput >= 100 packet builds per minute.
- DW23-SLO-05: availability >= 99.9 percent for DPIA APIs.
- DW23-SLO-06: mandatory evidence detection accuracy must be 100 percent.
- DW23-SLO-07: review audit emission p95 <= 3 seconds.
- DW23-SLO-08: catalog exposure block propagation <= 5 seconds.

## Failure Modes + Recovery
- DW23-FAIL-01: Classification evidence missing; keep packet draft and block catalog exposure.
- DW23-FAIL-02: Vendor access export malformed; quarantine evidence item and require remap.
- DW23-FAIL-03: Critical risk lacks mitigation; reject approval and route workflow task.
- DW23-FAIL-04: Evidence bundle store unavailable; do not approve packet until bundle persists.
- DW23-FAIL-05: Reviewer lacks authority; Cedar denies and emits rejection event.
- DW23-FAIL-06: Packet superseded during review; abort review and redirect to replacement packet.

## Migration Notes
- DW23-MIG-01: Snowflake tags and grants are evidence, not policy authority.
- DW23-MIG-02: BigQuery policy tags require taxonomy id normalization.
- DW23-MIG-03: Redshift IAM role mappings require identity alias reconciliation.
- DW23-MIG-04: Databricks SQL ACLs require workspace and catalog context.
- DW23-MIG-05: Synapse Analytics Purview labels require confidence scoring.
- DW23-MIG-06: Firebolt grants require engine and database scope evidence.
- DW23-MIG-07: ClickHouse Cloud row policies require role and database evidence.
- DW23-MIG-08: Vertica roles require inherited grants expansion.
- DW23-MIG-09: Teradata Vantage profiles require account mapping.
- DW23-MIG-10: Yellowbrick grants require group mapping before import.

## Cross-Microservice Handoffs
- DW23-HANDOFF-01: Compliance receives approved DPIA packet and evidence bundle.
- DW23-HANDOFF-02: Catalog receives approval or block status.
- DW23-HANDOFF-03: Policy receives Cedar review context.
- DW23-HANDOFF-04: Audit-chain receives ADR-0263 DPIA events.
- DW23-HANDOFF-05: Tenant-admin receives privacy evidence status.
- DW23-HANDOFF-06: Workflow receives missing evidence and mitigation tasks.
- DW23-HANDOFF-07: Ontology receives classification and purpose projections.
- DW23-HANDOFF-08: Data-pipeline receives malformed vendor evidence remediation.
- DW23-HANDOFF-09: SLO gate receives catalog exposure readiness.
- DW23-HANDOFF-10: Marketplace receives DPIA approval status for governed shares.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-023-dpia-evidence-packet.md` matched `p99, SLO, financial`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-023-dpia-evidence-packet.md` matched `emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
