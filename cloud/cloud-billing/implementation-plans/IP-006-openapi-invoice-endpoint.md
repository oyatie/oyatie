---
ip_id: IP-006
microservice: cloud-billing
title: OpenAPI 3.2.0 invoice REST contract — generate / read / void / credit-memo
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0145, ADR-0253, ADR-0243, ADR-0244, ADR-0263, ADR-0131]
counterpart_parity: [Stripe Billing API, Recurly v3 API, Chargebee API v2, Zuora REST API]
capabilities_touched:
  - cap.cloud.billing.issue_invoice
  - cap.cloud.billing.read_invoice
  - cap.cloud.billing.void_invoice
  - cap.cloud.billing.issue_credit_memo
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-006 — REST OpenAPI 3.2.0 invoice endpoint

## §A Objective

Document the existing OpenAPI 3.2.0 invoice contract at `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` (544 lines) and its runtime adapter in `oya-cloud-billing-tax-app` (276 lines). The contract defines:

- `POST /v1/cloud/billing/invoices/{invoice_id}` — generate a tenant-scoped tax invoice.
- Headers: `X-Request-Id` (required), `X-Tenant-Id` (required), `Idempotency-Key` (required), bearer `Authorization`.
- Request body: `CloudBillingInvoiceGenerateRequest` with full invoice shape (account snapshot, line items, totals, tax fields).
- Response codes: 201 (created), 400 (bad request), 401 (unauthorized), 403 (forbidden), 409 (conflict), 422 (unprocessable entity).
- Response body: `CloudBillingInvoiceGenerateSuccessResponse` containing `CloudBillingInvoiceRecord` + metadata.

REST is the primary integration surface for tenant-facing ERPs (SAP, NetSuite, Oracle EBS) and for the finops-portal product. gRPC (IP-008) is the canonical inter-µservice surface; REST is the outside-facing translation.

## §B Scope

In scope:

- The single existing endpoint `POST /v1/cloud/billing/invoices/{invoice_id}`.
- Companion endpoints documented in proto3 but not yet in OpenAPI: `GET /v1/cloud/billing/invoices/{invoice_id}`, `POST /v1/cloud/billing/invoices/{invoice_id}/void`, `POST /v1/cloud/billing/credit-memos` (planned for IP-006-extension; tracked as REMEDIATION-NOTES item).
- Header semantics: idempotency, tenant scoping, request-id provenance.
- Status code mapping rules (which kernel error maps to which HTTP status).
- Error response body shape (`CloudBillingTaxApiErrorBody`).

Out of scope:

- AsyncAPI event surface (IP-007).
- gRPC service surface (IP-008).
- The internal `oya-cloud-billing-domain` aggregate (IP-001).

## §C Architecture

### §C.1 Endpoint shape

```
POST /v1/cloud/billing/invoices/{invoice_id}
Authorization: Bearer <STS-token>
X-Request-Id: <ulid>
X-Tenant-Id: <ten_*>
Idempotency-Key: <opaque>
Content-Type: application/json

{
  "id": "inv_alpha_202605_001",
  "account": {...},
  "tenant_id": "ten_alpha",
  "regional_pack": "oya-pack-electronic-tax",
  "period": {"start_epoch_seconds": ..., "end_epoch_seconds": ...},
  "line_items": [...],
  "subtotal": {"currency": "OYC", "minor_units": 100000},
  "tax": {"currency": "OYC", "minor_units": 10000},
  "total": {"currency": "OYC", "minor_units": 110000},
  "tax_invoice_format": "ElectronicTaxInvoice",
  "tax_registration_id": "taxid/electronic/1234567890",
  "issued_at_epoch_seconds": 1700086500,
  "due_at_epoch_seconds": 1700604900,
  "data_class": "FINANCIAL"
}
```

Path-id and body-id MUST match (per `tax-app` line 198: `request.id.is_empty()` is checked, but cross-equality is enforced by tenant_mismatch check + idempotency key collision detection).

### §C.2 HTTP/3 + QUIC default (ADR-0253)

The endpoint serves HTTP/3 + QUIC primarily; HTTPS/2 is the fallback. The OpenAPI server stanza names the canonical address `https://api.oyatie.com`; deployment-context overlays (on-prem, colo, guest-on-aws/oci) substitute the appropriate hostname.

### §C.3 Status code rules

