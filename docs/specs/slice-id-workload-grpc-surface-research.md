# Research: workload-identity gRPC surface (slice-id-workload-grpc-surface)

This document records the canonical MUST/SHOULD rules, pitfalls, and how they
constrain the implementation of the tonic gRPC delivery surface for
`crates/identity-workload-rest`.  It is a companion to
`docs/specs/slice-id-workload-grpc-surface.md` (the architectural spec) and
`tasks/id-workload-grpc-surface-plan.md` (the plan).

All claims are grounded in sources cited at the end of each section.

---

## 1. tonic 0.14 service implementation

### 1.1 Trait generation and `#[tonic::async_trait]`

**MUST**: Every tonic service impl block carries `#[tonic::async_trait]`.  tonic
generates a server trait (e.g. `WorkloadAuthorizer`) whose methods are `async`.
The `async_trait` attribute is the macro bridge that makes a `struct`'s `async fn`
satisfy the generated trait, which internally uses `BoxFuture`.  Without the
attribute the impl will not compile.

**MUST**: The server struct and all its type-parameter bounds must satisfy
`Send + Sync + 'static`.  tonic's generated service wrapper (`WorkloadAuthorizerServer<T>`)
requires `T: WorkloadAuthorizer` where the trait bounds include `Send + Sync + 'static`.
This propagates to every generic parameter on the server struct — all four of
`R, D, A, S` in `WorkloadGrpcServer<R, D, A, S>` must carry `Send + Sync + 'static`.

**Pitfall**: Forgetting `Sync` on one bound while having it on others causes a
confusing "trait not implemented" error at the `add_service` call site (or at the
`tonic::Server::builder().add_service(WorkloadAuthorizerServer::new(...))` line)
rather than at the impl block.  Always add `Sync` to every bound on a struct
intended for a tonic server.

Source: https://docs.rs/tonic/0.14.6/tonic/ (trait bounds section);
Context7 `/websites/rs_tonic_0_14_6_tonic`

### 1.2 `tonic::include_proto!` macro

**MUST**: Use `tonic::include_proto!("package.name")` to include generated code,
not a manual `include!` with an `OUT_DIR` path.  The macro expands to
`include!(concat!(env!("OUT_DIR"), "/package.name.rs"))` and is the stable,
version-portable way to pull in codegen output.

**MUST**: The argument is the proto *package* name (dots preserved), not the
file name.  For `package oyatie.identity.workload.v1` the argument is
`"oyatie.identity.workload.v1"`.

**Pitfall**: Using `include_proto!("workload")` (file stem) instead of
`include_proto!("oyatie.identity.workload.v1")` (package name) causes a compile-time
`include!` expansion failure because the generated file is named after the
package, not the source file.

Source: https://docs.rs/tonic/0.14.6/tonic/macro.include_proto.html

### 1.3 Proto enum values and `as i32` casts

**MUST**: Proto3 enums compile to `i32` fields in prost-generated Rust structs.
Setting an enum field requires casting: `effect: DecisionEffect::Deny as i32`.
Reading it back requires `DecisionEffect::try_from(response.effect)` or a direct
comparison against the casted constant (`response.effect == DecisionEffect::Deny as i32`).

**MUST**: The zero/default value of every proto3 enum (`DECISION_EFFECT_UNSPECIFIED = 0`,
`VALIDATION_ERROR_KIND_UNSPECIFIED = 0`) is the protobuf wire default.  A response
whose `effect` field is not set on the wire will decode as `0`.  Tests that assert
on `DecisionEffect::Deny as i32` (value `2`) are thus unambiguous.

**Pitfall**: Comparing `response.effect == DecisionEffect::Deny` (without `as i32`)
does not compile because the field type is `i32`, not the enum.  Always cast.

Source: https://docs.rs/prost/0.14.3/prost/ (enum encoding section);
https://github.com/hyperium/tonic (generated code conventions)

