# Spec: Control-Plane Drain Failure Taxonomy

| Field | Value |
|-------|-------|
| Task slug | `control-plane-drain-failure-taxonomy` |
| Vertical | infra |
| Crate | `managed-k8s-control-plane-host-kernel` |
| Branch | `feat/cd-control-plane-drain-failure-taxonomy` |
| Stage | SPEC |

## Objective

Extend the pure-domain `managed-k8s-control-plane-host-kernel` crate
(ADR-0376) with:

1. A typed **`FailureReason`** taxonomy — machine-readable codes attached to
   the `Failed` transition so operators and reconcilers can branch on cause
   without parsing log strings.
2. A **`DrainPolicy`** value type — bounded graceful-drain parameters (eviction
   timeout, grace period, force flag) validated at construction.
3. A **`DrainPhase`** sub-state model — the three phases a draining control
   plane moves through, with a `can_proceed_to` guard enforcing linear
   progression.
4. A **`transition_to_failed`** convenience method on `ControlPlaneStatus`
   that packages the existing graph-checked `Failed` transition together with
   the typed reason.

Scope is strictly kernel-internal value types and transition-graph extensions.
No port signatures, no I/O, no HTTP, no async.

## Crate Context

`managed-k8s-control-plane-host-kernel` already owns:

- `ControlPlaneTier` — `HostedKamaji` | `DedicatedTalosSpoke`
- `DatastoreClass` — `EtcdPerTenant` | `PooledRelational`
- `ControlPlaneStatus` — full 9-variant state machine with `can_transition_to`
  / `transition` and `is_terminal` / `is_serving`
- `IllegalTransition` — typed error for graph violations

All new types are **additive** — no existing public surface is altered.

## Module Layout (flat clean-arch, single `src/lib.rs`)

```
src/lib.rs
  // --- existing surface (unchanged) ---
  pub enum ControlPlaneTier
  pub enum DatastoreClass
  pub enum ControlPlaneStatus
    pub fn can_transition_to(&self, next: Self) -> bool
    pub fn transition(self, next: Self) -> Result<Self, IllegalTransition>
    // ... (all existing methods unchanged)
  pub struct IllegalTransition

  // --- new: failure reason taxonomy ---
  pub enum FailureReason          // DatastoreBindTimeout | MediaBuildFailed | EndpointUnreachable
  impl FailureReason
    pub const fn as_str(&self) -> &'static str
    pub fn parse(value: &str) -> Option<Self>

  // --- new: drain policy value type ---
  pub enum DrainPolicyError       // ZeroEvictionTimeout
  pub struct DrainPolicy
    pub max_eviction_seconds: u32
    pub grace_period_seconds: u32
    pub force_after_timeout: bool
  impl DrainPolicy
    pub fn new(max_eviction_seconds, grace_period_seconds, force_after_timeout)
      -> Result<Self, DrainPolicyError>
    pub fn validate(&self) -> Result<(), DrainPolicyError>

  // --- new: drain phase sub-state ---
  pub enum DrainPhase              // EvictingPods | AwaitingPodTermination | FinalizingDeletion
  impl DrainPhase
    pub const fn as_str(&self) -> &'static str
    pub fn parse(value: &str) -> Option<Self>
    pub const fn can_proceed_to(&self, next: Self) -> bool

  // --- new: transition_to_failed convenience ---
  impl ControlPlaneStatus
    pub fn transition_to_failed(self, reason: FailureReason)
      -> Result<(Self, FailureReason), IllegalTransition>
```

## Type Contracts

### `FailureReason`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    /// Hosted-tier: datastore bind exceeded the allotted deadline.
    DatastoreBindTimeout,
    /// Dedicated-tier: control-plane installation-media build failed.
    MediaBuildFailed,
    /// Both tiers: API-server endpoint unreachable within deadline.
    EndpointUnreachable,
}
```

Stable slug table:

| Variant | Slug |
|---------|------|
| `DatastoreBindTimeout` | `datastore_bind_timeout` |
| `MediaBuildFailed` | `media_build_failed` |
| `EndpointUnreachable` | `endpoint_unreachable` |

### `DrainPolicy`

```rust
pub struct DrainPolicy {
    pub max_eviction_seconds: u32,   // must be > 0
    pub grace_period_seconds: u32,   // per-pod termination grace (0 = immediate)
    pub force_after_timeout: bool,   // force-terminate pods after max_eviction_seconds
}
```

Invariant: `max_eviction_seconds > 0`. `DrainPolicy::new` rejects `0` with
`DrainPolicyError::ZeroEvictionTimeout`.

### `DrainPhase`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrainPhase {
    EvictingPods,
    AwaitingPodTermination,
    FinalizingDeletion,
}
```

Legal linear progression: `EvictingPods -> AwaitingPodTermination ->
FinalizingDeletion`. Skipping or reversing returns `false` from
`can_proceed_to`.

### `transition_to_failed`

```rust
impl ControlPlaneStatus {
    pub fn transition_to_failed(
        self,
        reason: FailureReason,
    ) -> Result<(Self, FailureReason), IllegalTransition> {
        self.transition(Self::Failed).map(|s| (s, reason))
    }
}
```

Reuses the existing graph check. Terminal states still cannot transition.

## Testing Strategy

All tests live in `src/lib.rs` under `#[cfg(test)]`. No new test files.

| Test | Assertion |
|------|-----------|
| `failure_reason_roundtrips_through_slug` | All 3 variants parse/format correctly |
| `failure_reason_parse_unknown_returns_none` | Unknown slug returns `None` |
| `failure_reason_serde_uses_snake_case` | JSON round-trip via serde_json |
| `drain_policy_valid_construction` | `max_eviction_seconds > 0` succeeds |
| `drain_policy_zero_eviction_timeout_rejected` | `max_eviction_seconds = 0` errors |
| `drain_phase_roundtrips_through_slug` | All 3 variants parse/format correctly |
| `drain_phase_linear_progression_legal` | All 2 forward steps return `true` |
| `drain_phase_skip_and_reverse_illegal` | Skip + reverse return `false` |
| `transition_to_failed_from_every_non_terminal` | All 7 non-terminal states succeed |
| `transition_to_failed_from_terminal_is_error` | Both terminal states return `Err` |
| `transition_to_failed_carries_reason` | Returned tuple contains the given reason |

## Observability / SLO

These types are kernel-internal. Adapters emit OTel span attributes using the
stable slugs produced by `as_str()` / `FailureReason::as_str()`. No OTel
import in the kernel crate.

The existing `provisioning-latency.openslo.yaml` SLO is unaffected; a future
`drain-latency.openslo.yaml` SLO can reference `DrainPhase` slugs as log
correlation keys.

## Crate Boundary

- Dependencies: `serde` (workspace, already present). No new deps.
- Dev-deps: `serde_json` (workspace, already present). No new dev-deps.
- No new workspace members.
- Root `Cargo.toml` not touched.
- `ControlPlaneProvisioning` port signature (in `managed-k8s-control-plane-host-api`) not changed.
