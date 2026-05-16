---
doc_class: ArchiveIndex
purpose: Pre-grit-cutover archive directory for legacy ultragoal agent artifacts (2026-05-12). All artifacts here are retired; canonical agentic-pipeline state lives under sanctioned primitives (grit + icm + oya-tooling-agent-read).
parent: ../../../README.md
---

# pre-grit-cutover-2026-05-12 — archive

This directory archives agent-pipeline artifacts that predate the grit
cutover described in ADR-0052. Contents here are read-only; no agent
authors new files in this tree.

## What's archived
- Legacy monolithic checklists.
- Pre-cutover bash entry points.
- Snapshot of agent-instruction surfaces before the banned-primitives lane was enforced.

## Why archived (not deleted)
Per `markdown-retirement-policy.json`, retired artifacts are first moved
to an archive directory so the markdown-retirement-ledger can record
their lifecycle event, then deleted in a subsequent ChangeSet once the
ledger entry is fresh.

## Glue to M-CC-P01-IP-008
This directory closes the "archive glue" IP — `archive-orphan` lane
checks that no live agent path references this archive.
