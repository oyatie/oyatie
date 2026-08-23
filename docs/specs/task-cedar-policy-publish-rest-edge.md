# Spec: cedar-policy-publish-rest-edge

## Objective

Add a REST control-plane edge for the `cedar.policy.publish` surface inside the existing
`policy-cedar-api` crate. A new `src/rest/` module exposes an axum router that:

1. Accepts `POST /policies/{policy_id}/versions/{version}` with JSON request body.
2. Extracts boundary headers (`X-Request-Id`, `X-Tenant-Id`, `Idempotency-Key`), principal fields
   (`X-Principal-Tenant-Id`, `X-Principal-Id`), and authorization fields (`X-Authorization-Decision-Id`,
   `X-Authorization-Tenant-Id`, `X-Authorization-Principal-Id`, `X-Authorization-Surfaces`) from HTTP headers.
3. Delegates validation + publication to the existing typed boundary fns in `src/lib.rs`
   (`validate_cedar_policy_publish_request`, `publish_cedar_policy_from_api`).
4. Maps `CedarPolicyPublishApiError` → HTTP status codes via the existing `cedar_policy_publish_status_code()`.
5. Honours the idempotency ledger (mutex-guarded `CedarPolicyPublishIdempotencyLedger`).
6. Emits a `tracing` OTel span (`cedar.policy.publish`) on every handler call with structured attributes.

## Crate Boundary

- Crate: `policy-cedar-api` (path: `crates/policy-cedar-api/`).
- No new workspace member; no edits to root `Cargo.toml`.
- ADR-0509 flat-clean-architecture: clean-arch layers are mods in `src/`.

## Module Layout (flat clean arch)

```
crates/policy-cedar-api/
  src/
    lib.rs          <- existing; add `pub mod rest;`
    rest/
      mod.rs        <- NEW: axum router, handlers, state, JSON DTOs, OTel spans
  tests/
    cedar_policy_publish_api.rs  <- existing boundary tests (unchanged)
    rest_router.rs               <- NEW: router-level tests
```

## Contracts

- **OpenAPI**: `contracts/openapi/platform/platform-policy-cedar-v1.yaml`
  (constant `CEDAR_POLICY_PUBLISH_OPENAPI_CONTRACT`).
- Surface: `cedar.policy.publish` (constant `CEDAR_POLICY_PUBLISH_SURFACE`).
- HTTP verb+path: `POST /policies/{policy_id}/versions/{version}`.
- Content-Type: `application/json`.

## Request / Response Shape

### Headers (all required unless noted)

| Header                         | Maps to                                    |
|--------------------------------|--------------------------------------------|
| `X-Request-Id`                 | `boundary.request_id`                      |
| `X-Tenant-Id`                  | `boundary.tenant_id`                       |
| `Idempotency-Key`              | `boundary.idempotency_key`                 |
| `X-Principal-Tenant-Id`        | `principal.tenant_id`                      |
| `X-Principal-Id`               | `principal.principal_id`                   |
| `X-Authorization-Decision-Id`  | `authorization.decision_id`                |
| `X-Authorization-Tenant-Id`    | `authorization.tenant_id`                  |
| `X-Authorization-Principal-Id` | `authorization.principal_id`               |
| `X-Authorization-Surfaces`     | `authorization.allowed_surfaces` (CSV)     |

### Request JSON Body (`CedarPolicyPublishRequestDto`)

```json
{
  "policy_id": "pol_tenant_admin",
  "version": "1.0.0",
  "scope": { "kind": "tenant", "tenant_id": "ten_alpha" },
  "supersedes": null,
  "rules": [
    {
      "effect": "allow",
      "principal_role": "tenant-admin",
      "action": "tenant.settings.update",
      "resource_prefix": "tenant:",
      "required_attribute": { "key": "region", "value": "region-home" }
    }
  ]
}
```

### Success Response `201 Created`

```json
{
  "data": {
    "policy_id": "pol_tenant_admin",
    "version": "1.0.0",
    "scope": { "kind": "tenant", "tenant_id": "ten_alpha" },
    "supersedes": null,
    "rules": [...],
    "schema_version": 1
  },
  "metadata": {
    "request_id": "req_cedar_policy_001",
    "operator_tenant_id": "ten_platform",
    "principal_id": "usr_platform_admin"
  }
}
```

### Error Response (4xx)

```json
{
  "error": {
    "code": "CEDAR_POLICY_KERNEL_VERSION_ALREADY_EXISTS",
    "message": "Policy version already exists",
    "message_localized": null,
    "request_id": "req_cedar_policy_001",
    "details": [{ "field": "policy_kernel", "issue": "policy_id/version pair must be immutable" }],
    "retry_after_seconds": null
  }
}
```

## HTTP Status Mapping

| `CedarPolicyPublishApiStatus` | HTTP Code |
|-------------------------------|-----------|
| `Created`                     | 201       |
| `BadRequest`                  | 400       |
| `Unauthorized`                | 401       |
| `Forbidden`                   | 403       |
| `Conflict`                    | 409       |
| `UnprocessableEntity`         | 422       |

## OTel / Observability

- Span name: `cedar.policy.publish`
- Span attributes:
  - `cedar.policy.publish.status_code` (u16): HTTP status code of the response.
  - `cedar.policy.publish.policy_id` (str): path-extracted policy_id.
  - `cedar.policy.publish.version` (str): path-extracted version.
  - `cedar.policy.publish.idempotent_replay` (bool): true when the response is a ledger replay.

## Testing Strategy

- **Router-level tests** in `tests/rest_router.rs` using `tower::ServiceExt::oneshot`.
- Tests assert HTTP status + JSON body shape for:
  1. Happy path `201 Created`.
  2. Idempotent replay `201 Created` (same key + body, no re-publish).
  3. Path/body mismatch `400 Bad Request`.
  4. Missing principal → `401 Unauthorized`.
  5. Authorization denied → `403 Forbidden`.
  6. Duplicate version → `409 Conflict`.
  7. Reused idempotency key → `422 Unprocessable Entity`.
- No business logic in the router; the existing `cedar_policy_publish_api.rs` integration tests
  cover the boundary fns end-to-end.

## SLO / ADR-0130

No new OpenSLO file required: `policy-cedar-api` is a library crate (no independently
deployable µservice). SLO authoring requirement applies to µservice promotions.

## Disjointness Gate

`git diff --stat origin/dev` must touch ONLY:
- `crates/policy-cedar-api/**`
- `docs/specs/task-cedar-policy-publish-rest-edge.md`
- `tasks/cedar-policy-publish-rest-edge-plan.md`
