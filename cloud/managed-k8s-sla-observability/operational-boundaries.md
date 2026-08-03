# Managed K8s SLA Observability — Operational Boundaries

## Capacity Model

- Deterministic summary math runs in the pure kernel over normalized snapshots;
  the kernel has no clock, network, Kubernetes, Prometheus, or HTTP dependency.
- The in-memory adapter is suitable for local/dev verification of the port and
  summary contract only. It is not measured production SLO evidence.
- Live collectors must normalize observations into `ControlPlaneSlaSnapshot`
  before calling `SlaObservabilityPort::ingest_status_snapshot`; concrete
  Kubernetes/Prometheus adapters remain follow-on work behind that port.

## Incident Response

- Missing or stale samples are unavailable/no-data for SLA evidence unless a
  reviewed follow-on contract says otherwise; they never count as healthy.
- If control-plane status and live scrape evidence disagree, the lower-claim,
  higher-risk state wins for alerting and rollout/rollback hold decisions.
- Unknown tenant or cluster reads return typed missing-cluster/read-denial
  outcomes rather than guessed summaries.
- Broad observability trace/OTLP outages prevent live-evidence claims but do not
  invalidate deterministic local summary tests.

## Multi-region and Cells

- Observation windows, collection time, freshness deadline, collector identity,
  region, and cell must be carried with live evidence records.
- Tenant-zero/dogfood is an ordinary tenant value; there is no internal bypass.
- Regional or cell-level rollups must be derived from tenant-scoped cluster
  summaries and must not expose another tenant's evidence.
