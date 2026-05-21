---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-21
related_adrs: [ADR-0329, ADR-0330, ADR-0331]
---

# Tasks µservice README

Tasks owns task storage, projects and boards, custom fields, dependencies,
recurrence, workflow-state links, realtime board views, search, bulk edit, and
AI-assist bounds.

## Tenant Class Model

Tasks follows ADR-0330. Customer access is modeled with `tenant_class`
(`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial behavior is enforced by
usage caps and gateway policy; paid tenants receive the same product surface
with billing components composed by contract.

## Quick Links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Manifest: `manifest.json`
- Cost budget: `cost-budget.md`
- SLOs: `slos/*.openslo.yaml`
- Cedar fragments: `policy/*.cedar`
- ADR-0330: `../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`
