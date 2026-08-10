---
doc_class: Runbook
title: Per-plugin rate-limit bypass suspected
microservice: plugin-app-store
severity: "Sev-1 (security)"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - marketplace/observability/slos/per-plugin-rate-limit-correctness.openslo.yaml
doc_status: published
---

# Runbook: Per-plugin rate-limit bypass suspected

## Trigger

- `oya_plugin_app_store_rate_limit_bypass_total` rate > 0
- Plugin request rate exceeds configured limit

## Severity

Sev-1 (security)

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — Valkey Lua atomic decrement race

1. Audit the Lua script for atomicity.
2. Verify Valkey single-shard config (no cluster splits).
3. If race confirmed: page council-security.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/plugin-app-store/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
