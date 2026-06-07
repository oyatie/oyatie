# IP-CWG-001 — Webhook receiver + HMAC verification + pipeline dispatch

- Status: implemented (scaffold, 2026-05-26)
- Design ADR: ADR-0374
- Crate: `oya-ci-webhook-gateway-app` (`microservices/ci-webhook-gateway/`)

## Scope

The first hop of the gated change-coordination pipeline (ADR-0363/0366/0367):
verify the GitHub webhook HMAC, parse the PR event, dispatch the Jenkins lane.

## Modules

- `signature.rs` — HMAC-SHA256 verification (RustCrypto `hmac`/`sha2`/`subtle`).
  Fail-closed, constant-time, secret redacted in `Debug`. Supports both
  `X-Hub-Signature-256` (`sha256=<hex>`) and the legacy `X-Gitea-Signature`
  (raw hex).
- `event.rs` — closed router table. `pull_request` (opened/reopened/
  synchronized|synchronize) against the gated branch → `Dispatch`; wrong base /
  draft / non-gated action → `Ignored`; unknown event → typed `UnroutableEvent`.
- `dispatch.rs` — the `PipelineDispatcher` port + `JenkinsDispatcher` adapter.
  Kicks admission + `oya gate run-all` (both stages of the Jenkins `oyaCiLane`
  pipeline). The reviewer gate + merge-queue are the typed `Unimplemented`
  boundary.
- `receiver.rs` — the axum HTTP boundary. Verify → route → dispatch, with the
  correct HTTP status mapping.
- `config.rs` — env-resolved config + secret resolution.
- `main.rs` — wires it together; the Jenkins kick is a real HTTP/1.1 POST over
  a tokio TCP stream; graceful shutdown on SIGINT/SIGTERM.

## Acceptance criteria

- AC-1: HMAC verification runs on the raw body BEFORE parsing; an invalid or
  missing signature yields HTTP 401 and no dispatch. (`receiver::tests`,
  `signature::tests`)
- AC-2: A valid `pull_request:opened` against `dev` kicks Jenkins and returns
  HTTP 202 with the kicked stage + honest boundary. (`receiver::tests`)
- AC-3: `synchronize`/`synchronized` is treated as a fix-at-any-stage
  re-validation kick. (`dispatch::tests`, `event::tests`)
- AC-4: Unknown event → HTTP 422; ping → HTTP 200; wrong base → HTTP 200
  ignored. (`receiver::tests`)
- AC-5: Missing Jenkins dispatch URL → typed transport error, not silent
  success. (`dispatch::tests`)

## Verification

`cargo build` / `cargo fmt --check` / `cargo clippy --all-targets -- -D
warnings` / `cargo test` all green (43 tests).

## Deferred (tracked in `registry/placeholder-debt/adr-follow-ups.yaml`)

- `adr-0374-reviewer-gate-dispatch` — wire the adversarial reviewer gate
  (Intelligence service, ADR-0367 D2) as a dispatch target.
- `adr-0374-merge-queue-admit` — wire the merge-queue admit step (ADR-0111),
  if/when concurrent-PR volume justifies it (ADR-0363 §3).
- `adr-0374-delivery-dedup-log` — append-only delivery-id dedup log (ADR-0112
  idempotency carried forward); v1 relies on GitHub's at-least-once + the
  idempotent kick.
