---
doc_class: Runbook
title: Sandbox provision slow
microservice: developer-sdk
severity: "Sev-3"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/developer-sdk/PRD.md §sandbox-provisioner
doc_status: published
---

# Runbook: Sandbox provision slow

## Trigger

- Sandbox provision p99 > 60s
- Synthetic-seed worker queue backlog > 100

## Severity

Sev-3

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — tenancy µservice slow

1. Check tenancy µservice latency.
2. Page tenancy owner if degraded.
3. Throttle sandbox provision rate until restored.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/developer-sdk/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
