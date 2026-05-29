# oya-flags

Bespoke Rust feature flag server speaking the OpenFeature Remote Evaluation
Protocol (OFREP). See **ADR-0481** for the full design rationale.

## Crate layout

| Crate | Layer | Purpose |
|---|---|---|
| `oya-flags-kernel` | kernel | Pure-Rust OpenFeature evaluation state machine; sub-ms target; no I/O |
| `oya-flags-rest` | rest | OFREP HTTP request/response types + dispatch helpers |
| `oya-flags-app` | app | Binary composition root (`oya-flags`) |

## Build

```
cargo check -p oya-flags-kernel -p oya-flags-rest -p oya-flags-app
cargo test  -p oya-flags-kernel -p oya-flags-rest -p oya-flags-app
```
