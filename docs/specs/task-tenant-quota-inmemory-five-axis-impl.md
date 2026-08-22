# Spec: tenant-quota-inmemory-five-axis-impl

**Crate**: `shared-tenant-quota-kernel`  
**ADR**: ADR-0155  
**Priority**: high | **Effort**: M | **Lane**: foundation

## Problem

The `TenantQuotaKernel` trait has no concrete implementation; all three methods return
`SkeletonNotYetImplemented`. Runtime µservices cannot gate per-tenant resource use.

## Solution

Add `InMemoryTenantQuota` — a pure in-memory, deterministic reference implementation of
`TenantQuotaKernel` covering all five `QuotaAxis` variants. No I/O. No external crates.

## Data Model

```
AxisState { limit: u64, used: u64 }
TenantState = HashMap<QuotaAxis, AxisState>
InMemoryTenantQuota { state: Mutex<HashMap<TenantId, TenantState>> }
```

## Behaviour Contract

### `check(&self, tenant_id, axis, amount) -> Result<QuotaDecision, QuotaError>`

- Non-mutating (only reads state under lock).
- `UnknownTenant` if tenant absent.
- `Allowed { remaining: limit - used }` when `used + amount <= limit`.
- `Denied { limit, used, retry_after_seconds: 1 }` when `used + amount > limit`.

### `consume(&self, tenant_id, axis, amount) -> Result<QuotaDecision, QuotaError>`

- `UnknownTenant` if tenant absent.
- Calls check logic; on `Allowed` atomically sets `used += amount` and returns `Allowed`.
- On `Denied` returns `Denied` without mutating state.

### `release(&self, tenant_id, axis, amount) -> Result<(), QuotaError>`

- `UnknownTenant` if tenant absent.
- Decrements `used` by `amount`, clamped to 0 (never negative/underflow).
- Returns `Ok(())`.

## Constructor

```rust
InMemoryTenantQuota::builder()
    .register(tenant_id, axis, limit)  // can call multiple times
    .build() -> InMemoryTenantQuota
```

## Acceptance Tests

1. `check` on allowed tenant under limit returns `Allowed { remaining }`.
2. `check` at/over limit returns `Denied`.
3. `check` is non-mutating (subsequent check returns same result).
4. `consume` decrements used when allowed.
5. `consume` returns `Denied` without decrementing when at limit.
6. `consume` on unknown tenant returns `UnknownTenant`.
7. `release` increments used back (decrements usage counter).
8. `release` clamps to 0, never negative.
9. `release` on unknown tenant returns `UnknownTenant`.
10. Full cycle: consume to limit -> denied -> release -> allowed again.
11. All five axes can be configured and queried independently.

## Constraints

- `#![forbid(unsafe_code)]` retained.
- No new `[dependencies]` in `Cargo.toml`.
- Only `std` library used.
- Hermetic: no file I/O, no network, no time dependency.
