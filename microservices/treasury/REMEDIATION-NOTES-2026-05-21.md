# treasury remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/treasury/performance-benchmark-numbers-2026-05-20.md

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)
- DR posture (ADR-0343): Values are manifest RTO p99 <= 1800s, RPO p99 <= 300s, `multi_region_active_active=true`, backup substrate `postgres_wal_g`/`valkey`/`object_storage_versioned`/`audit_chain_merkle_seal`, and failover runbook `microservices/treasury/runbooks/regional-failover.md`. WHY: cash position, payment release, bank-channel selection, FX exposure, and hedge approval must not duplicate bank submissions or use stale exposure watermarks. Alternative considered: active-passive treasury only. Rejected because manifest selects active-active warm posture for stricter treasury recovery. Cost: promotion evidence and conflict-proof idempotency before payment replay.
- Capacity model (ADR-0340): Values are manifest `0.16` vCPU, `512MiB` RAM, `16GB` storage, connections `{postgres:4,valkey:3,outbound_http:10}`, scaling `per_query`, `pod_runtime_tier=2`, `cell_placement_class=Tier-3`. WHY: cash-position, FX, and bank graph screens are query-heavy with outbound bank/SWIFT fan-out. Alternative considered: `per_request`. Rejected because control-path reads dominate operating load. Cost: bank-adapter isolation and stricter query-budget enforcement.
- Sustainability and cost attribution (ADR-0344): Values are cost, CO2, and watt-hour fields on each audit row, no carbon routing for payment, cash, FX, or hedge control paths, and finops-portal visibility by payment batch, bank channel, and FX snapshot. WHY: climate disclosure needs treasury attribution while SOX-grade controls require deterministic cutoff behavior. Alternative considered: carbon-aware provider choice for bank adapters. Rejected because cutoff and market-risk timing outrank carbon optimization. Cost: emissions are reporting-only for critical paths.
- API versioning posture (ADR-0342): Values are date carriers in header, URL, and proto3, SDK semver, last 3 versions for at least 180 days, tenant pinning yes, internal mesh exemption yes. WHY: bank profile and ISO 20022 variants need controlled transition windows. Alternative considered: bank-format versioning only. Rejected because public Oyatie contracts need one date carrier model. Cost: bank/ERP compatibility registry maintenance.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because this PRD does not declare an OLAP warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.16 vCPU, 512 MiB RAM, 16 GB storage, Valkey/Postgres/outbound connections 3/4/10, scaling_dimension=per_query, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.16 vCPU/512 MiB/16 GB covers cash-position graph reads, FX exposure, and outbound financial network calls.
- Rejected: per_message was rejected because SWIFT ingestion matters, but read/query pressure dominates cell sizing.
- Cost: Commits to larger outbound connection pools and active-active readiness.

### Block 2: dr
- Value: RTO 1800s, RPO 300s, active_active=true, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Thirty-minute RTO and active-active posture fit liquidity and payment-factory operational exposure.
- Rejected: SOX floor recovery was rejected because intraday cash and FX exposure cannot wait four hours.
- Cost: Requires warm cross-region cells and audit-chain protected failover checks.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/treasury/ARCHITECTURE.md, microservices/treasury/IP-017-payment-execution-via-iso20022-pain-001.md, microservices/treasury/IP-020-fx-exposure-intraday-delta-hedging.md, microservices/treasury/IP-023-swift-mt-mx-message-ingestion.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because treasury consumes financial substrate surfaces but is not the shared payment substrate.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Treasury exposes REST/event/proto contracts for cash, FX, and bank-account workflows.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, oyatie-as-cloud-provider/audit-chain-sink@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Audit-chain-sink and paid-scoped primitives reflect bank and FX evidence requirements.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
