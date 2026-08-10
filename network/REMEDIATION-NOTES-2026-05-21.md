## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references in `network/`.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now sets manifest RTO 600s/RPO 300s, multi-region active-active control intent, and `runbooks/network-control-plane-failover.md` as the failover reference. Alternative considered: reuse generic cloud failover prose; rejected because network route/mTLS state is the tenant-visible continuity boundary. Cost: validation must prove last-known-good data-plane forwarding during control recovery.
- Capacity model (ADR-0340): PRD now records manifest values: 0.14 vCPU, 256 MiB RAM, 2 GiB route/security metadata, 2 Valkey, 2 Postgres, 10 outbound slots, `per_capability` scaling, Tier-1 cell placement, and 2-to-20 controller/evaluator autoscaling. Alternative considered: `per_request`; rejected because D-2 manifest doctrine already names network capability objects as the scaling unit. Cost: quota enforcement and namespace sharding logic must exist before high-churn tenants are admitted.
- Sustainability + cost attribution (ADR-0344): PRD now requires network control actions and privileged flow-log reads to emit cost/carbon/energy fields with audit rows, with carbon routing excluded from live routing, mTLS, DDoS, and realtime compliance paths. Alternative considered: carbon-aware route placement; rejected because it can delay protective controls. Cost: FinOps needs egress and flow-telemetry capability filters.
- API versioning posture (ADR-0342): PRD now requires the date carrier triplet for public network APIs, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption. Alternative considered: semver-only API paths; rejected because tenant pinning must be independent of SDK release cadence. Cost: migration tooling must carry tenant date-version pins.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.14 vCPU, 256 MiB RAM, 2 GB storage per active tenant; Valkey/Postgres/outbound connections 2/2/10; scaling_dimension=per_capability; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: VPC, route, LB, security-rule, and telemetry work scales by network capability objects and route/policy cardinality.
- Rejected: Tier-4 edge placement was rejected because this service is the network control plane, not the edge forwarding/data-plane runtime.
- Cost: Commits route/policy metadata to active-active control-plane replication and cross-cell route recovery drills.

### Block 2: dr
- Values: RTO=600s, RPO=300s, multi_region_active_active=true, backup_substrate=postgres_wal_g+valkey_cluster+object_storage_versioned+audit_chain_merkle_seal, failover_runbook=runbooks/network-control-plane-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns tenant VPC, routing, load balancing, ingress/egress policy, mTLS, flow telemetry; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=network/README.md, network/performance-benchmark-numbers-2026-05-20.md, network/core/domain/src/lib.rs.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Network substrate control plane: tenant VPCs, routes, ingress/egress policy, and mTLS enforcement touch tenant data-plane topology, so ADR-0338 Tier-1 is required instead of first-party app placement.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: VPC/LB APIs are public cloud surfaces and need pinned versions for tenant infrastructure automation.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=cilium,istio,valkey,postgresql,kyverno; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
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
