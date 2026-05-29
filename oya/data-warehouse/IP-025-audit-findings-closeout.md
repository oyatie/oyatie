---
doc_class: IP
ip_id: IP-025
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0251, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0321]
journey_ref: J89-uk-aadc-minor-ux-adaptation
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-025: Audit Findings Closeout

## Context
- DW25-CTX-01: This IP closes data-warehouse audit findings with evidence, owners, residual risk, and promotion effects.
- DW25-CTX-02: Snowflake migration findings close only after local policy, catalog, and settlement evidence replaces vendor evidence.
- DW25-CTX-03: BigQuery migration findings close only after project and dataset sprawl is reconciled.
- DW25-CTX-04: Redshift migration findings close only after datashare and snapshot exposure controls pass.
- DW25-CTX-05: Databricks SQL findings close only after token, ACL, and warehouse warm-pool evidence passes.
- DW25-CTX-06: Synapse Analytics findings close only after linked-service and Purview evidence is mapped.
- DW25-CTX-07: Firebolt findings close only after engine grants and cost controls pass.
- DW25-CTX-08: ClickHouse Cloud findings close only after row policy and replica freshness controls pass.
- DW25-CTX-09: Vertica findings close only after role inheritance and projection freshness evidence passes.
- DW25-CTX-10: Teradata Vantage findings close only after account-sharing and workload controls pass.
- DW25-CTX-11: Yellowbrick findings close only after cluster network and queue controls pass.
- DW25-CTX-12: Closeout is not a note; it is a state transition with audit events and verification evidence.
- DW25-CTX-13: Findings can close as remediated, risk accepted, duplicate, superseded, or not applicable.
- DW25-CTX-14: Risk acceptance requires expiry, owner, compensating control, and tenant-visible record.
- DW25-CTX-15: Closed findings feed SLO promotion, DPIA packets, and release readiness.

## Data Model Deltas
- DW25-DDL-01: Add audit finding table.
```sql
CREATE TABLE warehouse_audit_findings (
    finding_id UUID PRIMARY KEY,
    tenant_id UUID,
    finding_key TEXT NOT NULL,
    title TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low','medium','high','critical')),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('internal_audit','dpia','threat_model','chaos_drill','vendor_migration','slo_gate')),
    source_ref TEXT NOT NULL,
    current_status TEXT NOT NULL CHECK (current_status IN ('open','in_remediation','closed_remediated','closed_risk_accepted','closed_duplicate','closed_superseded','closed_not_applicable')),
    owner_team TEXT NOT NULL,
    due_at TIMESTAMPTZ,
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX wh_audit_finding_key_idx ON warehouse_audit_findings(tenant_id, finding_key);
```
- DW25-DDL-02: Add closeout evidence table.
```sql
CREATE TABLE warehouse_audit_finding_closeout_evidence (
    evidence_id UUID PRIMARY KEY,
    finding_id UUID NOT NULL REFERENCES warehouse_audit_findings(finding_id) ON DELETE CASCADE,
    evidence_kind TEXT NOT NULL CHECK (evidence_kind IN ('test_result','policy_decision','audit_event','dpia_packet','threat_control','chaos_run','slo_gate','risk_acceptance')),
    evidence_ref TEXT NOT NULL,
    evidence_hash BYTEA NOT NULL,
    accepted_by_principal_id UUID,
    accepted_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
);
CREATE INDEX wh_audit_closeout_evidence_finding_idx ON warehouse_audit_finding_closeout_evidence(finding_id, evidence_kind);
```
- DW25-RUST-01: Audit finding type.
```rust
pub struct WarehouseAuditFinding {
    pub finding_id: AuditFindingId,
    pub tenant_id: Option<TenantId>,
    pub finding_key: FindingKey,
    pub title: FindingTitle,
    pub severity: Severity,
    pub source: FindingSource,
    pub current_status: FindingStatus,
    pub owner_team: OwnerTeam,
    pub due_at: Option<DateTime<Utc>>,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
    pub closed_at: Option<DateTime<Utc>>,
}
```
- DW25-RUST-02: Closeout evidence type.
```rust
pub struct WarehouseAuditFindingCloseoutEvidence {
    pub evidence_id: CloseoutEvidenceId,
    pub finding_id: AuditFindingId,
    pub evidence_kind: CloseoutEvidenceKind,
    pub evidence_ref: EvidenceRef,
    pub evidence_hash: Sha256Digest,
    pub accepted_by_principal_id: Option<PrincipalId>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}
```
- DW25-RUST-03: `FindingStatus::ClosedRiskAccepted` requires expiry and accepter.
- DW25-RUST-04: Critical findings cannot close not-applicable without compliance approval.
- DW25-RUST-05: Duplicate findings retain link to canonical finding through evidence ref.

