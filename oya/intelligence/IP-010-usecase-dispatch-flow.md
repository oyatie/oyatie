---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-010-usecase-dispatch-flow
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-010: Usecase — dispatch-flow orchestrator

## Intent

`oya-intelligence-model-routing-usecase`: orchestrator that reads ports + executes the dispatch
pipeline per ARCHITECTURE.md §3 (Library-first dispatch flow).

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-model-routing-usecase/Cargo.toml` | create |
| `.../oya-intelligence-model-routing-usecase/src/lib.rs` | create |
| `.../oya-intelligence-model-routing-usecase/src/dispatch_usecase.rs` | create |
| `.../oya-intelligence-model-routing-usecase/src/dispatch_stream_usecase.rs` | create |

## Code shape

```rust
pub struct DispatchUsecase<R, C, H, G, A, Au, S> {
    router: R,                 // ModelRouterPort
    catalog: C,                // ProviderCatalogPort
    health: H,                 // ProviderHealthPort
    guardrails: G,             // GuardrailStackPort (pre + post + baseline + annex_iii + abuse_defence)
    adapters: A,               // ProviderAdapterRegistry
    audit_tap: Au,             // AuditTapPort
    credential_resolver: S,    // CredentialResolverPort
}

impl<...> DispatchUsecase<...> {
    pub async fn dispatch(&self, request: DispatchRequest) -> Result<DispatchOutcome, DispatchError> {
        // 1. abuse-defence pre-call gate
        // 2. dispatch-authorization Cedar evaluation
        // 3. pre-call classification + refusal-baseline + Annex III
        // 4. routing decision
        // 5. credential-resolver
        // 6. provider adapter invocation
        // 7. post-call classification + refusal-baseline (output side)
        // 8. attribution rendering
        // 9. audit-tap atomic commit
        // 10. return DispatchOutcome
    }

    pub async fn dispatch_stream(&self, request: DispatchRequest)
        -> Result<impl Stream<Item = Result<DispatchChunk, DispatchError>>, DispatchError> { ... }
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-usecase
buck2 build //:quality-lane-registry-authority-check # lane=library-first-dispatch-invariant --microservice intelligence
buck2 build //:quality-lane-registry-authority-check # lane=audit-tap-atomicity --microservice intelligence
```

## Test plan

- Happy path: dispatch admits → routes → adapter invokes → output filters → audit-tap commits.
- Refusal path: every refusal reason terminates the pipeline with audit-tap committed.
- Audit-tap failure pre-commit: dispatch returns `AuditTapEmitFailed`; no provider invocation.
- Streaming: chunks emit while audit-tap commit is deferred to final chunk.

## Next IP

[`IP-011-adapter-anthropic.md`](IP-011-adapter-anthropic.md)

## References

- `microservices/intelligence/ARCHITECTURE.md` §3.
- `microservices/intelligence/threat-model.md` (T-T-01..05).

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-010-usecase-dispatch-flow.md` matched `attribution`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
