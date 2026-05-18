---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-013-rest-websocket-protocol
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: REST + WebSocket frame protocol

## Intent

Author the document-store-rest + sharing-rest + comments-rest + export-import-rest endpoints per OpenAPI 3.2.0 + WebSocket gateway frame protocol for CRDT op streaming. OIDC + per-tenant API key + share-link token (Ed25519) auth surfaces.

## ChangeSet boundary

Per-BC `-rest` crates already scaffolded in earlier IPs; this IP wires the cross-cutting auth + protocol layer.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-shared-protocol/src/{lib,oidc,api_key,share_link_verify,ws_frame}.rs` | create (shared protocol crate) |
| `microservices/docs/src/crates/oya-docs-document-store-rest/src/lib.rs` | extend |
| `microservices/docs/src/crates/oya-docs-collab-crdt-worker/src/ws_gateway.rs` | extend (WebSocket protocol) |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-shared-protocol -- oidc_tenant_claim_extraction
cargo nextest run -p oya-docs-shared-protocol -- share_link_constant_time_verify
cargo nextest run -p oya-docs-shared-protocol -- ws_frame_signature_required
```

## References

- OpenAPI 3.2.0 spec — `spec.openapis.org/oas/v3.2.0`.
- RFC 6455 (WebSocket).
- `contracts/openapi/docs.yaml`.
- `contracts/asyncapi/docs-events.yaml`.
