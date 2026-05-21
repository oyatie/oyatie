---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
---

# Observability µservice README

Observability owns OpenSLO validation and evaluation, telemetry ingest,
promotion evidence, rollback signals, ClickHouse extension work, dashboards,
and runbooks for operating the telemetry substrate.

## Tenant Class Model

Observability follows ADR-0330. Customer access is modeled with
`tenant_class` (`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial use is bounded by
retention, sampling, and OCI Always Free constraints; paid use scales by
deployment context, cell topology, and compliance-pack obligations rather than
customer capability ladders.

## Quick Links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Manifest: `manifest.json`
- Cost budget: `cost-budget.md`
- SLOs: `slos/*.openslo.yaml`
- Cedar fragments: `policy/*.cedar`
- ADR-0330: `../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`
