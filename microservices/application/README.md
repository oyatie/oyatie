# application

Tenant access follows ADR-0330: `tenant_class` is `demo_trial` or `paid`, and paid commercial shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Shell routing, module loading, sessions, and tenant-admin behavior are not segmented by customer ladder labels.

This service accepts both tenant classes. `demo_trial` uses cap-bound runtime profiles; `paid` receives the same application surface with scaling controlled by deployment context and cell topology.
