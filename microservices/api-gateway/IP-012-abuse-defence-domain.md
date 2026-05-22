# IP-012: `oya-api-gateway-abuse-defence-domain` crate

**Status:** design-ready
**Owner:** axis-network + ops-security
**Authority:** ADR-0297 (in flight).

## A — Scope

Domain primitives for anti-bot + anti-spoof + anti-scrape. Pure.

```rust
pub struct BotScore(pub u8); // 0..=100

pub struct Fingerprint {
    pub ja4: Ja4Hash,
    pub ja4plus: Ja4PlusHash,
    pub frame_pattern: FramePatternHash,
}

pub enum AbuseClass {
    Volumetric,
    CredentialStuffing,
    Scrape,
    HoneypotHit,
    SequentialIdScrape,
    AlphabeticalPagination,
}
```

## B — Acceptance criteria

- Pure; no I/O.
- Property tests on JA4 hash stability.

## Wave 15 A-G substance

### A - Problem
The gateway needs abuse-defence primitives that can explain bot, spoof, scrape, and honeypot decisions before Wasm, WAF, Valkey, or audit adapters run.

### B - Approach
Implement `oya-api-gateway-abuse-defence-domain` from `catalog/oya-api-gateway-abuse-defence-domain.yaml` as pure scoring and classification logic for JA4/JA4+, frame patterns, route sensitivity, tenant age, pagination shape, and honeypot evidence.

### C - Deliverables
- `BotScore` bounded to `0..=100` with explicit confidence metadata.
- `Fingerprint` value object for JA4, JA4+, frame-pattern hash, user-agent family, and TLS capability class.
- `AbuseSignal` values for volumetric burst, credential stuffing, scrape traversal, honeypot hit, sequential-id scrape, and alphabetical pagination.
- `AbuseDecision` with observe, challenge, throttle, forbid, and audit-only outcomes.
- Property tests for hash stability, score monotonicity, and no-I/O purity.

### D - Ordered implementation steps
1. Define value types for fingerprint and route-observation inputs.
2. Encode score composition without wall-clock or network dependencies.
3. Add classifiers for bot, spoof, scrape, honeypot, and deep-pagination signals.
4. Map classifiers to decision levels that Wasm and REST/gRPC can consume.
5. Add fixtures from `policy/abuse-defence.cedar` and `runbooks/bot-storm.md`.
6. Prove serialization stability for audit-chain payload builders.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-abuse-defence-domain` passes.
- Property tests prove stable JA4/JA4+ hash handling and bounded score composition.
- No dependency on Envoy, Wasm host APIs, Valkey, OpenBao, or audit-chain clients.
- Decisions map to `oya.api_gateway.bot-score.high`, `oya.api_gateway.waf.triggered`, and `oya.api_gateway.honeypot.activated` evidence.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-abuse-defence-domain.yaml`, `ARCHITECTURE.md`, `policy/abuse-defence.cedar`, `iac/edge-waf.yaml`, `runbooks/bot-storm.md`, `runbooks/ddos-mitigation.md`, and `contracts/metric-naming-convention.md`.

### G - Counterpart comparison
Twilio API edge is the concrete counterpart because SMS/voice/webhook APIs face spoofing, credential stuffing, and scrape pressure at high volume. Oyatie must match that edge classification discipline while keeping the domain pure and adding Cedar-compatible deny reasons.

## Remediation notes

- Expanded this foundation IP from a primitive sketch into service-specific A-G content tied to abuse-defence policy, runbooks, metrics, and catalog evidence.
- Keep this file focused on domain scoring; Envoy Wasm host calls and per-tenant xDS configuration belong to IP-013.
- Future remediation should add fixture names once the crate exists so the property-test gate can be checked mechanically.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Bot score | `policy/abuse-defence.cedar` | Score thresholds can map to observe, challenge, throttle, and forbid. |
| Fingerprint | `ARCHITECTURE.md` | JA4, JA4+, and frame pattern inputs are represented. |
| Spoof signal | `iac/edge-waf.yaml` | Header/TLS inconsistency can become a domain signal. |
| Scrape signal | `runbooks/bot-storm.md` | Sequential and alphabetical traversal are classified. |
| Honeypot signal | `IP-018-honeypot-route-mgr.md` | Honeypot hits become abuse evidence, not route data. |
| Twilio pressure | Twilio API edge | Spoofing and high-volume webhook abuse are explicit counterpart cases. |
| Audit mapping | `contracts/api-gateway.asyncapi.yaml` | Bot-score and WAF trigger events can carry decision class. |
| Metrics | `dashboards/bot-score-distribution.json` | Score distribution can be reported without leaking fingerprints. |
| Purity | `catalog/oya-api-gateway-abuse-defence-domain.yaml` | No Envoy, Wasm, Valkey, or network dependency is required. |
| Property tests | `runbooks/ddos-mitigation.md` | Volumetric and scrape scenarios stay deterministic under fixture load. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-012-abuse-defence-domain.md`.
