# oya-connector-gusto-adapter

Gusto Embedded Payroll connector — SMB-focused.

## Coverage

* `employee` — employees + compensations
* `payroll` — pay runs
* `contractor` — contractor pay

## Auth

OAuth 2.0 with company-scoped access tokens.

## Sandbox

Gusto Embedded Demo: dev.gusto-demo.com — sandbox companies seeded
with synthetic SMB shapes. SecretReference resolves to
`sref://<tenant>/gusto/oauth`.

## Rate limits

Embedded standard tier ~4 req/sec; daily quota 100k.

## Capabilities

`list`/`get`/`create`/`update`. `delete` UNSUPPORTED (payroll records
are immutable). `subscribe` UNSUPPORTED (webhooks tracked separately).

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
