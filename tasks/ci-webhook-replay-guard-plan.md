# Plan: ci-webhook-replay-guard

## Slice summary

Add a pure, time-injected delivery-replay/dedup guard to
`oya-ci-webhook-gateway-app`, wired after ed25519 verify + Cedar authz + route
but before Jenkins dispatch, per ADR-0367/0387.

## Canonical target

**Root-workspace member** (the ed25519 + Cedar + Steps-1-8 pipeline):
`microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app/`

There is a second package with the same crate name at
`microservices/ci-webhook-gateway/` (HMAC-based, nested `[workspace]`,
auto-excluded from root workspace). This slice does NOT touch that tree.

Absolute paths for edited files:
- `/tmp/oya-slice-ci-webhook-replay-guard-2026-05-28/microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app/src/replay.rs` (new)
- `/tmp/oya-slice-ci-webhook-replay-guard-2026-05-28/microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app/src/lib.rs` (wiring)
- `/tmp/oya-slice-ci-webhook-replay-guard-2026-05-28/microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app/src/bin/ci-webhook-gateway.rs` (AppState construction)
- `/tmp/oya-slice-ci-webhook-replay-guard-2026-05-28/microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app/tests/integration_webhook_flow.rs` (updated AppState)

## Architecture decisions (resolved from plan-review)

### Canonical implementation
The target is the root-workspace member (crates-app, ed25519 path). The flat
twin at `microservices/ci-webhook-gateway/` is a divergent HMAC-based
implementation predating the multi-crate layout; reconciling it is not in scope.

### Key type
`CiTriggerEvent.action` is `CiAction` (PrOpened/PrSynchronized/PrClosed/Ping),
NOT `PrAction` (which is an internal kernel routing enum never exposed on
`CiTriggerEvent`). The fallback key uses `action_disc: u8` (stable discriminant)
because `CiAction` does not implement `Hash` and we must not touch the kernel
crate.

### delivery_id sentinel
`delivery_id` at the wiring point is a `String` (not `Option<String>`); the
handler injects `"unknown"` when the header is absent. The guard treats
`""` and `"unknown"` as "not present" and falls back to ContentHash, ensuring
the fallback branch is reachable and distinct header-less events don't collide.

### Storage / concurrency
`DeliveryGuard` owns a `HashMap` (no internal sync). Stored as
`Arc<Mutex<DeliveryGuard>>` in `AppState` to satisfy `Clone + Send + Sync`.
Mutex acquired, pruned, checked, and released in a single synchronous block —
no async hold.

### Record-on-receipt
The key is recorded BEFORE `state.jenkins.trigger` fires. This dedupes
concurrent in-flight replays at the cost of suppressing retries within the TTL
if Step 5 fails. Documented trade-off; not a correctness guarantee.

### Insertion point (corrected from initial plan)
The pipeline is Steps 1-8 (not 1-5). The guard sits at **Step 4.5**:
- AFTER Step 2 (ed25519 verify)
- AFTER Step 3 (Cedar authz gate) — guard state is never mutated before authz
- AFTER Step 4 (`route_forgejo_event` → `RouteOutcome::Trigger(event)`)
- BEFORE Step 5 (`state.jenkins.trigger`)

### Idempotent ack status code
First delivery → 202 Accepted (unchanged).
Replay within TTL → 200 OK "duplicate delivery, already accepted".
200 is consistent with the existing "ignored" event path; distinct from 202
dispatch to allow observability differentiation.

### SLO impact
The `ci-webhook-gateway-dispatch-latency-p99` SLO uses
`ci_webhook_gateway_dispatch_latency_ms_count` as its total denominator. Replay
200 acks must NOT be counted in the dispatch-latency numerator or denominator —
they are not dispatches. The `ci-webhook-gateway-signature-verify-availability`
SLO counts non-5xx outcomes as "good"; replay 200s are non-5xx and thus count as
good, which is correct (the receiver processed the delivery correctly).

### Prune strategy
`prune(now_ms)` is called opportunistically on every `record_and_check` invocation.
Cost is O(n) over the seen map; in practice the map stays very small (TTL=5 min,
typical delivery rate is low for a pre-merge gate).

### TTL choice
300 000 ms (5 minutes). Long enough to catch all realistic Forgejo re-delivery
duplicates (which arrive within seconds); short enough to keep memory bounded.

## Follow-ups (out of scope for this slice)

- **Task #62**: ed25519-signed delivery IDs (Forgejo→Jenkins commit-status
  best-practice). The guard's dedup is most reliable with authentic delivery IDs.
- **Distributed dedup**: multi-pod deployments need sticky routing or
  Redis/Valkey shared store.
- **Record-on-success variant**: trades in-flight dedup for retry-after-502
  ability.
- **Flat-twin reconciliation**: `microservices/ci-webhook-gateway/` HMAC-based
  twin needs its own consolidation IP.

## Verification command

```bash
cargo check -p oya-ci-webhook-gateway-app --all-targets
cargo nextest run -p oya-ci-webhook-gateway-app
```

Never run `cargo check --workspace` (masks test-target + feature breaks per
doctrine). The correct scope for the crates-app is `-p oya-ci-webhook-gateway-app`
from the worktree root, which resolves to the root-workspace member unambiguously
(the flat twin's nested `[workspace]` auto-excludes it from the root workspace).
