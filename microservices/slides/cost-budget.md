---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + fin-eng
doc_status: published
---

# Cost budget — slides µservice

Per-pack monthly steady-state cost envelope at the M03 preview target (10k active editor sessions per region; 500 baseline / 5000 max broadcast viewers per deck).

## Per-cell baseline (USD / month)

| Component | Driver | Baseline cost | Source |
|---|---|---|---|
| editor-rest (4 replicas × 2 vCPU × 4 GiB) | OCI VM.Standard.E5.Flex 2-OCPU | $620 | OCI list |
| collab-crdt-worker (3 replicas × 2 vCPU × 8 GiB) | OCI VM.Standard.E5.Flex | $660 | OCI list |
| broadcast-mode-worker (2 replicas × 2 vCPU × 4 GiB) | OCI VM.Standard.E5.Flex | $310 | OCI list |
| Export pool — gVisor workers (4 replicas × 4 vCPU × 8 GiB) | OCI VM.Standard.E5.Flex 4-OCPU | $1,240 | OCI list |
| Postgres (Citus 3-node × 8 vCPU × 32 GiB + 1 TB SSD) | OCI VM.DenseIO.E5 | $3,400 | OCI list + storage |
| Redis (3-node sentinel cluster × 4 vCPU × 16 GiB) | OCI VM.Standard.E5.Flex | $920 | OCI list |
| S3 (deck snapshots + assets; 5 TB; 1M PUT + 5M GET / month) | OCI Object Storage standard | $130 | OCI Object pricing |
| CDN (WASM + theme/template gallery; 2 TB egress / month) | OCI CDN egress | $200 | OCI CDN |
| WAF + API gateway | OCI WAF | $80 | OCI list |
| Cross-µservice SDK egress (chart-live-link to sheets, embed-bridge, broadcast LiveKit reuse) | intra-region | $50 | network |
| ImageMagick + ClamAV + OPSWAT sidecar (2 replicas) | OCI VM.Standard.E5.Flex 1-OCPU | $160 | OCI list |
| ffmpeg transcode pool (3 replicas × 4 vCPU × 8 GiB; gVisor) | OCI VM.Standard.E5.Flex 4-OCPU | $930 | OCI list |
| Pandoc + WeasyPrint pool (2 replicas × 2 vCPU × 4 GiB; gVisor) | OCI VM.Standard.E5.Flex | $310 | OCI list |
| Observability + audit-chain egress (cross-µservice) | metrics + traces + audit seals | $90 | observability tier |
| Encryption (per-pack KMS) | KMS request volume | $60 | OCI KMS |
| OpenBao | shared | $30 | shared substrate cost share |
| **Baseline subtotal** | | **$9,190** | |

### AI capability costs (T1/T2)

| Capability | Driver | Baseline | Burst |
|---|---|---|---|
| T1 alt-text suggest | 100k images / month × foundry-runtime cost | $400 | $1,000 |
| T1 design-assist | 50k invocations / month | $600 | $1,800 |
| T1 layout-suggest | 50k invocations / month | $400 | $1,200 |
| T1 copy-refine | 30k invocations / month | $300 | $900 |
| T1 slide-summary | 10k invocations / month | $200 | $600 |
| T2 full-deck-from-prompt | 2k invocations / month | $1,800 | $5,400 |
| T2 auto-translate (per slide × language) | 5k slide-translations / month | $300 | $1,200 |
| T2 theme-cascade | 3k invocations / month | $400 | $1,200 |
| **AI subtotal** | | **$4,400** | **$13,300** |

### Pack uplift (incremental per pack beyond first)

| Pack | Uplift | Rationale |
|---|---|---|
| kr | +0% | baseline pack |
| eu | +15% | residency duplication + GDPR audit retention 7y |
| us | +10% | residency duplication |
| us-healthcare | +35% | HIPAA + 6y retention + BAA controls + PHI redaction infra |
| jp | +10% | APPI compliance + JP region |
| sg | +10% | SG region |
| au | +10% | AU region |
| in | +12% | DPDPA + IN region |
| br | +12% | LGPD + BR region |
| ae | +12% | UAE PDPL + AE region |
| ksa | +12% | KSA PDPL + KSA region |

## Per-tenant marginal cost (estimate)

| Tier | Activity | Marginal cost / tenant / month |
|---|---|---|
| Free / sandbox | up to 10 decks, 5 viewers | $1.50 |
| Pro | up to 100 decks, 100 viewers, T1 AI included | $18 |
| Team | up to 1,000 decks, 500 viewers per deck, T1 + T2 included | $120 |
| Enterprise | unlimited; broadcast 5000 viewers; per-pack residency; HIPAA | $1,200 + per-seat |

## Total cluster envelope at XL (200k editor sessions per region)

- Baseline scaling: ×8 (linear in editor sessions until Postgres shard limit) → **$73,520 / month / region** at XL with AI baseline included.
- Burst envelope: AI burst + export pipeline burst + broadcast cascade → **+$20,000 / month** peak.

## Cost controls

- Per-tenant AI quota (T1: 10k/month / pack default; T2: 100/month / pack default; pack-tunable).
- Per-tenant export quota (PPTX/PDF/MP4) — burst-tunable.
- Broadcast-mode SFU cascade triggers at >500 viewers; cost passed-through line item.
- Cold deck eviction — S3 transition to OCI Archive after 90d no-access (per-pack tunable; us-healthcare 6y minimum).
- CDN cache TTL tuned per asset class; immutable WASM chunks → max TTL; theme/template gallery → 1h with revocation propagation; deck-rendered preview → 60s with per-tenant key.
- gVisor worker pool right-sized per-pack daily; weekend scale-down except us-healthcare clinical pack.

## Cost SLO

- Per-tenant cost variance ≤ 10% week-over-week (alarm on >15% burst not explained by tenant action).
- AI cost / total cost ratio ≤ 40% steady-state; ≤ 60% burst.

## References

- OCI pricing (public list, 2026-Q1).
- foundry-runtime per-capability unit cost (per foundry-runtime cost-budget.md).
- messenger LiveKit cost (shared; reused via broadcast-mode).
- ADR-0130 SLO-gated promotion.
