# EMR Remediation Notes - 2026-05-21

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/emr/README.md
- microservices/emr/PRD.md
- microservices/emr/ARCHITECTURE.md
- microservices/emr/decisions/ADR-MS-001-bounded-contexts.md
- microservices/emr/implementation-plans/IP-003-postgres-citus-adapters.md
- microservices/emr/implementation-plans/IP-009-portal-session-valkey.md

Counterpart-fact preservations:
- None.

Files renamed:
- microservices/emr/implementation-plans/IP-009-portal-session-redis.md -> microservices/emr/implementation-plans/IP-009-portal-session-valkey.md
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 10.
- Trigger A matched: 3.
- Trigger B matched: 3.
- Trigger C matched: 2.
- Trigger D matched: 1.
- IPs unmatched: 5.

### IP changes
- `microservices/emr/implementation-plans/IP-004-rest-fhir-r5-r4-bridge.md` — added API Versioning, Sustainability emission.
- `microservices/emr/implementation-plans/IP-005-asyncapi-events.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/emr/implementation-plans/IP-006-grpc-internal-handoffs.md` — added API Versioning.
- `microservices/emr/implementation-plans/IP-008-timescale-vital-hypertable.md` — added DR posture.
- `microservices/emr/implementation-plans/IP-010-app-composition-deploy.md` — added DR posture, Pod runtime tier.

### Unmatched IPs
- `microservices/emr/implementation-plans/IP-001-bc-scaffold-kernel-domain.md` — no trigger match; no doctrine section added.
- `microservices/emr/implementation-plans/IP-002-usecase-application-layers.md` — no trigger match; no doctrine section added.
- `microservices/emr/implementation-plans/IP-003-postgres-citus-adapters.md` — no trigger match; no doctrine section added.
- `microservices/emr/implementation-plans/IP-007-workers-bcma-vitals-bulk.md` — no trigger match; no doctrine section added.
- `microservices/emr/implementation-plans/IP-009-portal-session-valkey.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/emr/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `emr`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 900s, RPO 60s, multi-region active-active, failover_runbook `microservices/emr/runbooks/emr-cell-failover.md`.
- ADR: ADR-0343; HIPAA/SOC2/ISO/KR-PIPA floors are looser than the clinical-record target.
- Alternative considered: inherit only HIPAA 3600s/300s; rejected because chart and order continuity need a tenant-visible 15-minute/60-second bound.
- Cost: requires active-active PHI replication, quarterly drills, and a new runbook file.

### Capacity model
- Values: 0.75 vCPU, 1536 MiB RAM, 35 GB storage, 8 Postgres connections, 6 Valkey connections, 10 outbound HTTP connections; `per_request` scaling; Tier-2 placement; 2-40 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: scale by user sessions; rejected because the manifest declares chart/order/portal/FHIR API request pressure as the capacity driver.
- Cost: reserves PHI application data-plane capacity without claiming Tier-0 substrate ownership.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for live chart/order/HIPAA emergency paths and enabled for de-identified exports.
- ADR: ADR-0344.
- Alternative considered: aggregate emissions nightly; rejected because tenant evidence must align to audit-chain rows.
- Cost: adds per-call metering overhead and finops cardinality.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, per-tenant pinning, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: SDK semver only; rejected because FHIR and portal contracts need tenant-visible date pinning.
- Cost: maintains three concurrent public contract versions and pin metadata.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.75 vCPU, 1536 MiB RAM, 35 GB storage, and per_request scaling come from EMR chart, order, medication, portal, and FHIR API pressure rather than per-user session count.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-3 placement because pod_runtime_tier=1 cannot co-vary with Tier-3 under ADR-0340 D-6.
- Cost: Commits EMR to Kata/nodepool overhead and cross-cell chart replication capacity instead of cheaper runc-only app placement.

### Block 2: dr
- Values: RTO 900s, RPO 60s, active-active true, backup substrates postgres_wal_g, valkey_cluster, object_storage_versioned, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected generic HIPAA 3600s RTO because chart-of-truth outage would block medication reconciliation and encounter continuity.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/emr-cell-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; EMR owns the longitudinal PHI chart-of-truth and touches tenant clinical data across clinician, portal, order, medication, and billing workflows. It does not execute tenant-customer code, so Tier 0 is not justified, but the blast radius and HIPAA data-plane duties require Tier 1 isolation and Tier-2 cell placement.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 0 because there is no tenant-customer code execution surface in the manifest, PRD, architecture, or contracts.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only the current v1 contract files exist in-tree.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, valkey, cedar, openbao, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected declaring non-registry names so the manifest stays aligned with specs/oss-stewardship-registry.json.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/valkey-cluster, oci-guest/tenant-namespace, oci-guest/postgres-wal-g, oci-guest/valkey-cluster, on-prem/tenant-namespace, colo/tenant-namespace, oyatie-as-cloud-provider/shard-cell, oyatie-as-cloud-provider/per-cell-nodepool-kata.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected legacy context spellings such as guest-on-aws and oyatie-public-cloud in the manifest because the schema requires canonical context enums.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
