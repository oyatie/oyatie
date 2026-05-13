---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P05
title: Search Preview (pgroonga + Morphology + pgvector + RAG)
status: stub
purpose: Stand up search foundations — pgroonga day-1, KR/JP/EN morphology, vector index, tenant-private indexes, RAG endpoint to Foundry.
---

# M03-P05 — Search Preview

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.5. Search axis foundations.

## Acceptance
- pgroonga inverted index sharding operational.
- KR morphology (mecab-ko / khaiii); JP morphology; EN tokenizer.
- pgvector vector index; per-tenant-private indexes.
- RAG endpoint exposed to Foundry capabilities.
- Per-class data boundary enforcement (Data Use Boundary ADR-0008).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | pgroonga inverted index + KR/JP/EN morphology | stub | [`IP-001-pgroonga-morphology.md`](IP-001-pgroonga-morphology.md) |
| IP-002 | pgvector vector index + tenant-private indexes | stub | [`IP-002-pgvector-tenant-private.md`](IP-002-pgvector-tenant-private.md) |
| IP-003 | RAG endpoint to Foundry + data boundary enforcement | stub | [`IP-003-rag-endpoint-data-boundary.md`](IP-003-rag-endpoint-data-boundary.md) |

## Estimated parallelism
3 agents in parallel.

## Symbols-touched
`crates/oya-search-{crawler,parser,index-inverted,index-vector,rank,query,serp,rag}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P05 complete: search preview pgroonga + pgvector + morphology + RAG to Foundry" -i critical -k "M03,P05,search,complete"
```
