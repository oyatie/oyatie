# Capacity model — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0174 (FinOps), ADR-0180 (DR/BC).

## 1. Workload shape

- Transactional, request/response over REST.
- Bursty: account-signup waves, marketing-trigger waves,
  regulatory disclosure batches.
- Steady-state QPS varies by tenant size.

## 2. Reference dimensions

- Sends per second per cluster: p50 = 100, p99 = 1,000,
  peak = 10,000.
- Concurrent template renders per cluster: ≤ 200.
- Webhook ingest QPS: ≈ 3-4× send QPS (each send → 3-4
  delivery events).
- Suppression-list lookup QPS: 1× send QPS.

## 3. Compute sizing (Phase 1)

| Workload | Replicas | CPU req | Mem req |
| -------- | -------- | ------- | ------- |
| API service | 4 | 500m | 512Mi |
| Render worker | 4 | 1000m | 1Gi |
| Webhook ingest | 4 | 500m | 512Mi |
| DKIM rotation job | 1 | 100m | 128Mi |

## 4. Storage sizing

- Suppression list Postgres: ~50M rows at maturity (10 tenants
  ×5M each). Index size ~10GB. PVC: 100GB headroom.
- Idempotency-key store: ~10M rows live (1h TTL). PVC: 10GB.
- Audit chain emission buffer: in-memory ≤ 5min; bursts to
  ≤ 1GB.

## 5. Horizontal scale model

- API + render + webhook ingest are horizontally scalable;
  HPA tied to CPU + custom `comms_email_queue_depth` metric.
- Postgres: vertically scaled until 50M rows; then horizontal
  sharding per tenant pack (Phase 2).

## 6. Latency budgets

| Stage | p99 budget |
| ----- | ---------- |
| Preflight | 10 ms |
| MJML compile (cache hit) | 1 ms |
| MJML compile (miss) | 50 ms |
| Liquid sub | 5 ms |
| Suppression lookup | 5 ms |
| DKIM sign | 5 ms |
| Provider call | 400 ms |
| Total p99 send | **500 ms** |

## 7. Throughput headroom

- 2× peak headroom: at 10k sends/s peak, sizing supports
  20k sends/s.
- Provider quotas (SES + Mailgun) are the binding constraint
  at sustained > 100k sends/s — multi-region routing + pack
  segmentation prevent any one provider hitting its quota.

## 8. SLO mapping

- `send-latency p99 ≤ 500ms` — capacity model derived above.
- `webhook-success rate ≥ 99.99%` — derived from DLQ ratio.
- `audit-chain-emit-lag p99 ≤ 5s` — derived from chain
  buffer + audit chain SLO.

## 9. Cost-vs-throughput tradeoffs

- Render worker is the most expensive per-send component.
  Aggressive caching (IP-006 §5) drops the dominant cost.

## 10. Forecast

- 12-month projection: 10× send volume on cloud-hosted
  clusters; sovereign tier remains steady.
- Postgres re-sharding milestone hits at ~24 months under
  forecast.
