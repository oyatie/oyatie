# Spec: intel-provider-pool-cooldown-rotation

| Field       | Value                                                          |
|-------------|----------------------------------------------------------------|
| Vertical    | intelligence                                                   |
| Crate       | `intelligence-provider-pool-kernel`                        |
| Task slug   | `intel-provider-pool-cooldown-rotation`                        |
| ADR anchor  | M02-P02-IP-001 (pool kernel); ADR-0083 (Tier 3 test doctrine)  |
| Stage       | SPEC                                                           |

---

## Objective

Extend the pure-value provider-pool rotation kernel with quota-aware cooldown/quarantine
rotation. Quarantined accounts must be excluded during their anti-correlation window and
automatically re-admitted once the window elapses, with a deterministic
`PoolRoutingReason::FailoverFrom` fallback chain. All logic is pure (std-only, no I/O,
no async).

---

## Background

`intelligence-provider-pool-kernel` implements `pick_account`: a pure rotation kernel
that filters `Unhealthy` members and applies a `PoolRoutingStrategy`. The existing kernel
has `anti_correlation_window_ms: DurationMs` on `ProviderAccountPool` and a
`PoolMembershipChange::Quarantined` event, but no per-account quarantine timestamp and no
time-windowed cooldown filter. This task closes that gap without introducing I/O or
external dependencies.

---

## Types added (module: `src/lib.rs` — flat crate, mod-based subsystems)

### `CooldownPolicy`

```rust
/// data_class: INTERNAL_ONLY — encapsulates the cooldown window and the
/// evaluation instant so callers pass a single, self-describing input rather
/// than two separate scalars.
///
/// Cooldown semantics: an account is *in cooldown* when
/// `now.0 - last_quarantined_at.0 < window_ms.0`.  An account with
/// `last_quarantined_at = None` is never in cooldown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CooldownPolicy {
    pub window_ms: DurationMs,  // data_class: INTERNAL_ONLY
    pub now: UnixMillis,        // data_class: INTERNAL_ONLY
}
```

`CooldownPolicy::from_pool(pool: &ProviderAccountPool, now: UnixMillis) -> Self` is a
convenience constructor that extracts `window_ms` from `pool.anti_correlation_window_ms`.

### `AccountHealth` amendment

`last_quarantined_at_unix_ms: Option<UnixMillis>` is added to the existing
`AccountHealth` struct with a default of `None`. The existing `AccountHealth::healthy()`
constructor continues to produce `None` (no behavioural change to `pick_account`).

```rust
// data_class: INTERNAL_ONLY
pub struct AccountHealth {
    pub state: HealthState,
    pub consecutive_failures: u32,
    pub last_quarantined_at_unix_ms: Option<UnixMillis>,  // NEW
}
```

---

## New entry point

### `pick_account_with_cooldown`

```rust
pub fn pick_account_with_cooldown(
    pool: &ProviderAccountPool,
    request: &RequestMetadata,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    cooldown: CooldownPolicy,
) -> Result<PoolRoutingDecision, PoolError>
```

**Algorithm (pure, deterministic):**

1. Return `PoolError::EmptyMembers` if `pool.members` is empty.
2. Build `eligible: Vec<ProviderAccountId>` in BTree order by filtering out members that
   are `HealthState::Unhealthy` **or** in cooldown:
   `health.get(m).and_then(|h| h.last_quarantined_at_unix_ms).map(|t| cooldown.now.0.saturating_sub(t.0) < cooldown.window_ms.0).unwrap_or(false)`.
3. Return `PoolError::NoHealthyMembers` if `eligible` is empty.
4. Apply `pool.routing_strategy` to `eligible` using the existing internal helpers
   (`round_robin`, `least_used`, `least_latency`, `least_remaining`, `sticky`).
5. Build `fallback_chain` from remaining eligible members in BTree order (mirrors
   `pick_account`).
6. Return `PoolRoutingDecision { account_id, reason, fallback_chain, decided_at_unix_ms: cooldown.now }`.

**Cooldown guard applied before quota strategy** — step 2 occurs unconditionally before
step 4, ensuring `LeastRemaining` never selects a quarantined high-quota account.

---

## Flat mod layout (existing, unchanged)

```
src/lib.rs      — all types + pick_account + pick_account_with_cooldown + tests
```

No new modules, no new files beyond `lib.rs`. Follows the flat-crate doctrine
(ADR-0509 / hyperscaler-service-pattern).

---

## Contracts

### No HTTP/gRPC surface

This is a pure-value kernel crate. It exposes no REST or gRPC API. The kernel is
consumed by adapter crates (IP-002 Anthropic-compat, IP-003 OpenAI-compat) which own
their own surfaces.

### Proto3 usage type (informational, for adapter reference)

Adapters that map `AccountHealth` over gRPC should add:

```proto
// proto3 — informational; not emitted by this kernel crate
message AccountHealthProto {
  HealthStateProto state = 1;
  uint32 consecutive_failures = 2;
  optional uint64 last_quarantined_at_unix_ms = 3;  // NEW
}
```

### OpenAPI 3.2.0 note (informational)

No REST surface in this kernel. Adapters exposing pool routing decisions over HTTP
should add `last_quarantined_at_unix_ms` (nullable integer, format: `int64`) to their
`AccountHealthDto` schema.

---

## OpenSLO anchor

SLO authoring for this kernel is inherited from the intelligence µservice SLO at
`microservices/intelligence/slos/`. No new `.openslo.yaml` is emitted by this kernel
subtask.

---

## Testing strategy

All tests live in `#[cfg(test)] mod tests` inside `src/lib.rs` (ADR-0083 Tier 3).

| Test case | What it verifies |
|-----------|-----------------|
| `cooldown_excludes_in_window_account` | Account with `last_quarantined_at` within window is not chosen |
| `cooldown_readmits_elapsed_account` | Account with `last_quarantined_at` outside window is chosen |
| `all_in_cooldown_returns_no_healthy_members` | `PoolError::NoHealthyMembers` when every member in cooldown |
| `cooldown_fallback_chain_deterministic` | Fallback chain matches BTree iteration order of eligible members |
| `quarantined_high_quota_skipped` | Quarantined 100 % quota account skipped; healthy lower-quota account chosen (ST3 guard) |
| Existing `pick_account` suite | All green — no behavioural regression |

---

## Boundaries

| In scope | Out of scope |
|----------|-------------|
| `src/lib.rs` in `intelligence-provider-pool-kernel` | Any other crate |
| `tasks/intel-provider-pool-cooldown-rotation-plan.md` | Root `Cargo.toml` |
| `docs/specs/task-intel-provider-pool-cooldown-rotation.md` | New workspace members |
| Adding `last_quarantined_at_unix_ms` to `AccountHealth` | Modifying adapter crates |
| New `CooldownPolicy` struct + `pick_account_with_cooldown` | New async / I/O |

---

## Acceptance summary (per subtask)

| ST  | Acceptance gate |
|-----|----------------|
| ST1 | `cargo check -p intelligence-provider-pool-kernel --all-targets` green; zero new `Cargo.toml` deps |
| ST2 | `cargo nextest run -p intelligence-provider-pool-kernel` green; 4 new cooldown tests pass |
| ST3 | Quarantined-high-quota test passes; full existing suite green |
