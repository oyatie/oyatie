# oya-billing

Bespoke Rust billing engine per **ADR-0478** (supersedes Lago).

## Scope

Subscription lifecycle, plan management, and invoice state machines owned
entirely within this µservice. No external managed billing service dependency.

## Crates

| Crate | Layer | Purpose |
|---|---|---|
| `oya-billing-kernel` | kernel (D1-D2) | Pure subscription/plan/invoice state machines — no I/O |
| `oya-billing-rest` | api (D3) | axum HTTP + Connect-RPC surface |
| `oya-billing-app` | app (D4-D5) | Composition root — wires kernel + REST + PostgreSQL + Pulsar |

## Status

Scaffold only. Implementation tracks ADR-0478 D1-D5 delivery phases.

See [ADR-0478](../../docs/adr-archive/ADR-0478-oya-billing-bespoke-billing-engine.md).
