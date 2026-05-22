# api-gateway — Cost + FinOps model

**Authority:** ADR-0157 + ADR-0248 + ADR-0250.
**Last reviewed:** 2026-05-20.

## A — Per-request cost model

| Component | Cost (CPU-µs) | RAM (KiB) | Net in (B) | Net out (B) |
|---|---:|---:|---:|---:|
| TLS 1.3 handshake (cached PSK) | 50 | 32 | 0 | 256 |
| TLS 1.3 handshake (full + ECH + PQC) | 1,200 | 64 | 1,500 | 5,000 |
| QUIC ACK / flow control | 20 | 8 | 80 | 80 |
| JA4 fingerprint compute | 10 | 1 | 0 | 0 |
| Bot-score eval (Wasm filter) | 80 | 16 | 0 | 0 |
| Rate-limit lookup (per-cell Valkey) | 30 | 0.5 | 64 | 64 |
| Cedar eval (caller-side library) | 200 | 8 | 0 | 0 |
| URL canonicalisation | 15 | 4 | 0 | 0 |
| Auth handoff (cache hit) | 20 | 2 | 0 | 0 |
| Auth handoff (cache miss → identity) | 500 | 8 | 256 | 1,500 |
| Route map lookup | 5 | 0.5 | 0 | 0 |
| Upstream mTLS (reused) | 5 | 0.5 | 0 | 0 |
| Upstream call + response | 50 | 8 | 256 | 4,000 |
| Response mediation | 10 | 1 | 0 | 1,200 |
| Audit emission (async) | 5 | 2 | 0 | 0 |
| **Total (warm)** | **500** | **80** | **656** | **6,600** |
| **Total (cold)** | **2,250** | **152** | **2,156** | **12,100** |

## B — Per-million-requests cost

| Scenario | Cost / M-req |
|---|---:|
| Warm | $0.018 |
| Cold | $0.042 |
| DDoS scrub | $0.005 ingress + $0.0003 egress |

At 6M req/s globally ≈ 518B req/month ≈ $9.3k compute alone. Egress dominates at $130k/cell/mo.

## C — Per-tenant attribution

Per `tenant_id` per ADR-0244. Cost = (req/s × cost/req) + (egress GB × $0.02/GB). Reported via `finops-portal` µservice.

## D — Cost ceilings

| Tenant tier | Soft cap ($/mo) | Hard cap ($/mo) |
|---|---:|---:|
| Free | $10 | $50 |
| Starter | $100 | $500 |
| Business | $5,000 | $25,000 |
| Enterprise | $50,000 | $250,000 |
| Sov / IL5 | Per contract | Per contract |

Hard cap → Cedar refuses via `policy/rate-limit.cedar` `cost-ceiling-exceeded`; emit audit + customer notification.

## E — Per-region COGS

| Region | $/M-req | $/GB egress |
|---|---:|---:|
| us-east | $0.018 | $0.02 |
| us-west | $0.018 | $0.02 |
| eu-* | $0.022 | $0.025 |
| ap-seoul (sov) | $0.030 | $0.04 |
| cn-shanghai | $0.035 | $0.06 |
| sa-saopaulo | $0.038 | $0.08 |

## F — Optimisation roadmap

- PQC hardware acceleration → +0.05ms p50 (vs +0.2ms today).
- ECH DoH-cached → 0 added latency.
- Bot-score 30s cache → 50% Wasm CPU saving.
- Cedar 1s decision cache → 30% saving.
- Brotli + Zstd compression → 25% egress saving.

## G — Pre-Wave-3-A baseline

The legacy single-paragraph FinOps boundary stated:

> Cost drivers are Envoy replicas, WAF rule evaluation, rate-limit storage, and telemetry emission volume. The design budget treats edge admission as shared substrate and attributes incremental cost by tenant request count, route family, and cell.

Extended above with intern-buildable cost breakdowns.

## H — References

- ADR-0157, ADR-0248, ADR-0250
- `docs/standards/documentation-rigor.md` §1.1 + §1.2 Optimization
- `microservices/finops-portal/`
