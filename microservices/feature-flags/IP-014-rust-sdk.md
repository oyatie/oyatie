# IP-014 — Rust SDK

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: rust-sdk
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0211, ADR-0245, ADR-0248, ADR-0253, ADR-0255, ADR-0258
**companion_ips**: IP-013, IP-015, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

Rust SDK implementing the OpenFeature provider interface. Primary SDK per ADR-0211 (Rust-primary stack). Used by all internal Rust µservices (46+ consumers). Sync + async APIs; tonic+QUIC transport; `DashMap` cache; SSE stream via `reqwest`.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieFeatureFlagProvider` struct | Implements OpenFeature `FeatureProvider` trait; async Tokio runtime |
| 2 | `tonic+quic` transport | `tonic` gRPC over QUIC; X25519MLKEM768 KEM negotiation if server supports; TLS 1.3 floor |
| 3 | `DashMap` cache | `DashMap<(Arc<str>, Arc<str>), CachedFlag>`; TTL 30s; LKG: serde_json disk file at `$XDG_CACHE_HOME/oya-ff/lkg.json` |
| 4 | SSE invalidation | `reqwest` EventSource to `/api/v1/flags/stream`; per-tenant channel; reconnects with exponential backoff |
| 5 | Sync wrapper | `resolve_boolean_value_sync()` via `tokio::task::block_in_place` for non-async callers |
| 6 | `FlagEvaluationError` | Implements `std::error::Error`; maps all OpenFeature error codes |
| 7 | Crate features | `feature = ["grpc"]` (default), `feature = ["http"]` (REST fallback), `feature = ["wasm"]` (no-std WASM target) |
| 8 | Tests | 95%+ coverage; benchmark: `resolve_boolean_value` in-cache ≤5µs; SSE reconnect test |

## Usage

```rust
use oya_feature_flags_sdk::OyatieFeatureFlagProvider;

let provider = OyatieFeatureFlagProvider::builder()
    .endpoint("https://feature-flags.internal")
    .tenant_id("tenant_abc")
    .build()
    .await?;

let ctx = EvaluationContext::builder()
    .audience_type(AudienceType::B2B)
    .targeting_key("user_xyz")
    .build();

let enabled: bool = provider.resolve_boolean_value("my-flag", false, &ctx).await?;
```

## Definition of Done

- `cargo test -p oya-feature-flags-rust-sdk` green
- Benchmark: in-cache resolution ≤5µs
- WASM feature builds with `cargo build --target wasm32-unknown-unknown --features wasm`
- OpenFeature conformance suite passes (Rust target)
