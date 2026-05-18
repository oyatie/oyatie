---
id: ADR-identity-004
scope: microservice
microservice: identity
status: Accepted
date: 2026-05-18
owner: axis-identity + ops-security
related: [ADR-0189]
---

# ADR-identity-004 — Session class tiers + max session age

## Decision

| ACR class | Access token TTL | Refresh token TTL | Idle timeout | Session max age |
|---|---|---|---|---|
| routine | 15min | 90d | 4h | 90d |
| elevated | 15min | 30d | 1h | 4h |
| sensitive | 15min | 24h | 15min | 1h |
| critical | 5min | 1h | 5min | 15min |

Re-authentication (full Passkey ceremony) required at session-max-age boundary regardless of refresh-token validity.

## Consequences

- Critical-class sessions impose 15min budget on operator actions; runbook authors plan accordingly.
- Refresh-token TTL bounded per ACR (not just per overall session).
