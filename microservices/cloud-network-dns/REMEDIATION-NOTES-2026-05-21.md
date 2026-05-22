## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references in `microservices/cloud-network-dns/`.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now sets manifest RTO 300s/RPO 60s, multi-region zone intent, and `runbooks/dns-zone-failover.md` as the failover reference. Alternative considered: use only migration cutover docs; rejected because D-2 manifest already names a DNS-specific runbook. Cost: DNS failover tests must prove 60s RPO for zone-policy state.
- Capacity model (ADR-0340): PRD now records manifest values: 0.06 vCPU, 128 MiB RAM, 1 GiB zone/query-log index, 3 Valkey, 1 Postgres, 4 outbound slots, `per_request` scaling, Tier-4 edge placement, and 2-to-24 publisher scaling. Alternative considered: Tier-1 only; rejected because customer-visible DNS is latency-sensitive anycast edge and manifest declares Tier-4. Cost: edge and control capacity must be modeled separately in admission.
- Sustainability + cost attribution (ADR-0344): PRD now requires zone, DNSSEC, resolver-policy, health-check, and privileged query-log audit rows to carry cost/carbon/energy fields, while live resolution and failover ignore carbon routing. Alternative considered: per-query audit rows for all DNS reads; rejected because query volume would overwhelm audit-chain and billing should aggregate routine queries. Cost: query rollup jobs must feed FinOps without weakening privileged-access audit detail.
- API versioning posture (ADR-0342): PRD now requires the date carrier triplet for zone/resolver/DNSSEC/health APIs, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption. Alternative considered: provider-compatible API versioning only; rejected because Oyatie native policy needs stable tenant pins across Route53/NS1 migrations. Cost: adapters must translate provider semantics into native versioned contracts.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.06 vCPU, 128 MiB RAM, 1 GB storage per active tenant; Valkey/Postgres/outbound connections 3/1/4; scaling_dimension=per_request; cell_placement_class=Tier-4.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=3.
- Why: DNS query/control load scales by request volume and zone-policy cache churn; ADR-0340 Tier-4 matches edge/perf-critical DNS placement.
- Rejected: Tier-1 cell placement was rejected because ADR-0340 reserves Tier-4 for edge/perf-critical surfaces that must not inherit substrate Kata overhead.
- Cost: Commits DNS zone state to active-active replication, low TTL failover, and cache warmup costs.

### Block 2: dr
- Values: RTO=300s, RPO=60s, multi_region_active_active=true, backup_substrate=valkey_cluster+postgres_wal_g+object_storage_versioned, failover_runbook=runbooks/dns-zone-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns authoritative DNS, recursive DNS, zone scoping, DNSSEC, health checks, routing policy, anycast; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=3; evidence=microservices/cloud-network-dns/README.md, microservices/cloud-network-dns/performance-benchmark-numbers-2026-05-20.md, contracts/openapi/cloud/cloud-network-dns-v1.yaml.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-4.
- Why: DNS is edge/performance-critical: authoritative/recursive serving, health checks, and anycast routing demand dedicated edge runc placement per ADR-0338 Tier 3 rather than Kata substrate placement.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: DNS zone APIs are tenant-facing automation surfaces and routing-policy changes need date-version pinning.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=cilium,istio,valkey,postgresql; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared an empty invocation array because no service-local iac/<context>/ directory is present; this keeps ADR-0339 machine-decidable without inventing wrapper usage.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
