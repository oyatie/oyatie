---
ip_id: IP-034
microservice: data-warehouse
title: Unity-Catalog-class 3-level namespace
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-02
binding_adrs: [ADR-0131, ADR-0145, ADR-0243, ADR-0244, ADR-0329]
counterpart_parity: Databricks Unity Catalog
capabilities_touched: [unity-catalog-namespace-bind, governed-share-create]
billing_components: []
---

# IP-034 — Unity-Catalog-class 3-level namespace

## §1 Objective

Land the Unity-Catalog-class 3-level namespace (`catalog.schema.object`)
inside `data-warehouse`. This is the metadata layer that binds Delta /
Iceberg / Hudi tables, semantic-model views, ML model references, and
governed shares to Cedar entities under a single tenant-scoped tree.

This closes F-D4-L-02 ("Unity-Catalog-class 3-level namespace governance").

## §2 Scope

In scope:

- The 3-level namespace tree: `catalog → schema → object`.
- The `object` family: `lake_table`, `semantic_view`, `share`,
  `external_table`, `materialized_view`, `vector_index`, `model_reference`.
- Tenant scoping: every catalog is owned by exactly one `tenant_id`; the
  reserved `oyatie` tenant owns the system catalog.
- Cedar binding: every `object` is a Cedar entity; ACLs are evaluated by
  the substrate `compliance` µservice.
- Lineage column-level capture (read-side projection to `ontology`).
- System tables (Databricks-class): `system.access`, `system.billing`,
  `system.lineage`, `system.query_history`.

Out of scope:

- Cross-µservice catalog (oyatie has a separate `oya-catalog-shared-*`
  family that holds the µservice-level layer-registration catalog; this IP
  is the *table-namespace* catalog, not that).

## §3 Architecture

### §3.1 Storage

The catalog is a Postgres-backed (per `cloud-data` µservice direct gRPC) row
store with the following key tables (per tenant):

```
catalogs(catalog_id, tenant_id, name, owner_principal_id, created_at)
schemas(schema_id, catalog_id, name, owner_principal_id, created_at)
objects(object_id, schema_id, kind, name, ref, owner_principal_id, version,
        created_at, last_updated_at, deleted_at)
object_acls(object_id, principal_pattern, action, decision)
lineage_edges(downstream_object_id, downstream_column,
              upstream_object_id, upstream_column, transform_op, version)
```

### §3.2 Resolution

`catalog.schema.object` resolves to an `object` row via two indexed lookups.
P99 latency target ≤ 30 ms.

### §3.3 Cedar entity model

```cedar
entity Catalog in DataWarehouse {
  tenant_id: String,
};
entity Schema in DataWarehouse {
  catalog_id: String,
};
entity DwObject in DataWarehouse {
  schema_id: String,
  kind: String, // lake_table, semantic_view, etc.
};
```

ACLs in `object_acls` are pre-evaluated and emitted to the compliance
µservice on every change. The catalog is the *source of truth*; compliance
is the *enforcement point*.

### §3.4 System tables

- `system.access` — Cedar decision rows for objects in this tenant.
- `system.billing` — per-`billing_component` accrual rows.
- `system.lineage` — column-level lineage edges.
- `system.query_history` — last 30 days of queries (paid only).

All system tables are read-only views over the catalog tables; they expose
no mutation API.

## §4 Cedar binding

Every CRUD action on `Catalog`, `Schema`, or `DwObject` requires Cedar
evaluation. The default-deny stance from `local-warehouse-query-access.cedar`
is extended in `local-unity-catalog-class.cedar` (new in Wave-15A).

## §5 Billing accrual

The catalog itself is free — no `billing_component` accrues to catalog
operations. The objects it points to accrue per their own capabilities.

## §6 SLO bindings

- `slos/unity-catalog-namespace-resolve-latency.openslo.yaml` — p99
  resolve ≤ 30 ms.

## §7 Failure modes

- Catalog DB unavailable → resolve fails with `catalog_unavailable`;
  warehouse query path falls back to per-object cache (TTL ≤ 5 s).
- Tenant rename → forbidden (rename a catalog/schema; tenant IDs are
  immutable).
- Object name collision → refused with `object_already_exists`.
- Cross-tenant object reference → refused.

## §8 Acceptance criteria

- A new catalog is creatable by a tenant operator.
- A schema under that catalog is creatable.
- A Delta / Iceberg / Hudi table is registrable as an `object` of kind
  `lake_table`.
- The `system.access` view returns Cedar decision rows for the last 30
  days.
- The `system.lineage` view returns column-level lineage for any
  published dataset.
- Cross-tenant object reference is refused at the catalog layer (before
  Cedar even fires).

## §9 Risks

- The catalog DB row count for a large multi-tenant deployment is
  substantial; partition by tenant_id at the DB layer.
- Catalog vs Cedar PEP cache invalidation is a known hard problem; we
  bound staleness at ≤ 5 s.

End of IP-034.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-034-unity-catalog-class-namespace.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
