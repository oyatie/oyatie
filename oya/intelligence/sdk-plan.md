---
doc_class: SDKPlan
template_id: TPL-SDK-PLAN
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + dx-developer-experience
related_adrs: [ADR-0255, ADR-0255-amendment-library-first]
doc_status: published
---

# SDK Plan — intelligence µservice

## Purpose

Define the SDK surface (Rust, TypeScript, Python, Swift, Kotlin, Dart, Go, CLI) for the
intelligence µservice, the canonical envelope shape, the per-SDK packaging plan, the version-pin
strategy, and the publication cadence.

## Library-first invariant (ADR-0255 amendment)

The canonical dispatch path is **in-process** via the SDK. The REST + gRPC network surfaces are
the opt-in fallback for cross-language callers and the consumer brand UX surface running in a
browser. Every SDK exposes both the in-process dispatch and the network-fallback dispatch under
the same `dispatch()` shape.

## Per-language SDK matrix

| SDK | Package name | Distribution | First-class | Multi-modal | Streaming | Audience |
|---|---|---|---|---|---|---|
| Rust | `oya-intelligence-dispatch-sdk-rs` | crates.io | yes | yes | yes (SSE + WebSocket) | every Rust workload + Foundry agents |
| TypeScript | `@oyatie/intelligence-dispatch-sdk-ts` | npm | yes | yes | yes (Fetch + EventSource + WebSocket) | Forge developer console + Application Shell + tenant Node backends |
| Python | `oyatie-intelligence-dispatch-sdk` | PyPI | yes | yes | yes (httpx + sse-starlette + websockets) | tenant data-science + AI workloads |
| Swift | `OyatieIntelligenceDispatchSDK` | SwiftPM | yes | yes | yes (URLSession + EventSource + WebSocket) | iOS / macOS native callers |
| Kotlin | `dev.oyatie:intelligence-dispatch-sdk-kotlin` | Maven Central | yes | yes | yes (OkHttp + EventSource + WebSocket) | Android + KMP callers |
| Dart | `oyatie_intelligence_dispatch_sdk` | pub.dev | yes | yes | yes | Flutter callers |
| Go | `github.com/oyatie/intelligence-dispatch-sdk-go` | Go modules | yes | yes | yes (net/http + sse + gorilla/websocket) | Go callers |
| CLI | `oya-intelligence` (binary) | brew + cargo install + apt + docker | yes | text only | yes | operator + developer console |

## Canonical envelope (cross-language)

All SDKs marshall to/from this envelope shape:

```yaml
DispatchEnvelope:
  envelope_version: "1.0"
  envelope_id: ulid                      # ULID per call
  tenant_id: "tenant:<hashed-id>"
  audience_tag: consumer | developer | internal-foundry
  purpose: consumer-chat | developer-codegen | foundry-planning | ... | <tenant-custom>
  modality: text | image | audio | video | multi
  provider_hint: anthropic | openai | google | bedrock | ... | auto
  model_hint: claude-opus-4-7 | gpt-5 | gemini-2.5-pro | ...
  prompt:
    parts:
      - role: system | user | assistant | tool
        kind: text | image_url | image_base64 | audio_blob | video_blob | tool_response
        content: string | bytes | ref
        untrusted_content: bool          # caller-flagged untrusted content
  tools:                                 # optional
    - name: string
      description: string
      schema: jsonschema
  config:
    temperature: number | null
    top_p: number | null
    max_tokens: int | null
    stop: string[] | null
    stream: bool
    json_mode: bool
    seed: int | null
  secret_reference:                      # provider-BYOK; resolved by credential-resolver
    kind: openbao_path | platform_default
    value: "${openbao:secret/<tenant>/<provider>}" | null
  pack: pack-kr | pack-eu | pack-us | ... | null  # inferred from tenant if null
  consent_grant_id: ulid | null          # for sensitive-data dispatch
  rfia_id: ulid | null                   # for Annex III deployment

DispatchOutcome:
  envelope_id: ulid
  status: streaming | completed | refused | failed
  refusal:
    decision_id: ulid
    reason: csam | violence | self_harm | extremism | coppa | eu_ai_act_annex_iii_cat_<n>
          | pci_scope_refused | data_residency_violation | cost_cap_exceeded
          | credential_unavailable | provider_saturated | consent_missing
    pack_overlay_applied: pack-kr | pack-eu | ...
  output:                               # when status == completed
    parts: [ ... same shape as prompt parts ... ]
    citations: [{ source_uri, span, confidence }, ...]
    eval_score: number | null
  routing_decision:
    provider: anthropic | openai | ...
    model: claude-opus-4-7 | ...
    latency_first_token_ms: int
    streaming_tokens_per_sec: number
  audit_tap_record_id: ulid
  cost:
    input_tokens: int
    output_tokens: int
    total_cost_usd: decimal
    cost_owner: tenant_byok | platform_default
```

