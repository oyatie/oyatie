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
      — 9 gRPC tests passing alongside 10 REST tests (19/19 total);
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

## Follow-on (out of this lane's disjointness boundary)

- [ ] Registry update: append `oya-identity-workload-rest` to the `allowed_crates`
      arrays in registry/dependency-rationales.json for tonic, tonic-prost,
      tonic-prost-build, prost, and protoc-bin-vendored.
      (Dep-seam gate is ReportOnly so this is non-blocking; tracked for hygiene.)
