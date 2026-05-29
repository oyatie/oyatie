---
ip_id: IP-044
microservice: data-warehouse
title: SQL-callable ML / LLM
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-05
binding_adrs: [ADR-0131, ADR-0145, ADR-0255, ADR-0329]
counterpart_parity: BigQuery ML + Snowflake Cortex + Databricks Mosaic AI
capabilities_touched: [sql-ml-train]
billing_components: [ml_training_units]
---

# IP-044 — SQL-callable ML / LLM

## §1 Objective

Land SQL-callable model training + inference inside `data-warehouse`. The
warehouse owns the call site (`CREATE MODEL`, `SELECT PREDICT(…)`); the
`intelligence` µservice owns the model registry, model weights, and
inference compute. BigQuery ML, Snowflake Cortex, and Databricks Mosaic AI
all sit in this slot.

Closes F-D4-D-05 ("SQL-callable ML / LLM — Missing primitive").

## §2 Scope

In scope:

- `CREATE MODEL` for tabular models (ARIMA, gradient-boosted trees,
  linear, logistic).
- `CREATE REMOTE MODEL` binding to a foreign LLM (OpenAI, Anthropic,
  Mistral, oyatie's own; per `intelligence` µservice catalog).
- `SELECT PREDICT(MODEL=…, …)` SQL function.
- `SELECT EMBED(MODEL=…, text)` SQL function for vector use cases.
- AutoML (BigQuery-style); tenant configurable.

Out of scope:

- Model fine-tuning UX (intelligence µservice).
- Real-time model serving for non-SQL callers (intelligence µservice).

## §3 Architecture

### §3.1 Training

```sql
CREATE MODEL revenue_forecast
  OPTIONS(model_type='ARIMA', time_series_col='date', value_col='revenue')
AS SELECT date, revenue FROM orders_daily;
```

The warehouse:

1. Materializes the training data into a staged location.
2. Calls `intelligence.train(...)` over direct gRPC (ADR-0145).
3. Receives a model reference, registers it in the catalog as
   `kind=model_reference`.
4. Accrues `ml_training_units` based on the trainer's reported unit count.

### §3.2 Inference

```sql
SELECT date, PREDICT(MODEL=revenue_forecast, date) AS forecast
FROM future_dates;
```

The warehouse calls `intelligence.predict(...)` for each row (with
batching). Per-prediction accrual is to whatever the model is configured
as (BYOK LLM provider charges the tenant; oyatie default models charge
`ml_training_units` for inference too).

### §3.3 BYOK

If the model is a remote LLM with tenant BYOK credentials (ADR-0255 §D-4),
the intelligence µservice forwards the call with tenant credentials.
Oyatie does not store the tenant's API key.

## §4 Cedar binding

`local-warehouse-query-access.cedar` extends — `MODEL.invoke` requires
`paid` tenant_class + `ml_training_units` billing enabled.

## §5 SLO bindings

Inference latency is graded inside intelligence µservice; the warehouse
adds ≤ 50 ms overhead per prediction call.

## §6 Failure modes

- Intelligence µservice unavailable → query fails with
  `intelligence_unavailable`.
- Tenant BYOK credentials invalid → query fails with
  `byok_credential_invalid`.
- Model not found in catalog → `model_not_registered`.

## §7 Acceptance criteria

- A `paid` tenant trains an ARIMA model from warehouse data.
- `SELECT PREDICT(…)` returns forecasts.
- `ml_training_units` accrues.
- A `demo_trial` tenant cannot train.

## §8 Risks

- The SQL ↔ intelligence µservice round-trip latency is the bottleneck;
  batching and caching are essential.

End of IP-044.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-044-sql-callable-ml-llm.md` matched `SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
