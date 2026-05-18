---
id: ADR-COMM-0005
status: Accepted
date: 2026-05-17
microservice: community
deciders: axis-community, council-architecture, council-ux
owner: axis-community
supersedes: []
superseded_by: []
related:
  - ADR-COMM-0002
  - ADR-0105
  - ADR-0126
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/community/PRD.md (FR-02, FR-04, §"Performance" threaded-reply-render row)
  - microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md (IP-005 thread-tree)
  - microservices/community/IP-005-thread-tree-materialised-path.md
  - microservices/community/catalog/oya-community-thread-tree-domain.yaml
purpose: Close PRD-community FR-04's open threading-shape question — fix the canonical discussion-threading model (nested replies with materialised path + depth cap 6, Reddit-style), with an additional flat-render mode for Stack-Overflow-style Q&A surfaces where flat answer order beats nested replies.
---

# ADR-COMM-0005: Discussion threading — nested replies with materialised path + depth cap 6 (Reddit-style); flat-render mode for Q&A surface

## Status

Accepted — 2026-05-17.

## Context

PRD-community FR-04 commits the µservice to "reply in a threaded discussion forum" — conversation depth is preserved without flattening. §"Performance" pins the SLO at p99 ≤ 350 ms for a 1,000-node thread render. The PRD does not, however, fix the *threading shape* — and shape determines both data model and UX.

Three industry threading shapes compete:

1. **Flat threading** (Slack channel, Twitter/X mainline, Mastodon timeline): all replies are siblings; chronological order; no parent-child semantics in render. Trivial data model; awful UX for deep discussions.
2. **Nested unlimited-depth threading** (Hacker News, Slashdot pre-2010): every reply has a parent; depth is unbounded; indentation grows linearly. Beautiful for technical discussion; UX cliff past depth ~10 (horizontal scroll, narrow column).
3. **Nested depth-limited threading** (Reddit, Discourse, Lemmy, Stack Exchange comments): every reply has a parent; depth-cap enforced (Reddit caps at ~9 on web, deeper threads collapse into "continue this thread"); indentation managed by collapsing past the cap. Battle-tested UX; data model needs path representation.
4. **Q&A-style flat answers + nested comments** (Stack Overflow / Stack Exchange): the question is the root; answers are flat siblings ordered by ADR-COMM-0002 Wilson rank; under each answer, comments are flat siblings (no nested-comments-on-comments). Hybrid shape; question-and-answer-first; minimises off-topic side conversations.

The community µservice has **two distinct surfaces** with different shape needs:

- **Discussion forum** (FR-04, IP-005): users want nested-back-and-forth; depth-limited Reddit shape is the industry winner. Bounded nesting prevents UX cliff.
- **Q&A** (FR-02): users want best answer to win; Stack-Overflow flat-answer-ordered-by-Wilson is the industry winner.

The data model question: how do we store thread structure such that a 1,000-node tree renders in p99 ≤ 350 ms (PRD performance budget)?

Storage candidates:
- **Adjacency list** (`parent_reply_id` column): natural; cheap insert; expensive subtree query (recursive CTE per parent).
- **Materialised path** (`/root_id/child_id/grandchild_id/...` string): cheap subtree query (prefix match); fixed-size cost; needs path-rewrite on subtree move (rare).
- **Nested set model** (`lft, rgt` integers): cheapest range query; expensive insert (every insert renumbers a range).
- **Closure table** (separate `(ancestor_id, descendant_id, depth)` table): cheap query; O(depth) inserts; storage overhead.

PRD §"Performance" 1,000-node thread render at p99 ≤ 350 ms requires sub-tree retrieval in < 200 ms (leaving budget for transform + serialisation). Materialised path with a btree index on `path text_pattern_ops` is the industry-standard PostgreSQL pattern; IP-005 already commits to "materialised path." This ADR formalises *why*.

## Decision

The community µservice ships **two threading shapes** behind a `ThreadRenderer` trait in `oya-community-thread-tree-kernel`:

1. **`NestedThreadRenderer` (default) — Reddit/Discourse-style nested with depth cap 6.**
   - Data model: materialised path stored in `community.replies.path text` (e.g., `/post_abc123/reply_def456/reply_ghi789`).
   - PostgreSQL index: `CREATE INDEX ... ON community.replies (path text_pattern_ops);`
   - Depth cap **6** is enforced at insert time: `array_length(string_to_array(path, '/'), 1) - 1 ≤ 6`. Replies attempted at depth > 6 are rewritten to attach to the depth-6 parent with a UX-rendered "in reply to" pointer (Reddit "continue this thread" pattern).
   - Render: subtree query is `WHERE path LIKE '/post_abc/%' ORDER BY path` — fully prefix-indexed; sibling order at each level is `(rank_score DESC, created_at ASC)` where `rank_score` is per ADR-COMM-0002.
   - Lazy-load: at render time, the first 100 nodes of a subtree are returned; deeper subtrees ship as collapsed pointers; the client expands on demand.

