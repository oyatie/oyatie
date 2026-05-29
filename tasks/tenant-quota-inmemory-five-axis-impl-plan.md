# Plan: tenant-quota-inmemory-five-axis-impl

## Goal

Add `InMemoryTenantQuota` — a pure in-memory, deterministic implementation of `TenantQuotaKernel` — covering all five `QuotaAxis` values. Replace `SkeletonNotYetImplemented` placeholders with real counter logic.

## Acceptance Criteria (from backlog)

- `check` is non-mutating; returns `Allowed { remaining }` when under limit and `Denied { limit, used, retry_after_seconds }` at/over limit.
- `consume` atomically decrements only when allowed; returns `QuotaError::UnknownTenant` for unknown tenants.
- `release` increments back without exceeding the configured limit.
- Per-axis configuration constructor seeds limits.
- Hermetic unit tests only (no I/O).
- All existing tests still pass.
- `#![forbid(unsafe_code)]` retained.

## Steps

1. [x] Write plan (this file).
2. [x] Write spec (`docs/specs/task-tenant-quota-inmemory-five-axis-impl.md`).
3. [x] Write red tests covering all acceptance criteria.
4. [x] Implement `InMemoryTenantQuota` with `AxisConfig` and `AxisState`.
5. [x] Run `cargo check -p oya-shared-tenant-quota-kernel --all-targets` (green).
6. [x] Run `cargo nextest run -p oya-shared-tenant-quota-kernel` (green).
7. [x] Self-review and simplify.
8. [x] Commit, push, open PR.

## Design Decisions

- Use `std::collections::HashMap<TenantId, HashMap<QuotaAxis, AxisState>>` protected by `std::sync::Mutex` for interior mutability (`&self` on consume/release).
- `AxisState { limit: u64, used: u64 }` — simple counter pair.
- `retry_after_seconds` on denial: fixed value of `1` (deterministic, no clock dependency).
- `InMemoryTenantQuotaBuilder` pattern: register tenants with per-axis limits, then build.
- No external dependencies; no I/O.
