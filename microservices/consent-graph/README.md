# consent-graph

Tenant access follows ADR-0330: `tenant_class` is `demo_trial` or `paid`, and paid commercial shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Consent enforcement reads `tenant_class` from the principal/context claim; compliance-sensitive behavior belongs under `compliance_pack` or `cell_topology`.

This service accepts both tenant classes. `demo_trial` is cap-bound; `paid` receives the same capability surface with compliance-pack eligibility governed by Cedar.
