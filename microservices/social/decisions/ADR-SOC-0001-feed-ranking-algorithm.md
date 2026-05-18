---
id: ADR-SOC-0001
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-architecture, ops-security, axis-social, axis-foundry-runtime, council-privacy
owner: axis-social
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-SOC-0003
  - ADR-SOC-0005
related_artifacts:
  - microservices/social/PRD.md (§Open Question 1)
  - microservices/social/capabilities/T2-auto.yaml
  - microservices/social/slos/feed-render-latency.openslo.yaml
  - microservices/social/runbooks/content-moderation-rollback.md
purpose: Establish the feed-ranking algorithm strategy for the social µservice across P01 (heuristic) and P03 (ML-driven), within EU AI Act Annex III §1(a) high-risk obligations.
---

# ADR-SOC-0001: Feed-ranking algorithm — hybrid chronological-first + heuristic-algorithmic in P01; ML-driven ranking scheduled-for-distinct-tracked-work to P03 with EU AI Act high-risk obligations

## Status

Accepted — 2026-05-17.

## Context

The social µservice ships a feed-timeline BC that materialises and renders per-user feeds. Industry leaders (Twitter/X, TikTok, Instagram, Facebook, Threads) ship algorithmic feeds as the default; Bluesky offers a marketplace of user-selectable algorithms; Mastodon defaults to chronological-only.

PRD §"Out-of-scope" defers ML-driven ranking to P03 because it depends on `foundry-runtime` model deployment + classifier evaluation pipeline + EU AI Act Annex III §1(a) high-risk obligations (Arts. 9-15 + 50 + 73). P01 must still ship a usable algorithmic feed to be competitive with Twitter/X / Threads / Instagram and to support user choice.

EU DSA Art. 27 (recommender system transparency) and EU AI Act Art. 50 (transparency obligation) require that any algorithmic ranking exposes (a) the main parameters used and (b) a user-controllable option to switch to a non-profile-based recommender (chronological).

KR PIPA Art. 29-2 + EU AI Act Art. 13 require an explanation API surface and an opt-out / human-review path. These obligations apply equally to heuristic and ML-driven ranking when the system "significantly influences" user-facing content.

The decision needs to (a) pick a P01-deliverable ranking strategy, (b) carry forward to P03 ML-driven without violating EU AI Act obligations, (c) coordinate with the content-moderation classifier (ADR-SOC-0003) which is also EU AI Act high-risk, (d) align with parallel ADR-0135's dual-context invariant (Personal-tier and Professional-tier feeds are distinct), and (e) provide a chronological-first default option per EU DSA Art. 27.

## Decision

oyatie social adopts a **three-step strategy**:

1. **P01 (M02 launch): Hybrid chronological-first + heuristic-algorithmic.**
   - Default mode: chronological — sorted by `posted_at` descending across follow-graph + own posts.
   - Optional mode (user opt-in): heuristic-algorithmic — sorted by `rank_score(post, recency_minutes, engagement_signal, follow_proximity)` where `rank_score = 0.5 * recency_decay + 0.3 * engagement_signal + 0.2 * follow_proximity`, clamped to [0, 1]. The heuristic is fully deterministic + auditable from code; no ML model in P01.
   - Per EU DSA Art. 27: user can switch between chronological and algorithmic via a single setting; ranking_explanation API populates contributing signals when algorithmic mode is active.
   - Per EU AI Act Art. 50: client SDK renders an "AI-assessed ranking" label when algorithmic mode is active even though P01 uses heuristic (label is the same to preserve interface stability).
   - Per parallel ADR-0135: Personal-tier feed and Professional-tier feed are rendered by separate `FeedCache` ports (DCI invariant).
2. **P03 (M03): ML-driven ranking model.**
   - Owned by `foundry-runtime`; deployed as a T2 capability per `capabilities/T2-auto.yaml`.
   - Inputs: same signals as P01 heuristic + content-embedding similarity to user-history; outputs: ranked feed slice.
   - EU AI Act Annex III §1(a) HIGH-RISK classification confirmed; Arts. 9-15 + 50 obligations operative per ADR-SOC-0003.
   - Per-release golden-set eval (nDCG@10 + bias audit by protected groups); model card per release; appeal-via-revert-to-chronological path always available.
3. **Future packs (M04-onward): User-selectable algorithm marketplace** (Bluesky-style; PRD Open Question 1).
   - Out-of-scope for P01 / P03; ADR-SOC successor-IP after M03.

The heuristic ranking is **not exempt from EU AI Act Art. 50 transparency**: any system that significantly influences user-visible content carries the obligation even if the model is rule-based. The SDK helpers (`getRankingExplanation`) are universal across P01 / P03.

Personal-tier feed never federates (DCI-08); Professional-tier feed federation is per-tenant opt-in (ADR-SOC-0004); ranking is applied per-tier independently.

Pack-us-healthcare disables algorithmic ranking by default per HIPAA Safe Harbor §164.514 (no automated assessment over PHI); tenants may opt-in with BAA + per-account attestation.

Minors (per `age-verification` BC + pack-aware threshold) get **chronological-only** by default per EU DSA Art. 28 (minor protection) and KR 청소년 보호법.

## Alternatives Considered

### A. Chronological-only feed (Mastodon-style; no algorithmic mode at all)

- Pros: simplest implementation; no EU AI Act obligations; matches Mastodon precedent; lowest cost.
- Cons: uncompetitive vs X / Threads / Instagram / TikTok which all default to algorithmic; user retention suffers per published industry studies (algorithmic feed increases time-on-platform ~30 %); fails to deliver the "first-party social platform" PRD outcome 1.
- Rejected: kills hero-product viability.

