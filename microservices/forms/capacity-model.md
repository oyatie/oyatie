---
doc_class: CapacityModel
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + ops-sre-reliability + cloud-iac
review_cadence: quarterly + on every cell-cluster provisioning
doc_status: published
---

# Forms — Capacity Model

## Workload Classes

| Class | Description | Driver | GA size |
|---|---|---|---|
| F1 — interactive form-render | Submitter loads a form | CDN + form-rest | 50k RPS peak per pack |
| F2 — submission | Submitter submits a response | response-collector-rest + Postgres + audit-chain | 5k RPS sustained, 20k peak per pack |
| F3 — field-validate | Submitter live-validates a field | form-rest server-validate | 50k RPS peak per pack |
| F4 — bulk-distribute | Tenant sends form to ≤ 10k recipients | bulk-distribute-worker + mail/messenger SDK | 100k recipients / minute per pack |
| F5 — export | Tenant exports responses | export-worker + Citus + object storage | 10k concurrent exports per pack |
| F6 — analytics | Dashboard render | Mimir aggregate + cache | 1k concurrent dashboards per pack |
| F7 — AI-form-build (T2) | Tenant invokes AI | foundry-providers SDK + LLM | 100 RPS per pack |
| F8 — webhook delivery | On-submit fan-out | webhook worker | 10k/sec per pack |
| F9 — file upload + scan | Submitter uploads file | drive bridge + ClamAV | 1k uploads/sec per pack; 100MB per file cap |

## Per-Component Sizing (pack-kr baseline GA)

| Component | Instances | Resources | Notes |
|---|---|---|---|
| form-rest (Deployment) | 4-80 (HPA target CPU 70%) | 500m / 1Gi req; 2 / 4Gi limit | TLS-terminated at Istio |
| response-collector-rest (Deployment) | 4-80 (HPA) | 1 / 2Gi req; 4 / 8Gi limit | sticky-shard via tenant_id |
| form-builder-wasm | CDN-only | – | served from pack-resident OCI CDN edge |
| Postgres + Citus (primary + 3 workers) | 1 + 3 | 4 / 16Gi req; 8 / 32Gi limit per node | tenant_id shard key; 32 shards |
| Redis 7.2 (Sentinel HA) | 3 (1 primary + 2 replica) | 2 / 8Gi req; 4 / 16Gi limit | rate-limit + session |
| Meilisearch | 3 | 2 / 8Gi req; 4 / 16Gi limit | per-pack index |
| ClamAV sidecar | 4-20 (HPA) | 1 / 2Gi | streaming scan; 100MB/file |
| Captcha sidecar | 2-10 (HPA) | 500m / 512Mi | hCaptcha proxy + Turnstile + Friendly Captcha verifier |
| Form-CDN edge | n/a (OCI CDN-managed) | – | pack-resident PoPs |
| Form-WAF | n/a (OCI WAF-managed) | – | rate-limit + bot rules |
| Export-worker | 2-20 (HPA) | 1 / 2Gi req; 4 / 8Gi limit | streaming CSV/XLSX |
| Bulk-distribute-worker | 2-20 (HPA on queue depth) | 1 / 2Gi req; 4 / 8Gi limit | Kafka-backed |
| Webhook-worker | 2-20 (HPA on queue depth) | 500m / 1Gi req; 2 / 4Gi limit | retries with exp backoff |
| AI-form-build-worker | 2-10 (HPA) | 500m / 1Gi req; 2 / 4Gi limit | calls foundry-providers |

## Growth Forecast (next 4 quarters)

| Quarter | Form-render RPS | Submissions/day | Bulk-distribute recipients/day | Storage (responses) |
|---|---|---|---|---|
| Q1 (GA) | 5k peak | 500k | 100k | 100GB |
| Q2 | 15k peak | 1.5M | 500k | 500GB |
| Q3 | 30k peak | 3M | 1M | 1.5TB |
| Q4 | 50k peak | 5M | 2M | 3TB |

Pack-eu / pack-us reach parity 1 quarter after pack-kr GA; pack-us-healthcare scales independently (HIPAA tenant onboarding gated by BAA).

## Cell Migration Triggers

Per ADR-0164 cell-pinning policy; trigger migration when:
- Single-tenant > 1M responses (mega-tenant) → dedicated cell.
- Pack-wide Citus shard skew > 3:1 → re-shard.
- Pack-wide Postgres connection saturation > 70% sustained → cell add.

## Scale-out Plan per Workload

| Workload | Bottleneck | Scale-out |
|---|---|---|
| F1 form-render | CDN cache hit rate | tune CDN; add edges |
| F2 submission | Postgres write | add Citus workers; per-tenant cell |
| F4 bulk-distribute | mail/messenger SDK throughput | back-pressure into queue; cap per-tenant rate |
| F5 export | object storage write | per-tenant export quota; queue priority |
| F7 AI-form-build | foundry-providers throughput | per-tenant token-bucket; pack-resident LLM |

## Cost Envelope (pack-kr GA, monthly)

| Component | Estimated USD |
|---|---|
| Postgres + Citus | $4,500 |
| Redis | $800 |
| Meilisearch | $1,200 |
| Compute (rest + workers) | $3,800 |
| ClamAV + Captcha sidecars | $400 |
| OCI CDN | $600 |
| OCI WAF | $300 |
| Object storage (exports) | $200 |
| Total | ~$11,800 / month / pack |

Scales sub-linearly with response volume due to Citus + streaming-export design.

## Verification

- `cargo run -p oya-dev-cli -- gate validate forms-capacity-model-conformance` (asserts manifests match this doc).
- Quarterly load test via k6: F1+F2+F4+F5+F7 synthetic burst to forecast Q+1.
- Quarterly chaos drill: induce Postgres primary failover, verify recovery ≤ 2min.

## References

- ADR-0131 per-microservice flat layout.
- ADR-0164 cell-pinning policy.
- `multi-region.md`.
- `cost-budget.md`.
- Citus docs.
- Google SRE Workbook — Capacity Planning.
