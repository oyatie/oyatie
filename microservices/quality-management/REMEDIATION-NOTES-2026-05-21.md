# quality-management remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/quality-management

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 3600s, RPO p99 <= 300s, `multi_region_active_active=false`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`/`audit_chain_merkle_seal`, and failover runbook `microservices/quality-management/runbooks/regional-failover.md`. WHY: inspection lots, CoA approval, quality holds, calibration gates, and Part 11 signatures must recover without duplicate regulated release decisions. Alternative considered: allow multi-region writes during failover. Rejected because signed quality records need a single promoted write cell. Cost: replay queue and cell-promotion evidence.
- Capacity model (ADR-0340): Values are manifest `0.1` vCPU, `384MiB` RAM, `8GB` storage, connections `{postgres:3,valkey:3,outbound_http:6}`, scaling `per_capability`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: regulated quality work scales by enabled inspection, hold, CoA, calibration, and audit-evidence capability. Alternative considered: `per_request` scaling. Rejected because capability activation better matches tenant pack overlays. Cost: audit-export queues may defer under load.
- Sustainability and cost attribution (ADR-0344): Values are cost, CO2, and watt-hour fields on each audit row, no carbon routing for regulated release/e-signature paths, and finops-portal visibility by quality record, CoA, calibration, and audit export. WHY: emissions must be reportable without weakening quality release controls. Alternative considered: carbon-aware routing for every quality operation. Rejected because hold release and Part 11 signatures prioritize provenance and policy. Cost: only async exports get carbon optimization.
- API versioning posture (ADR-0342): Values are date carriers in header, URL, and proto3, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: validation batches and evidence packs need reproducible contract dates. Alternative considered: validation-pack-specific API versions. Rejected because ADR-0342 makes carrier date the shared public contract. Cost: migration support for regulated tenants.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.1 vCPU, 384 MiB RAM, 8 GB storage, Valkey/Postgres/outbound connections 3/3/6, scaling_dimension=per_capability, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.10 vCPU/384 MiB/8 GB reflects moderate inspection load plus audit-evidence writes.
- Rejected: per_query was rejected because capability enablement, not ad hoc reads, drives steady demand.
- Cost: Adds audit-chain sink capacity and Postgres storage for e-signature evidence.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Five-minute RPO is required because audit evidence, quality holds, and HACCP checks cannot be loosely reconstructed.
- Rejected: 15-minute RPO was rejected as too loose for regulated quality evidence chains.
- Cost: Requires audit-chain seal preservation in addition to ordinary database restore.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/quality-management/ARCHITECTURE.md, microservices/quality-management/IP-006-domain-layer-for-audit-evidence.md, microservices/quality-management/IP-020-21-cfr-part-11-esignature-integration-on-quality-records.md, microservices/quality-management/IP-021-haccp-critical-control-point-monitoring.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because it records regulated evidence but does not provide a shared tenant-data substrate.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Quality APIs/events are public contract surfaces for inspections and holds.
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
- Why: Audit-chain-sink is selected because ADR-0343 recovery depends on sealed evidence continuity.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
