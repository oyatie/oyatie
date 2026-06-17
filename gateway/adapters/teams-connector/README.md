# gateway-teams-connector

Microsoft Teams enterprise connector (Microsoft Graph API).

## Coverage

* `channel` — list/get/create/update/delete Teams channels.
* `message` — post / update / delete channel messages.
* `adaptive-card` — render interactive cards in message attachments.

## Auth

OAuth 2.0 client-credentials. SecretReference resolves to a
service-principal client_secret stored at
`sref://<tenant>/teams/client-secret` in OpenBao.

## Sandbox

Microsoft 365 developer sandbox: create a tenant at
developer.microsoft.com/en-us/microsoft-365/dev-program, register an app
in Entra ID, grant `ChannelMessage.Send` + `Channel.ReadBasic.All`.

## Rate limits

Graph API: 10k requests per 10 minutes per app per tenant
(~16 req/sec; burst 100). Daily quota 1M.

## Retry semantics

`Retry-After` headers honored on 429; the live impl uses an
exponential-backoff jitter with 3× retry budget.

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.