### 1.4 `oneof` fields in generated Rust

**MUST**: A proto3 `oneof` block generates a Rust `enum` in a sub-module named
after the containing message (snake_case).  For `ValidateTokenResponse.oneof outcome`
with variants `principal` and `error`, the generated type is
`validate_token_response::Outcome` with variants `Principal(VerifiedPrincipal)` and
`Error(ValidationError)`.  The containing message has field
`outcome: Option<validate_token_response::Outcome>`.

**Pitfall**: Accessing `response.principal` directly (as if the oneof fields were
top-level) does not compile.  Always pattern-match on `response.outcome`.

Source: https://docs.rs/prost/0.14.3/prost/ (oneof section)

---

## 2. tonic-prost-build codegen wiring (build.rs inside existing crate)

### 2.1 Workspace version: tonic-prost-build, not tonic-build

**MUST**: This workspace uses `tonic-prost-build` (version `0.14.6`), the prost
2.0 split of the original `tonic-build`.  The API is identical except the crate
name.  Using `tonic_build::configure()` will fail to compile because `tonic-build`
is not in the workspace.

**MUST**: In `build.rs`:
```rust
tonic_prost_build::configure()
    .build_client(false)
    .build_server(true)
    .emit_rerun_if_changed(false)  // manual rerun-if-changed lines used instead
    .compile_protos(&[&proto_file], &[&proto_root])?;
```

**MUST**: Provide the vendored `protoc` via `protoc-bin-vendored` before calling
`compile_protos`.  Set `PROTOC` via `std::env::set_var` in the build script.
This is the workspace-established pattern (see
`crates/shared-backbone-grpc-generated-adapter/build.rs`).

**Pitfall**: Not setting `PROTOC` causes the build to try to find `protoc` on
`PATH`, which fails in hermetic CI environments where protoc is not installed.

**Pitfall**: Using `emit_rerun_if_changed(true)` (the default) together with
manual `cargo:rerun-if-changed` lines causes duplicate rerun triggers and
incremental-build noise.  This workspace pattern uses `emit_rerun_if_changed(false)`
and emits `cargo:rerun-if-changed` for the proto directory and file manually.

Source: https://docs.rs/tonic-prost-build (tonic-prost-build 0.14.x);
https://crates.io/crates/tonic-prost-build/0.14.2;
Workspace pattern: `crates/shared-backbone-grpc-generated-adapter/build.rs`

### 2.2 No new workspace member required

**MUST NOT** add a new workspace member to the root `Cargo.toml` when adding a
`build.rs` to an existing crate.  `build.rs` is a per-crate build script; it
requires only `[build-dependencies]` in the crate's own `Cargo.toml`.

**MUST NOT** change `root Cargo.toml` at all.  Disjointness requires this
lane touch only `crates/identity-workload-rest/`.

Source: Cargo reference https://doc.rust-lang.org/cargo/reference/build-scripts.html

### 2.3 tonic feature selection

**MUST**: Enable `features = ["router", "transport"]` on the tonic dependency for
server-side use.  The `router` feature provides `tonic::transport::server::Router`
(needed for `Server::builder().add_service(...)`); the `transport` feature
provides `tonic::transport::Server`.  Without these features the generated server
stubs compile but cannot be wired into a running server or used in in-process
tests via `tower::ServiceExt`.

**SHOULD**: `build_client(false)` reduces codegen output for a server-only crate.

Source: https://docs.rs/tonic/0.14.6/tonic/transport/index.html;
Context7 `/websites/rs_tonic_0_14_6_tonic`

---

## 3. Envoy ext_authz gRPC contract and fail-closed PEP semantics

### 3.1 ext_authz protocol overview

Envoy's external authorization filter (`ext_authz`) calls an authorization
service implementing `envoy.service.auth.v3.Authorization` (a standard gRPC
service with `Check(CheckRequest) -> CheckResponse`).

