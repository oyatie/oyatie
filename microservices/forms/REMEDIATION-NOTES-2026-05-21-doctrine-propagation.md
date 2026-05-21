## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.08 vCPU, 256 MiB RAM, 5 GB storage, Valkey/PostgreSQL/outbound connections 2/2/5, scaling_dimension=per_request, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration and ADR-0248 cell criticality numbering.
- Why: 0.08 vCPU and 256 MiB per tenant reflect high-throughput request/validation paths with small durable response records.
- Rejected: copying another product manifest's capacity profile, because this service's PRD/capacity plan has a distinct load driver.
- Cost: Karpenter and FinOps now have a per-tenant sizing commitment that must be revised when the cited SLO/IP evidence changes.

### Block 2: dr
- Value: RTO 900s, RPO 60s, active-active multi-region=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey, audit_chain_merkle_seal, failover_runbook=runbooks/dr-failover.md, dr_tier=T1.
- ADR: ADR-0343 DR matrix and compliance-pack floor overlay.
- Why: 900s RTO and 60s RPO are chosen because healthcare/PII submissions and audit-chain seals are tenant records, not disposable telemetry.
- Rejected: using only the HIPAA floor mechanically; the selected values reflect the service's data-loss tolerance and recovery surface.
- Cost: Each cell must keep the declared backup substrates and runbook drillable, with audit-chain evidence on the next drill.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2.
- ADR: ADR-0338 runtime tiering plus ADR-0340 D-6 pod/runtime co-variance.
- Why: Forms is a first-party application that collects tenant response data and PII, but it does not execute tenant-customer code. Because submissions and audit events are product-owned workloads, ADR-0338 Tier 2 and ADR-0340 Tier-3 placement are the valid co-variant declarations.
- Rejected: promoting the service to a stricter runtime tier without tenant-code or substrate evidence, because that would spend isolation budget without reducing the documented risk.
- Cost: Admission policy and nodepool placement now depend on the manifest declaration.

### Block 4: tenant_version_pinning
- Value: declared_versions=2026-05-21, 2026-02-20, 2025-11-20, default_version=2026-05-21, supported window=3 versions for at least 180 days, per-tenant pinning=true.
- ADR: ADR-0342 date-versioned public API and tenant pinning.
- Why: Forms exposes public respondent, analytics, and event contracts; per-tenant pinning prevents breaking embedded forms during migrations.
- Rejected: internal-only exemption, because the manifest declares public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: Breaking changes require a dated successor version, migration document, and sunset calendar entry.

### Block 5: consumes_upstream_oss
- Value: cedar, postgresql, valkey, opentelemetry, opentofu; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 OSS stewardship registry and class vocabulary.
- Why: Forms uses the common registry OSS stack; audit-chain backup substrate is a DR substrate, not a separate local OSS override.
- Rejected: inline stewardship-class objects in the manifest, because specs/microservices/manifest-schema.json makes consumes_upstream_oss a registry-backed dep_name index.
- Cost: SBOM and CVE response evidence now ties this service to the registry owners and SLAs for each dependency.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/vpc@v1.
- ADR: ADR-0339 shared OpenTofu module invocation contract.
- Why: The service has deployable workload and substrate-adjacent wrapper needs that should reference shared module primitives instead of carrying unpinned bespoke IaC.
- Rejected: module_path/pin string objects from the dispatch prose, because manifest-schema.json is the authority and requires context, primitive, and version_pin.
- Cost: Wrapper IaC must remain thin and pinned to shared module releases; new primitives require schema/ADR-compatible registration.
