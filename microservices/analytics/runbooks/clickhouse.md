# ClickHouse Runbook — Analytics

**Authority:** ADR-0193
**Owner:** council-analytics + ops-sre-reliability
**Last reviewed:** 2026-05-18

Mirrors `microservices/observability/runbooks/clickhouse.md` for the analytics-namespace cluster. Differences:

- **Per-tenant focus.** Every alert / triage step pivots on `tenant_id` since this cluster serves tenant-facing queries.
- **Higher availability target.** Tenant dashboard SLO is 99.95% per IP-014; observability's is 99.9%.
- **Quota-exceeded handling.** Tenant-tier quota-exceeded events are a tenancy-team escalation, not a capacity-team escalation.

## Cluster health quick-check

Same as observability runbook; substitute `analytics` namespace.

## Per-tenant quota exceeded

`ClickHouseQuotaExceeded{tenant_id=...}` alert:

1. Identify tenant + tier.
2. If burst is tier-appropriate, document + back off.
3. If misbehaving, contact account team.
4. Quota upgrade path: tenancy µservice emits `tenant.tier_changed` → IP-002 controller re-applies QUOTA.

## Dashboard query latency burn

Same as observability runbook, but the consumer is tenant-facing — escalation priority is higher.

## Escalation

- Page → PagerDuty `analytics-oncall` + Opsgenie `analytics-oncall`.
- Slack: `#analytics-incidents`.
