---
id: ADR-SHORTS-0005
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-architecture, ops-security, axis-shorts, axis-foundry-runtime, council-privacy, ops-legal
owner: axis-shorts
supersedes: []
superseded_by: []
related:
  - ADR-0022
  - ADR-0126
  - ADR-0131
  - ADR-0132
  - ADR-SHORTS-0003
  - ADR-SHORTS-0006
  - ADR-SOC-0001  # paired sibling ranking ADR
related_artifacts:
  - microservices/shorts/PRD.md
  - microservices/shorts/capabilities/T2-auto.yaml
  - microservices/shorts/slos/feed-load-latency.openslo.yaml
  - microservices/shorts/runbooks/moderation-classifier-rollback.md
purpose: Establish the feed-ranking algorithm strategy for the shorts µservice across P01 (heuristic) and P03 (ML-driven), within EU AI Act Annex III §1(a) high-risk obligations; aligned with social ADR-SOC-0001.
---

# ADR-SHORTS-0005: Feed-ranking algorithm — hybrid chronological-first + heuristic-algorithmic in P01; ML-driven ranking deferred to P03 with EU AI Act high-risk obligations

## Status

Accepted — 2026-05-17.

## Context

shorts ships a feed-timeline BC that materialises and renders per-user feeds — both algorithmic For-You and chronological. Industry leaders (TikTok, Reels, Shorts, Spotlight, Likee, Kuaishou, Douyin) ship algorithmic feeds as the default with chronological as optional; Bluesky offers a marketplace of user-selectable algorithms; Mastodon defaults chronological-only.

PRD §"Out-of-scope" defers ML-driven ranking to P03 because it depends on:
- `foundry-runtime` model deployment + classifier evaluation pipeline.
- EU AI Act Annex III §1(a) high-risk obligations (Arts. 9-15 + 50 + 73).

P01 must still ship a usable algorithmic feed to be competitive with TikTok / Reels / Shorts; same Annex III §1(a) high-risk obligations apply to **any** system "significantly influencing" user-visible content — heuristic AND ML.

EU DSA Art. 27 (recommender system transparency) and EU AI Act Art. 50 (transparency obligation) require:
- Main parameters used in ranking, exposed via ranking_explanation API.
- User-controllable option to switch to a non-profile-based recommender (chronological).

KR PIPA Art. 29-2 + EU AI Act Art. 13 require an explanation API surface and an opt-out / human-review path.

Per parallel ADR-0126 dual-context invariant: Personal-tier feed and Professional-tier feed are rendered by separate `FeedCache` ports (DCI invariant).

Per ADR-SHORTS-0006 minor-protection: minor accounts default to chronological-only per EU DSA Art. 28 + KR 청소년 보호법.

This ADR mirrors social `ADR-SOC-0001` (feed-ranking algorithm in social).

## Decision

oyatie shorts adopts a **three-step strategy** (paired with ADR-SOC-0001):

### Step 1 (M03 launch): Hybrid chronological-first + heuristic-algorithmic

- **Default mode**: chronological — sorted by `posted_at` descending across follow-graph + own posts + per-pack public-feed surface.
- **Optional mode (user opt-in)**: heuristic-algorithmic — sorted by:
  ```
  rank_score = 0.4 * watch_time_completion_signal
             + 0.2 * engagement_signal (like + share + comment)
             + 0.15 * creator_affinity_signal (own-watch history of creator)
             + 0.15 * sound_popularity_signal
             + 0.10 * recency_decay
  ```
  Heuristic is fully deterministic + auditable from code; no ML model in P01.
- Per EU DSA Art. 27: user can switch between chronological + algorithmic via single setting; `ranking_explanation` API populates contributing signals when algorithmic mode active.
- Per EU AI Act Art. 50: client SDK renders "AI-assessed ranking" label when algorithmic mode active (label same for P01 heuristic + P03 ML to preserve interface stability).
- Per parallel ADR-0126: Personal-tier feed + Professional-tier feed rendered by separate `FeedCache` ports.

### Step 2 (M04+): ML-driven ranking model

