# cell-lifecycle

cell-lifecycle owns the logical Cell aggregate state machine introduced by ADR-0276 D-3.

It is not the retired generic `cell` service from ADR-0276. It is a narrow lifecycle authority for Registered, Activated, Promoted-T4, Promoted-T3, Promoted-T2, Promoted-T1, Promoted-T0, Draining, and Decommissioned states.

## Boundaries
- Infrastructure provisioning is delegated to cloud-iac.
- Tenant migration during drain is delegated to cell-rebalancer.
- Routing is delegated to api-gateway.
- Resident count and tenant-class coverage are read from tenancy.
- SLO, canary, and mesh evidence are read from observability.
- Transition evidence is sealed through audit-chain.
- Privileged operations are authorized through Cedar.

## Authoring Contents
- `PRD.md` defines product scope, personas, triggers, state machine, promotion gates, Cedar model, and compliance invariants.
- `ARCH.md` defines hexagonal architecture, layer placement, data model, diagrams, persistence, sharding, DR, and transport.
- `IPs/` carries eight implementation plans for downstream Wave 15-ZD-impl execution.
- `contracts/openapi.yaml` defines the OpenAPI 3.2.0 REST contract.
- `cedar/policies.cedar` defines policy fragments for Foundry automation, drain, decommission, and tier-specific promotion.
- `slos/cell-lifecycle.openslo.yaml` defines the service SLOs from ADR-0276 D-3.4.
- `runbooks/` covers promote, emergency drain, decommission, rollback, and on-call flows.

## Implementation Status
This scaffold is documentation and contract authoring only. No Rust crates, handlers, migrations, or deployment manifests are created in this wave.
