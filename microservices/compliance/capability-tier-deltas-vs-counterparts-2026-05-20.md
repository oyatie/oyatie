# Compliance customer-class delta memo superseded

Audit date: 2026-05-20.
Supersession date: 2026-05-21.
Target microservice: `microservices/compliance/`.

This memo is retained only as a historical audit pointer. ADR-0329 retired the
old customer-class ladder and ADR-0330 replaced it with:

- `tenant_class`: `demo_trial` or `paid`
- `billing_components`: `revenue_share`, `per_seat`, `per_usage`
- `compliance_pack` and `cell_topology` for regulatory or placement differences

Current compliance capability language:

- `demo_trial` may run bounded pack publishing and DSAR drills under OCI Always
  Free limits.
- `paid` carries the full compliance surface with commercial shape expressed by
  `billing_components`.
- Sovereign residency, regulator-attested publishing, air-gap custody, and
  cross-jurisdiction transfer evidence are `compliance_pack` and
  `cell_topology` requirements, not customer-class upsell steps.

The old detailed comparison should not be used for implementation planning.
Use `README.md`, `ARCHITECTURE.md`, `PRD.md`, ADR-0329, ADR-0330, and ADR-0331
for the active model.
