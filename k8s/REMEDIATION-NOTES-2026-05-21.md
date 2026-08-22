## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/cloud-k8s/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/cloud-k8s/coherence-audit-2026-05-20.md`
- `microservices/cloud-k8s/feature-parity-matrix-2026-05-20.md`
- `microservices/cloud-k8s/PRD.md`
- `microservices/cloud-k8s/performance-benchmark-numbers-2026-05-20.md`
- `microservices/cloud-k8s/multi-region.md`
- `microservices/cloud-k8s/iac/kustomize/components/storage-classes/kustomization.yaml`
- `microservices/cloud-k8s/iac/kustomize/components/storage-classes/storage-class-valkey-hot.yaml`

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- `microservices/cloud-k8s/iac/kustomize/components/storage-classes/storage-class-redis-hot.yaml` -> `microservices/cloud-k8s/iac/kustomize/components/storage-classes/storage-class-valkey-hot.yaml`

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 updated `PRD.md` frontmatter with ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because cloud-k8s hosts substrate state and cluster events, not the data-warehouse OLAP writer path.

### DR posture

Values: RTO 1800 seconds, RPO 300 seconds, runbooks `control-plane-restore.md`, `etcd-quorum-recovery.md`, and `kubeadm-upgrade.md`; active-active is false for single etcd/control-plane writer and true only at pack/cell federation via separate clusters. ADR: ADR-0343. Alternatives considered: active-active etcd across packs or relaxed SOC2/ISO floors; rejected because Kubernetes consistency and residency boundaries override a cross-pack writer. Cost: manifest `dr` block remains absent and must be backfilled.

### Capacity model

Values: cluster bootstrap p99 <= 30 minutes, node join p99 <= 5 minutes, 10 baseline launch nodes, 5,000 max nodes/cluster, workload-class NodePools `app`, `batch`, `gpu`, `regulatory`, and manifest `criticality_tier: T2` mapped to Tier-2 cell placement. ADR: ADR-0340. Alternatives considered: tenant-app request-rate sizing or Tier-1 ledger placement; rejected because cloud-k8s capacity is cluster/node churn and it is substrate but not the canonical money/identity/key ledger. Cost: exact per-tenant CPU/RAM/storage/connection baselines still require D-2 manifest data.

### Sustainability + cost attribution

Values: cluster/node/network-policy/mesh/upgrade/API-proxy audit rows emit cost, CO2, and watt-hours; carbon routing applies to non-urgent node placement, batch scale-out, and planned maintenance, and is excluded for HIPAA emergency mode, PCI realtime-fraud dependency recovery, control-plane restore, and regulatory region pins. ADR: ADR-0344. Alternatives considered: only rely on OpenCost aggregate labels; rejected because ADR-0344 requires audit-row emission. Cost: OpenCost and audit-chain must align on workload-class/cell/compliance-pack axes.

### API versioning posture

Values: cluster lifecycle, node lifecycle, network-policy, CSI, ingress, and kubernetes-api-proxy contracts use the YYYY-MM-DD carrier triplet; operator SDKs use semver; last 3 versions supported for at least 180 days; paid/regulatory tenants can pin during maintenance/audit windows; in-cluster mesh traffic is exempt. ADR: ADR-0342. Alternatives considered: treat Kubernetes API proxy as purely internal; rejected because operators, agents, and CI cross the boundary and need deprecation guarantees. Cost: API proxy and SDK generation must enforce carrier triplets.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.22 vCPU, 512 MiB RAM, 4 GB storage per active tenant; Valkey/Postgres/outbound connections 1/2/12; scaling_dimension=per_capability; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Cluster bootstrap, node lifecycle, CNI/service-mesh, and API proxy load scales by managed runtime capability rather than end-user count.
- Rejected: Tier-0 was rejected because cloud-k8s is critical substrate but not the identity/key authority root; Tier-1 keeps isolation without conflating it with IAM/KMS.
- Cost: Commits runtime control-plane restore to cross-region warm capacity and module-pin hygiene for kubeadm/CNI/mesh primitives.

### Block 2: dr
- Values: RTO=1800s, RPO=300s, multi_region_active_active=true, backup_substrate=postgres_wal_g+object_storage_versioned+audit_chain_merkle_seal, failover_runbook=runbooks/control-plane-restore.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns Kubernetes cluster bootstrap, runtime, node lifecycle, network policy, service mesh, ingress; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/cloud-k8s/PRD.md, microservices/cloud-k8s/ARCHITECTURE.md, microservices/cloud-k8s/IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Shared workload-runtime substrate: cloud-k8s owns cluster bootstrap, kube API proxying, CNI, service mesh, and node lifecycle for tenant workloads, so it is tenant-impacting substrate even though it does not execute tenant-customer code directly.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: cluster lifecycle and runtime APIs change operational semantics and need pinned tenant/operator contract windows.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=cilium,istio,kyverno,opentofu,valkey; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared 8 shared module primitive invocations from the service's IaC context evidence; inline OpenTofu resource bodies remain a migration risk until Wave 15Q lands module bodies.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
