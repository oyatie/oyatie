---
doc_class: Onboarding
microservice: notes
persona: notes-engineer + knowledge-graph-engineer
related_adrs: [ADR-0316, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Notes Engineer onboarding — first 5 working days

Audience: a new notes engineer or knowledge-graph engineer joining the `notes` rotation. By Day-5 they will have: opened + edited a 100k-block workspace, walked a Yjs CRDT conflict drill, debugged a bidirectional-link traversal issue, exercised the per-block ACL flow, and shadowed an AI T2 auto-apply review.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 + `decisions/ADR-NOTES-0001-yjs-crdt-selection.md` + `decisions/ADR-NOTES-0002-block-data-model.md` + `decisions/ADR-NOTES-0003-bidirectional-link-graph.md`.
2. Open the Grafana folder `notes`. Identify boards: `notes-page-open-latency`, `notes-block-edit-latency`, `notes-crdt-sync-lag`, `notes-search-query-latency`, `notes-vector-search-recall`, `notes-ai-t2-rate`, `notes-graph-traversal-depth`.
3. Walk the runbook index. On-call runbooks: `crdt-divergence.md`, `block-acl-violation.md`, `link-graph-traversal-timeout.md`, `search-index-drift.md`, `ai-t2-error.md`, `embedding-pipeline-stall.md`, `daily-notes-not-generating.md`.
4. Sit in on Friday's notes handoff.

Acceptance: you can sketch the request path: client → Cedar gate → block-store query → CRDT merge → search-index update → AI suggestion pipeline.

## Day 2 — Open + edit a 100k-block workspace

```sh
oya notes workspace open --tenant drill-acme --workspace research-notes-2026-q3 --measure-latency
```

The synthetic workspace contains ~ 100k blocks (research papers cited, daily notes for 90 days, ~ 50 categorised collections).

Time the cold-open via Grafana:

- Expected page-open p95 ≤ 280 ms (paid tenant_class).

Edit a block:

```sh
oya notes block edit \
    --workspace research-notes-2026-q3 \
    --page "Daily/2026-05-19" \
    --block-id blk-7f3a9b2c \
    --new-text "Following up on [[Paper-OOM-Killer-2024]] — review section 3.2"
```

Watch the substrate respond:

- Block-edit-to-persist: ~ 30 ms.
- Bidirectional-link parser detects `[[Paper-OOM-Killer-2024]]` + creates backlink on that page within ~ 100 ms.
- Search-index update fires within ~ 200 ms.
- AI T1 advisor may suggest related blocks within ~ 1.5 s (advisory).

Acceptance: page open + edit + link traversal + indexing observed end-to-end.

## Day 3 — Yjs CRDT conflict drill

Read `decisions/ADR-NOTES-0001-yjs-crdt-selection.md` + `runbooks/crdt-divergence.md`.

Provision a drill:

```sh
oya notes drill yjs-crdt-conflict \
    --tenant drill-acme \
    --workspace synthetic-collab \
    --collaborators drill-user-a,drill-user-b \
    --shape simultaneous-edit-same-block
```

The drill simulates two users editing the same block simultaneously:

- Both users insert text at the same character position.
- Yjs merges: each user's insertion preserved (text becomes both users' insertions interleaved).
- No data loss; no surprising delete.

Verify on the Loro/Yjs panel:

- `crdt-sync-lag` p99 ≤ 200 ms.
- `crdt-conflict-merge-count` ≥ 1 (a merge happened).

Now provoke a worse case:

```sh
oya notes drill yjs-divergence \
    --tenant drill-acme \
    --workspace synthetic-collab \
    --collaborators drill-user-a,drill-user-b
```

The divergence shape: each client gets a slightly different CRDT state. Yjs has a "rebase" mechanism; the runbook covers it.

Acceptance: you can articulate when Yjs merges silently vs surfaces divergence + you know the divergence-recovery runbook.

## Day 4 — Bidirectional-link traversal + per-block ACL

A user complains: "I clicked on `[[Project-X]]` but the linked page is empty."

```sh
oya notes link inspect \
    --tenant drill-acme \
    --link "[[Project-X]]" \
    --source-page "Daily/2026-05-19"
```

Expected output:

```
[backlink target] page-id: page-789xyz, title: "Project-X"
[forward link] [[Project-X]] in block blk-7f3a9b2c
[acl evaluation] target page ACL: PRIVATE; viewer has NO READ permission.
[result] viewer sees empty page (no leaked content); link warning shows.
```

The user is hitting per-block ACL working correctly. To grant:

```sh
oya notes block acl grant \
    --workspace research-notes-2026-q3 \
    --page page-789xyz \
    --grantee drill-user-questioner \
    --permission read \
    --signoff workspace-admin@drill-acme
```

Acceptance: you can diagnose ACL-based empty-page + grant.

## Day 5 — AI T2 auto-apply review shadow

Read `decisions/ADR-NOTES-0004-ai-t2-auto-apply.md` + `runbooks/ai-t2-error.md`.

Pull pending T2 reviews:

```sh
oya notes ai-t2 pending-reviews --tenant drill-acme --reviewer-role notes-engineer
```

For each pending action (T2 = AI auto-apply across multiple blocks):

1. Read the prose ("summarise all daily-notes from 2026-05 in a single page").
2. Read the AI's proposed change (the new summary block created + which blocks linked).
3. Check: does the summary respect data-class markers? Does the new block reference correct blocks? Does the summary contain any PII/PHI from source blocks that shouldn't be exposed?
4. Approve, modify, or reject.

The Cedar gate `notes::ai-t2::apply` evaluates the reviewer's signoff + the ChangeSet is committed.

Acceptance: T2 review walked; you can articulate why T2 needs Cedar + ChangeSet gating.

## What you've learned

- The block-store + CRDT collab substrate.
- The Yjs CRDT merge + divergence semantics.
- The bidirectional-link parser + graph traversal.
- The per-block ACL flow + Cedar enforcement.
- The AI T1 advisory vs T2 auto-apply gating.

Next week: vector-embedding pipeline shadow, large-workspace migration drill, daily-notes substrate review.
