# supply-chain-planning remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/supply-chain-planning/performance-benchmark-numbers-2026-05-20.md

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 14400s, RPO p99 <= 900s, `multi_region_active_active=false`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`, and failover runbook `microservices/supply-chain-planning/runbooks/regional-failover.md`. WHY: ATP/CTP, replenishment, and transportation decisions must survive regional loss without double-promising inventory. Alternative considered: declare active-active now. Rejected because current pack floors and manifest replication shape select warm backup-restore. Cost: drill evidence and queued replay discipline.
- Capacity model (ADR-0340): Values are manifest `0.12` vCPU, `384MiB` RAM, `12GB` storage, connections `{postgres:3,valkey:4,outbound_http:8}`, scaling `per_query`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: quote and planning-scenario bursts are query-heavy but tenant-bound. Alternative considered: `per_request`. Rejected because ATP/CTP reads dominate user-visible load. Cost: stricter per-tenant queue/backpressure tuning inside a low baseline.
- Sustainability and cost attribution (ADR-0344): Values are `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on each audit row, carbon-aware routing for async planning only, and finops-portal plan-run transparency. WHY: CSRD, SB-253, and SEC climate disclosure require attributable planning emissions. Alternative considered: cost-only attribution. Rejected because ADR-0344 requires emissions dimensions. Cost: extra audit payload fields and rollup freshness checks.
- API versioning posture (ADR-0342): Values are `Oyatie-Version`, `/v/YYYY-MM-DD/`, proto3 `oyatie_version`, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: partner RFQ and ERP migration callers need stable dated carriers. Alternative considered: SDK semver only. Rejected because public carrier dates are the ADR-0342 contract. Cost: version router and deprecation bookkeeping.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.12 vCPU, 384 MiB RAM, 12 GB storage, Valkey/Postgres/outbound connections 4/3/8, scaling_dimension=per_query, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.12 vCPU/384 MiB/12 GB fits read-heavy ATP and scenario-query load; per_query avoids pretending the service scales by named users.
- Rejected: per_user was rejected because replenishment and ATP workloads are driven by planning queries and joins.
- Cost: Commits each paid cell to warm Valkey plus Postgres pool headroom for shortage/allocation spikes.

### Block 2: dr
- Value: RTO 14400s, RPO 900s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: SOC2-style planning continuity is enough; losing more than 15 minutes of forecast/ATP state would force manual replanning.
- Rejected: active-active was rejected because planning recovery can replay from WAL and events without synchronous region writes.
- Cost: Requires WAL-G restores, Valkey projection warmup, and regional failover rehearsal.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/supply-chain-planning/ARCHITECTURE.md, microservices/supply-chain-planning/IP-019-atp-compute.md, microservices/supply-chain-planning/IP-023-constraint-based-supply-heuristic.md, microservices/supply-chain-planning/slos/supply-chain-planning-latency-p99.openslo.yaml.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because this product service does not own tenant-data substrate primitives.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Public REST, AsyncAPI, and proto contracts exist, so ADR-0342 tenant pinning applies.
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
- Why: Thin wrappers invoke tenant namespace, runc nodepool, Postgres, Valkey, demo cache, and event-topic primitives.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
