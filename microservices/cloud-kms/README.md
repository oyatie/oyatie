# cloud-kms

`cloud-kms` owns CMK/KEK/DEK lifecycle semantics, envelope encryption, HSM custody, rotation, cryptoshred receipts, signing paths, and no-roll-your-own-crypto enforcement.

This microservice follows the ADR-0330 `tenant_class` model:

- `demo_trial`: OCI Always Free default profile with explicit time and usage caps.
- `paid`: full production availability with composable `billing_components` (`revenue_share`, `per_seat`, `per_usage`).

Capability availability is no longer expressed through customer ladder labels. Product-quality differences must be modeled through `compliance_pack`, `cell_topology`, or context-specific capacity envelopes.

Reference: `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.
