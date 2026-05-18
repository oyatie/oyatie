# oya-connector-adp-adapter

ADP Workforce Now connector.

## Coverage

* `worker` — workers + assignments
* `pay-statement` — payroll statements (read)
* `time-card` — time-card entries
* `benefit-coverage` — benefit elections + dependents

## Auth

ADP requires mutual TLS (client cert + key) plus an OAuth2 access token.
SecretReference resolves to `sref://<tenant>/adp/mtls` containing the
PKCS#12 bundle + token endpoint metadata.

## Sandbox

ADP provides developer sandboxes via developer.adp.com after Marketplace
ISV approval. The seed fixture mimics Workforce Now shapes for hermetic
testing.

## Rate limits

Marketplace standard tier ~5 req/sec; daily quota 100k.

## Capabilities

* `list`, `get`, `create`, `update` — supported.
* `delete` — UNSUPPORTED (ADP uses termination events).
* `subscribe` — UNSUPPORTED for v1 (event-notification webhooks tracked
  as follow-up).

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
