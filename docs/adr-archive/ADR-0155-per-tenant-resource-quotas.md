---
id: ADR-0155
status: Superseded
superseded_by: [ADR-702]
---

# ADR-0155: Per-Tenant Resource Quotas

- Status: Accepted
- Date: 2026-05-18
- Deciders: axis-tenancy, council-architecture
- Tier-A hyperscaler pattern: AWS SaaS Lens Tenant Isolation

## Context

Rate-limit alone (per-tenant rps cap) is not sufficient noisy-neighbor
protection. A tenant can exhaust:

- Memory (large request → allocator blow-up).
- Storage (writes filling the cell volume).
- Connection pool (long-lived sockets).
- Concurrent in-flight requests (slow-loris).

AWS's SaaS Lens prescribes per-tenant quotas on all four axes plus
rate-limit. oyatie has the tenancy µservice but no canonical quota
substrate.

## Decision

Adopt per-tenant quotas on five canonical axes (rate, concurrent,
memory, storage, connections) as MANDATORY across every µservice.

1. The canonical spec is
   `docs/standards/per-tenant-resource-quotas-canonical.md`.
2. The trait surface lives in `crates/oya-shared-tenant-quota-kernel/`.
3. The tenancy µservice OWNS canonical quota definitions; runtime
   µservices query it.
4. Exceeded quota → `429 Too Many Requests` with `Retry-After` plus
   `X-Tenant-Quota-Axis` + `X-Tenant-Quota-Limit` + `X-Tenant-Quota-Used`.
5. Cell-level isolation: per-tenant quota cannot bleed cells.

## Consequences

Positive:
- Hard noisy-neighbor isolation.
- Clear refusal contract for tenants.
- Cell capacity = sum of per-tenant quotas (predictable).

Negative:
- Per-µservice quota check on every request (low latency cost).
- Tenancy µservice becomes critical-path; needs caching.

## Alternatives considered

- Rate-limit only — REJECTED, memory/storage/connection blind.
- Best-effort isolation — REJECTED, fails at scale.
- Per-cell quotas only — REJECTED, ignores per-tenant variance.

## References

- AWS Well-Architected SaaS Lens — Tenant Isolation.
- AWS service quotas pattern.
- docs/standards/per-tenant-resource-quotas-canonical.md.
- crates/oya-shared-tenant-quota-kernel/.
