---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-019-library-first-caller-eval
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0255-amendment-library-first, ADR-0246]
---

# IP-019: Library-first caller-side eval (oya-intelligence-dispatch-sdk-rs)

## Intent

Ship `oya-intelligence-dispatch-sdk-rs`: the in-process Rust library that callers link to for
library-first dispatch per ADR-0255 amendment. This is the default path for Foundry agents,
Application Shell, and Builder backend code. No network hop; identical pipeline to the REST
handler (kernel, guardrails, credential-resolver, audit-tap) all in-process.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-dispatch-sdk-rs/Cargo.toml` | create |
| `crates/oya-intelligence-dispatch-sdk-rs/src/lib.rs` | create |
| `crates/oya-intelligence-dispatch-sdk-rs/src/client.rs` | create |
| `crates/oya-intelligence-dispatch-sdk-rs/src/builder.rs` | create |

## Code shape

```rust
/// Primary entry point for in-process callers.
pub struct IntelligenceClient {
    inner: Arc<DispatchUsecase<...>>,
}

impl IntelligenceClient {
    /// Synchronous builder — no network; all deps injected via DI container.
    pub fn builder() -> IntelligenceClientBuilder { ... }

    pub async fn dispatch(&self, req: DispatchRequest)
        -> Result<DispatchOutcome, DispatchError>;

    pub async fn dispatch_stream(&self, req: DispatchRequest)
        -> Result<impl Stream<Item = Result<DispatchChunk, DispatchError>>, DispatchError>;
}
```

## Key implementation notes

- `DispatchRequest` includes `audience_tag` — callers MUST set this (ADR-0244 §tenant-scoping).
- `policy_evaluation_mode = library_first`: Cedar eval runs in-process via `oya-shared-policy-eval`.
- Tracing: every in-process dispatch emits an OTLP span with `microservice=intelligence` + `dispatch_path=library_first`.
- No HTTP client instantiated for the in-process path; provider adapters share the same tokio runtime.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-dispatch-sdk-rs
cargo run -p oya-dev-cli -- gate validate library-first-dispatch-invariant --microservice intelligence
cargo run -p oya-dev-cli -- gate validate audience-tag-required --sdk oya-intelligence-dispatch-sdk-rs
```

## Test plan

- Unit: in-process dispatch round-trip with mock provider adapter.
- Unit: audience_tag missing → DispatchError::MissingAudienceTag.
- Unit: Cedar library-first eval fires; no network call to policy-engine µservice.
- Benchmark: in-process dispatch latency p99 < 5 ms overhead vs raw provider call.

## References

- `microservices/intelligence/ARCHITECTURE.md §3.1` (in-process dispatch).
- ADR-0255 amendment (library-first network-opt-in clarification).
- ADR-0246 amendment (policy-engine library-first).

## Wave 15 substance conversion — library-first dispatch SDK

### §A Problem

The architecture says library-first is canonical, but callers need a concrete SDK path or they will use REST for
in-cluster dispatch and add latency, policy drift, and auth complexity.
This IP closes the in-process dispatch seam for Foundry agents, builder backends, and product services.

### §B Approach

Provide `oya-intelligence-dispatch-sdk-rs` as a thin composition layer over the same usecase, guardrail,
credential-resolver, router, provider, and audit ports used by network handlers.
No SDK path bypasses audience tag, Cedar, or audit tap.

### §C Deliverables

- `crates/oya-intelligence-dispatch-sdk-rs/src/client.rs`
- `builder.rs`, `deps.rs`, and mock provider test support
- benchmarks for SDK overhead versus provider adapter call

### §D Implementation

1. Build `IntelligenceClient` from dependency-injected ports.
2. Require `audience_tag` at request construction.
3. Run Cedar library-first evaluation before provider routing.
4. Resolve credentials through the sidecar, not SDK environment variables.
5. Share stream handling with IP-016/IP-017.
6. Emit OTLP spans and audit records with `dispatch_path=library_first`.

### §E Acceptance

The library-first invariant gate must prove no HTTP policy-engine hop, missing audience tag rejection, and p99
overhead below the IP target.

### §F Evidence

Local anchors: `ARCHITECTURE.md` §3.1, `policy/dispatch-authorization.cedar`, `policy/byok-gating.cedar`.

### §G Counterparts

OpenAI, Anthropic, and Google mostly expose HTTP SDKs; oyatie closes a differentiated substrate gap by providing
in-process dispatch while keeping central policy and audit semantics.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-019-library-first-caller-eval.md` matched `p99`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.
