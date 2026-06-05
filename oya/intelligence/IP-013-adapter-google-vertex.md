---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-013-adapter-google-vertex
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0296, ADR-0253]
---

# IP-013: Provider adapter — Google Vertex AI + AI Studio

## Intent

`oya-intelligence-providers-adapter-google`: implements `ProviderAdapterPort` for Google AI Studio
(Gemini 2.5 Pro / Flash) and Vertex AI. Text + vision + audio + video modalities. SSE streaming.
FedRAMP-eligible via Vertex AI GovCloud. EU-pinned routing.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-providers-adapter-google/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-google/src/lib.rs` | create |
| `crates/oya-intelligence-providers-adapter-google/src/client.rs` | create |
| `crates/oya-intelligence-providers-adapter-google/src/streaming.rs` | create |
| `crates/oya-intelligence-providers-adapter-google/src/video.rs` | create |

## Code shape

```rust
pub struct GoogleVertexAdapter {
    http_client: reqwest::Client,
    project_id: String,
    location: String,   // us-central1 | europe-west4 | asia-northeast3 (KR)
    use_gov_cloud: bool,
}

impl ProviderAdapterPort for GoogleVertexAdapter {
    async fn invoke(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<ProviderResponse, ProviderError>;
    async fn invoke_stream(&self, req: ProviderRequest, handle: CredentialHandle)
        -> Result<impl Stream<Item = Result<ProviderChunk, ProviderError>>, ProviderError>;
}
```

## Key implementation notes

- Video modality via Gemini's `inlineData` or Cloud Storage URI.
- FedRAMP: `use_gov_cloud = true` when `pack-us-federal`; routes to Vertex AI GovCloud.
- KR pack: `location = asia-northeast3` for KR-resident routing.
- OAuth2 service-account credential via sidecar (ADR-0296); never in adapter memory.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-providers-adapter-google
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-byok --provider google
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-video-modality --provider google
```

## References

- `microservices/intelligence/ARCHITECTURE.md §6`.
- ADR-0255, ADR-0296, ADR-0253.

## Wave 15 substance conversion — Google Vertex adapter

### §A Problem

Google Gemini/Vertex is needed for enterprise multimodal parity, but a direct Vertex client would bypass oyatie
router, BYOK semantics, and audit tap.
This IP closes the Google-provider path using the same adapter contract as OpenAI, Anthropic, and Bedrock.

### §B Approach

Implement Vertex/Gemini request assembly in a provider adapter that consumes `RoutingDecision`, `CredentialHandle`,
and canonical dispatch payloads.
Vertex grounding and video support are exposed only when provider catalog and pack policy allow them.

### §C Deliverables

- `crates/oya-intelligence-providers-adapter-google/src/lib.rs`
- `vertex.rs`, `gemini_live.rs`, `streaming.rs`, and `errors.rs`
- routing tests for Gemini text, image, audio, and video capability flags

### §D Implementation

1. Bind region from `RoutingDecision` to Vertex endpoint selection.
2. Convert prompt/media parts into Gemini request payloads.
3. Normalize Gemini stream deltas into canonical `DispatchChunk`.
4. Preserve grounding/citation data for IP-006 attribution.
5. Map quota and safety feedback to internal errors/refusals.
6. Emit provider/model/region audit evidence before terminal response.

### §E Acceptance

Adapter nextest must include multimodal fixtures, a quota fallback, one denied pack route, and one attribution
round-trip into `AttributionGraph`.

### §F Evidence

Local anchors: `contracts/provider-adapter-trait.md`, `policy/provider-routing.cedar`,
`capabilities/attribution.yaml`, and `runbooks/provider-outage-google.md`.

### §G Counterparts

Google AI Studio and Vertex AI lead in Gemini multimodal/grounding; oyatie closes parity while preserving central
Cedar routing, residency packs, and sealed audit tap semantics.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-013-adapter-google-vertex.md` matched `attribution`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
