---
doc_class: Runbook
title: Vetting pipeline stuck (submission backlog)
microservice: plugin-app-store
severity: "Sev-2"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/plugin-app-store/PRD.md §vetting-pipeline
doc_status: published
---

# Runbook: Vetting pipeline stuck (submission backlog)

## Trigger

- Vetting throughput < 50% of submission rate for ≥ 30 min
- Backlog age p99 > 24h

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

### Recovery Path A — Trivy scanner pod CrashLooping

1. Check pod logs.
2. Verify Trivy DB freshness; if stale > 7d: force-refresh.
3. Restart pod.

### Recovery Path B — Wasmtime isolation validator OOM

1. Check Wasmtime pod memory.
2. Increase memory limit; restart.
3. File IP to make Wasmtime engine pool elastic.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/plugin-app-store/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
