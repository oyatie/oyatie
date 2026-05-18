---
id: ADR-COMM-0002
status: Accepted
date: 2026-05-17
microservice: community
deciders: axis-community, council-architecture, axis-data-science
owner: axis-community
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0135
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/community/PRD.md (FR-02, FR-05, §"Performance" vote-cast row)
  - microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md (IP-006 voting-engine)
  - microservices/community/IP-006-voting-engine.md
  - microservices/community/capabilities/vote-cast.yaml
purpose: Close PRD-community FR-02 + FR-05's open ranking question — fix the canonical ranking algorithm for posts and answers (Wilson lower-bound + Reddit-style logarithmic time decay), with a documented fallback for low-vote regimes (Hacker News-style score) and tie-break rules.
---

# ADR-COMM-0002: Voting engine ranking — Wilson lower-bound + Reddit-style logarithmic time decay; Hacker News fallback for low-vote regimes

## Status

Accepted — 2026-05-17.

## Context

PRD-community FR-02 mandates Stack-Overflow-grade voted Q&A with accepted-answer semantics. FR-05 mandates upvote / downvote with p99 ≤ 100 ms vote-cast latency. The PRD does not, however, fix *how* the votes combine into a rank score. The choice is load-bearing because the same vote tally can produce wildly different orderings depending on the algorithm — and ranking is the surface that determines whether the µservice feels like Stack Overflow (best answer wins), Reddit (fresh-and-popular wins), Hacker News (fresh-and-meritocratic wins), or YouTube (engagement-maximising). The wrong choice can train tenant communities into bad-faith behaviour (downvote brigading, recency-gaming, low-quality-but-fresh dominance).

Industry algorithms in the candidate set:

- **Wilson lower bound of the binomial confidence interval at z=1.96 (95% CI)** — Evan Miller, 2009; used by Reddit for "best" comment ordering. Treats up/down votes as a Bernoulli trial; returns the lower bound of the true proportion. Resistant to small-sample noise; great for Q&A "best answer" semantics.
- **Reddit "hot" formula** — `log10(max(|s|,1)) * sign(s) + (t - t_epoch) / 45000`. Post score combined with logarithmic time decay; old posts decay regardless of score.
- **Hacker News ranking** — `(votes - 1) / (t_hours + 2)^1.8`. Fresh and meritocratic; aggressive decay penalises gaming.
- **Reddit "best" (modern)** — uses the Wilson lower-bound for comment ranking, separate from "hot" for posts.
- **YouTube / TikTok engagement ranking** — opaque, ML-driven; not appropriate for an auditable community µservice.
- **Eigenvector / PageRank-on-comments** — high-quality but expensive; appropriate at Quora-scale only.

PRD §"Security" + ADR-COMM-0001 commit moderation actions to audit-chain sealing; the ranking algorithm must be a *pure function* of public state (vote counts + timestamps) so that tenants and auditors can verify why a post is ranked where it is. ML-driven ranking would break this auditability.

The ranking surface has three distinct sub-surfaces with different requirements:

1. **Accepted-answer / "best answer" ranking under a question** (FR-02): the goal is "most-likely-correct answer rises"; vote sample sizes can be small (≤ 5 votes is common); demands *small-sample resistance*. Wilson lower-bound is the industry-canonical choice.
2. **Feed ranking of fresh posts within a space** (FR-01): the goal is "fresh + popular wins"; balances novelty against vote signal; demands *time decay*. Reddit-hot or HN-rank are the industry choices.
3. **Search-result ranking** (FR-07): out of scope for this ADR; lives in the search backend ADR-COMM-0004 with BM25 + vote-as-feature.

Tie-breaking is independent of the ranking formula. The PRD does not fix the tie-break order.

## Decision

The community voting engine ships **three ranking modes** behind a `VoteRanker` trait in `oya-community-voting-engine-kernel`:

1. **`BestAnswerRanker` — Wilson lower-bound at z=1.96 (95% CI).**
   - Formula: `(p + z²/(2n) − z·√((p(1−p) + z²/(4n))/n)) / (1 + z²/n)` where `p = up/(up+down)`, `n = up+down`, `z = 1.96`.
   - Applied to: accepted-answer ordering under a question (FR-02), top-comment ordering under a post.
   - Behaviour: small-n posts are penalised (5 votes at 5/0 ranks below 100 votes at 95/5). Resistant to brigade-and-bury attacks because adding one downvote to a high-n post moves Wilson little; adding one downvote to a low-n post moves Wilson a lot.