The workload-identity gRPC surface (`WorkloadAuthorizer`, `WorkloadTokenValidator`)
is **not** the Envoy `envoy.service.auth.v3.Authorization` service — it is a
**custom authorization backend** that PEPs (Envoy waypoint, sidecars, api-gateway)
call directly.  The ext_authz protocol is noted here because the PEP layer that
calls this service follows the same fail-closed principles.

### 3.2 Envoy ext_authz failure-mode semantics

When the authorization backend returns a **non-OK gRPC status** (e.g.
`UNAVAILABLE`, `INTERNAL`, `DEADLINE_EXCEEDED`), Envoy ext_authz treats this as
an **error** condition, distinct from an explicit deny:

- With `failure_mode_allow: false` (the secure default): the request is **denied**
  (fail-closed).  The `ext_authz_error` stat is incremented.
- With `failure_mode_allow: true`: the request is **allowed** (fail-open, unsafe).

An **explicit deny** is signaled by a `CheckResponse` with `status.code != OK`
inside the response body (specifically `google.rpc.Status` with a non-0 code in
`denied_response`), or equivalently by returning `Status::OK` at the gRPC layer
but with a `DeniedHttpResponse` in the `CheckResponse` body.

**The critical distinction for this slice**:

| Condition | gRPC transport status | Envoy/PEP behavior |
|---|---|---|
| Explicit authorization deny | `Status::OK` with DENY payload | Deny with configured HTTP status (default 403) |
| Backend error / store unavailable | `Status::Unavailable` | Deny (fail-closed) if `failure_mode_allow=false` |
| Backend error / store unavailable | `Status::Unavailable` | Allow (fail-open) if `failure_mode_allow=true` |

**MUST**: A deny decision MUST be encoded as an `AuthorizeResponse` value with
`effect: DECISION_EFFECT_DENY` inside a successful `Status::OK` gRPC response.
Never encode a deny as a non-OK gRPC status.

Rationale: A non-OK status would be indistinguishable from a backend error to
the PEP layer.  The PEP's behaviour on non-OK depends on `failure_mode_allow`,
which means a misconfigured PEP (failure_mode_allow=true) would allow a request
that should have been denied.  Encoding deny as a response value is the only
encoding that is unconditionally fail-closed regardless of PEP configuration.

**MUST**: A store/JWKS unavailable condition — where the backend cannot determine
authorization — MUST return `Status::Unavailable` (non-OK), NOT a DENY response
value.  With `failure_mode_allow=false` this still denies the request, but it
also signals to the PEP that the backend is degraded (surfacing in metrics,
circuit-breaker logic, retry policies).  Encoding a store outage as a DENY
response value would mask the outage and prevent the PEP from triggering
back-off or alerting.

Source:
- https://www.envoyproxy.io/docs/envoy/latest/api-v3/service/auth/v3/external_auth.proto
- https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/ext_authz_filter
- https://github.com/envoyproxy/envoy/issues/10158 (failure_mode_allow + 5xx behaviour)

### 3.3 Batch requests and fail-closed encoding

For `AuthorizeBatch`, the gRPC response is always `Status::OK` at the transport
layer; errors on individual items are encoded as per-item `DECISION_EFFECT_DENY`
values in the `BatchAuthorizeResponse.decisions` repeated field.

**MUST**: A store/JWKS outage on a single batch item MUST produce a `DENY`
decision value for that item (not a top-level `Status::Unavailable`), because the
batch contract requires all items to be decided, and a transport-level error would
abort the entire batch rather than giving per-item results.

This matches the REST batch handler's behaviour: a 503-class error on a single
batch item is reflected as a DENY decision in the per-item body, while the batch
response HTTP status is 200.

---

## 4. proto3 enum mapping and tonic Status code selection

### 4.1 `DECISION_EFFECT_DENY` vs tonic error

