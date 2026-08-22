# Spec: Cloud Compute K8s Cluster Delete API Boundary

| Field | Value |
|-------|-------|
| Task slug | `cloud-k8s-cluster-delete-api-boundary` |
| Vertical | infra |
| Crate | `cloud-compute-k8s-api` |
| Branch | `feat/task-cloud-k8s-cluster-delete-api-boundary-2026-05-28` |
| Stage | SPEC |

## Objective

Extend the `cloud-compute-k8s-api` boundary crate with a tenant-safe,
idempotent cluster DELETE (teardown) request-boundary surface. The surface
mirrors the existing create-path discipline — request-id / tenant /
idempotency-key normalization, authorization-proof checks against a new
`cloud.compute.k8s.cluster.delete` surface, path cluster-id well-formedness
and tenant-match validation, and a typed delete success/error response that
projects the cluster into a `Deleting` / `Deleted` state.

Scope is pure boundary logic. No provider adapters, no reconciliation I/O, no
new workspace crate.

## Vertical and Crate Context

The `cloud-compute-k8s-api` crate already owns:

- `CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE` surface constant
- `CloudComputeK8sApiBoundaryContext` — request-id / tenant / idempotency-key
- `CloudComputeK8sApiPrincipal` — tenant + principal identity
- `CloudComputeK8sApiAuthorization` — decision-id, tenant, principal,
  allowed-surfaces
- `CloudComputeK8sApiError` — typed error enum with `code()`, `message()`,
  `details()`, `error_response()`, `cluster_create_status()` helpers
- `CloudComputeK8sCreateIdempotencyLedger` — BTreeMap keyed on
  `(tenant_id, principal_id, surface, idempotency_key)`
- `validate_boundary`, `validate_cluster_resource_id`,
  `validate_authorization` — private validators reused by the delete path

The delete surface reuses all of the above structural types and private helpers
without modifying any of their signatures.

## Module Layout (flat clean-arch, mods inside `src/lib.rs`)

```
src/lib.rs
  // --- existing create surface (unchanged) ---
  pub const CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE
  pub struct CloudComputeK8sClusterCreateApiRequest
  pub fn validate_cloud_compute_k8s_cluster_create_request
  pub fn create_cloud_compute_k8s_cluster_from_api
  pub fn create_cluster   (stable planned entrypoint)

  // --- new delete surface (additive) ---
  pub const CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE
  pub enum  CloudComputeK8sClusterDeleteApiStatus
  pub struct CloudComputeK8sClusterDeleteApiRequest
  pub struct CloudComputeK8sClusterDeleteSuccessResponse
  pub struct CloudComputeK8sDeleteIdempotencyLedger
  pub fn validate_cloud_compute_k8s_cluster_delete_request
  pub fn delete_cloud_compute_k8s_cluster_from_api
  pub fn delete_cluster   (stable planned entrypoint)

  // --- shared (additive methods on existing types) ---
  impl CloudComputeK8sApiError
    pub fn cluster_delete_status      (new method, mirrors cluster_create_status)
    pub fn cluster_delete_status_code (new method)
```

All new code lives inside the existing single `src/lib.rs` file per the
flat-clean-arch / single-crate-per-service doctrine (ADR-0509).

## Contracts

### OpenAPI 3.2.0 fragment

```yaml
paths:
  /v1/clusters/{cluster_id}:
    delete:
      operationId: deleteCloudComputeK8sCluster
      summary: Request cluster teardown
      parameters:
        - name: cluster_id
          in: path
          required: true
          schema:
            type: string
            example: "oya:cloud:region-home:ten_alpha:k8s:prod"
        - name: X-Request-Id
          in: header
          required: true
          schema:
            type: string
        - name: X-Tenant-Id
          in: header
          required: true
          schema:
            type: string
        - name: Idempotency-Key
          in: header
          required: true
          schema:
            type: string
      responses:
        "202":
          description: Cluster teardown accepted; state transitions to Deleting
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/CloudComputeK8sClusterDeleteResponse"
        "400":
          $ref: "#/components/responses/BadRequest"
        "401":
          $ref: "#/components/responses/Unauthorized"
        "403":
          $ref: "#/components/responses/Forbidden"
        "404":
          $ref: "#/components/responses/NotFound"
        "422":
          $ref: "#/components/responses/UnprocessableEntity"

components:
  schemas:
    CloudComputeK8sClusterDeleteResponse:
      type: object
      required: [data, metadata]
      properties:
        data:
          $ref: "#/components/schemas/CloudComputeK8sClusterRecord"
        metadata:
          $ref: "#/components/schemas/CloudComputeK8sMetadata"
    CloudComputeK8sClusterRecord:
      type: object
      description: |
        Cluster projection at the time of the delete request.
        state will be "deleting" or "deleted".
      required:
        - resource_id
        - tenant_id
        - region
        - flavor
        - control_plane_version
        - control_plane_private
        - node_pool_count
        - residency
        - state
        - data_class
        - created_at_epoch_seconds
        - schema_version
      properties:
        resource_id:
          type: string
        tenant_id:
          type: string
        region:
          type: string
        state:
          type: string
          enum: [deleting, deleted]
    CloudComputeK8sMetadata:
      type: object
      required: [request_id]
      properties:
        request_id:
          type: string
```