2. **`HotFeedRanker` — Reddit "hot" with logarithmic decay.**
   - Formula: `sign(s) * log10(max(|s|, 1)) + (t_seconds_since_epoch − 1136066400) / 45000` where `s = up − down`.
   - The `/ 45000` term gives roughly a 12.5-hour half-life relative to the log score; this matches the Reddit-published constant and is what tenant communities expect from a "fresh + popular" feed.
   - Applied to: per-space announcement + discussion feed (FR-01, FR-04).
   - Behaviour: a 1-day-old post needs ~10× the score of a fresh post to outrank it; a 1-week-old post is effectively buried unless score is exceptional.

3. **`HnFallbackRanker` — Hacker News formula for low-vote regimes.**
   - Formula: `(votes − 1) / (t_hours + 2)^1.8`.
   - Applied to: tenants smaller than 100 active members per space (where Wilson + Reddit-hot are both starved of signal); also as the default for the first 30 days of a brand-new space within an established tenant.
   - Behaviour: aggressively rewards fresh-and-supported posts; aggressively penalises old posts; gives small spaces "something is happening" UX while signal accumulates.

**Mode selection is a per-space configuration** (`space.ranking_mode`) defaulting to:
- Question + Accepted-Answer → `BestAnswerRanker` (non-configurable; Q&A semantics depend on Wilson)
- Per-space feed → `HotFeedRanker` if active-members ≥ 100, `HnFallbackRanker` otherwise
- New space (< 30 days) → `HnFallbackRanker` regardless of size

**Tie-break rule** (applies to all three modes when ranks are equal to within `1e-9`):
1. Higher absolute up-count first.
2. Earlier `created_at` first (rewarding "first to post a quality answer").
3. Lexicographic post_id (ULID prefix-time → second-level recency).

**No engagement / dwell-time / view-count signals** are used. The ranking is a pure function of `(up, down, created_at, accepted)`. This is a regulatory-defensible posture (no opaque ML) and matches the Stack-Overflow + Reddit auditable-ranking precedent.

**Vote storage**: per `IP-006` Redis-buffered counter + Postgres flush, with idempotency-key per `(member, post)`. Vote state is `up | down | clear` (clear is a real verb; setting `clear` removes both the up and the previous-direction vote). The ranking formula consumes the post-flush Postgres tally; the Valkey layer is read-through cache only.

**Vote reputation gates** (per `capabilities/vote-cast.yaml`):
- account_age ≥ 24 h OR rate-limit multiplier 10×.
- reputation ≥ 100 OR downvote daily cap = 30.
These are pre-tally gates and do not alter the ranking formula.

## Alternatives Considered

### A. Reddit "hot" formula alone (no Wilson + no HN fallback)
- Pros: single formula; well-understood; published constants.
- Cons: small-sample noise (one downvote on a 1-up post halves the rank); poor "accepted answer" UX because Q&A is fundamentally a small-n setting where time decay is anti-goal; tenants who want Stack-Overflow Q&A semantics get Reddit-feed semantics.
- Rejected: Q&A and feed are different surfaces; one formula cannot serve both well.

### B. Hacker News formula alone
- Pros: simple; well-understood; defends against gaming.
- Cons: extremely time-aggressive; not suitable for "best answer survives indefinitely" Q&A semantics; not suitable for announcements that should remain prominent for days.
- Rejected: only suitable as a low-vote-regime fallback.

### C. Engagement-driven ranking (YouTube-style; ML + dwell-time + scroll-depth)
- Pros: maximises engagement metrics; modern industry pattern.
- Cons: opaque; not auditable; trains tenant communities into engagement-maximising behaviour (clickbait, controversy, rage-content); not regulatory-defensible under EU DSA Art. 27 transparency requirements; fundamentally misaligned with "tenant community surface" mission.
- Rejected: opaque ranking is incompatible with the µservice's audit-chain posture.

### D. PageRank-on-comments / eigenvector-of-citation-graph
- Pros: high-quality results at Quora-scale.
- Cons: per-vote computation cost is O(graph) not O(1); incompatible with p99 ≤ 100 ms vote-cast latency; over-engineered for tenant communities at < 10⁶ members per tenant.
- Rejected: cost vs. benefit unfavourable at oyatie's tenant scale.

