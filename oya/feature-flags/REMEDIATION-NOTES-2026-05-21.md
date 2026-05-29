## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory found no Redis references in `microservices/feature-flags/`.

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set RTO 60s and RPO 5s under ADR-0343 because the PRD already commits to 5s definition replication and 99.99% evaluation availability, which is stricter than HIPAA/PCI/SOC2 floors. Alternative considered: use compliance floors only; rejected because kill-switch safety needs faster restore than regulatory minima. Cost: active-active cells, SDK fallback, and audit replay remain mandatory.
- Capacity model: anchored ADR-0340 to 0.1 vCPU/128MiB, 10MiB flag definitions, 100MiB eval cache, one SDK stream class, and max 100 evaluator replicas per cell. Alternative considered: scale by tenant plan label; rejected because eval rate, rule complexity, and audit-required volume are the real load drivers. Cost: evaluator, mutation, and audit-replay lanes require separate quota and autoscale controls.
- Sustainability + cost attribution: added ADR-0344 fields to flag mutation, audit-required eval, kill-switch, experiment, pack override, rollout, and rollback rows. Alternative considered: omit online evaluations as too small; rejected because high-volume automated decision flags create material aggregate emissions and audit obligations. Cost: online kill-switch paths do not carbon-route, so emissions are attributed rather than optimized there.
- API versioning posture: adopted ADR-0342 carrier triplet, SDK semver, tenant pinning, and last-three/180-day support for OpenFeature REST/gRPC and SDK contracts. Alternative considered: rely on OpenFeature provider compatibility alone; rejected because tenant pack overlays and audit fields are Oyatie-specific public contracts. Cost: SDK compatibility test lanes must run for three live contract dates.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.08 vCPU; baseline_ram_per_tenant: 192 MiB; storage_per_tenant: 2 GB.
- connections_per_tenant: valkey=4, postgres=3, outbound_http=6.
- scaling_dimension: per_request; cell_placement_class: Tier-1.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.08 vCPU / 192 MiB / 2 GB is intentionally small because hot-path evaluation is cache-heavy but per-request latency critical.
- Rejected: per_user sizing was rejected because SDK evaluations are driven by service calls and cohorts, not only human users.
- Cost: Tier-1 placement commits this low-footprint service to substrate-grade isolation and admission checks for kill-switch safety.

### Block 2: dr
- rto_p99_seconds: 900; rpo_p99_seconds: 60; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, valkey, object_storage_versioned; failover_runbook: runbooks/audit-replay.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 900s / RPO 60s follows the service's fast recovery posture for flags and audit replay while preserving active-passive writer safety.
- Rejected: RPO 300s was rejected because stale kill-switch and rollout state can create immediate tenant safety exposure.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 1; evidence: microservices/feature-flags/PRD.md, microservices/feature-flags/ARCHITECTURE.md, microservices/feature-flags/IP-002-flag-kernel.md, microservices/feature-flags/contracts/openapi-v1.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Feature Flags is a shared substrate for tenant rollout, targeting, and kill-switch decisions; tenant-serving data and safety controls justify ADR-0338 Tier 1 placement.
- Rejected: Tier 2 was rejected because feature flags are a shared control substrate consumed by other tenant-serving services.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: unpinned OpenFeature-style contracts were rejected because SDK clients need long-lived tenant-pinned compatibility.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, kafka, clickhouse, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Kafka, ClickHouse, Cedar, OpenBao, and OpenTofu cover source-of-truth, hot cache, event/audit streams, analytics, policy, secrets, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/dns@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and DNS modules are declared because SDK and control APIs expose a shared substrate endpoint.
- Rejected: service-owned DNS/secret primitives were rejected because ADR-0339 requires cloud-iac shared modules.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
