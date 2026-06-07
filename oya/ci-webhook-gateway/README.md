# ci-webhook-gateway

The CI webhook gateway: the **first hop** of the gated change-coordination
pipeline (ADR-0363/0366/0367/0374). It is the missing trigger that turns a
GitHub `pull_request` event into a real, gated CI run — so PRs against `dev`
are gated by REAL Jenkins-produced status checks and the manual
admin-relax-merge seam is retired.

## What it does

1. Receives GitHub webhooks at `POST /webhook/github`.
2. Verifies the `X-Hub-Signature-256` HMAC on the raw body, **fail-closed**,
   constant-time, before any parsing.
3. Parses `pull_request` events (opened/reopened/synchronized) against the
   gated branch (`dev`).
4. Dispatches the gated pipeline by kicking the Jenkins `oyaCiLane` lane
   (admission → `oya gate run-all`, the ADR-0367 trusted runner that posts the
   GitHub commit statuses).

It does NOT run gates, post statuses, review code, or merge — those are the
trusted runner (Jenkins), the reviewer (Intelligence service), and GitHub
auto-merge respectively. Stages not yet built are typed `Unimplemented` (501)
boundaries tracked in `registry/placeholder-debt/adr-follow-ups.yaml`.

## Layout

- `src/signature.rs` — HMAC-SHA256 verify (pure domain).
- `src/event.rs` — PR-event parse + closed router table (pure domain).
- `src/dispatch.rs` — the `PipelineDispatcher` port + `JenkinsDispatcher`.
- `src/receiver.rs` — the axum HTTP boundary.
- `src/config.rs` / `src/main.rs` — env config + binary entrypoint.

## Build & test

```
cd microservices/ci-webhook-gateway
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Provision & deploy

See `SETUP-RUNBOOK.md` for the human-provisioning steps (webhook registration,
HMAC secret, commit signing, deploy) and `runbooks/on-call.md` for operations.

## Blessed dependencies only

Tokio / Axum / Tower / Hyper / serde / tracing (approved runtime allowlist) +
RustCrypto `hmac`/`sha2`/`subtle` for the HMAC (MIT/Apache-2.0, OSI-clean). No
`reqwest`/`hyper-client` — the Jenkins kick is a minimal HTTP/1.1 POST over a
tokio TCP stream.