### E. Pure raw `up − down` net score (StackOverflow default)
- Pros: simplest possible; matches user mental model.
- Cons: ignores sample size (50/0 ranks below 100/49 = +51 vs. +50, but the 50/0 is clearly higher-quality); ignores time entirely (old posts never decay); trivially gameable by brigading.
- Rejected: fails on both small-sample resistance and time decay.

### F. Per-tenant configurable formula (the "let tenants pick" escape hatch)
- Pros: maximally flexible.
- Cons: tenants do not have data scientists; each tenant configuring their own formula will get bad results; cross-tenant analytics + benchmarking becomes impossible; support burden explodes.
- Rejected: tenant choice between three named modes (the decision above) is the right granularity; arbitrary-formula configuration is not.

## Consequences

### Positive

- Stack-Overflow-grade Q&A semantics out of the box via Wilson lower-bound; FR-02 satisfied directly.
- Reddit-grade feed semantics via Reddit "hot"; users coming from Reddit / Discourse have aligned mental models.
- Small-tenant + new-space UX is not starved for signal because HN fallback fires; satisfies the "tenant just bought oyatie, has 12 members, the feed should feel alive" UX requirement.
- Ranking is a pure function of public state → auditable, deterministic, regression-testable. CI lane `community-ranking-deterministic-snapshot` BLOCKS PRs that change the formula without an ADR amendment.
- No ML, no opaque signals → EU DSA Art. 27 transparency compliant; tenants can explain the ranking to their members.

### Negative

- Three ranking modes triple the implementation + test surface compared to a single-formula design. Mitigated by the modes being ≤ 30 lines each in `oya-community-voting-engine-kernel`.
- The `HnFallbackRanker` → `HotFeedRanker` cutover when a space crosses 100 active members is a UX cliff (posts may visibly reorder). Mitigated by a documented 24-hour blended-rank smoothing window in `IP-006` successor-IP.
- Tie-break by `created_at` rewards being-first; can be perceived as zero-sum by late-joining members. Accepted as the industry-canonical tie-break; documented in tenant-facing help.

### Operational

- New crate signatures: `VoteRanker` trait in `oya-community-voting-engine-kernel`; concrete `BestAnswerRanker`, `HotFeedRanker`, `HnFallbackRanker` in `-domain`.
- Postgres view: `community.post_ranked_feed` materialised every 5 min per space; idle spaces skipped.
- Valkey hot-cache: top-N rank per space cached for 30 s.
- Per-space configuration column `community.spaces.ranking_mode` (enum); migration in IP-006 successor-IP.
- Dashboards: `dashboards/vote-rate.json` extended with rank-mode-distribution panel.
- CI lane `community-ranking-deterministic-snapshot`: fixed input vote sequences produce identical rank order across runs.

### Regulatory

- **EU Digital Services Act Art. 27** — transparency of recommender systems: the algorithm is documented in this ADR + in tenant-facing help text; tenants can explain to members why a post ranks where it does.
- **GDPR Art. 22** (automated decision-making): voting-rank affects visibility but does not affect a "decision producing legal effects or similarly significant" outcome; ranking is informational, not decisional.
- **KR PIPA Art. 27** (rights of data subjects): pack-kr deployments expose a "why is this ranked here?" endpoint exposing the formula + the inputs, satisfying explainability.

## References

- Evan Miller, "How not to sort by average rating," 2009 — Wilson lower-bound for the canonical small-sample-resistant rank — `https://www.evanmiller.org/how-not-to-sort-by-average-rating.html`
- Reddit ranking algorithm post-mortem — `https://medium.com/hacking-and-gonzo/how-reddit-ranking-algorithms-work-ef111e33d0d9`
- Hacker News ranking formula — Paul Graham, Y Combinator post — `https://www.righto.com/2013/11/how-hacker-news-ranking-really-works.html`
- Stack Overflow ranking explainer — `https://stackoverflow.blog/2009/09/01/code-and-creativity/`
- ADR-0135 — Connect-unbundle (parent ADR establishing the community µservice)
- ADR-0131 — Per-microservice flat layout
- EU Digital Services Act Art. 27 — recommender-system transparency — `https://eur-lex.europa.eu/eli/reg/2022/2065`
- `microservices/community/PRD.md` FR-02, FR-05
- `microservices/community/IP-006-voting-engine.md`
- `microservices/community/capabilities/vote-cast.yaml`
