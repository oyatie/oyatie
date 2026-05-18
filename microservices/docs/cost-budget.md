---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: axis-docs + finops
related_adrs: [ADR-0117, ADR-0131, ADR-DOCS-0001, ADR-DOCS-0003]
doc_status: published
---

# Cost Budget — docs µservice

## Purpose

Define per-tenant and per-cell unit economics for docs. FinOps gate: per-tenant monthly cost per active doc / edit / export must stay within budget; cross-budget breach holds promotion + auto-throttles.

## Unit economics (per tenant per month, baseline pricing OCI ap-seoul-1, 2026-05)

| Unit | Baseline cost | Notes |
|---|---|---|
| Active document (1 user, 30d active) | $0.12 / mo | Postgres row + content blob + index footprint |
| Edit-op (1 CRDT op) | $0.000005 / op | Loro op + Valkey spool + audit emit |
| Comment | $0.0001 / comment | Postgres + audit emit |
| Suggestion | $0.0002 / suggestion | Postgres + audit emit + state-machine |
| Version snapshot | $0.0008 / snapshot | Postgres + S3 blob |
| Share-link issuance | $0.00005 / issuance | OpenBao + audit emit |
| Export PDF (50-page doc) | $0.018 / job | gVisor sandbox + WeasyPrint CPU |
| Export DOCX | $0.010 / job | Pandoc CPU |
| Import DOCX (50-page) | $0.014 / job | Pandoc + sanitisation |
| Attachment upload (10MB) | $0.003 / upload | S3 PUT + ClamAV scan + image re-encode |
| Embed-refresh | $0.00005 / refresh | cross-µservice mTLS + Valkey cache |
| AI writing-assist (T1 suggestion) | $0.012 / suggestion | foundry-runtime LLM inference |
| Auto-summary (T2) | $0.025 / summary | LLM + tenant-DEK envelope |
| Per-block ACL check | $0.000002 / check | Cedar policy eval cached |

## Per-cell cost envelope (steady-state, p50 load)

| Component | Baseline / cell / mo | Notes |
|---|---|---|
| Postgres (document-metadata; 3-replica HA + per-tenant RLS) | $5,800 | OCI VM.Standard3.Flex 16 OCPU × 3 + 4TB persistent block + WAL retention |
| S3 (content blobs + attachments; per-pack) | $1,800 | OCI Object Storage standard tier; 50TB baseline |
| Valkey (collab presence + CRDT spool + cache; cluster mode 5-shard) | $1,400 | OCI Caching Service |
| Kubernetes nodes (rest + worker + gVisor pool) | $4,200 | OCI VM.Standard.E5.Flex × 12 baseline |
| gVisor pool for export workers (pre-warmed 10 sandboxes) | $600 | E5.Flex × 2 dedicated |
| ClamAV scanner | $80 | OCI VM.Standard.E5.Flex × 1 |
| OPSWAT MetaDefender (pack-us-healthcare only) | $300 | licensed; pack-us-healthcare overlay |
| Egress (cross-µservice embed + share-via-mail + audit) | $500 | intra-region mesh |
| Cross-pack mesh egress (SCC-gated cross-pack embed) | $30 | per-pack pair |
| Observability fan-out (metrics + logs + traces) | $400 | per observability µservice envelope |
| Backup storage (Postgres + S3 snapshots; cold-tier WORM) | $350 | retention 12mo cold; pack-us-healthcare 6y |
| LLM inference budget (foundry-runtime; tenant-default) | $1,200 | T1+T2 capability variable; this is steady-state |
| **Total per cell baseline** | **$16,660 / mo** | sized for 1M active docs |

## Per-tenant breakeven

| Tenant tier | Monthly bill | Notes |
|---|---|---|
| Free | $0; 5 active docs, 100 edits, 0 AI-assist | hard quota, cost-controlled |
| Starter | $8; 50 active docs, 5k edits, 100 AI-assist | covers infra + 30% margin |
| Pro | $30; 500 active docs, 100k edits, 1k AI-assist | covers infra + 40% margin |
| Enterprise | custom + per-seat | covers infra + 50% margin + SLA premium |

Breakeven: per-tenant gross margin ≥ 30% at Starter; ≥ 40% at Pro; ≥ 50% at Enterprise.

## Cost governance gates

| Gate | Threshold | Action |
|---|---|---|
| Per-tenant monthly cost > 80% of tier bill | warn FinOps | review for plan upgrade |
| Per-tenant monthly cost > 100% of tier bill | warn FinOps + tenant | over-quota notification + offer upgrade |
| Per-tenant monthly cost > 150% of tier bill | throttle (rate-limit lowered to baseline-tier limits) | auto-throttle |
| Per-cell total cost > 130% of baseline | hold promotion + finops review | review |
| Cross-pack egress > 5% of cell cost | finops review | identify cause |
| Export worker cost > 20% of cell cost | engineering review | optimisation needed |
| LLM inference cost > 25% of cell cost | engineering review | model swap / caching |

## Cost-meter implementation

Cost-meter emitted as Mimir metric per ADR-0123 + finops standards:

| Metric | Cardinality | Labels |
|---|---|---|
| `docs_tenant_cost_dollars_per_month` | per-tenant | `tenant_id_hashed`, `tier`, `pack_tag` |
| `docs_unit_cost_dollars` | per-unit | `unit_type` (doc/edit/comment/suggestion/version/share/export-pdf/export-docx/import-docx/attachment/embed-refresh/ai-assist/auto-summary/acl-check), `pack_tag` |
| `docs_cell_cost_dollars_per_month` | per-cell | `pack_tag`, `cell_id` |
| `docs_llm_inference_cost_dollars` | per-capability | `capability_id` (T1-cal-grammar-check / T2-cal-auto-summary / etc.), `pack_tag` |
| `docs_egress_dollars` | per-cross-pack-pair | `from_pack`, `to_pack` |

## Optimisations identified at design time

| Optimisation | Estimated savings | Status |
|---|---|---|
| Loro snapshot compaction (version-aligned) | 60% on CRDT op-log storage | Implemented in PHASE-01 CS-05 |
| Per-doc cache TTL ≤ 5min + single-flight | 70% on doc-read cost for warm docs | Implemented in PHASE-01 CS-03 |
| Postgres per-tenant partition pruning | 40% on read latency + 25% on storage | Implemented in PHASE-01 CS-02 |
| .docx streaming parse (not full materialisation) | 80% on memory; required for 50-page docs | Implemented in PHASE-01 CS-10 |
| Pre-warm gVisor pool (10 standby) | sub-1s cold-start vs. 5s on-demand | Implemented in PHASE-01 CS-10 |
| Embed-resolver per-(source,ref) coalescing | 90% on embed-refresh cost | Implemented in PHASE-01 CS-12 |
| Per-block ACL projection cache | 50% on ACL-check cost | Implemented in PHASE-01 CS-09 |
| LLM prompt-template cache (tenant-DEK-wrapped) | 30% on AI-assist cost | Implemented in PHASE-01 CS-15 (foundry-runtime integration) |

## FinOps reporting cadence

- Per-tenant cost report: monthly.
- Per-cell cost report: monthly.
- Cross-pack egress report: weekly.
- Cost-anomaly alert: daily (≥ 30% deviation from rolling 7d baseline).
- LLM-inference cost report: weekly (per ADR-DOCS-0005 ongoing monitoring).

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- ADR-DOCS-0001 (Loro CRDT); ADR-DOCS-0003 (export backend).
- `capacity-model.md`, `multi-region.md`.
- OCI ap-seoul-1 pricing (2026-05).
