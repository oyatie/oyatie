---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P09
title: Doc Auto-Generation + Freshness
status: complete
purpose: Generate docs from machine-readable truth (rustdoc, OpenAPI, ADR-INDEX, fitness reports); CI lane catches drift.
---

# M01-P09 — Doc Auto-Generation + Freshness

## Purpose
Per MASTERPLAN §2 Directive 10. Hand-written docs are reserved for narrative; everything machine-derivable is generated.

## Acceptance
- `oya-intelligence-mdbook-kernel` publishes rustdoc + OpenAPI Redoc + AsyncAPI + ADR-INDEX + glossary + COMPLIANCE-MATRIX as a single mdbook site.
- `oya-governance-doc-freshness` lane CI-blocks PRs that change source-of-truth without regenerating dependent docs.
- `oya-governance-doc-style` lane enforces canonical doc-style per `docs/standards/doc-style.md`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | mdbook publishing pipeline kernel + source walkers | complete | [`IP-001-mdbook-pipeline.md`](IP-001-mdbook-pipeline.md) |
| IP-002 | Doc-freshness CI lane | complete | [`IP-002-doc-freshness-lane.md`](IP-002-doc-freshness-lane.md) |
| IP-003 | Doc-style enforcement lane + auto-format | complete | [`IP-003-doc-style-lane.md`](IP-003-doc-style-lane.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-intelligence-mdbook-kernel`, `crates/oya-governance-{doc-freshness,doc-style}-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P09 complete: doc auto-generation + freshness lane green" -i critical -k "M-CC,P02,doc-automation,freshness,complete"
```
