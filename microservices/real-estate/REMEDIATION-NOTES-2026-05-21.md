# real-estate remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/real-estate

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): Values mirror manifest `dr`: RTO 14400s, RPO 900s, `multi_region_active_active=false`, `dr_tier=T3`, `replication_shape=backup-restore-cross-region-warm`, `failover_runbook=runbooks/regional-failover.md`. Alternative considered: applying a 60s SOX general-ledger RPO to lease-accounting prose. Rejected because the manifest does not mark real-estate as a general-ledger journal writer. Cost: restored/warm DR is cheaper than active-active but needs evidence that pack gates reject future stricter data-class claims until the manifest changes.
- Capacity model (ADR-0340): Values mirror manifest `capacity_model`: 0.08 CPU, 256 MiB RAM, 6 GiB storage, connections Valkey 2/Postgres 3/outbound HTTP 5, `scaling_dimension=per_request`, `cell_placement_class=Tier-3`, `pod_runtime_tier=2`. Alternative considered: using large illustrative tenant classes from the companion stress model. Rejected because PRD prose must match manifest values. Cost: autoscaling must derive from a small per-tenant baseline and rely on stress-model expansion.
- Sustainability and cost attribution (ADR-0344): Values require `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on each real-estate audit row, with carbon routing only for replay/import/export and non-urgent facility workflows; manifest `sustainability_emission_model` remains absent. Alternative considered: aggregate-only monthly carbon totals. Rejected because CSRD, SB-253, and SEC climate disclosure need event-derived evidence. Cost: audit row width increases and a manifest follow-up must codify the emission model.
- API versioning posture (ADR-0342): Values set public carrier triplet, SDK semver, last 3 versions for at least 180 days, paid-tenant pinning, and ADR-0145 internal mesh exemption. Alternative considered: SDK semver only. Rejected because external lease/facility integrations need request-time carrier negotiation. Cost: contract registry and tenant-pin validation must be maintained during migrations.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.08 vCPU, 256 MiB RAM, 6 GB storage, Valkey/Postgres/outbound connections 2/3/5, scaling_dimension=per_request, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.08 vCPU/256 MiB/6 GB fits lease accounting and rent-roll generation without high event throughput.
- Rejected: Tier-2 cell placement was rejected because this is an application workload, not a shared capability substrate.
- Cost: Keeps modest per-tenant storage for lease artifacts and valuation records.

### Block 2: dr
- Value: RTO 14400s, RPO 900s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: Four-hour recovery and fifteen-minute RPO fit lease/rent-roll workflows without overbuying active-active regions.
- Rejected: One-hour RTO was rejected because no service evidence shows real-time safety or money movement pressure.
- Cost: Requires backup restore and artifact bucket recovery but no active-active standby.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/real-estate/ARCHITECTURE.md, microservices/real-estate/IP-016-ifrs16-right-of-use-computation.md, microservices/real-estate/IP-022-rent-roll-generation.md, microservices/real-estate/IP-025-portfolio-analytics-dashboard.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because it has tenant data but no substrate ownership.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Lease and accounting contract surfaces are tenant-facing and pin-worthy.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, aws-guest/object-storage-bucket@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Object-storage bucket is included for lease document and valuation artifact recovery.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
