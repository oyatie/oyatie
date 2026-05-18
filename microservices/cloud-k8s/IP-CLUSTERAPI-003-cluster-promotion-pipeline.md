# IP-CLUSTERAPI-003 — Cluster promotion pipeline

> ADR anchor: ADR-0202, ADR-0139, ADR-0171.
> Owner: `oya-cloud-k8s`.
> Estimate: 4 days.

## Goal

Promote a cluster from dev → staging → prod via the agentic
SLO-gated promotion pipeline (ADR-0139).

## Tasks

### 1. Promotion stages

- dev: any cluster.
- staging: must pass 24h soak with SLO budgets ≥ 99%.
- prod: must pass 7d soak with SLO budgets ≥ 99.9%.

### 2. SLO gate

- ADR-0139 SLO-gated promotion reads the cluster's SLO
  rollup and blocks promotion on budget exhaustion.

### 3. Audit emission

- Every promotion emits an ADR-0145 entry with the SLO
  evidence.

## Acceptance criteria

- Promotion can never bypass the SLO gate.

## References

- ADR-0202, ADR-0139, ADR-0171.
