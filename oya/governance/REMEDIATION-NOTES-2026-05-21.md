<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: governance
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 39
  ADR_0316_citations_replaced: 6
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

- Rewritten in place as bespoke substance: `IP-WASMTIME-001-envoy-wasm-filter-substrate.md`, `IP-WASMTIME-003-regulatory-response-shaper.md`, `IP-WASMTIME-004-authz-filter.md`, `IP-001-scaffold-umbrella-bcs.md`, `IP-002-migrate-tier-a-check-crates-batch-1.md`, `IP-003-migrate-tier-a-check-crates-batch-2.md`, `IP-013-aggregation-index-generation-lane.md`.
- Preserved as already substantive with explicit counterpart verification note where needed: remaining governance IP files.
- Deleted as duplicative: none. Journey IPs share generator structure but contain jurisdiction/journey-specific anchors and were not safe to merge in this scrub.
- Counterpart anchors added: GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, Renovate, OpenAI/Anthropic for regulated response shaping.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/governance/benchmarks/drata-vanta-onetrust-vs-oyatie.md
- microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl
- microservices/governance/onboarding/governance-engineer-first-week.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now records manifest RTO 900 s / RPO 60 s, cites HIPAA/SOC2/ISO floors, names `runbooks/evidence-replay.md`, and states active-active per ADR-0343. Alternative rejected: rollback-only governance recovery, because evidence replay and lane state also need DR. Cost: pack-local evidence-store replicas and replay capacity.
- Capacity model: PRD now binds manifest values 0.20 vCPU, 384 MiB RAM, 6 GB storage, connections `{valkey:2, postgres:4, outbound_http:10}`, per-workflow-run scaling, Tier-1 placement, ARC runner pool 8-200, and ADR-0338 Tier-1 runtime to ADR-0340. Alternative rejected: static runner pool, because PR storms would block fleet-wide admission. Cost: runner prewarm, S3 evidence capacity, and Postgres write headroom.
- Sustainability + cost attribution: PRD now requires ADR-0344 FinOps fields on lane/finding/baseline/index audit rows, with carbon routing only for non-blocking refresh and replay. Alternative rejected: live gate carbon routing, because security blockers and merge gates must be freshness-first. Cost: per-lane cost labeling and FinOps rollups.
- API versioning posture: PRD now adopts ADR-0342 date carriers, SDK semver, N=3 / 180-day support, auditor/conformance API pinning, and ADR-0145 mesh exemption. Alternative rejected: lane-registry-only versioning, because external replay/status users need stable API dates. Cost: three supported public surfaces plus lane registry compatibility.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.2 vCPU; baseline_ram_per_tenant 384 MiB; storage_per_tenant 6 GB; connections valkey=2, postgres=4, outbound_http=10; scaling_dimension per_workflow_run; cell_placement_class Tier-1.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Governance load is driven by policy lane executions, evidence emission, bundled checks, and policy replay rather than steady end-user traffic.
- Rejected: cell_placement_class=Tier-2 because governance gates shared promotion and evidence paths across services and needs substrate placement.
- Cost: Reserves extra memory and outbound capacity for policy runners and evidence fan-out.

### Block 2: dr
- Values: rto_p99_seconds 900; rpo_p99_seconds 60; multi_region_active_active true; backup_substrate postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal; failover_runbook runbooks/evidence-replay.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Governance recovery must restore policy decisions and evidence replay quickly enough to avoid unsafe promotion gaps.
- Rejected: four-hour SOC2 floor because stalled policy gates would block or desynchronize delivery evidence across the fleet.
- Cost: Requires replicated lane state and audit-sealed replay for governance findings.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/governance/PRD.md, microservices/governance/ARCHITECTURE.md, microservices/governance/IP-007-policy-engine-usecase-adapter.md, microservices/governance/IP-015-runbooks-iac-finalization.md, microservices/governance/runbooks/evidence-replay.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Governance is a shared policy and promotion substrate touching tenant-scoped evidence, policy decisions, and release-gate state. It hosts first-party policy/rule execution rather than tenant-customer code, so Tier 1 isolation is the correct boundary.
- Rejected: pod_runtime_tier=0 because the Wasm/WAF references are first-party policy substrates, not tenant-uploaded executable code.
- Cost: Tier 1 runtime isolation increases policy-lane compute cost but preserves evidence integrity.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Governance exposes policy, lane, and evidence contracts to internal service teams and tenant-visible compliance flows.
- Rejected: unversioned policy API because policy semantics become audit evidence and need tenant pinning.
- Cost: Keeps three policy contract windows and migration guidance active.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, cedar, opentofu, wasmtime, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Governance composes registry-governed persistence, policy, IaC, Wasm runtime, telemetry, mesh, and admission dependencies.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: No overrides means governance consumes upstream stewardship and CVE SLAs from the registry.

### Block 6: iac_module_invocations
- Values: oci-guest/postgresql-cluster@v1, on-prem/lane-runner-pool@v1, colo/evidence-store@v1, oyatie-as-cloud-provider/service-mesh-waypoint@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Governance lanes and evidence stores need shared modules to keep policy enforcement portable.
- Rejected: lane-local IaC because governance promotion behavior must not drift by environment.
- Cost: Policy lane infrastructure changes are now tied to shared module pin validation.
