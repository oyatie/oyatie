# TODO: ci-webhook-replay-guard

## Tasks

- [x] T1: Add `src/replay.rs` — pure, std-only, time-injected `DeliveryGuard`
      with `record_and_check(key, now_unix_millis) -> Verdict` and
      `prune(now_unix_millis)`. `DeliveryKey::from_parts` implements
      delivery-id-vs-ContentHash precedence with sentinel handling.
      No async, no clock read inside the type, no new external crate,
      no root Cargo.toml edit.
      Accept: `cargo check -p oya-ci-webhook-gateway-app --all-targets` passes.

- [x] T2: Wire `DeliveryGuard` into `lib.rs` at Step 4.5 (after verify+authz+route,
      before Jenkins trigger). Add `delivery_guard: Arc<Mutex<DeliveryGuard>>`
      to `AppState`. Replay returns 200 OK idempotent ack; first delivery still
      202 on dispatch. Update binary entry point and integration test `AppState`
      constructions.
      Accept: replayed delivery returns idempotent ack, does NOT call
      `state.jenkins.trigger` a second time; first delivery path unchanged.

- [x] T3: Unit tests in `src/replay.rs` `#[cfg(test)]` covering all five cases
      (deterministic via injected timestamps, no wall clock):
      1. FirstSeen then Replay for identical keys.
      2. Two distinct keys both FirstSeen.
      3. TTL expiry restores FirstSeen via injected now.
      4. `prune()` removes only expired entries, retains fresh ones.
      5. delivery-id-vs-ContentHash key-derivation precedence + sentinel "unknown"
         fallback + distinct sentinel events don't collide.
      Accept: `cargo nextest run -p oya-ci-webhook-gateway-app` green (7/7).

- [x] T4: Lane-namespaced docs:
      - `docs/specs/slice-ci-webhook-replay-guard.md` (objective, vertical,
        contracts, mod layout, testing strategy, acceptance, boundaries,
        open questions including task #62 and distributed-dedup follow-up)
      - `tasks/ci-webhook-replay-guard-plan.md` (this plan)
      - `tasks/ci-webhook-replay-guard-todo.md` (this file)
      Confirm `microservices/ci-webhook-gateway/slos/ci-webhook-gateway.openslo.yaml`
      remains valid per ADR-0130.
      Accept: three lane docs exist; SLO file valid; affected gates pass locally.

## Verification evidence

```
cargo check -p oya-ci-webhook-gateway-app --all-targets  → Finished (0 errors)
cargo nextest run -p oya-ci-webhook-gateway-app          → 7 passed, 0 skipped
```

Root `Cargo.toml` diff: unchanged (no new workspace member, no new dependency).

## SLO status

`microservices/ci-webhook-gateway/slos/ci-webhook-gateway.openslo.yaml` — valid
OpenSLO v1, two SLOs:

1. `ci-webhook-gateway-dispatch-latency-p99` — ratio metric over
   `ci_webhook_gateway_dispatch_latency_ms_{bucket,count}`. Replay 200 acks are
   NOT dispatches and must be excluded from this metric's instrumentation (Stage-8
   OTel wiring responsibility). No change to the YAML required.

2. `ci-webhook-gateway-signature-verify-availability` — counts non-5xx outcomes
   as "good" over `ci_webhook_gateway_deliveries_total`. Replay 200 acks are
   non-5xx and correctly count as "good" (the receiver handled the delivery
   correctly). No YAML change required.
