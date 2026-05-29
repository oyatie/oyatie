# oya-connector-rippling-adapter

Rippling unified HRIS / IT / Finance connector.

## Coverage

* `employee` — HRIS records (10-employee smoke fixture)
* `device` — IT device fleet
* `transaction` — Finance / expense ledger transactions

## Auth

API key (header bearer). SecretReference resolves to
`sref://<tenant>/rippling/key` in OpenBao.

## Sandbox

Rippling sandboxes are issued via developer.rippling.com after partner
approval. The seed fixture mimics Rippling shapes for hermetic testing.

## Rate limits

Default app rate-limit ~10 req/sec; daily quota 250k.

## Capabilities

`list`/`get`/`create`/`update`/`delete` supported. `subscribe`
unsupported in v1.

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
