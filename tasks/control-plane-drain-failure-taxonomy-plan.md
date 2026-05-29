# Plan: Control-Plane Drain Failure Taxonomy

| Field | Value |
|-------|-------|
| Task slug | `control-plane-drain-failure-taxonomy` |
| Vertical | infra |
| Crate | `oya-managed-k8s-control-plane-host-kernel` |
| Branch | `feat/cd-control-plane-drain-failure-taxonomy` |
| Priority | medium |
| Effort | M |

## Requirements Analysis

### Problem Statement

The ADR-0376 control-plane state machine currently has:
- A `Draining` state on the teardown path (`Active -> Draining -> Deleted`)
- A `Failed` terminal state reachable from any non-terminal state

Missing:
1. **Graceful-drain sub-model**: a bounded, structured way to express what a
   drain operation entails — timeout, allowed phases, whether the drain can
   be finalized — without coupling to I/O or time primitives.
2. **Typed `FailureReason` taxonomy**: when a `Failed` transition fires, the
   current model carries no cause. Operators/reconcilers need machine-readable
   reason codes to branch on (retry vs. escalate vs. notify).

### Acceptance Criteria

1. A `FailureReason` enum with exactly three variants is introduced:
   - `DatastoreBindTimeout` — hosted-tier datastore bind exceeded the allotted
     deadline (applies to `HostedKamaji` branch).
   - `MediaBuildFailed` — dedicated-tier installation-media build failed
     (applies to `DedicatedTalosSpoke` branch).
   - `EndpointUnreachable` — API-server endpoint not reachable within deadline
     (applies to both tiers post-`Provisioning`).

2. A `DrainPolicy` value type encapsulates the graceful-drain sub-model:
   - `max_eviction_seconds: u32` — upper-bound on eviction time (must be > 0).
   - `grace_period_seconds: u32` — per-pod termination grace period.
   - `force_after_timeout: bool` — whether to force-terminate after deadline.
   - Constructor validates that `max_eviction_seconds > 0`.

3. A `DrainPhase` enum models the internal phases of a graceful drain:
   - `EvictingPods` — actively evicting workload pods.
   - `AwaitingPodTermination` — waiting for pod termination grace.
   - `FinalizingDeletion` — control-plane resources being deleted.
   - `can_proceed_to` enforces `EvictingPods -> AwaitingPodTermination ->
     FinalizingDeletion` (linear, no skipping).

4. `ControlPlaneStatus::transition_to_failed(reason: FailureReason)` is added
   as a convenience that calls the existing `transition(Failed)` and packages
   the reason alongside (returns `Result<(ControlPlaneStatus, FailureReason), IllegalTransition>`).

5. All new types are `#[derive(Clone, Copy, Debug, Eq, PartialEq, ...)]` with
   `serde(rename_all = "snake_case")` matching existing patterns.

6. `DrainPolicy::validate()` is the single fallible entry-point; returns a
   typed `DrainPolicyError`.

7. The existing `ControlPlaneProvisioning` port signature is NOT changed.

8. `cargo nextest run -p oya-managed-k8s-control-plane-host-kernel` passes
   green with full deterministic coverage of:
   - All `FailureReason` slug roundtrips.
   - `DrainPolicy` validation (valid + zero-timeout rejection).
   - `DrainPhase::can_proceed_to` graph (legal + illegal transitions).
   - `transition_to_failed` from every non-terminal status.

### Edge Cases

- `max_eviction_seconds = 0` must be rejected by `DrainPolicy::validate()` —
  a zero-second eviction window is operationally invalid.
- `FailureReason` must be `Copy` so it can be threaded through const contexts.
- `DrainPhase` transitions are linear; attempting to go backwards
  (`FinalizingDeletion -> EvictingPods`) must return `false` from
  `can_proceed_to`.
- `Failed` terminal state still enforces no outgoing transitions — the
  `FailureReason` is a companion, not a replacement for the existing graph.

### K8s/Cloud-Native Context

- The `Draining` state maps to Kubernetes graceful node/pod eviction semantics:
  `max_eviction_seconds` corresponds to `terminationGracePeriodSeconds` + drain
  timeout; `force_after_timeout` corresponds to `--force --ignore-daemonsets`.
- `FailureReason` codes are designed to be emitted as OTel span attributes and
  structured log fields without any I/O in the kernel.
- These types stay pure (no OTel import) — the adapter layer attaches them to
  spans.

## Ordered Subtasks

- [ ] 1. Write plan file (this file).
- [ ] 2. Write spec: `docs/specs/task-control-plane-drain-failure-taxonomy.md`.
- [ ] 3. Write failing tests in `oya-managed-k8s-control-plane-host-kernel`
         covering all acceptance criteria.
- [ ] 4. Implement `FailureReason`, `DrainPolicy`, `DrainPhase`,
         `DrainPolicyError`, and `transition_to_failed` in kernel `src/lib.rs`.
- [ ] 5. Run `cargo nextest run -p oya-managed-k8s-control-plane-host-kernel`
         (must go green).
- [ ] 6. Self-review (correctness / architecture / security / perf / cloud-native).
- [ ] 7. Simplify (dead-code, naming, guard clauses); rerun nextest.
- [ ] 8. Disjointness check + commit + push + PR.
