# oya-meter

Bespoke Rust usage metering µservice. See [ADR-0479](../../docs/adr-archive/ADR-0479-oya-meter-bespoke-usage-metering.md) for design and delivery phases (D1–D5).

## Crates

| Crate | Layer | Purpose |
|---|---|---|
| `oya-meter-kernel` | kernel | Pure state machine: event ingestion + aggregation |
| `oya-meter-rest` | rest | Axum HTTP surface |
| `oya-meter-app` | app | Composition root binary |