## Versioning strategy

- SemVer 2.0; major-version-pinned per `Cargo.toml` / `package.json` / `Package.swift` / etc.
- Network contracts pinned to OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 in
  `microservices/intelligence/contracts/`.
- Breaking changes go through `oya-lean-a10-no-silent-regression` lane (see
  `feedback_no_silent_regression.md`); requires ADR + version bump + sunset notice.

## Publication cadence

| Cadence | Channel | SDK |
|---|---|---|
| Patch release | `<sdk>:<version>` to crates.io / npm / PyPI / Maven / SwiftPM / pub.dev / Go modules / Homebrew | all |
| Minor release | as above + Storybook + Swift Previews + Android Previews + DartPad | all |
| Major release | as above + migration guide + dual-publish (N + N-1 lanes for 6 months) | all |

## Per-SDK example (Rust)

```rust
use oya_intelligence_dispatch_sdk::{DispatchEnvelope, Audience, Modality, Provider, Dispatch};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dispatch = Dispatch::in_process()?;        // library-first
    let envelope = DispatchEnvelope::builder()
        .tenant_id("tenant:abcd1234abcd1234")
        .audience(Audience::Developer)
        .purpose("developer-codegen")
        .modality(Modality::Text)
        .provider_hint(Provider::Anthropic)
        .model_hint("claude-opus-4-7")
        .prompt_text("Write a Rust function that returns the n-th Fibonacci number.")
        .stream(true)
        .build()?;

    let mut stream = dispatch.issue_stream(envelope).await?;
    while let Some(chunk) = stream.next().await {
        print!("{}", chunk?.text);
    }
    Ok(())
}
```

## Per-SDK example (TypeScript)

```ts
import { dispatch, DispatchEnvelope } from "@oyatie/intelligence-dispatch-sdk-ts";

const envelope: DispatchEnvelope = {
  envelope_version: "1.0",
  tenant_id: "tenant:abcd1234abcd1234",
  audience_tag: "consumer",
  purpose: "consumer-chat",
  modality: "text",
  provider_hint: "auto",
  prompt: { parts: [{ role: "user", kind: "text", content: "Hello!" }] },
  config: { stream: true },
};

for await (const chunk of dispatch.stream(envelope)) {
  process.stdout.write(chunk.text);
}
```

## Per-SDK example (Python)

```python
from oyatie_intelligence_dispatch_sdk import Dispatch, DispatchEnvelope

dispatch = Dispatch.in_process()
envelope = DispatchEnvelope(
    tenant_id="tenant:abcd1234abcd1234",
    audience_tag="developer",
    purpose="developer-codegen",
    modality="text",
    provider_hint="anthropic",
    model_hint="claude-opus-4-7",
    prompt=[{"role": "user", "kind": "text", "content": "Explain Big-O."}],
    config={"stream": True},
)
for chunk in dispatch.stream(envelope):
    print(chunk.text, end="")
```

## Per-SDK example (Swift)

```swift
import OyatieIntelligenceDispatchSDK

let dispatch = try Dispatch.inProcess()
let envelope = DispatchEnvelope(
    tenantId: "tenant:abcd1234abcd1234",
    audienceTag: .consumer,
    purpose: "consumer-chat",
    modality: .text,
    providerHint: .auto,
    prompt: [.text(role: .user, content: "Hello from iOS!")],
    config: .init(stream: true)
)
for try await chunk in dispatch.stream(envelope) {
    print(chunk.text, terminator: "")
}
```

## Per-SDK example (CLI)

```sh
oya-intelligence dispatch \
  --tenant tenant:abcd1234abcd1234 \
  --audience developer \
  --purpose developer-codegen \
  --provider anthropic \
  --model claude-opus-4-7 \
  --prompt "Write a Rust function that returns the n-th Fibonacci number." \
  --stream
```

## Quality gates

| Gate | Threshold |
|---|---|
| Unit-test coverage | ≥ 90 % branch |
| Integration-test (against in-cluster intelligence) | ≥ 95 % critical-path |
| E2E (against staging) | ≥ 90 % flows |
| Bundle size (TS) | ≤ 50 kB minified+gzipped (core); modality plugins lazy-loaded |
| Cold-start (Python) | ≤ 100 ms import time |
| API docs | rust-doc / typedoc / pdoc / SwiftDocC / Dokka coverage 100 % public |

## References

- ADR-0255, ADR-0255 amendment.
- `microservices/intelligence/PRD.md`.
- `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`.
- `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`.
- `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
