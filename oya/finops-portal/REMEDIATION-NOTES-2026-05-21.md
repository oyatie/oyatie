# FinOps Portal remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- No source files required rewrite; the assigned service already had zero Redis references during Wave 15-Valkey inventory.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states manifest-matching RTO 3600s/RPO 300s while noting SOX journal-control RPO 60 where the process floor applies; active-active covers invoice finalization, credit ledger, FOCUS metadata, regulator evidence, and six-axis rollups; failover references multi-region strategy, anomaly, bill mismatch, FOCUS export, and regulator emit runbooks. Rejected a flat 15m RPO because financial-control evidence has stricter process floors. Cost: evidence paths need more frequent replication and restore drills than dashboard reads.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.12 vCPU, 256 MiB RAM, 6 GiB storage, 6 Postgres, 3 Valkey, 20 outbound HTTP sockets, `per_query` scaling, Tier-2 dashboard placement with Tier-1 regulator/SOX export placement, and 1 to 10 replicas per tenant. Rejected pure per-tenant scaling because invoice drilldown and anomaly explanation are query driven. Cost: month-end and quarterly worker lanes must be reserved.
- Sustainability and cost attribution, ADR-0344 plus ADR-0337: PRD now makes this µservice the transparency surface for the six-axis tenant/product/capability/provider/cell/compliance_pack model and states that audit rows and rollup facts carry cost, CO2, and watt-hour fields; Iceberg-backed rollup refresh stays on the canonical OLAP path. Rejected hiding sustainability data behind OpenCost-only views because customers need tenant-facing explanation. Cost: rollup refresh and export jobs carry warehouse and catalog operational load.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected finance-pipeline hard cuts because close calendars and regulator workflows are tenant-specific. Cost: invoice, FOCUS, and evidence consumers need multi-version contract testing.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.12 vCPU, 256 MiB RAM, 6 GB storage, and 3/6/20 connections per tenant; analytical FinOps dashboards and exports make per_query the right axis.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, multi-region active-active true, backup substrate postgres_wal_g, object_storage_versioned, iceberg_snapshot, clickhouse_iceberg_layered, failover runbook runbooks/quarterly-regulator-emit-miss.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence microservices/finops-portal/PRD.md, microservices/finops-portal/ARCHITECTURE.md, microservices/finops-portal/IP-journey-j94-sox404-public-company-controls.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, iceberg, clickhouse, opentelemetry, opentofu, openbao; no local stewardship override declared. Iceberg and ClickHouse are declared because FOCUS exports and allocation analytics depend on the OLAP layer.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/postgres-wal-g@v1, oyatie-as-cloud-provider/iceberg-snapshot@v1, oyatie-as-cloud-provider/clickhouse-iceberg-layered@v1, on-prem/openbao-policy@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
