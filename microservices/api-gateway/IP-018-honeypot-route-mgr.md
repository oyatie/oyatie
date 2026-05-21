# IP-018: Honeypot route manager

**Status:** design-ready
**Owner:** axis-network + ops-security
**Authority:** ADR-0297 (in flight).

## A — Scope

Worker that mints honeypot route patterns + canary payloads + tracks attempted exfiltration. Per documentation-rigor.md §3.2.3 anti-bot row #8.

## B — Acceptance criteria

- Honeypot routes never serve real load (synthetic responses only).
- Canary payloads contain unique markers per tenant.
- Scraper-ingestion detection via downstream-scrape audit.
- Adaptive challenge on honeypot-hit (no immediate block — observe first).

## Wave 15 A-G substance

### A - Problem
The gateway needs controlled synthetic routes that reveal scraping, credential stuffing reconnaissance, and data-exfiltration reuse without risking real tenant data or blocking too early to learn attacker behavior.

### B - Approach
Implement the honeypot route manager as a worker that mints synthetic route patterns, publishes them through the routing worker, tags canary payloads per tenant/cell, observes access attempts, and escalates from observe to challenge/throttle/forbid through abuse-defence policy.

### C - Deliverables
- Honeypot route model for synthetic path, tenant scope, route class, marker ID, expiry, allowed friendly principals, and synthetic response template.
- Canary payload marker generator with tenant/cell/request provenance and audit-chain hash.
- Route publication handoff to routing-worker/xDS with a hard guard that synthetic routes cannot map to real upstreams.
- Detection pipeline for honeypot hit, marker reuse, downstream-scrape audit, and repeated sequential-id probing.
- Adaptive response policy that starts observe-only, then challenge, throttle, and forbid based on signal confidence.
- Runbook hooks for bot-storm and DDoS mitigation activation/deactivation.

### D - Ordered implementation steps
1. Define synthetic route and marker DTOs.
2. Add marker generation with tenant/cell uniqueness and no real data payloads.
3. Add route publication handoff that rejects real upstream bindings.
4. Add hit observer and downstream marker-reuse matcher.
5. Add adaptive response state machine tied to abuse-defence decisions.
6. Emit audit events for activated, hit, marker-reused, challenge-raised, throttled, and deactivated.
7. Test friendly-principal allowlist, attacker path, expiry, and rollback/deactivation.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-honeypot-route-mgr --features fixtures` passes once the crate is introduced.
- Synthetic routes cannot resolve to a production upstream in fixtures or route-bundle validation.
- Canary marker uniqueness holds per tenant/cell/route/epoch.
- First honeypot hit observes and audits before immediate block unless a higher-confidence abuse signal is already present.
- Metrics align with `oya_api_gateway_honeypot_hits_total` and audit events include `oya.api_gateway.honeypot.activated`.

### F - Evidence
Grounding files: `policy/abuse-defence.cedar`, `runbooks/bot-storm.md`, `runbooks/ddos-mitigation.md`, `contracts/metric-naming-convention.md`, `ARCHITECTURE.md`, `iac/edge-waf.yaml`, and `dashboards/bot-score-distribution.json`.

### G - Counterpart comparison
GitHub webhook/API ingress is the concrete counterpart for synthetic trap routing because public endpoints face automated recon, secret replay, and scraper reuse. Oyatie adds tenant-specific canary payload markers, synthetic-only route enforcement, and audit-chain escalation.

## Remediation notes

- Expanded IP-018 from a short honeypot sketch into a worker plan with route minting, synthetic response constraints, marker detection, adaptive response, metrics, and audit evidence.
- The file is not currently listed in `manifest.json`; follow-up should add IP-018 to the machine-readable IP inventory if this plan remains part of the accepted API gateway IP set.
- Keep observe-first behavior explicit so abuse analysis can learn from early hits without prematurely teaching attackers which paths are traps.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Abuse policy | `policy/abuse-defence.cedar` | Honeypot hit maps to observe/challenge/throttle/forbid progression. |
| Bot storm | `runbooks/bot-storm.md` | Activation and deactivation are operator-visible. |
| DDoS | `runbooks/ddos-mitigation.md` | Scrape signatures can trigger temporary honeypot routes. |
| Metrics | `contracts/metric-naming-convention.md` | `oya_api_gateway_honeypot_hits_total` is the named counter. |
| GitHub pressure | GitHub webhook/API ingress | Public API reconnaissance and secret replay have synthetic traps. |
| Route worker | `IP-008-routing-worker-crate.md` | Synthetic routes publish through validated route bundles. |
| Abuse domain | `IP-012-abuse-defence-domain.md` | Hits become domain signals rather than route data. |
| Wasm adapter | `IP-013-abuse-defence-adapter-wasm.md` | Filter metadata can flag honeypot marker access. |
| Synthetic guard | `failure-modes.md` | Real upstream binding is impossible in fixtures. |
| Audit | `contracts/api-gateway.asyncapi.yaml` | Activated, hit, marker-reused, and deactivated events are emitted. |
| Dashboard | `dashboards/bot-score-distribution.json` | Honeypot effects are visible alongside bot-score movement. |
| Manifest debt | `manifest.json` | IP-018 absence is recorded as follow-up, not hidden. |

## Remediation follow-up checklist

- Add GitHub-style public endpoint reconnaissance fixture.
- Add synthetic-route fixture proving no production upstream binding is possible.
- Add marker-reuse fixture with tenant/cell/route/epoch uniqueness checks.
- Add observe-first fixture before challenge/throttle/forbid escalation.
- Add audit assertions for activated, hit, marker-reused, escalated, and deactivated events.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-018-honeypot-route-mgr.md`.
