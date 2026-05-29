---
microservice: connector
doc_class: CostBudget
date: 2026-05-20
owner_team: axis-integration + finops
status: Accepted
related_adrs: [ADR-0248]
doc_status: published
---

# Cost Budget — connector (Integration Substrate)

## Per-call cost model

| Component | Unit | Cost (USD) | Notes |
|---|---|---|---|
| Connector action invocation (overhead) | per-call | $0.000001 | CPU + RAM in Kata sandbox |
| Webhook receive | per-event | $0.0000005 | HMAC verify + enqueue |
| OAuth grant creation | per-grant | $0.0001 | OAuth dance + PG write + OpenBao write |
| OAuth token refresh | per-refresh | $0.00001 | Vendor call + PG update + OpenBao update |
| Catalog query | per-query | $0.00000005 | ES query |
| Audit event seal | per-event | $0.0000005 | KMS sign |
| DLQ entry storage | per-entry-day | $0.000001 | PG row + index |

## Annual budget (at 50k tenants, 24mo target)

- Adapter actions: 100k/sec × 86400 × 365 = 3.15T/yr × $0.000001 = **$3.15M/yr**
- Webhook receives: 100M/day × 365 = 36.5B/yr × $0.0000005 = **$18.25k/yr**
- OAuth refreshes: 100k grants × 12 refreshes/yr = 1.2M/yr × $0.00001 = $12/yr (negligible)
- Catalog queries: 1M/day × 365 = 365M/yr × $0.00000005 = $18/yr (negligible)
- Audit events: 100M/day × 365 = 36.5B/yr × $0.0000005 = **$18.25k/yr**
- DLQ: avg 1M entries × 7d retention × $0.000001 = $7/day = $2.5k/yr

**Total: ~$3.2M/yr compute + ~$40k/yr metadata** at 50k tenants.

## Cost drivers

1. **Adapter dispatch CPU/RAM** (dominant): scales with action volume. Mitigation: WASM-vs-native trade-off study at M02; if WASM startup ≤5ms it could replace Kata for stateless connectors.
2. **OpenBao TTL renewals**: ~1.6k reads/sec/pack. Mitigation: token cache with lazy refresh; budget headroom 10×.
3. **Audit chain KMS signs**: 100M signs/day. Mitigation: batch-signing for low-severity events (per ADR-0263 §F).

## Cost attribution

Per ADR-0249 multi-category marketplace + FinOps portal integration:
- Per-tenant: tracked via `oya_finops_attribution_total{tenant_id,bc,action}`.
- Per-connector: tracked via `oya_finops_attribution_total{connector_name}`.
- Tenant invoices (via finops-portal): split vendor-cost-pass-through from oyatie-substrate-cost.

## Pre-launch optimization

| Optimization | Estimated saving |
|---|---|
| Lazy adapter load (Kata spin-up on first use) | 80% RAM at idle |
| Hedged request elimination on cold paths | 5% latency tail; 0 cost |
| Tail sampling per ADR-0263 §G | 90% trace storage reduction |
| Aggressive DLQ retention cap (default 7d, was 14d) | 50% PG storage |

## References

- ADR-0249 multi-category marketplace + finops integration
- `microservices/finops-portal/PRD.md`

## Retirement-coordination addendum

The retirement-tracking scope (umbrella dissolution) retains a near-zero operating cost (`<$100/yr` for coordination events); the substrate budget above is independent and supersedes for the active scope.
