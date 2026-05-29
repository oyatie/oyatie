---
ip_id: IP-005
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/tenant-billing-presentation/api
related_adrs: [ADR-0131, ADR-0174, ADR-0183, ADR-0199]
depends_on: [IP-001, IP-002, IP-004]
target_lines: 150
---

# IP-005 — `tenant-billing-presentation` API slice

## Why this slice

The API tier exposes the usecase functions as HTTP endpoints behind
the `oya-finops-portal-tenant-billing-presentation-api` crate. It
owns:

- OpenAPI schema authoring (the artifact lives under
  `contracts/tenant-invoice-public.openapi.yaml`).
- Axum router + handler + DTO conversion.
- Authn (JWT verification via shared `oya-authn` middleware) and
  authz (Cedar policy evaluation via `oya-cedar-runtime`).
- Locale + currency presentation conversion.
- Streaming PDF download endpoint.

This is the layer that tenants and the workflow studio call.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-tenant-billing-presentation-api/`
   depends on the usecase from IP-004 plus `axum`, `tower-http`,
   `oya-authn-middleware`, `oya-cedar-runtime`.
2. Endpoints:
   - `GET /v1/tenants/{tenant_id}/invoices` — list periods (page).
   - `GET /v1/tenants/{tenant_id}/invoices/{period}` — fetch invoice.
   - `GET /v1/tenants/{tenant_id}/invoices/{period}.pdf` — stream PDF.
   - `GET /v1/tenants/{tenant_id}/invoices/{period}.html` — render HTML.
   - `POST /v1/tenants/{tenant_id}/invoices/{period}/finalize` —
     finalize (ops-finops only).
3. OpenAPI 3.1 spec at
   `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`
   matches handler signatures (contract-test enforced via
   `oya-check-openapi-server-parity`).
4. Cedar policy `policy/cedar/tenant-isolation.cedar` is evaluated
   before every handler; unit-tested against synthetic principals.
5. Latency budget: handler overhead < 50 ms p99 (excluding
   downstream calls); enforced via micro-benchmark set.
6. ≥ 6 integration tests using `axum::Router` directly.
7. `cargo test -p oya-finops-portal-tenant-billing-presentation-api`
   green.

## File-level work plan

1. `Cargo.toml`.
2. `src/lib.rs` — `router(state: AppState) -> Router`.
3. `src/handlers.rs` — endpoint handlers.
4. `src/dto.rs` — request / response DTOs with serde derives.
5. `src/error.rs` — Axum `IntoResponse` impl for usecase errors.
6. `src/auth.rs` — Cedar evaluation middleware.
7. `tests/integration.rs` — full router tests.

## API surface (Public contract)

```yaml
# Excerpt; full file at contracts/tenant-invoice-public.openapi.yaml.
GET /v1/tenants/{tenant_id}/invoices:
  parameters:
    - name: tenant_id
      in: path
      required: true
      schema: { type: string, format: uuid }
    - name: limit
      in: query
      schema: { type: integer, minimum: 1, maximum: 50, default: 12 }
  responses:
    "200": { schema: InvoicePeriodList }
    "401": { schema: AuthError }
    "403": { schema: TenantBoundaryViolation }
GET /v1/tenants/{tenant_id}/invoices/{period}.pdf:
  responses:
    "200":
      content:
        application/pdf: { schema: { type: string, format: binary } }
      headers:
        Content-Disposition: { schema: { type: string } }
```

## Authz (Cedar)

- Every request gets a `Principal` (the JWT subject), an `Action`
  (e.g. `Action::"ReadInvoice"`), and a `Resource`
  (`Tenant::"<tenant_id>"`).
- The policy at `policy/cedar/tenant-isolation.cedar` permits only
  principals whose `tenant_id` claim matches the requested
  `tenant_id`, OR principals with the `ops-finops` group claim, OR
  the `regulator` principal for explicit-emit endpoints.
- A denied request returns `403` with a normalized
  `TenantBoundaryViolation` body (no tenant-id leakage).

## Locale + currency presentation

- The DTO emits cents as integer `total_amount_cents`.
- A `?locale=ko-KR` query parameter triggers KRW conversion using
  the daily ECB rate cached in `oya-fx-rate`; failing fetch falls
  back to USD with an explicit `presentation_currency=usd` field.
- Locale-formatted strings (currency, date) are computed in the API
  layer, not in domain.

## Error mapping

| UseCaseError                | HTTP | Body kind                  |
|-----------------------------|------|----------------------------|
| `TenantNotFound`            | 404  | `TenantNotFound`           |
| `InvoiceNotFound`           | 404  | `InvoiceNotFound`          |
| `BoundaryViolation`         | 403  | `TenantBoundaryViolation`  |
| `SourceUnavailable`         | 503  | `Upstream{name:opencost}`  |
| `AuditSealFailed`           | 500  | `Internal` (rate-limited)  |

## Risk + mitigation

- **Risk**: PDF generation blocks the request loop. **Mitigation**:
  PDF endpoint streams via `Body::from_stream` and the renderer
  emits in 64KB chunks.
- **Risk**: Cedar policy evaluation adds latency. **Mitigation**:
  the policy bundle is pre-compiled at process start; evaluation
  is O(policies × principal-attrs) and benchmarked at < 1 ms.

## Out-of-scope

- The app tier (axum binary wire-up) — IP-006.

## References

- ADR-0183 — Cedar policy discipline.
- ADR-0199 — cost-attribution canonical.

## Verification

- `cargo test -p oya-finops-portal-tenant-billing-presentation-api`.
- `oya check openapi-server-parity --crate
  oya-finops-portal-tenant-billing-presentation-api`.