| Situation | gRPC return |
|---|---|
| Allow | `Ok(Response::new(AuthorizeResponse { effect: DECISION_EFFECT_ALLOW as i32, ... }))` |
| Deny (any reason) | `Ok(Response::new(AuthorizeResponse { effect: DECISION_EFFECT_DENY as i32, ... }))` |
| Store/JWKS unavailable (unary) | `Err(Status::unavailable("..."))` |
| Token-validation failure | `Ok(Response::new(ValidateTokenResponse { ok: false, outcome: Error(...) }))` |
| Store/JWKS unavailable (batch item) | `Ok` response, per-item DENY value |

**MUST**: `Status::unavailable` (gRPC `UNAVAILABLE = 14`) is the correct code for
a transient dependency outage.  It signals retryable unavailability to the caller.
Do not use `Status::internal` (a permanent programming error), `Status::unknown`,
or `Status::failed_precondition` for a transient store outage.

Source: https://docs.rs/tonic/0.14.6/tonic/struct.Status.html (unavailable constructor)

### 4.2 Token-validation failure: typed response not transport error

**MUST**: A token that fails OIDC validation (signature, expiry, issuer, etc.)
MUST return a successful `ValidateTokenResponse { ok: false, outcome: Error(...) }`,
not `Err(Status::unauthenticated(...))` or any other transport error.

Rationale: the proto contract encodes the typed `ValidationErrorKind` for the
caller to inspect.  A transport error would lose the error kind and prevent the
caller from distinguishing "malformed token" from "expired token" from "unknown
key".  The PEP layer consults the typed error to decide how to respond to the
upstream client (e.g. 401 vs 422 vs 503).

**MUST**: When token validation fails, the policy engine MUST NOT be consulted.
The code structure enforces this: `validate_workload_token` returns `Err` before
the `authorize` path is reached.

### 4.3 `OidcValidationError` to `ValidationErrorKind` collapse

The OIDC adapter has more error variants than the proto enum has kinds.  The
mapping collapses as follows:

| OidcValidationError variants | ValidationErrorKind |
|---|---|
| `MalformedToken`, `DecodeError`, `MalformedKey` | `MALFORMED` |
| `AlgNone` | `ALG_NONE` |
| `InvalidType` | `INVALID_TYPE` |
| `UntrustedKeySourceUrl` | `UNTRUSTED_KEY_SOURCE_URL` |
| `AlgorithmMismatch`, `UnsupportedAlgorithm` | `ALGORITHM_MISMATCH` |
| `UnknownKey` | `UNKNOWN_KEY` |
| `SignatureInvalid` | `SIGNATURE_INVALID` |
| `IssuerMismatch` | `ISSUER_MISMATCH` |
| `AudienceMismatch` | `AUDIENCE_MISMATCH` |
| `Expired` | `EXPIRED` |
| `NotYetValid` | `NOT_YET_VALID` |
| `MissingClaim(_)`, `Domain(_)` | `MISSING_CLAIM` |

`MalformedKey` collapses to `MALFORMED` because the proto has no `MALFORMED_KEY`
kind.  `Domain` collapses to `MISSING_CLAIM` as the closest semantic fit.

The `detail` field in the proto `ValidationError` carries the full
`error.to_string()` for operator diagnostics regardless of kind.

Source: `microservices/identity/contracts/proto/workload.proto` (ValidationErrorKind enum);
`crates/identity-workload-oidc-adapter/src/lib.rs` (OidcValidationError enum)

---

## 5. Crypto path and ADR-0506 non-regression

**MUST NOT** set `default-features = false` on `aws-lc-rs` anywhere in
`crates/identity-workload-rest/Cargo.toml`.  ADR-0506 mandates `aws-lc-rs`
as the crypto backend; disabling default features can silently disable the
`aws-lc-rs` feature flag, falling back to the ring/openssl backend in TLS
libraries.

