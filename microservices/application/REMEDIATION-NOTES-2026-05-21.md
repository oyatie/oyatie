## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/application/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/application/capacity-model.md`
- `microservices/application/cost-budget.md`
- `microservices/application/PRD.md`
- `microservices/application/runbooks/session-storm.md`
- `microservices/application/threat-model.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.28 vCPU; baseline_ram_per_tenant: 512 MiB; storage_per_tenant: 6 GB.
- connections_per_tenant: valkey=4, postgres=4, outbound_http=10.
- scaling_dimension: per_user; cell_placement_class: Tier-2.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.28 vCPU / 512 MiB / 6 GB reserves shell/session, tenant-context, module-loader, and auth-gateway coordination per active user.
- Rejected: per_request was rejected because frontend shell pressure is session/user shaped and includes cached tenant context.
- Cost: Tier-2 placement treats the application shell as a critical product capability without paying Tier-1 substrate isolation cost.

### Block 2: dr
- rto_p99_seconds: 30; rpo_p99_seconds: 5; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, valkey, object_storage_versioned; failover_runbook: runbooks/tenant-context-recovery.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 30s / RPO 5s follows the documented warm-standby application shell target because login and tenant context are entry-path dependencies.
- Rejected: RTO 900s was rejected because an application-shell outage blocks access to downstream capabilities.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/application/PRD.md, microservices/application/ARCHITECTURE.md, microservices/application/IP-008-auth-gateway-kernel-domain.md, microservices/application/contracts/openapi/application.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Application is the first-party shell, routing, auth-gateway integration, and tenant-context surface; it is critical product entry infrastructure but not the tenant key or customer-code execution substrate.
- Rejected: Tier 1 was rejected because auth-gateway integration consumes substrate controls but does not own cloud-iam/key custody itself.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: unpinned shell APIs were rejected because module-loader and tenant-admin clients need release-window compatibility.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Cedar, OpenBao, and OpenTofu cover tenant context, cache/session, policy, secret references, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/dns@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and DNS modules are declared because Application is the externally addressed shell/API entry point.
- Rejected: declaring KMS ownership was rejected because key substrate responsibilities remain with cloud-kms/cloud-secrets.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: RTO 30s/RPO 5s, active-active false, runbook `runbooks/tenant-context-recovery.md`, ADR-0343. Alternative considered: product-service 900s class; rejected because Application is the tenant front door. Cost: hot enough standby and session replay evidence for shell recovery.
- Capacity model: 0.28 vCPU, 512 MiB RAM, 6 GB storage, Postgres 4, Valkey 4, outbound 10, `per_user`, Tier-2, ADR-0340/ADR-0341. Alternative considered: per-request route sizing only; rejected because active users and session hydration drive the shell. Cost: higher entry-path reserve than ordinary product services.
- Sustainability + cost attribution: shell, route, session, module-load, tenant-admin, and CDN-purge rows emit cost/carbon/watt dimensions, ADR-0344. Alternative considered: carbon-aware interactive routing; rejected for auth, route, and tenant-admin latency. Cost: per-tenant FinOps dimensions on front-door calls.
- API versioning posture: date carrier triplet plus SDK semver, last 3 versions for 180 days, tenant pinning enabled, ADR-0342. Alternative considered: semver-only public API; rejected because tenant-admin and shell clients need date-pinned wire contracts. Cost: release calendar and compatibility testing for shell clients.
