# developer-sdk

Tenant access follows ADR-0330: `tenant_class` is `demo_trial` or `paid`, and paid commercial shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`). SDK language availability, signing, sandboxing, and payout behavior are not separated by customer ladder labels.

This service accepts both tenant classes. `demo_trial` uses usage and time caps; `paid` uses the full capability surface with billing determined by composable components.
