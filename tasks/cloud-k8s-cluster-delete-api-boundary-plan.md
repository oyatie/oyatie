# Plan: cloud-k8s-cluster-delete-api-boundary

Vertical: infra
Crate: oya-cloud-compute-k8s-api
Branch: feat/task-cloud-k8s-cluster-delete-api-boundary-2026-05-28

## Objective

Extend the existing `oya-cloud-compute-k8s-api` boundary crate with a
tenant-safe, idempotent cluster DELETE (teardown) request surface that mirrors
the create-path discipline. Pure boundary logic only — no provider adapters or
reconciliation I/O.

## Subtasks

### [k8s-del-1] Surface constant + request type + boundary/principal/authorization validators

Add:
- `CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE: &str = "cloud.compute.k8s.cluster.delete"`
- `CloudComputeK8sClusterDeleteApiRequest` struct: `path_cluster_id`, `boundary`
  (`CloudComputeK8sApiBoundaryContext`), `principal`
  (`CloudComputeK8sApiPrincipal`), `authorization`
  (`CloudComputeK8sApiAuthorization`). No mutable body beyond cluster identity.

Reuse existing validators:
- `validate_boundary` — request-id / tenant / idempotency-key emptiness
- `validate_cluster_resource_id` — parse + kind check
- `validate_authorization` — decision-id, tenant-match, principal-match,
  allowed-surfaces check against the new delete surface constant

New delete-specific tenant validator:
- `validate_delete_tenant_binding` — path cluster-id tenant extracted from
  `ResourceId` must equal `boundary.tenant_id` and `principal.tenant_id`

Acceptance:
- `cargo check -p oya-cloud-compute-k8s-api --all-targets` passes
- Unit tests assert empty request-id / tenant / idempotency-key / principal and
  a missing delete surface in `allowed_surfaces` each map to the correct
  `CloudComputeK8sApiErrorCode` with the existing 400/401/403 status mapping

### [k8s-del-2] validate + execute delete with idempotency and state projection

Add:
- `validate_cloud_compute_k8s_cluster_delete_request` — calls boundary, path-id
  parse, delete tenant binding, authorization validators in order
- `CloudComputeK8sDeleteIdempotencyLedger` (sibling to the create ledger,
  keyed on the same `CloudComputeK8sIdempotencyLedgerKey` type but private to
  the delete path) — or reuse `CloudComputeK8sCreateIdempotencyLedger` with a
  delete surface key so a replayed delete key returns the identical typed result
  rather than a second teardown
- `delete_cloud_compute_k8s_cluster_from_api` — validates, checks idempotency
  ledger, looks up the cluster record in the `CloudComputeCatalog`, projects its
  state to `Deleting` / `Deleted` (via the domain's existing state model),
  records the ledger entry, returns a typed
  `CloudComputeK8sClusterDeleteSuccessResponse`
- `delete_cluster` — stable planned entrypoint that delegates to the above

Projection rules:
- If cluster exists and is not already `Deleting`/`Deleted`: project to
  `Deleting` (202 Accepted) — boundary records the idempotency key as
  in-flight
- If cluster does not exist in the catalog: `ComputeNotFound` → 404
- If path-vs-authorization tenant mismatch: `TenantMismatch` → 403

Acceptance:
- `cargo nextest run -p oya-cloud-compute-k8s-api` green
- Tests cover:
  - Happy-path delete (202 status + `Deleting` projection)
  - Tenant mismatch → `Forbidden` (403)
  - Unknown/missing cluster → `NotFound` (404)
  - Idempotency-key replay returning the identical typed response with no
    double-teardown

### [k8s-del-3] Error response mapping + rustdoc + additive-only diff

Add:
- `CloudComputeK8sClusterDeleteApiStatus` enum with `Accepted`(202) +
  shared error codes (400/401/403/404/422)
- `cluster_delete_status` / `cluster_delete_status_code` methods on
  `CloudComputeK8sApiError` mirroring `cluster_create_status`
- `error_response` already exists on `CloudComputeK8sApiError` — reuse it;
  the delete path calls it with the delete request's `request_id` so the id
  round-trips in the error body
- Crate-level rustdoc updated to describe the delete surface alongside the
  create surface

Invariants:
- No existing create/validate function signatures or error variants altered
- Root `Cargo.toml` not touched; no new workspace member

Acceptance:
- `cargo check -p oya-cloud-compute-k8s-api --all-targets` green
- `cargo nextest run -p oya-cloud-compute-k8s-api` green
- A test asserts the delete error response `request_id` round-trips and the
  error body shape matches the create surface
- `git diff` shows only additive changes to `crates/oya-cloud-compute-k8s-api/`

## Acceptance Summary

| Gate | Command |
|------|---------|
| Type-check | `cargo check -p oya-cloud-compute-k8s-api --all-targets` |
| Tests | `cargo nextest run -p oya-cloud-compute-k8s-api` |
| Diff scope | Additive only; root `Cargo.toml` unchanged |

## Boundaries

- This task owns only `crates/oya-cloud-compute-k8s-api/`.
- No provider adapter, reconciler, or HTTP handler is introduced.
- No new workspace crate is created.
- All error variants and success types follow the `CloudComputeK8s*` naming
  convention already established by the create path.