### Proto3 fragment

```proto
syntax = "proto3";
package oya.cloud.compute.k8s.v1;

service CloudComputeK8sService {
  // Existing
  rpc CreateCluster (CreateClusterRequest) returns (CreateClusterResponse);
  // New
  rpc DeleteCluster (DeleteClusterRequest) returns (DeleteClusterResponse);
}

message DeleteClusterRequest {
  string request_id       = 1;
  string tenant_id        = 2;
  string idempotency_key  = 3;
  string cluster_id       = 4;
  string principal_id     = 5;
  string decision_id      = 6;
  repeated string allowed_surfaces = 7;
  string authorization_tenant_id   = 8;
  string authorization_principal_id = 9;
}

message DeleteClusterResponse {
  ClusterRecord data       = 1;
  ResponseMetadata metadata = 2;
}
```

## State Machine (delete path only)

```
Creating  ─┐
Ready     ─┤──► Deleting ──► Deleted
Reconciling┘
Draining  ─┘
Deleting  ──► (idempotent replay — return same Deleting record)
Deleted   ──► (idempotent replay — return same Deleted record)
```

The boundary layer projects to `Deleting` on acceptance (202). Transition to
`Deleted` is recorded if the domain returns a fully-deleted state. The
reconciler (out of scope for this task) drives the actual teardown.

## Validation Order (delete path)

1. `validate_boundary` — `request_id`, `tenant_id`, `idempotency_key` non-empty
2. `validate_path_cluster_id_only` — `path_cluster_id` non-empty (no body to
   match against for delete; path is the sole cluster identity source)
3. `validate_cluster_resource_id` — canonical `oya:cloud:…:k8s:…` parse + kind
4. `validate_delete_tenant_binding` — `ResourceId.tenant_id()` == boundary tenant
   == principal tenant; principal_id non-empty (→ 401 if empty, 403 if mismatch)
5. `validate_authorization` — decision-id non-empty, auth tenant/principal match,
   `CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE` in `allowed_surfaces`

## Error Codes (delete path)

All existing `CloudComputeK8sApiErrorCode` variants are reused. No new variants
are needed for subtasks k8s-del-1 through k8s-del-3. The `status_kind` mapping
is extended with a `cluster_delete_status` method that mirrors
`cluster_create_status`:

| Error variant | Delete status |
|---------------|---------------|
| `EmptyRequestId` / `EmptyTenantHeader` / `EmptyIdempotencyKey` / `EmptyPathClusterId` / `InvalidClusterId` / `ClusterKindMismatch` | 400 Bad Request |
| `EmptyPrincipalId` | 401 Unauthorized |
| `TenantMismatch` / `EmptyAuthorizationDecisionId` / `AuthorizationTenantMismatch` / `AuthorizationPrincipalMismatch` / `AuthorizationDenied` | 403 Forbidden |
| `ComputeNotFound` | 404 Not Found |
| `IdempotencyKeyReused` | 422 Unprocessable Entity |

## Idempotency Semantics

- Delete ledger is keyed on `(tenant_id, principal_id, "cloud.compute.k8s.cluster.delete", idempotency_key)`.
- A replayed key with the same `path_cluster_id` fingerprint returns the
  identical `CloudComputeK8sClusterDeleteSuccessResponse` without a second
  teardown.
- A replayed key with a different `path_cluster_id` returns
  `CloudComputeK8sApiError::IdempotencyKeyReused` (422).

## Testing Strategy

Integration tests live in `tests/cloud_compute_k8s_api.rs` (existing file,
new test functions appended):

| Test | Coverage |
|------|---------|
| `k8s_delete_api_surface_constants` | Surface constant value + status codes |
| `k8s_delete_api_accepts_valid_teardown` | 202 + `Deleting` state projection |
| `k8s_delete_api_replay_returns_same_response` | Idempotency-key replay, no double-teardown |
| `k8s_delete_api_rejects_empty_request_id` | `EmptyRequestId` → 400 |
| `k8s_delete_api_rejects_empty_tenant` | `EmptyTenantHeader` → 400 |
| `k8s_delete_api_rejects_empty_idempotency_key` | `EmptyIdempotencyKey` → 400 |
| `k8s_delete_api_rejects_empty_principal` | `EmptyPrincipalId` → 401 |
| `k8s_delete_api_rejects_missing_delete_surface` | `AuthorizationDenied` → 403 |
| `k8s_delete_api_rejects_tenant_mismatch` | `TenantMismatch` → 403 |
| `k8s_delete_api_rejects_unknown_cluster` | `ComputeNotFound` → 404 |
| `k8s_delete_api_rejects_reused_key_different_cluster` | `IdempotencyKeyReused` → 422 |
| `k8s_delete_error_response_request_id_roundtrips` | `error_response.error.request_id` == request's `request_id` |

## Boundary Constraints

- `crates/cloud-compute-k8s-api/` is the only directory modified.
- Root `Cargo.toml` is not touched.
- No new workspace member is introduced.
- No existing public function signatures or error variants are altered.
- No provider adapter or reconciliation I/O is introduced.
