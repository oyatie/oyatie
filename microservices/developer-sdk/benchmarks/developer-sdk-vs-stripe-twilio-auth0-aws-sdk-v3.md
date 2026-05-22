# `developer-sdk` µservice — Benchmark vs Stripe SDKs, Twilio SDKs, Auth0 SDKs, AWS SDK v3

> Measured 2026-04-26 to 2026-05-15. Four reference SDKs are widely seen as state-of-the-art for B2B developer APIs (Stripe, Twilio,
> Auth0) plus the AWS SDK v3 as the high-fanout incumbent. Comparison surfaces are: cold-import time, first-call latency, retry
> ergonomics, type-safety, telemetry surface, language matrix, and version-deprecation discipline.

## Language matrix coverage

| Surface | Official languages | Generated-from-spec | Hand-tuned ergonomics | LTS policy published |
| --- | --- | --- | --- | --- |
| `developer-sdk` (tenant_class paid) | 13 (Rust/TS/Py/Go/Ruby/Java/Kotlin/Swift/ObjC/C#/C++/Dart/PHP) | yes (OpenAPI 3.2.0 + protobuf) | yes (custom prelude per lang) | ✅ 24 mo tenant_class paid / 36 mo compliance_pack-bound paid |
| `developer-sdk` (compliance_pack-bound paid) | adds 9 (Elixir/Erlang/Clojure/Scala/Haskell/OCaml/Zig/Crystal/Nim) | yes | partial (Rust + Go fully tuned; others template) | ✅ 36 mo |
| Stripe | 7 (Ruby/Node/Python/PHP/Java/Go/.NET) | hybrid (spec-augmented) | yes (hand-tuned) | ✅ 12 mo deprecation |
| Twilio | 9 (Node/Python/PHP/Ruby/Java/C#/Go/Swift/Kotlin) | yes (legacy + new versions both shipped) | partial | partial |
| Auth0 | 7 (Node/Python/Java/.NET/Go/PHP/Ruby) | partial (spec-augmented) | yes | ✅ 18 mo |
| AWS SDK v3 | 12 (JS/Python/Java/Go/.NET/Ruby/PHP/Rust/Kotlin/Swift/C++/CLI) | yes (Smithy IDL) | yes (per-service customization) | ✅ Smithy-driven; varies per service |

## Cold-import time (process start → first import finished)

Workload: cold start + import the SDK + construct a client + immediately exit. 100 trials per surface. Numbers are p50/p99 in ms.

| Surface | Rust (cargo run --release) | TypeScript (node 22) | Python 3.13 | Go 1.23 |
| --- | --- | --- | --- | --- |
| `developer-sdk` | 4 / 9 | 38 / 71 | 72 / 140 | 11 / 22 |
| Stripe | n/a | 130 / 240 | 180 / 340 | 47 / 92 |
| Twilio | n/a | 250 / 410 | 290 / 510 | 78 / 150 |
| Auth0 | n/a | 110 / 195 | 165 / 305 | 62 / 120 |
| AWS SDK v3 (single client) | 12 / 28 | 195 / 360 | 280 / 510 | 28 / 58 |

`developer-sdk` cold-imports faster because the generated code path is leaner (no dynamic-config reflection like AWS SDK v3,
no chained polyfills like Stripe Node).

## First-call latency (warm cache, 1 KiB request, 1 KiB response, single region)

| Surface | Rust p95 | TS p95 | Python p95 | Go p95 |
| --- | --- | --- | --- | --- |
| `developer-sdk` | **22 ms** | **31 ms** | **38 ms** | **24 ms** |
| Stripe | n/a | 88 ms | 102 ms | 71 ms |
| Twilio | n/a | 96 ms | 121 ms | 84 ms |
| Auth0 | n/a | 75 ms | 89 ms | 64 ms |
| AWS SDK v3 (latency-tuned service e.g. DDB GetItem) | 18 ms | 42 ms | 51 ms | 28 ms |

AWS SDK v3 wins on raw Rust + Go because DDB is a hyperscaler-shaped point-API; `developer-sdk` wins across the broader REST surface
because of HTTP/3 + warm-pool effects (vendor SDKs are HTTP/2 only).

## Retry ergonomics

| Surface | Default policy | Jitter | Idempotency-Key handling | Custom policy hook |
| --- | --- | --- | --- | --- |
| `developer-sdk` | exp backoff 3 attempts | full | auto-set on POST when `x-oya-idempotent: safe` | `RetryPolicy::custom()` |
| Stripe | exp backoff 2 attempts | full | requires user-set header | yes |
| Twilio | exp backoff 5 attempts | partial | n/a | yes |
| Auth0 | none (manual retry) | n/a | n/a | yes via interceptor |
| AWS SDK v3 | adaptive (Standard / Adaptive / Legacy) | partial | per-service | yes |

## Telemetry surface

| Surface | OTLP traces | OTLP metrics | OTLP logs | Cardinality control | Per-tenant collector | Permit-denied counter |
| --- | --- | --- | --- | --- | --- | --- |
| `developer-sdk` | ✅ | ✅ | ✅ | ✅ explicit | ✅ | ✅ |
| Stripe | ✅ (Node + Python) | partial | ❌ | ❌ | ❌ | n/a |
| Twilio | partial (Node + Java) | ❌ | ❌ | ❌ | ❌ | n/a |
| Auth0 | partial (Node) | ❌ | ❌ | ❌ | ❌ | n/a |
| AWS SDK v3 | ✅ via X-Ray + OTel | partial | ❌ | partial | ❌ | partial |

## Type-safety + ergonomics

| Surface | Strong types in TS? | Discriminated union errors? | Tenant primitive? | Cedar at SDK? |
| --- | --- | --- | --- | --- |
| `developer-sdk` | ✅ (zod-shaped guards generated) | ✅ `OyatieError` discriminated union | ✅ first-class | ✅ |
| Stripe | ✅ | partial | ❌ (account ID) | ❌ |
| Twilio | partial | ❌ | ❌ | ❌ |
| Auth0 | ✅ | partial | ❌ (tenant in URL only) | ❌ |
| AWS SDK v3 | ✅ | ✅ (Smithy-driven) | ❌ (account in creds) | ❌ |

## Cost

The SDKs are free downloads. Costs flow from underlying API consumption per the originating µservice's tier.

## Where `developer-sdk` wins

1. 13 languages day-1; 22 at compliance_pack-bound paid vs vendor 7-12.
2. HTTP/3 default — wins on tail latency.
3. Cedar at the SDK — fail-fast without server round-trip.
4. Tenant primitive — vendors require URL or account-ID workarounds.
5. Cosign signing + attestations — vendors don't publish supply-chain attestations.

## Where vendors still win

1. **Maturity** — Stripe SDKs have been hand-tuned for 12+ years; ergonomics in some edge cases are still finer.
2. **Public docs ecosystem** — vendor docs have decades of Stack Overflow + YouTube + Medium; Oyatie is new.
3. **Per-language idiom mastery** — vendors hand-craft idiomatic Rubyisms / Pythonisms / Swiftisms; Oyatie has them for Rust + Go,
   templated for others.

## Reproducibility

```bash
make benchmarks.developer-sdk.run \
  VENDORS="developer-sdk,stripe,twilio,auth0,aws-sdk-v3" \
  LANGS="rust,typescript,python,go" \
  WORKLOADS="cold-import,first-call,retry-storm"
```

Evidence: `.foundry/evidence/benchmarks/developer-sdk/2026-05-15T11:08:42Z/`.
