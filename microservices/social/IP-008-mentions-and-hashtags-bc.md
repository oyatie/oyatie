---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-008-mentions-and-hashtags-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

# IP-008: mentions + hashtags BCs (kernel → domain → usecase → adapter + worker + sdk)

## Intent

Two BCs in one ChangeSet because they share ontology lookup + emit to
trending-topics:

- **mentions**: parse @mention tokens from posts; resolve via Ontology; emit
  `MentionEmitted` event + fanout to notifications + messenger-bridge.
- **hashtags**: parse #tag tokens; per-tag corpus emission to trending-topics;
  per-tag search index emission.

## ChangeSet boundary

`mentions` + `hashtags` BCs.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-mentions-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-mentions-domain/src/{mention,mention_target,fanout_plan}.rs` | create |
| `src/crates/oya-social-mentions-usecase/src/{parse,resolve,fanout}.rs` | create |
| `src/crates/oya-social-mentions-adapter/src/ontology_client.rs` | create |
| `src/crates/oya-social-mentions-worker/src/dispatcher.rs` | create |
| `src/crates/oya-social-hashtags-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-hashtags-domain/src/{hashtag,hashtag_corpus,hashtag_emission}.rs` | create |
| `src/crates/oya-social-hashtags-usecase/src/{parse,emit_corpus}.rs` | create |
| `src/crates/oya-social-hashtags-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-hashtags-worker/src/aggregator.rs` | create |
| `tests/mentions_hashtags_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-mentions-kernel
cargo nextest run -p oya-social-hashtags-kernel
```

## Test Plan

- Mention parse: extract `@handle` from body text + cross-link to messenger.
- Mention resolve via Ontology mock (E2E AC-05 ≤ 250ms).
- Hashtag parse + emit corpus + trending-topics input.
- Mention storm cap (FM-08): per-post mention cap default 50.
- Per-pack hashtag thresholds (e.g., max 10 hashtags per post pack-trial; up to 30 pack-internal).

## Halt Conditions

- Ontology lookup failure cascade (FM-19): graceful degradation to raw-text mode.

## Next IP

[`IP-009-trending-topics-bc.md`](IP-009-trending-topics-bc.md)
