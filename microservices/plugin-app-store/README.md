# plugin-app-store

Tenant access follows ADR-0330: `tenant_class` is `demo_trial` or `paid`, and paid commercial shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Publisher trust is modeled as `trust_verdict`; commercial eligibility is modeled through `tenant_class` plus billing components.

This service accepts both tenant classes. `demo_trial` is usage-capped; `paid` enables marketplace billing flows through composable billing components.
