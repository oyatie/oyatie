---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-014-adapter-bedrock
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0296, ADR-0253]
---

# IP-014: Provider adapter — AWS Bedrock

## Intent

`oya-intelligence-providers-adapter-bedrock`: implements `ProviderAdapterPort` for AWS Bedrock.
FedRAMP-eligible (GovCloud). Claude / Cohere / Mistral / Titan. provider-BYOK via IAM role assumption
through sidecar.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-providers-adapter-bedrock/Cargo.toml` | create |
| `crates/oya-intelligence-providers-adapter-bedrock/src/lib.rs` | create |
| `crates/oya-intelligence-providers-adapter-bedrock/src/client.rs` | create |
| `crates/oya-intelligence-providers-adapter-bedrock/src/streaming.rs` | create |

## Key implementation notes

- AWS SigV4 signing via short-lived credentials from OpenBao sidecar (IAM role → temp credentials).
- FedRAMP: route to `bedrock.us-gov-west-1.amazonaws.com` when `pack-us-federal`.
- HIPAA BAA: Bedrock is BAA-eligible on appropriate account — Cedar gate verifies before routing.
- Response stream: Bedrock uses `application/vnd.amazon.eventstream`; adapter normalises to ProviderChunk.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-providers-adapter-bedrock
buck2 build //:quality-lane-registry-authority-check # lane=provider-adapter-fedramp --provider bedrock
```

## References

- `microservices/intelligence/ARCHITECTURE.md §6`.
- ADR-0255, ADR-0296.

## Wave 15 substance conversion — Bedrock adapter

### §A Problem

This IP closes the enterprise model-gateway gap for regulated tenants that cannot use direct OpenAI,
Anthropic, or Google endpoints.
`manifest.json` already lists `oya-intelligence-providers-adapter-bedrock`, but without this adapter the
provider catalog cannot honour `pack-us-federal` or AWS-resident tenant routing claims.
The gap is not "add another provider"; it is IAM-shaped Bedrock dispatch with the same `DispatchEnvelope`,
`SecretReference`, refusal, attribution, and audit-tap semantics used by the rest of Layer-A.

### §B Approach

Implement a Bedrock `ProviderAdapterPort` that translates the existing provider-adapter trait contract in
`contracts/provider-adapter-trait.md` into Bedrock Converse/InvokeModel calls.
The adapter must consume `CredentialHandle` from the OpenBao sidecar, not AWS keys in process memory.
Model selection is driven by `policy/provider-routing.cedar` and `RoutingDecision`, with Bedrock allowed
only where region, BAA/FedRAMP posture, modality, and pack rules match.

### §C Deliverables

- Create `crates/oya-intelligence-providers-adapter-bedrock/src/lib.rs` with the trait impl boundary.
- Create `crates/oya-intelligence-providers-adapter-bedrock/src/converse.rs` for text/tool calls.
- Create `crates/oya-intelligence-providers-adapter-bedrock/src/credentials.rs` for handle-to-signed-request assembly.
- Extend `microservices/intelligence/policy/provider-routing.cedar` coverage tests for Bedrock pack allow/deny.
- Add provider-specific fixtures under the crate tests without storing customer prompts or AWS secrets.

### §D Implementation

1. Map `DispatchRequest.modality` to Bedrock supported request shapes; reject unsupported video early.
2. Bind `RoutingDecision.region` to the AWS regional endpoint and refuse cross-region fallback for residency packs.
3. Convert tool definitions into the Bedrock tool-use JSON shape while preserving oyatie tool ids.
4. Stream partial output into the same chunk type used by IP-016 and IP-017.
5. Emit `CallRecord.provider = "bedrock"` before the external call and seal it through IP-022.
6. On AWS throttling, return `ProviderSaturated` so the router can try the next compliant provider.
7. Exercise `byok-gating.cedar` so tenant Bedrock credentials are mandatory where `byok_required_by_pack` applies.

### §E Acceptance

Acceptance requires the existing `cargo nextest run -p oya-intelligence-providers-adapter-bedrock` gate plus
provider-routing fixtures proving `pack-us-federal` can select Bedrock and `pack-cn` cannot.
Audit evidence must include `audit-emission-success.openslo.yaml` and the `provider-outage-*` runbook mapping for
fallback behaviour.

### §F Evidence

Local anchors: `manifest.json` providers BC, `contracts/provider-adapter-trait.md`,
`policy/provider-routing.cedar`, `policy/byok-gating.cedar`, and `runbooks/provider-rate-limit-saturation.md`.
Doctrine anchors: ADR-0255 provider catalog, ADR-0296 sidecar credential handle, ADR-0263 audit tap.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| AWS Bedrock | Multi-model enterprise gateway under AWS IAM | Match gateway dispatch while adding Cedar refusal, provider-BYOK, and sealed audit records |
| Azure OpenAI | Enterprise identity and regional routing | Keep equivalent enterprise controls without coupling the substrate to one cloud |
| OpenRouter | Unified provider fanout | Preserve fanout ergonomics but add pack-aware residency and audit atomicity |

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-014-adapter-bedrock.md` matched `attribution, emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
