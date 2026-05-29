# meet remediation notes

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.28 vCPU, 768 MiB RAM, 4 GB storage per active tenant; connections valkey=4, postgres=3, outbound_http=8; scaling_dimension=per_user; cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: template-stamped values copied from another service; meet rejects Tier-3 pod runtime because the assigned service is room/signaling control plus product media orchestration, not north-south edge proxying.
- Cost: cell sizing and autoscaler budgets must reserve this per-tenant baseline before admitting more tenants.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey_cluster, audit_chain_merkle_seal, failover_runbook=runbooks/dr-failover.md.
- ADR: ADR-0343 plus compliance-pack floors; HIPAA/us-healthcare floors drive the 1h/5m baseline where applicable.
- Rejected: looser 24h PCI-only recovery because this service can serve healthcare or sensitive tenant workflows.
- Cost: warm cross-region replication and quarterly drill evidence are required for the declared runbook.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/meet/PRD.md, microservices/meet/IP-005-meeting-instance-and-livekit.md, microservices/meet/IP-007-screen-share-and-tracks.md, microservices/meet/runbooks/sfu-degraded.md.
- ADR: ADR-0338 runtime tiering; ADR-0340/ADR-0248 co-variance with cell_placement_class=Tier-3.
- Rejected: weaker runtime class that would contradict the documented tenant-data or first-party-app surface.
- Cost: runtime placement, nodepool capacity, and incident severity inherit this tier.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public APIs with per-tenant pinning.
- Rejected: internal-only exemption because this service has public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: at least three supported public API windows and migration docs for any future breaking change.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, openbao, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship overrides because the registry default class is sufficient for each declared upstream.
- Cost: SBOM and CVE-response evidence must trace this service to each upstream owner team.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper-only IaC because module reuse and pinning are the admission surface.
- Cost: module pins must be advanced deliberately when cloud-iac publishes new primitives.