## API Endpoints
- DW25-API-01: REST close finding.
```http
POST /v1/data-warehouse/audit-findings/{finding_id}:close
Idempotency-Key: wh-audit-close-025
Content-Type: application/json

{"target_status":"closed_remediated","closeout_summary":"row policy bypass fixed by Cedar context enforcement and chaos drill passed","evidence_refs":["evidence://threat-control/WH-C-CEDAR-ROW-SCOPE","evidence://chaos/01JWH22RUN","audit://WarehouseThreatControlTestPassed/01J"]}
```
- DW25-API-02: REST accept risk.
```http
POST /v1/data-warehouse/audit-findings/{finding_id}:accept-risk
Content-Type: application/json

{"accepted_by_principal_id":"01JRISKOWNER025","expires_at":"2026-08-20T00:00:00Z","compensating_control_ref":"WH-C-COMPENSATING-025","tenant_visible_reason":"vendor history export gap bounded to read-only evidence"}
```
- DW25-API-03: gRPC closeout.
```proto
rpc CloseWarehouseAuditFinding(CloseWarehouseAuditFindingRequest) returns (CloseWarehouseAuditFindingResponse);
message CloseWarehouseAuditFindingRequest {
  string finding_id = 1;
  string target_status = 2;
  repeated string evidence_refs = 3;
  string closeout_summary = 4;
}
```
- DW25-API-04: AsyncAPI event.
```yaml
warehouse.audit_finding.closed.v1:
  payload:
    finding_id: 01JWH25FINDING
    target_status: closed_remediated
    evidence_count: 3
    audit_event_class: WarehouseAuditFindingClosed
```
- DW25-API-05: REST returns 422 when mandatory evidence kinds are missing.
- DW25-API-06: gRPC returns `PERMISSION_DENIED` when owner team does not match principal authority.
- DW25-API-07: Async risk-accepted events include expiry and compensating control ref.

## Cedar Policy Hooks
- DW25-CEDAR-01: principal = `Oyatie::Principal::"audit_owner:{principal_id}"`.
- DW25-CEDAR-02: action = `Oyatie::Action::"warehouse_audit_finding_close"`.
- DW25-CEDAR-03: resource = `Oyatie::WarehouseAuditFinding::"{finding_id}"`.
- DW25-CEDAR-04: context.owner_team must match finding owner team unless compliance override exists.
- DW25-CEDAR-05: context.target_status must be allowed for finding severity.
- DW25-CEDAR-06: context.evidence_kinds_complete must be true for remediated closure.
- DW25-CEDAR-07: context.risk_acceptance_expiry must be present for risk accepted closure.
- DW25-CEDAR-08: context.compensating_control_active must be true for risk accepted closure.
- DW25-CEDAR-09: context.audit_event_class must equal `WarehouseAuditFindingClosed`.
- DW25-CEDAR-10: deny if principal lacks `warehouse.audit.finding.close`.

## Ontology Projection
- DW25-ONTO-01: Snowflake audit finding id -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-02: BigQuery recommender finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-03: Redshift security finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-04: Databricks SQL audit log finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-05: Synapse Analytics security recommendation -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-06: Firebolt access finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-07: ClickHouse Cloud security finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-08: Vertica audit issue -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-09: Teradata Vantage audit exception -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-10: Yellowbrick security finding -> `WarehouseAuditFinding.source_ref`.
- DW25-ONTO-11: Vendor remediation note -> `WarehouseAuditFindingCloseoutEvidence.evidence_ref`.
- DW25-ONTO-12: Local closeout status -> `WarehouseAuditFinding.current_status`.

