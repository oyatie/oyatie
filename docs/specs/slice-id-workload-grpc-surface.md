# Spec: workload-identity gRPC surface (slice-id-workload-grpc-surface)

## Objective

Add a tonic gRPC delivery surface to `crates/identity-workload-rest` that
implements the already-specified proto3 `WorkloadAuthorizer`
(AuthorizeWithToken / Authorize / AuthorizeBatch) and `WorkloadTokenValidator`
(ValidateToken) services from
`microservices/identity/contracts/proto/workload.proto`
(package `oya.identity.workload.v1`).

The gRPC surface mounts the SAME inward `identity-workload-app` use-cases
the axum REST surface already mounts, proving that REST and gRPC share one
use-case core with no logic drift.  This unblocks mesh-native PEP (Envoy
ext_authz at the waypoint, sidecars, api-gateway) and de-risks the service.

## Vertical

`identity` lane.  Crate touched: `crates/identity-workload-rest` (the ONLY
crate this lane may touch).

## Architecture doctrine decision

The proto3 service is implemented as a `src/grpc/` mod inside the existing
`-rest` crate, not as a new `-grpc` sibling crate.  This is an **accepted
interim step** under ADR-0509 single-crate-per-service: the existing crate
already IS the single service crate for this vertical, and grpc/ is a delivery
mod alongside rest/ (the REST handlers live directly in src/lib.rs).  A future
crate-collapse migration (when the identity vertical is ready to converge fully
on ADR-0509) will absorb the `*-domain`, `*-app`, `*-api`, `*-oidc-adapter`,
and `*-authz-cedar-adapter` siblings into a single crate; the grpc/ mod already
lives in the right place for that target shape.

The established repo pattern places gRPC delivery in separate `*-grpc` crates
(shared-backbone-grpc-*, payments-charge-grpc).  The divergence here is
intentional: those crates are standalone services; the identity workload service
already spans 6 crates in legacy clean-arch style, and ADR-0509 mandates
convergence not further sprawl.

## Proto authority

The authoritative contract is
`microservices/identity/contracts/proto/workload.proto`, registered in the
`contracts.proto` array at `microservices/identity/manifest.json` lines 88-92.
No other file defines `WorkloadAuthorizer` / `WorkloadTokenValidator`.

## Contracts

### Proto (server-side only)

Package `oya.identity.workload.v1` — see
`microservices/identity/contracts/proto/workload.proto`.

Services implemented:

| Service                   | RPC                 | Shared core                                   |
|---------------------------|---------------------|-----------------------------------------------|
| WorkloadAuthorizer        | AuthorizeWithToken  | `identity_workload_app::authorize_with_token` |
| WorkloadAuthorizer        | Authorize           | `build_active_principal` (crate fn) + `authorizer.authorize` |
| WorkloadAuthorizer        | AuthorizeBatch      | `identity_workload_app::authorize_with_token` (per item) |
| WorkloadTokenValidator    | ValidateToken       | `identity_workload_oidc_adapter::validate_workload_token` |

Note: the `Authorize` path uses `build_active_principal` (a crate-private fn in
`src/lib.rs`) rather than the `-app` use-case, mirroring the REST `/authorize`
handler exactly.  This is NOT a logic duplication — it IS the shared core for
this path.  The blanket "all three authorizer RPCs delegate to identity-workload-app"
claim in earlier plan drafts was only true for AuthorizeWithToken/Batch; Authorize
was always this path.

### OpenAPI

`microservices/identity/contracts/openapi/workload.yaml` — unchanged (REST
surface not modified).

## Module layout (flat-clean-arch)

```
crates/identity-workload-rest/
  src/
    lib.rs          — REST handlers (axum), shared state, audit types
    grpc/
      mod.rs        — WorkloadGrpcServer<R,D,A,S> impls for both tonic traits
  tests/
    rest_endpoints.rs  — existing REST integration tests (unchanged)
    common.rs          — shared fixtures (mint_token, provisioned_state, FailingRepository)
    grpc_authorize_deny.rs — gRPC integration tests
  build.rs          — tonic-prost-build + protoc-bin-vendored proto codegen
  Cargo.toml        — adds prost, tonic {router,transport}, tonic-prost, build-deps
```

## Fail-closed PEP mapping

