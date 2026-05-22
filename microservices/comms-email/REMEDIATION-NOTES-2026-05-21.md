# REMEDIATION-NOTES - comms-email - 2026-05-21

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with EU-AI/HIPAA/KR-PIPA/SOC2/ISO27001/PCI floors cited. Rejected provider-native retry as the only recovery model because accepted transactional email must preserve DKIM, suppression, and audit evidence. Cost: active-active enqueue/provider routing, Postal/SES failover drills, and replicated suppression state.
- Capacity model: declared manifest `0.08 vCPU`, `192 MiB RAM`, `2 GiB storage`, Valkey/Postgres/outbound baselines, `per_message` scaling, Tier-1 placement, `pod_runtime_tier=1`, and `3..48` replica bounds per ADR-0340. Rejected send-rate-only sizing because webhook ingest, DKIM rotation, and suppression lookups have separate bottlenecks. Cost: queue partitioning by tenant/provider and separate webhook lag autoscaling.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on sends, templates, suppression, provider calls, webhook events, and audit emissions per ADR-0344. Rejected provider-invoice-only accounting because Postal and SaaS routes have different tenant and emissions profiles. Cost: provider-level metering and carbon-routing exclusions for transactional/HIPAA/EU-AI notices.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet, SDK semver for the shared EmailComms trait, N=3/180-day support, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected adapter-version-only compatibility because webhook consumers need stable payload contracts. Cost: versioned send/webhook contracts and tenant pin state.
- Frontmatter: created PRD frontmatter and added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because transactional email emits audit/provider events, not Iceberg warehouse writes.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.08 vCPU, 192 MiB RAM, 2 GB storage per tenant; connections valkey=2, postgres=2, outbound_http=10; scaling_dimension=per_message; cell_placement_class=Tier-1.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: product Tier-3 placement because DKIM key custody, suppression state, and provider failover are shared communications substrate responsibilities.
- Cost: Tier-1 cell capacity must reserve signing-key and provider-route overhead per tenant.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal.
- ADR: ADR-0343 plus compliance-pack floors.
- Rejected: provider-native retry as the recovery plan because accepted email, DKIM custody, and audit evidence are Oyatie-owned.
- Cost: active-active enqueue/provider routing and quarterly failover drills are required.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/comms-email/PRD.md, microservices/comms-email/IP-005-dkim-key-rotation-pipeline.md, microservices/comms-email/runbooks/dkim-key-rotation.md, microservices/comms-email/iac/openbao-policy.hcl.
- ADR: ADR-0338 runtime tiering and ADR-0340/ADR-0248 co-variance with Tier-1 cells.
- Rejected: Tier-0 because comms-email does not execute tenant-authored code; rejected Tier-2 because it owns tenant-domain signing custody.
- Cost: runtime placement inherits substrate isolation and incident severity.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public contracts.
- Rejected: adapter-version-only compatibility because webhook consumers need stable send and delivery payloads.
- Cost: future breaking changes require deprecation calendar and migration docs across three supported windows.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, openbao, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship override because registry defaults already cover data, key, event, and telemetry owners.
- Cost: SBOM and CVE-response evidence must trace each declared upstream.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/dns@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper IaC because module reuse and pinning are the admission surface.
- Cost: module pin upgrades must be deliberate when cloud-iac publishes new primitives.
