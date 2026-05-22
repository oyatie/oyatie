# api-gateway — Compliance + ADR-adherence

**Authority:** `docs/standards/documentation-rigor.md` §3.2.1 (per-µservice ADR-adherence checklist) + §3.2.3 (abuse-defence baseline).
**Last reviewed:** 2026-05-20.

## A — ADR-adherence matrix (§3.2.1, 28 rows)

| # | ADR | Question | Answer |
|---:|---|---|---|
| 1 | ADR-0242 (oyatie-is-a-tenant) | Principals? | `oyatie.api_gateway.envoy@cell-N`, `oyatie.api_gateway.controlplane@cell-N`, `oyatie.api_gateway.policy-loader@cell-N`, `oyatie.api_gateway.audit-emitter@cell-N`. Tenant-scoped callers: every product µservice; every external API client. |
| 2 | ADR-0243 (Cedar universal gate) | Cedar fragments? | `policy/route-authorization.cedar`, `policy/rate-limit.cedar`, `policy/abuse-defence.cedar`, `policy/tls-policy.cedar`, `policy/sov-cloud-overlay.cedar`, `policy/tenant-scope.cedar` (default-deny baseline), `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/public-read.cedar`. |
| 3 | ADR-0244 (tenant scoping) | Tables/events/rows with `tenant_id`? | No tables (stateless data-plane). All audit events carry `tenant_id`; all rate-limit bucket keys salted with `tenant_id`. `audience_type ∈ {anonymous, b2c, b2b, partner, machine}`. `provider_credential_mode = tenant-byok`. |
| 4 | ADR-0245 (substrate vs product) | Substrate or product? | Substrate. Every product µservice depends on it. It depends on: identity, policy-engine (control-plane only), audit-chain, observability, cloud-secrets, spire. |
| 5 | ADR-0246 + amendment | Policy library-first? | Yes. `policy_evaluation_mode = caller-side-library` using `oya-shared-policy-eval`. Fragments loaded push-based from policy-engine ledger ≤30s freshness. |
| 6 | ADR-0247 (self-modification) | Self-modification artifacts? | No — gateway is not a Foundry-touching µservice. Self-modification of Cedar fragments handled by policy-engine µservice. |
| 7 | ADR-0248 (cellular) | Cell tier? | Tier-0 (edge). Deployed in every cell of every region. Shuffle-sharded by tenant. |
| 8 | ADR-0249 (marketplace) | Marketplace surfaces? | No direct marketplace surfaces; partner tenant_class authentication routes are surfaced as marketplace ToS-of-service crawlers per the abuse-defence anti-scrape control. |
| 9 | ADR-0250 (build-ahead-of-certification) | Day-one cert? | Day-one ready for FedRAMP High, FIPS 140-3, IL5, KR-CSAP, EU-sovereign, CN-PIPL, HIPAA, PCI DSS Level 1, SOC 2 Type II. |
| 10 | ADR-0251 + CN-PIPL pack | Pack overlays? | `pack-us`, `pack-eu`, `pack-kr`, `pack-cn-pipl-2021`, `pack-us-healthcare`, `pack-fedramp-high`, `pack-il-5-6`, `pack-ksa-pdpl`, `pack-ae-pdpl`. See §pack-overlay-roster below. |
| 11 | ADR-0252 (HLC/TrueTime) | Time tier? | HLC default. TrueTime opt-in via Spanner backbone for cross-region rate-limit causality (when needed). |
| 12 | ADR-0253 (HTTP/3 + ECH + PQC) | h3 default + fallback + ECH + PQC? | See §transport in `ARCHITECTURE.md`. Yes for all. |
| 13 | ADR-0254 (deployment) | K8s + Cloud Hypervisor + Kata? | Envoy in Kata Containers (Cloud Hypervisor) on K8s. Bot-management as Wasm filter in Envoy. |
| 14 | ADR-0255 + amendment (Intelligence) | Calls Intelligence? | No on hot path. Bot-score ML runs in bot-management subsystem (separate). `intelligence_dispatch_mode = none`. |
| 15 | ADR-0257 + amendment (Ontology) | Reads Ontology? | No. `ontology_read_mode = none`. |
| 16 | ADR-0258 (versioning) | SemVer? | SemVer 2.0; contracts at v1; deprecation cadence 12mo notice + 6mo sunset. |
| 17 | ADR-0263 (observability emission) | Audit classes? | 15 classes listed in `ARCHITECTURE.md §B-13`. Cardinality budgets: `tenant_id` cap 200k, `route_id` cap 5k, `code` cap 60. Trace span shape documented. |
| 18 | ADR-0272 (cookie consent) | Per-purpose consent? | Yes. Consent surface served by `oyatie-consent-µservice`; gateway forwards `X-Oya-Consent-State` header. |
| 19 | ADR-0273 (DKIM/SPF/DMARC) | Mail-emitting? | No — gateway emits no mail. The gateway *terminates* mail-protocol routes (SMTP-API) but does not originate mail. Cross-ref only. |
| 20 | ADR-0276 (backup portability) | Portable export? | Audit events portable in JSON-LD per ADR-0276. Per-tenant exports via `backfill-replay.md`. |
| 21 | ADR-0280 (substrate-of-substrate) | Dependency DAG? | `substrate_dependencies: [identity, policy-engine, audit-chain, observability, cloud-secrets, spire]`. No backward edges. |
| 22 | ADR-0284 (platform-owner indirection) | Hard-coded `oyatie`? | No hard-coded strings. Brand surface served from per-cell config map. |
| 23 | ADR-0292 (minor user) | Consumer-facing? | Yes — minor-protection forwarding headers per pack; COPPA <13 refusal handled at downstream consumer-product tier; gateway just stamps `X-Oya-Minor-Class`. |
| 24 | ADR-0293 (meta-trust-root) | Foundry-touching? | No. |
| 25 | ADR-0294 (Cedar fragment soak) | ≥60s soak? | Yes. Per-fragment `fragment_soak_seconds: 60` minimum; emergency override via `runbooks/cedar-fragment-emergency-rollback.md`. |
| 26 | ADR-0295 (SPIFFE + kill-switch) | Workload identity? | Yes. SPIFFE SVID rotation every 1h. Kill-switch wired per cell. |
| 27 | ADR-0296 (credential sidecar) | Library-first sidecar? | Yes. TLS private keys + SPIFFE bundle delivered by sidecar; data-plane process never sees OpenBao token. |
| 28 | ADR-0297 (abuse-defence baseline) — in flight | Anti-bot + anti-spoof + anti-scrape wired? | Yes — see §abuse-defence-roster below. Cedar in `policy/abuse-defence.cedar`. WAF in `iac/edge-waf.yaml`. |

