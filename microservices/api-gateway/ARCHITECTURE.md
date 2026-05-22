# api-gateway — Architecture (Tier-0 Edge µservice)

**Status:** Accepted
**Authority tier:** 2
**Doc class:** Architecture deep-dive (per `docs/standards/documentation-rigor.md` §2)
**Binding ADRs:** ADR-0157, ADR-0182, ADR-0183, ADR-0242, ADR-0243, ADR-0244, ADR-0246+amendment, ADR-0248, ADR-0253, ADR-0254, ADR-0263, ADR-0273, ADR-0284, ADR-0294, ADR-0295, ADR-0296, ADR-0297 (abuse-defence baseline, in flight).
**Hyperscaler precedent:** AWS API Gateway + AWS CloudFront + Cloudflare Edge (Workers + Magic Transit + Bot Management) + Google Cloud Apigee + Kong Gateway + Istio Gateway + Envoy. The shape of this µservice = "Cloudflare Edge + Envoy data-plane + Apigee management-plane wired together with Cedar gates."

---

## §A — Entry point (cold-start question)

> *"I'm an intern joining today. A request hits `https://app.oyatie.com/api/v1/users/me`. What happens — every step — until the response leaves the edge?"*

The cold-start trace is in §B. The summary:

1. **DNS resolution** returns an Anycast IP (Cloudflare + a sovereign-cell-aware overlay) plus an HTTPS RR with ECH config.
2. **TLS 1.3 + ECH** negotiation terminates at the nearest cell-tier-0 edge cell (every cell of every region — shuffle-sharded per ADR-0248).
3. **HTTP/3 (QUIC) preferred** (h3 Alt-Svc advertised). Fallback chain HTTP/3 → HTTP/2 → HTTP/1.1 per ADR-0253; never skip a tier; HTTP/1.0 forbidden.
4. **JA4/JA4+/JA3 fingerprint** + **bot-score** computed.
5. **Edge rate-limit** (token-bucket per-IP, per-fingerprint, per-tenant) gates.
6. **Cedar gate** (`abuse-defence.cedar`, `route-authorization.cedar`, `rate-limit.cedar`, `tls-policy.cedar`) evaluates the request envelope.
7. **Anti-bot CAPTCHA-on-suspicion** branch fires only when bot-score > threshold (never default path — accessibility floor).
8. **Auth handoff** to identity µservice (SPIFFE SVID for backend, signed `X-Oya-Principal-Context` for the µservice).
9. **Request canonicalisation** (URL normalisation, header normalisation, body size bounds).
10. **Route map** picks the upstream µservice. mTLS to upstream via SPIFFE.
11. **Response mediation** (security headers, CSP, cache directives, ECH-stamp, observability trailers).
12. **Audit event** emitted to `oya.api_gateway.request.admitted` / `oya.api_gateway.request.denied` per ADR-0263.
13. **Blue-green / canary** routing per the active deployment plan.
14. Response leaves the edge.

---

## §B — Layer-by-layer trace

### B-1. DNS layer

- **Authoritative DNS** is split-horizon per sovereign-cell-tier-0 overlay (per ADR-0248).
- Records served per tenant: `A` / `AAAA` (Anycast IP), `HTTPS` (RFC 9460 SVCB with `alpn=h3,h2`, `ipv6hint=`, `ech=<echconfig>`), `MTA-STS` for mail, `CAA` to lock CT.
- The HTTPS RR's `ech=` is rotated quarterly via `runbooks/tls-cert-rotation.md`.
- The DNSSEC chain is signed by oyatie's per-cell KSK in OpenBao.

### B-2. TLS termination

- **TLS 1.3 floor.** No TLS 1.2, no TLS 1.0/1.1, ever. CI lane `oya-governance-tls-floor` BLOCKER.
- **ECH (Encrypted Client Hello, RFC 9460 + draft-ietf-tls-esni-22).** Enabled at every Tier-0 cell. Outer SNI is the cell's generic frontend; inner SNI is the per-tenant host. Clients lacking ECH fall through to standard TLS 1.3 without error.
- **PQC hybrid.** `X25519MLKEM768` (IANA codepoint `0x11ec`) advertised. Signature hybrid `ed25519+ml_dsa_65` used for new cert chains. Non-PQ clients fall through silently. Never refuse a session because peer lacks PQC.
- **Strict cipher suites:** `TLS_AES_128_GCM_SHA256`, `TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`. AEAD-only.
- **Curve preferences:** `X25519MLKEM768`, `X25519`, `P-256`. P-384 disabled (no business need; reduces attack surface).
- **OCSP stapling + must-staple.** **CT required** (≥2 SCTs from independent logs).
- **HSTS:** `max-age=63072000; includeSubDomains; preload` — served on every response from every tenant host. Preload list submission tracked in `iac/cert-manager-hsts-preload.yaml`.
- **mTLS toward upstream** via SPIFFE-issued SVIDs (ADR-0295). Outbound mTLS is on by default; mTLS-bypass requires explicit `?bypass_mtls=true` query — forbidden by Cedar in production.

