# oya-connector-salesforce-adapter

Salesforce CRM connector.

## Coverage

* `Account` — companies
* `Contact` — people
* `Opportunity` — deals

Bulk API 2.0 is used for large list loads; Streaming API (PushTopic / CDC)
drives `subscribe()` events.

## Auth

OAuth 2.0 (JWT bearer flow for server-to-server, web-server flow for
delegated). SecretReference resolves to `sref://<tenant>/salesforce/oauth`.

## Sandbox

Salesforce Developer Edition (free) at developer.salesforce.com/signup
or Trailhead Playground. The seed fixture mimics standard-object shapes.

## Rate limits

Enterprise edition: 100k daily API requests / org (~25 req/sec sustained;
burst 100).

## Capabilities

All six verbs supported (ALL).

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
