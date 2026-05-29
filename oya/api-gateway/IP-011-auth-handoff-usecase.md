# IP-011: `oya-api-gateway-auth-handoff-usecase` crate

**Status:** design-ready
**Owner:** axis-network + ops-security

## A — Scope

Use-case: forward inbound authn evidence (cookie / JWT / mTLS) to identity µservice, receive PrincipalContext, sign and forward upstream.

## B — Acceptance criteria

- Cache hit ≤60s.
- p99 ≤0.2ms (cache hit) or ≤2ms (cache miss).
- Signature verify on every cached principal.

## Wave 15 A-G substance

### A - Problem
The gateway must convert external authentication evidence into a bounded `PrincipalContext` before route, residency, rate-limit, and abuse decisions can run, without turning the edge into the identity service.

### B - Approach
Implement `oya-api-gateway-auth-handoff-usecase` as a usecase crate that validates inbound auth evidence, calls identity only on cache miss or policy-required refresh, verifies signed principal context, and emits a forwarding envelope for upstream mTLS/SPIFFE calls.

### C - Deliverables
- Auth evidence model for cookie session, bearer JWT, mTLS client certificate, webhook signature, machine token, and anonymous public read.
- Principal cache keyed by tenant, subject, credential hash, route auth class, and cell epoch.
- Signature verification path for cached and freshly returned principals.
- Identity handoff port that returns `PrincipalContext` with tenant, cell, packs, scopes, and freshness.
- Failure mapping for unauthenticated, expired credential, invalid signature, stale cell epoch, identity unavailable, and forbidden auth class.
- Audit intent for auth handoff success/failure without logging credential material.

### D - Ordered implementation steps
1. Define auth evidence and principal context DTOs from contract and policy fields.
2. Add cache lookup with max 60s TTL and signed-context verification.
3. Add identity handoff port and miss path with timeout budget.
4. Add route-auth-class compatibility checks before upstream forwarding.
5. Add audit-safe redaction for JWT, cookie, mTLS, and webhook inputs.
6. Add fixtures for cache hit, cache miss, expired token, bad signature, and identity timeout.
7. Wire output into routing-usecase and rate-limit bucket key construction.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-auth-handoff-usecase` passes.
- Cache-hit path verifies signatures and stays within the documented p99 budget in benchmark fixtures.
- Cache-miss path fails closed for protected routes when identity is unavailable.
- Redaction tests prove no cookie, bearer token, private key, or webhook secret reaches audit logs.
- Principal fields satisfy `policy/tenant-scope.cedar`, `policy/route-authorization.cedar`, and `policy/sov-cloud-overlay.cedar`.

### F - Evidence
Grounding files: `ARCHITECTURE.md`, `PRD.md`, `contracts/api-gateway.openapi.yaml`, `policy/tenant-scope.cedar`, `policy/route-authorization.cedar`, `policy/sov-cloud-overlay.cedar`, `operational-boundaries.md`, and `decisions/ADR-MS-001-edge-admission-policy-and-pqc-contract.md`.

### G - Counterpart comparison
GitHub webhook/API ingress is the concrete counterpart for mixed auth handoff because webhook signatures, bearer/API tokens, app installations, and anonymous public endpoints all enter the same edge while producing different principal shapes. Oyatie follows that pattern but requires signed principal context and cell-aware scope fields before forwarding.

## Remediation notes

- Rewrote the thin auth handoff stub into a usecase plan with explicit evidence types, cache behavior, signed-principal verification, and redaction gates.
- This IP intentionally avoids identity-service internals; it defines the edge handoff contract and failure semantics.
- Future remediation should add concrete identity proto/OpenAPI references once the identity service contract path is finalized.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Evidence model | `contracts/api-gateway.openapi.yaml` | Cookie, JWT, mTLS, webhook, machine, and anonymous classes fit the request envelope. |
| Principal fields | `policy/tenant-scope.cedar` | Tenant, cell, subject, scopes, and pack roster are available. |
| Route auth | `policy/route-authorization.cedar` | Principal context maps to route auth classes. |
| Residency | `policy/sov-cloud-overlay.cedar` | Principal cell and pack fields feed residency checks. |
| GitHub auth mix | GitHub webhook/API ingress | Webhook signatures and API tokens resolve to different principal shapes. |
| Cache bound | `performance-benchmark-numbers-2026-05-20.md` | Cache hit and miss budgets are explicit. |
| Redaction | `operational-boundaries.md` | Credentials cannot enter audit logs. |
| Audit | `contracts/api-gateway.asyncapi.yaml` | Auth handoff failure can be correlated with request denial. |
| Upstream mTLS | `iac/spire-trust-bundle.yaml` | Forwarding context can bind to SPIFFE identity. |
| Failure modes | `failure-modes.md` | Expired credential, bad signature, identity timeout, and stale cell remain distinct. |
| Rate-limit handoff | `IP-009-rate-limit-domain-crate.md` | Principal context supplies tenant and route bucket keys. |
| Abuse handoff | `IP-012-abuse-defence-domain.md` | Anonymous and suspicious auth classes can influence abuse scoring. |

## Remediation follow-up checklist

- Add GitHub webhook signature, app token, bearer token, and anonymous fixtures.
- Add bad signature and expired credential fixtures that redact raw secrets.
- Add identity-timeout fixture that fails closed for protected routes.
- Add cache-hit signature verification fixture under the 60 second TTL.
- Add principal-to-rate-limit bucket fixture so tenant/route keys stay aligned.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-011-auth-handoff-usecase.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-011-auth-handoff-usecase.md`.
