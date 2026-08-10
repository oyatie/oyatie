## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references in `secrets/kms/`.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 did not modify `PRD.md` because `secrets/kms/PRD.md` and `secrets/kms/manifest.json` are absent. The D-3 procedure requires manifest-bound values before PRD doctrine can be propagated.

Values: no authoritative RTO/RPO, capacity_model, pod_runtime_tier, tenant_version_pinning, or OSS stewardship declarations were available. ADRs implicated once artifacts exist: ADR-0338, ADR-0340, ADR-0342 if public/operator key APIs exist, ADR-0343, ADR-0344, and ADR-0345; ADR-0341 likely applies because key custody is cell-placement sensitive; ADR-0339 applies only if `iac/<context>/` wrappers exist. Alternatives considered: infer KMS posture from billing invoice-signing needs or cloud-k8s control-plane encryption; rejected because key custody and HSM/BYOK behavior need their own manifest truth. Cost: missing PRD/manifest keeps KMS doctrine propagation blocked until foundational artifacts exist.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 0.
Trigger A matches: 0.
Trigger B matches: 0.
Trigger C matches: 0.
Trigger D matches: 0.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- none.

Unmatched IPs:
- none; no root `IP-*.md` files exist for the exact dispatch pattern.

Follow-ups:
- Confirm whether this service intentionally has no root `IP-*.md` dispatch surface for D-4 doctrine propagation.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.1 vCPU, 256 MiB RAM, 1 GB storage per active tenant; Valkey/Postgres/outbound connections 2/2/4; scaling_dimension=per_request; cell_placement_class=Tier-0.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Encrypt/decrypt/sign/generate data-key hot paths scale by crypto requests and HSM partition queues, not user sessions.
- Rejected: Tier-1 cell placement was rejected because KMS key authority is a foundation dependency for storage, secrets, and billing cryptoshred.
- Cost: Commits to zero-RPO key metadata replication discipline, HSM failover drills, and audit-chain seal continuity.

### Block 2: dr
- Values: RTO=300s, RPO=0s, multi_region_active_active=true, backup_substrate=postgres_wal_g+openbao_seal_unseal+audit_chain_merkle_seal+object_storage_versioned, failover_runbook=runbooks/hsm-cluster-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns CMK/KEK/DEK lifecycle, envelope encryption, HSM custody, signing, crypto-shred receipts; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=secrets/kms/README.md, secrets/kms/feature-parity-matrix-2026-05-20.md, secrets/kms/runbooks/hsm-cluster-failover.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-0.
- Why: Foundation key-management substrate: cloud-kms touches tenant key metadata, HSM operations, signing, and crypto-shred receipts, matching ADR-0338 Tier-1 and ADR-0340 Tier-0 placement.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: KMS API behavior is a public cloud compatibility contract and must support tenant pinning for rotation/import semantics.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=openbao,postgresql,valkey,cedar; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
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
