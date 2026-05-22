# Dashboard — Edge Overview

**Owner:** axis-network.
**Source:** `dashboards/edge-overview.json`.
**Audience:** SRE on-call + axis leads.

## Purpose

Top-level edge health snapshot. First dashboard opened during an incident.

## Panels

| # | Title | What it shows | What "good" looks like |
|---:|---|---|---|
| 1 | Requests/sec (global) | Global throughput | 4-7M req/s steady-state |
| 2 | 5xx ratio | Edge availability inverse | < 0.001 |
| 3 | Latency p50/p95/p99 | Tail-latency budget | p50 ≤50ms, p95 ≤200ms, p99 ≤500ms |
| 4 | Per-cell requests/sec | Cell load balance | All cells within ±20% of mean |
| 5 | HTTP/3 negotiation ratio | h3 adoption | ≥0.8 |
| 6 | TLS handshake success ratio | TLS health | ≥0.9995 |
| 7 | Bot-score distribution | Bot-traffic share | ≥70% at score ≤80 |
| 8 | Circuit breaker open count | Upstream health | 0 sustained |
| 9 | Rate-limit 429 ratio | Rate-limit pressure | < 0.005 of total |
| 10 | Cedar permit/deny ratio | Policy gate health | permit/deny stable |

## Common views

- **Default view:** all cells, all tenants.
- **Per-cell view:** select `$cell` variable.
- **Per-tenant view:** select `$tenant` variable (cardinality cap 200k).

## Related dashboards

- `dashboards/rate-limit-hits.json`
- `dashboards/tls-health.json`
- `dashboards/bot-score-distribution.json`

## SLO bindings

Tied to: `slos/edge-availability.openslo.yaml`, `slos/edge-latency-p50/p95/p99.openslo.yaml`, `slos/tls-handshake-success.openslo.yaml`, `slos/h3-negotiation-rate.openslo.yaml`.

## References

- `microservices/api-gateway/incident-response.md`
- ADR-0263 (observability emission)
