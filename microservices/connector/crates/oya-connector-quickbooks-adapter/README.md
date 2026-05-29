# oya-connector-quickbooks-adapter

QuickBooks Online connector — SMB accounting.

## Coverage

* `Customer` — A/R parties
* `Vendor` — A/P parties
* `Invoice` — outgoing invoices
* `Bill` — incoming bills

## Auth

OAuth 2.0 — QuickBooks → access token (1h) + refresh token (100d).
SecretReference resolves to `sref://<tenant>/qbo/oauth`.

## Sandbox

Intuit Developer dashboard provides per-app sandbox companies at no cost.
Create at developer.intuit.com.

## Rate limits

500 requests per minute per realm (~8 req/sec; daily quota 500k).

## Capabilities

`list`/`get`/`create`/`update`/`delete` supported. `subscribe`
UNSUPPORTED — webhooks are app-scoped not tenant-scoped; per-tenant
fan-out is a separate adapter.

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
