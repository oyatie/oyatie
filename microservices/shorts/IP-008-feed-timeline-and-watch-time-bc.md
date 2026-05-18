---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-008-feed-timeline-and-watch-time-bc
status: pending
owner: axis-shorts
depends_on: [IP-005, IP-007]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: feed-timeline + watch-time-tracking BC end-to-end

## Intent

- `feed-timeline` BC: chronological + For-You algorithmic feed; precompute for hot accounts; fanout-on-read for cold. Per ADR-SHORTS-0005 hybrid heuristic in P01; ML-driven in P03.
- `watch-time-tracking` BC: per-(viewer, video) watch-seconds + completion-ratio + scroll-velocity signal; ranking input.

## ChangeSet boundary

10 + 8 = 18 crates.

## Concrete File Targets

Key entities: `FeedEntry`, `ForYouSlot`, `RankSnapshot`, `FanoutPlan`, `WatchSession`, `WatchTotal`, `CompletionRatio`, `ScrollVelocity`.

Ports: `FeedCache`, `RankingProvider`, `WatchTimeStore`, `FanoutPlanner`.

Minor-account constraint: chronological-only feed default; algorithmic-recommendation requires parental-consent attestation. Per EU DSA Art. 28 + KR 청소년 보호법 + CA AB-2273 + UT SMRA + EU AVMSD Art. 28b(2).

## Acceptance Gates

```bash
cargo build -p oya-shorts-feed-timeline-app
cargo nextest run -p oya-shorts-feed-timeline-{kernel,domain,usecase,adapter-postgres,adapter-redis}
cargo nextest run -p oya-shorts-watch-time-tracking-{kernel,domain,usecase,adapter-postgres,adapter-redis}
```

E2E: feed-load p95 ≤ 250ms (top 10 videos); watch-time signal emits at completion-ratio ≥ 0.1; ranking_explanation API emits EU DSA Art. 27 signals; minor account renders chronological-only.

## Halt Conditions

- Redis split-brain — engage cell + cloud-secrets.
- Feed-load p95 > 250ms — capacity revision.

## Next IP

[`IP-009-like-share-comment-and-repost-bc.md`](IP-009-like-share-comment-and-repost-bc.md)

## References

- PRD FR-07, FR-08.
- ADR-SHORTS-0005 (ranking algorithm).
- `slos/feed-load-latency.openslo.yaml`.
- `slos/content-policy-enforcement-correctness.openslo.yaml`.
- EU DSA Arts. 27, 28.
