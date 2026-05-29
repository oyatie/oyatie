# financial-planning remediation notes
## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O review for `financial-planning`.
- IPs rewritten in place: 0.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 30.
- Counterpart anchors were made explicit where the verification regex lacked the service's native benchmark vocabulary.
- Follow-up: none for stamp-shell conversion; future service owners may still improve individual IP depth outside this bucket.

## Wave 15J-final-cleanup
- Scope: F-BUCKET-2 final residue verification for ERP stamped docs.
- Renamed stale 2026-05-20 audit artifacts to 2026-05-21 and scrubbed retired B/S/G/P and `capability_tier` vocabulary under this service path.
- Replaced IP front-matter residue with tenant-class metadata and rewrote benchmark rows to avoid retired plan-token vocabulary.
- Verification: assigned Wave 15J grep checks return zero non-remediation residue.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/financial-planning/coherence-audit-2026-05-21.md
- microservices/financial-planning/catalog/oya-financial-planning-forecast-scenario-adapter-valkey.yaml

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none; source catalog path was untracked, so filesystem rename was used:
  microservices/financial-planning/catalog/oya-financial-planning-forecast-scenario-adapter-redis.yaml -> microservices/financial-planning/catalog/oya-financial-planning-forecast-scenario-adapter-valkey.yaml

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 3600s, RPO p99 <= 300s, `multi_region_active_active=false`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`/`audit_chain_merkle_seal`, and manifest failover runbook `runbooks/regional-failover.md` with current tangible artifact `microservices/financial-planning/iac/dr-failover.yaml`. WHY: forecast versions, board packets, scenarios, and consolidation evidence need recoverability without cross-cell write ambiguity. Alternative considered: active-active finance writes. Rejected because home-cell mutation is already specified by IP-010. Cost: runbook materialization/alignment and close-cycle drill evidence.
- Capacity model (ADR-0340): Values are manifest `0.22` vCPU, `768MiB` RAM, `20GB` storage, connections `{postgres:5,valkey:4,outbound_http:8}`, scaling `per_workflow_run`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: scenario recalculation and consolidation close burn workflow-sized compute. Alternative considered: shared spreadsheet-style capacity. Rejected because IP-017 requires tenant budget enforcement before work starts. Cost: queue metering and budget-lock enforcement.
- Sustainability and cost attribution (ADR-0344): Values are cost, CO2, and watt-hour fields on each audit row, carbon-aware routing for planning/replay/render workloads when floors permit, and cost transparency through the cost-budget enforcer plus finops-portal. WHY: finance owners need climate and spend attribution in the same approval surface. Alternative considered: use only marketplace settlement cost lines. Rejected because emissions and watt-hours are mandatory ADR-0344 dimensions. Cost: additional rollup axes and freshness checks.
- API versioning posture (ADR-0342): Values are date carriers in header, URL, and proto3, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: close-cycle cutovers and vendor migrations need reproducible planning API dates. Alternative considered: internal-only versioning. Rejected because financial-planning has public REST/AsyncAPI contracts. Cost: deprecation calendar and tenant pin registry.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.22 vCPU, 768 MiB RAM, 20 GB storage, Valkey/Postgres/outbound connections 4/5/8, scaling_dimension=per_workflow_run, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.22 vCPU/768 MiB/20 GB is higher than ERP peers because forecast recalc and close consolidation retain scenario state.
- Rejected: per_query was rejected because tenant pain comes from workflow recalculation, not isolated reads.
- Cost: Commits to warm worker and storage headroom during close cycles.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: One-hour RTO and five-minute RPO match close-cycle and board-report evidence tolerance.
- Rejected: Four-hour SOC2 floor was rejected because close-cycle delays directly block executive reporting.
- Cost: Adds a dedicated regional failover runbook and audit-chain replay checks.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/financial-planning/ARCHITECTURE.md, microservices/financial-planning/IP-010-multi-region-cell-layout.md, microservices/financial-planning/IP-022-chaos-drill-pack.md, microservices/financial-planning/slos/local-close-cycle-latency.openslo.yaml.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because financial planning touches tenant financial records but does not own substrate secrets or payment rails.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Primary public and local contract files exist; the manifest pins the primary public surfaces.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, on-prem/object-storage-bucket@v1, aws-guest/replay-worker@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Replay-worker and object-storage primitives reflect close-cycle restore and board-report artifact recovery.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
