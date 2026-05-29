## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references under microservices/consent-graph.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now sets manifest RTO 1800s/RPO 300s, multi-region agreement/revocation state, and `runbooks/audit-chain-divergence-recovery.md` plus revocation recovery. Alternative considered: RPO 0s; rejected because D-2 manifest already declares the durable recovery objective and PRD prose must match it. Cost: revocation-path tests still need to prove runtime p99 propagation independent of DR RPO.
- Capacity model (ADR-0340): PRD now records manifest values: 0.12 vCPU, 192 MiB RAM, 4 GiB agreement/index storage, 3 Valkey, 3 Postgres, 5 outbound Pulsar/audit slots, `per_message` scaling, Tier-1 placement, and enforcement/revocation/projection worker bounds. Alternative considered: `per_capability`; rejected because D-2 manifest doctrine already names consent events/messages as the scaling unit. Cost: agreement graph partitioning must exist before extreme partner fan-out.
- Sustainability + cost attribution (ADR-0344): PRD now requires grant, accept, revoke, projection, enforcement, handshake, and bilateral audit rows to carry cost/carbon/energy fields, while revocation and enforcement ignore carbon routing. Alternative considered: bill only the grantor tenant; rejected because bilateral visibility consumes both sides' cells and must be transparent. Cost: FinOps rollups must preserve grantor and grantee attribution.
- API versioning posture (ADR-0342): PRD now requires date carriers for public agreement/revocation/partner/projection APIs, SDK semver, 3-version/180-day support, tenant pinning, and mesh exemption for enforcement. Alternative considered: mesh-only versioning; rejected because grantor/grantee SDK clients are public integration contracts. Cost: partner onboarding must negotiate date-version pins.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.12 vCPU; baseline_ram_per_tenant 192 MiB; storage_per_tenant 4 GB; connections valkey=3, postgres=3, outbound_http=5; scaling_dimension per_message; cell_placement_class Tier-1.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Consent enforcement and revocation fan-out are message-driven and tenant-data-sensitive, with moderate graph state and partner lookup load.
- Rejected: cell_placement_class=Tier-2 because ADR-0340 classifies consent-graph as Tier-1 substrate.
- Cost: Maintains substrate placement and cache/database headroom for revocation propagation.

### Block 2: dr
- Values: rto_p99_seconds 1800; rpo_p99_seconds 300; multi_region_active_active true; backup_substrate postgres_wal_g, valkey_cluster, audit_chain_merkle_seal; failover_runbook runbooks/audit-chain-divergence-recovery.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Consent revocation latency is privacy-critical; HIPAA/GDPR-style floors require quick recovery and bounded consent-state loss.
- Rejected: RTO=3600 only because stale revocations can continue unauthorized processing during outage recovery.
- Cost: Active consent graph replication and audit divergence drills are required.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/consent-graph/PRD.md, microservices/consent-graph/ARCHITECTURE.md, microservices/consent-graph/IP-007-revocation-kernel-worker.md, microservices/consent-graph/IP-012-audit-bridge-bilateral-emitter.md, microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Consent-graph stores and enforces tenant consent, revocation, partner, and audit-bridge state. It does not run tenant code, but it directly governs tenant-data processing, so Tier 1 runtime isolation is required.
- Rejected: pod_runtime_tier=2 because consent and revocation state is substrate data-plane control, not merely first-party business logic.
- Cost: Tier 1 isolation adds capacity overhead to enforcement and revocation workers.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Tenant and partner integrations consume agreement, revocation, and enforcement contracts directly.
- Rejected: unversioned consent contracts because privacy semantics are regulator-visible and tenant-specific.
- Cost: Maintains three consent contract windows and migration documentation.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, valkey, cedar, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Consent-graph uses registry-governed persistence, cache, policy, telemetry, mesh, and admission surfaces.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: No stewardship overrides means the service must consume upstream fixes on registry cadence.

### Block 6: iac_module_invocations
- Values: oci-guest/postgresql-cluster@v1, on-prem/service-mesh-waypoint@v1, colo/valkey-cluster@v1, oyatie-as-cloud-provider/audit-chain-merkle-seal@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Consent enforcement needs shared database, cache, mesh, and audit seal modules across deployment contexts.
- Rejected: local revocation infra modules because cross-context enforcement must be identical.
- Cost: Shared module upgrades now gate consent rollout changes.
