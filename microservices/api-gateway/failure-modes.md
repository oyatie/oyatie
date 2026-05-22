# api-gateway — Failure modes

**Authority:** `docs/standards/documentation-rigor.md` §1.1 (failure-mode tree mandate).
**Last reviewed:** 2026-05-20.

The legacy table below is preserved for continuity. The full failure-mode tree (extended Wave-3-A) sits in §A..§J.

## Legacy table (pre-Wave-3-A)

| Failure | Designed response | Evidence emitted |
|---|---|---|
| JWKS refresh unavailable | Use unexpired cached keys, then fail closed when stale | `oya.api_gateway.request.denied` |
| WAF rule pack regression | Roll back to previous signed bundle | `oya.api_gateway.waf.triggered` |
| Rate-limit backend unavailable | Fall back to per-replica limiter and mark degraded | `oya.api_gateway.request.admitted` with degraded flag |
| Cross-cell route requested | Deny before workload dispatch | `oya.api_gateway.request.denied` |

## A — Network-layer failures

| Failure | Behaviour | Detection | Mitigation |
|---|---|---|---|
| BGP withdrawal of Anycast prefix | Traffic drops; clients reroute to next-nearest cell via BGP. | NMS alert + `oya_api_gateway_bgp_withdraw_total` | RPKI signed ROAs; multi-prefix; `runbooks/cell-evac.md` |
| QUIC blocked by upstream network | Clients fall back to h2/TCP-443 automatically per RFC 9000 | SLO `h3-negotiation-rate` < 0.8 | Documented; verified by `runbooks/h3-fallback-verification.md` |
| MTU < 1280 (QUIC min) | QUIC handshake fails; client falls back to TCP | TLS handshake metric | Path-MTU discovery; ICMP-PTB; documented in runbook |
| DDoS L3/L4 flood | Traffic scrubbed at BGP layer; gateway sees post-scrub traffic only | `oya_api_gateway_ddos_scrub_dropped_bytes` | `runbooks/ddos-mitigation.md` |
| DDoS L7 (HTTP-flood) | Rate-limit + bot-score + adaptive challenge engage | SLO burn-rate fast-burn-1h | `runbooks/rate-limit-saturation.md` + `runbooks/bot-storm.md` |
| Reflection/amplification attack | Upstream blackhole; not visible at gateway | NMS | BCP38/84 at provider |
| Per-cell network partition | Cell traffic re-routed within 60s via NS1 health-check | `oya_api_gateway_partition_detected_total` | `runbooks/cell-evac.md` |

## B — TLS layer failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Cert expiry | Pre-expiry alert at -30/-14/-7 days; auto-renew via cert-manager | `runbooks/tls-cert-rotation.md` |
| CT log unavailable | Cert issuance delayed; existing certs serve | `runbooks/tls-cert-rotation.md` |
| ECH config compromise | Rotate ECH key chain; advertise new HTTPS RR | `runbooks/ech-key-rotation.md` |
| PQC algorithm break (MLKEM-768) | Crypto-agility: rotate cert chain; FrodoKEM fallback | `runbooks/pqc-cert-rotation.md` |
| HSTS bypass attempt (downgrade to HTTP) | 301 to https://; emit audit | Default behaviour |
| OCSP responder down | Use stapled OCSP from cert-manager refresh; serve stale ≤24h | Default behaviour |
| CA compromise | Pin alternate CA per ADR-0295 supply-chain doctrine | `runbooks/tls-cert-rotation.md` §emergency |

## C — Bot-management failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Bot-score false positive | CAPTCHA-on-suspicion (not lockout); friendly-crawler always-pass list | Human-rights review; SLO `bot-score-false-positive-rate` |
| Bot-score Wasm filter crash | Filter restart ≤200ms; fail-open by design | Audit `oya.api_gateway.botscore.filter.crashed` |
| hCaptcha unavailable | Failover to Turnstile; if both down, fail-open with elevated audit | `runbooks/bot-storm.md` |
| Model poison | Anomaly detection on model output; rollback model version | Model card + `runbooks/bot-storm.md` |

## D — Cedar policy failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Policy fragment fails soak | Fragment rejected at publisher; existing fragment continues | ADR-0294 |
| Cedar eval timeout (>1ms p99) | Eval cancelled; default-deny applied; audit | Default behaviour |
| Cedar loader can't reach policy-engine | Cached fragments (≤30s stale); alert | `runbooks/cedar-fragment-emergency-rollback.md` |
| Malformed fragment in push | Reject at soak gate | ADR-0294 |
| Cedar permit storm | Default-deny baseline + forbid-overrides-permit | `policy/tenant-scope.cedar` forbid baseline |

## E — Auth handoff failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Identity µservice down | Cached principal contexts (≤60s); new sessions 503 | Circuit breaker |
| Identity µservice slow | Per-route deadline; 503 if exceeded | Default behaviour |
| Forged `X-Oya-Principal-Context` | Signature verify fails; 502 + audit | Default behaviour |
| Session-id collision | HMAC + secret rotation; per-tenant salting | Default behaviour |

## F — Upstream µservice failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| 5xx burst | Circuit breaker trips per cell after 5 consecutive 5xx | `runbooks/circuit-breaker-engaged.md` |
| Slow upstream | Per-route deadline; 504; audit | Default |
| Unreachable upstream | DNS / SPIFFE resolution fails; 503 | Default |
| mTLS handshake fail | 502; audit; alert | `runbooks/circuit-breaker-engaged.md` |
| Malformed response | 502; audit; alert | Default |

## G — Rate-limit failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Valkey cluster node down | Cell-local shuffle-shard re-routes | `runbooks/rate-limit-saturation.md` |
| Per-tenant bucket exhaust DDoS | 429 with Retry-After; bucket auto-refills | Default |
| Burst beyond cap | 429; audit | Default |
| Cross-cell counter drift | Cell-local rate-limit; eventual consistency via Kafka tick | Documented |

## H — Audit-chain failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Audit-chain down | Local buffer (≤10min); flush on recovery; alert | Cross-ref audit-chain µservice runbook |
| Local buffer overflow | Backpressure; never drop without audit | Default |
| Audit signing key compromise | Rotate per-µservice key via sidecar; quarantine | `runbooks/audit-key-rotation.md` |

## I — Cell-level failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Single-cell power loss | NS1 de-pools ≤60s | `runbooks/cell-evac.md` |
| Region power loss | Inter-region DR per `multi-region.md` | `runbooks/cell-evac.md` |
| K8s API outage | Existing pods continue; new pods cannot schedule | `runbooks/cell-evac.md` |
| SPIRE down | Cached SVIDs valid 1h | ADR-0295 |

## J — Sovereign-cell failures

| Failure | Behaviour | Mitigation |
|---|---|---|
| Sov-cell-kr unreachable | Traffic to sov-cell-kr blocked; non-resident re-routed | `compliance.md §pack-kr` |
| Sov-cell-cn unreachable | Traffic blocked; non-PRC re-routed | `compliance.md §pack-cn-pipl-2021` |
| FedRAMP High cell SLA breach | Per-contract escalation | Per-contract |

## K — References

- `microservices/api-gateway/ARCHITECTURE.md`
- `microservices/api-gateway/threat-model.md`
- `microservices/api-gateway/runbooks/`
- `docs/standards/documentation-rigor.md` §1.1
