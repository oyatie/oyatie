# Spec: pooling-kernel-cooldown-window

**Crate:** `intelligence-provider-pool-kernel`
**Lane:** `pooling`
**Priority:** high
**Effort:** M

---

## Objective

Extend the pure pool kernel with time-windowed cooldown and per-failure-kind
exponential backoff. All additions are std-only, panic-free, and deterministic.
No new crate dependencies. No I/O, no async.

---

## Contracts

This crate has no external HTTP/gRPC/proto contracts — it is a pure-value kernel
consumed by adapter crates. No OpenAPI/AsyncAPI/proto changes are required.

---

## Mod layout (flat-clean-arch per ADR-0509)

All code lives in `src/lib.rs` (single-file flat crate). No submodules added.

---

## Data model additions

### `FailureKind`

```
pub enum FailureKind {
    UpstreamRateLimit429,
    UpstreamServerError5xx,
    ConnectionTimeout,
    AuthFailure,
}
```

`Copy + Clone + Debug + Eq + PartialEq + Hash`

data_class: INTERNAL_ONLY

### `AccountHealth` extension

Add `cooldown_until: Option<UnixMillis>` (informational field; `None` on `healthy()`).

The field records the absolute epoch at which the account transitions out of
cooldown according to the caller's backoff computation. Routing decisions in
`pick_account_with_cooldown` remain driven by `QuarantineMap`, not this field,
preserving backward compatibility for callers that construct `AccountHealth`
directly.

### `CooldownPolicy::window_for`

```
pub fn window_for(kind: FailureKind, consecutive_failures: u32) -> DurationMs
```

Pure const-style function; no `self`. Returns the per-failure-kind exponential
backoff window for the given failure count (1-indexed; 0 treated as 1).

Backoff tables (all values in milliseconds):

| FailureKind            | f=1     | f=2     | f=3      | f=4+     |
|------------------------|---------|---------|----------|----------|
| UpstreamRateLimit429   | 30_000  | 60_000  | 120_000  | 300_000  |
| UpstreamServerError5xx | 10_000  | 30_000  | 60_000   | 60_000   |
| ConnectionTimeout      |  5_000  | 15_000  | 30_000   | 30_000   |
| AuthFailure            | 60_000  | 300_000 | 900_000  | 900_000  |

### `populate_quarantine_from_changes`

```
pub fn populate_quarantine_from_changes(
    changes: &[PoolMembershipChange],
    now: UnixMillis,
    quarantines: &mut QuarantineMap,
)
```

Scans `changes`; for each `PoolMembershipChange::Quarantined(id)` inserts
`(id, now)` into `quarantines` (overwriting any stale entry). `Added` and
`Removed` variants are ignored.

---

## Testing strategy

Pure unit tests, all in `#[cfg(test)] mod tests` in `src/lib.rs`.

| Test                               | Coverage target                                   |
|------------------------------------|---------------------------------------------------|
| `failure_kind_backoff_rate_limit`  | ST2: UpstreamRateLimit429 escalation 1–4+        |
| `failure_kind_backoff_server_error`| ST2: UpstreamServerError5xx escalation 1–4       |
| `failure_kind_backoff_timeout`     | ST2: ConnectionTimeout escalation 1–3+           |
| `failure_kind_backoff_auth`        | ST2: AuthFailure escalation 1–4+                 |
| `backoff_zero_failures_treated_as_one` | ST2: consecutive_failures=0 safe              |
| `populate_quarantine_from_changes_basic` | ST3: Quarantined entries inserted          |
| `populate_quarantine_ignores_non_quarantined` | ST3: Added/Removed ignored             |
| `account_health_cooldown_until_field` | ST1: field present on AccountHealth            |

Existing 37 tests remain green (no behavioural regression).

---

## Observability / SLO

No new SLO required for a pure-value kernel crate. The adapter crates that
consume this kernel are responsible for emitting OTel spans (IP-002/IP-003).

---

## Crate boundary

Changes are 100% inside `intelligence-provider-pool-kernel`. No other crate
is touched. `Cargo.toml` is unchanged.
