---
purpose: Oyatie — API Design Standard
doc_status: published
---

# Oyatie — API Design Standard

> **Status:** Draft v0.1 — 2026-05-09. Standard for every public-API authored under the Oyatie new ADR pack.
> **Owner:** `platform-api-sdk` + `council-architecture`.
> **Companion:** [SPEC.md](../SPEC.md), [DESIGN.md §10](../DESIGN.md), `contracts/`.

## 1. Wire formats

| Surface | Wire format |
|---|---|
| External public REST | OpenAPI 3.2 |
| Internal service-to-service | gRPC + Protobuf |
| Async events | CloudEvents 1.0 envelope + Protobuf payload + AsyncAPI schema |
| Real-time client | WebSocket + JSON or Protobuf-over-WS |
| Agent-discoverable | MCP (Model Context Protocol) |

All schemas live in `contracts/` per ADR-0011 cross-axis contract registry.

## 2. URL conventions (REST)

- Base: `https://api.oyatie.com/v<version>/`
- Per-tenant: `https://<tenant-slug>.api.oyatie.com/v<version>/` (when tenant-scoped + path-based)
- Resource-oriented: `/<noun-plural>/{id}` (e.g. `/tenants/{tenant_id}`, `/workflows/{workflow_id}/runs/{run_id}`)
- Sub-resource: `/<noun-plural>/{id}/<sub>` (e.g. `/tenants/{id}/users`)
- Verb-namespaced for actions: `/{noun}/{id}:<action>` (e.g. `/workflows/{id}:publish`)
- Per-region: subdomain or path prefix (e.g. `kr.api.oyatie.com` or `/kr/`)

## 3. HTTP method semantics

| Method | Use |
|---|---|
| GET | Idempotent read; no side effects |
| POST | Create or non-idempotent action |
| PUT | Full replace (idempotent) |
| PATCH | Partial update (typically idempotent with If-Match) |
| DELETE | Idempotent delete (with optional cascade behavior in body) |
| QUERY | OpenAPI 3.2 fixed operation for safe complex queries that do not fit GET query-string limits; requires council/API-review approval before public use |

OpenAPI 3.2 `additionalOperations` MAY describe non-fixed HTTP methods only. It MUST NOT duplicate fixed operation methods (`GET`, `PUT`, `POST`, `DELETE`, `OPTIONS`, `HEAD`, `PATCH`, `TRACE`, `QUERY`). Any public non-fixed method requires an explicit governance rationale in the contract metadata before adoption. Runtime-bound `query` and `additionalOperations` operations follow the same explicit numeric response/status/schema parity rules as conventional REST methods.

## 4. Headers

Required:
- `Authorization: Bearer <token>` — STS short-lived token
- `Idempotency-Key: <uuid>` — for non-idempotent actions
- `X-Request-Id: <uuid>` — request tracing
- `X-Tenant-Id: <id>` — when multi-tenant scope ambiguous (signed)
- `Content-Type: application/json` (REST) | `application/protobuf` (binary)
- `Accept-Language: <locale>` — per-pack
- `If-Match` / `If-None-Match` — optimistic concurrency

Forbidden:
- Long-lived API keys in headers (per ADR-0013 license + ADR-0043 secrets)
- PII / PHI / PCI in URL path / query params (always in body, encrypted)

## 5. Response shape

```jsonc
// Success
{
  "data": { ... },
  "metadata": {
    "request_id": "uuid",
    "page": { "cursor_next": "...", "cursor_prev": "..." } // when paginated
  }
}

// Error
{
  "error": {
    "code": "ERROR_CODE_SCREAMING_SNAKE",
    "message": "Human-readable English",
    "message_localized": "...",
    "request_id": "uuid",
    "details": [ { "field": "...", "issue": "..." } ],
    "retry_after_seconds": 30   // when retryable
  }
}
```

Status codes: standard HTTP semantics; 422 for validation; 429 for rate-limit; 503 for retry-after. Runtime-bound OpenAPI operations list explicit numeric response keys so `oya doc openapi` can prove the API status enum matches the contract; `default` and `1XX`-through-`5XX` ranges require an explicit governance rationale before they can be used on runtime-bound surfaces. Runtime-bound status enums use fieldless variants and explicit `Self::Variant => <status>` code arms; wildcard/default arms are not accepted for contract parity. Runtime-bound responses also declare concrete `application/json` schema refs and exact status-to-schema mappings; public success responses use the typed `{ "data": ..., "metadata": ... }` envelope and public error responses use the typed `{ "error": ... }` envelope, with both schema-bound to runtime structs.

## 6. Pagination

- Cursor-based always (per [TOOLCHAIN.md](../TOOLCHAIN.md))
- Page size: query param `?limit=N` (default 50; max 1000)
- Cursor: opaque token; clients pass back as `?cursor=<token>`
- Per-PR test for cursor stability across data mutation

## 7. Idempotency

Per [DESIGN.md §9](../DESIGN.md):
- Every non-idempotent action requires `Idempotency-Key`
- Server stores key + response for replay window (24h default)
- Replays return same response (not 409)
- Different body with same key → 422 conflict

## 8. Versioning

- URL-based major versions: `/v1/`, `/v2/`
- Header-based minor versions: `Api-Version: 2.5` (optional)
- Stability tiers per ADR-0037: preview / stable / GA
- Deprecation: 12 months for GA per-endpoint telemetry; 6 months for stable; preview no commitment
- Per-deprecation event emission to `oyatie.platform.api.deprecated`

## 9. Rate limiting

- Per-tenant per-endpoint default tier
- Per-tenant per-capability cost ceiling per [TOOLCHAIN.md §4.4](../TOOLCHAIN.md)
- 429 with `Retry-After` header
- Per-API-key burst + sustain limits

## 10. Authentication + Authorization

- AuthN: OAuth2 + STS (short-lived); per-tenant per-purpose
- AuthZ: Cedar policy per ADR-0007 + per-capability autonomy-ceiling per ADR-0022
- MFA enforcement on admin operations
- Audit-emit per call (per ADR-0003)

## 11. Per-data-class handling

Per [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md) + ADR-0008:
- Every request/response field tagged with `data_class`
- HARD_DENY classes (PHI / PCI / PIPA-Art23 / CHILDREN_UNDER_14) cannot be returned in non-encrypted body
- Per-tenant per-class allowlist enforced at API gate
- DSR-cascade integration

## 12. Error codes (closed enumeration)

```
GENERIC_*     — generic error families
TENANT_*      — tenant-scoped errors
CAPABILITY_*  — Foundry capability errors
PRIVACY_*     — Data Use Boundary violations
COMPLIANCE_*  — regulatory denials
RATE_LIMIT    — rate limit
QUOTA         — quota exhaustion
```

## 13. Pagination + bulk endpoints

- Bulk read: `/<noun-plural>?ids=<id1>,<id2>,...` (max 100)
- Bulk write: `POST /<noun-plural>:batch` with array body (max 100)
- Bulk dispatch (per [DESIGN §9](../DESIGN.md)): per-axis bulk endpoint contract

## 14. Deprecation telemetry

Per ADR-0037 + ADR-0019:
- Every deprecated endpoint emits `EVT-API-ENDPOINT-DEPRECATED-INVOKED`
- Per-tenant deprecated-endpoint dashboard
- Migration guide auto-generated from deprecation diff

## 15. Sources
[SPEC.md](../SPEC.md), [DESIGN.md §10](../DESIGN.md), ADR-0007, ADR-0008, ADR-0011, ADR-0019, ADR-0022, ADR-0037, ADR-0043, [TOOLCHAIN.md](../TOOLCHAIN.md).
