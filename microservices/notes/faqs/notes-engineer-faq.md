---
doc_class: FAQ
microservice: notes
persona: notes-engineer + knowledge-graph-engineer
date: 2026-05-20
doc_status: published
---

# Notes Engineer FAQ

## Why Yjs and not Loro or Automerge?

Per ADR-NOTES-0001. For notes-style outliner content:

- Yjs has the largest ecosystem for prosemirror / block-based editors.
- Yjs's Y.Map + Y.XmlFragment map cleanly to block trees.
- Yjs has been battle-tested in Notion-class products + has multi-language client support.

We use Loro for sheets (ADR-SHEETS-0001) for its compact-storage on 1M-cell workbooks. Notes' block-tree shape is different + Yjs handles it well.

## Why bidirectional links + graph view?

Per ADR-NOTES-0003. Bidirectional links are how Roam Research + Obsidian distinguish from outline-only notes. The link graph encodes the knowledge structure:

- Search by reference (find all blocks that mention `[[Customer-X]]`).
- Discover-from-graph (suggest related pages).
- Visual graph view (see clusters).
- Knowledge-completion (AI suggests "you cited Paper-OOM-Killer; also cite Paper-Heap-Exhaustion?").

A unidirectional system (demo_trial tenant_class) misses the backlink discovery; the user manually finds references.

## What's the difference between block-edit and page-edit?

Per ADR-NOTES-0002. The block is the atom:

- Block: paragraph, list-item, heading, code-block, etc.
- Page: ordered list of blocks (root or sub-page).
- Each block has its own ID + ACL + version.
- Each page has its own ACL (inherited by blocks unless overridden).

Edit semantics:

- Block-edit: change block content; preserves block ID + link references.
- Page-edit: reorder blocks; add/remove blocks at page level.
- Both are CRDT-merged.

## When should I use a sub-page vs a section heading within a page?

Heuristic:

- Sub-page: when content is large + meant to be referenced from elsewhere (gets its own URL + bidirectional links + permalink).
- Section heading: when content is small + always part of the parent page (sub-page noise is excess).

For research notes: top-level pages for major topics; sub-pages for papers / experiments; sections for daily-notes sub-categories.

## Why does my AI T1 suggestion take 3-5 seconds?

T1 AI calls Whisper-class models on tenant-controlled hardware (per the paid/paid capacity envelope). Latency is bounded by:

- Model inference: 1-3 s (depending on model + batch size).
- Tokenisation: 50-100 ms.
- Network round-trip: 50-100 ms.
- Cedar gate eval: ~ 3 ms.

T1 is ADVISORY; user accepts each suggestion. So the 3-5 s latency is acceptable for the suggestion to appear (the user reads + reacts).

For T2 (auto-apply), the latency is the same but the change is automatic; we therefore use Cedar + ChangeSet review to gate.

## What's the search query path?

Per IP-008 + IP-009. A search query:

1. Lex parse the query (e.g., `customer-x:status` + filters + text).
2. Cedar permissions filter: which pages can this user search?
3. Tantivy text search on permitted pages.
4. Meilisearch fuzzy search on permitted pages (handles typos).
5. Qdrant vector search on permitted blocks (semantic similarity at paid).
6. Merge results; rerank.

Latency budget: ≤ 180 ms p99 at paid, ≤ 120 ms at paid (with vector).

## Why does daily-notes not generate today?

Per IP-006. Daily-notes is a calendar-based template:

- Each day at midnight (tenant TZ), the scheduler creates a `Daily/YYYY-MM-DD` page from the template.
- The page is empty (or has the template's default blocks).
- If the scheduler stalls, no daily-notes are created.

Check:

```sh
oya notes scheduler status --tenant drill-acme --since 24h
```

Expected: a `daily-notes-generated` event for today (within 1 hour of midnight tenant-TZ). If missing: investigate scheduler.

## How does the capture inbox work?

Per IP-007. Capture inbox is a per-user destination for ad-hoc input:

- User taps "capture" → text input → block added to inbox.
- Captures stay in inbox until user moves to a page.
- AI T1 categorises: suggests where each capture might go (project page, daily-notes, todo list).
- User accepts or moves manually.

Use case: ideation, quick-notes, "remember to call X".

## Block-level ACL — when do I use it instead of page-level?

Heuristic:

- Page-level ACL: whole page is shared with team / role.
- Block-level ACL: specific block (e.g., budget table, salary list) has narrower audience.

The cost: per-block ACL eval adds latency (~ 5-10 ms per block) at large scale. Use sparingly; most pages should have page-level ACL.

## What's the embedding pipeline at paid tenant_class?

Per IP-014. Every block:

1. On block-write, queue for embedding.
2. Embedding worker pulls block text + creates 1536-dim vector (text-embedding-ada-002 or local replacement).
3. Vector stored in Qdrant per-tenant collection.
4. Vector indexed for ANN search (Approximate Nearest Neighbour).
5. Block edit triggers re-embed.

Cost: ~ 1 ms per block for embed + storage. Storage: ~ 6 KB per block (1536-dim float32 + metadata).

For 10M blocks per workspace: ~ 60 GB storage per workspace; vector search ~ 50 ms p99.

## How does cross-workspace search work?

Per IP-015. Cross-workspace search at paid:

1. User queries: "find anything about Project-X".
2. Cedar filter: which workspaces does this user have access to?
3. Search each permitted workspace's index.
4. Merge + rank results.

A user can only search workspaces they have access to. The cost: ~ 1 RPC per workspace; we parallelise.

Limit: cross-workspace search bounded to 50 workspaces per query at paid; can be increased per-tenant.

## A user says "I deleted a block but it came back." What happened?

Possible causes:

1. CRDT undo from another collaborator. Yjs merges your delete + the other user's edit; both win. Result: the block content was restored.
2. Block was referenced from elsewhere; the "delete" only removed the inline mention; the referenced block still exists.
3. Sync lag: the delete propagated to other clients; one client had a slow sync; eventually consistent.

Runbook `runbooks/block-undelete-confusion.md` covers diagnostics.