## Workflow Steps
- DW25-WF-01: Node `LoadFinding` reads current finding and severity.
- DW25-WF-02: Node `CollectEvidence` verifies evidence refs and hashes.
- DW25-WF-03: Node `CheckMandatoryKinds` ensures required closeout evidence exists.
- DW25-WF-04: Branch `MissingEvidence` rejects closeout and returns missing kinds.
- DW25-WF-05: Branch `RiskAccepted` validates expiry and compensating control.
- DW25-WF-06: Node `EvaluatePolicy` runs Cedar for target status.
- DW25-WF-07: Branch `PolicyDenied` leaves finding open and emits denial.
- DW25-WF-08: Node `PersistCloseout` updates status and close timestamp.
- DW25-WF-09: Node `SealEvidence` freezes accepted evidence rows.
- DW25-WF-10: Node `EmitAudit` emits `WarehouseAuditFindingClosed`.
- DW25-WF-11: Node `NotifySloGate` unblocks or keeps promotion blocked.
- DW25-WF-12: Node `NotifyCompliance` updates tenant and internal audit views.

## Audit Events
- DW25-AUDIT-01: `WarehouseAuditFindingOpened` records source and severity.
- DW25-AUDIT-02: `WarehouseAuditFindingEvidenceAttached` records evidence kind and hash.
- DW25-AUDIT-03: `WarehouseAuditFindingCloseoutRejected` records missing evidence or denial.
- DW25-AUDIT-04: `WarehouseAuditFindingRiskAccepted` records expiry and compensating control.
- DW25-AUDIT-05: `WarehouseAuditFindingClosed` records final status and evidence count.
- DW25-AUDIT-06: `WarehouseAuditFindingReopened` records stale evidence or expired risk.
- DW25-AUDIT-07: `WarehouseAuditFindingPromotionUnblocked` records affected gate ids.

## SLO Targets
- DW25-SLO-01: p50 closeout validation <= 80 ms.
- DW25-SLO-02: p95 closeout validation <= 350 ms.
- DW25-SLO-03: p99 closeout validation <= 900 ms.
- DW25-SLO-04: throughput >= 200 closeout validations per minute.
- DW25-SLO-05: availability >= 99.9 percent for audit finding APIs.
- DW25-SLO-06: promotion unblock propagation p95 <= 10 seconds.
- DW25-SLO-07: expired risk acceptance reopen latency <= 60 seconds.
- DW25-SLO-08: closed finding without evidence count must be 0.

## Failure Modes + Recovery
- DW25-FAIL-01: Evidence ref is missing or hash mismatch; reject closeout and keep finding open.
- DW25-FAIL-02: Risk acceptance expiry is absent; reject risk closure.
- DW25-FAIL-03: Compensating control expires; reopen finding and block promotions.
- DW25-FAIL-04: Owner team mismatch; Cedar denies and records policy denial.
- DW25-FAIL-05: SLO gate handoff fails; retry outbox while closeout remains authoritative.
- DW25-FAIL-06: Vendor finding later changes severity; supersede local finding and reopen if required.

## Migration Notes
- DW25-MIG-01: Snowflake security findings close only against local Cedar and sidecar evidence.
- DW25-MIG-02: BigQuery recommender findings close only after local tenant boundary controls pass.
- DW25-MIG-03: Redshift snapshot and datashare findings close after export controls pass.
- DW25-MIG-04: Databricks SQL token findings close after credential sidecar evidence passes.
- DW25-MIG-05: Synapse Analytics linked-service findings close after adapter scope evidence passes.
- DW25-MIG-06: Firebolt engine grant findings close after capacity and policy controls pass.
- DW25-MIG-07: ClickHouse Cloud row policy findings close after Cedar row-scope evidence passes.
- DW25-MIG-08: Vertica role inheritance findings close after policy drift controls pass.
- DW25-MIG-09: Teradata Vantage shared account findings close after principal alias controls pass.
- DW25-MIG-10: Yellowbrick network findings close after endpoint exposure controls pass.

## Cross-Microservice Handoffs
- DW25-HANDOFF-01: Compliance receives closed finding state and evidence bundle.
- DW25-HANDOFF-02: SLO gate receives unblock or continued-block decisions.
- DW25-HANDOFF-03: Audit-chain receives ADR-0263 finding events.
- DW25-HANDOFF-04: Policy receives Cedar closeout decision evidence.
- DW25-HANDOFF-05: Security receives reopened and expired-risk notices.
- DW25-HANDOFF-06: Tenant-admin receives tenant-visible risk acceptance summaries.
- DW25-HANDOFF-07: Workflow receives remediation and reopened-finding tasks.
- DW25-HANDOFF-08: DPIA packet receives closeout evidence references.
- DW25-HANDOFF-09: Threat model receives control evidence updates.
- DW25-HANDOFF-10: Release controller receives promotion readiness signal.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-025-audit-findings-closeout.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-025-audit-findings-closeout.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