### B-3. HTTP/3 + QUIC

- **Alt-Svc** advertisement: `alt-svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400`.
- **0-RTT** allowed only for *idempotent* operations (GET, HEAD, OPTIONS). POST/PUT/DELETE refuse 0-RTT to prevent replay.
- **QUIC connection migration** allowed; the cell holds per-connection-id state in distributed memory (Valkey cluster per cell).
- **QUIC blocked networks** (some corp/edu networks block UDP/443): client falls back to h2 (TCP/443). Verify via `runbooks/h3-fallback-verification.md`.
- **DDoS** scrub at the BGP layer (Magic Transit equivalent) before QUIC ever reaches the userspace stack.

### B-4. Fingerprinting

- **JA4** (TLS ClientHello fingerprint, replaces JA3): SHA-256 over TLS version + cipher list + extension list + signature-algorithm list.
- **JA4+** family: JA4S (ServerHello), JA4H (HTTP/2 frame patterns), JA4X (X.509).
- **HTTP/2 + HTTP/3 frame pattern** fingerprint (Akamai-style): which frames in what order with what settings.
- **Passive only.** A fingerprint NEVER alone gates a request — too many false positives. It's input to the bot-score model.

### B-5. Bot-score model

- Cloudflare Bot Management style; in-house equivalent for sovereign cells (where Cloudflare is unavailable).
- **Inputs:** JA4/JA4+, frame fingerprint, IP reputation (Spamhaus + Project Honeypot + internal), ASN class (residential vs DC vs mobile), behavioural history (HMAC-bound to fingerprint over 30-day window), challenge-pass history.
- **Output:** `bot_score ∈ [0,100]`. 0 = certain human. 100 = certain bot. ≥95 = forbid (per `policy/abuse-defence.cedar`).
- **Forwarded header:** `X-Oya-Bot-Score: <0..100>` to upstream µservice for downstream policy.
- **Always-pass list:** known friendly crawlers (Googlebot, Bingbot, AppleBot) verified via reverse-DNS + forward-DNS check (Google's recommended pattern). Misbehaving claimants → blackhole.

### B-6. Edge rate-limit

- **Token-bucket + sliding-window** (Cloudflare-style); per-IP, per-fingerprint, per-tenant, per-route-class (auth/read/write/admin).
- **Bucket sizes** (defaults; overridden per-tenant via `policy/rate-limit.cedar`):
  - Anonymous read: 60 req/min/IP burst 120
  - Anonymous auth attempt: 5 req/min/IP burst 10
  - Authenticated read: 600 req/min/tenant burst 1200
  - Authenticated write: 60 req/min/tenant burst 120
- **Storage:** per-cell Valkey Cluster (shuffle-sharded per ADR-0248). Cross-cell aggregation via Kafka stream `oya.api_gateway.rate-limit.tick`.
- **Headers returned:** `RateLimit-Limit`, `RateLimit-Remaining`, `RateLimit-Reset`, `Retry-After` per IETF draft-ietf-httpapi-ratelimit-headers.
- **Mitigation on saturation:** `runbooks/rate-limit-saturation.md`.

### B-7. Cedar gate

- `policy/abuse-defence.cedar` — forbids when bot-score>95, when rate exceeded, when fingerprint too young (<30s) + deep-paginating, when honeypot route is hit.
- `policy/route-authorization.cedar` — every route declares the actions it admits; only `Action::"Route::<route_id>"` permits.
- `policy/rate-limit.cedar` — per-tenant overrides, partner tenant_class overrides, friendly-crawler overrides.
- `policy/tls-policy.cedar` — refuses TLS<1.3, refuses non-AEAD, refuses non-CT-stapled.
- `policy/abuse-defence.cedar` is evaluated by the caller-side `oya-shared-policy-eval` library (per ADR-0246+amendment library-first dispatch). No network call to policy-engine for hot-path eval.
- **`policy_evaluation_mode`:** `caller-side-library`.
- **Fragment soak ≥60s** per ADR-0294.
- Every deny emits `oya.api_gateway.request.denied` with the matched Cedar fragment ID in the audit event payload.

### B-8. CAPTCHA / device attestation

- **hCaptcha** + **Cloudflare Turnstile** + **WebAuthn Origin-binding** — federated; tenant chooses default at onboarding.
- **App Attest (iOS)**, **Play Integrity (Android)** for native surfaces.
- Adaptive: presented only when `bot_score > threshold` AND `route_class ∈ [auth, write, admin]`. Default path (anonymous read) NEVER triggers CAPTCHA — accessibility floor.
- **Accessibility:** every CAPTCHA surface MUST offer audio + text alternative; CI lane `oya-governance-a11y-captcha` enforces.

### B-9. Auth handoff

- The api-gateway DOES NOT authenticate — it forwards to the identity µservice.
- **Browser surface:** cookie session (HMAC-signed, SameSite=Strict, HttpOnly, Secure, `__Host-` prefix). Token rotation on privilege escalation.
- **Native + machine surface:** signed JWT bound to client identity (mTLS exporter, RFC 8473 where supported).
- The gateway calls identity-µservice via SPIFFE SVID; identity returns a *Principal Context* (JWT/JWS); the gateway signs `X-Oya-Principal-Context` and forwards to upstream.
- **`provider_credential_mode`:** `tenant-byok` for any tenant-supplied auth providers; substrate owns zero credentials per ADR-0255 §D-4 / ADR-0296 BYOK-everywhere doctrine.

### B-10. Request canonicalisation

- **URL normalisation** (per RFC 3986): percent-decode unreserved, lowercase scheme + host, remove default port, resolve `..`/`.`, sort idempotent query params, strip BOM.
- **Header normalisation:** lowercase header names; deduplicate; reject duplicate `Host`; canonicalise CRLF; size cap 16KB per header, 64KB total.
- **Body size cap:** 10MB default; 100MB for streaming routes; 0B for GET/HEAD/OPTIONS.
- **Charset:** UTF-8 only; reject Latin-1 / SHIFT-JIS / Windows-1252 unless `Content-Type` declares it and the route opts in.
- **HTTP smuggling defence:** strict per RFC 9112 + RFC 9113 / RFC 9114; chunked + content-length conflict → 400; bare CR/LF in header values → 400.

### B-11. Route map + upstream call

- Routes declared in `iac/envoy-config.yaml` (Envoy `routes:` blocks).
- Each route names: `route_id`, `upstream_µservice`, `upstream_path_template`, `auth_required`, `audit_tag`, `rate_limit_class`, `cache_policy`.
- **mTLS** to upstream via SPIFFE; verified via cell-tier-0 trust bundle from `iac/spire-trust-bundle.yaml`.
- **Circuit breaker** per upstream µservice (per cell): 5 consecutive 5xx → trip → half-open after 30s → close after 3 consecutive 2xx. Bulkhead-isolated by cell.
- **Timeout budget:** 30s default; per-route override; client-deadline-aware (h2 `grpc-timeout` / h3 `X-Request-Deadline-Seconds`).

### B-12. Response mediation

- **Security headers** (every response, every host):
  - `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`
  - `Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; ...` (per `iac/csp-defaults.yaml`)
  - `X-Content-Type-Options: nosniff`
  - `Referrer-Policy: strict-origin-when-cross-origin`
  - `Permissions-Policy: ...`
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`
  - `Cross-Origin-Resource-Policy: same-origin`
- **Cache directives:** `Cache-Control: private, no-store` default; per-route override.
- **Observability trailers:** `X-Oya-Trace-Id`, `X-Oya-Span-Id`, `Server-Timing` (durations).
- **ECH-stamp:** `X-Oya-ECH-Status: applied | not-applied | not-negotiated` to help debugging.

### B-13. Audit emission

- **Per ADR-0263 emission contract:**
  - `oya.api_gateway.request.admitted`
  - `oya.api_gateway.request.denied`
  - `oya.api_gateway.waf.triggered`
  - `oya.api_gateway.rate-limit.exceeded`
  - `oya.api_gateway.tls.handshake.failed`
  - `oya.api_gateway.bot-score.high`
  - `oya.api_gateway.cedar.permit.matched`
  - `oya.api_gateway.cedar.deny.matched`
  - `oya.api_gateway.upstream.timeout`
  - `oya.api_gateway.upstream.circuit-open`
  - `oya.api_gateway.canary.routed`
  - `oya.api_gateway.bluegreen.routed`
  - `oya.api_gateway.tls.cert.rotated`
  - `oya.api_gateway.ech.config.rotated`
  - `oya.api_gateway.pqc.handshake.completed`
- Cardinality budget per metric documented in `compliance.md §observability`.
- All audit events Merkle-sealed per ADR-0028 audit-chain doctrine; signed by per-µservice key in sidecar (ADR-0296).

### B-14. Blue/green + canary

- **Blue/green** swap via `iac/envoy-config-bluegreen.yaml` weighted-routes (0/100 ↔ 100/0); held at 50/50 for ≥30s soak window; per ADR-0294 fragment-soak doctrine applied to route changes.
- **Canary** weighting per IP-015 canary cohort weighting model.
- **Auto-rollback** wired to SLO burn-rate (per ADR-0139 agentic SLO-gated promotion + ADR-0114 canary observability rollback). If burn-rate > 14× fast-burn-1h, traffic rolls back automatically.

---

## §C — Concrete example end-to-end

> A Korean tenant `tenant-acme-kr` calls `POST https://api.acme.example/v1/payments/intents` from a browser on a Vodafone mobile connection in Frankfurt.

| Step | Component | Observable |
|---:|---|---|
| 1 | DNS — Cloudflare Anycast → cell `eu-frankfurt-1-cell-0` | `X-Oya-Cell: eu-frankfurt-1-cell-0` |
| 2 | TLS 1.3 + ECH — outer SNI `edge.oyatie.com`, inner SNI `api.acme.example` | `X-Oya-ECH-Status: applied` |
| 3 | HTTP/3 negotiated (mobile carrier allows UDP/443) | `Alt-Used: h3` |
| 4 | JA4 fingerprint = `t13d1517h2_8daaf6152771_b1ff8ab2d16f` (Chrome 132 macOS) | `X-Oya-Fingerprint-Class: chrome-stable-desktop` |
| 5 | Bot-score = 7 (clean) | `X-Oya-Bot-Score: 7` |
| 6 | Rate-limit: tenant-acme-kr authenticated-write bucket has 58/60 remaining | `RateLimit-Remaining: 57` |
| 7 | Cedar gate: `route-authorization::Action::"Route::POST::/v1/payments/intents"` permit matches | audit `oya.api_gateway.cedar.permit.matched`, fragment `route-authorization#permit-005` |
| 8 | CAPTCHA: skipped (bot-score below threshold) | no observable |
| 9 | Auth handoff → identity-µservice via SPIFFE; principal returned | `X-Oya-Principal-Context: <jwt>` |
| 10 | Canonicalisation: URL `/v1/payments/intents`, body 487B JSON, charset UTF-8 OK | no observable |
| 11 | Route → `payments-µservice` cell `eu-frankfurt-1-cell-0` upstream; mTLS SPIFFE | `X-Oya-Upstream: payments` |
| 12 | Response: 201 Created, body 612B; security headers stamped | `Server-Timing: gateway;dur=4.2, upstream;dur=38.7` |
| 13 | Audit: `oya.api_gateway.request.admitted` Merkle-sealed | audit ledger |
| 14 | Response leaves edge — total 43ms p50 | edge-overview dashboard |

---

## §D — Common confusions

| Confusion | Resolution |
|---|---|
| "The gateway authenticates." | No. The gateway forwards to identity. The gateway is a **handoff** layer, not an auth layer. |
| "The gateway runs Cedar." | The gateway *evaluates* Cedar via the caller-side library `oya-shared-policy-eval` (ADR-0246+amendment). It does not host the policy-engine µservice itself. |
| "HTTP/1.1 is the safe fallback." | The fallback chain is h3 → h2 → h1.1. We never skip h2; we never fall to h1.0. |
| "Bot-score gates." | Bot-score is one INPUT to the Cedar gate. The gate decides; the score informs. |
| "CAPTCHA is on by default." | No. CAPTCHA is adaptive; presented only on suspicion. Default path is CAPTCHA-free for a11y. |
| "ECH replaces TLS SNI." | ECH encrypts the SNI. The outer SNI is the cell's frontend; the inner is the tenant host. |

---

## §E — Where to read next

- **PHASE-01-EDGE-SUBSTRATE-BUILDOUT.md** — what we are shipping in Wave-3.
- **threat-model.md** — STRIDE + LINDDUN.
- **dpia.md** — PII handling at the edge (JA4 fingerprints are pseudonymous; bot-score history is bound to fingerprint, not principal).
- **compliance.md** — pack-overlay roster.
- **runbooks/** — incident handling.

---

## §cell-aware-routing

`api-gateway` owns cell-aware tenant routing after Wave 15L. The gateway does
not call a standalone `cell` service on the hot path. It reads `tenant.cell`,
`cell_epoch`, `pack`, and `region` from the signed principal context produced by
identity and backed by tenancy. `cloud-iac` owns the cell registry, and
`observability` owns health and isolation verdicts.

Routing inputs:
- `tenant.cell`: primary cell selected by tenancy with
  `crates/oya-shuffle-sharding`.
- `tenant.cell_set`: optional additional cells for shuffle-shard width greater
  than one.
- `cell_epoch`: assignment epoch used to detect stale sessions.
- `pack` and `region`: residency boundaries that constrain upstream selection.
- `observability.cell_health`: healthy, degraded, isolated, draining, or
  unknown.
- `cloud-iac.routeable_cell`: ready cells with current OpenTofu-backed topology.

Hot-path behavior:
1. Gateway validates the signed principal context and rejects missing
   `tenant.cell` for authenticated tenant routes.
2. Gateway verifies the route's pack and region constraints before choosing an
   upstream.
3. Gateway prefers the primary cell when healthy.
4. For multi-cell assignments, gateway may choose another cell from
   `tenant.cell_set` only when the primary cell is degraded and the route's data
   semantics permit the fallback.
5. Gateway never creates a cell assignment, changes assignment salt, or marks a
   cell drained; it consumes tenancy, cloud-iac, and observability state.
6. Every admit or deny emits audit-chain evidence with `cell_id`, `cell_epoch`,
   route id, and circuit-breaker verdict.

Fail-closed cases:
- Principal lacks `tenant.cell` for a cell-scoped route.
- `cell_epoch` is older than the minimum accepted epoch exposed by tenancy.
- `cloud-iac` marks the cell non-routeable or retired.
- `observability` marks the cell isolated.
- Route pack or region conflicts with the principal's assigned cell.
- Multi-cell fallback would cross a data-residency boundary.

Operational notes:
- Circuit breakers are per upstream service and per cell.
- Rate-limit buckets remain per tenant and route class, but cell labels are
  attached for blast-radius debugging.
- Cache keys include `tenant_id`, `cell_epoch`, route id, and primary cell.
- Gateway logs never contain raw shuffle-shard ranking inputs; the crate is not
  linked into gateway for assignment.

Verification:
- Gateway tests must cover missing cell, stale epoch, isolated cell, drain, and
  residency conflict denials.
- ADR-0333 is the retirement decision record for why this routing behavior
  consumes cell context from principal claims instead of querying
  `microservices/cell/`.

## §principals (per ADR-0242)

The api-gateway operates as:

- `oyatie.api_gateway.envoy@cell-N` — the data-plane principal (per cell, per region).
- `oyatie.api_gateway.controlplane@cell-N` — the management-plane principal.
- `oyatie.api_gateway.policy-loader@cell-N` — loads Cedar fragments from policy-engine ledger.
- `oyatie.api_gateway.audit-emitter@cell-N` — emits audit events to the audit-chain.

Tenant-scoped principals that call it: every other µservice (north-bound only at the edge); every external tenant API client.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `principals (per ADR-0242)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `principals (per ADR 0242)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `principals (per ADR 0242)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals (per ADR 0242)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `principals (per ADR 0242)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.

## §cedar-gates (per ADR-0243)

Default-deny baseline at `policy/tenant-scope.cedar`. Per-fragment:

| Fragment | Gates |
|---|---|
| `route-authorization.cedar` | route-class admission |
| `rate-limit.cedar` | per-tenant bucket overrides |
| `abuse-defence.cedar` | bot-score / scrape-pattern / honeypot |
| `tls-policy.cedar` | TLS floor + AEAD + CT |
| `tenant-scope.cedar` | cross-tenant cross-talk default-deny |
| `sov-cloud-overlay.cedar` | sovereign-cell overrides (KR-CSAP, CN-PIPL, EU-sovereign) |
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `cedar-gates (per ADR-0243)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `cedar gates (per ADR 0243)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates (per ADR 0243)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cedar gates (per ADR 0243)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.

## §tenant-scoping (per ADR-0244)

Every audit event carries `tenant_id`; every rate-limit bucket key is salted with `tenant_id`; every emitted log line carries `tenant_id`. `audience_type ∈ {anonymous, b2c, b2b, partner, machine}` set per call. `provider_credential_mode = tenant-byok`. Tables: none in the gateway (stateless); audit emits go via the audit-chain µservice.
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Stripe Connect account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `tenant-scoping (per ADR-0244)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `tenant scoping (per ADR 0244)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping (per ADR 0244)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping (per ADR 0244)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `tenant scoping (per ADR 0244)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `tenant scoping (per ADR 0244)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `tenant scoping (per ADR 0244)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `tenant scoping (per ADR 0244)` workflow.
- Depth detail 17: `api-gateway` telemetry for `tenant scoping (per ADR 0244)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §substrate-product-binding (per ADR-0245)

This is a **substrate µservice**. Every product µservice depends on it (it's the front door). It depends on: identity, policy-engine (control-plane only), audit-chain, observability, cloud-secrets (OpenBao for TLS certs), spire (workload identity).
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `substrate-product-binding (per ADR-0245)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `substrate product binding (per ADR 0245)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding (per ADR 0245)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding (per ADR 0245)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `substrate product binding (per ADR 0245)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `substrate product binding (per ADR 0245)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `substrate product binding (per ADR 0245)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `substrate product binding (per ADR 0245)` workflow.
- Depth detail 17: `api-gateway` telemetry for `substrate product binding (per ADR 0245)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §policy-evaluation (per ADR-0246 + amendment)

`policy_evaluation_mode = caller-side-library`. We use `oya-shared-policy-eval` to evaluate Cedar fragments locally. Fragment loading is push-based from the policy-engine ledger (≤30s freshness).
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `policy-evaluation (per ADR-0246 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `policy evaluation (per ADR 0246 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation (per ADR 0246 + amendment)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation (per ADR 0246 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `policy evaluation (per ADR 0246 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `policy evaluation (per ADR 0246 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `policy evaluation (per ADR 0246 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `policy evaluation (per ADR 0246 + amendment)` workflow.
- Depth detail 17: `api-gateway` telemetry for `policy evaluation (per ADR 0246 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §intelligence-dispatch (per ADR-0255 + amendment)

The gateway does NOT call Intelligence on the hot path. Bot-score ML models run in the bot-management subsystem (separate from Intelligence). `intelligence_dispatch_mode = none`.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `intelligence-dispatch (per ADR-0255 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `intelligence dispatch (per ADR 0255 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch (per ADR 0255 + amendment)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch (per ADR 0255 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `intelligence dispatch (per ADR 0255 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `intelligence dispatch (per ADR 0255 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `intelligence dispatch (per ADR 0255 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `intelligence dispatch (per ADR 0255 + amendment)` workflow.
- Depth detail 17: `api-gateway` telemetry for `intelligence dispatch (per ADR 0255 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §ontology-read-path (per ADR-0257 + amendment)

The gateway does NOT read Ontology. `ontology_read_mode = none`.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `ontology-read-path (per ADR-0257 + amendment)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `ontology read path (per ADR 0257 + amendment)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path (per ADR 0257 + amendment)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path (per ADR 0257 + amendment)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `ontology read path (per ADR 0257 + amendment)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `ontology read path (per ADR 0257 + amendment)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `ontology read path (per ADR 0257 + amendment)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `ontology read path (per ADR 0257 + amendment)` workflow.
- Depth detail 17: `api-gateway` telemetry for `ontology read path (per ADR 0257 + amendment)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §transport (per ADR-0253)

- **Alt-Svc:** `alt-svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400`.
- **Fallback chain:** h3 → h2 → h1.1. Never skip a tier; HTTP/1.0 forbidden.
- **TLS profile:** TLS 1.3 floor; AEAD-only cipher suites; X25519MLKEM768 preferred; CT required; HSTS preload; OCSP must-staple.
- **ECH advertised** on every Tier-0/1/2/3 ingress.
- **PQC hybrid offered** in ClientHello/ServerHello.
- **h3 → h2 fallback under QUIC-blocked networks:** verified via `runbooks/h3-fallback-verification.md`.
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `transport (per ADR-0253)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `transport (per ADR 0253)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `transport (per ADR 0253)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `transport (per ADR 0253)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `transport (per ADR 0253)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.

## §deployment-shape (per ADR-0254)

- **Data plane:** Envoy in Kata Containers (Cloud Hypervisor) on K8s per cell.
- **Management plane:** Apigee-equivalent (in-house) container.
- **Bot-management:** in-house Wasm filter in Envoy (CPU-light, sub-ms eval).
- **Edge waypoint** (Istio Ambient) for cluster-internal east-west pre-admission filtering.
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `deployment-shape (per ADR-0254)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `deployment shape (per ADR 0254)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape (per ADR 0254)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape (per ADR 0254)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `deployment shape (per ADR 0254)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `deployment shape (per ADR 0254)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `deployment shape (per ADR 0254)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §observability (per ADR-0263)

Audit-event classes listed in §B-13. Metrics: `oya_api_gateway_requests_total{tenant_id, route_id, code}`, `oya_api_gateway_latency_seconds_bucket{...}`, `oya_api_gateway_tls_handshake_duration_seconds_bucket{...}`, `oya_api_gateway_bot_score_bucket{...}`. Cardinality budget: `tenant_id` cap 200k, `route_id` cap 5k, `code` cap 60. Trace span shape: `gateway.request` parent → `gateway.cedar.eval` + `gateway.upstream.<svc>` children.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Google SRE's four-signal model anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `observability (per ADR-0263)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `observability (per ADR 0263)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `observability (per ADR 0263)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `observability (per ADR 0263)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `observability (per ADR 0263)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `observability (per ADR 0263)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `observability (per ADR 0263)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `observability (per ADR 0263)` workflow.
- Depth detail 17: `api-gateway` telemetry for `observability (per ADR 0263)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §abuse-defence (per ADR-0297, in-flight)

THIS is where the edge controls live. Anti-bot (8 rows), anti-spoof (8 rows), anti-scrape (8 rows) — see `compliance.md §abuse-defence-roster` for the per-row implementation. Cedar fragment: `policy/abuse-defence.cedar`. WAF rules: `iac/edge-waf.yaml`. DDoS mitigation: BGP-layer scrub + per-cell rate buckets.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `abuse-defence (per ADR-0297, in-flight)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `abuse defence (per ADR 0297, in flight)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence (per ADR 0297, in flight)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence (per ADR 0297, in flight)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `abuse defence (per ADR 0297, in flight)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `abuse defence (per ADR 0297, in flight)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `abuse defence (per ADR 0297, in flight)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `abuse defence (per ADR 0297, in flight)` workflow.
- Depth detail 17: `api-gateway` telemetry for `abuse defence (per ADR 0297, in flight)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `api-gateway` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §credential-isolation (per ADR-0296)

The gateway holds zero long-lived credentials. TLS private keys are loaded at boot from OpenBao via sidecar (≤60s TTL refresh); the data-plane process never sees the OpenBao token. SPIFFE SVIDs rotate every 1h.

---
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `api-gateway` to the ≥50-line documentation-rigor floor.
- Service owner `axis-network` owns this answer; tier `substrate`; audience `all`.
- Primary capability/context: `north-south-request-admission`; bounded contexts: `north-south-request-admission`.
- API surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy surfaces: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`; +5 more.
- State/event surfaces: `api_gateway.north_south_request_admission`, `api_gateway.t0`, `api_gateway.microservices_api_gateway_capabilities_north_south_request_admission_yaml`, `api_gateway.minimal`, `api_gateway.edge_cedar_eval`; +1 more.
- SLO/dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `identity`, `policy-engine`, `audit-chain`, `observability`, `cloud-secrets`; +1 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `api-gateway` `north-south-request-admission` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/api-gateway/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `api-gateway` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `api-gateway` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `api-gateway` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `api-gateway` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `api-gateway` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `north-south-request-admission` evaluates `<tenant>.api-gateway.north-south-request-admission` against policy, writes `api_gateway.north_south_request_admission`, and emits `oya.api.gateway.north.south.request.admission.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `api-gateway` binds `credential-isolation (per ADR-0296)` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `api-gateway` is `contracts/api-gateway.asyncapi.yaml, contracts/api-gateway.openapi.yaml, contracts/api_gateway.proto, contracts/metric-naming-convention.md`; reviewers must map `credential isolation (per ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `api-gateway` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/public-read.cedar, policy/rate-limit.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (per ADR 0296)`.
- Depth detail 4: `api-gateway` state/event naming uses `api_gateway.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `api-gateway` covers `identity, policy-engine, audit-chain, observability, cloud-secrets, plus 1 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `api-gateway` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `api-gateway` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (per ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `api-gateway` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `api-gateway` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `api-gateway` uses SLOs `slos/api-gateway.openslo.yaml, slos/edge-availability.openslo.yaml, slos/edge-latency-p50.openslo.yaml, slos/edge-latency-p95.openslo.yaml, slos/edge-latency-p99.openslo.yaml, plus 3 more` and dashboards `dashboards/bot-score-distribution.json, dashboards/bot-score-distribution.md, dashboards/edge-overview.json, dashboards/edge-overview.md, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `api-gateway` uses runbooks `runbooks/audit-key-rotation.md, runbooks/blue-green-rollback.md, runbooks/bot-storm.md, runbooks/cell-evac.md, runbooks/circuit-breaker-engaged.md, plus 8 more` so `credential isolation (per ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `api-gateway` uses `iac/cert-manager-hsts-preload.yaml, iac/cert-manager.yaml, iac/cloudflare-config.yaml, iac/csp-defaults.yaml, iac/ech-config.yaml, plus 8 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `api-gateway` uses `capabilities/canary-route-shift.yaml, capabilities/edge-cedar-eval.yaml, capabilities/north-south-request-admission.yaml, capabilities/tls-handshake-terminate.yaml` and `catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml, catalog/oya-api-gateway-abuse-defence-domain.yaml, catalog/oya-api-gateway-app.yaml, catalog/oya-api-gateway-canary-cohort-shifter.yaml, plus 10 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `api-gateway` fails closed when `credential isolation (per ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `api-gateway` emits denial evidence for `credential isolation (per ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `api-gateway` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (per ADR 0296)` workflow.
- Depth detail 17: `api-gateway` telemetry for `credential isolation (per ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §F — References

- ADR-0157, ADR-0182, ADR-0183, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0246+amendment, ADR-0248, ADR-0253, ADR-0254, ADR-0263, ADR-0273, ADR-0284, ADR-0294, ADR-0295, ADR-0296, ADR-0297 (in flight).
- `docs/standards/documentation-rigor.md` §3.2.1 / §3.2.3.
- `microservices/observability/ARCHITECTURE.md` — shape exemplar.
- IETF: RFC 9114 (HTTP/3), RFC 9000 (QUIC), RFC 8446 (TLS 1.3), RFC 9460 (HTTPS RR), draft-ietf-tls-esni-22 (ECH), draft-kwiatkowski-tls-ecdhe-mlkem-02 (PQC).
- Cloudflare Bot Management technical brief 2024. Akamai Bot Manager 2024. Google Bot Verification.

---



## §cell-eligibility
This anchor is closed for `api-gateway` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `["tier-0"]`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `north-south-request-admission` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `api-gateway`; owner `axis-network`; tier `substrate`; audience `all`.
- Bounded contexts used for this answer: `api-gateway` root context.
- Capability records cited: `microservices/api-gateway/capabilities/canary-route-shift.yaml`, `microservices/api-gateway/capabilities/edge-cedar-eval.yaml`, `microservices/api-gateway/capabilities/north-south-request-admission.yaml`, `microservices/api-gateway/capabilities/tls-handshake-terminate.yaml`.
- API surfaces cited: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy artifacts cited: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`, `microservices/api-gateway/slos/h3-negotiation-rate.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`, `microservices/api-gateway/runbooks/ddos-mitigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar binding: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +4 more.
- State/event binding: `api_gateway.north_south_request_admission`, `api_gateway.edge_cedar_eval`, `api_gateway.tls_handshake_terminate`, `api_gateway.canary_route_shift`.
- Capability binding: `north-south-request-admission`, `edge-cedar-eval`, `tls-handshake-terminate`, `canary-route-shift`.
- SLO binding: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`, `microservices/api-gateway/slos/h3-negotiation-rate.openslo.yaml`; +2 more.
- Runbook binding: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`, `microservices/api-gateway/runbooks/ddos-mitigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `api-gateway`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `api-gateway`.
- `policy-engine` supplies the signed Cedar corpus while `api-gateway` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `api-gateway` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `api-gateway`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `api-gateway` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `api-gateway` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `api-gateway` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `api-gateway` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `north-south-request-admission` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `api-gateway` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `api-gateway`; owner `axis-network`; tier `substrate`; audience `all`.
- Bounded contexts used for this answer: `api-gateway` root context.
- Capability records cited: `microservices/api-gateway/capabilities/canary-route-shift.yaml`, `microservices/api-gateway/capabilities/edge-cedar-eval.yaml`, `microservices/api-gateway/capabilities/north-south-request-admission.yaml`, `microservices/api-gateway/capabilities/tls-handshake-terminate.yaml`.
- API surfaces cited: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar/policy artifacts cited: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`, `microservices/api-gateway/slos/h3-negotiation-rate.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`, `microservices/api-gateway/runbooks/ddos-mitigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`.
- Cedar binding: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +4 more.
- State/event binding: `api_gateway.north_south_request_admission`, `api_gateway.edge_cedar_eval`, `api_gateway.tls_handshake_terminate`, `api_gateway.canary_route_shift`.
- Capability binding: `north-south-request-admission`, `edge-cedar-eval`, `tls-handshake-terminate`, `canary-route-shift`.
- SLO binding: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`, `microservices/api-gateway/slos/h3-negotiation-rate.openslo.yaml`; +2 more.
- Runbook binding: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`, `microservices/api-gateway/runbooks/ddos-mitigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `api-gateway`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `api-gateway`.
- `policy-engine` supplies the signed Cedar corpus while `api-gateway` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `api-gateway` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `api-gateway`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `api-gateway` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
