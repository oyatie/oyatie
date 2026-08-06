<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: cloud-storage
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 0
  tier_references_scrubbed: 473
  ADR_0316_citations_replaced: 2
  cellular_criticality_preserved: 0
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references in `microservices/cloud-storage/`.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now sets manifest RTO 3600s/RPO 300s for storage metadata and replication intent, with `runbooks/storage-replication-failover.md` as the failover reference. Alternative considered: object-store durability alone; rejected because control-plane metadata and retention evidence are tenant-visible recovery state. Cost: storage-region failover tests must prove 300s RPO across replicated metadata.
- Capacity model (ADR-0340): PRD now records manifest values: 0.3 vCPU, 768 MiB RAM, 100 GiB storage allowance, 3 Valkey, 3 metadata-store/Postgres, 8 outbound slots, `per_request` scaling, Tier-1 placement, and 2-to-24 API scaling. Alternative considered: `per_capability`; rejected because hot-path cost follows object operation volume while bytes remain quota-governed. Cost: bucket/prefix sharding and quota admission must be explicit before large paid tenants onboard.
- Sustainability + cost attribution (ADR-0344): PRD now requires object, lifecycle, restore, lock, quota, KMS, and backup actions to emit cost/carbon/energy audit fields, while online IO and emergency restores ignore carbon routing. Alternative considered: carbon-aware storage placement for all writes; rejected because compliance and restore deadlines override carbon scheduling. Cost: FinOps must expose bucket, prefix, and storage-class rollups.
- API versioning posture (ADR-0342): PRD now requires native date-version carriers, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption while provider-compatible adapters translate into the native contract. Alternative considered: S3 API compatibility as the only version model; rejected because Oyatie-native storage semantics span object, block, file, archive, KMS, and billing. Cost: adapter compatibility profiles must be maintained beside native SDK versions.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.3 vCPU, 768 MiB RAM, 100 GB storage per active tenant; Valkey/Postgres/outbound connections 3/3/8; scaling_dimension=per_request; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Object GET/PUT, block control, bucket metadata, replication, and lifecycle work scale by storage request volume and tenant capacity commitments.
- Rejected: Tier-3 application placement was rejected because storage is substrate for tenant data, not a first-party application surface.
- Cost: Commits storage metadata and object payload paths to versioned object replication plus warm cross-region restore capacity.

### Block 2: dr
- Values: RTO=3600s, RPO=300s, multi_region_active_active=true, backup_substrate=object_storage_versioned+seaweedfs_replicated+postgres_wal_g, failover_runbook=runbooks/storage-replication-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns object and block storage APIs, tenant bucket/volume metadata, replication and retention controls; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/cloud-storage/performance-benchmark-numbers-2026-05-20.md, microservices/cloud-storage/feature-parity-matrix-2026-05-20.md, crates/oya-cloud-storage-object-api/src/lib.rs.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Tenant data-plane storage substrate: object and block APIs handle tenant data, bucket/volume metadata, KMS-wrapped writes, and replication controls, requiring ADR-0338 Tier-1 isolation.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: object/block APIs are tenant-facing cloud contracts and must support pinned versions for SDK and workload compatibility.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql,valkey,iceberg,clickhouse; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
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