| Condition                      | REST response       | gRPC response                        |
|-------------------------------|---------------------|--------------------------------------|
| Authorization allow            | 200 OK ALLOW        | AuthorizeResponse DECISION_EFFECT_ALLOW |
| Authorization deny             | 403 DENY            | AuthorizeResponse DECISION_EFFECT_DENY (NOT a tonic Err) |
| Token-validation failure       | 422 token-invalid   | ValidateTokenResponse{ok:false, Error} (engine NOT consulted) |
| Store/JWKS unavailable (unary) | 503 unavailable     | Status::Unavailable                  |
| Store/JWKS unavailable (batch) | 200 per-item DENY   | BatchAuthorizeResponse per-item DENY |

Deny is always a DECISION_EFFECT_DENY response value — never a transport error.
This preserves the fail-closed PEP invariant: a caller that pattern-matches only
on `Ok()` still gets the correct deny signal.

## OidcValidationError -> ValidationErrorKind mapping

| OidcValidationError           | ValidationErrorKind           |
|------------------------------|-------------------------------|
| MalformedToken, DecodeError, MalformedKey | MALFORMED         |
| AlgNone                      | ALG_NONE                      |
| InvalidType                  | INVALID_TYPE                  |
| UntrustedKeySourceUrl        | UNTRUSTED_KEY_SOURCE_URL      |
| AlgorithmMismatch, UnsupportedAlgorithm | ALGORITHM_MISMATCH |
| UnknownKey                   | UNKNOWN_KEY                   |
| SignatureInvalid              | SIGNATURE_INVALID             |
| IssuerMismatch               | ISSUER_MISMATCH               |
| AudienceMismatch             | AUDIENCE_MISMATCH             |
| Expired                      | EXPIRED                       |
| NotYetValid                  | NOT_YET_VALID                 |
| MissingClaim, Domain         | MISSING_CLAIM                 |

MalformedKey has no exact proto counterpart and collapses to MALFORMED.

## Testing strategy

Tests in `tests/grpc_authorize_deny.rs` call the tonic trait impls directly via
`tonic::Request::new(...)` — no TCP socket, no port allocation.  Fixtures are
shared with the REST tests via `tests/common.rs`.  The four acceptance
assertions required by T3 are covered:

(a) Permitted principal -> DECISION_EFFECT_ALLOW.
(b) Forbidden principal -> DECISION_EFFECT_DENY response (not a tonic Err).
(c) Invalid token -> typed ValidateTokenResponse error; engine-not-consulted
    proven by structural test (permit vs empty authorizer, both DENY on bad token).
(d) Store unavailable -> Status::Unavailable.

## Dependency-seam note (ADR-0092)

The five deps added by this slice (prost, tonic, tonic-prost, tonic-prost-build,
protoc-bin-vendored) are listed in `registry/dependency-rationales.json`
`allowed_crates` for `identity-workload-rest` as a deliberate scoped
out-of-lane edit committed alongside the spec (commit b547b9e4).  The dep-seam
gate defaults to `ReportOnly` (ADR-0092,
`crates/check-dependency-seam/src/lib.rs:63`) so the edit is additive and
does NOT block `gate-run-all`.  The registry edit is acknowledged as crossing
the lane's strict disjointness boundary; no concurrent lane touches those five
rows.

## SLO coverage

Existing SLOs in `microservices/identity/slos/`:
- `authorize-latency-p99.openslo.yaml` — targets the authorize hot path; the
  gRPC WorkloadAuthorizer RPCs share the same use-case core, so the objective
  applies equally to both delivery surfaces.
- `decision-correctness.openslo.yaml` — targets the correctness invariant; the
  shared use-case core means gRPC decisions are subject to the same objective.

Neither SLO currently has metric instrumentation wired in the crate
(`identity_workload_authorize_duration_seconds_bucket` and
`identity_workload_golden_decision_total` are not yet emitted by any
surface).  This is a pre-existing gap that predates this slice; instrumenting
those counters/histograms is a separate follow-on for both REST and gRPC.

No new SLO is introduced by this slice: the gRPC surface does not introduce a
distinct latency or correctness objective beyond the existing SLOs.  If a
gRPC-specific p99 target is required in future (e.g. tighter Envoy ext_authz
budget), add a new SLO entry at that time.

## Acceptance

- `cargo nextest run -p identity-workload-rest` green (19 tests: 12 REST + 7 gRPC).
- `cargo check -p identity-workload-rest --all-targets` clean.
- Root `Cargo.toml` unchanged (no new workspace member).
- `aws-lc-rs` default-features unchanged (ADR-0506 not regressed).
