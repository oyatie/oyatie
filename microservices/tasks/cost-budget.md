---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + finops
related_adrs: [ADR-0117, ADR-0131]
doc_status: published
---

# Cost Budget — tasks µservice

## Purpose

Define per-tenant and per-cell unit economics for tasks. FinOps gate: per-tenant monthly cost per active project / task / search-query / AI-invocation must stay within budget; cross-budget breach holds promotion + auto-throttles.

## Unit economics (per tenant per month, baseline pricing OCI ap-seoul-1, 2026-05)

| Unit | Baseline cost | Notes |
|---|---|---|
| Active project (1 project) | $0.50 / mo | Postgres rows + RLS overhead + custom-field schema |
| Task row (year-of-active-data) | $0.0002 / task-mo | Postgres + tenant-DEK envelope encryption overhead |
| Task update | $0.000005 / update | write throughput + audit emit |
| Cross-project search query | $0.00003 / query | Meilisearch CPU + memory |
| Recurrence materialisation | $0.0003 / expansion | worker CPU |
| Webhook fire | $0.00002 / fire | outbound dispatch + ack |
| Bulk-edit 100 tasks | $0.0003 / op | atomic transaction |
| .ics-style export (CSV/JSON; 10k tasks) | $0.02 / job | streaming emit |
| CSV import (10k tasks) | $0.05 / job | streaming parse |
| Jira / Asana / Trello / Linear / Todoist importer (10k tasks) | $0.10 / job | per importer (sandbox overhead) |
| Time-tracking tick (M02-onward) | $0.000001 / tick | append-only |
| T0 LLM suggest invocation | $0.001 / inv | foundry-runtime tenant-DEK wrapped |
| T1 LLM assist invocation | $0.005 / inv | larger context window |
| T2 LLM auto-action invocation | $0.01 / inv | full decision + reversibility window |

## Per-cell cost envelope (steady-state, p50 load)

| Component | Baseline / cell / mo | Notes |
|---|---|---|
| Postgres (task-store + project-list + dependency-edge; 3-replica HA + per-tenant RLS) | $4,800 | OCI VM.Standard3.Flex 16 OCPU × 3 + 4TB persistent block + S3 backup |
| Valkey (view-cache + presence; cluster mode 3-shard) | $1,100 | OCI Caching Service standard service class |
| Meilisearch (search-index; 3-node cluster) | $1,800 | OCI VM.Standard.E5.Flex × 3 (4 OCPU + 32 GB + 500 GB SSD per node) |
| Kubernetes nodes (rest + worker pods) | $2,800 | OCI VM.Standard.E5.Flex × 8 baseline |
| Foundry-runtime quota (T0/T1/T2 LLM invocations) | $2,500 | tenant_class and usage-cap bound; pay-per-invocation |
| Egress (webhook fanout + cross-µservice handoffs) | $500 | OCI egress to mail / messenger / calendar / drive µservices intra-region |
| Cross-pack mesh egress (when SCC-gated) | $50 | per-pack pair |
| Observability fan-out | $400 | per the observability µservice envelope |
| Backup storage (Postgres snapshots + Meilisearch backups + CSV/JSON export retention) | $300 | S3-compatible cold-storage class |
| **Total per cell baseline** | **$14,250 / mo** | sized for 10M active tasks / 500k projects |

## Per-tenant breakeven

| Tenant class / paid billing component | Monthly bill | Notes |
|---|---|---|
| demo_trial | $0; usage-capped project, task, AI, and importer volume | hard quota |
| paid + per_seat | per named user | covers collaboration and seat-governed usage |
| paid + per_usage | metered by task/update/search/import/export/timer/ai units | scales with admitted usage |
| paid + revenue_share | contract percentage where marketplace or downstream revenue applies | settlement owned by cloud-billing |

Breakeven: per-tenant gross margin target is contract-specific for paid tenant_class and fixed at hard quota for demo_trial.

## Cost governance gates

| Gate | Threshold | Action |
|---|---|---|
| Per-tenant monthly cost > 80% of contracted bill | warn FinOps | review billing-component fit |
| Per-tenant monthly cost > 100% of contracted bill | warn FinOps + tenant | over-quota notification + paid billing-component review |
| Per-tenant monthly cost > 150% of contracted bill | throttle (rate-limit lowered to tenant_class baseline limits) | auto-throttle |
| Per-cell total cost > 130% of baseline | hold promotion + finops review | review |
| Cross-pack egress > 5% of cell cost | finops review | identify cause |
| AI invocation cost > 25% of cell cost | engineering review | optimisation needed |
| Meilisearch full-rebuild storm > 1/day cumulative | finops + sre review | per-tenant rebuild quota lowered |

## Cost-meter implementation

Cost-meter emitted as Mimir metric per ADR-0123 + finops standards:

| Metric | Cardinality | Labels |
|---|---|---|
| `tasks_tenant_cost_dollars_per_month` | per-tenant | `tenant_id_hashed`, `tenant_class`, `paid_billing_component`, `pack_tag` |
| `tasks_unit_cost_dollars` | per-unit | `unit_type` (task/update/search/expansion/booking/webhook/import/export/timer/ai-t0/ai-t1/ai-t2), `pack_tag` |
| `tasks_cell_cost_dollars_per_month` | per-cell | `pack_tag`, `cell_id` |
| `tasks_egress_dollars` | per-cross-pack-pair | `from_pack`, `to_pack` |

## Optimisations identified at design time

| Optimisation | Estimated savings | Status |
|---|---|---|
| Per-tenant partition pruning on Postgres | 40% on read latency + 25% on storage | Implemented in PHASE-01 CS-02 |
| Valkey view-cache TTL ≤ 60s + single-flight | 70% on view-render Postgres reads | Implemented in PHASE-01 CS-07 |
| Meilisearch incremental indexing (delta-only) | 80% on indexing CPU vs full-rebuild | Implemented in PHASE-01 CS-08 |
| Pre-warm pod pool (5 standby) | sub-1s cold-start vs 5s on-demand | Implemented in PHASE-01 CS-03 |
| Foundry-runtime quota caching (T0 same-prompt) | 50% on T0 invocation cost | Implemented in PHASE-01 CS-15 |
| Webhook fanout batching | 60% on dispatch overhead | Implemented in PHASE-01 CS-15 |
| Bulk-edit transaction batching | 70% on transaction commit overhead | Implemented in PHASE-01 CS-01 |

## FinOps reporting cadence

- Per-tenant cost report: monthly.
- Per-cell cost report: monthly.
- Cross-pack egress report: weekly.
- AI invocation cost report: weekly.
- Cost-anomaly alert: daily (≥ 30% deviation from rolling 7d baseline).

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- `capacity-model.md`, `multi-region.md`.
- OCI ap-seoul-1 pricing (2026-05).
- `microservices/calendar/cost-budget.md` — sibling reference template.