## B — Abuse-defence roster (§3.2.3)

### B-1. Anti-bot controls

| # | Control | Implementation |
|---:|---|---|
| 1 | Edge rate-limiting | Envoy token-bucket; per-IP/fingerprint/tenant/route; per `iac/envoy-config.yaml` |
| 2 | Behavioural fingerprinting | JA4/JA4+/JA3 + HTTP/2-3 frame fingerprint; per `iac/edge-waf.yaml` |
| 3 | Bot-management with ML scoring | In-house model in Wasm filter; Cloudflare Bot Management at edge front; per `iac/cloudflare-config.yaml` |
| 4 | CAPTCHA-on-suspicion | hCaptcha + Turnstile federation; adaptive trigger via `policy/abuse-defence.cedar` |
| 5 | Device attestation | App Attest / Play Integrity / WebAuthn origin-binding; per upstream identity µservice |
| 6 | Stolen-credential check | HIBP + internal credential-stuffing-detector at identity µservice (gateway forwards) |
| 7 | Per-action quota gates | Cedar in `policy/rate-limit.cedar` |
| 8 | Honeypot routes + canary payloads | Routes in `iac/envoy-config.yaml`; canary IDs minted by abuse-defence µservice |

### B-2. Anti-spoof controls

| # | Control | Implementation |
|---:|---|---|
| 1 | Email anti-spoof | N/A at gateway (cross-ref ADR-0273 for mail-emitting µservices) |
| 2 | Domain anti-spoof / cert pinning | TLS 1.3 strict + CT-required + HSTS preload; native apps cert-pin via `iac/cert-manager-hsts-preload.yaml` |
| 3 | Identity anti-spoof | Step-up auth per `docs/standards/step-up-auth-classes.md`; WebAuthn passkeys; gateway forwards to identity |
| 4 | Session anti-spoof | HMAC-signed `__Host-` cookies; SameSite=Strict; rotating session-id |
| 5 | Payload anti-spoof | Webhook HMAC verify per ADR-0273-style; mTLS / signed JWT for machine clients |
| 6 | Audit-trail anti-spoof | Per ADR-0263; Merkle-sealed per ADR-0028; per-µservice signing key in sidecar |
| 7 | Webhook anti-spoof | Inbound webhook HMAC; replay window ≤5min; idempotency-key required |
| 8 | Caller anti-spoof (workload identity) | SPIFFE SVID per ADR-0295; Cedar gate on caller |

