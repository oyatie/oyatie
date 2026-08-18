<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: cloud-data
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 0
  tier_references_scrubbed: 210
  ADR_0316_citations_replaced: 0
  cellular_criticality_preserved: 0
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `data/cloud-data/onboarding/data-engineer-first-week.md`
- `data/cloud-data/migration-playbooks/from-aurora-and-dynamodb.md`
- `data/cloud-data/feature-parity-matrix-2026-05-20.md`
- `data/cloud-data/REMEDIATION-NOTES-2026-05-21-tier-scrub.md`
- `crates/oya-cloud-data-domain/src/lib.rs`
- `crates/oya-cloud-data-kernel/src/data_service.rs`
- `crates/oya-cloud-resource-domain/src/lib.rs`

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 did not modify `PRD.md` because, at the time of this 2026-05-21 decision, the artifacts did not exist at their then-current location `microservices/cloud-data/` (`PRD.md` and `manifest.json`). They exist today at `data/cloud-data/`; this note records why the wave was blocked THEN and must not be read as a claim about the current tree. The D-3 instruction requires reading both artifacts and matching manifest-declared values before writing DR, capacity, sustainability, and API-version posture.

Values: no authoritative RTO/RPO, capacity_model, pod_runtime_tier, tenant_version_pinning, or OSS stewardship declarations were available. ADRs implicated once artifacts exist: ADR-0338, ADR-0340, ADR-0342 if public contracts exist, ADR-0343, ADR-0344, and ADR-0345; ADR-0337 likely applies only if cloud-data writes OLAP via the canonical Iceberg/data-warehouse path; ADR-0339 applies only if `iac/<context>/` wrappers exist. Alternatives considered: infer from cloud-storage/cloud-iac patterns or create placeholder sections; rejected because data-plane durability and OLAP ownership are doctrine-sensitive. Cost: missing first-class PRD/manifest blocks this wave for cloud-data.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.35 vCPU, 768 MiB RAM, 50 GB storage per active tenant; Valkey/Postgres/outbound connections 3/6/8; scaling_dimension=per_query; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Data substrate work scales with tenant queries, OLAP snapshots, and metadata writes; ADR-0337 pushes canonical analytical writes through Iceberg-backed paths.
- Rejected: Tier-2 was rejected because ADR-0340 reserves Tier-2 for capability services; this one directly touches tenant data substrate state.
- Cost: Commits the data plane to Iceberg/object-versioned snapshots and cross-region query metadata recovery.

### Block 2: dr
- Values: RTO=3600s, RPO=300s, multi_region_active_active=true, backup_substrate=postgres_wal_g+iceberg_snapshot+object_storage_versioned+clickhouse_iceberg_layered, failover_runbook=runbooks/data-substrate-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns tenant data substrate, analytical surfaces, query/storage control semantics; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=data/cloud-data/feature-parity-matrix-2026-05-20.md, crates/oya-cloud-data-domain/src/lib.rs, crates/oya-cloud-data-kernel/src/lib.rs.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Substrate-touching tenant-data service: it owns data-query and analytical storage semantics, so tenant data-plane access requires Kata/Cloud Hypervisor isolation rather than first-party runc placement.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: query and data-control APIs are public cloud surfaces where tenant workloads need deterministic version pinning.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql,valkey,kafka,iceberg,clickhouse; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared an empty invocation array because no service-local iac/<context>/ directory is present; this keeps ADR-0339 machine-decidable without inventing wrapper usage.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
