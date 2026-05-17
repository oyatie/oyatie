---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-004-router-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

# IP-004: oya-foundry-providers-router-api

## Intent

Protocol-neutral typed contracts (DTOs) shared by REST + gRPC + SDK + adapter layers. Mirrors OpenAPI + proto shapes but is the canonical Rust source of truth.

## ChangeSet boundary

One new crate `microservices/foundry-providers/src/crates/oya-foundry-providers-router-api/`. Depends on `oya-foundry-providers-router-kernel`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/dto/router.rs` | create — `RoutingRequestDto`, `RouterDecisionDto` |
| `.../src/dto/invoke.rs` | create — `InvokeRequestDto`, `InvokeResponseDto`, `InvokeStreamChunkDto` |
| `.../src/dto/health.rs` | create — `ProviderHealthSnapshotDto` |
| `.../src/dto/config.rs` | create — `TenantProviderConfigDto` |
| `.../src/dto/disclosure.rs` | create — `EuAiActDisclosureDto` |
| `.../src/conv.rs` | create — `From` impls between DTOs and kernel entities |

## Test Plan

| Test | Verifies |
|---|---|
| DTO ⇄ entity roundtrip | parity with kernel |
| DTO serde JSON | matches OpenAPI schema |
| DTO serde proto | matches proto schema |
| DTO does not include credential bytes | grep regex sweep |

## Acceptance Gates

Standard + `contract-conformance` lane (DTOs match `contracts/openapi/provider-router.yaml` + `contracts/proto/provider-invoke.proto`).

## Next IP

[`IP-005-router-adapter.md`](IP-005-router-adapter.md)
