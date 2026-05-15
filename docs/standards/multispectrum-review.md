---
purpose: "Thin human gateway for the multispectrum review bar. The canonical spec is `/specs/cross-cutting/multispectrum-review.json` v2.1.0. The doctrine principles are `/specs/cross-cutting/oyatie-doctrine.json` P0..P9."
---

---
doc_class: Standard
status: Accepted
date: 2026-05-14
authority_tier: 2
shape: thin-pointer-gateway
purpose: |
  Thin human gateway for the multispectrum review bar. The canonical
  spec is `/specs/cross-cutting/multispectrum-review.json` v2.1.0. The doctrine
  principles are `/specs/cross-cutting/oyatie-doctrine.json` P0..P9. This markdown
  does NOT restate spec content (per P9 no-sprawl).
canonical_authority: /specs/cross-cutting/multispectrum-review.json
related_specs:
  - /specs/cross-cutting/oyatie-doctrine.json (canonical P0..P9 principles)
  - /specs/cross-cutting/iterative-fix-loop.json (loop state machine)
  - /templates/checklists/pre-pr-multispectrum.json (evidence template)
  - /registries/cross-cutting/fixuptasks.jsonl (FixupTask registry)
  - /evidence/audit-chain.jsonl (audit-chain stream)
related_adrs:
  - ADR-0054 (grit protocol)
  - ADR-0056 (12-layer enum)
  - ADR-0062 (Quality/Performance/Scalability bar)
  - ADR-0069 (active-artifact-contract)
  - ADR-0092 (workspace dependency-seam policy — first ADR citing this standard)
---

# Multispectrum Review — Human Gateway

> **Canonical spec:** [`/specs/cross-cutting/multispectrum-review.json`](../..//specs/cross-cutting/multispectrum-review.json) v2.1.0.
> Read THAT for the schema, enums, rigor matrix, and evidence contract. This markdown does not restate.

## What it is

A multi-facet review bar applied to every changeset. The lane `oya-check-dependency-seam` mechanically refuses promotion when required evidence is absent / malformed / lacks a canonical change_class.

## How to use it

1. Declare your change_class (CC-1..CC-7).
2. Fill the evidence template `/templates/checklists/pre-pr-multispectrum.json`.
3. Write to `/evidence/multispectrum/<change_id>-<unix_ts>.json`.
4. Run `cargo run -p oya-check-dependency-seam --quiet`.
5. Lane must exit 0 before `grit done`.

## Where each concept lives (canonical homes)

| Concept | Canonical home |
|---|---|
| Principles (P0..P9) | `/specs/cross-cutting/oyatie-doctrine.json#principles` |
| Facets (F1..F13, M1/M2) + change_classes + evidence_schema + scorecard + consensus_debate | `/specs/cross-cutting/multispectrum-review.json` |
| Loop state machine | `/specs/cross-cutting/iterative-fix-loop.json` |
| Pre-PR template | `/templates/checklists/pre-pr-multispectrum.json` |
| FixupTask registry | `/registries/cross-cutting/fixuptasks.jsonl` |
| Audit-chain stream | `/evidence/audit-chain.jsonl` |
| Per-PR evidence files | `/evidence/multispectrum/<change_id>-<unix_ts>.json` |
| 12-layer enum + BNF | `docs/decisions/ADR-0056` |
| Active-artifact-contract | `/specs/cross-cutting/active-machine-readable-artifact-contract.json` |

Per P9 no-sprawl: no facet table, no principle list, no rigor matrix is duplicated in this markdown.

## Anti-patterns explicitly forbidden

Listed in `/specs/cross-cutting/multispectrum-review.json#anti_patterns_explicitly_forbidden` (per facet). Read there.

## Decision-log row (Linus good-taste)

This markdown was originally authored as a parallel content gateway with facet tables, protocol summaries, and enforcement narrative. Per user directive 2026-05-14 ('prevent documentation sprawls') it was slimmed to thin-pointer-gateway shape. P9 no-sprawl is now load-bearing.
