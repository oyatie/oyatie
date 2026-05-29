---
doc_class: FAQ
microservice: api-gateway
persona: edge-engineer + sre
date: 2026-05-20
doc_status: published
---

# Edge Engineer FAQ

## Why Envoy and not nginx or HAProxy?

Per ADR-0253 + ADR-0131. Envoy is selected because: (a) native HTTP/3+QUIC support without the bolt-on nginx-quic patch; (b) xDS dynamic config push without process restart; (c) first-class gRPC routing + stream-level filters; (d) Wasm filter chain (per IP-013) for abuse-defence; (e) better observability surface (per-route metrics + access-log filter). nginx has a larger ecosystem but its config model is reload-based and its HTTP/3 maturity lags. HAProxy is competitive on raw L4 but its L7 + xDS story is weaker.

## Why HTTP/3+QUIC by default — what's the rollback story?

Per ADR-0253. HTTP/3 reduces head-of-line blocking + cuts handshake RTT to 1 (vs 2-3 for TLS 1.3 over TCP). Performance win: edge p99 drops ~ 15 % vs HTTP/2. Rollback: HTTP/2 + HTTP/1.1 are listeners alongside H3 in the same Envoy. Clients negotiate via Alt-Svc; if QUIC is blocked by middleboxes (some corporate firewalls), clients fall back to H2 automatically. We monitor `h3-fallback-rate`; if a tenant sees > 5 % fallback, we file a tenant-bound investigation (could be tenant network policy or client-runtime bug).

## A tenant says "WAF is blocking my traffic." What do I check?

In order:

1. Confirm via `oya api-gateway waf incidents --tenant <id> --since 1h`.
2. Identify the rule via `waf inspect-request`. Most false-positives come from CRS rules 942xxx (SQLi) and 941xxx (XSS) when the tenant's API uses SQL-like or HTML-like body content.
3. Check the body sample for legitimate-but-noisy patterns: SQL DSLs, GraphQL queries with single quotes, JSON-with-embedded-script-text.
4. Author a Cedar exception (per `oya api-gateway waf exception add`).
5. Validate via re-request.

If the WAF block is correct (the request IS hostile), the runbook `runbooks/waf-attack-confirmed.md` covers tenant notification + escalation.

## When does a tenant need a dedicated mTLS root vs the shared root?

Per ADR-0329 + ADR-0330 + ADR-0331, use tenant_class rather than retired feature gates. paid tenant_class tenants can request per-tenant certificates; dedicated mTLS roots are controlled by Cedar and compliance-pack policy.

A tenant needs dedicated root if: (a) they have hardware-key clients (smart cards, YubiKeys) where the device cert needs to chain to a specific tenant root; (b) they have regulatory pressure to demonstrate cert-issuance isolation (some pack-us-financial customers); (c) they run their own CA and need cross-signing.

## Why is rate-limit a token-bucket instead of fixed-window or sliding-window?

Per IP-009 + IP-010. Token-bucket handles bursts naturally (the bucket fills at the sustained rate; spikes consume tokens up to burst envelope). Fixed-window has the boundary-effect problem (2× burst at boundary). Sliding-window is correct but expensive (per-request lookup against a sorted set). Token-bucket gives us: (a) configurable burst envelope per tenant_class policy; (b) O(1) per request decrement; (c) clear refund semantics on circuit-breaker-tripped upstream (we refund the token if we didn't actually hit upstream).

## A tenant exceeds rate-limit but the request gets through. Why?

Possible causes:

1. The bucket has tokens from previous low-traffic period; the bucket refills + drains, not a hard reset.
2. The tenant's rate-limit is per-listener; if their traffic split across 2 PoPs, each PoP has its own bucket. (paid tenant_class configurations can aggregate via Valkey-cluster cross-replicated state; demo_trial configurations use cap-bounded per-PoP buckets.)
3. The request is below the burst envelope, not the sustained envelope. Burst envelope can absorb large spikes briefly.

If the tenant insists rate-limit is misbehaving, query `oya api-gateway rate-limit state --tenant <id>` to see the current bucket level + last-N-decrement-events.

## Why is the WAF Cedar-evaluated and not just CRS-based?

Per IP-012 + IP-013. CRS rules are signature-based; they catch known attack patterns. Cedar evaluations let us encode tenant-policy bounds: e.g., "tenant `pack-us-healthcare` MUST deny requests with `X-PHI-Indicator: 1` from non-VPN-source IPs"; or "tenant `pack-kr-pipa` MUST deny requests with `data-class=PII` from non-KR-CIDR sources". CRS handles the signature layer; Cedar handles the policy layer. Both fire; combined verdict gates the request.

## How does the gateway propagate `X-Oya-Tenant-Id`?

Per ADR-0244 + IP-002. The gateway extracts tenant-id from: (a) JWT claim `tenant_id`; (b) SNI server-name (if the tenant has a dedicated listener); (c) `X-Oya-Tenant-Id` header (only honored from trusted internal callers). The extracted id is propagated to upstream as `X-Oya-Tenant-Id` (header) + `tenant-id` (gRPC metadata) + audit-event envelope.

If tenant-id is inconsistent (header says A but JWT claims B), the gateway returns 400 + emits `tenant-id-mismatch` event for fraud-investigation.

## When do we trip the honeypot-route-manager?

Per IP-018. We expose a set of "honeypot" routes (`/admin/db-dump`, `/.env`, `/wp-admin/login.php`, `/api/internal/secrets`). Hits on these = signal of automated scanning. The manager:

1. Logs the request with IP + UA + JWT claims (if any).
2. Marks the source IP for elevated rate-limit (1 RPS for 24 h).
3. Tags the JWT principal (if any) for fraud review.
4. Emits `honeypot_hit` event.

A spike in honeypot hits = active scan; runbook `runbooks/honeypot-hit.md` covers the response.

## How does XDS push avoid race-conditions on listener config?

Per IP-002 + IP-015. The XDS control-plane:

1. Versions each config push (`v2026-05-20-1500`).
2. Applies the new config to Envoy nodes round-robin, one node at a time.
3. Each Envoy ACKs the push; if any Envoy NACKs (config invalid), the push rolls back across the whole tenant.
4. The canary-cohort-shifter ensures the new config affects only X % of traffic at first.

Race conditions are bounded: at most one Envoy is on the new config while siblings are on the old; tenant requests routing to that Envoy briefly see the new behaviour; others see old. The shifter watches SLOs to halt if behaviour diverges.

## How does the gateway handle JWT expiration during a long-lived gRPC stream?

The JWT is validated at stream-establishment. For long-lived streams (> 30 min), the gateway emits a `jwt_renewal_required` envelope; the client SHOULD refresh + re-establish. If the JWT expires mid-stream and the client hasn't renewed, the gateway terminates the stream with `unauthenticated` after a 5-minute grace.

This avoids holding open streams with stale credentials while not killing long streams immediately on expiration.
