## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references in `microservices/cloud-billing-tax/`.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 did not modify `PRD.md` because `microservices/cloud-billing-tax/PRD.md` and `microservices/cloud-billing-tax/manifest.json` are absent. The D-3 instruction requires reading both artifacts and matching manifest-declared DR/capacity/runtime values; inventing those values would violate the manifest-PRD consistency rule.

Values: no authoritative RTO/RPO, capacity_model, pod_runtime_tier, tenant_version_pinning, or OSS stewardship declarations were available. ADRs implicated once the missing artifacts exist: ADR-0338, ADR-0340, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; ADR-0339 only if `iac/<context>/` wrappers are added; ADR-0337 only if tax writes OLAP through the warehouse path. Alternatives considered: create a generic PRD, infer from cloud-billing, or append only this blocker note. The generic/inferred paths were rejected because cloud-billing-tax has jurisdiction-specific tax semantics and cannot safely inherit billing SLOs or capacity. Cost: D-2/D-3 must first author the manifest and PRD before doctrine propagation can be truthfully completed.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.08 vCPU, 192 MiB RAM, 2 GB storage per active tenant; Valkey/Postgres/outbound connections 2/2/6; scaling_dimension=per_request; cell_placement_class=Tier-3.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=2.
- Why: In-process Cedar tax engine and hot catalog cache scale by tax-calculation requests; batch and filing paths are bounded by paid tenant workload tiers.
- Rejected: Tier-2 was rejected because this is a product application extension of billing, not a platform capability substrate.
- Cost: Commits the tax catalog to multi-region cache warmup and WAL-G/object-versioned catalog restore drills.

### Block 2: dr
- Values: RTO=300s, RPO=60s, multi_region_active_active=true, backup_substrate=postgres_wal_g+object_storage_versioned+audit_chain_merkle_seal, failover_runbook=runbooks/tax-catalog-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns tax calculation, rate-card catalog, exemption certificate checks, filing artifact generation; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/cloud-billing-tax/README.md, microservices/cloud-billing-tax/performance-benchmark-numbers-2026-05-20.md, microservices/cloud-billing-tax/feature-parity-matrix-2026-05-20.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-3.
- Why: First-party tax application service: it evaluates Oyatie-owned tax rules and rate catalogs for billing flows, with no tenant-customer code execution and no direct foundation substrate ownership.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: tax calculation and filing APIs need per-tenant pinning because rate-card and exemption semantics can change by date.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql,valkey,cedar; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
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
