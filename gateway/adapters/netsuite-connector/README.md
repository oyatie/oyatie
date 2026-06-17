# gateway-netsuite-connector

NetSuite ERP connector — SuiteTalk REST (SOAP fallback uses same shape).

## Coverage

* `customer` — A/R parties
* `vendor` — A/P parties
* `salesOrder` — sales orders (multi-subsidiary)
* `journalEntry` — accounting journals

## Auth

Token-Based Authentication (TBA): OAuth1-style signed requests with
account id + consumer key/secret + token id/secret. SecretReference
resolves to `sref://<tenant>/netsuite/tba`.

## Sandbox

NetSuite issues sandbox accounts to subscribers (1 per production
account). Use Account ID with `_SB1` suffix.

## Rate limits

SuiteTalk concurrency governance: 10 concurrent requests per Enterprise
account. Effective ~4 req/sec sustained.

## Capabilities

`list`/`get`/`create`/`update`/`delete` supported. `subscribe`
UNSUPPORTED (would require SuiteScript SDF deploy).

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
