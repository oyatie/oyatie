<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: ontology
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 42
  ADR_0316_citations_replaced: 2
  cellular_criticality_preserved: 1
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/ontology/onboarding/ontology-engineer-first-week.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now records manifest RTO 900 s / RPO 60 s, cites HIPAA/SOC2/ISO floors, names `runbooks/postgres-citus-rebalance.md`, and states active-active per ADR-0343. Alternative rejected: projection-only recovery, because Object Type writes and Citus placement also need DR. Cost: warm DR pair storage and projection rebuild capacity.
- Capacity model: PRD now binds manifest values 0.24 vCPU, 512 MiB RAM, 10 GB storage, connections `{valkey:3, postgres:5, outbound_http:6}`, per-query scaling, Tier-1 placement, 1M object instances/tenant, and ADR-0338 Tier-1 runtime to ADR-0340. Alternative rejected: read-only capacity sizing, because Actions and schema evolution drive write pressure. Cost: Citus, ClickHouse, Kafka, and Valkey capacity floors.
- Sustainability + cost attribution: PRD now requires ADR-0344 FinOps fields on entity/query/type/Cedar audit rows, with carbon-aware routing for OLAP/rebuilds and exclusions for PHI, break-glass, DSAR, and mutations. Alternative rejected: OLAP-only cost tagging, because Functions and Actions are tenant-visible. Cost: extra tags on hot query/write paths and FinOps dimensions.
- API versioning posture: PRD now adopts ADR-0342 date carriers, SDK semver, N=3 / 180-day support, schema/API per-tenant pinning, and ADR-0145 mesh exemption. Alternative rejected: Object Type schema versioning alone, because public REST/proto callers need carrier-level compatibility. Cost: API and schema-version matrix tests.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.24 vCPU; baseline_ram_per_tenant 512 MiB; storage_per_tenant 10 GB; connections valkey=3, postgres=5, outbound_http=6; scaling_dimension per_query; cell_placement_class Tier-1.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Ontology carries entity, link, projection, and history queries for product services, with higher memory and storage pressure from graph and ClickHouse mirrors.
- Rejected: cell_placement_class=Tier-2 because ADR-0340 names the ontology projection backbone as Tier-1 substrate.
- Cost: Allocates larger per-tenant graph/history storage and database connection reserves.

### Block 2: dr
- Values: rto_p99_seconds 900; rpo_p99_seconds 60; multi_region_active_active true; backup_substrate postgres_wal_g, clickhouse_iceberg_layered, audit_chain_merkle_seal, openbao_seal_unseal; failover_runbook runbooks/postgres-citus-rebalance.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Ontology is the typed substrate behind product entities and agent graph access; stale projections can mislead downstream workflows and evidence.
- Rejected: backup-restore cold recovery because product services depend on ontology reads for live semantics.
- Cost: Requires warm Citus/Postgres and ClickHouse/Iceberg recovery paths plus audit seal verification.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/ontology/PRD.md, microservices/ontology/ARCHITECTURE.md, microservices/ontology/IP-004-entity-store-rls-citus.md, microservices/ontology/IP-010-audit-chain-merkle-ed25519.md, microservices/ontology/runbooks/postgres-citus-rebalance.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Ontology stores tenant entity graphs, links, histories, and agent-facing projections. It does not run tenant code, but it is a tenant-data-touching substrate and therefore requires Tier 1 runtime isolation.
- Rejected: pod_runtime_tier=2 because tenant graph and projection state are substrate data-plane state.
- Cost: Tier 1 isolation adds overhead to graph read and projection workers.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Product teams and tenant integrations bind to ontology entity, link, and projection contracts.
- Rejected: single shared ontology schema version because tenants need pinned semantics for graph migrations.
- Cost: Maintains three ontology contract windows and graph migration documentation.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, clickhouse, cedar, openbao, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Ontology depends on registry-governed relational, analytical, policy, secret, telemetry, mesh, and admission substrates.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: No service-local stewardship override; CVE and pin movement follow registry owners.

### Block 6: iac_module_invocations
- Values: oci-guest/postgresql-cluster@v1, on-prem/clickhouse-iceberg-layer@v1, colo/openbao-secret-binding@v1, oyatie-as-cloud-provider/service-mesh-waypoint@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Ontology needs shared Postgres, analytical layer, secret, and mesh modules across deployment contexts.
- Rejected: bespoke graph-store provisioning because projection and recovery semantics must be uniform.
- Cost: Graph infrastructure changes now depend on shared module pin promotion.
