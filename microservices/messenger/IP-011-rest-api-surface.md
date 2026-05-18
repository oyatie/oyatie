---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-011-rest-api-surface
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, openapi-conformance, contract-test-pact]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: REST API surface (OpenAPI 3.2.0 conformance)

## Intent

Land the `-rest` crates per BC, bound to the OpenAPI 3.2.0 contract at
`contracts/openapi/messenger.yaml`. Per-endpoint Cedar evaluation; OIDC bearer
authn; X-Scope-OrgID enforcement.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-*-rest/src/{handlers,middleware}.rs` | create per BC |
| `tests/openapi_conformance.rs` | create — schemars-derived schema validated against contracts/openapi/ |

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-channel-store-rest
cargo nextest run -p oya-messenger-message-stream-rest
cargo nextest run --test openapi_conformance
oya gate validate openapi-spec --microservice messenger
```

## Test Plan

- For every `paths:` entry: at least 1 success + 1 401 + 1 403 + 1 404 case.
- Pact contract test against TypeScript SDK consumer (M01+1 SDK).
- Negative: missing X-Scope-OrgID → 401; wrong tenant → 401; Cedar deny → 403.

## Next IP

[`IP-012-websocket-frame-protocol.md`](IP-012-websocket-frame-protocol.md)
