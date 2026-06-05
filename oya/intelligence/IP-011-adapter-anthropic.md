---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-011-adapter-anthropic
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0296, ADR-0253]
---

# IP-011: Provider adapter — Anthropic

## Intent

`oya-intelligence-providers-adapter-anthropic`: implements the `ProviderAdapterPort` for the
Anthropic Messages API (Claude Opus 4.7 / Sonnet 4 / Haiku 4). Handles SSE streaming, provider-BYOK
credential handle injection, EU-residency routing, HIPAA-BAA provider gating.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-providers-adapter-anthropic/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-anthropic/src/lib.rs` | create |
| `crates/oya-intelligence-providers-adapter-anthropic/src/client.rs` | create |
| `crates/oya-intelligence-providers-adapter-anthropic/src/streaming.rs` | create |
| `crates/oya-intelligence-providers-adapter-anthropic/src/error.rs` | create |

## Code shape

```rust
pub struct AnthropicAdapter {
    http_client: reqwest::Client,   // HTTP/3 + QUIC per ADR-0253; h2 fallback
    base_url: Url,                   // configurable for EU-residency override
}

impl ProviderAdapterPort for AnthropicAdapter {
    async fn invoke(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<ProviderResponse, ProviderError>;

    async fn invoke_stream(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<impl Stream<Item = Result<ProviderChunk, ProviderError>>, ProviderError>;
}
```

## Key implementation notes

- `CredentialHandle` injected at HTTP-call assembly time; never stored in adapter memory (ADR-0296).
- SSE streaming via `reqwest` event-source; backpressure via bounded channel.
- EU-residency: `base_url` overridden to Anthropic EU endpoint when `pack-eu` active.
- HIPAA BAA: refuse if `pack-us-healthcare` active and provider not BAA-signed (Cedar gate upstream, but adapter double-checks).
- `X-Oya-Bot-Score` forwarded from upstream request metadata.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-providers-adapter-anthropic
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-byok --provider anthropic
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-streaming --provider anthropic
```

## Test plan

- Unit: mock HTTP server returns SSE stream; adapter emits chunks in order.
- Unit: credential handle injected as `x-api-key`; never logged.
- Unit: EU-residency URL override applied when `pack-eu`.
- Integration: against Anthropic sandbox (CI secret injected).

## Next IP

[`IP-012-adapter-openai.md`](IP-012-adapter-openai.md)

## References

- `microservices/intelligence/ARCHITECTURE.md §6` (provider matrix).
- `microservices/intelligence/contracts/provider-adapter-trait.md`.
- ADR-0296 (sidecar credential handle).
- ADR-0253 (HTTP/3 default).

## Wave 15 substance conversion — Anthropic provider adapter

### §A Problem

Anthropic is a primary text/vision counterpart, but product teams must not call it directly or safety, cost, and
audit controls fragment.
This IP closes the Anthropic adapter path under the canonical intelligence provider contract.

### §B Approach

Implement Messages API request/stream translation behind `ProviderAdapterPort`.
The adapter consumes `CredentialHandle`, receives already-authorized `DispatchRequest`, and emits canonical chunks,
usage, and refusal/error mapping.

### §C Deliverables

- `crates/oya-intelligence-providers-adapter-anthropic/src/lib.rs`
- `messages.rs`, `streaming.rs`, `tools.rs`, and `errors.rs`
- mock SSE tests and credential redaction tests

### §D Implementation

1. Convert prompt parts to Anthropic message blocks.
2. Convert tool definitions to Anthropic tool schema without changing oyatie tool ids.
3. Inject API key only through the sidecar-derived handle path.
4. Normalize message deltas into IP-016/IP-017 chunks.
5. Map provider safety output into `RefusalDecision` where applicable.
6. Emit provider/model/usage audit fields before terminal state.

### §E Acceptance

Adapter tests must prove ordered streaming, tool schema mapping, EU-region override where configured, and no logged
credential material.

### §F Evidence

Local anchors: `contracts/provider-adapter-trait.md`, `policy/provider-routing.cedar`,
`runbooks/provider-outage-anthropic.md`, ADR-0296, and ADR-0253.

### §G Counterparts

Anthropic API is the direct counterpart; oyatie matches Messages/streaming/tool-use access while adding provider
routing, Cedar policy, BYOK handle isolation, and audit sealing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-011-adapter-anthropic.md` matched `cost`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.

## Pod runtime tier (per ADR-0338)

- Authority: ADR-0338.
- `pod_runtime_tier`: `0`.
- Justification: tenant-customer code exists in this IP execution path; Kata Containers + Cloud Hypervisor are required.
- Surface evidence: `microservices/intelligence/IP-011-adapter-anthropic.md`, `microservices/intelligence/manifest.json`; trigger terms `sandbox`.
