---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-025-cn-pipl-pack-adapter
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0251, ADR-0255, ADR-0244]
---

# IP-025: CN-PIPL pack adapter — Alibaba Qwen + Tencent Hunyuan only

## Intent

When `pack-cn` is active for a tenant, the dispatch router MUST route exclusively to
CN-resident providers (Alibaba Qwen / Tencent Hunyuan). Refuse all outbound dispatches to
US/EU providers. Per ADR-0251 CN-PIPL pack + CN Generative AI Provisions 2023.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-providers-adapter-alibaba-qwen/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-alibaba-qwen/src/lib.rs` | create |
| `crates/oya-intelligence-providers-adapter-tencent-hunyuan/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-tencent-hunyuan/src/lib.rs` | create |
| `crates/oya-intelligence-model-routing-kernel/src/cn_pack_gate.rs` | create |

## CN-pack routing invariants

1. `provider_routing.cedar` FORBID: any non-CN provider when `pack_cn == true`.
2. Alibaba Qwen adapter: DashScope API; CN datacenter only; provider-BYOK via OpenBao sidecar.
3. Tencent Hunyuan adapter: Hunyuan API; CN datacenter only; provider-BYOK via OpenBao sidecar.
4. No cross-border data transfer: prompt + completion never leave CN datacenter.
5. CN Generative AI Provisions 2023: algorithm filing requirement logged per dispatch.

## Key implementation notes

- Router checks `pack_cn` flag from tenant context before provider selection.
- Cedar gate: `policy/provider-routing.cedar` has FORBID for `pack_cn + non_cn_provider`.
- Audit: every CN-pack dispatch emits `CnPiplDispatchRecord` with provider + datacenter-region.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-kernel -- cn_pack_gate
buck2 build //:quality-lane-registry-authority-check # lane=cn-pipl-provider-isolation --microservice intelligence
```

## References

- `microservices/intelligence/compliance.md §pack-overlay-roster` (CN PIPL row).
- ADR-0251 (compliance packs — CN-PIPL-2021).
- ADR-0255 §9 (CN-PIPL posture).
- `microservices/intelligence/policy/provider-routing.cedar`.

## Wave 15 substance conversion — CN PIPL provider isolation

### §A Problem

`pack-cn` is declared in the service architecture as a residency and provider-isolation posture, but a generic
provider catalog can accidentally route Chinese tenant prompts to US/EU providers.
This IP closes that concrete cross-border transfer gap for the intelligence dispatch path.
It is not a generic localization task: it is the hard routing invariant that Alibaba Qwen and Tencent Hunyuan are
the only permitted provider families for CN-resident dispatch.

### §B Approach

Make `pack-cn` a first-class router and Cedar input.
The router checks tenant pack before normal provider scoring, then `policy/provider-routing.cedar` supplies the
defence-in-depth forbid for any non-CN provider.
The adapter crates expose the same `ProviderAdapterPort` as OpenAI/Anthropic/Bedrock so the rest of the substrate
still receives `RoutingDecision`, `RefusalDecision`, `CallRecord`, and stream chunks in canonical form.

### §C Deliverables

- Create Alibaba Qwen and Tencent Hunyuan adapter crates listed in the original target table.
- Add `crates/oya-intelligence-model-routing-kernel/src/cn_pack_gate.rs`.
- Extend provider catalog data with `cn_resident=true` entries.
- Add Cedar tests for `pack_cn + non_cn_provider` forbid in `policy/provider-routing.cedar`.
- Add audit event fixture `CnPiplDispatchRecord` without embedding prompt text.

### §D Implementation

1. Parse the tenant pack into `Pack::CnPipl` before provider selection.
2. Filter catalog entries to Qwen/Hunyuan with CN datacenter regions before normal scoring.
3. Refuse platform-default credentials unless the pack policy explicitly allows them.
4. Map Qwen and Hunyuan response chunks into the same dispatch stream abstraction used by IP-016/IP-017.
5. Emit region, provider, datacenter class, and algorithm-filing evidence to audit tap.
6. Return `DataResidencyViolation` for any fallback that would leave CN.
7. Add a negative test proving Bedrock/OpenAI/Anthropic cannot be selected for `pack-cn`.

### §E Acceptance

The `cn-pipl-provider-isolation` gate must prove allow for Qwen/Hunyuan and deny for OpenAI, Anthropic, Google,
Bedrock, Azure OpenAI, and OpenRouter.
Evidence also needs a passing Cedar fixture for `policy/provider-routing.cedar` and a local compliance reference
from `compliance.md`.

### §F Evidence

Local anchors: `manifest.json` provider roster, `competitor-parity-matrix.md` provider matrix,
`policy/provider-routing.cedar`, `policy/byok-gating.cedar`, and `runbooks/provider-outage-google.md`.
Doctrine anchors: ADR-0251, ADR-0255, ADR-0244, ADR-0263.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| Alibaba Qwen / DashScope | CN-resident model API | Add canonical adapter behind oyatie Cedar/audit semantics |
| Tencent Hunyuan | CN-resident model API | Add compliant fallback without cross-border dispatch |
| Google Vertex AI / OpenAI | Non-CN direct provider APIs | Explicitly denied for CN pack instead of treated as ordinary fallback |
