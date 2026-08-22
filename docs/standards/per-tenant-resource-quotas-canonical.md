---
doc_class: Standard
title: Per-Tenant Resource Quotas (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: axis-tenancy
deciders: axis-tenancy, council-architecture
related_adrs: [ADR-0155]
review_cadence: annually
doc_status: published
---

# Per-Tenant Resource Quotas (Canonical)

## Authority

ADR-0155-per-tenant-resource-quotas landed this contract. Per-tenant
isolation is the AWS Well-Architected SaaS Lens "noisy-neighbor"
control. Rate-limit alone is insufficient — long-running operations,
memory, storage, and connections each need their own quota.

## Contract

### 1. Quota axes

Every µservice MUST honor per-tenant quotas on the five canonical
axes:

| Axis              | Unit        | Default per tier        |
|-------------------|-------------|-------------------------|
| Request rate      | rps         | per tenancy µservice    |
| Concurrent reqs   | inflight    | per tenancy µservice    |
| Memory            | MiB         | per tenancy µservice    |
| Storage           | GiB         | per tenancy µservice    |
| Connection count  | open conns  | per tenancy µservice    |

Tier-based defaults are declared in the tenancy µservice PRD.

### 2. Quota authority

The tenancy µservice owns canonical quota definitions. Each runtime
µservice queries quota via:

```rust
pub trait TenantQuotaKernel: Send + Sync {
    fn check(&self, tenant_id: &TenantId, axis: QuotaAxis, amount: u64)
        -> Result<QuotaDecision, QuotaError>;
    fn consume(&self, tenant_id: &TenantId, axis: QuotaAxis, amount: u64)
        -> Result<(), QuotaError>;
    fn release(&self, tenant_id: &TenantId, axis: QuotaAxis, amount: u64)
        -> Result<(), QuotaError>;
}
```

Lives in `shared-tenant-quota-kernel`.

### 3. Refusal contract

Quota exceeded → `429 Too Many Requests` with:

```
Retry-After: <seconds>
X-Tenant-Quota-Axis: <axis>
X-Tenant-Quota-Limit: <limit>
X-Tenant-Quota-Used: <used>
```

### 4. Burst allowance

Each axis declares a burst (token-bucket capacity) per tier. Default:
2× sustained rate, refilled at sustained rate.

### 5. Cell-level isolation

Per-tenant quotas are enforced PER CELL (per ADR-0083 cell model).
A tenant's quota cannot bleed across cells. Cell-wide quota = sum of
per-tenant quotas.

### 6. Sovereign-tenant tier

Sovereign-tier tenants (per ADR-0147) have DEDICATED cells; their
"quota" is the cell-wide allocation.

### 7. Validation

Quota enforcement is integration-tested per µservice via the
existing `cross-tenant-access-fuzz` gate; the new
`shared-tenant-quota-kernel` trait surface lets the fuzz harness
inject quota-exhaustion scenarios.

## References

- AWS Well-Architected SaaS Lens — Tenant Isolation.
- AWS service quotas pattern.
- Stripe's per-account quota dashboard.
- ADR-0155-per-tenant-resource-quotas.
