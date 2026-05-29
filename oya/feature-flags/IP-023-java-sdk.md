# IP-023 — Java SDK (Phase 2)

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: java-sdk
**status**: design-ready
**acceptance_status**: design-ready
**phase**: 2
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-015, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

Java SDK implementing the OpenFeature `Provider` interface. Phase 2 SDK targeting JVM-based µservices and Android clients. OkHttp3 + HTTP/3; `ConcurrentHashMap` cache; `CompletableFuture` async API; SSE via `okhttp3-sse`.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` class | Implements `dev.openfeature.sdk.FeatureProvider` interface |
| 2 | OkHttp3 transport | `OkHttpClient` with `ConnectionSpec.MODERN_TLS`; HTTP/3 via OkHttp QUIC support |
| 3 | `ConcurrentHashMap` cache | Key: `TenantFlagKey(tenantId, flagKey)`; TTL 30s via `ScheduledExecutorService`; LKG: Jackson JSON to `System.getProperty("user.home")/.cache/oya-ff/lkg.json` |
| 4 | SSE stream | `okhttp3-sse` `EventSource`; per-tenant; exponential backoff reconnect |
| 5 | Android support | `minSdk 26`; ProGuard/R8 keep rules included |
| 6 | Tests | JUnit 5 + Mockito; `OkHttp MockWebServer` for transport tests; OpenFeature Java conformance set |

## Definition of Done

- `./gradlew test` green
- Android lint clean
- OpenFeature Java conformance set passes
- Published to Maven Central as `io.oyatie:feature-flags-sdk:1.x`
