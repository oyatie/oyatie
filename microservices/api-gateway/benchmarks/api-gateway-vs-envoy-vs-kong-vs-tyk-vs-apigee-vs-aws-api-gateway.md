---
doc_class: Benchmark
microservice: api-gateway
benchmark_date: 2026-05-20
related_adrs: [ADR-0253, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie api-gateway vs Envoy / Kong / Tyk / Apigee / AWS API Gateway / Cloudflare Workers

Workloads measured: (a) edge p99 latency at 50 k RPS, (b) TLS handshake latency (cold + warm), (c) WAF inspection p99, (d) rate-limit decrement p99, (e) HTTP/3 negotiation success rate, (f) annual TCO at 2 000 tenants × 50 k RPS each.

Hardware (oyatie paid tenant_class profile): 16× Envoy + 12× rate-limit + 4× control-plane × 3 regions.

Comparators measured against published latency figures (AWS docs, Apigee performance whitepaper, Kong benchmark suite) + our independent test rig.

## Workload (a) — edge p99 latency at 50 000 RPS sustained

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie api-gateway paid tenant_class | 3.8 | 22 |
| Envoy (raw, no WAF) | 3.2 | 18 |
| Kong Gateway Enterprise (3.x) | 6.4 | 38 |
| Tyk Gateway (5.x) | 8.2 | 52 |
| Apigee X (Google Cloud) | 12 | 78 |
| AWS API Gateway (REST) | 22 | 145 |
| AWS API Gateway (HTTP) | 14 | 92 |
| Cloudflare Workers (edge, no WAF) | 4.1 | 28 |

Reading: oyatie paid tenant_class profile beats every managed offering. Raw Envoy is faster (no WAF, no Cedar), but the gap is < 5 ms p99.

PRD target: edge p99 is tenant_class-uniform; ≤ 22 ms measured.

## Workload (b) — TLS handshake latency (cold and warm)

| Platform | Cold p99 (ms) | Warm (session-resumption) p99 (ms) |
|---|---:|---:|
| oyatie api-gateway paid tenant_class (HSM-resident key) | 28 | 4 |
| Envoy (in-memory key) | 18 | 3 |
| Kong Gateway | 42 | 9 |
| Tyk Gateway | 56 | 12 |
| Apigee X | 72 | 14 |
| AWS API Gateway | 95 | 22 |
| Cloudflare Workers | 35 | 5 |

Reading: HSM costs ~ 10 ms on cold handshake (HSM round-trip); warm is dominated by session-ticket lookup which is in-Valkey for oyatie. Apigee and AWS API Gateway both pay multi-tenant control-plane overhead.

## Workload (c) — WAF inspection p99 (per-request, OWASP CRS 4.3, 200B body)

| Platform | p99 (ms) |
|---|---:|
| oyatie api-gateway (ModSecurity + libinjection + Cedar overlay) | 7.2 |
| Envoy + Coraza WAF | 6.8 |
| Kong Gateway WAF plugin | 11 |
| AWS WAF on ALB | 22 |
| Cloudflare WAF | 8.5 |
| Imperva WAF (cloud) | 14 |

Reading: WAF inspection is bounded by libinjection cost + Cedar evaluator. Cedar overlay adds ~ 1 ms p99 but provides per-tenant policy semantics that signature WAFs lack.

## Workload (d) — rate-limit decrement p99 (token-bucket, Valkey-backed)

| Platform | p99 (ms) |
|---|---:|
| oyatie api-gateway (Valkey 7.2, token-bucket Lua-script) | 1.2 |
| Envoy local rate-limit | 0.4 |
| Envoy global rate-limit (Redis; counterpart-fact) | 2.1 |
| Kong Gateway (Redis-backed; counterpart-fact) | 2.8 |
| AWS API Gateway (built-in throttle) | 5 (per AWS docs; actual SLA "best-effort") |
| Cloudflare Rate Limiting (1.2.x) | 1.8 |

Reading: in-Envoy local rate-limit is fastest but can't span PoPs. Valkey-backed gives us cross-PoP consistency at ~ 1.2 ms.

## Workload (e) — HTTP/3 negotiation success rate (corporate-network clients)

| Platform | H3 success % | H2 fallback % |
|---|---:|---:|
| oyatie api-gateway | 94.5 | 5.5 |
| Cloudflare (mature H3) | 96 | 4 |
| AWS CloudFront → API Gateway | 87 | 13 |
| Kong Gateway (H3 GA in 3.5+) | 88 | 12 |
| Apigee X (H3 in preview) | 72 | 28 |

Reading: H3 negotiation depends on client + middlebox cooperation. Mature deployments (Cloudflare) get ~ 96 %; we're close. Apigee H3-preview lags.

## Workload (f) — annual TCO at 2 000 tenants × 50 k RPS each (avg 10 % utilisation)

| Platform | Annual cost (USD) | Notes |
|---|---:|---|
| oyatie api-gateway paid tenant_class | 720 000 / cell × N cells (3 cells covers 100 M aggregate RPS) = ~ 2 160 000 | All-in: hardware + ops + HSM |
| AWS API Gateway (REST, 1 B req/mo per tenant) | ~ 3.50 / 1M req × 12 × 2 000 = 84 000 / tenant = 168 000 000 | Per-request; scales linearly |
| AWS API Gateway (HTTP) | ~ 1.00 / 1M req × 12 × 2 000 = 24 000 / tenant = 48 000 000 | Cheaper than REST |
| Kong Konnect Enterprise (cloud-managed) | ~ 60 000 / tenant = 120 000 000 | Per-tenant; cluster-managed |
| Apigee X (Apigee Cloud) | ~ 100 000 / tenant = 200 000 000 | Per-tenant; full API mgmt |
| Tyk Cloud Enterprise | ~ 36 000 / tenant = 72 000 000 | Per-tenant; mid-tier |
| Cloudflare Workers (paid plan, with custom-domain SSL) | ~ 2 000 / tenant + per-req = 12 000 000 | Per-domain + per-request |

Reading: at scale, per-request pricing of AWS / Apigee is punitive. oyatie's cell-cost model is competitive above ~ 50 tenants.

Note: the 2 160 000 oyatie figure includes 3 cells; per-cell amortised at ~ 670 tenants. Real ops typically run 5-8 cells in a region for blast-radius isolation per ADR-0248.

## Reproducibility

Benchmark harness at `benchmarks/api-gatewaybench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks api-gateway \
    --workload edge-latency-50krps \
    --tenant-class paid \
    --duration 30m \
    --output ./benchmark-results.json
```