### B-3. Anti-scrape controls

| # | Control | Implementation |
|---:|---|---|
| 1 | Rate-limit per-tenant + per-fingerprint | Per `iac/envoy-config.yaml` + `policy/rate-limit.cedar` |
| 2 | Pattern-anomaly detection | Sequential-ID + alphabetical-pagination + high-page-depth detector in Wasm filter |
| 3 | robots.txt + Sitemaps + crawl-delay | Per-tenant robots.txt served from edge cache |
| 4 | Paid-API tier for legitimate scrapers | `Partner-API` route class with ToS-of-service acceptance check |
| 5 | Content fingerprinting / per-user watermarking | Forwarded to downstream µservices; gateway does not watermark |
| 6 | Adaptive challenge on scrape-pattern | `policy/abuse-defence.cedar` permit-with-challenge-class |
| 7 | Dynamic content rewriting | Forwarded to downstream µservices; gateway does not rewrite |
| 8 | Legal-channel registration | DMCA agent + abuse-report email surfaced via `https://abuse.oyatie.com/` |

## C — Pack-overlay roster

| Pack | Variation | Cedar overlay |
|---|---|---|
| pack-us | Default. | `sov-cloud-overlay.cedar` permit-default |
| pack-eu | EU-sovereign-cell routing; cross-border restricted. | `sov-cloud-overlay.cedar` permit-eu-only when `tenant.tier == "eu-sovereign"` |
| pack-kr | KR-CSAP-cell routing; PIPA Art. 28 cross-border consent | `sov-cloud-overlay.cedar` permit-kr-only |
| pack-cn-pipl-2021 | CN-sov-cell routing; PIPL Art. 38 assessment | `sov-cloud-overlay.cedar` permit-cn-only |
| pack-us-healthcare | HIPAA BAA; PHI never in audit body | `sov-cloud-overlay.cedar` + body-redaction policy |
| pack-fedramp-high | FedRAMP High control mapping; CT mandatory; PQC mandatory | `tls-policy.cedar` strict-pqc-required |
| pack-il-5-6 | IL5/6 sovereign cell; FIPS 140-3 crypto only | `tls-policy.cedar` fips-140-3 |
| pack-ksa-pdpl | KSA sov-cell; PDPL Art. 29 cross-border | `sov-cloud-overlay.cedar` permit-ksa-only |
| pack-ae-pdpl | UAE sov-cell; PDPL cross-border per Federal Decree-Law 45 of 2021 | `sov-cloud-overlay.cedar` permit-ae-only |

