# warehouse remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/warehouse

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): Values mirror manifest `dr`: RTO 7200s, RPO 900s, `multi_region_active_active=false`, `dr_tier=T3`, `replication_shape=active-passive-cross-region-continuous`, `failover_runbook=runbooks/regional-failover.md`. Alternative considered: a generic 4h/15m warehouse floor. Rejected because manifest already declares a stricter 2h RTO. Cost: continuous active-passive replication costs more than backup-restore but protects dock and stock-state continuity.
- Capacity model (ADR-0340): Values mirror manifest `capacity_model`: 0.15 CPU, 512 MiB RAM, 14 GiB storage, connections Valkey 4/Postgres 4/outbound HTTP 8, `scaling_dimension=per_message`, `cell_placement_class=Tier-3`, `pod_runtime_tier=2`. Alternative considered: one shared ERP capacity envelope. Rejected because dock, wave, yard, and labor bursts are message-heavy and manifest-specific. Cost: message queues and queue-split logic must scale from the manifest baseline.
- Sustainability and cost attribution (ADR-0344): Values require `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on each warehouse audit row, with carbon routing only for batch wave planning, export, replay, and non-urgent yard optimization; manifest `sustainability_emission_model` remains absent. Alternative considered: carbon-aware routing for every job. Rejected because fulfillment recovery cannot wait for a lower-carbon window. Cost: finops rollups must support facility, wave, bounded-context, and future manifest emission-model attribution.
- API versioning posture (ADR-0342): Values set public carrier triplet, SDK semver, last 3 versions for at least 180 days, paid-tenant WMS pinning, and ADR-0145 internal mesh exemption. Alternative considered: URL-only versioning. Rejected because warehouse events and proto3 consumers also need deterministic version selection. Cost: contract fixtures and generated SDKs must remain multi-version during tenant migrations.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.15 vCPU, 512 MiB RAM, 14 GB storage, Valkey/Postgres/outbound connections 4/4/8, scaling_dimension=per_message, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.15 vCPU/512 MiB/14 GB accounts for picking-wave optimization and message-driven warehouse events.
- Rejected: per_query was rejected because throughput is dominated by inbound/outbound event flow.
- Cost: Commits event-topic and Valkey capacity for bursty wave release.

### Block 2: dr
- Value: RTO 7200s, RPO 900s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Two-hour RTO keeps warehouse execution inside a shift recovery window.
- Rejected: Cold restore was rejected because cross-dock and hazmat holds are operationally time-bound.
- Cost: Requires continuous event replay and warm cache rebuild during failover.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/warehouse/ARCHITECTURE.md, microservices/warehouse/IP-017-picking-wave-optimization-tsp-steiner.md, microservices/warehouse/IP-018-cross-docking-workflow.md, microservices/warehouse/IP-024-hazmat-segregation.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because warehouse is a product application despite high operational criticality.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Warehouse REST, event, and proto contracts are public enough for tenant pinning.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, aws-guest/event-topic@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Event-topic primitive is included because queue continuity drives recovery.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
