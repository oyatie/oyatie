---
doc_class: SdkPlan
title: SDK + client surface plan
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-translate + gtm-developer-relations
related_adrs: [ADR-0056, ADR-0105, ADR-0131]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/contracts/openapi/translate.yaml
  - microservices/translate/contracts/openapi/translate-stream.yaml
  - microservices/translate/contracts/openapi/translate-files.yaml
  - microservices/translate/contracts/proto/translate.proto
  - microservices/translate/contracts/asyncapi/translate-events.yaml
review_cadence: per-release
doc_status: published
---

# SDK Plan — translate µservice

## Languages

| Language | Status M01 | Status M02 | Owner | Notes |
|---|---|---|---|---|
| Rust | Ship (canonical) | Ship | axis-translate | Generated from `oya-translate-router-sdk` crate |
| TypeScript / Node.js | Ship | Ship | axis-translate + gtm-dr | Generated from OpenAPI + AsyncAPI |
| Python | Scaffold | Ship | axis-translate + gtm-dr | openapi-python-client codegen |
| Go | Scaffold | Ship | gtm-dr | openapi-go-client + grpc-go from proto |
| Java/Kotlin | Tracked | Tracked | gtm-dr | Tracked; M03 |
| Swift / iOS | Tracked | Tracked | shorts µservice (iOS-bound consumer) | Tracked |
| Browser (TS / WASM) | Scaffold (TS) | Ship (TS + WASM) | axis-translate | Used by Workflow Studio + docs/sheets/slides UI |

## Per-Language Surface

### Rust (`oya-translate-router-sdk`)

```rust
use oya_translate_router_sdk::{TranslateClient, TranslateRequest, ContentClass, PackId};

let client = TranslateClient::new("https://translate.kr.oyatie.com/v1")
    .with_oidc_token(token)
    .with_tenant("acme-corp")
    .with_pack(PackId::Kr);

let req = TranslateRequest::builder()
    .source_lang("en")
    .target_lang("ko")
    .text("Hello, world!")
    .content_class(ContentClass::UiString)
    .build();

let res = client.translate(req).await?;
println!("{}", res.translated_text);
println!("engine={} cost_usd={} latency_ms={}", res.engine, res.cost_usd, res.latency_ms);
```

Async-first; tokio runtime.

Generated entity types from `oya-translate-router-api` crate (the canonical types live in `api`; sdk re-exports them).

### TypeScript / Node.js (`@oyatie/translate-client`)

```ts
import { TranslateClient } from '@oyatie/translate-client';

const client = new TranslateClient({
  baseUrl: 'https://translate.kr.oyatie.com/v1',
  oidcToken: token,
  tenantId: 'acme-corp',
  pack: 'kr',
});

const res = await client.translate({
  sourceLang: 'en',
  targetLang: 'ko',
  text: 'Hello, world!',
  contentClass: 'ui-string',
});

console.log(res.translatedText, res.engine, res.costUsd, res.latencyMs);
```

Surface mirrors Rust; identical naming up to language convention (camelCase JS).

Promise-based; ESM + CJS published.

### Python (`oyatie-translate-client`)

```python
from oyatie_translate_client import TranslateClient, ContentClass

client = TranslateClient(
    base_url="https://translate.kr.oyatie.com/v1",
    oidc_token=token,
    tenant_id="acme-corp",
    pack="kr",
)

res = client.translate(
    source_lang="en",
    target_lang="ko",
    text="Hello, world!",
    content_class=ContentClass.UI_STRING,
)

print(res.translated_text, res.engine, res.cost_usd, res.latency_ms)
```

Async (`AsyncTranslateClient`) + sync surfaces.

### Browser TS / WASM (`@oyatie/translate-browser`)

Embedded in Workflow Studio + docs/sheets/slides editor; uses OIDC bearer; WS for real-time stream.

## Capability Coverage per SDK

