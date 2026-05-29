# cedar-policy-publish-rest-edge — Implementation Plan

## Objective

Add an HTTP control-plane edge for `cedar.policy.publish` to the `oya-policy-cedar-api` crate.
A new `src/rest/mod.rs` axum router wires `POST /policies/{policy_id}/versions/{version}` to the
existing typed boundary fns (`validate_cedar_policy_publish_request` + `publish_cedar_policy_from_api`),
maps `CedarPolicyPublishApiError` → HTTP status codes, honours the idempotency ledger, and emits OTel spans.

## Constraints

- Flat clean architecture (ADR-0509): one module in `src/rest/mod.rs` inside the existing crate.
- No new workspace member; never edit root `Cargo.toml`.
- axum 0.8 for control-plane REST (ADR-0090 amendment; WARN acceptable).
- Crypto: `aws-lc-rs` (never `default-features = false`).
- Router-level tests only; no business logic in the router.
- OTel via `tracing` crate (`info_span!` + `instrument` attribute-macro pattern).

## Ordered Subtasks

1. **plan** — write this file.
2. **spec** — write `docs/specs/task-cedar-policy-publish-rest-edge.md`.
3. **Cargo.toml** — add `axum`, `tokio`, `serde`, `serde_json`, `tracing` to `oya-policy-cedar-api/Cargo.toml`; add `axum`, `http-body-util`, `tower`, `tokio` to `[dev-dependencies]`.
4. **red tests** — write `src/rest/mod.rs` stubs (compile-fail / `todo!`) so tests fail on cargo check.
5. **green implementation** — implement `src/rest/mod.rs` with handler, state, router, JSON DTOs, error mapping, OTel spans.
6. **wire lib.rs** — expose `pub mod rest;` from `src/lib.rs`.
7. **verify** — `cargo check -p oya-policy-cedar-api --all-targets` then `cargo nextest run -p oya-policy-cedar-api`.
8. **review + simplify**.

## Acceptance Criteria

- `POST /policies/{policy_id}/versions/{version}` returns:
  - `201 Created` + JSON body on success.
  - `409 Conflict` when `PolicyVersionAlreadyExists`.
  - `422 Unprocessable Entity` when `IdempotencyKeyReused`.
  - `401 Unauthorized` when principal is missing.
  - `403 Forbidden` when authorization is denied.
  - `400 Bad Request` for all other validation errors.
- Path / body binding mismatch → `400`.
- Idempotent replay (same key + same fingerprint) → `201` with same body (no duplicate kernel call).
- OTel span emitted on every handler invocation with `cedar.policy.publish.status_code` attribute.
- All tests pass: `cargo nextest run -p oya-policy-cedar-api`.
- `git diff --stat origin/dev` touches ONLY `crates/oya-policy-cedar-api/`, `docs/specs/task-cedar-policy-publish-rest-edge.md`, `tasks/cedar-policy-publish-rest-edge-plan.md`.
