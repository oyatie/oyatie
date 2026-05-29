# Plan: id-workload-grpc-surface

## Goal

Deliver a tonic gRPC surface for the workload-identity service that implements
`WorkloadAuthorizer` and `WorkloadTokenValidator` from
`microservices/identity/contracts/proto/workload.proto`, mounted inside the
existing `crates/oya-identity-workload-rest` crate (ADR-0509 single-crate
extension), sharing the `oya-identity-workload-app` use-case core with the REST
surface to eliminate logic drift between the two delivery layers.

## Architecture decisions (resolving plan-review blockers)

### ARCHITECTURE-DOCTRINE (CRITICAL resolved)

ADR-0509 mandates single-crate-per-service.  The identity workload vertical
currently has 6 crates (legacy clean-arch sprawl).  The plan adds gRPC delivery
as `src/grpc/` inside `oya-identity-workload-rest` rather than a new `-grpc`
sibling crate.  This is the correct ADR-0509 interim step: do not add sprawl,
put the delivery mod inside the existing service crate.  A future collapse of
the 6 identity crates into one is out of scope for this slice.

### DEPENDENCY-SEAM (CRITICAL resolved — non-blocking)

The dep-seam gate (`crates/oya-check-dependency-seam/`) defaults to `ReportOnly`
(`src/lib.rs:63`; dev-cli `dependency_seam_gates.rs:24`).  Adding tonic/prost
deps to `oya-identity-workload-rest` fires `SEAM_DEP_DECL_OUTSIDE_ISOLATED_CRATE`
and `SEAM_IMPORT_OUTSIDE_ISOLATED_CRATE` unless `registry/dependency-rationales.json`
`allowed_crates` includes `oya-identity-workload-rest`.  The registry update
appending `oya-identity-workload-rest` to the five dep rows was committed as a
deliberate scoped out-of-lane edit (commit b547b9e4) alongside the spec.  The
gate is ReportOnly so this does NOT block `gate-run-all`; no concurrent lane
touches those rows.

### BUILD WIRING (MAJOR resolved)

The workspace uses tonic 0.14.6.  The correct build dependencies are:
- `prost.workspace = true`
- `tonic = { workspace = true, features = ["router", "transport"] }`
- `tonic-prost.workspace = true`
- build-deps: `tonic-prost-build.workspace = true` + `protoc-bin-vendored.workspace = true`

Plain `tonic-build` is the tonic-0.13 name and does NOT exist in this workspace.
The `build.rs` mirrors `crates/oya-shared-backbone-grpc-generated-adapter/build.rs`
with `build_server(true)`, `build_client(false)` (server-side PEP only).

### AUTHORIZE vs AuthorizeWithToken (MINOR resolved)

`AuthorizeWithToken` and `AuthorizeBatch` delegate to `authorize_with_token`
(the `-app` use-case).  `Authorize` uses `build_active_principal` (a
crate-private fn in `src/lib.rs`) + `authorizer.authorize` — mirroring the REST
`/authorize` handler.  The shared core for the `Authorize` RPC is
`build_active_principal`, not the `-app` crate.  No logic is duplicated; the
fn is `pub(crate)`.

### T3 TEST SHAPE (MAJOR resolved)

Tests call tonic trait impls directly via `tonic::Request::new(...)`.  No TCP
socket or port allocation required.  Fixtures shared via `tests/common.rs`
(hoisted from the REST test file's private fns; common.rs is a module in the
`tests/` tree, so both test binaries can `mod common;`).

### BATCH FAIL-CLOSED PARITY (MINOR resolved)

A store/JWKS outage on a single batch item returns a DENY decision VALUE in that
item's `AuthorizeResponse`, matching REST batch semantics (HTTP 200 with per-item
deny).  A top-level `Status::Unavailable` is reserved for unary RPCs.

## Tasks

| ID  | Description                                   | Status    |
|-----|-----------------------------------------------|-----------|
| T1  | Cargo.toml deps + build.rs proto codegen      | done      |
| T2  | src/grpc/ tonic impls                         | done      |
| T3  | tests/grpc_authorize_deny.rs + common.rs      | done      |
| T4  | Lane docs + SLO review                        | done      |

## Acceptance criteria

- `cargo check -p oya-identity-workload-rest --all-targets` clean.
- `cargo nextest run -p oya-identity-workload-rest` green (19 tests: 12 REST + 7 gRPC; all 4 T3 assertions covered).
- Root Cargo.toml unchanged.
- No aws-lc-rs default-features regression.
- Three lane docs exist and describe surface + fail-closed + parity.
- microservices/identity/slos/*.openslo.yaml covers gRPC paths (no new SLO required).
