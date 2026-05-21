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

## Wave 15 substance conversion — message stream domain

### §A Problem

Messenger cannot be Slack/Teams-class if message semantics live only in database rows.
This IP closes the domain gap for message identity, edit windows, reactions, tombstones, and content-hash evidence.

### §B Approach

Define pure kernel ports and domain entities for the message stream before any Postgres, Meilisearch, or Valkey
adapter code lands.
The domain owns invariants; adapters persist and broadcast already-validated state.

### §C Deliverables

- `src/crates/oya-messenger-message-stream-kernel/src/{ports,entities,errors}.rs`
- `src/crates/oya-messenger-message-stream-domain/src/{message,reaction,edit,tombstone,content_hash}.rs`
- tests for edit-window, tombstone, reaction, and hash invariants

### §D Implementation

1. Model `Message` with tenant, channel/direct-conversation, author, context, body-or-ciphertext, and posted time.
2. Enforce the 24-hour edit window in the domain layer.
3. Convert deletes into tombstones while preserving audit metadata and hash.
4. Hash timestamp, author, channel, and body/ciphertext deterministically.
5. Reject Personal/Professional context coercion at type boundary.
6. Expose only ports for store, search index, and realtime broadcaster.

### §E Acceptance

Nextest must prove edit rejection after 24 hours, tombstone body wipe, content hash stability, and no adapter imports
inside kernel/domain crates.

### §F Evidence

Local anchors: `policy/dual-context-isolation.md`, `policy/personal-dm-scope.cedar`,
`policy/tenant-scope.cedar`, `slos/message-send-latency.openslo.yaml`.

### §G Counterparts

Slack and Teams anchor edit/delete expectations, Discord anchors high-volume reactions, and Matrix anchors event
immutability; oyatie closes parity with stronger content-hash/audit semantics.