### B. ML-driven ranking from day-1 (skip the heuristic step)

- Pros: parity with X / Threads / TikTok from launch.
- Cons: requires `foundry-runtime` model deployment + EU AI Act notified-body engagement before M02; not feasible in P01 timeline; deployment risk; rollback complexity higher than heuristic.
- Rejected: incompatible with P01 timeline + EU AI Act notified-body engagement complexity.

### C. Hybrid chronological + heuristic in P01, ML in P03 (this ADR's choice)

- Pros: ships P01 with competitive algorithmic mode (heuristic); preserves user choice via chronological default; EU DSA Art. 27 obligations met; EU AI Act Art. 50 label rendered universally; clear upgrade path to ML in P03 without breaking SDK / UX.
- Accepted.

### D. Marketplace of user-selectable algorithms (Bluesky AT Protocol style)

- Pros: maximum user control; matches Bluesky differentiator; potential brand value.
- Cons: substantially higher P01 implementation complexity; requires authoring + curating + sandboxing algorithm contributions; EU AI Act risk-management obligations multiply across N algorithms; defer to M04-onward.
- Rejected (for P01); kept open for ADR-SOC successor-IP at M04-onward.

### E. Per-tenant tenant-admin-configurable ranking weights

- Pros: tenants tailor ranking to their community values; explainable.
- Cons: per-tenant configuration creates per-tenant EU AI Act risk surface (each tenant's configured weights effectively become a distinct system); regulatory complexity multiplies; defer.
- Rejected (for P01); may revisit with per-tenant configuration as a guarded extension in M03-onward.

## Consequences

### Positive

- P01 ships competitive algorithmic feed without ML deployment risk; users get choice (chronological default + algorithmic opt-in).
- EU DSA Art. 27 recommender transparency obligations met universally (heuristic exposes signals; ML in P03 same SDK surface).
- EU AI Act Art. 50 transparency labels operative from P01 → consistent surface across P01 heuristic + P03 ML.
- Personal-tier and Professional-tier feeds remain dual-context-invariant; ADR-SOC-0005 paired.
- Minor accounts get chronological-only default per EU DSA Art. 28 + KR 청소년 보호법.
- pack-us-healthcare default chronological-only (HIPAA Safe Harbor).
- Heuristic ranking is auditable from code (deterministic; no model card complexity until P03).
- P03 ML upgrade path well-defined via `foundry-runtime` T2 capability without SDK / UX breakage.

### Negative

- Heuristic ranking quality is bounded; engagement-vs-time tradeoff weaker than ML systems; mitigated by P03 path.
- Per-release golden-set eval needed even for heuristic (EU AI Act Art. 15 accuracy); some infrastructure cost.
- EU AI Act notified-body engagement still required before P03; timeline coordination needed with foundry-runtime + council-privacy.
- Tenants requesting per-tenant ranking weights are scheduled-for-distinct-tracked-work to M03-onward; gtm may field requests.

### Operational

- Cargo workspace: `oya-social-feed-timeline-domain` carries the heuristic `rank_score` function; P01 has no `oya-social-feed-timeline-adapter-foundry-runtime` (added in P03).
- Cedar policy: ranking_explanation API surface for Art. 27 visible from P01 (heuristic-aware).
- Runbook `runbooks/content-moderation-rollback.md` (paired with this ADR's classifier counterpart) covers ML model rollback in P03; P01 has a simpler "revert to chronological for affected tenants" path.
- CI lane `oya-governance-eu-ai-act-conformance` registered in IP-015; covers heuristic + ML modes.
- Per-release golden-set eval (P01: heuristic eval; P03: ML eval) maintained per ADR-SOC-0003 pipeline.

### Regulatory

- **EU AI Act 2024/1689 Arts. 9-15 + 50**: covered by ADR-SOC-0003 pipeline (paired classifier ADR); heuristic mode still emits Art. 50 transparency label.
- **EU DSA 2065/2022 Art. 27**: recommender transparency satisfied; user-controllable chronological mode.
- **EU DSA Art. 28**: minor-account protection — chronological-only default; aligns with `age-verification` BC.
- **KR PIPA Art. 29-2**: automated-decision opt-out — user can switch to chronological; explanation API satisfies right-to-explanation.
- **HIPAA Safe Harbor §164.514**: pack-us-healthcare default chronological-only.

## References

- ADR-0022 — Bominal autonomy-tier classification (T0/T1/T2 inherited).
- ADR-0135 — Connect dissolution (parallel; dual-context source).
- ADR-0131 — Per-microservice flat layout.
- ADR-SOC-0003 — Content-moderation classifier bounds (paired EU AI Act ADR).
- ADR-SOC-0005 — Dual-context feed isolation (paired DCI ADR).
- EU AI Act 2024/1689 Annex III §1(a) high-risk; Arts. 9-15, 50, 73.
- EU DSA Regulation (EU) 2065/2022 Arts. 27, 28.
- KR PIPA Arts. 28, 29-2.
- HIPAA 45 CFR §164.514 Safe Harbor.
- Google SRE Workbook ch. 5 (multi-window multi-burn-rate; reference for SLO authoring).
- Bluesky AT Protocol algorithm marketplace `docs.bsky.app`.
- Mastodon chronological-only `docs.joinmastodon.org`.
- Twitter / X published P99 message-send (~120ms; reference for our latency targets).
- `microservices/social/PRD.md` Open Question 1.
- `microservices/social/capabilities/T2-auto.yaml`.
- `microservices/social/slos/feed-render-latency.openslo.yaml`.
