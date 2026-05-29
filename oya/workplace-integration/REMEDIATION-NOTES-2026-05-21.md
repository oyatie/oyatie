# Workplace Integration remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/workplace-integration/IP-journey-j140-internal-audit-dlp-egress-cross-tenant-trace.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states RTO 3600s/RPO 300s for ESignSession, WorkplaceAgreement, offer, roster, clock, and DLP trace paths; SOC2/ISO/KR-PIPA floors; active-active legal evidence metadata; and failover references through `iac/region-failover.yaml`, e-sign corruption, closing-package archive, clock geofence cascade, and DLP replay runbooks. Rejected a dashboard-only recovery target because signatures and clock attestations are legal evidence. Cost: residency-aware active-active metadata raises storage and recovery-test overhead.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.09 vCPU, 224 MiB RAM, 5 GiB storage, 5 Postgres, 4 Valkey, 18 outbound HTTP sockets, `per_workflow_run` scaling, Tier-3 capacity class, and 1 to 8 replicas per tenant. Rejected per-user scaling because onboarding, shift starts, and DLP investigations spike by workflow. Cost: burst workers are capped, so large archive replay may queue.
- Sustainability and cost attribution, ADR-0344: PRD now requires cost, CO2, and watt-hour fields on WorkplaceAgreement, ESignSession, offer, roster, clock, archive, and DLP audit rows; carbon routing applies to archive/replay/import/export and not active signing, clock-in, consent, or commitment sealing. Rejected carbon-preferred placement for legal commitments because signature correctness and latency dominate. Cost: evidence rows gain FinOps dimensions.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected one global contract cutover because e-sign, staffing, HRIS, and roster partners operate on tenant-specific contracts. Cost: contract fixtures must cover pinned external integrations.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.09 vCPU, 224 MiB RAM, 5 GB storage, and 4/5/18 connections per tenant; e-sign packages, DLP traces, and schedules justify workflow and storage weighting.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, multi-region active-active true, backup substrate postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal, failover runbook runbooks/e-sign-session-corruption-recovery.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/workplace-integration/PRD.md, microservices/workplace-integration/ARCHITECTURE.md, microservices/workplace-integration/ip/IP-002-esign-session-domain.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, opentelemetry, opentofu, openbao, cosign; no local stewardship override declared. Cosign is included for document/e-sign evidence provenance; registry stewardship defaults remain sufficient.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/postgres-wal-g@v1, colo/audit-chain-merkle-seal@v1, on-prem/openbao-policy@v1, oyatie-as-cloud-provider/object-storage-versioned@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