2. **`FlatAnswerRenderer` — Stack-Overflow-style flat answers + flat comments.**
   - Used for posts where `post.kind == 'question'`.
   - Data model: same materialised path, but the render is depth-flattened:
     - Depth 1 (answers to the question) — flat, ordered by ADR-COMM-0002 Wilson rank.
     - Depth 2 (comments on an answer) — flat, ordered by created_at.
     - Depth ≥ 3 — forbidden by Cedar policy `policy/qa-shape.cedar` (NEW; added by IP-005 follow-up). The Cedar policy emits `Deny` on insert with reason `qa_shape_disallows_nested_comments`.
   - The depth-cap is enforced at the kernel layer (Cedar is the belt-and-braces double-check at the policy layer).

3. **Shape selection** is per-post:
   - `post.kind = 'question'` → `FlatAnswerRenderer`.
   - `post.kind in ('announcement', 'discussion')` → `NestedThreadRenderer` (depth cap 6).
   - `post.kind = 'kb_article'` → no inline replies; comments are siblings, not nested (effectively depth 1 only).

4. **Materialised path is the canonical storage**:
   - Storage form: `text` column with `text_pattern_ops` btree index.
   - Path is regenerated on subtree move (rare; only happens during moderator-driven thread merge).
   - Path component is `<post_id>` for the root and `<reply_id>` for each interior node; reply_ids are ULIDs so the path is naturally time-prefix-sortable.

5. **Sibling order at each depth is `(rank_score DESC, created_at ASC)`** where `rank_score` is the ADR-COMM-0002 algorithm output. Order is a function of stored state; deterministic; idempotent render.

6. **Closed-set of supported shapes**. Tenants cannot configure arbitrary shapes; they pick a post.kind which transitively selects the renderer. The reasoning: tenant communities migrate between platforms (Slack → Discourse → Stack Overflow Teams → oyatie); offering each platform's shape per-tenant would explode the test matrix without adding meaningful tenant value.

## Alternatives Considered

### A. Unlimited-depth nested threading (Hacker News / Slashdot pre-2010)
- Pros: full conversation fidelity; no cap-related "where did the rest of the thread go?" UX surprise.
- Cons: horizontal-scroll UX cliff past depth ~10; mobile UX is unsalvageable past depth ~6; community norms drift toward parent-of-parent-of-parent narrow tangents that are unreadable; PRD performance budget at 1,000 nodes per thread suffers because depth-unbounded path strings get long.
- Rejected: UX cliff is the industry-known reason Reddit and Discourse picked depth cap.

### B. Flat threading only (Slack / Mastodon / Twitter mainline)
- Pros: trivial data model; trivial UX; cheap render.
- Cons: PRD FR-04 explicitly demands "conversation depth is preserved without flattening" — flat threading is the literal anti-goal; awful for technical discussion; deep back-and-forth becomes "@user said X" prefix soup.
- Rejected: violates FR-04.

### C. Adjacency-list (parent_id) data model with recursive CTE on subtree
- Pros: simplest insert; canonical relational data model.
- Cons: recursive CTE per subtree query at 1,000 nodes is borderline for p99 ≤ 350 ms; gets worse with PostgreSQL's planner sometimes choosing nested-loop joins; harder to index for ORDER BY across the subtree.
- Rejected: materialised path is the better PG pattern at our render-size + latency budget.

### D. Nested set model (`lft, rgt` integers)
- Pros: cheapest subtree query (range scan on integer column).
- Cons: every insert into a busy thread renumbers the rest of the tree — incompatible with high-write workloads; well-known scaling cliff in production literature.
- Rejected: insert cost is unacceptable at the per-post-thread write QPS we expect.

### E. Closure table (separate `(ancestor_id, descendant_id, depth)`)
- Pros: cheap subtree query; clean separation.
- Cons: insert at depth D writes D rows to the closure table; storage overhead; subtree move rewrites a quadratic-ish slice; more substrate to maintain.
- Rejected: storage overhead + insert overhead unattractive vs. materialised path.

