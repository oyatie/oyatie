# Spec: intel-capability-status-lifecycle-guard

**Vertical:** intelligence
**Crate:** `intelligence-capability-registry-kernel`
**Task branch:** `feat/task-intel-capability-status-lifecycle-guard-2026-05-28`
**Stage:** SPEC

## Objective

Extend the pure-value capability-registry kernel (`intelligence-capability-registry-kernel`) with:

1. A **status lifecycle transition guard** — a method on `CapabilityStatus` returning `Result<CapabilityStatus, CapabilityStatusTransitionError>` that encodes legal lifecycle edges and rejects illegal or same-state transitions.
2. A **registry-view helper** — a pure function that partitions an iterator of `(CapabilityId, CapabilityStatus)` pairs into discoverable and invocable subsets using the existing `is_discoverable` / `is_invocable` predicates, with deterministic ordering via `BTreeMap`.

No I/O, no async, no new crate, no new Cargo.toml dependencies.

## Existing surface (baseline)

`src/status.rs` already provides:

```rust
pub enum CapabilityStatus { Active, Deprecated, Disabled }
impl CapabilityStatus {
    pub fn is_discoverable(self) -> bool { … }   // Active only
    pub fn is_invocable(self) -> bool { … }       // Active | Deprecated
    pub fn as_str(self) -> &'static str { … }
}
impl TryFrom<&str> for CapabilityStatus { … }
pub struct CapabilityStatusParseError(pub String);
```

`src/lib.rs` exposes `CapabilityId`, `CapabilityStatus`, `CapabilityStatusParseError`, `Capability`, `AutonomyTier`, `EvidenceRef`.

## Lifecycle transition matrix

```
Active      -> Deprecated   OK  (soft deprecation)
Active      -> Disabled     OK  (administrative suspend)
Deprecated  -> Active       OK  (rescind deprecation)
Deprecated  -> Disabled     OK  (escalate suspend)
Disabled    -> Active       OK  (re-activation)
Disabled    -> Deprecated   ERR (undefined; must transit through Active)
*same-state*               ERR (no-op misuse)
```

## Module layout (flat-clean-arch inside `src/`)

```
src/
  lib.rs                — re-exports; adds registry_view mod
  status.rs             — CapabilityStatus + try_transition_to + CapabilityStatusTransitionError
  registry_view.rs      — partition_views / RegistryViews  (NEW)
```

No new top-level modules beyond `registry_view`. No new crates.

## Contracts

### Transition guard (Rust API — no HTTP/gRPC surface; kernel-only)

```rust
/// Attempt a lifecycle transition.
///
/// Legal edges:
///   Active      -> Deprecated  (soft deprecation)
///   Active      -> Disabled    (administrative suspend)
///   Deprecated  -> Active      (rescind deprecation)
///   Deprecated  -> Disabled    (escalate suspend)
///   Disabled    -> Active      (re-activation)
///
/// All other transitions, including same-state, return Err.
pub fn try_transition_to(
    self,
    next: CapabilityStatus,
) -> Result<CapabilityStatus, CapabilityStatusTransitionError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityStatusTransitionError {
    pub from: CapabilityStatus,
    pub to:   CapabilityStatus,
}
impl std::fmt::Display for CapabilityStatusTransitionError { … }
impl std::error::Error  for CapabilityStatusTransitionError {}
```

### Registry-view helper (Rust API)

```rust
/// Partition an iterator of (CapabilityId, CapabilityStatus) into
/// discoverable and invocable subsets.
///
/// Ordering is deterministic (BTreeMap / lexicographic on CapabilityId).
pub fn partition_views(
    entries: impl IntoIterator<Item = (CapabilityId, CapabilityStatus)>,
) -> RegistryViews;

pub struct RegistryViews {
    pub discoverable: BTreeMap<CapabilityId, CapabilityStatus>,
    pub invocable:    BTreeMap<CapabilityId, CapabilityStatus>,
}
```

No OpenAPI or proto3 surface is required for this kernel slice. Higher-level adapters expose the views via HTTP/gRPC after projecting from these pure types.

## Testing strategy

All tests are inline `#[cfg(test)]` modules — no integration test files needed for this kernel-only slice.

| Test group | Location | Coverage |
|---|---|---|
| Transition: legal edges | `status.rs` | 5 Ok transitions |
| Transition: illegal edges | `status.rs` | Disabled->Deprecated, 3 same-state |
| Transition: error Display | `status.rs` | human-readable message |
| View: Active in both | `registry_view.rs` | both maps contain Active entry |
| View: Deprecated invocable only | `registry_view.rs` | not in discoverable, in invocable |
| View: Disabled in neither | `registry_view.rs` | absent from both maps |
| View: ordering stability | `registry_view.rs` | BTreeMap key order asserted |
| Predicate regression | `status.rs` | existing tests unchanged |
| as_str round-trip | `status.rs` | existing tests unchanged |

## Boundaries

- **Only** files touched: `src/status.rs`, `src/registry_view.rs` (new), `src/lib.rs`
- `Cargo.toml` is **not modified** — std-only, zero new deps
- Root `Cargo.toml` is **not modified**
- No other crates are touched
