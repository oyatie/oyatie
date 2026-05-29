# IP-022 — Go SDK (Phase 2)

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: go-sdk
**status**: design-ready
**acceptance_status**: design-ready
**phase**: 2
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-015, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

Go SDK implementing the OpenFeature `Provider` interface. Phase 2 SDK targeting Go-based µservices (gateway, proxy layers). gRPC + quic-go transport; `sync.Map` cache; goroutine-based SSE stream.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` struct | Implements `go.openfeature.dev/sdk` `FeatureProvider` interface |
| 2 | gRPC+QUIC transport | `google.golang.org/grpc` over `quic-go`; TLS 1.3; X25519MLKEM768 preference |
| 3 | `sync.Map` cache | Key: `(tenantID, flagKey)`; TTL 30s; LKG: JSON file in `os.UserCacheDir()` |
| 4 | SSE goroutine | `net/http` EventSource; goroutine per tenant channel; context cancellation on `Shutdown()` |
| 5 | Type safety | Generics (`ResolveBoolean[T bool]`, `ResolveString[T string]`, etc.) |
| 6 | Tests | `go test ./...` green; race detector clean (`go test -race`); benchmark ≤5µs in-cache |

## Definition of Done

- `go test -race ./...` green
- OpenFeature Go conformance set passes
- Goroutine leak test: 1000 provider initializations + shutdowns → zero leaked goroutines
- Bundle: importable as `go get go.oyatie.io/feature-flags-sdk`