- Owned by `foundry-runtime`; deployed as T2 capability per `capabilities/T2-auto.yaml`.
- Inputs: same signals as P01 heuristic + content-embedding similarity to viewer-history + sound-embedding similarity + creator-similarity.
- Outputs: ranked feed slice with `contributing_signals` array.
- EU AI Act Annex III §1(a) HIGH-RISK classification confirmed; Arts. 9-15 + 50 obligations operative per ADR-SHORTS-0003.
- Per-release golden-set eval (nDCG@10 + bias-audit by protected groups); model card per release; appeal-via-revert-to-chronological path always available.

### Step 3 (M05+): User-selectable algorithm marketplace

- Bluesky-style; per PRD Open Question follow-up.
- Out-of-scope for M03 / M04; ADR-SHORTS follow-up.

### Minor-account default (per ADR-SHORTS-0006)

Minor accounts get **chronological-only** by default per EU DSA Art. 28 + KR 청소년 보호법; algorithmic-recommendation requires parental-consent attestation. Per EU AVMSD Art. 28b(2) minor-protection floor.

### Pack-specific defaults

- **pack-us-healthcare**: algorithmic ranking disabled by default per HIPAA Safe Harbor §164.514 (no automated assessment over PHI); tenants may opt-in with BAA + per-account attestation.
- **pack-kr**: KR PIPA Art. 29-2 individual right to opt-out of automated decisions; UI exposes opt-out.

The heuristic ranking is **not exempt from EU AI Act Art. 50 transparency**: any system that significantly influences user-visible content carries the obligation even if the model is rule-based.

## Alternatives Considered

### A. Chronological-only feed (Mastodon-style; no algorithmic mode at all)

- Pros: simplest implementation; no EU AI Act obligations triggered by ranking model (only by moderation classifier); lowest cost.
- Cons: uncompetitive vs TikTok / Reels / Shorts / Spotlight which all default algorithmic; user retention suffers per published industry studies (algorithmic feed increases time-on-platform ~30 %); fails to deliver PRD Tenant Outcome 1 "TikTok-class".
- Rejected: kills hero-product viability.

### B. ML-driven ranking from day-1 (skip the heuristic step)

- Pros: parity with TikTok / Reels / Shorts from launch.
- Cons: requires `foundry-runtime` model deployment + EU AI Act notified-body engagement before M03; not feasible in P01 timeline; rollback complexity higher than heuristic; deployment risk.
- Rejected: incompatible with M03 timeline + notified-body engagement.

### C. Hybrid chronological + heuristic in P01, ML in P03 (this ADR's choice; paired with ADR-SOC-0001)

- Pros: ships P01 with competitive algorithmic mode (heuristic); preserves user choice via chronological default; EU DSA Art. 27 obligations met; EU AI Act Art. 50 label rendered universally; clear upgrade path to ML in P03 without breaking SDK / UX.
- Accepted.

### D. Marketplace of user-selectable algorithms (Bluesky AT Protocol style)

- Pros: maximum user control; matches Bluesky differentiator; potential brand value.
- Cons: substantially higher P01 implementation complexity; requires authoring + curating + sandboxing algorithm contributions; EU AI Act risk-management obligations multiply across N algorithms; defer to M05+.
- Rejected (for M03 / M04); kept open for ADR-SHORTS follow-up at M05+.

### E. Per-tenant tenant-admin-configurable ranking weights

