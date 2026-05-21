## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.15 vCPU, 512 MiB RAM, 12 GB storage, Valkey/PostgreSQL/outbound connections 3/2/6, scaling_dimension=per_user, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration and ADR-0248 cell criticality numbering.
- Why: 0.15 vCPU, 512 MiB, and 12 GB per tenant reflect deck storage plus bursty export and broadcast workloads.
- Rejected: copying another product manifest's capacity profile, because this service's PRD/capacity plan has a distinct load driver.
- Cost: Karpenter and FinOps now have a per-tenant sizing commitment that must be revised when the cited SLO/IP evidence changes.

### Block 2: dr
- Value: RTO 1800s, RPO 120s, active-active multi-region=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey, failover_runbook=runbooks/dr-failover.md, dr_tier=T2.
- ADR: ADR-0343 DR matrix and compliance-pack floor overlay.
- Why: 1800s RTO and 120s RPO satisfy HIPAA floor while preserving practical recovery of deck save and broadcast state.
- Rejected: using only the HIPAA floor mechanically; the selected values reflect the service's data-loss tolerance and recovery surface.
- Cost: Each cell must keep the declared backup substrates and runbook drillable, with audit-chain evidence on the next drill.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2.
- ADR: ADR-0338 runtime tiering plus ADR-0340 D-6 pod/runtime co-variance.
- Why: Slides is a first-party presentation authoring and delivery service. Broadcast and export paths touch tenant decks but do not execute tenant code or run edge packet paths, so ADR-0338 Tier 2 with ADR-0340 Tier-3 cell placement is the correct pair.
- Rejected: promoting the service to a stricter runtime tier without tenant-code or substrate evidence, because that would spend isolation budget without reducing the documented risk.
- Cost: Admission policy and nodepool placement now depend on the manifest declaration.

### Block 4: tenant_version_pinning
- Value: declared_versions=2026-05-21, 2026-02-20, 2025-11-20, default_version=2026-05-21, supported window=3 versions for at least 180 days, per-tenant pinning=true.
- ADR: ADR-0342 date-versioned public API and tenant pinning.
- Why: Slides owns public deck, broadcast, and export contracts, so date-versioned tenant pinning prevents presentation-client lockstep upgrades.
- Rejected: internal-only exemption, because the manifest declares public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: Breaking changes require a dated successor version, migration document, and sunset calendar entry.

### Block 5: consumes_upstream_oss
- Value: cedar, postgresql, valkey, opentelemetry, opentofu; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 OSS stewardship registry and class vocabulary.
- Why: Slides consumes registry-default Cedar/PostgreSQL/Valkey/OpenTelemetry/OpenTofu dependencies with no divergent stewardship class.
- Rejected: inline stewardship-class objects in the manifest, because specs/microservices/manifest-schema.json makes consumes_upstream_oss a registry-backed dep_name index.
- Cost: SBOM and CVE response evidence now ties this service to the registry owners and SLAs for each dependency.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/vpc@v1.
- ADR: ADR-0339 shared OpenTofu module invocation contract.
- Why: The service has deployable workload and substrate-adjacent wrapper needs that should reference shared module primitives instead of carrying unpinned bespoke IaC.
- Rejected: module_path/pin string objects from the dispatch prose, because manifest-schema.json is the authority and requires context, primitive, and version_pin.
- Cost: Wrapper IaC must remain thin and pinned to shared module releases; new primitives require schema/ADR-compatible registration.
