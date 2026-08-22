# flags

Bespoke Rust feature flag server speaking the OpenFeature Remote Evaluation
Protocol (OFREP). See **ADR-0481** for the full design rationale.

## Crate layout

| Crate | Layer | Purpose |
|---|---|---|
| `flags-kernel` | kernel | Pure-Rust OpenFeature evaluation state machine; sub-ms target; no I/O |
| `flags-rest` | rest | OFREP HTTP request/response types + dispatch helpers |
| `flags-app` | app | Binary composition root (`flags`) |

## Build

```
cargo check -p flags-kernel -p flags-rest -p flags-app
cargo test  -p flags-kernel -p flags-rest -p flags-app
```
