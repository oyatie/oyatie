# REMEDIATION-NOTES - connect - 2026-05-21

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with EU-AI/HIPAA/SOC2/ISO27001/PCI floors cited. Rejected product-retirement deferral because the connector substrate still owns accepted webhooks, OAuth grants, DLQ evidence, and tenant side effects. Cost: active-active webhook/OAuth admission, idempotent replay controls, and DLQ storage replication.
- Capacity model: declared manifest `0.06 vCPU`, `128 MiB RAM`, `1 GiB storage`, Valkey/Postgres/outbound baselines, `per_request` scaling, Tier-1 placement, `pod_runtime_tier=1`, and `2..40` replica bounds per ADR-0340. Rejected a generic workflow-engine capacity model because vendor rate limits and webhook admission are connect-owned bottlenecks. Cost: token-bucket sharding and per-vendor queue isolation.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on OAuth, webhook, connector-action, DLQ, mapping, and audit rows per ADR-0344. Rejected marketplace-level cost accounting because tenants need cost by connector wiring and provider. Cost: metering per connector/action/provider plus carbon-routing exclusions for PCI/fraud/HIPAA-emergency paths.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet, connector SDK semver, N=3/180-day support, tenant/publisher pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected semver-only connector adapter governance because webhook URLs and callback payloads need date-pinned public contracts. Cost: per-tenant callback compatibility testing and pinned adapter lifecycle.
- Frontmatter: added ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0339 was not added because `connect` has no child `iac/<context>/` directories; ADR-0337 was not added because no Iceberg warehouse writer evidence exists.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.06 vCPU, 128 MiB RAM, 1 GB storage per tenant; connections valkey=1, postgres=2, outbound_http=8; scaling_dimension=per_request; cell_placement_class=Tier-1.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: retirement-only lightweight sizing because OAuth broker, webhook admission, and revocation evidence still carry tenant side effects.
- Cost: Tier-1 cell capacity must reserve token and webhook admission overhead until Connect is fully retired.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal.
- ADR: ADR-0343 plus compliance-pack floors.
- Rejected: deferring DR to successor services because accepted webhooks and OAuth grants remain Connect-owned until retirement completes.
- Cost: active-active webhook/OAuth admission and quarterly failover drills are required.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/connect/PRD.md, microservices/connect/IP-003-oauth-broker-domain-kernel.md, microservices/connect/runbooks/oauth-token-revocation-cascade.md, microservices/connect/iac/openbao-policy.hcl.
- ADR: ADR-0338 runtime tiering and ADR-0340/ADR-0248 co-variance with Tier-1 cells.
- Rejected: Tier-2 because OAuth/OpenBao token custody is substrate-touching tenant data despite product retirement.
- Cost: runtime placement inherits substrate isolation until the retirement plan removes these responsibilities.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public contracts.
- Rejected: semver-only connector governance because webhook URLs and callback payloads need date-pinned compatibility.
- Cost: future breaking changes require deprecation calendar and migration docs across three supported windows.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, cedar, openbao, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship override because registry defaults already cover policy, key, event, and telemetry owners.
- Cost: SBOM and CVE-response evidence must trace connector and OAuth dependencies.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1, oyatie-as-cloud-provider/dns@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: no-IaC declaration because the service has live IaC assets and must declare shared module pins.
- Cost: module pin upgrades must be deliberate when cloud-iac publishes new primitives.
