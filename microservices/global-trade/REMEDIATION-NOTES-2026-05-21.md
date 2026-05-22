<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: global-trade
  tenant_classes_directory_deleted: yes (already absent; rm blocked by local policy)
  manifest_tier_fields_removed: 4
  prd_md_tier_references_scrubbed: 480
  architecture_md_tier_references_scrubbed: 0
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 12
  total_lines_changed: 522 (untracked subtree; counted scrubbed reference/report lines)
  ADR_0316_citations_replaced_with_0329_0330_0331: 21
  cellular_tier_references_preserved: 9 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15J-final-cleanup
- Scope: F-BUCKET-2 final residue verification for ERP stamped docs.
- Renamed stale 2026-05-20 audit artifacts to 2026-05-21 and scrubbed retired B/S/G/P and `capability_tier` vocabulary under this service path.
- Preserved the tenant-class replacement model in IP metadata and audit references.
- Verification: assigned Wave 15J grep checks return zero non-remediation residue.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/global-trade

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 3600s, RPO p99 <= 300s, `multi_region_active_active=false`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`/`audit_chain_merkle_seal`, and failover runbook `microservices/global-trade/runbooks/regional-failover.md`. WHY: sanctions screening, export-control classification, customs declaration, denied-party evidence, and broker filing must remain legally explainable under regional loss. Alternative considered: active-active filing writes. Rejected because list-version and filing evidence need one promoted write cell. Cost: queued replay and promotion evidence before submission replay.
- Capacity model (ADR-0340): Values are manifest `0.13` vCPU, `384MiB` RAM, `10GB` storage, connections `{postgres:3,valkey:3,outbound_http:9}`, scaling `per_request`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: shipment cutoff and onboarding spikes arrive as screening, classification, and filing requests. Alternative considered: batch classification scaling. Rejected because denied-party checks are transaction-path gates. Cost: low-priority classification batches queue first.
- Sustainability and cost attribution (ADR-0344): Values are cost, CO2, and watt-hour fields on each audit row, no carbon routing for sanctions/export-control/customs/broker paths, and finops-portal visibility by screening lookup, HS classification, FTA preference, and filing. WHY: climate disclosure needs attribution, but trade compliance needs deterministic legal decisions. Alternative considered: carbon-aware routing for all list lookups. Rejected because legal timing and list-version freshness outrank provider carbon score. Cost: critical paths report emissions but do not optimize on them.
- API versioning posture (ADR-0342): Values are date carriers in header, URL, and proto3, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: broker, customs, sanctions-list, and ERP integrations need dated migration windows. Alternative considered: broker-specific version families. Rejected because ADR-0342 standardizes the public carrier. Cost: adapter compatibility testing across pinned tenants.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.13 vCPU, 384 MiB RAM, 10 GB storage, Valkey/Postgres/outbound connections 3/3/9, scaling_dimension=per_request, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.13 vCPU/384 MiB/10 GB fits sanctions and broker-filing request fan-out.
- Rejected: per_user was rejected because compliance screening load is transaction/request driven.
- Cost: Keeps outbound HTTP headroom for broker and screening integrations.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Five-minute RPO is chosen because embargo and denied-party evidence must replay cleanly.
- Rejected: Backup-only RPO was rejected because customs filings need audit-chain continuity.
- Cost: Requires audit-chain anchored failover and replay validation.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/global-trade/ARCHITECTURE.md, microservices/global-trade/IP-017-denied-party-screening-lookup-with-cedar-consent.md, microservices/global-trade/IP-019-broker-edi-ingestion-cusdec.md, microservices/global-trade/IP-022-embargo-event-audit-chain-anchor.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because global-trade is a regulated application, not a shared substrate service.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Public trade and broker filing contracts require tenant date-version pinning.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, oyatie-as-cloud-provider/audit-chain-sink@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Audit-chain-sink is selected for embargo and denied-party evidence recovery.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
