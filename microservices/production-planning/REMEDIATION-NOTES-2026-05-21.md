# production-planning remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/production-planning/IP-009-usecase-layer-for-capacity-calendar.md
- microservices/production-planning/IP-013-adapter-integrations-for-production-planning.md
- microservices/production-planning/IP-014-rest-grpc-and-worker-surfaces-for-production-planning.md
- microservices/production-planning/IP-015-integration-tests-for-production-planning.md

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 7200s, RPO p99 <= 900s, `multi_region_active_active=false`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`, and failover runbook `microservices/production-planning/runbooks/regional-failover.md`. WHY: MRP, finite scheduling, production-order release, and MES handoff evidence must not split during regional loss. Alternative considered: replay production-order commits in both regions. Rejected because that risks conflicting shop-floor schedules. Cost: promoted-cell evidence gate before write replay.
- Capacity model (ADR-0340): Values are manifest `0.14` vCPU, `512MiB` RAM, `10GB` storage, connections `{postgres:4,valkey:3,outbound_http:7}`, scaling `per_workflow_run`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: MRP explosions and finite scheduling are workflow bursts. Alternative considered: simple `per_request` scaling. Rejected because IP-021 finite scheduling and IP-024 MES sync need queue-shaped capacity. Cost: queue isolation and workflow-run metering.
- Sustainability and cost attribution (ADR-0344): Values are cost, CO2, and watt-hour fields on each audit row, carbon-aware routing for MRP simulations and previews only, and finops-portal visibility by plant and MES sync stream. WHY: manufacturing planning needs emissions attribution without delaying plant-state reconciliation. Alternative considered: carbon route every workload. Rejected because MES drift can become operationally unsafe. Cost: split routing policy between async and commit paths.
- API versioning posture (ADR-0342): Values are date carriers in header, URL, and proto3, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: MES and SAP PP migrations need stable integration dates. Alternative considered: single `/v1` carrier. Rejected because it hides breaking public contract dates. Cost: adapter compatibility matrix across plant cutovers.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.14 vCPU, 512 MiB RAM, 10 GB storage, Valkey/Postgres/outbound connections 3/4/7, scaling_dimension=per_workflow_run, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.14 vCPU/512 MiB/10 GB reserves more RAM for finite scheduling and MRP workflow bursts.
- Rejected: per_request was rejected because the expensive path is the scheduled planning run.
- Cost: Commits cells to batch-worker capacity instead of only API pod capacity.

### Block 2: dr
- Value: RTO 7200s, RPO 900s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Two-hour recovery keeps production release and MRP handoff inside a shift-level disruption window.
- Rejected: T4 cold restore was rejected because shop-floor release latency is operationally visible.
- Cost: Requires continuous WAL and event-topic replay for planning-run continuity.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/production-planning/ARCHITECTURE.md, microservices/production-planning/IP-016-mrp-explosion-to-supply-chain-planning-handoff.md, microservices/production-planning/IP-021-capacity-leveling-finite-scheduling-forward-backward-bottleneck.md, microservices/production-planning/IP-024-mes-handshake-bidirectional-event-flow-isa-95.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because MES/shop-floor data is handled by first-party application code, not substrate ownership.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: External planning APIs and events are tenant-visible and must honor date-version pinning.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, aws-guest/batch-worker-autoscaler@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Batch autoscaler invocation is included because planning-run load is not captured by the common API primitives.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
