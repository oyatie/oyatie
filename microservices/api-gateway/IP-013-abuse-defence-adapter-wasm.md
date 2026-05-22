# IP-013: `oya-api-gateway-abuse-defence-adapter-wasm` crate

**Status:** design-ready
**Owner:** axis-network + ops-security

## A — Scope

Wasm filter compiled to be loaded into Envoy as a per-request CPU-light filter.

## B — Acceptance criteria

- Wasm binary < 1MB.
- p99 < 200µs per request.
- Bot-score eval + JA4 fingerprint + rate-limit lookup.
- Per-tenant config from Envoy xDS.

## C — References

- ADR-0297 (in flight)
- `iac/edge-waf.yaml`

## Wave 15 A-G substance

### A - Problem
The abuse-defence domain produces pure decisions, but the gateway needs an Envoy-loadable filter that can extract per-request signals, call the domain logic, and enforce challenge/throttle/forbid outcomes without overloading the edge.

### B - Approach
Implement `oya-api-gateway-abuse-defence-adapter-wasm` from `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml` as a CPU-bounded Envoy Wasm adapter. It reads request metadata, JA4/JA4+ hints, route class, tenant config, rate-limit hints, and honeypot route markers from xDS; then it returns headers/metadata for downstream admission and audit.

### C - Deliverables
- Wasm host bindings for request headers, dynamic metadata, peer TLS attributes, route config, and local response.
- Signal extractor for JA4/JA4+, frame pattern hash, user-agent family, pagination depth, body hash class, and honeypot marker.
- Domain decision bridge for observe, challenge, throttle, forbid, and audit-only.
- Bounded rate-limit hint lookup using already-computed metadata rather than blocking Valkey calls from Wasm.
- Per-tenant xDS config reader with version pinning and safe defaults.
- Metrics and audit metadata for bot-score bucket, signal class, decision class, and filter latency.

### D - Ordered implementation steps
1. Define Wasm ABI wrapper around domain input/output structs.
2. Add signal extraction for headers, TLS metadata, and route metadata.
3. Load per-tenant config from xDS with explicit version and fallback behavior.
4. Map domain decisions to Envoy local response, dynamic metadata, or continue.
5. Add latency budget instrumentation and binary-size checks.
6. Test with Envoy fixture requests for bot, human, scraper, spoofed client, and honeypot hit.
7. Confirm filter metadata feeds routing, rate-limit, and audit paths without direct network I/O.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-abuse-defence-adapter-wasm --features fixtures` passes.
- Wasm artifact size remains under 1 MB in release build.
- Fixture p99 evaluation remains under 200 microseconds on representative request metadata.
- Filter denies or challenges according to `policy/abuse-defence.cedar` without calling external services.
- Audit metadata maps to `oya.api_gateway.waf.triggered`, `oya.api_gateway.bot-score.high`, and `oya.api_gateway.honeypot.activated`.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml`, `policy/abuse-defence.cedar`, `iac/edge-waf.yaml`, `runbooks/bot-storm.md`, `runbooks/ddos-mitigation.md`, `dashboards/bot-score-distribution.json`, and `ARCHITECTURE.md`.

### G - Counterpart comparison
Twilio API edge is the concrete counterpart because communications ingress must filter spoofing, high-rate abuse, and webhook replay while keeping latency low. Oyatie maps that edge pressure into Envoy Wasm, Cedar-compatible decisions, and audit metadata.

## Remediation notes

- Rewrote the Wasm stub into a runtime-adapter plan with host bindings, metadata flow, decision mapping, and latency/size gates.
- Keep blocking Valkey, identity, and policy-engine calls out of the Wasm path; use metadata produced by earlier adapters or caller-side libraries.
- Future remediation should add a checked fixture set for Envoy request metadata once the Wasm crate exists.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Host bindings | `iac/envoy-config.yaml` | Headers, TLS metadata, route metadata, and local response APIs are available. |
| Domain bridge | `IP-012-abuse-defence-domain.md` | Wasm input/output maps to pure decision types. |
| Cedar parity | `policy/abuse-defence.cedar` | Filter decisions match policy thresholds. |
| xDS config | `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml` | Per-tenant config source is declared. |
| Twilio pressure | Twilio API edge | Spoof, replay, and high-volume abuse cases stay under latency budget. |
| No network calls | `ARCHITECTURE.md` | Hot-path policy evaluation is caller-side/local. |
| Metrics | `dashboards/bot-score-distribution.json` | Latency and score buckets are observable. |
| DDoS hook | `runbooks/ddos-mitigation.md` | Scrub/challenge paths can consume filter metadata. |
| Bot storm hook | `runbooks/bot-storm.md` | Temporary threshold changes are represented through config. |
| Audit | `contracts/api-gateway.asyncapi.yaml` | Filter decisions emit WAF/bot-score/honeypot metadata. |
| Size gate | `performance-benchmark-numbers-2026-05-20.md` | Binary size and p99 budget are checked as release constraints. |
| Rate-limit handoff | `IP-010-rate-limit-adapter-valkey.md` | Filter reads precomputed rate-limit hints rather than blocking Valkey. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md`.
