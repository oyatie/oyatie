# Contact Center remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/contact-center/coherence-audit-2026-05-20.md
- microservices/contact-center/catalog/oya-contact-center-voice-routing-adapter-valkey.yaml

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- microservices/contact-center/catalog/oya-contact-center-voice-routing-adapter-redis.yaml -> microservices/contact-center/catalog/oya-contact-center-voice-routing-adapter-valkey.yaml

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states manifest-matching RTO 3600s/RPO 300s for voice routing, queue rebalance, agent-state sync, callback scheduling, recording-consent, and emergency bypass; HIPAA/PCI/KR-PIPA/SOC2/ISO floors; active-active routing and consent metadata; and runbooks for DR failover, PSTN failover, call-drop burn, queue overflow, recording consent mismatch, and emergency bypass audit. Rejected PCI-only 86400/3600 because realtime contact routing has stronger tenant-impact requirements. Cost: active-active routing requires extra warm sockets and provider failover evidence.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.20 vCPU, 384 MiB RAM, 10 GiB storage, 4 Postgres, 10 Valkey, 36 outbound HTTP sockets, `per_message` scaling, Tier-3 capacity class, and 2 to 12 realtime replicas per paid tenant. Rejected per-user scaling because channel events, not licensed seats, drive surge load. Cost: routing headroom is expensive during idle periods but protects queues during spikes.
- Sustainability and cost attribution, ADR-0344: PRD now requires cost, CO2, and watt-hour fields on voice, queue, agent-state, consent, redaction, callback, bypass, and export audit rows; carbon routing is disabled for live voice, HIPAA-EM, and PCI realtime paths and enabled for async QA/export/backfill. Rejected carbon routing for live calls because routing latency and emergency handling are hard constraints. Cost: async jobs carry additional provider and pack dimensions.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected URL-only versioning because agent desktops, supervisor tools, and vendor migrations need pinned SDK and proto compatibility. Cost: migration support spans multiple vendor adapters at once.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.20 vCPU, 384 MiB RAM, 10 GB storage, and 10/4/36 connections per tenant; voice routing, consent markers, and recordings make this the heaviest app baseline in the bucket.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, multi-region active-active true, backup substrate valkey_cluster, postgres_wal_g, object_storage_versioned, seaweedfs_replicated, failover runbook runbooks/pstn-provider-failover.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/contact-center/PRD.md, microservices/contact-center/ARCHITECTURE.md, microservices/contact-center/IP-026-omnichannel-routing-policy-engine.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, kafka, opentelemetry, opentofu, openbao; no local stewardship override declared. Kafka and Valkey support real-time contact event fan-out and agent-state coordination.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/valkey-cluster@v1, aws-guest/postgres-wal-g@v1, oyatie-as-cloud-provider/seaweedfs-replicated@v1, oci-guest/voice-provider-egress-policy@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
