---
doc_class: Runbook
title: Plugin install failure spike
microservice: plugin-app-store
severity: "Sev-2"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/plugin-app-store/PRD.md §plugin-install
doc_status: published
---

# Runbook: Plugin install failure spike

## Trigger

- Install 5xx rate > 0.05%
- Cedar policy materialization timeout > 1s

## Severity

Sev-2

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — Cedar evaluator unreachable

1. Check governance µservice Cedar evaluator health.
2. If down: page council-architecture (governance is upstream).
3. Hold install flow with retry-able 503 until restored.

### Recovery Path B — Postgres connection pool saturation

1. Check pgbouncer/sqlx pool metrics.
2. Scale plugin-app-store-plugin-install-app replicas.
3. If still saturated: increase pool size via Helm values.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/plugin-app-store/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
