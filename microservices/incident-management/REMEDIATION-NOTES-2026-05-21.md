# Incident Management remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/incident-management/decisions/ADR-IM-001-escalation-routing-and-incident-command-state-machine.md
- microservices/incident-management/performance-benchmark-numbers-2026-05-20.md
- microservices/incident-management/catalog/oya-incident-management-sre-incident-command-adapter-valkey.yaml

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- microservices/incident-management/catalog/oya-incident-management-sre-incident-command-adapter-redis.yaml -> microservices/incident-management/catalog/oya-incident-management-sre-incident-command-adapter-valkey.yaml

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states manifest-matching RTO 3600s/RPO 900s for paging, escalation, incident-room, stakeholder-update, and status-update paths; HIPAA/KR-CSAP/SOC2/ISO floors; active-active command recovery; and runbooks for DR failover, paging storms, mobile push, incident-room creation, and statuspage provider outage. Rejected overriding the manifest with a stricter live-paging target in this PRD pass because D-2 manifest values are authoritative. Cost: active HIPAA deployments need an RPO 300 override below the baseline.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.16 vCPU, 256 MiB RAM, 2 GiB storage, 4 Postgres, 8 Valkey, 32 outbound HTTP sockets, `per_message` scaling, Tier-2 capacity class, and 2 to 12 command replicas per paid tenant. Rejected treating postmortem and paging as one capacity class because event streams, not request count, govern the hot path. Cost: paging worker caps protect providers but may queue non-critical notifications.
- Sustainability and cost attribution, ADR-0344: PRD now requires cost, CO2, and watt-hour fields on on-call, escalation, incident-room, status-update, page-dispatch, and postmortem audit rows; carbon routing is disabled for live emergency command and enabled for postmortem/export/backfill. Rejected carbon-preferred routing for page dispatch because emergency correctness and latency are mandatory. Cost: live paths still collect emissions metadata even when routing ignores carbon.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected a hard-cut public API migration because PagerDuty/OpsGenie/xMatters and mobile clients roll tenant by tenant. Cost: on-call integrations require longer compatibility test coverage.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.16 vCPU, 256 MiB RAM, 2 GB storage, and 8/4/32 connections per tenant; paging and status fan-out are bursty per_message workloads.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 3600s, RPO 900s, multi-region active-active true, backup substrate valkey_cluster, postgres_wal_g, object_storage_versioned, failover runbook runbooks/paging-storm.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/incident-management/PRD.md, microservices/incident-management/ARCHITECTURE.md, microservices/incident-management/IP-029-firehydrant-rootly-incident-command-postmortem-displacement.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, kafka, opentelemetry, opentofu, openbao; no local stewardship override declared. Kafka is declared for incident event fan-out and backlog replay; no local stewardship-class override is justified.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/valkey-cluster@v1, aws-guest/postgres-wal-g@v1, oci-guest/notification-egress-policy@v1, oyatie-as-cloud-provider/object-storage-versioned@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