**MUST NOT** add `ring` as a non-dev dependency.  `ring` is in `[dev-dependencies]`
for test JWT minting only (existing pattern from `tests/rest_endpoints.rs`); it
is OSI-clean but must not enter the production dependency graph.

The tonic/prost dependency additions in this slice do not touch the crypto path:
tonic's TLS features are not enabled (no `tls` feature in the `tonic` dep entry),
and prost has no crypto dependency.

Source: `docs/adr-archive/ADR-0506-aws-lc-rs-canonical-crypto-provider.md`; workspace `Cargo.toml`
(aws-lc-rs workspace dep, no default-features override)

---

## 6. In-process test strategy for tonic services

**MUST**: Tests call the tonic trait impls directly via
`service_struct.rpc_method(tonic::Request::new(proto_request)).await`.  No TCP
socket or port allocation is needed.  This avoids port-allocation flakiness and
makes tests hermetic.

**MUST**: The generated server trait methods take `Request<T>` and return
`Result<Response<U>, Status>`.  Both `Request::new` and `Response::into_inner`
are available without the transport feature — they live in `tonic` (not
`tonic::transport`).

**SHOULD**: Share fixtures between REST and gRPC tests via a `tests/common.rs`
module.  Both test binaries can declare `mod common;` because `tests/` files in
Cargo are integration-test binaries that each compile independently; a module
declaration in each binary resolves to the same source file.

**Pitfall**: Using `tests/common.rs` as an independent test binary (i.e. placing
it without a `mod common;` declaration in the consuming test file) will cause
Cargo to try to compile it as a standalone integration test, which will fail
because it has no `#[test]` functions.  Always declare `mod common;` from the
consuming test file.

Source: Cargo integration test documentation https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests

---

## 7. Shared-core design constraint

**MUST**: The gRPC delivery module MUST delegate all authorization and validation
decisions to the same use-case functions the REST handlers call.  The specific
shared-core mapping:

| gRPC RPC | Shared core |
|---|---|
| `AuthorizeWithToken` | `identity_workload_app::authorize_with_token` |
| `AuthorizeBatch` (per item) | `identity_workload_app::authorize_with_token` |
| `Authorize` | `build_active_principal` (crate fn in `src/lib.rs`) + `authorizer.authorize` |
| `ValidateToken` | `identity_workload_oidc_adapter::validate_workload_token` |

The `Authorize` RPC uses `build_active_principal` because the REST `/authorize`
handler also uses `build_active_principal` — this IS the shared core for that
path, not the `-app` use-case.  This is not a logic duplication; the function is
`pub(crate)` and is the definitive principal-construction logic for the
"trusted-PEP asserts principal" path.

**MUST NOT** reproduce decision logic (Cedar evaluation, OIDC validation, deny
thresholds) in the gRPC module.  Any divergence would create a dual-maintenance
surface and risk semantic drift between the two delivery layers.

---

## 8. Single-crate-per-service constraint (ADR-0509)

**MUST NOT** add a new crate to the workspace for the gRPC surface.  The
implementation lives in `src/grpc/mod.rs` inside the existing
`identity-workload-rest` crate.

**MUST NOT** edit the root `Cargo.toml` workspace member list.

**Pitfall**: The established pattern in this repo for other services places gRPC
delivery in standalone `*-grpc` crates (e.g. `shared-backbone-grpc-*`,
`payments-charge-grpc`).  This pattern is correct for those services (they
ARE standalone service crates).  For the identity workload vertical the existing
`-rest` crate is the single service crate under ADR-0509; the gRPC surface is
a delivery module inside it, not a second service crate.

Source: `docs/adr-archive/ADR-0509-hyperscaler-service-decomposition-pattern.md`;
`tasks/id-workload-grpc-surface-plan.md` (ARCHITECTURE-DOCTRINE section)

---

## 9. Audit emission constraint

**MUST**: Every authorize RPC and every `ValidateToken` RPC emits exactly one
`AuditRecord` before the response is returned, regardless of outcome (allow, deny,
token-rejected, store-unavailable).  This mirrors the REST surface and satisfies
AC-W-13.

