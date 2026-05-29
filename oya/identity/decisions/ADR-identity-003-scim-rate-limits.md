---
id: ADR-identity-003
scope: microservice
microservice: identity
status: Accepted
date: 2026-05-18
owner: axis-identity
related: [ADR-0190, ADR-0178, ADR-0191]
---

# ADR-identity-003 — SCIM rate limits per tenant

## Decision

Per-tenant SCIM bearer rate limits (enforced at origin tier per ADR-0191):

| Endpoint | Default | Burst |
|---|---|---|
| GET /Users (list) | 50 rps | 3× for 10s |
| POST /Users | 100 rps | 10× for 60s (bulk import) |
| PATCH /Users | 200 rps | 5× for 10s |
| DELETE /Users | 50 rps | n/a |
| GET /Groups | 50 rps | n/a |
| POST /Groups | 50 rps | n/a |
| PATCH /Groups | 100 rps | 3× for 10s |

Enterprise-tier tenants negotiate higher quotas via procurement; raised in tenant-edge-policy registry.

## Consequences

- Bulk import (10× burst) supports tenant onboarding scenarios.
- Per-bearer rate-limit isolates noisy tenants per shuffle-sharding posture.