- Pros: tenants tailor ranking to their community values; explainable.
- Cons: per-tenant configuration creates per-tenant EU AI Act risk surface (each tenant's configured weights effectively become a distinct system); regulatory complexity multiplies; defer.
- Rejected (for P01); kept open for ADR-SHORTS follow-up.

### F. Pure-engagement ranking (TikTok-style: watch-time + completion-ratio dominant)

- Pros: simplest signal; matches TikTok's reportedly-dominant approach.
- Cons: doomscroll-prone; EU DSA Art. 28 minor-protection requires user controllability; per AvAct Art. 28b commercial-communication carveouts; over-reliance on single signal fails accuracy bound.
- Rejected: over-simple; underprotects.

## Consequences

### Positive

- M03 ships competitive algorithmic feed without ML deployment risk; users get choice (chronological default + algorithmic opt-in).
- EU DSA Art. 27 recommender transparency obligations met universally (heuristic exposes signals; ML in M04+ same SDK surface).
- EU AI Act Art. 50 transparency labels operative from M03 → consistent surface across M03 heuristic + M04+ ML.
- Personal-tier and Professional-tier feeds remain dual-context-invariant; DCI-10 preserved.
- Minor accounts get chronological-only default per EU DSA Art. 28 + KR 청소년 보호법 + AVMSD Art. 28b.
- pack-us-healthcare default chronological-only (HIPAA Safe Harbor).
- Heuristic ranking is auditable from code (deterministic; no model-card complexity until M04+).
- M04+ ML upgrade path well-defined via `foundry-runtime` T2 capability without SDK / UX breakage.

### Negative

- Heuristic ranking quality is bounded; engagement-vs-time tradeoff weaker than ML systems; mitigated by M04+ path.
- Per-release golden-set eval needed even for heuristic (EU AI Act Art. 15 accuracy); some infrastructure cost.
- EU AI Act notified-body engagement required before M04+; timeline coordination with foundry-runtime + council-privacy.
- Tenants requesting per-tenant ranking weights deferred to M05+; gtm may field requests.

### Operational

- Cargo workspace: `oya-shorts-feed-timeline-domain` carries the heuristic `rank_score` function; M03 has no `oya-shorts-feed-timeline-adapter-foundry-runtime` (added in M04+).
- Cedar policy: ranking_explanation API surface for Art. 27 visible from M03 (heuristic-aware).
- Runbook `runbooks/moderation-classifier-rollback.md` (paired with this ADR's classifier counterpart) covers ML model rollback in M04+; M03 has simpler "revert to chronological for affected tenants" path.
- CI lane `oya-governance-eu-ai-act-conformance` registered in IP-015; covers heuristic + ML modes.
- Per-release golden-set eval (M03: heuristic eval; M04+: ML eval) maintained per ADR-SHORTS-0003 pipeline.

### Regulatory

- **EU AI Act 2024/1689 Arts. 9-15 + 50**: covered by ADR-SHORTS-0003 pipeline (paired classifier ADR); heuristic mode still emits Art. 50 transparency label.
- **EU DSA 2065/2022 Art. 27**: recommender transparency satisfied; user-controllable chronological mode.
- **EU DSA Art. 28**: minor-account protection — chronological-only default; aligns with `age-gate` BC.
- **EU AVMSD Art. 28b(2)**: minor-protection floor met.
- **KR PIPA Art. 29-2**: automated-decision opt-out — user can switch to chronological; explanation API satisfies right-to-explanation.
- **HIPAA Safe Harbor §164.514**: pack-us-healthcare default chronological-only.

## References

- ADR-0022 Bominal autonomy-tier classification (T0/T1/T2 inherited).
- ADR-0126 Connect dissolution (parallel; dual-context source).
- ADR-0131 per-microservice flat layout.
- ADR-0132 suite-and-bundle dissolution.
- ADR-SHORTS-0003 (content-moderation classifier bounds; paired EU AI Act ADR).
- ADR-SHORTS-0006 (minor protection + age-gate; paired).
- ADR-SOC-0001 (sibling social ranking ADR; paired pattern).
- EU AI Act 2024/1689 Annex III §1(a) high-risk; Arts. 9-15, 50, 73.
- EU DSA Regulation 2065/2022 Arts. 27, 28.
- EU AVMSD 2018/1808 Art. 28b.
- KR PIPA Arts. 28, 29-2.
- HIPAA 45 CFR §164.514 Safe Harbor.
- Google SRE Workbook ch. 5 (multi-window multi-burn-rate).
- TikTok / Reels / Shorts published engagement studies.
- Bluesky AT Protocol algorithm marketplace `docs.bsky.app`.
- Mastodon chronological-only `docs.joinmastodon.org`.
- `microservices/shorts/PRD.md` §Open Questions 4, 6.
- `microservices/shorts/capabilities/T2-auto.yaml`.
- `microservices/shorts/slos/feed-load-latency.openslo.yaml`.
- `microservices/shorts/policy/dual-context-isolation.md` DCI-10.
