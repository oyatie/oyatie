# IP-024 — .NET SDK (Phase 2)

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: dotnet-sdk
**status**: design-ready
**acceptance_status**: design-ready
**phase**: 2
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-015, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

.NET SDK implementing the OpenFeature `FeatureProvider` interface. Phase 2 SDK targeting .NET 8+ services. `HttpClient` with HTTP/3; `MemoryCache` with TTL; `IHostedService` background SSE listener.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` class | Implements `OpenFeature.SDK.IFeatureProvider` interface |
| 2 | `HttpClient` transport | `HttpClient` with `HttpVersion.Version30`; `HttpVersionPolicy.RequestVersionOrLower` fallback |
| 3 | `IMemoryCache` | `MemoryCacheEntryOptions` with `AbsoluteExpirationRelativeToNow = TimeSpan.FromSeconds(30)`; LKG: JSON to `Environment.GetFolderPath(SpecialFolder.LocalApplicationData)` |
| 4 | SSE `IHostedService` | Background hosted service per tenant; `CancellationToken` on shutdown; Polly retry policy |
| 5 | DI integration | `IServiceCollection.AddOyatieFeatureFlags(options)` extension method |
| 6 | Tests | xUnit + Moq; `MockHttpMessageHandler`; OpenFeature .NET conformance suite |

## Definition of Done

- `dotnet test` green
- DI integration test: provider registered via `AddOyatieFeatureFlags` resolves correctly
- OpenFeature .NET conformance suite passes
- Published to NuGet as `Oyatie.FeatureFlags.Sdk`
