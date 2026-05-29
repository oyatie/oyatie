# api-gateway — Metric naming convention

**Authority:** ADR-0263 (observability emission) + `docs/standards/observability-slo.md`.

## A — Prefix

All metrics start with `oya_api_gateway_`.

## B — Suffix

- `_total` for monotonic counters.
- `_count` for counter samples.
- `_seconds` for durations (in seconds).
- `_seconds_bucket` for histogram buckets.
- `_ratio` for derived ratios (avoid in raw emission — compute at query time).

## C — Labels

Cardinality budgets per ADR-0263:

| Label | Cap | Notes |
|---|---:|---|
| `tenant_id` | 200k (downsample to tenant_class 2 for histograms) | Per ADR-0244 |
| `route_id` | 5k | Pre-allocated, not user-generated |
| `cell` | 24 | Per ADR-0248 |
| `code` | 60 (aggregate to `code_class` 6 for histograms) | HTTP status codes |
| `cipher_suite` | 5 | TLS_AES_128/256, ChaCha20 |
| `kem_group` | 5 | X25519MLKEM768, X25519, P-256, FrodoKEM |
| `tls_version` | 3 | TLS_1_3 only in steady state |
| `ech_status` | 3 | applied/not-applied/not-negotiated |
| `bot_score_class` | 5 | low/medium/high/very-high/unknown |
| `asn_class` | 8 | residential/datacenter/mobile/etc. |

## D — Canonical metrics

| Metric | Type | Description |
|---|---|---|
| `oya_api_gateway_requests_total{cell,tenant_id,route_id,code}` | counter | Request count by status code |
| `oya_api_gateway_latency_seconds_bucket{cell,route_id,le}` | histogram | Gateway-side latency |
| `oya_api_gateway_tls_handshake_total{cell,status,cipher_suite,tls_version,kem_group,ech_status}` | counter | TLS handshakes |
| `oya_api_gateway_tls_handshake_duration_seconds_bucket{le}` | histogram | TLS handshake duration |
| `oya_api_gateway_bot_score_bucket{cell,le}` | histogram | Bot-score distribution |
| `oya_api_gateway_alpn_negotiated_total{cell,alpn}` | counter | ALPN selection |
| `oya_api_gateway_rate_limit_per_tenant_hit_total{tenant_id}` | counter | |
| `oya_api_gateway_rate_limit_per_ip_hit_total` | counter | |
| `oya_api_gateway_rate_limit_per_fingerprint_hit_total` | counter | |
| `oya_api_gateway_upstream_circuit_open_total{upstream,cell}` | counter | |
| `oya_api_gateway_upstream_circuit_state{upstream,cell}` | gauge | 0=closed, 1=open, 2=half-open |
| `oya_api_gateway_cedar_decision_total{decision,fragment_id}` | counter | |
| `oya_api_gateway_cedar_eval_duration_us_bucket{le}` | histogram | µs resolution |
| `oya_api_gateway_captcha_challenge_issued_total{cell}` | counter | |
| `oya_api_gateway_captcha_passed_total` | counter | |
| `oya_api_gateway_honeypot_hits_total` | counter | |
| `oya_api_gateway_cert_expiry_days_remaining{cell}` | gauge | |
| `oya_api_gateway_ocsp_staple_total{status}` | counter | |
| `oya_api_gateway_ddos_scrub_dropped_bytes{cell}` | counter | |

## E — Trace span shape

Parent span: `gateway.request`. Attributes: `cell`, `tenant_id`, `route_id`, `bot_score`, `ech_status`, `pqc_negotiated`.

Children:
- `gateway.tls.handshake`
- `gateway.fingerprint.compute`
- `gateway.bot-score.eval`
- `gateway.rate-limit.lookup`
- `gateway.cedar.eval`
- `gateway.canonicalise`
- `gateway.auth.handoff`
- `gateway.upstream.<svc>`
- `gateway.response.mediate`
- `gateway.audit.emit`

## F — References

- ADR-0263
- `microservices/observability/contracts/metric-naming-convention.md`
- `microservices/api-gateway/compliance.md §E observability`
