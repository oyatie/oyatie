# drive

The `drive` microservice follows the ADR-0330 tenant-class model:
`tenant_class` is either `demo_trial` or `paid`, and paid commercial shape is
expressed through `billing_components` (`revenue_share`, `per_seat`,
`per_usage`). Feature availability is not customer-level laddered; product
quality is uniform, while demo_trial caps, compliance_pack activation, and
cell_topology placement carry the legitimate gates.

See `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.
