# `cloud-billing` µservice — Benchmark vs AWS CUR + Billing, GCP Billing, Azure Cost Management

> Measured 2026-04-28 to 2026-05-12 across 3 trial windows × 4 workloads (high-volume metering ingestion, end-of-period close,
> FOCUS export, multi-currency invoice). All vendors over HTTPS/HTTP-2. `cloud-billing` runs HTTP/3 (QUIC) by default per ADR-0253.
> Pricing as of 2026-05-12 from each vendor's published sheet.

## Metering ingestion throughput

| Surface | Sustained (events/sec) | Peak (events/sec) | Median ingest-to-query latency | Schema enforcement |
| --- | --- | --- | --- | --- |
| `cloud-billing` (Paid, Kafka 5×) | **5,000,000** | **18,000,000** | **3.2 s** | strict (proto schema) |
| AWS CUR 2.0 | n/a (vendor-managed; ~24 h batch) | n/a | 24 h | parquet schema |
| GCP Billing Export to BigQuery | n/a (streamed; ~5 m batch) | ~50,000 (BQ streaming) | 5-15 m | BQ schema |
| Azure Cost Mgmt Export | n/a (daily file) | n/a | 24 h | CSV |
| Stripe Sigma | n/a (~1 h batch) | n/a | 1-2 h | Stripe schema |

`cloud-billing` is the only surface here designed for second-fresh metering — vendors are batched at minutes-to-day cadence.

## End-of-period close latency (10 M usage events, mid-market tenant)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-billing` (Paid) | **38 min** | **74 min** | 118 min |
| AWS Billing (CUR generation + invoice) | 4-12 h | 18 h | 24 h |
| GCP Billing (Invoice rendering) | 2-6 h | 8 h | 14 h |
| Azure Cost Management (Invoice export) | 6-18 h | 22 h | 24 h |
| Stripe Invoicing | 30-90 min | 2 h | 3 h |

## FOCUS 1.1 conformance

| Surface | FOCUS 1.1 conformant | Native FOCUS export | Schema validation built-in | Extension columns |
| --- | --- | --- | --- | --- |
| `cloud-billing` | ✅ | ✅ Parquet + Kafka stream | ✅ | `oya_tenant_id`, `oya_cost_center`, `oya_pack_id` |
| AWS CUR 2.0 | partial (mapping table) | preview | ❌ (external tooling) | none |
| GCP Billing | partial | beta | ❌ | none |
| Azure Cost Mgmt | partial | preview | ❌ | none |
| Vantage | ✅ (post-ingest) | ✅ | ✅ | per-Vantage tags |
| CloudZero | ✅ (post-ingest) | ✅ | ✅ | CloudZero tags |

## Multi-currency surface

| Surface | Native currencies | FX rate source | Lock semantics | Sovereign overrides |
| --- | --- | --- | --- | --- |
| `cloud-billing` (Paid) | 28 | ECB-reference-rates-daily | locked at issuance, immutable | KR K-FSI VAT, IN GST, AE 5% VAT, etc. |
| AWS Billing | USD only | n/a | n/a | n/a (must use 3rd-party) |
| GCP Billing | 60+ | Google FX (proprietary) | locked at issuance | partial |
| Azure Cost Mgmt | 20+ | Microsoft FX | locked at issuance | partial |

## Chargeback / cost-center surface

| Surface | Max cost centers | Multi-axis attribution | Transfer pricing | OECD BEPS export |
| --- | --- | --- | --- | --- |
| `cloud-billing` (Paid) | 500 | cost-center × project × pack × region | ✅ (paid) | ✅ Pillar Two GloBE |
| AWS Cost Allocation Tags | unlimited (tag-based) | tag-based only | ❌ (external ERP) | ❌ |
| GCP Cost Allocation Tags | unlimited (tag-based) | tag-based only | ❌ | ❌ |
| Azure Cost Allocation Rules | 200 | rule-based | partial (Cost Management Preview) | ❌ |
| Apptio Cloudability | unlimited | dimension-based | ✅ | partial |

## TCO at 50,000 monthly active users, $2M monthly cloud spend, mid-market scope

| Surface | License | Metering | Export | Anomaly detection | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-billing` (Paid) | included | included | included | included | **$2,800** | **$33,600** |
| AWS CUR + Cost Explorer | $0 | $0 | $0 | $0.10/check (Cost Anomaly Detection) | $300 (anomaly only) + ERP integration ops | n/a |
| GCP Billing + Anomaly | $0 | $0 | $0 | $0 (preview) | $0 (vendor); +integration ops | n/a |
| Azure Cost Mgmt | $0 | $0 | $0 | $0 | $0 (vendor); +integration ops | n/a |
| Apptio Cloudability | per-spend tier | $0 | $0 | included | ~$8,000 (0.4 % of $2M spend) | $96,000 |
| Vantage | per-spend tier | $0 | $0 | included | ~$6,000 (0.3 %) | $72,000 |
| CloudZero | per-spend tier | $0 | $0 | included | ~$5,500 (0.28 %) | $66,000 |

Direct vendor (AWS/GCP/Azure native) is "free" but doesn't give chargeback / cost-center / OECD reporting — you need a third-party
(Apptio, Vantage, CloudZero) on top, which lands at $5,500-$8,000/mo. `cloud-billing` (Paid) at $2,800 is **49-65 % below
3rd-party FinOps platforms** AND replaces ERP integration ops cost.

## Where vendors still win

1. **Vendor-native vendor data fidelity.** AWS CUR sees AWS down to the per-second; `cloud-billing` re-aggregates from vendor CUR (lag).
2. **Marketplace catalog breadth.** Vantage + Cloudability + CloudZero are wired into 20+ vendor billing APIs;
   `cloud-billing` ingests via 4 ingestors (AWS, GCP, Azure, Stripe) at v1.
3. **Public sign-up.** AWS Billing Console + Cloudability + Vantage all have self-service tiers; `cloud-billing` requires tenant provisioning.
4. **Vendor-specific savings recommendations.** AWS Compute Optimizer + GCP Recommender produce vendor-tuned recommendations;
   `cloud-billing` defers to `finops-portal` for recommendations.

## Where `cloud-billing` wins

1. **Second-fresh metering** — 3 s ingest-to-query vs vendor's 5 min - 24 h.
2. **End-of-period close ≤ 74 min p95** — vendors take 4-24 h.
3. **FOCUS 1.1 native + validated** — vendors are preview/external.
4. **OECD BEPS Pillar Two export** — Apptio is the only other surface that ships this.
5. **Multi-axis chargeback** — cost-center × project × pack × region; vendors are tag-based only.
6. **Cedar-gated credit memos** — vendor systems allow direct ledger writes.
7. **Audit-chain BLAKE3** — tamper-evident; vendors append-only.
8. **HTTP/3 QUIC RPC** — ADR-0253.
9. **Per-tenant compliance pack overlays** — SOX-404, K-FSI, MAS-TRM flip per tenant.

## Reproducibility

```bash
make benchmarks.cloud-billing.run \
  VENDORS="cloud-billing,aws-cur,gcp-billing,azure-cost-mgmt,cloudability,vantage,cloudzero" \
  WORKLOADS="ingestion,period-close,focus-export,multi-currency" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-billing/2026-05-12T18:04:33Z/`.
