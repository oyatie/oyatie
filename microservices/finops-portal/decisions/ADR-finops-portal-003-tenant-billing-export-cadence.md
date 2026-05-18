---
adr_id: finops-portal-003
authored: 2026-05-18
status: accepted
authority_chain: ADR-0174 + ADR-0199
microservice: finops-portal
---

# ADR finops-portal-003 — Tenant billing export cadence

## Context

Tenants need cost data on a cadence that fits their finance
pipeline. Three cadence options exist:

1. **Daily** — frequent; good for real-time monitoring.
2. **Monthly** — calendar-aligned; matches contractual billing.
3. **On-demand** — tenant pulls when ready.

The platform default cadence affects load on OpenCost + Mimir +
SeaweedFS and the audit-chain event volume.

## Decision

- **Invoice finalization**: monthly, on calendar-month close +
  3-day cure window (covers late-arriving cost data).
- **Drill-down data**: live (5min freshness per OpenCost scrape).
- **FOCUS export**: on-demand via the public API; pre-warmed at
  invoice finalization for the prior month.
- **Quarterly regulator emit**: quarterly, on calendar-quarter
  close + 5-day cure window (per IP-015).

## Rationale

1. **Monthly invoice** aligns with most tenant CFO pipelines + the
   chargeback formula in ADR-0174.
2. **Live drill-down** preserves the interactive UX (5min lag is
   acceptable; matches OpenCost cadence).
3. **On-demand FOCUS** caps SeaweedFS storage; pre-warming the
   prior month covers the common case at zero extra cost.
4. **Quarterly regulator** matches typical regulatory cadence
   (PIPA, GDPR Art. 30 records, SOX ICFR attestations).

## Consequences

- Postgres + SeaweedFS sized per the monthly + quarterly cadence.
- Audit-chain event volume is bounded predictably.
- Tenants pulling FOCUS more than 12/hour hit the rate limit
  (`sdk-reference.md`).

## Alternatives considered

- **Daily invoice**: rejected because most tenants do not have
  daily finance reconciliation.
- **Weekly regulator emit**: rejected because regulators expect
  quarterly + the audit-chain event volume would grow ~13×.

## References

- ADR-0174 chargeback formula.
- ADR-0199 FinOps canonical.
- IP-014, IP-015.
- `capacity-model.md`.