| Capability | Rust | TS | Python | Go | Browser |
|---|---|---|---|---|---|
| `translate` (single segment) | ✅ | ✅ | ✅ | M02 | ✅ |
| `translateBatch` (≤ 100 segments) | ✅ | ✅ | ✅ | M02 | ✅ |
| `detectLanguage` | ✅ | ✅ | ✅ | M02 | ✅ |
| `leverageQuery` (TM) | ✅ | ✅ | M02 | M02 | M02 |
| `qualityEstimate` | ✅ | ✅ | M02 | M02 | M02 |
| `bulkTranslate` (XLIFF/TMX/TBX) | ✅ | ✅ | ✅ | M02 | n/a (server-side) |
| `documentTranslate` (DOCX/PPTX/XLSX/PDF) | ✅ | ✅ | ✅ | M02 | n/a |
| `streamTranslate` (real-time caption WS) | ✅ | ✅ | M02 | M02 | ✅ |
| `termbase.import` (TBX) | ✅ | ✅ | M02 | M02 | n/a |
| `termbase.export` | ✅ | ✅ | M02 | M02 | n/a |
| `tm.export` (TMX) | ✅ | ✅ | M02 | M02 | n/a |
| `webhook.subscribe` | ✅ | ✅ | ✅ | M02 | n/a |

## Code-Generation Pipeline

| Source | Output | Tool |
|---|---|---|
| `contracts/openapi/translate.yaml` | TS client + Python client + Go client | openapi-generator-cli |
| `contracts/openapi/translate-stream.yaml` (WS spec) | TS WS client + Python | manual scaffold + openapi-generator |
| `contracts/openapi/translate-files.yaml` | TS bulk client | openapi-generator |
| `contracts/proto/translate.proto` | Rust grpc client + Go grpc client | tonic-build + protoc-gen-go-grpc |
| `contracts/asyncapi/translate-events.yaml` | TS event client + Python event client | asyncapi-cli |

CI lane `oya-translate-sdk-codegen` regenerates clients on every contract change; PR fails if generated artifacts diverge from committed.

## Versioning + Backward Compatibility

- API versioning: `v1` → `v2` requires migration guide + 18-month overlap per `lean-a10` lane (no silent regression).
- SDK versioning: semver; minor = additive only; major requires migration runbook.
- Generated SDK published per release with `release-pointer` from `release-pointers/translate/<env>.json`.

## Distribution

| Language | Registry |
|---|---|
| Rust | crates.io (`oyatie-translate`) + private mirror |
| TypeScript / Node.js | npm (`@oyatie/translate-client`) |
| Python | PyPI (`oyatie-translate-client`) |
| Go | go.oyatie.dev/translate-client (Athens proxy) |
| Browser WASM | npm (`@oyatie/translate-browser`) + jsdelivr CDN |

## Documentation Site

- `developer.oyatie.dev/translate` — generated from OpenAPI + AsyncAPI + per-language quickstart.
- `developer.oyatie.dev/translate/api` — interactive OpenAPI explorer (Swagger UI).
- `developer.oyatie.dev/translate/streaming` — WS quickstart + correction-replay semantics.
- `developer.oyatie.dev/translate/i18n` — XLIFF/TMX/TBX integration guides.
- `developer.oyatie.dev/translate/eu-ai-act` — Art. 50 disclosure consumption guide.

## Examples + Quickstarts

| Example | Languages | Story |
|---|---|---|
| Quickstart: single translate | Rust, TS, Python | 5 lines to first translation |
| Single-page i18n with ARB | TS browser | Flutter ARB tenant workflow |
| Subtitle-stream translate | TS browser | meet captions live |
| Document round-trip | Python | DOCX → XLIFF → translate → DOCX |
| TM-leverage opt-in | Rust | Per-tenant TM accumulation |
| Termbase enforcement | TS | TBX upload + verify enforcement |

## Verification

- `cargo run -p oya-dev-cli -- gate validate sdk-codegen --microservice translate` exits 0.
- Quickstart examples in `examples/` directory tested against staging per release.
- Per-SDK npm/PyPI/crates.io publish gated on lane green.

## References

- ADR-0056 — Rust BNF (sdk layer per ADR-0105).
- ADR-0131 — per-µservice flat layout (sdk lives under `microservices/translate/src/crates/oya-translate-router-sdk`).
- OpenAPI Generator — `openapi-generator.tech/`.
- AsyncAPI Generator — `www.asyncapi.com/tools/generator`.
- tonic-build — `docs.rs/tonic-build`.
- protoc-gen-go-grpc — gRPC Go.
