# cloud-billing-tax

`cloud-billing-tax` calculates tax, jurisdiction evidence, filing handoffs,
and tax-code catalog binding for the billing substrate.

## Tenant-class model

This microservice follows ADR-0330. The retired capability ladder is gone.
Tenant posture is expressed as `tenant_class`:

- `demo_trial`: $0 evaluation posture with OCI Always Free defaults and
  explicit calculation, catalog, and filing-simulation caps.
- `paid`: production posture with composable `billing_components`
  (`revenue_share`, `per_seat`, `per_usage`) inherited from cloud-billing.

Tax capability availability is not segmented by customer ladder. Paid tenants
receive production filing, compliance-pack, and jurisdiction coverage through
Cedar gates for `tenant_class`, `billing_components`, and active
`compliance_pack` state. Demo-trial tenants see the same model shape with
non-production caps.

Canonical replacement authority: ADR-0330.

