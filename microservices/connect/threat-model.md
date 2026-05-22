---
microservice: connect
doc_class: ThreatModel
date: 2026-05-20
owner_team: axis-integration + ops-security
status: Accepted
related_adrs: [ADR-0243, ADR-0244, ADR-0263, ADR-0295, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connect/PRD.md
  - microservices/connect/policy/abuse-defence.cedar
inbound_citations: [microservices/connect/ARCHITECTURE.md]
review_cadence: semiannually + on every BC addition
doc_status: published
---

# Threat Model — connect (Integration Substrate)

STRIDE-per-BC threat model. Each row: threat → control → audit signal. Hyperscaler precedent for adversary capability: bot farms (residential proxy networks), credential-stuffing rings (combolist databases), nation-state APT (vendor-API impersonation), insider abuse (operator pivots into tenant data).

## Trust boundaries

```
[Internet] → [Edge / Cloudflare-equivalent + JA4 + WAF + bot-mgmt]
            ↓
[oyatie public surfaces: catalog browse, oauth-callback, webhook-receive]
            ↓
[mTLS / SPIFFE workload identity (ADR-0295)]
            ↓
[internal: connector-adapter-worker, dlq-replay-worker]
            ↓
[OpenBao (credentials), Postgres (state), Mimir/Loki (telemetry), workflow-engine (downstream)]
```

## connector-catalog BC

| Threat (STRIDE) | Scenario | Control | Audit signal |
|---|---|---|---|
| T-S-01 (Spoof) | Attacker impersonates an MPO publisher | Marketplace publisher identity via WebAuthn + signed adapter binaries | `MarketplacePublisherAuthFailed` |
| T-T-01 (Tamper) | Attacker mutates listed connector to embed malicious code | Adapter binaries are cosign-signed (sigstore); Cedar gate verifies signature on load | `AdapterSignatureVerifyFailed` |
| T-R-01 (Repudiation) | MPO denies publishing a malicious adapter | Audit chain: every publish is `MarketplaceConnectorPublished` event signed by publisher's key | `MarketplaceConnectorPublished` |
| T-I-01 (Info disclosure) | Catalog scrape reveals tenant-private connectors | `connector-authorization.cedar` filters by `audience_type` + `compliance_packs` | `CatalogQueryDenied` |
| T-D-01 (Denial of service) | Bot scrapes catalog at high RPS | Rate-limit per-IP/fingerprint; CAPTCHA on bot-score > threshold; UX-floor preserved | `AbuseDefenceChallengeIssued` |
| T-E-01 (Elevation) | Tenant principal reads other tenant's published private connectors | Default-deny Cedar; `tenant_id` filter on every query | `CatalogQueryDenied` |

## oauth-broker BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-S-02 | Attacker steals OAuth state nonce, hijacks callback | State nonce is HMAC-signed + bound to session; 10min TTL | `OAuthCallbackStateInvalid` |
| T-T-02 | Attacker tampers with redirect_uri to exfiltrate code | Strict `redirect_uri` allow-list per OAuth client; Cedar gate | `OAuthRedirectUriRejected` |
| T-R-02 | TAB denies provisioning a malicious OAuth client | Step-up auth (WebAuthn) on provision; audit `ProviderCredentialProvisioned` | `ProviderCredentialProvisioned` |
| T-I-02 | Attacker reads tenant's refresh tokens | Refresh tokens never leave OpenBao; only access tokens (≤60s TTL) reach adapter workers (ADR-0296) | `OAuthTokenAccessAttempt` |
| T-D-02 | Mass OAuth callback flood exhausts broker | Per-tenant rate-limit; circuit-breaker on Salesforce/Google etc. backends | `OAuthBrokerCircuitOpen` |
| T-E-02 | Stolen access token used outside grant's scope | Cedar gate validates `scopes[]` on every action invocation | `OAuthScopeViolation` |

## webhook-receiver BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-S-03 | Attacker forges Shopify webhook payload | HMAC-SHA256 verify with per-tenant signing secret; constant-time compare | `WebhookSignatureVerifyFailed` |
| T-T-03 | Attacker replays a captured webhook | Replay-window ≤5min (timestamp check); idempotency-key dedup (24h Valkey TTL) | `WebhookReplayBlocked` |
| T-R-03 | Vendor disputes webhook delivery | Audit chain: every receive emits `WebhookReceived` with payload digest | `WebhookReceived` |
| T-I-03 | Attacker enumerates webhook URLs to find active tenants | URLs include per-tenant DNS subdomain (ADR-0273); no enumeration via TLS SNI (ECH per ADR-0253) | (passive: ECH masks SNI) |
| T-D-03 | Burst webhook traffic causes backend OOM | Backpressure: 429 with `Retry-After` if queue depth > threshold; DLQ for verified-but-undeliverable | `WebhookBackpressureEngaged` |
| T-E-03 | Spoofed webhook triggers privileged workflow | Workflow-engine validates webhook-source vs wiring's expected connector | `WorkflowSourceMismatch` |

## signature-verification BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-S-04 | Timing attack on HMAC compare | Constant-time compare (`subtle::ConstantTimeEq`) | (preventive) |
| T-T-04 | Attacker downgrades signing alg (SHA1) | Strict allow-list of HMAC algorithms per vendor; SHA1 forbidden | `SignatureAlgRejected` |
| T-I-04 | Signing secret leaks via logs | Secret never logged; sidecar isolation per ADR-0296 | (preventive) |

## connector-adapter BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-S-05 | Adapter impersonates another tenant on outbound call | SPIFFE SVID + Cedar gate validates caller's `tenant_id` on every action | `ConnectorActionDeniedTenantMismatch` |
| T-T-05 | Adapter tampers with vendor response before returning to caller | Response canonicalization in isolated Kata pod; output schema-validated | `AdapterResponseSchemaInvalid` |
| T-I-05 | Adapter leaks one tenant's data into another's logs | Per-tenant log routing via OTel resource attributes; Cedar gate on log read | `LogAccessDenied` |
| T-D-05 | Salesforce 5xx cascades into other connector latency | Per-connector circuit-breaker (ADR-0145 §invariant-1); shuffle-sharding (ADR-0248) | `ConnectorCircuitOpen` |
| T-E-05 | Compromised adapter binary reads OpenBao secrets beyond its grant | Adapter runs in Kata sandbox with OpenBao token scoped to the adapter's grants only | `OpenBaoAccessDenied` |

## data-mapping BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-I-06 | Mapper persists PII field-mapping that leaks to non-PII downstream | Data-class tagging enforced on every mapped field; downstream consumers honor tags | `DataClassViolation` |
| T-T-06 | Vendor schema-drift goes undetected, causing silent data corruption | Hourly schema-diff worker; emit `SchemaDriftDetected` on change | `SchemaDriftDetected` |

## retry-and-DLQ BC

| Threat | Scenario | Control | Audit signal |
|---|---|---|---|
| T-D-07 | DLQ fills disk, blocking new entries | Per-tenant DLQ retention cap (default 7d, max 30d); disk-pressure alert | `DLQRetentionCap` |
| T-E-07 | DLQ replay double-invokes vendor (double-charge) | Idempotency-key preserved on replay; vendor sees same key → idempotent | `DLQEntryReplayed` |
| T-I-07 | DLQ payloads contain PII visible to ops | DLQ stored encrypted-at-rest; payload digest only in operator UI; full payload access requires step-up auth | `DLQPayloadAccessAttempt` |

## Cross-cutting

| Threat | Control |
|---|---|
| Insider abuse | Step-up auth (`docs/standards/step-up-auth-classes.md`) on every privileged action; audit chain Merkle-sealed |
| Supply chain | Cosign keyless OIDC for all artifacts (sigstore + Fulcio); SBOM per release (CycloneDX) |
| Cryptographic regression | PQC hybrid `X25519MLKEM768` advertised; non-PQ clients fall through gracefully (ADR-0253) |
| Kill-switch | `kubectl annotate ns connect oya.kill-switch=true` per ADR-0295 |

## Severity table

| Severity | Threats | Mitigation status |
|---|---|---|
| Critical | T-S-03, T-T-03, T-I-02, T-E-05 | Mitigated by HMAC + OpenBao isolation + Kata sandbox + Cedar |
| High | T-S-02, T-T-02, T-I-01, T-D-02, T-D-05, T-E-02 | Mitigated by Cedar + rate-limit + circuit-breaker |
| Medium | T-D-01, T-I-03, T-D-03, T-I-06, T-T-06 | Mitigated by abuse-defence + ECH + schema-drift detection |
| Low | T-T-04, T-I-04, T-D-07 | Mitigated by allow-list + sidecar + retention cap |

## References

- ADR-0297 abuse-defence baseline (the canonical anti-bot/spoof/scrape taxonomy)
- ADR-0263 audit-event-class emission contract
- ADR-0295 SPIFFE workload identity + kill-switch
- ADR-0296 library-first credential sidecar
- docs/standards/documentation-rigor.md §3.2.3 abuse-defence baseline

## Retirement-coordination addendum

This document supersedes the prior umbrella-retirement threat model (pre-2026-05-20). The umbrella-retirement scope is retained in `RETIREMENT-PLAN.md` + `IP-001-connect-retirement-design-readiness.md` for cross-reference; the substrate scope above is the live posture.
