# TODO: id-workload-grpc-surface

## Completed

- [x] T1: Add prost/tonic/tonic-prost deps + build.rs (tonic-prost-build + protoc-bin-vendored)
      — `cargo check -p oya-identity-workload-rest --all-targets` passes; generated stubs
        for WorkloadAuthorizer + WorkloadTokenValidator are referenceable; root Cargo.toml
        unchanged; aws-lc-rs default-features not changed.

- [x] T2: Implement src/grpc/mod.rs with WorkloadGrpcServer impls
      — AuthorizeWithToken/AuthorizeBatch delegate to authorize_with_token app use-case;
        Authorize uses build_active_principal + authorizer.authorize (crate-private, no duplication);
        ValidateToken uses validate_workload_token (OIDC adapter);
        deny -> DECISION_EFFECT_DENY response (never tonic Err);
        token-validation failure -> typed ValidateTokenResponse error (engine not consulted);
        store/JWKS unavailable -> Status::Unavailable (unary) or per-item DENY (batch);
        one AuditRecord per authorize and per token-validation.

- [x] T3: Add tests/common.rs + tests/grpc_authorize_deny.rs
      — 7 gRPC tests passing alongside 12 REST tests (19/19 total);
        assertions (a) allow, (b) deny-as-response, (c) invalid-token-typed-error,
        (d) store-unavailable-Unavailable all green;
        engine-not-consulted proven structurally (permit vs empty authorizer);
        fixtures shared via tests/common.rs.

- [x] T4: Lane docs written
      — docs/specs/slice-id-workload-grpc-surface.md: surface, fail-closed mapping,
        OidcValidationError mapping table, proto authority decision, dep-seam note,
        SLO coverage note.
      — tasks/id-workload-grpc-surface-plan.md: architecture decisions resolving all
        plan-review blockers.
      — tasks/id-workload-grpc-surface-todo.md (this file).
      — SLOs: authorize-latency-p99.openslo.yaml + decision-correctness.openslo.yaml
        cover the gRPC authorize/validate paths (shared use-case core). No new SLO added.

## Post-review cleanup (no behavior change)

- [x] ValidateToken now maps each `OidcValidationError` to its distinct proto
      `ValidationErrorKind` via `oidc_error_to_kind`, mirroring the spec mapping
      table, instead of collapsing all 12 kinds to `MALFORMED`. The `detail`
      string is unchanged. A mesh PEP / Envoy ext_authz consumer can now branch
      on expired vs forged-signature vs issuer-mismatch. No test assertion or
      fail-closed semantic changes (tests assert only the `Error` variant).
- [x] Corrected the `run_authorize_with_token_grpc` docstring: it always returns
      `Ok(outcome)`; the unary caller maps `StoreUnavailable` to
      `Status::unavailable` and batch maps it to a per-item DENY.
- [x] Removed two no-op `.map_err(|e| e)?` (clippy::map_identity) in favor of `?`.

### Deliberate deferrals (out of slice scope)

- `AuthorizeResponse.reason` stays `None` on gRPC; the slice's no-drift contract
  is framed around the decision EFFECT, not the reason. The audit record still
  carries the decision label. REST/gRPC reason-payload parity is future work.
- A missing proto `Resource` becomes `Resource::new("", "")`; default-deny keeps
  this fail-closed-safe on both surfaces.

## Registry note

The `registry/dependency-rationales.json` `allowed_crates` arrays for tonic,
tonic-prost, tonic-prost-build, prost, and protoc-bin-vendored were updated to
include `oya-identity-workload-rest` as a deliberate scoped out-of-lane edit
(commit b547b9e4).  Dep-seam gate is ReportOnly (non-blocking); no concurrent
lane touches those rows.
