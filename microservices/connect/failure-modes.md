---
microservice: connect
doc_class: FailureModes
date: 2026-05-20
owner_team: axis-integration + ops-sre-reliability
status: Accepted
related_adrs: [ADR-0145, ADR-0248, ADR-0263]
doc_status: published
---

# Failure Modes — connect (Integration Substrate)

Enumeration of ≥3 failure modes per BC + cross-cutting modes. Per documentation-rigor §1.1, every primitive enumerates failures + system behavior.

## connector-catalog

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Catalog index stale | ElasticSearch reindex lag > 5min | Catalog returns older data; banner "results may be stale"; SLO error budget burns on freshness | `CatalogStaleResponse` |
| Catalog index partition | ES cluster split-brain | Read-only mode; writes queued in PG outbox | `CatalogReadOnlyMode` |
| Catalog poison entry | Malformed MPO submission | Validation rejects at publish; catalog never serves poison | `CatalogValidationRejected` |

## oauth-broker

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Vendor OAuth endpoint down | Salesforce/Google/Microsoft OAuth 5xx | Circuit-breaker opens after 3 consecutive 5xxs; surface "Retry later" to TIE | `OAuthBrokerCircuitOpen` |
| Refresh token rotation race | Two concurrent token refreshes for same grant | Pessimistic lock in PG; second waits; both succeed eventually | (no audit; expected) |
| provider-credential BYOK client misconfigured (ADR-0255 §D-4) | TAB provisioned wrong client_secret | OAuth flow fails with `invalid_client`; TAB notified | `OAuthClientInvalid` |
| OpenBao access denied | Sidecar Vault token expired | Sidecar refreshes; if persistent, worker self-terminates → re-spawn | `OpenBaoTokenExpired` |

## webhook-receiver

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| HMAC verify fail | Forged/corrupted payload | 401 + audit; vendor sees rejection | `WebhookSignatureVerifyFailed` |
| Replay attack | Captured payload resubmitted within 24h | 401 (timestamp OR idempotency-key); audit | `WebhookReplayBlocked` |
| Backpressure | Queue depth > threshold | 429 + `Retry-After`; vendor retries per their policy | `WebhookBackpressureEngaged` |
| DNS resolution fail for `hooks.<tenant>.oyatie.app` | Per-tenant DNS misconfigured | Vendor sees DNS error; tenant notified via dashboard | `WebhookDNSResolveFailed` |
| Edge cell partition | Cilium L4 connectivity loss | DNS GSLB fails over to DR cell; <30s RTO | (mesh-level event) |

## signature-verification

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Algorithm downgrade attempt | Vendor uses SHA1 instead of SHA256 | Reject (allow-list); audit | `SignatureAlgRejected` |
| Constant-time compare bypass | (Hypothetical) compiler optimizes away constant-time | CI lane `oya-governance-constant-time-crypto` verifies compile output; FAIL blocks merge | (CI lane) |
| Per-tenant secret rotation race | Vendor sends webhook with old secret during rotation window | Both old and new secrets accepted during 5min rotation grace; audit each | `SignatureSecretRotationGrace` |

## connector-adapter

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Vendor 5xx cascade | Salesforce 5xx for >10min | Circuit-breaker opens; affected tenants surface; DLQ accumulates | `ConnectorCircuitOpen` |
| Vendor rate-limit (429) | Tenant exceeds Salesforce daily limit | Per-action 429 + `Retry-After`; tenant dashboard surfaces | `ConnectorRateLimitHit` |
| Adapter binary corruption | Cosign verify fails on load | Adapter refuses to load; tenant gets "adapter unavailable"; ops notified | `AdapterSignatureVerifyFailed` |
| Kata sandbox OOM | Adapter process exceeds memory limit | Kata kills process; worker pool spawns replacement; entry goes to DLQ | `AdapterSandboxOOM` |
| TLS connect timeout | Vendor TLS handshake > 10s | Action fails with `ConnectTimeout`; retry with backoff | `ConnectorActionFailed` |

## data-mapping

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Vendor schema drift (additive) | New field appears in vendor response | Auto-map if confident; flag for review otherwise | `SchemaDriftDetected` |
| Vendor schema drift (breaking) | Field deleted or renamed | Wiring auto-pauses; TIE notified | `SchemaDriftBreaking` |
| Mapper expression eval error | TIE writes invalid mapping expression | Reject at save-time (compile check); cannot save broken | (no audit; pre-empt) |

## retry-and-DLQ

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| DLQ disk full | Per-tenant retention cap reached | Oldest entries dropped; `DLQRetentionCap` audit; tenant notified | `DLQRetentionCap` |
| DLQ replay storm | TIE clicks "replay all" on 10k entries | Replay rate-limited; queue depth monitored | (no audit; rate-limit) |
| Idempotency-key collision | Vendor's idempotency-key reused across wirings | Vendor returns 409; replay rejects; audit | `DLQReplayConflict` |

## Cross-cutting

| Mode | Trigger | Behavior | Audit signal |
|---|---|---|---|
| Region outage | Full OCI region down | DR failover; <15min RTO; <60s RPO | `RegionFailoverInitiated` |
| Audit chain seal failure | Sealer cannot reach KMS | Worker queues unsealed; alarms; eventual seal on KMS recovery | `AuditSealDeferred` |
| Cedar fragment publish fail | Soak window violation (ADR-0294 <60s) | Publish rejected; CI lane fails | `CedarFragmentSoakViolation` |
| Kill-switch engaged | Operator triggered emergency stop | All connect activity halts within 30s | `KillSwitchEngaged` |
| Bot-net spike | DDoS-class traffic on webhook ingress | Bot-mgmt + WAF; rate-limit; CAPTCHA-on-suspicion; UX-floor preserved | `AbuseDefenceMassChallengeIssued` |

## References

- ADR-0145 inter-microservice communication (circuit-breaker semantics)
- ADR-0248 cellular architecture (region failover)
- ADR-0263 audit-event emission contract
