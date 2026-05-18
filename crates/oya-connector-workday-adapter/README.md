# oya-connector-workday-adapter

Workday HCM connector — Workday REST API v40+.

## Coverage

* `worker` — workers (employees), employments
* `job-profile` — job catalog
* `compensation` — compensation plans

Bulk extracts via Workday Studio are tracked in a follow-up adapter IP.

## Auth

OAuth 2.0 client-credentials with ISU (Integration System User) JWT.
SecretReference resolves to `sref://<tenant>/workday/isu-jwt` in OpenBao.

## Sandbox

Workday Customer Care provides "Implementation Tenant" sandboxes per
contract. The smoke fixture seeds 10 workers + 2 job-profiles per
tenant so `cargo test` can run without contract-tenant access.

## Smoke test

`cargo test -p oya-connector-workday-adapter -- smoke_lists_ten_workers`

## Rate limits

REST per-ISU throttle ~10 req/sec; daily quota 500k. The kernel
publishes `RateLimitDescriptor { requests_per_second: 10, burst: 30 }`.

## Capabilities

* `list`, `get`, `create`, `update` — supported.
* `delete` — UNSUPPORTED. Workday HRIS uses termination dates, not hard
  delete; this connector returns `ConnectorError::Unsupported`.
* `subscribe` — UNSUPPORTED. Workday REST has no CDC; use bulk extracts.

## Retry semantics

* 429 / 503 → 3× retry with exponential backoff
* 401 → re-authenticate via ISU JWT then retry once

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
