# REMEDIATION-NOTES - calendar - 2026-05-21

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with HIPAA/KR-PIPA/SOC2/ISO27001/KR-CSAP floors cited. Rejected plain cross-region replication language because calendar must respect pack-pinned region pairs and free/busy degradation semantics. Cost: active-active event/invitation state, calendar restore drills, and availability-cache rebuild ownership.
- Capacity model: declared manifest `0.08 vCPU`, `192 MiB RAM`, `3 GiB storage`, Valkey/Postgres/outbound baselines, `per_request` scaling, Tier-3 placement, `pod_runtime_tier=2`, and `3..48` replica bounds per ADR-0340. Rejected event-count-only sizing because recurrence, free/busy, CalDAV, room-booking, and import/export paths stress different resources. Cost: separate worker caps and recurrence/import throttles.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on event, free/busy, recurrence, room, invitation, CalDAV, and import/export audit rows per ADR-0344. Rejected tenant-average scheduling cost because recurrence expansion and room analytics can dominate emissions. Cost: per-capability metering plus carbon-routing exclusions for live scheduling/HIPAA/legal-hold paths.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet, SDK semver, N=3/180-day support, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected RFC-only compatibility because oyatie REST/CalDAV extensions and proto events require governed public versioning. Cost: versioned bridge testing and partner migration support.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because calendar has no Iceberg warehouse writer evidence.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.08 vCPU, 192 MiB RAM, 3 GB storage per tenant; connections valkey=3, postgres=3, outbound_http=4; scaling_dimension=per_request; cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: per_user-only sizing because recurrence, free/busy, CalDAV/ICS sync, and invite fanout scale with schedule requests.
- Cost: product-cell capacity must reserve recurrence and availability-cache headroom.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal.
- ADR: ADR-0343 plus compliance-pack floors.
- Rejected: plain database restore because calendar recovery also needs availability cache rebuild and audit seal continuity.
- Cost: active-active event/invitation state and quarterly restore drills are required.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/calendar/PRD.md, microservices/calendar/IP-005-recurrence-engine.md, microservices/calendar/IP-006-availability-resolver.md, microservices/calendar/runbooks/calendar-restore.md.
- ADR: ADR-0338 runtime tiering and ADR-0340/ADR-0248 co-variance with Tier-3 cells.
- Rejected: Tier-1 because calendar handles sensitive tenant product data but does not own shared substrate key custody.
- Cost: runtime placement remains first-party app isolation with product SLO and incident severity.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public contracts.
- Rejected: RFC-only compatibility because Oyatie REST, CalDAV extensions, and proto events require governed public versioning.
- Cost: future breaking changes require deprecation calendar and migration docs across three supported windows.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship override because registry defaults already cover data, policy, event, and telemetry owners.
- Cost: SBOM and CVE-response evidence must trace scheduling, cache, and event dependencies.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper IaC because module reuse and pinning are the admission surface.
- Cost: module pin upgrades must be deliberate when cloud-iac publishes new primitives.
