---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-007-kernel-model-router
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-007: Kernel — model-router port traits

## Intent

Author port traits in `oya-intelligence-model-routing-kernel` that the usecase + adapter layers
implement. Pure trait definitions; no concrete I/O.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-model-routing-kernel/Cargo.toml` | create |
| `.../oya-intelligence-model-routing-kernel/src/lib.rs` | create |
| `.../oya-intelligence-model-routing-kernel/src/model_router_port.rs` | create |
| `.../oya-intelligence-model-routing-kernel/src/provider_catalog_port.rs` | create |
| `.../oya-intelligence-model-routing-kernel/src/provider_health_port.rs` | create |

## Code shape

```rust
#[async_trait]
pub trait ModelRouterPort: Send + Sync + 'static {
    async fn route(
        &self,
        request: &DispatchRequest,
        catalog: &ProviderCatalog,
        health: &ProviderHealthSnapshot,
    ) -> Result<RoutingDecision, RoutingError>;
}

#[async_trait]
pub trait ProviderCatalogPort: Send + Sync + 'static {
    async fn current(&self) -> Result<ProviderCatalog, CatalogError>;
}

#[async_trait]
pub trait ProviderHealthPort: Send + Sync + 'static {
    async fn snapshot(&self) -> Result<ProviderHealthSnapshot, HealthError>;
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-kernel
```

## Test plan

- Trait satisfies object-safe constraint.
- Mock implementations compile.

## Next IP

[`IP-008-kernel-guardrail-stack.md`](IP-008-kernel-guardrail-stack.md)

## References

- ADR-0145 (direct gRPC), ADR-0056 (BNF).

## Wave 15 substance conversion — model router kernel

### §A Problem

Provider choice cannot live in REST handlers or provider adapters because routing depends on tenant, pack, modality,
health, credentials, and policy.
This IP closes the kernel seam that keeps provider selection testable and independent of I/O.

### §B Approach

Define `ModelRouterPort`, `ProviderCatalogPort`, and `ProviderHealthPort` traits in the model-routing kernel.
Usecases compose these ports; adapters implement health/catalog lookups outside the kernel.

### §C Deliverables

- `crates/oya-intelligence-model-routing-kernel/src/model_router_port.rs`
- `provider_catalog_port.rs`, `provider_health_port.rs`, and `routing_error.rs`
- object-safety and mock-router tests

### §D Implementation

1. Accept `DispatchRequest`, `ProviderCatalog`, and `ProviderHealthSnapshot`.
2. Return `RoutingDecision` without calling any provider.
3. Treat Cedar denial and no compliant provider as distinct errors.
4. Preserve fallback count for audit and cost attribution.
5. Keep direct gRPC/REST imports out of kernel types.
6. Exercise pack-cn, pack-eu, and byok-required scenarios in mock tests.

### §E Acceptance

Nextest must prove object-safe trait use, compile mock implementations, and reject cross-layer imports.

### §F Evidence

Local anchors: `manifest.json` model-routing BC, `policy/provider-routing.cedar`, `multi-region.md`.

### §G Counterparts

AWS Bedrock, OpenRouter, OpenAI, and Anthropic all expose model-selection or fallback surfaces; oyatie closes that
gap with a provider-neutral kernel that adds Cedar and pack-aware routing before adapter dispatch.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-007-kernel-model-router.md` matched `multi-region`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-007-kernel-model-router.md` matched `attribution, cost`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
