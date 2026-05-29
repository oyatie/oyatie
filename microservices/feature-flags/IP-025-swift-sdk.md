# IP-025 — Swift SDK (Phase 2)

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: swift-sdk
**status**: design-ready
**acceptance_status**: design-ready
**phase**: 2
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-015, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

Swift SDK implementing the OpenFeature `FeatureProvider` protocol. Phase 2 SDK targeting iOS/macOS clients. `URLSession` with HTTP/3; `NSCache` + `UserDefaults` LKG; `URLSessionDataDelegate` SSE stream; Swift concurrency (async/await + actors).

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` class | Implements `OpenFeature.FeatureProvider` protocol (Swift OpenFeature SDK) |
| 2 | `URLSession` transport | `URLSessionConfiguration.default` with `httpVersion = .version3`; TLS 1.3; App Transport Security compliant |
| 3 | `NSCache` + `UserDefaults` | In-memory `NSCache` TTL 30s; LKG serialized to `UserDefaults` as JSON (survives app restart) |
| 4 | SSE `AsyncSequence` | `URLSessionDataDelegate`-based `AsyncStream<FlagStateChangedEvent>`; per-tenant actor |
| 5 | Swift concurrency | All public APIs `async`; actor-isolated cache; Sendable conformance |
| 6 | Privacy manifest | `PrivacyInfo.xcprivacy` declaring no data collection; required for App Store submission |
| 7 | Tests | XCTest + swift-testing; network mocking via `URLProtocol`; OpenFeature Swift conformance set |

## Definition of Done

- `swift test` green
- Privacy manifest present and valid
- OpenFeature Swift conformance set passes
- Swift Package Index listing at `https://swiftpackageindex.com/oyatie/feature-flags-swift-sdk`
- iOS 16+ / macOS 13+ deployment target
