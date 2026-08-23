# Slice: ci-webhook-replay-guard

## Objective

Add a pure, time-injected delivery-replay/dedup guard (idempotency) to the
trustless webhook receiver `ci-webhook-gateway-app`, wired AFTER ed25519
verify + Cedar authz + event parse/route (Steps 2-4) but BEFORE Jenkins dispatch
(Step 5), so a first delivery dispatches normally and an identical replay within
the TTL returns a benign idempotent acknowledgement (200 OK, no second Jenkins
kickoff).

ADR provenance: ADR-0367 (trustless pre-merge verification gateway),
ADR-0387 (ci-webhook-gateway GitHub-to-Jenkins commit-status).

## Vertical

`ci-webhook-gateway` (substituted for the on-dev-absent llm-gateway vertical).

Crate scope: `microservices/ci-webhook-gateway/crates/ci-webhook-gateway-app`
only. This is the root-workspace member (root Cargo.toml line 705) containing
the full Steps 1-8 pipeline with ed25519 verify + Cedar authz + Jenkins dispatch.

### Canonical implementation note

There are two packages with the same name `ci-webhook-gateway-app` on
`origin/dev`:

1. **Root-workspace member** (this slice's target):
   `microservices/ci-webhook-gateway/crates/ci-webhook-gateway-app/`
   — ed25519 verify, Cedar authz, Steps 1-8, `state.jenkins.trigger`.

2. **Standalone flat twin** (out of scope):
   `microservices/ci-webhook-gateway/` (declares its own nested `[workspace]`,
   so it is auto-excluded from the root workspace)
   — HMAC-based, different pipeline.

This slice targets (1). The flat twin is a divergent implementation predating
the multi-crate layout; its reconciliation is not in scope here. The guard added
here is wired only into the root-workspace member's request path.

## Handler pipeline (actual, Steps 1-8)

```
1. Extract X-GitHub-Signature-256 / X-GitHub-Event / X-GitHub-Delivery /
   X-GitHub-Timestamp headers.
2. ed25519 verify (raw bytes, before JSON parse).
3. Cedar authz gate.
4. route_github_event → RouteOutcome::Trigger(CiTriggerEvent).
4.5. [NEW] Replay guard: DeliveryKey::from_parts → record_and_check.
     - Verdict::FirstSeen  → continue to Step 5.
     - Verdict::Replay     → return 200 OK "duplicate delivery, already accepted".
5. Jenkins trigger → JenkinsJob.
6. GitHub post_all pending statuses.
7. Jenkins poll_status loop.
8. GitHub post_all final statuses → return 202 Accepted.
```

The guard is placed AFTER verify+authz+route so guard state is never mutated
before verification succeeds (no state pollution from unauthenticated requests).

## Key-derivation precedence

```
delivery_id (from X-GitHub-Delivery header) is non-empty AND != "unknown"
  → DeliveryKey::DeliveryId(delivery_id)
else
  → DeliveryKey::ContentHash { head_sha, pr_number, action_disc: u8 }
```

The fallback uses `action_disc: u8` (stable discriminant) because
`ci_webhook_gateway_kernel::CiAction` does not implement `Hash`; we avoid
touching the kernel crate.

The sentinel `"unknown"` is the value the handler injects via
`header_str(&headers, "x-github-delivery").unwrap_or("unknown")` when the
header is absent. Treating it as "not present" ensures the fallback branch is
genuinely reachable and that distinct header-less events do not collide on the
single sentinel key.

## TTL

Default TTL: **300 000 ms (5 minutes)**.

Rationale: GitHub's re-delivery window is up to 24 h, but duplicate
re-deliveries in normal operation arrive within seconds. 5 min catches all
realistic duplicates while keeping the in-memory map small.

## Record-on-receipt policy

The key is recorded **before** `state.jenkins.trigger` fires (Step 4.5, not
post-Step-5). This ensures a concurrent replay arriving while the first delivery
is in-flight (the handler is still synchronously polling Jenkins in Steps 7-8)
is deduped rather than double-fired.

Trade-off: if the first delivery fails at Step 5 (Jenkins returns 502), the key
is already recorded. A legitimate retry within the TTL will be treated as a
replay and return 200 without triggering Jenkins. This is an accepted limitation
of best-effort single-instance dedup. Operators can wait for the TTL to expire
(5 min) before retrying.

## Concurrency model

`DeliveryGuard` uses a plain `HashMap` with no internal synchronization.
It is stored as `Arc<Mutex<DeliveryGuard>>` in `AppState` so the
`Clone + Send + Sync` router state constraint is satisfied. The `Mutex` is
acquired, pruned, checked, and released within a single synchronous block —
no async hold.

## Scope limitations (single-instance only)

The guard is **best-effort single-instance dedup**. In a horizontally-scaled
deployment (multiple pods), a replay routed to a different pod bypasses the
guard entirely. Distributed dedup (sticky routing or shared Redis/Valkey store)
is an open question, named alongside task #62 as an explicit follow-up.

## Module layout (flat-clean-arch per ADR-0509)

```
microservices/ci-webhook-gateway/crates/ci-webhook-gateway-app/
  src/
    lib.rs        # AppState + handler — guard wired at Step 4.5
    replay.rs     # [NEW] DeliveryGuard, DeliveryKey, Verdict, prune, tests
    bin/
      ci-webhook-gateway.rs   # binary entry point
  tests/
    integration_webhook_flow.rs   # existing integration tests (updated)
```

No new crate, no root Cargo.toml edit. `replay` is a `pub mod` inside the
existing crate, per ADR-0509 single-crate-per-service flat-clean-arch.

## Contracts

This slice adds no new HTTP endpoints and does not change the external API
surface. The only observable behaviour change is:

| Scenario | Before | After |
|---|---|---|
| First delivery | 202 Accepted | 202 Accepted (unchanged) |
| Replay within TTL | 202 Accepted (double-dispatch) | 200 OK "duplicate delivery, already accepted" |
| After TTL expiry | — | 202 Accepted (re-dispatched as FirstSeen) |
| ping event | 200 OK | 200 OK (unchanged, guard not reached — Ignored before Step 4.5) |

No OpenAPI contract change (replay path is an implementation detail, not a new
resource or status code that clients depend on; 200 is already used for ignored
events).

## Testing strategy

Unit tests in `src/replay.rs` `#[cfg(test)]` (five cases, no wall clock):

1. FirstSeen then Replay for identical keys.
2. Two distinct keys both FirstSeen.
3. TTL expiry restores FirstSeen via injected now.
4. `prune()` removes only expired entries, retains fresh ones.
5. Key-derivation precedence: delivery-id > (head_sha, pr_number, action_disc);
   sentinel "unknown" falls back correctly; two distinct sentinel events don't
   collide.

Integration tests updated (`tests/integration_webhook_flow.rs`) — existing
happy-path (202) and ping (200) tests continue to pass with `delivery_guard`
added to `AppState`.

## Acceptance criteria

- `cargo check -p ci-webhook-gateway-app --all-targets` passes.
- `cargo nextest run -p ci-webhook-gateway-app` green (7/7).
- `git diff` shows root `Cargo.toml` unchanged and no new dependency added.
- Code shows guard placed after Step 4 route and before Step 5 trigger.
- A replayed delivery returns the idempotent ack and does NOT call
  `state.jenkins.trigger` a second time.
- The first delivery path is unchanged (still 202 on dispatch).

## Open questions / follow-ups

1. **Task #62** — CI-webhook + ed25519: best-practice GitHub→Jenkins
   commit-status signing. The guard's idempotency guarantee is strongest when
   delivery IDs are present and authentic; signed delivery IDs (task #62)
   would close this loop.

2. **Distributed dedup** — this guard is single-instance. A multi-pod
   deployment needs sticky routing or a shared store (Redis/Valkey) to
   guarantee no double-dispatch across pods.

3. **Record-on-receipt vs record-on-success** — current policy records before
   trigger (dedup in-flight but loses retry-ability on 502). A record-on-success
   variant would restore retry-ability at the cost of in-flight double-fire risk.
   Choose based on observed Jenkins reliability.