## D — Day-one cert readiness

| Standard | Status | Evidence |
|---|---|---|
| SOC 2 Type II | Day-one ready | Audit-chain Merkle-sealed; access controls Cedar; encryption in transit + at rest |
| ISO 27001 | Day-one ready | Threat model + control roster + access management |
| PCI DSS Level 1 | Day-one ready | TLS 1.3 + PQC; CHD never in audit body; tokenisation at payments µservice |
| HIPAA | Day-one ready | BAA template; PHI redaction; access controls per §164.312 |
| FedRAMP High | Day-one ready | Control mapping per `iac/fedramp-control-map.yaml`; CT + PQC mandatory |
| KR-CSAP | Day-one ready | Sov-cell-kr routing; PIPA Art. 28 controls |
| EU-sovereign | Day-one ready | Sov-cell-eu routing; cross-border restricted |
| CN-PIPL-2021 | Day-one ready | Sov-cell-cn routing; PIPL Art. 38 assessment |
| IL5/6 | Day-one ready | Sov-cell-il5/6 routing; FIPS 140-3 |

## E — Observability

Per ADR-0263. Audit-event classes (15) listed in `ARCHITECTURE.md §B-13`. Metrics cardinality budgets:

| Metric | Cardinality cap |
|---|---|
| `oya_api_gateway_requests_total{tenant_id, route_id, code}` | `tenant_id` 200k × `route_id` 5k × `code` 60 = 60M (over budget; aggregate `code` to `code_class` 6) |
| `oya_api_gateway_latency_seconds_bucket{tenant_id, route_id, le}` | tenant_id 200k × route_id 5k × le 12 = 12M (over budget; downsample `tenant_id` to tenant-tier) |
| `oya_api_gateway_tls_handshake_duration_seconds_bucket{tls_version, cipher, le}` | 3 × 5 × 12 = 180 (well under) |
| `oya_api_gateway_bot_score_bucket{tenant_id, le}` | tenant_id-tier 5 × le 12 = 60 |

Aggregation is applied at the OTel collector tier (per `microservices/observability/IP-022-otel-to-clickhouse-bridge.md`).

## F — Self-modification

N/A — gateway is not Foundry-touching.

## G — Bootstrap trust chain

Per ADR-0295. Bootstrap CI signs the SPIRE root; kill-switch wired per cell at the SPIFFE-bundle-loader.

## H — Vendor dependency (defense in depth)

Multi-CDN-vendor: Cloudflare + Fastly + sov-cell-internal edge. Multi-DNS-vendor: Cloudflare DNS + NS1 + sov-cell DNS. RPKI from APNIC + RIPE + ARIN routes. If any vendor degrades, traffic shifts within 60s via `runbooks/cell-evac.md`.

## I — Email deliverability

N/A — gateway does not originate mail. Mail-protocol routes terminate at downstream comms-email µservice.

## J — Platform-owner indirection

No hard-coded `oyatie` strings. Per-cell config map sources brand identity. Whitelabel tenants get per-tenant brand overlay.

## K — Minor protection

The gateway stamps `X-Oya-Minor-Class` header forwarded to downstream consumer µservices. COPPA <13 refusal + KOSA 14-17 tier + EU age-verification handled downstream (the relevant consumer-product µservice + identity µservice). The gateway does not collect age; it only forwards the class as set by identity.

## L — References

- `docs/standards/documentation-rigor.md` §3.2.1 + §3.2.3
- `microservices/api-gateway/ARCHITECTURE.md`
- `microservices/api-gateway/threat-model.md`
- `microservices/api-gateway/dpia.md`
- ADRs listed in §A.

---



