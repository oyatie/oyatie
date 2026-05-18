---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-006-resolver-adapter-openbao
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cargo-test, integration-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-cloud-secrets-secret-reference-resolver-adapter-openbao

## Intent

Implement `OpenBaoClient` port against OpenBao 2.x HTTP API. mTLS via SPIFFE SVID. Per-tenant Kubernetes auth method backed authentication.

## ChangeSet boundary

One new crate at `…/oya-cloud-secrets-secret-reference-resolver-adapter-openbao/`. Adapter-* per ADR-0105 Amendment 3.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/Cargo.toml` | create — deps: `reqwest`, `rustls`, `spiffe`, async-trait |
| `…/src/lib.rs` | create |
| `…/src/client.rs` | create — `OpenBaoHttpClient { http: reqwest::Client, endpoint: Url, svid_provider: SpiffeProvider }` |
| `…/src/auth.rs` | create — Kubernetes auth method exchange (SVID → OpenBao token) |
| `…/src/kv_v2.rs` | create — KV v2 read/write/list |
| `…/src/revoke.rs` | create — revoke endpoint |
| `…/src/sse.rs` | create — server-sent events client for revocation push |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-adapter-openbao.yaml` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-adapter-openbao
# Integration test against sandbox OpenBao:
docker-compose -f microservices/cloud-secrets/tests/integration/docker-compose.yml up -d
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-adapter-openbao --features integration
```

## Test Plan

- Integration: read/write/list/revoke against real OpenBao 2.x instance via testcontainers.
- SSE: simulate disconnect + reconnect; verify catch-up via `since=<last-event-id>`.

## Halt Conditions

- mTLS handshake without SPIFFE SVID validation — BLOCKER.

## Next IP

`IP-007-resolver-rest-and-sdk-rust.md`
