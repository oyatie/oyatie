---
microservice: compliance
doc: CostBudget
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-finops]
date: 2026-05-18
related_adrs: [ADR-0174, ADR-0209]
---

# Compliance — Cost Budget

## Steady-state monthly budget

| Line item | Estimate (USD / month) |
|---|---|
| Evidence-collector tier (3 replicas × 250m CPU + 512Mi RAM) | $30 |
| Storage adapter pod | $10 |
| Backstage auditor plugin host | (shared with Backstage; no additional cost) |
| SeaweedFS evidence storage — hot bucket (90 days × 32 µservices × ~ 50 GB/quarter / 3) | $300 |
| SeaweedFS evidence storage — cold archive (7 years × ~ 200 GB/yr) | $50 |
| Cosign keyless OIDC chain — Sigstore Fulcio reads | $20 |
| AlertManager + PagerDuty webhook routes | (shared with observability) |
| **Total** | **~ $410/month** |

## Per-tenant additional cost

| Driver | Per tenant / month |
|---|---|
| Per-tenant DSAR volume (10 DSARs/day × $0.05 cost) | $15/mo |
| Per-tenant min-necessary log volume (HIPAA tenants only; varies) | $20/mo typical |
| Per-tenant auditor portal sessions during engagement | $0 (in budget) |

## Annual budget

- Steady-state: ~ $5,000 / year baseline + ~ $35/tenant/month variable.
- Compare vs Drata baseline ~$25k/year + per-employee fees → 80% cost reduction at 100-employee scale.

## Budget exceptions

| Event | Estimated burst | Mitigation |
|---|---|---|
| Pen-test report upload | +$5 one-time | absorbed |
| Annual audit engagement | +$200/mo for 1-2 months | budgeted |
| Sev-1 audit-chain seal failure investigation | +$500 one-time | budgeted |
| Mass DSAR attack (50× normal rate) | +$1000/mo until mitigated | circuit-break activates |

## FinOps tags

Per ADR-0174-finops-cost-attribution-chargeback:

- `oya.cost.component=compliance`
- `oya.cost.tier=substrate`
- `oya.cost.tenant_id=<tenant>` (for tenant-scoped resources)

## References

- ADR-0174 — finops cost attribution.
- ADR-0209 — substrate authority.
