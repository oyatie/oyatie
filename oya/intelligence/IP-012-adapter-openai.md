---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-012-adapter-openai
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0296, ADR-0253]
---

# IP-012: Provider adapter — OpenAI

## Intent

`oya-intelligence-providers-adapter-openai`: implements `ProviderAdapterPort` for OpenAI API
(GPT-5 / GPT-4o / o-series). Text + vision + audio modalities. SSE streaming. provider-BYOK.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-providers-adapter-openai/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-openai/src/lib.rs` | create |
| `crates/oya-intelligence-providers-adapter-openai/src/client.rs` | create |
| `crates/oya-intelligence-providers-adapter-openai/src/streaming.rs` | create |
| `crates/oya-intelligence-providers-adapter-openai/src/modalities.rs` | create |

## Code shape

```rust
pub struct OpenAiAdapter {
    http_client: reqwest::Client,
    base_url: Url,
    audio_base_url: Url,
}

impl ProviderAdapterPort for OpenAiAdapter {
    async fn invoke(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<ProviderResponse, ProviderError>;
    async fn invoke_stream(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<impl Stream<Item = Result<ProviderChunk, ProviderError>>, ProviderError>;
}
```

## Key implementation notes

- Audio modality routed to `/v1/audio/speech` or `/v1/audio/transcriptions`.
- Structured output (JSON mode) mapped from `DispatchEnvelope.output_schema`.
- EU-residency: OpenAI EU endpoint when `pack-eu`.
- Realtime API (WebSocket) for audio streaming — see IP-018.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-providers-adapter-openai
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-byok --provider openai
```

## References

- `microservices/intelligence/ARCHITECTURE.md §6`.
- `microservices/intelligence/contracts/provider-adapter-trait.md`.
- ADR-0296, ADR-0253.

## Wave 15 substance conversion — OpenAI provider adapter

### §A Problem

OpenAI is a primary direct-provider counterpart in the parity matrix, but direct calls from products would bypass
tenant policy, provider-BYOK, refusal taxonomy, and audit evidence.
This IP closes that direct-provider gap with a canonical adapter behind `ProviderAdapterPort`.

### §B Approach

Translate `DispatchRequest` into OpenAI Responses/Reatime-compatible calls while keeping credential resolution,
routing, guardrails, and audit-tap outside provider-specific code.
The adapter implements text, vision, audio/realtime where the provider catalog permits it.

### §C Deliverables

- `crates/oya-intelligence-providers-adapter-openai/src/lib.rs`
- `responses.rs`, `realtime.rs`, `streaming.rs`, and `errors.rs`
- fixtures for tool calls, structured output, refusal, and rate-limit handling

### §D Implementation

1. Resolve `CredentialHandle` through the sidecar path from IP-002/IP-023.
2. Convert prompt parts and tool definitions to OpenAI request JSON.
3. Normalize Responses API deltas into IP-016/IP-017 chunks.
4. Map provider refusal/rate-limit errors to `DispatchError` variants.
5. Apply `policy/provider-routing.cedar` before any provider request.
6. Emit provider, model, usage, and terminal state into audit tap.

### §E Acceptance

Acceptance includes nextest for the adapter, one mocked tool-call dispatch, one streamed response, one rate-limit
fallback, and one BYOK credential-deny test.

### §F Evidence

Local anchors: `contracts/provider-adapter-trait.md`, `policy/byok-gating.cedar`, `policy/provider-routing.cedar`,
and `runbooks/provider-outage-openai.md`.

### §G Counterparts

OpenAI Platform is the counterpart; oyatie matches Responses/Reatime ergonomics while adding central Cedar, tenant
pack routing, cost attribution, and Ed25519-sealed audit evidence.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-012-adapter-openai.md` matched `attribution, cost`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
