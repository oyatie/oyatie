# Managed K8s Tenant Quota — Operational Boundaries

## Capacity Model

- `evaluate()` is O(1); sub-microsecond on the lifecycle hot path.
- In-memory store: suitable for single-node bring-up only.
- Production: Postgres-backed adapter (follow-on wave).

## Incident Response

- On store failure: return `QuotaPortError::Persistence`; cluster-lifecycle
  treats this as deny (fail-closed).
- On Cedar boot failure: service exits non-zero; orchestrator restarts.
- On quota not found: HTTP 404; caller must set quota first.

## Multi-region

- In-memory store is not replicated. Production Postgres adapter should use
  per-cell Postgres (ADR-0339 shared IaC module library).
