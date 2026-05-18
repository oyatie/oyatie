---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-007-collab-crdt-adapter-redis-worker
status: pending
execution_unit: ChangeSet
owner: axis-docs + ops-sre-reliability
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: collab-crdt -adapter-redis + worker (WebSocket gateway)

## Intent

Implement PresenceBroadcast + CrdtOpSpool against Valkey 8.1 (Redis wire-compat) cluster mode + a long-running WebSocket gateway worker that fans out CRDT ops via consistent-hash on `document_id`.

## ChangeSet boundary

3 crates: adapter-redis + worker + sdk + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-collab-crdt-adapter-redis/src/{lib,presence,op_spool,lease}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-worker/src/{lib,ws_gateway,lease_manager,fanout,reconcile}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-sdk/src/{lib,client,reconnect,op_buffer}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-app/src/main.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-collab-crdt-adapter-redis -- single_flight_op_publish
cargo nextest run -p oya-docs-collab-crdt-worker -- ws_lease_per_tenant_doc
cargo nextest run -p oya-docs-collab-crdt-worker -- crdt_op_signature_verified
```

## References

- ADR-DOCS-0001.
- `policy/editor-isolation.md` §"CRDT Op Authenticity".
