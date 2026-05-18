---
doc_class: COST-BUDGET
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-finops + axis-foundry
related_adrs: [ADR-0136, ADR-0137]
---

# Cost Budget — foundry (consolidated)

## Scope

This consolidated cost budget aggregates the six BC budgets. Per-BC budgets
remain authoritative at `bc-sources/<bc>/cost-budget.md`.

## Aggregate budget (M01 launch, pack-kr, XS tier)

| BC | Compute (USD/mo) | Storage (USD/mo) | Egress (USD/mo) | Total (USD/mo) |
|---|---|---|---|---|
| runtime | 8,400 | 1,200 (Redis HA + Postgres) | 600 | 10,200 |
| supervisor | 2,800 | 800 (Postgres) | 200 | 3,800 |
| eval | 4,200 (incl. GPU pool baseline) | 1,800 (ClickHouse + S3 golden) | 400 | 6,400 |
| evidence | 2,400 | 2,800 (Postgres + S3 blob) | 300 | 5,500 |
| guardrails | 3,800 (incl. ONNX classifier serving) | 600 (Postgres rule store) | 100 | 4,500 |
| providers | 1,800 (router + 8 adapters) | 400 (Redis rate-limit + OpenBao) | 200 | 2,400 |
| **Total** | **23,400** | **7,600** | **1,800** | **32,800** |

Plus shared overhead (observability + audit-chain bridge + control-plane
overhead): ~4,000 USD/mo. **Total M01 foundry: ~36,800 USD/mo at XS tier.**

## Variable-cost components

| Component | Driver | Unit cost | Notes |
|---|---|---|---|
| LLM token spend (passthrough) | provider invocation | per-provider per-1k tokens | Recorded by providers BC as receipts; passed to tenant billing |
| Eval GPU hours | eval-run scheduling | ~3 USD/hr per A100 slot | Pool scales 0 → 16 slots; idle hours = 0 |
| Evidence pack S3 egress | regulator-export downloads | ~0.05 USD/GB | Per-tenant cap; scheduled-for-distinct-tracked-work-queue prevents spike |
| Audit-chain Merkle seal | invocation rate | ~0.0001 USD per seal | Aggregated; batched per (tenant,minute) |

## Per-BC cost archives

- `bc-sources/runtime/cost-budget.md`
- `bc-sources/supervisor/cost-budget.md`
- `bc-sources/eval/cost-budget.md`
- `bc-sources/evidence/cost-budget.md`
- `bc-sources/guardrails/cost-budget.md`
- `bc-sources/providers/cost-budget.md`

## Budget alarms

- Per-BC daily spend > 110% of monthly/30 average → page ops-finops.
- Foundry aggregate daily spend > 105% of monthly/30 average → page
  ops-finops + axis-foundry.
- Per-tenant invocation rate > 200% of declared ceiling → throttle +
  notify tenant + axis-foundry.

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/capacity-model.md` — capacity envelope behind these
  costs.
