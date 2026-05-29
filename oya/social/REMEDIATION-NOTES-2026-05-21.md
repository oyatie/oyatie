# REMEDIATION-NOTES - social - 2026-05-21

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with EU-AI/HIPAA/KR-PIPA/SOC2/ISO27001/KR-CSAP floors cited. Rejected generic feed-cache recovery because moderation, appeals, abuse reports, and ranking evidence are regulated tenant-visible paths. Cost: active-active post/profile/moderation replication, safe-mode feed behavior, and media requeue drills.
- Capacity model: declared manifest `0.22 vCPU`, `512 MiB RAM`, `20 GiB storage`, Valkey/Postgres/outbound baselines, `per_request` scaling, Tier-3 placement, `pod_runtime_tier=2`, and `4..96` replica bounds per ADR-0340. Rejected media-throughput-only sizing because viral feed spikes and safety workflows compete for capacity. Cost: reserved moderation/feed capacity plus isolated transcode/classifier queues.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on post, feed, media, moderation, copyright, notification, and search audit rows per ADR-0344. Rejected blended social cost accounting because media and ranking dominate emissions differently from moderation. Cost: per-capability metering and carbon-routing exclusions for EU-AI/minor-safety paths.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet, SDK semver, N=3/180-day support, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected mobile-SDK-version-only governance because REST, ActivityPub, proto events, and moderation tooling need one public carrier. Cost: compatibility matrix across web/mobile/federation clients.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because the PRD has media/feed analytics but no Iceberg warehouse writer evidence.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.22 vCPU, 512 MiB RAM, 20 GB storage per tenant; connections valkey=6, postgres=4, outbound_http=8; scaling_dimension=per_request; cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: Tier-1 substrate placement because feed, follow graph, and moderation are product responsibilities, not shared substrate ownership.
- Cost: product-cell capacity must reserve cache and object-store headroom for feed spikes and safety queues.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey_cluster, audit_chain_merkle_seal.
- ADR: ADR-0343 plus compliance-pack floors.
- Rejected: cache-only feed rebuild because moderation evidence, profile state, and audit seals must survive region loss.
- Cost: active-active warm replication and quarterly drills are required for the declared runbook.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/social/PRD.md, microservices/social/IP-006-feed-timeline-bc.md, microservices/social/IP-011-content-moderation-bc.md, microservices/social/runbooks/feed-cache-rebuild.md.
- ADR: ADR-0338 runtime tiering and ADR-0340/ADR-0248 co-variance with Tier-3 cells.
- Rejected: Tier-1 because social handles sensitive tenant product data but does not own foundation keys or shared substrate custody.
- Cost: runtime placement remains first-party app isolation with product SLO and incident severity.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public contracts.
- Rejected: mobile-SDK-only governance because REST, AsyncAPI, and proto surfaces need the same tenant-pinned carrier.
- Cost: future breaking changes require deprecation calendar and migration docs across three supported windows.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, kafka, opentelemetry, opensearch; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship override because the registry default owner teams are sufficient for this product service.
- Cost: SBOM and CVE-response evidence must trace feed, search, moderation, and telemetry dependencies.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/dns@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: wrapper-only local IaC because shared module pins are the admission surface.
- Cost: module pin upgrades must be deliberate when cloud-iac publishes new primitives.
