# api-gateway — Capacity model

**Authority:** ADR-0157 + ADR-0248 (cellular) + `docs/standards/documentation-rigor.md` §1.1 (capacity math).
**Hyperscaler precedent:** AWS API Gateway capacity guide (2024) + Cloudflare 100Tbps record + Apigee enterprise scale brief.

## A — Per-cell capacity

Per Tier-0 edge cell:

| Resource | Capacity | Saturation point |
|---|---|---|
| TLS handshakes/sec | 50,000 | CPU-bound at 80% utilisation on AMD EPYC 9554P × 64 vCPU |
| Sustained connections | 5,000,000 | RAM-bound at 192GiB with 38KiB per HTTP/3 connection (≈ 4.8M; round to 5M) |
| HTTP requests/sec | 250,000 | wire-bound at 100Gbps with 4KiB average response (≈ 312k; round to 250k for headroom) |
| QUIC packets/sec | 4,000,000 | userspace QUIC stack saturates at 4M pps (Envoy QUIC benchmark) |
| Cedar evals/sec | 500,000 | sub-1ms p99 evaluation in caller-side library (oya-shared-policy-eval benchmark) |
| Bot-score evals/sec | 300,000 | Wasm filter benchmark; CPU-bound |
| Rate-limit lookups/sec | 1,000,000 | per-cell Valkey Cluster benchmark |

## B — Little's Law derivation

For target steady-state:

- λ (arrival rate) = 250,000 req/s per cell
- W (mean residence time at gateway) = 5ms (p50)
- L (mean number in system) = λ × W = 1,250

So at p50, the gateway holds 1,250 in-flight requests per cell. At p99 (50ms residence — circuit-breaker-tripped upstream), L = 12,500. RAM budget for in-flight queue: 12,500 × 4KiB request envelope = 50MiB. Well under the 192GiB pool.

## C — Multi-region capacity

| Region | Cells | Per-cell req/s | Region total |
|---|---:|---:|---:|
| us-east | 4 | 250k | 1.0M |
| us-west | 3 | 250k | 750k |
| eu-frankfurt | 3 | 250k | 750k |
| eu-ireland | 2 | 250k | 500k |
| ap-tokyo | 2 | 250k | 500k |
| ap-seoul (sov-cell-kr) | 3 | 250k | 750k |
| ap-singapore | 2 | 250k | 500k |
| cn-shanghai (sov-cell-cn) | 2 | 250k | 500k |
| sa-saopaulo | 1 | 250k | 250k |
| me-dubai (sov-cell-ae) | 1 | 250k | 250k |
| me-riyadh (sov-cell-ksa) | 1 | 250k | 250k |

**Global aggregate:** ~6M req/s at steady state. At burst (4× headroom per ADR-0248 shuffle-shard model), 24M req/s. Beyond that, cell auto-scale activates.

## D — Cost model

Per cell per month:

| Cost | Amount | Driver |
|---|---:|---|
| Compute (64 vCPU × 4 nodes × $0.04/vCPU-hr) | $7,372/mo | Cell baseline |
| RAM (192GiB × 4 nodes × $0.005/GiB-hr) | $2,765/mo | Cell baseline |
| Network egress (100Gbps × 30% util × $0.02/GB) | $129,600/mo | Variable with traffic |
| TLS cert (Let's Encrypt + cert-manager) | $0/mo | Free |
| ECH key management | $0/mo | In-house |
| Valkey cluster (per-cell rate-limit) | $1,500/mo | Managed |
| **Total per cell** | **$141k/mo** | At 30% util |

At full global footprint (24 cells), $3.4M/mo. Per-request cost at 6M req/s steady = $0.022/M-requests.

## E — Headroom + auto-scale

- Per-cell utilisation target: 30% steady-state, 70% burst.
- Auto-scale at >50% utilisation for >5min OR >70% for >30s.
- Scale-up gate: cell-tier-0 budget approval per `cost-budget.md`.
- Scale-down gate: ≤20% utilisation for >30min, single-cell delta.

## F — Bottlenecks

| Bottleneck | Mitigation |
|---|---|
| Cedar eval CPU | Caller-side library + recording-rule pre-eval for common patterns; budget 1ms p99 |
| Valkey hot key (single tenant DDoS) | Per-tenant shuffle-shard across 4+ Valkey nodes; key salt prevents single-node saturation |
| QUIC userspace packet processing | Pin Envoy QUIC threads to NIC IRQ-affinity cores; SO_REUSEPORT |
| TLS handshake CPU | Session resumption (TLS 1.3 PSK); ECH key cache; hardware-accelerated ECDSA where available |
| Bot-management Wasm filter latency | Cache scores per fingerprint for 30s; only re-score on score-tier change |

## G — References

- ADR-0157, ADR-0248
- `docs/standards/documentation-rigor.md` §1.1
- Cloudflare 100Tbps DDoS report 2024 H2
- Envoy QUIC benchmark 2024
- AWS API Gateway capacity guide 2024
