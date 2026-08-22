# gateway-slack-connector

Slack enterprise connector — implements `shared_connector_kernel::Connector`.

## Coverage

* `conversation` — list channels, get channel, archive (delete)
* `message` — `chat.postMessage`, `chat.update`, `chat.delete`
* `file` — `files.upload`, `files.list`
* Events API — `conversation`/`message`/`file` events as
  `EventStream`.

## Auth

Bot-user OAuth 2.0. SecretReference resolves to
`sref://<tenant>/slack/bot-token` in OpenBao.

## Sandbox

Slack sandbox: create a Standard plan workspace at slack.com, install a
bot user, copy the `xoxb-…` token into OpenBao:

```
bao kv put secret/<tenant>/slack/bot-token token=xoxb-... 
```

## Smoke test

```
cargo test -p gateway-slack-connector -- list_conversations_returns_seeded_fixture
```

The seeded sandbox fixture serves a Slack-shaped 10-entity dataset
without contacting upstream; live-network mode is gated behind a
future `live-network` feature flag.

## Rate limits

Tier-3 bot (~50 req/min ≈ 1 req/sec; burst 5).
See <https://api.slack.com/apis/rate-limits>.

## Retry semantics

* `RateLimited` → exponential backoff per `Retry-After` header (live impl)
* `Unreachable` → 3× retry, then surface
* `IdempotencyConflict` → caller must reconcile

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar` — tenant→slack access policy template.