### F. Hybrid (Discourse-style) — depth-1 only + linear "this is a reply" link otherwise
- Pros: avoids depth cap by collapsing to flat after depth 1.
- Cons: Discourse's UX is itself depth-limited (their actual implementation does cap at depth ~3 visually); the "infinite linear" pretense is illusory; tenants migrating from Reddit will find the model alien.
- Rejected: Reddit's depth-cap-6 model is the industry-attractor; Discourse's hybrid is more a UI convention than a data model.

## Consequences

### Positive

- Reddit-style depth-capped nested thread for discussions + announcements satisfies FR-04 without UX cliff.
- Stack-Overflow-style flat answers for questions satisfies FR-02 + best-answer-wins UX.
- Materialised path is the right PostgreSQL pattern at our latency + write QPS; PRD performance budget for 1,000-node render is achievable with prefix-index subtree query + lazy-load past 100 nodes.
- Sibling order is a deterministic function of stored state (ADR-COMM-0002 rank + created_at tie-break); render is idempotent + cache-friendly.
- Two shapes is the smallest closed set that covers the use case; tenants do not face arbitrary configuration choices.

### Negative

- Depth cap 6 (Reddit-style) requires UX for "continue this thread" past the cap; not free to implement. Mitigated by Reddit's published UX pattern being readily replicable.
- Q&A shape forbids nested-comments-on-comments; some tenants will request the back-and-forth; they can use a discussion post instead. Documented in tenant-facing help.
- Materialised path rewrites on subtree move (moderator-driven merge) are a non-trivial operation. Mitigated by being rare; runbook `runbooks/post-mass-deletion.md` (already exists) extended to cover thread merge.
- Render at exactly 1,000 nodes within p99 ≤ 350 ms is achievable but requires that the prefix index be hot in Postgres shared_buffers; pack-kr small instances at L tier may see p99 inflation. Mitigated by per-pack Postgres sizing tuned in IaC values.

### Operational

- Cargo workspace: `ThreadRenderer` trait + `NestedThreadRenderer` + `FlatAnswerRenderer` in `oya-community-thread-tree-domain`; materialised-path Postgres adapter in `-adapter-postgres`.
- Postgres schema: `community.replies.path text NOT NULL`; `CREATE INDEX replies_path_btree ON community.replies (path text_pattern_ops);`.
- Insert trigger: validate depth ≤ 6 for `NestedThreadRenderer` post.kinds; ≤ 2 for `FlatAnswerRenderer`. Trigger fires before insert; emits `qa_shape_disallows_nested_comments` or `nested_depth_cap_exceeded` to the audit-chain on violation (Cedar policy belt-and-braces).
- Dashboards: subtree-render-p99 panel added to a future `dashboards/feed-render.json`.
- CI lane `community-thread-depth-invariant`: property-based test that no inserted row has `array_length(string_to_array(path, '/'), 1) > 7` (path includes root post id + 6 reply ids).

### Regulatory

- **EU DSA Art. 14** (right of appeal): a depth-capped thread does not affect appeal flow; appeal happens at the reply-level, not the thread-shape level.
- **GDPR Art. 17 right to erasure**: tombstoning a reply leaves the path intact but rewrites the body to `[erased]`. Children of the tombstoned reply remain visible (industry pattern; tombstone-but-keep-structure). The audit-chain seal records the erasure.
- **Accessibility (WCAG 2.1 / 2.2)**: depth-capped nested threads with explicit "continue this thread" pointers are easier to navigate by screen reader than infinite-depth threads; WCAG SC 2.4.1 (Bypass Blocks) is materially supported by the cap.

## References

- Reddit threading cap discussion + UX rationale — `https://www.reddit.com/r/help/wiki/comments`
- Hacker News threading model (Paul Graham post) — `https://news.ycombinator.com/item?id=121903`
- Discourse threading model (community discussion) — `https://meta.discourse.org/`
- Stack Overflow answers + comments UX model — `https://stackoverflow.blog/`
- Materialised path pattern in PostgreSQL — `https://www.postgresql.org/docs/current/ltree.html` (and `text_pattern_ops` btree)
- Nested set model — Joe Celko, "Trees and Hierarchies in SQL for Smarties"
- Closure table pattern — Bill Karwin, "SQL Antipatterns"
- ADR-COMM-0002 — voting rank algorithm (sibling order at each depth)
- ADR-0126 — Connect-unbundle
- ADR-0131 — Per-microservice flat layout
- `microservices/community/PRD.md` FR-02, FR-04
- `microservices/community/IP-005-thread-tree-materialised-path.md`
- `microservices/community/catalog/oya-community-thread-tree-domain.yaml`