Per `oya-cloud-billing-tax-app::generate_cloud_billing_invoice_from_api`:

| Precondition | Status | code | message |
|---|---|---|---|
| `request_id.is_empty()` | 401 | `missing_request_id` | "authenticated request id evidence is required" |
| Tenant scope mismatch (header vs request vs account) | 403 | `tenant_mismatch` | "tenant header must match invoice and account tenant" |
| `idempotency_key.is_empty()` | 422 | `missing_idempotency_key` | "idempotency key is required" |
| Shape contract violation (id empty, account.id empty, line_items empty, currency mismatch, subtotal+tax≠total) | 400 | `invalid_invoice_request` | "invoice request violates billing value contract" |
| `account.state != "active"` | 409 | `billing_account_not_active` | "invoice generation requires an active billing account" |
| Else | 201 | (no error body) | (success body) |

This mapping is empirically tested in `crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs`. The mapping is intentionally a closed switch — there is no catch-all 500 path; any unexpected error from the domain kernel is wrapped as 422 with the kernel error message.

### §C.4 Idempotency semantics

The `Idempotency-Key` header is required on every POST. Semantics:

- First request with `(tenant_id, idempotency_key)` produces a new invoice or returns the domain-kernel error (above).
- Subsequent request with same `(tenant_id, idempotency_key)` and **identical request body fingerprint** returns the original 201 response (replay).
- Subsequent request with same key but **different body fingerprint** returns 422 `idempotency_key_collision` (per OpenAPI line 84: "Idempotency key was already used with a different request fingerprint").

The body fingerprint is SHA-256 of the canonicalized JSON (per RFC 8785 JSON Canonicalization Scheme).

### §C.5 Data-class tagging

The OpenAPI spec uses `x-oyatie-data-class` to tag each field. Three values appear:

- `INTERNAL_ONLY`: most identifier and money fields.
- `PUBLIC`: schema_version, region, data_class indicator.
- `FINANCIAL_REGULATED_CREDIT`: `tax_registration_id` only (per `oya-cloud-billing-tax-app::CloudBillingInvoiceGenerateRequest.tax_registration_id` field comment).

These tags are read by `data-boundary-kernel` at the API gateway layer to enforce field-level redaction policies (per ADR-0244 tenant scoping + data-boundary kernel).

### §C.6 Bearer-token / STS authentication

Per `securitySchemes.bearerAuth` (line 90–94): bearer scheme with `bearerFormat: STS`. The STS token is issued by cloud-iam and carries:

- `principal.tenant_id`
- `principal.tenant_class`
- `principal.billing_components`
- `principal.roles`
- `principal.cap_breached`
- `principal.byok_modes`

The Cedar evaluator at `api-gateway` reads these claims and enforces the `cap.cloud.billing.issue_invoice` gate before the request reaches `oya-cloud-billing-tax-app`.

## §D Lifecycle

### §D.1 Successful invoice issuance

1. Caller obtains STS token from cloud-iam.
2. Caller builds idempotency key (typically `ten_xxx_period_yyyymm`).
3. POST to endpoint.
4. api-gateway evaluates Cedar (issue_invoice gate).
5. `oya-cloud-billing-tax-app::generate_cloud_billing_invoice_from_api` validates preconditions.
6. cloud-billing internally invokes `oya-cloud-billing-domain::CloudBillingLedger::generate_invoice`.
7. audit-chain seal emitted (IP-010).
8. 201 returned with `CloudBillingInvoiceRecord` + request_id.

### §D.2 Idempotent replay

1. Caller retries POST with same Idempotency-Key + same body.
2. api-gateway evaluates Cedar (issue_invoice gate succeeds).
3. cloud-billing detects existing invoice by `(tenant_id, idempotency_key)` lookup.
4. Returns 201 with original `CloudBillingInvoiceRecord`.

### §D.3 Failure modes (HTTP-level)

- 401 missing_request_id → caller didn't include X-Request-Id; retry with header.
- 403 tenant_mismatch → STS token tenant disagrees with body tenant or account tenant; rotate token.
- 422 missing_idempotency_key → add Idempotency-Key header.
- 400 invalid_invoice_request → fix body shape (currency match, total math, non-empty line items).
- 409 billing_account_not_active → un-suspend account or fix tenant_class.

## §E Cedar Policy Bindings

