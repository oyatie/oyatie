---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-010-hashtag-and-trending-bc
status: pending
owner: axis-shorts
depends_on: [IP-007, IP-008]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: hashtag + trending BC end-to-end

## Intent

- `hashtag` BC: #tag parse + per-tag corpus + trending input emission.
- `trending` BC: windowed trend compute over hashtags + sounds + entities; sound-of-the-week derivation; 5-min compute window.

## ChangeSet boundary

7 + 8 = 15 crates.

## Concrete File Targets

Key entities: `Hashtag`, `HashtagCorpus`, `HashtagEmission`, `TrendWindow`, `TrendRank`, `SoundOfTheWeek`.

Ports: `HashtagStore`, `TrendingComputer`, `TrendingPublisher`.

Anti-poisoning: per-author influence cap; foundry-guardrails sybil detector; trending compute dedup keyed by `(tenant_id, sound_id, author_ref)`.

## Acceptance Gates

```bash
cargo nextest run -p oya-shorts-hashtag-{kernel,domain,usecase,adapter-postgres}
cargo nextest run -p oya-shorts-trending-{kernel,domain,usecase,adapter-postgres,adapter-redis,worker}
```

E2E: post with #tag emits hashtag-emission; trending compute produces sound-of-the-week within 5-min window.

## Next IP

[`IP-011-content-moderation-and-copyright-claim-bc.md`](IP-011-content-moderation-and-copyright-claim-bc.md)

## References

- PRD FR-11, FR-12.
- `threat-model.md` T-T-07.
