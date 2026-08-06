# oya-cost

Bespoke Rust K8s cost allocation engine per **ADR-0480** (supersedes OpenCost).

## Scope

Kubernetes resource usage aggregation, allocation rules, chargeback state
machines, and per-tenant cost reporting owned entirely within this µservice.
No external managed cost service dependency.

## Crates

| Crate | Layer | Purpose |
|---|---|---|
| `oya-cost-kernel` | kernel (D1-D2) | Pure usage aggregation / allocation rules / chargeback value objects — no I/O |
| `oya-cost-rest` | api (D3) | axum HTTP + Connect-RPC surface |
| `oya-cost-app` | app (D4-D5) | Composition root — wires kernel + REST + PostgreSQL + Pulsar |

## Status

Scaffold only. Implementation tracks ADR-0480 D1-D5 delivery phases.

See [ADR-0480](../../docs/adr-archive/ADR-0480-oya-cost-bespoke-k8s-cost-allocation.md).