- `cap.cloud.billing.issue_invoice` (cloud-billing.cedar lines 93–101) — primary gate.
- `cap.cloud.billing.read_invoice` (cloud-billing.cedar lines 173–183) — for GET invoice endpoint (planned).
- `cap.cloud.billing.void_invoice` (cloud-billing.cedar lines 103–111) — for void endpoint (planned).
- `cap.cloud.billing.issue_credit_memo` (cloud-billing.cedar lines 113–121) — for credit-memo endpoint (planned).

Context attributes used:

- `context.has_reviewer_approval` — required for void / credit-memo (two-person rule).
- `principal.has_role("oyatie-finance-operator")` — required for void / credit-memo.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` (544 lines).
- `/Users/jasonlee/oyatie/contracts/openapi/cloud/cloud-billing-invoice-v1.meta.yaml` (governance metadata).
- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-tax-app/src/lib.rs` (276 lines, runtime adapter).
- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs` (status-code round-trip tests).

### §F.2 Schema reference

The OpenAPI schema components are:

- `CloudBillingInvoiceGenerateRequest` — full request body.
- `CloudBillingAccountSnapshotRequest` — account state snapshot.
- `CloudBillingMoneyRequest` — currency + minor_units.
- `CloudBillingPeriodRequest` — period bounds.
- `CloudBillingInvoiceLineItemCreateRequest` — line item.
- `CloudBillingTaxMeterUnitRequest` — meter unit kind + quantity.
- `CloudBillingInvoiceGenerateSuccessResponse` — wrapper.
- `CloudBillingTaxApiMetadata` — request id.
- `CloudBillingInvoiceRecord` — returned record.
- `CloudBillingTaxApiErrorResponse`, `CloudBillingTaxApiErrorBody`, `CloudBillingTaxApiErrorDetail`.

### §F.3 ADR anchors

- ADR-0145: direct gRPC + 3 invariants (REST is the outside-facing translation).
- ADR-0253: HTTP/3 + QUIC default protocol.
- ADR-0243: Cedar gates every state mutation.
- ADR-0244: tenant scoping via X-Tenant-Id header.
- ADR-0263: every state mutation produces audit-chain seal.

## §G Counterpart parity

| Counterpart | Their endpoint shape | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Billing | `POST /v1/invoices` returns 200; `Idempotency-Key` header standard | `POST /v1/cloud/billing/invoices/{invoice_id}` returns 201 | Oyatie names the invoice in the path (caller owns id allocation); Stripe assigns id and returns it. Matter of philosophy. |
| Stripe Billing | Errors: 400/401/402/403/404/409/429/500 | Errors: 400/401/403/409/422 | Oyatie has tighter error taxonomy; no 429 (rate-limiting is at api-gateway, surfaces as 429 only there). |
| Stripe Billing | `idempotency_key` standard header | `Idempotency-Key` header (per IETF draft idempotency-header) | Same semantics. |
| Recurly v3 | `POST /sites/{site_id}/invoices` | `POST /v1/cloud/billing/invoices/{invoice_id}` | Recurly site = oyatie tenant; oyatie scopes via X-Tenant-Id header rather than path. |
| Recurly v3 | Returns full invoice; supports polling for collection | Returns CloudBillingInvoiceRecord; polling via separate dunning endpoint | Same shape. |
| Chargebee API v2 | `POST /api/v2/invoices` with `customer_id` body field | `POST .../{invoice_id}` with `tenant_id` + `account.id` body fields | Chargebee has customer + subscription; oyatie has tenant + billing_account. |
| Zuora REST | `POST /v1/object/invoice` | Same shape | Direct parity. |
| Zuora REST | Two-phase: stage → post | Single-phase: generate (issued state immediately) | Oyatie does not stage; void path covers retraction. |

## §H Open questions

- Whether to add `GET /v1/cloud/billing/invoices?tenant_id=...` paginated list endpoint. Current decision: list endpoints exist in gRPC (`MeteringApi::ReadMeterAggregate`); REST list endpoints will be added in IP-006-extension if finops-portal needs them.
- Whether to add `POST /v1/cloud/billing/invoices/{invoice_id}/pdf` for PDF rendering. Current decision: PDF generation lives in a separate `cloud-billing-pdf-renderer` µservice; the REST endpoint returns `pdf_object_ref` (storage handle) per proto3 `Invoice.pdf_object_ref` field.
