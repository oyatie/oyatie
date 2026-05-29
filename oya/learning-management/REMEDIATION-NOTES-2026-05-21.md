# learning-management remediation notes

## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O conversion for `learning-management`.
- IPs rewritten or deepened in place: 20.
- Files: IP-006-async-event-surface.md, IP-007-grpc-internal-surface.md, IP-008-policy-eval-library-binding.md, IP-009-credential-sidecar-binding.md, IP-010-multi-region-cell-layout.md, IP-011-observability-audit-events.md, IP-012-abuse-defence-edge-waf.md, IP-013-emergency-services-bypass.md, IP-014-marketplace-dealset-settlement.md, IP-015-data-residency-pack-overlays.md, IP-016-backfill-replay-worker.md, IP-017-cost-budget-enforcer.md, IP-018-capacity-admission-control.md, IP-019-sdk-client-generation.md, IP-020-catalog-layer-registration.md, IP-021-slo-gated-promotion.md, IP-022-chaos-drill-pack.md, IP-023-dpia-evidence-packet.md, IP-024-threat-model-control-map.md, IP-025-audit-findings-closeout.md.
- Deleted as duplicative: 0; no 80% duplicate pair was removed during this pass.
- Preserved as already-substantive: existing non-stamped IPs outside the short/stamped set retained in place.
- Verification target: no assigned IP remains in the 31-79 line stamp-shell band; rewritten IPs carry real path references and counterpart anchors.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/learning-management/catalog/oya-learning-management-credential-training-adapter-valkey.yaml

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- microservices/learning-management/catalog/oya-learning-management-credential-training-adapter-redis.yaml -> microservices/learning-management/catalog/oya-learning-management-credential-training-adapter-valkey.yaml

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states manifest-matching RTO 14400s/RPO 900s for catalog, enrollment, completion, assessment, and credential paths; HIPAA/KR-PIPA/SOC2/ISO floors; FERPA/KOSA as non-numeric overlays; active-active false; and runbooks for DR failover, content export, CDN failover, enrollment stall, and completion evidence mismatch. Rejected replacing the D-2 manifest target in the PRD because manifest is the service-local authority. Cost: HIPAA or KR-PIPA resident-id activation needs a stricter pack override below the baseline.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.09 vCPU, 224 MiB RAM, 8 GiB storage, 4 Postgres, 4 Valkey, 16 outbound HTTP sockets, `per_user` scaling, Tier-3 capacity class, and 1 to 8 replicas per paid tenant. Rejected a single bulk-import capacity model because live assessment submission must remain isolated from import and recommendation workers. Cost: worker caps can lengthen large catalog imports.
- Sustainability and cost attribution, ADR-0344: PRD now requires cost, CO2, and watt-hour fields on catalog, enrollment, assessment, credential, recommendation, import, and replay audit rows; carbon routing applies to recommendations/imports/backfills but not live assessments or credential sealing. Rejected carbon routing for live completion proof because regulatory evidence freshness is user-visible. Cost: FinOps attribution adds audit-row payload and reporting joins.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected SDK-semver-only versioning because LMS imports and credential-provider contracts need protocol-level pinning. Cost: tenant-specific migration windows require more contract fixtures.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.09 vCPU, 224 MiB RAM, 8 GB storage, and 4/4/16 connections per tenant; course media and completion evidence justify higher per-tenant storage.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 14400s, RPO 900s, multi-region active-active false, backup substrate postgres_wal_g, object_storage_versioned, valkey, failover runbook runbooks/content-export-failure.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/learning-management/PRD.md, microservices/learning-management/ARCHITECTURE.md, microservices/learning-management/IP-027-compliance-training-attestation-ledger.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, opentelemetry, opentofu, openbao; no local stewardship override declared. The service consumes the standard app stack; course content does not require a local OSS stewardship override.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/postgres-wal-g@v1, aws-guest/valkey-cluster@v1, oyatie-as-cloud-provider/object-storage-versioned@v1, on-prem/workload-deployment@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
