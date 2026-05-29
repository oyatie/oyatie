## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.4 vCPU, 1024 MiB RAM, 51200 GB storage, Valkey/PostgreSQL/outbound connections 4/4/8, scaling_dimension=per_request, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration and ADR-0248 cell criticality numbering.
- Why: 0.4 vCPU, 1024 MiB, and 50 TB per tenant follow the Drive capacity model's file-byte dominance and mixed upload/download/preview path.
- Rejected: copying another product manifest's capacity profile, because this service's PRD/capacity plan has a distinct load driver.
- Cost: Karpenter and FinOps now have a per-tenant sizing commitment that must be revised when the cited SLO/IP evidence changes.

### Block 2: dr
- Value: RTO 900s, RPO 60s, active-active multi-region=true, backup_substrate=postgres_wal_g, object_storage_versioned, seaweedfs_replicated, valkey, failover_runbook=runbooks/dr-failover.md, dr_tier=T1.
- ADR: ADR-0343 DR matrix and compliance-pack floor overlay.
- Why: 900s RTO and 60s RPO are tighter than the HIPAA floor because tenant files and share-link permissions are collaboration-critical records.
- Rejected: using only the HIPAA floor mechanically; the selected values reflect the service's data-loss tolerance and recovery surface.
- Cost: Each cell must keep the declared backup substrates and runbook drillable, with audit-chain evidence on the next drill.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2.
- ADR: ADR-0338 runtime tiering plus ADR-0340 D-6 pod/runtime co-variance.
- Why: Drive is a first-party application service that stores tenant files, share links, previews, and DLP scan state. It does not execute tenant-customer code or own foundation substrate keys, so ADR-0338 keeps it on the Tier 2 application runtime while ADR-0340 places the workload in a Tier-3 application cell.
- Rejected: promoting the service to a stricter runtime tier without tenant-code or substrate evidence, because that would spend isolation budget without reducing the documented risk.
- Cost: Admission policy and nodepool placement now depend on the manifest declaration.

### Block 4: tenant_version_pinning
- Value: declared_versions=2026-05-21, 2026-02-20, 2025-11-20, default_version=2026-05-21, supported window=3 versions for at least 180 days, per-tenant pinning=true.
- ADR: ADR-0342 date-versioned public API and tenant pinning.
- Why: Drive exposes public OpenAPI, AsyncAPI, and proto contracts at the api-gateway boundary, so per-tenant pinning is required.
- Rejected: internal-only exemption, because the manifest declares public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: Breaking changes require a dated successor version, migration document, and sunset calendar entry.

### Block 5: consumes_upstream_oss
- Value: cedar, postgresql, valkey, opentelemetry, opentofu; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 OSS stewardship registry and class vocabulary.
- Why: Drive directly consumes Valkey session/cache state, PostgreSQL metadata, Cedar permissions, OpenTelemetry evidence, and OpenTofu cell wrappers.
- Rejected: inline stewardship-class objects in the manifest, because specs/microservices/manifest-schema.json makes consumes_upstream_oss a registry-backed dep_name index.
- Cost: SBOM and CVE response evidence now ties this service to the registry owners and SLAs for each dependency.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/vpc@v1.
- ADR: ADR-0339 shared OpenTofu module invocation contract.
- Why: The service has deployable workload and substrate-adjacent wrapper needs that should reference shared module primitives instead of carrying unpinned bespoke IaC.
- Rejected: module_path/pin string objects from the dispatch prose, because manifest-schema.json is the authority and requires context, primitive, and version_pin.
- Cost: Wrapper IaC must remain thin and pinned to shared module releases; new primitives require schema/ADR-compatible registration.
