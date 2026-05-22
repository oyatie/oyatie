---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-018-multi-modal-audio-video
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0105]
---

# IP-018: Multi-modal — audio + video dispatch

## Intent

Extend `DispatchEnvelope` to support audio and video modalities. Wire audio routing to
OpenAI Realtime / Google Live / Anthropic (when available). Wire video routing to Gemini
Pro video understanding. Ensure guardrails apply to non-text modalities.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-model-routing-domain/src/modality.rs` | extend |
| `crates/oya-intelligence-guardrails-kernel/src/multimodal_classifier.rs` | create |
| `crates/oya-intelligence-providers-adapter-openai/src/realtime.rs` | create |
| `crates/oya-intelligence-providers-adapter-google/src/live_api.rs` | create |

## Modality routing rules

| Modality | Preferred provider | Fallback | Guardrail hook |
|---|---|---|---|
| audio | OpenAI Realtime API | Gemini Live API | MultimodalClassifier (pre-call) |
| video | Gemini 2.5 Pro | Vertex AI | MultimodalClassifier (pre-call) |
| image | Anthropic Vision / OpenAI Vision | Gemini Vision | MultimodalClassifier (pre-call) |
| multi | Gemini 2.5 Pro (native multimodal) | — | MultimodalClassifier (pre-call) |

## Key implementation notes

- Audio guardrail: content classification runs on transcript (ASR → text → refusal-baseline).
- Video guardrail: frame sampling (1 fps) → vision classifier → refusal decision.
- On-device audio (Apple Foundation Models): bypasses network; brand-ux-surface SDK handles locally; audit-tap still emits.
- CSAM hash-check: before any image/video frame reaches provider, PhotoDNA hash check fires.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-domain -- modality
cargo run -p oya-dev-cli -- gate validate multimodal-guardrail-coverage --microservice intelligence
```

## References

- `microservices/intelligence/ARCHITECTURE.md §6` (provider matrix — modalities column).
- `microservices/intelligence/IP-016-streaming-sse-transport.md`.
- `microservices/intelligence/IP-017-streaming-websocket-transport.md`.

## Wave 15 substance conversion — multimodal dispatch

### §A Problem

The architecture and parity matrix claim audio, image, video, and multi-modal routing, but the existing domain
IP only establishes text-oriented dispatch primitives.
Without this slice, `DispatchEnvelope.modality` can name audio/video while the guardrail stack, provider catalog,
stream transports, and audit tap do not prove how non-text bytes are classified, routed, or refused.
That gap is acute for `MINOR_TARGETED`, healthcare, and emergency-services paths where unsafe image or audio
classification cannot be delegated to provider defaults.

### §B Approach

Extend the model-routing domain around modality-specific payload descriptors and run a pre-provider classifier
in `oya-intelligence-guardrails-kernel`.
Audio is normalized through transcript metadata before refusal; video is sampled into bounded frame evidence;
image and multi-modal requests share the same refusal and audit path as text.
The dispatch path stays provider-neutral: OpenAI Realtime, Google Live/Vertex, Anthropic vision, and Bedrock
remain adapter choices under `policy/provider-routing.cedar`.

### §C Deliverables

- Extend `crates/oya-intelligence-model-routing-domain/src/modality.rs`.
- Add `crates/oya-intelligence-guardrails-kernel/src/multimodal_classifier.rs`.
- Add OpenAI realtime and Google live adapter seams listed in this IP.
- Bind modality fields to `contracts/openapi/intelligence-v1.yaml` and `contracts/proto/intelligence-v1.proto`.
- Add eval rows under `data/eval/intelligence/` for sampled-frame and transcript refusal cases.

### §D Implementation

1. Define explicit `AudioInput`, `VideoInput`, `ImageInput`, and `MultiModalInput` structures with size limits.
2. Reject unsupported media combinations before credential resolution to avoid provider-side leakage.
3. Run transcript/frame classification before provider selection when `audience_type` is minor, emergency, or regulated.
4. Pass classifier outputs into `RefusalDecision` so policy/refusal-baseline and EU AI Act records share taxonomy.
5. Route only to providers whose catalog entry supports the requested modality and pack.
6. Stream audio/video partials through IP-016/IP-017 chunk ordering semantics.
7. Emit audit evidence without raw media payloads; store hashes, modality, classifier result, and provider decision.

### §E Acceptance

Required checks are the modality nextest filter plus `multimodal-guardrail-coverage`.
Additional proof: one clean audio dispatch, one rejected minor-targeted unsafe image, one `pack-cn` video routing refusal
against non-CN providers, and one audit record validated against `audit-emission-success.openslo.yaml`.

### §F Evidence

Use `ARCHITECTURE.md` §2 two-layer model, `competitor-parity-matrix.md` modality rows, `policy/refusal-baseline.cedar`,
`policy/critical-path-emergency-services.cedar`, and `slos/streaming-throughput.openslo.yaml`.
Doctrine anchors are ADR-0255, ADR-0244, ADR-0292, and ADR-0263.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| OpenAI Realtime API | Low-latency speech interaction | Add equivalent realtime routing behind Cedar, tenant policy, and audit tap |
| Google Gemini Live / Vertex AI | Audio/video multimodal provider path | Preserve provider breadth while enforcing pack residency and modality guardrails |
| Anthropic vision | Image understanding in direct API | Route image tasks without bypassing refusal baseline or citation attribution |

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-018-multi-modal-audio-video.md` matched `attribution, emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
