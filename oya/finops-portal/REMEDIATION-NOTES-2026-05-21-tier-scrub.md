# Wave 15J-batch-4 Tier Scrub — finops-portal

## Scope

- Bucket: BUCKET-10.
- Service: `finops-portal`.
- Doctrine: ADR-0329, ADR-0330, ADR-0331.

## Files Modified

- `ARCHITECTURE.md` — 1058 lines.
- `README.md` — 90 lines.
- `benchmarks/aws-cost-explorer-gcp-billing-apptio-vs-oyatie.md` — 119 lines.
- `tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md` — 431 lines.
- `coherence-audit-2026-05-20.md` — 747 lines.
- `faqs/finops-engineer-faq.md` — 113 lines.
- `feature-parity-matrix-2026-05-20.md` — 430 lines.
- `manifest.json` — 158 lines.
- `migration-playbooks/from-apptio-cloudability.md` — 189 lines.
- `onboarding/finops-engineer-first-week.md` — 244 lines.
- `performance-benchmark-numbers-2026-05-20.md` — 312 lines.
- `reference-implementations/cost-query-rust-sdk.md` — 238 lines.
- `tutorials/build-chargeback-dashboard.md` — 331 lines.

## Retirement Actions

- `capability-tiers/` deleted: Y.
- Vocabulary replacement count: roughly 270 service-local replacements.
- Former capability-tier delta artifact renamed to tenant-class adoption language.
- README updated with ADR-0330 tenant_class and billing_components guidance.

## Design Decisions

- Forecast, dashboard, chargeback, and export behavior now use demo_trial caps, paid billing_components, compliance_pack activation, and cell_topology rather than customer capability tiers.
- The SLO and benchmark prose no longer presents differentiated customer quality levels; scale differences are framed as tenant_class caps or deployment-context capacity.

## Follow-ups

- None for the Wave 15J hard vocabulary gate.
