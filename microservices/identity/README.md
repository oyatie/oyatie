# identity µservice

`identity` issues and verifies principal claims for Oyatie tenants, including
the canonical `tenant_class` claim used by ADR-0330 and ADR-0331.

## Tenant Class Model

`identity` does not model customer capability tiers. Tokens and policy context
carry `tenant_class = demo_trial | paid`; paid commercial shape is supplied by
`billing_components` (`revenue_share`, `per_seat`, `per_usage`) owned by
cloud-billing and emitted as principal context for Cedar.

The service must not expose retired customer-tier fields, examples, or contract
enums. Product availability is uniform; differences are expressed as demo_trial
caps, paid billing_components, compliance_pack activation, or cell_topology and
criticality controls.

## Key Surfaces

- `ARCHITECTURE.md` and `PRD.md` describe identity substrate behavior.
- `contracts/` contains OpenAPI, AsyncAPI, and proto contracts.
- `policy/tenant-class.cedar` is the tenant_class policy anchor.
- `capabilities/tenant-class-caps.yaml` records demo_trial cap behavior.
- `slos/` and `dashboards/` expose service health without customer tiers.
