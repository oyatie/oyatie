## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/detection/decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now records manifest RTO 1800 s / RPO 300 s, cites HIPAA/PCI floors, names `runbooks/streaming-pipeline-lag.md` plus model/feature fallback runbooks, and follows manifest cell eligibility per ADR-0343. Alternative rejected: generic batch replay recovery, because live adverse-action scoring must fail into a safe scorer. Cost: regulated/sovereign warm workers and replay evidence retention.
- Capacity model: PRD now binds manifest values 0.30 vCPU, 768 MiB RAM, 15 GB storage, connections `{valkey:4, postgres:4, outbound_http:8}`, per-message scaling, Tier-2 placement, tier-1/tier-0 cell eligibility, and ADR-0338 Tier-1 runtime to ADR-0340. Alternative rejected: user-count scaling, because detection load follows event family and risk path. Cost: per-family partitions, min 3 workers/cell, and max 48 worker review boundary.
- Sustainability + cost attribution: PRD now requires ADR-0344 FinOps fields on ADR-0263 score/appeal/replay/fairness/drift rows, with carbon routing excluded for EU-AI Annex III, HIPAA emergency, PCI realtime fraud, and live adverse actions. Alternative rejected: carbon-aware live scoring, because protected decisions require latency and regulatory determinism. Cost: offline job scheduler and audit-dimension expansion.
- API versioning posture: PRD now adopts ADR-0342 date carriers, SDK semver, N=3 / 180-day support, regulated model rollout pinning, and ADR-0145 mesh exemption. Alternative rejected: model-version-only contracts, because tenants integrate REST/SDK decision surfaces. Cost: contract compatibility and emergency safety-patch bypass rules.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.3 vCPU; baseline_ram_per_tenant 768 MiB; storage_per_tenant 15 GB; connections valkey=4, postgres=4, outbound_http=8; scaling_dimension per_message; cell_placement_class Tier-2.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Detection processes tenant risk events, feature calculations, graph community updates, and investigation streams, so message volume dominates CPU, memory, and analytical storage.
- Rejected: cell_placement_class=Tier-3 because detection is not only an application surface; it is a regulated risk capability with EU AI and healthcare tenant data.
- Cost: Allocates higher RAM and storage for streaming, feature, graph, and investigation workloads.

### Block 2: dr
- Values: rto_p99_seconds 1800; rpo_p99_seconds 300; multi_region_active_active true; backup_substrate postgres_wal_g, clickhouse_iceberg_layered, object_storage_versioned, audit_chain_merkle_seal; failover_runbook runbooks/streaming-pipeline-lag.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Detection serves regulated fraud, abuse, healthcare, and EU AI high-risk flows; recovery must preserve scored event continuity and investigation evidence.
- Rejected: RTO=3600 because EU-AI high-risk floors and live incident handling require faster restoration.
- Cost: Maintains warm analytical recovery and replayable event/audit evidence streams.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/detection/PRD.md, microservices/detection/ARCHITECTURE.md, microservices/detection/IP-001-streaming-kernel.md, microservices/detection/IP-015-sandbox-replay-kernel.md, microservices/detection/runbooks/streaming-pipeline-lag.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Detection touches tenant risk events, features, graph signals, and investigation evidence. It does not execute tenant-customer code, but the tenant data plane and regulated scoring surface require Tier 1 runtime isolation.
- Rejected: pod_runtime_tier=2 because feature and risk event processing is tenant-data-touching substrate work.
- Cost: Tier 1 isolation raises streaming and scoring capacity overhead.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Detection contracts affect tenant risk decisions, investigation workflows, and regulated AI evidence.
- Rejected: latest-only scoring contract because tenants and investigators need pinned scoring semantics during model/rule migrations.
- Cost: Maintains three date windows for risk API, event, and proto surfaces.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss kafka, clickhouse, postgresql, cedar, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Detection consumes registry-covered streaming, analytical, relational, policy, telemetry, mesh, and admission dependencies.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: No override means detection must follow registry CVE ownership instead of owning fork stewardship.

### Block 6: iac_module_invocations
- Values: oci-guest/clickhouse-iceberg-layer@v1, on-prem/kafka-stream@v1, colo/postgresql-cluster@v1, oyatie-as-cloud-provider/service-mesh-waypoint@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Detection needs shared stream, analytical, relational, and mesh modules for replayable regulated scoring.
- Rejected: pipeline-specific IaC modules because DR and evidence replay depend on common primitives.
- Cost: Detection infra upgrades now depend on shared module pin promotion.
