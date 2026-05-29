---
doc_class: Runbook
title: Catalog search latency regression (p95 > 200ms)
microservice: plugin-app-store
severity: "Sev-3 (single budget) / Sev-2 (multi-budget OR tenant-impacting)"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/plugin-app-store/PRD.md §catalog-browse-latency
  - microservices/plugin-app-store/slos/catalog-browse-latency.openslo.yaml
  - microservices/plugin-app-store/dashboards/catalog-perf.json
doc_status: published
---

# Runbook: Catalog search latency regression (p95 > 200ms)

## Trigger

- Catalog search p95 > 200ms for ≥ 15 min
- Catalog search 5xx rate > 0.1% for ≥ 5 min
- Cilium L4 cache hit rate < 80%

## Severity

Sev-3 (single budget) / Sev-2 (multi-budget OR tenant-impacting)

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — Postgres tsvector index not used

1. EXPLAIN ANALYZE on a sample query.
2. If seq scan: rebuild GIN index `REINDEX INDEX CONCURRENTLY plugins_search_tsv_idx`.
3. Verify p95 returns within budget.

### Recovery Path B — Cilium L4 cache cold after deploy

1. Warm cache via synthetic-traffic generator: `cargo run -p oya-dev-cli -- warm --target plugin-app-store-catalog --requests 10000`.
2. Verify cache hit rate ≥ 95% before traffic resumes.

### Recovery Path C — Postgres replica lag

1. Check `pg_stat_replication` lag.
2. If > 5s: pause traffic to replica via Cilium `route off replica-N`.
3. Wait for catch-up; resume.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/plugin-app-store/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