**MUST**: Audit emission on the error path (store-unavailable, token-rejected)
MUST precede the `return Err(Status::unavailable(...))` or the `Ok(...)` call.
If the audit emission is skipped on error, the decision stage is lost from the
audit chain.

**MUST NOT**: An audit sink error must not surface as a deny or as a transport
error.  Emission is best-effort; swallow sink errors.

Source: `microservices/identity/workload-identity/PRD.md` §3.3 (AC-W-13);
`src/lib.rs` (AuditSink trait, InMemoryAuditSink impl)

---

## 10. Summary: MUST/SHOULD rules for implementers

| # | Rule | Where enforced |
|---|---|---|
| M-01 | `#[tonic::async_trait]` on every service impl block | Compiler |
| M-02 | All type params on server struct: `Send + Sync + 'static` | Compiler |
| M-03 | `tonic::include_proto!("oyatie.identity.workload.v1")` (package name, not file stem) | Compiler |
| M-04 | Proto enum fields set as `EnumVariant as i32` | Compiler |
| M-05 | Deny always `Ok(Response)` with DECISION_EFFECT_DENY, never `Err(Status)` | Test (b) |
| M-06 | Store/JWKS unavailable (unary): `Err(Status::unavailable(...))` | Test (d) |
| M-07 | Token-validation failure: `Ok(Response)` with `ok:false`, engine not consulted | Test (c) |
| M-08 | Batch item outage: per-item DENY value, not top-level Unavailable | Design |
| M-09 | `tonic-prost-build`, not `tonic-build` (workspace version) | Build |
| M-10 | `PROTOC` env var set from `protoc-bin-vendored` in build.rs | Build |
| M-11 | `features = ["router", "transport"]` on tonic dep | Build |
| M-12 | No `default-features = false` on `aws-lc-rs` (ADR-0506) | CI gate |
| M-13 | No new workspace member, root Cargo.toml unchanged | CI gate |
| M-14 | One AuditRecord per RPC call, emitted before response, including error paths | Test (e) |
| M-15 | No authorization/validation logic in grpc/mod.rs; delegate to shared core | Code review |
| S-01 | `build_client(false)` in build.rs (server-only crate) | Build |
| S-02 | Shared test fixtures via `tests/common.rs` (REST+gRPC parity visible) | Tests |

---

## Sources

- tonic 0.14.6 API docs: https://docs.rs/tonic/0.14.6/tonic/
- tonic `include_proto!` macro: https://docs.rs/tonic/0.14.6/tonic/macro.include_proto.html
- tonic `Status::unavailable`: https://docs.rs/tonic/0.14.6/tonic/struct.Status.html
- tonic transport: https://docs.rs/tonic/0.14.6/tonic/transport/index.html
- tonic-prost-build 0.14.x: https://docs.rs/tonic-prost-build
- prost 0.14 (enum / oneof encoding): https://docs.rs/prost/0.14.3/prost/
- Envoy ext_authz filter config: https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/ext_authz_filter
- Envoy ext_authz proto (CheckResponse): https://www.envoyproxy.io/docs/envoy/latest/api-v3/service/auth/v3/external_auth.proto
- Envoy failure_mode_allow + 5xx issue: https://github.com/envoyproxy/envoy/issues/10158
- Cargo build scripts: https://doc.rust-lang.org/cargo/reference/build-scripts.html
- Cargo integration tests: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests
- Local: `microservices/identity/contracts/proto/workload.proto`
- Local: `crates/identity-workload-rest/Cargo.toml`
- Local: `crates/identity-workload-rest/build.rs`
- Local: `crates/identity-workload-rest/src/grpc/mod.rs`
- Local: `docs/specs/slice-id-workload-grpc-surface.md`
- Local: `tasks/id-workload-grpc-surface-plan.md`
