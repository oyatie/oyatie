---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-005-message-stream-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, oya-governance-port-location, oya-governance-statelessness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: message-stream kernel + domain

## Intent

Port traits: `MessageStore`, `MessageSearchIndex`, `RealtimeBroadcaster`.
Domain rules: edit-window enforcement, tombstone semantics, content-hash
integrity over (body | ciphertext).

## ChangeSet boundary

`-kernel` + `-domain` of `message-stream` BC only.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-message-stream-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-messenger-message-stream-domain/src/{message,reaction,edit,tombstone,content_hash}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-message-stream-kernel
cargo nextest run -p oya-messenger-message-stream-domain
cargo run -p oya-dev-cli -- gate validate port-location --microservice messenger
cargo run -p oya-dev-cli -- gate validate statelessness --microservice messenger
```

## Test Plan

- content-hash: sha256 over (timestamp, body|ciphertext, channel_id, author_ref).
- edit-window: ≥ 24h after post-time, edit rejected at domain layer.
- tombstone: delete keeps audit row + content_hash; body wiped.

## Next IP

[`IP-006-message-stream-adapters.md`](IP-006-message-stream-adapters.md)