## §day-one-cert-readiness
This anchor is closed for `api-gateway` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `api-gateway` covers packs `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`, `fedramp-high`; +6 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/api-gateway/slos/api-gateway.openslo.yaml`, `microservices/api-gateway/slos/edge-availability.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p50.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p95.openslo.yaml`, `microservices/api-gateway/slos/edge-latency-p99.openslo.yaml`, `microservices/api-gateway/slos/h3-negotiation-rate.openslo.yaml`; +22 more.
- Example: `north-south-request-admission` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

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
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §pack-overlay-roster
This anchor is closed for `api-gateway` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `us`, `eu`, `kr`, `cn-pipl-2021`, `us-healthcare`, `fedramp-high`; +6 more.
- Pack overlays modify Cedar fragments `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +4 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `north-south-request-admission` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

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
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §platform-owner-indirection
This anchor is closed for `api-gateway` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `api-gateway` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`, `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`; +22 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `north-south-request-admission` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.api-gateway.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

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
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §detection-substrate-binding
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `api-gateway` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `north-south-request-admission` touches those data classes.
- Signal sources: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +20 more.
- Example event class: `oya.api.gateway.north.south.request.admission.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

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
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §investigation-binding
This anchor is closed for `api-gateway` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `api-gateway` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.api-gateway.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `north-south-request-admission` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `north-south-request-admission` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

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
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §insider-threat-controls
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `api-gateway` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`, `api_gateway.north_south_request_admission`, `api_gateway.edge_cedar_eval`; +2 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `api_gateway.north_south_request_admission` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

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
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §threat-intelligence-feeds
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `api-gateway` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/api-gateway/policy/abuse-defence.cedar`, `microservices/api-gateway/policy/auditor-scope.cedar`, `microservices/api-gateway/policy/ci-scope.cedar`, `microservices/api-gateway/policy/data-residency.md`, `microservices/api-gateway/policy/public-read.cedar`, `microservices/api-gateway/policy/rate-limit.cedar`; +16 more.
- Example: `north-south-request-admission` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

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
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §key-rotation-cadence
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.api-gateway` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/api-gateway/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/api-gateway/runbooks/audit-key-rotation.md`, `microservices/api-gateway/runbooks/blue-green-rollback.md`, `microservices/api-gateway/runbooks/bot-storm.md`, `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`, `microservices/api-gateway/runbooks/ddos-mitigation.md`; +6 more.
- Example: `north-south-request-admission` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

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
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §crypto-agility-plan
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `api-gateway` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`, `microservices/api-gateway/iac/cert-manager-hsts-preload.yaml`, `microservices/api-gateway/iac/cert-manager.yaml`; +10 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `north-south-request-admission` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

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
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §pentest-and-bounty-cadence
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `api-gateway` is in annual full-scope pentest and every major `north-south-request-admission` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/contracts/metric-naming-convention.md`, `microservices/api-gateway/iac/cert-manager-hsts-preload.yaml`, `microservices/api-gateway/iac/cert-manager.yaml`; +20 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `api-gateway` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

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
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §facility-controls
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `api-gateway` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `["tier-0"]` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `north-south-request-admission` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

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
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §supply-chain-risk
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `api-gateway` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/api-gateway/catalog/oya-api-gateway-abuse-defence-adapter-wasm.yaml`, `microservices/api-gateway/catalog/oya-api-gateway-abuse-defence-domain.yaml`, `microservices/api-gateway/catalog/oya-api-gateway-app.yaml`, `microservices/api-gateway/catalog/oya-api-gateway-canary-cohort-shifter.yaml`, `microservices/api-gateway/catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml`, `microservices/api-gateway/catalog/oya-api-gateway-rate-limit-domain.yaml`; +22 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `north-south-request-admission` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

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
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
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
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `api-gateway` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `north-south-request-admission` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `north-south-request-admission` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

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
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
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

## §data-classification
This anchor is closed for `api-gateway` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `api_gateway.north_south_request_admission`, `api_gateway.edge_cedar_eval`, `api_gateway.tls_handshake_terminate`, `api_gateway.canary_route_shift`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `north-south-request-admission` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

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
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
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
