---
doc_class: Runbook
title: KYC pipeline stuck (developer onboarding backlog)
microservice: developer-sdk
severity: "Sev-2"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/developer-sdk/PRD.md §developer-onboarding
doc_status: published
---

# Runbook: KYC pipeline stuck (developer onboarding backlog)

## Trigger

- KYC liveness check p99 > 60s for ≥ 30 min
- Sanctions screening API timeout rate > 5%

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

### Recovery Path A — OFAC SDN list refresh failure

1. Manually refresh OFAC list.
2. Verify SDN list freshness < 24h.
3. Resume pipeline.

### Recovery Path B — Liveness-check ML model overloaded

1. Scale liveness-check inference replicas.
2. Queue overflow re-tries with exponential backoff.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/developer-sdk/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
