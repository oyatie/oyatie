---
doc_class: Tutorial
microservice: notes
persona: research-engineer + knowledge-worker
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a research notebook with daily-notes, bidirectional links, and AI summarisation

You will: create a research workspace, build daily-notes habit over a sample week, author research-paper review pages with bidirectional links, generate an AI summary, and export to PDF. Total time ≤ 90 minutes active (the daily-notes accrue over a week in real use).

## Pre-requisites

- A paid tenant_class notes cell.
- Tenant `drill-acme` provisioned.
- A research engineer principal.

## Step 1 — Create the research workspace (≤ 5 min)

```sh
oya notes workspace create \
    --tenant drill-acme \
    --name research-notes-2026-q3 \
    --owner drill-research-engineer \
    --template research \
    --daily-notes-enabled true \
    --daily-notes-tz America/New_York \
    --ai-tier T1 \
    --pack-overlay public
```

The `research` template seeds:

- `Inbox` (capture page).
- `Daily` (sub-pages for each day).
- `Papers` (research paper reviews).
- `Projects` (ongoing projects).
- `People` (collaborator pages).
- `Topics` (concept pages).
- `Index` (auto-generated table of contents).

## Step 2 — First daily-notes entry (≤ 10 min)

```sh
oya notes daily-notes open --workspace research-notes-2026-q3
```

The page `Daily/2026-05-20` is auto-created. Add some content:

```sh
oya notes block add \
    --workspace research-notes-2026-q3 \
    --page "Daily/2026-05-20" \
    --type heading-2 \
    --text "Morning standup"

oya notes block add \
    --workspace research-notes-2026-q3 \
    --page "Daily/2026-05-20" \
    --type list-bullet \
    --text "Following up on [[Paper-OOM-Killer-2024]] — review section 3.2"

oya notes block add \
    --workspace research-notes-2026-q3 \
    --page "Daily/2026-05-20" \
    --type list-bullet \
    --text "Meeting with [[Brenda]] re [[Project-Stability]] at 2pm"

oya notes block add \
    --workspace research-notes-2026-q3 \
    --page "Daily/2026-05-20" \
    --type task \
    --text "Draft experiment plan for [[Heap-Exhaustion-Investigation]]" \
    --due 2026-05-22
```

The block parser detects `[[bidirectional links]]` + creates the linked pages if they don't exist:

- `Paper-OOM-Killer-2024` (sub-page of Papers).
- `Brenda` (sub-page of People).
- `Project-Stability` (sub-page of Projects).
- `Heap-Exhaustion-Investigation` (sub-page of Projects).

## Step 3 — Author paper review with citation (≤ 15 min)

```sh
oya notes page edit \
    --workspace research-notes-2026-q3 \
    --page "Papers/Paper-OOM-Killer-2024"
```

(Opens an interactive editor; or set blocks programmatically.)

Add:

```
# Paper: OOM Killer — A Survey of Linux Memory Management
Author: Smith et al., 2024
Citation: arxiv:2401.12345

## Summary
- The paper surveys 5 OOM-killer policies in Linux 6.x.
- Argues that LRU-based reclaim still wins over zram on 90% workloads.
- Section 3.2 details the slab-account drift problem.

## Relevant to
- [[Project-Stability]] — we should consider zram for heap-allocator
- [[Heap-Exhaustion-Investigation]] — section 3.2 applies directly

## Citations
- [[Paper-LRU-Reclaim-2022]]
- [[Paper-Zram-Performance-2023]]

## Questions to follow up
- Does the slab-account drift cause performance penalties under heavy IO? Section 3.2.4.
```

Each `[[link]]` automatically creates the backlink. Visit `Project-Stability`:

```sh
oya notes page open \
    --workspace research-notes-2026-q3 \
    --page "Projects/Project-Stability"
```

You'll see a "Linked From" section listing the daily-note + paper-review pages.

## Step 4 — AI T1 suggestion shadow (≤ 5 min)

After authoring, request an AI summary:

```sh
oya notes ai-t1 suggest \
    --workspace research-notes-2026-q3 \
    --page "Papers/Paper-OOM-Killer-2024" \
    --action "extract-key-takeaways"
```

The AI T1 returns a list:

```
1. Section 3.2 reports slab-account drift as a known issue.
2. LRU reclaim remains preferred for 90% of workloads.
3. Future work suggests workload-aware reclaim selection.
4. Relevant for memory-intensive systems (e.g., your Project-Stability).
```

You accept (creates a new block under the page) or modify or reject.

## Step 5 — Build a topic page from multiple sources (≤ 15 min)

After authoring several paper reviews + daily-notes, create a topic page:

```sh
oya notes page create \
    --workspace research-notes-2026-q3 \
    --page "Topics/Memory-Management" \
    --type topic
```

Add a synthesis:

```
# Memory Management — Concept Page

## Key insights from research
- [[Paper-OOM-Killer-2024]] — LRU > zram for 90% workloads
- [[Paper-LRU-Reclaim-2022]] — original LRU work
- [[Paper-Zram-Performance-2023]] — zram benchmarks

## My projects touching this
- [[Project-Stability]]
- [[Heap-Exhaustion-Investigation]]

## Questions
- Should we instrument slab-account drift?
- What's the cost of workload-aware reclaim?
```

The graph view (paid tenant_class) shows the connection clusters around `Memory-Management`.

## Step 6 — Per-block ACL example (≤ 5 min)

Some block content is sensitive. Mark a block PHI / sensitive:

```sh
oya notes block acl set \
    --workspace research-notes-2026-q3 \
    --page "Papers/Paper-OOM-Killer-2024" \
    --block-id blk-789xyz \
    --access "drill-research-engineer:read-edit" \
    --justification "patient-data-in-this-paper-citation"
```

Now other collaborators don't see this block when they view the page (even if they have page-read access).

## Step 7 — Export to PDF (≤ 5 min)

```sh
oya notes export pdf \
    --workspace research-notes-2026-q3 \
    --pages "Papers/Paper-OOM-Killer-2024,Topics/Memory-Management,Daily/2026-05-20" \
    --output ./memory-management-research.pdf \
    --include-graph false
```

The PDF preserves:

- Block types + formatting.
- Bidirectional links (as page-references with hyperlinks).
- Metadata (created/modified dates).
- ACL flags (sensitive blocks marked or excluded).

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --service notes --since 90m
```

Expected events:

- `workspace_created`
- `page_created` × N
- `daily_notes_generated` × 1
- `block_added` × M
- `bidirectional_link_created` × P
- `page_acl_set` × 1
- `block_acl_set` × 1
- `ai_t1_suggestion_requested` × 1
- `ai_t1_suggestion_accepted` × 1
- `pdf_export` × 1

## What you've learned

- The block + page + workspace model.
- The daily-notes pattern + capture inbox.
- The bidirectional link substrate + backlink generation.
- The topic-page synthesis pattern.
- The AI T1 suggestion workflow.
- The per-block ACL substrate.
- The export-to-PDF flow.
- The audit-chain shape for notes operations.

Next tutorial: `tutorials/build-kanban-board-with-block-types.md` — kanban / gallery / calendar / timeline views on the same block-store.
