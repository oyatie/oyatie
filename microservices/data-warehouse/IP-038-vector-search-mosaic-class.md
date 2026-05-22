---
ip_id: IP-038
microservice: data-warehouse
title: Vector search (Mosaic-AI-class)
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-06
binding_adrs: [ADR-0131, ADR-0145, ADR-0255, ADR-0329]
counterpart_parity: Databricks Mosaic AI Vector Search + BigQuery vector search + Snowflake Cortex Search
capabilities_touched: [vector-index-serve]
billing_components: [vector_index_serving]
---

# IP-038 — Vector search (Mosaic-AI-class)

## §1 Objective

Land vector indexing + similarity search inside `data-warehouse` for
tenant-scoped semantic lookup over warehouse data. This is the Mosaic-AI-class
vector surface; Snowflake Cortex Search and BigQuery `VECTOR_SEARCH` sit in
the same slot.

Closes F-D4-D-06 ("Vector search — Not authored at warehouse layer").

## §2 Scope

In scope:

- Per-column or per-row vector index on a warehouse table (typically
  Delta).
- Index types: HNSW + IVF-PQ (production); brute-force for small tables
  (≤ 100k rows).
- Embedding model: tenant supplies (via `intelligence` µservice) or
  oyatie default (per `intelligence` µservice catalog).
- Similarity functions: cosine, dot, L2.
- Filtered search (Cedar-evaluated metadata filter at query time).
- Top-k retrieval.

Out of scope:

- Multi-modal index (image + text); Wave-15B.
- Hybrid sparse+dense (BM25 + vector); Wave-15B.

## §3 Architecture

### §3.1 Index storage

The vector index lives alongside the source table:

```
s3://oyatie-<tenant>-warehouse/<catalog>/<schema>/<table>/_vector_index/
  <index_name>/
    manifest.json
    segments/<seg_id>.hnsw      # or .ivfpq
```

### §3.2 Build path

A `lake-table-write` commit triggers an incremental index rebuild for
each affected partition. Full rebuild is on-demand.

### §3.3 Query path

`GET /v1/lake/tables/{name}/vector-search`:

```json
{
  "index": "embeddings_idx",
  "vector": [0.1, 0.2, ...],
  "top_k": 10,
  "filter": "tenant_pack='hipaa' AND status='active'"
}
```

The filter is Cedar-evaluated against the projection (so PHI columns
masked at the semantic layer remain masked through vector search).

### §3.4 Embeddings via intelligence µservice

The data-warehouse does NOT hold embedding-model weights; it calls the
`intelligence` µservice over direct gRPC (ADR-0145) for batch
embedding. The model identifier is per-tenant configurable.

## §4 Cedar binding

`local-vector-search-access.cedar` (new) requires:

- `paid` tenant_class.
- `vector_index_serving` billing component enabled.
- The user passes the per-row Cedar filter check.

## §5 Billing accrual

- `vector_index_serving` accrues per query.
- Index build is `compute_credits` (one-time + incremental).

## §6 SLO bindings

- `slos/vector-search-latency.openslo.yaml` — p99 top-10 search on a 1 M
  vector index ≤ 50 ms.

## §7 Failure modes

- Index out of date (lag > 1 min after write) → emit `vector_index_lag`
  metric; query falls back to brute-force on the latest data.
- Filter eliminates all candidates → return empty.
- Embedding model unavailable → query refused with
  `embedding_model_unavailable`.

## §8 Acceptance criteria

- Build a vector index on a 1 M row Delta table within 30 min.
- Top-10 search in p99 ≤ 50 ms.
- Cedar filter on a HIPAA-masked column denies projection.
- Demo_trial tenant cannot serve queries.

## §9 Risks

- Embedding-model upgrade requires re-index; track per-index model
  version in the manifest.

End of IP-038.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-038-vector-search-mosaic-class.md` matched `p99, SLO, PHI`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
