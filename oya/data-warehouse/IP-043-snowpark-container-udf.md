---
ip_id: IP-043
microservice: data-warehouse
title: Snowpark Container UDF (procedural runtime)
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-04
binding_adrs: [ADR-0131, ADR-0145, ADR-0254, ADR-0329]
counterpart_parity: Snowflake Snowpark Container Services + BigQuery Remote Functions + Databricks Custom Container
capabilities_touched: [container-udf-execute]
billing_components: [container_udf_seconds, compute_credits]
---

# IP-043 — Snowpark Container UDF (procedural runtime)

## §1 Objective

Land a Snowpark-Container-Services-class procedural runtime so tenants can
register and invoke arbitrary container images as UDFs / UDTFs / UDAFs
inside SQL queries. Snowflake's Snowpark Container Services + Snowpark
Python/Scala/Java UDFs anchor this; BigQuery Remote Functions and
Databricks custom container compute sit in the same slot.

Closes F-D4-D-04 ("Procedural runtime / Container UDF — Missing primitive").

## §2 Scope

In scope:

- Container UDF registration (`CREATE FUNCTION … LANGUAGE CONTAINER …`).
- Per-tenant container registry binding (`cloud-marketplace` µservice
  signs / scans images).
- Sandboxed execution on Cloud Hypervisor pods (ADR-0254) with tenant-bound
  network policy.
- CPU + GPU container support.
- Per-invocation streaming arguments + streaming output.

Out of scope:

- Stored procedure language extension (Wave-15B).
- Cross-tenant function sharing (Wave-15B).

## §3 Architecture

### §3.1 Registration

```sql
CREATE FUNCTION classify_text(text STRING) RETURNS STRING
  LANGUAGE CONTAINER
  IMAGE 'oyatie-tenant-<uuid>/classify-text:v3.2'
  CPU 2
  MEMORY '8GiB'
  GPU 0
;
```

The image must be signed by `cloud-marketplace` µservice; unsigned images
are refused.

### §3.2 Execution model

Each invocation lands on a warm Cloud Hypervisor pod from the tenant's UDF
pool. Cold-start is ≤ 2 s; warm invocation ≤ 100 ms p99 (function body
runtime excluded).

### §3.3 Network policy

The pod has zero outbound network access by default. The tenant can
declare an explicit allow-list of egress destinations.

### §3.4 Billing

- `container_udf_seconds` accrues per CPU-second consumed.
- `compute_credits` accrues to the wrapping SQL query.

## §4 Cedar binding

`local-warehouse-query-access.cedar` extends — a query using a Container
UDF requires the user's principal to have `udf.invoke` on the function
catalog entry.

## §5 SLO bindings

- Cold-start ≤ 2 s p99 (graded informally; no separate OpenSLO file
  in Wave-15A).

## §6 Failure modes

- Image unsigned → registration refused.
- Container OOM → invocation fails with `container_oom`; SQL row treated
  as NULL or error per function declaration.
- Network egress to non-allowlisted destination → blocked at network
  policy.

## §7 Acceptance criteria

- A `paid` tenant registers a signed image as a UDF.
- A SQL query calling the UDF executes and accrues
  `container_udf_seconds`.
- A `demo_trial` tenant cannot register a Container UDF.
- An unsigned image is refused at registration.

## §8 Risks

- Container UDF gives the tenant a foothold for custom compute; sandbox
  hardening is critical (Cloud Hypervisor + tenant network policy).

End of IP-043.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-043-snowpark-container-udf.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Pod runtime tier (per ADR-0338)
- `pod_runtime_tier: 0`
- Runtime: Kata Containers plus Cloud Hypervisor are REQUIRED for this tenant-customer execution path.
- Justification: this IP matched `sandbox`, so tenant-customer or third-party code can enter the execution path.
- Surface evidence: `microservices/data-warehouse/IP-043-snowpark-container-udf.md` plus `microservices/data-warehouse/capabilities/container-udf-execute.yaml, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
