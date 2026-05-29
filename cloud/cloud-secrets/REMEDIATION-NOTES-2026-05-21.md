## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/cloud-secrets/onboarding/security-engineer-first-week.md`
- `microservices/cloud-secrets/migration-playbooks/from-hashicorp-vault.md`

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now keeps manifest RTO 120s/RPO 1s, evaluates it against pack floors, and names `runbooks/secret-substrate-failover.md` plus OpenBao, namespace, and audit-backlog recovery. Alternative considered: use HIPAA/EU floor RTO 1800s; rejected because secret resolution is a platform dependency for every consuming µservice. Cost: failover tests must prove the stricter-than-floor resolver objective.
- Capacity model (ADR-0340): PRD now records manifest values: 0.16 vCPU, 512 MiB RAM, 4 GiB namespace metadata, 4 Valkey, 3 Postgres, 6 outbound slots, `per_request` scaling, Tier-1 placement, and 2-to-16 resolver boundaries. Alternative considered: `per_user`; rejected because secret reads, rotations, and revocations are request/cadence driven. Cost: namespace sharding and emergency-revoke burst limits must be enforced.
- Sustainability + cost attribution (ADR-0344): PRD now requires secret lifecycle/access audit rows to carry cost/carbon/energy fields, while realtime resolution, revoke, HSM signing, and HIPAA/PCI paths ignore carbon routing. Alternative considered: carbon-aware secret resolver placement; rejected because it can break low-latency secret availability. Cost: security admin and FinOps views must reconcile namespace cost to tenant capability axes.
- API versioning posture (ADR-0342): PRD now requires date-versioned public admin/revocation APIs, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption. Alternative considered: SecretReference string versioning only; rejected because admin/revocation APIs need independent contract evolution. Cost: SDK config must carry both secret reference syntax and public API date pins.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 32.
Trigger A matches: 15.
Trigger B matches: 15.
Trigger C matches: 16.
Trigger D matches: 3.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- `microservices/cloud-secrets/IP-001-layer-a-openbao-postgres-hsm-iac.md`: Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-002-secretreference-uri-spec.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-003-resolver-kernel.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-004-resolver-domain.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-005-resolver-usecase.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-006-resolver-adapter-openbao.md`: Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/cloud-secrets/IP-007-resolver-rest-and-sdk-rust.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/cloud-secrets/IP-008-sdk-ts-python-bindings.md`: Trigger A -> API Versioning; Trigger D -> Pod runtime tier.
- `microservices/cloud-secrets/IP-009-openbao-operator.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-010-key-rotation-scheduler-worker.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-011-hsm-integration-adapter-hsm.md`: Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-012-per-tenant-namespace-controller.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-015-lean-a11-raw-secret-emission-lane-wiring.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j100-pack-rollout-first-action.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j25-key-envelope.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j80-provider-and-encryption-byok.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-journey-j81-provider-and-encryption-byok.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-journey-j83-provider-and-encryption-byok.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-journey-j86-provider-and-encryption-byok.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/cloud-secrets/IP-journey-j87-provider-and-encryption-byok.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md`: Trigger A -> API Versioning.
- `microservices/cloud-secrets/IP-journey-j91-us-msb-mtl-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j92-br-lgpd-us-parent-dsar.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j93-in-dpdpa-rbi-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j94-sox404-public-company-controls.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j95-iso27001-soc2-annual-audit.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j96-ksa-uae-mena-onboarding.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j97-sg-pdpa-mas-tenant.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j98-au-privacy-apra-cps234.md`: Trigger C -> Sustainability emission.
- `microservices/cloud-secrets/IP-journey-j99-multi-pack-conflict-resolution.md`: Trigger C -> Sustainability emission.

Unmatched IPs:
- none.

Follow-ups:
- Reconcile `manifest.json#dr` numeric service targets when the D-2 manifest DR fields land for this service.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.16 vCPU, 512 MiB RAM, 4 GB storage per active tenant; Valkey/Postgres/outbound connections 4/3/6; scaling_dimension=per_request; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Resolve qps, OpenBao reads/writes, HSM signing ops, and namespace count drive scale; capacity-model.md documents OpenBao 5k read qps and HSM 1k ops/s partitions.
- Rejected: Tier-0 was rejected because cloud-secrets is critical substrate but key authority root remains cloud-kms; Tier-1 matches ADR-0340 substrate placement.
- Cost: Commits OpenBao/HSM/Postgres state to <=120s RTO, near-zero RPO, and per-pack failover drills.

### Block 2: dr
- Values: RTO=120s, RPO=1s, multi_region_active_active=true, backup_substrate=openbao_seal_unseal+postgres_wal_g+audit_chain_merkle_seal+object_storage_versioned, failover_runbook=runbooks/secret-substrate-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns OpenBao secret manager substrate, SecretReference resolver, rotation scheduler, HSM integration, audit bridge; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/cloud-secrets/PRD.md, microservices/cloud-secrets/capacity-model.md, microservices/cloud-secrets/IP-005-resolver-usecase.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Secret-management substrate: cloud-secrets resolves SecretReference values, manages OpenBao namespaces, HSM integration, and key rotation, directly touching tenant secret data and requiring ADR-0338 Tier-1 isolation.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: SecretReference and resolver APIs are consumed by tenant-scoped services and need pinning for safe SDK/cache behavior.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=openbao,postgresql,valkey,cedar; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared 6 shared module primitive invocations from the service's IaC context evidence; inline OpenTofu resource bodies remain a migration risk until Wave 15Q lands module bodies.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
