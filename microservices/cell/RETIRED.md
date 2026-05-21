# Cell microservice retired

Status: retired as a standalone microservice.

Decision: [ADR-0333](../../docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md) retires the cell microservice. Cellular architecture remains active as the ADR-0248 pattern, but its responsibilities are absorbed by adjacent owners.

Absorption targets:

- Tenant to cell assignment: [tenancy architecture](../tenancy/ARCHITECTURE.md#cell-assignment).
- Cell provisioning, lifecycle, registry, and capacity model: [cloud-iac architecture](../cloud-iac/ARCHITECTURE.md#cell-provisioning).
- Cell health, SLO burn, blast-radius monitoring, and isolation alerts: [observability architecture](../observability/ARCHITECTURE.md#cell-health).
- Shuffle-sharding algorithm: [oya-shuffle-sharding crate](../../crates/oya-shuffle-sharding/README.md).
- Per-cell audit scoping: [audit-chain architecture](../audit-chain/ARCHITECTURE.md#cell-scoped-audit).
- Cell-aware tenant routing: [api-gateway architecture](../api-gateway/ARCHITECTURE.md#cell-aware-routing).

Rules:

- Do not add new service artifacts under this directory.
- Do not restore the retired PRD, contracts, dashboards, runbooks, catalogs, or implementation-plan files here.
- Preserve the cellular architecture doctrine through the successor owners listed above.
