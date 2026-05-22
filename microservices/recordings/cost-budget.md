---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-recordings
related_adrs: [ADR-RECORDINGS-0001, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005]
doc_status: published
---

# Cost Budget: recordings µservice

## Monthly Cost Envelope (per 1M hours ingested, US-East baseline)

| Component | Monthly cost |
|---|---|
| Whisper-large transcription (foundry-runtime GPU) | $300,000 |
| pyannote 3.x diarization | $150,000 |
| ffmpeg 7.x transcode (HLS multi-bitrate ladder) | $50,000 |
| S3 hot storage (avg 50 PB) | $115,000/mo |
| S3 cold storage (avg 500 PB, after 90d age-down per pack) | $200,000/mo |
| Postgres 16 (managed; 50 TB) | $25,000 |
| Valkey 8.1 (RESP3 wire-compatible) (256 GB) | $5,000 |
| Meilisearch 0.10.0 (5 TB) | $10,000 |
| CDN CloudFront egress (assume avg 100 Gbps sustained) | $250,000 |
| Pandoc 3.x (transcript-to-PDF/DOCX; CPU only) | $1,000 |
| OPSWAT MetaDefender / ClamAV (upload scan) | $20,000 |
| Cedar v4.2 evaluator (CPU only) | $5,000 |
| Audit-chain seals (cross-µservice; recordings's allocated share) | $10,000 |
| Total | ≈ $1.14 M/month at 1M hours |

## Unit Economics

| Metric | Cost per unit |
|---|---|
| Per recording-hour ingested | $1.14 |
| Per recording-hour played (warm CDN) | $0.25 |
| Per export (MP4, 1h source) | $2.00 |
| Per export (transcript-PDF, 1h source) | $0.05 |
| Per eDiscovery bundle (1k hours scope) | $50 |
| Per legal-hold/month | $0.10 |

## Budget Allocation Per Pack

| Pack | Allocation % (of total recordings spend) | Notes |
|---|---|---|
| pack-us | 35 % | dominant tenant base |
| pack-eu | 25 % | second largest |
| pack-us-healthcare | 8 % | high per-recording cost (PHI redaction + BAA infra) |
| pack-us-financial | 7 % | WORM retention drives cold storage |
| pack-kr | 10 % | self-host CDN reduces egress cost; PIPA infra adds compliance cost |
| pack-jp + pack-sg + pack-au | 8 % | combined APAC |
| pack-in + pack-br + pack-ae + pack-ksa | 7 % | combined emerging packs |

## Cost-Saving Levers

1. **Age-down hot → cold S3 per pack default** (saves ≈ 80 % storage cost
   on the 90d+ tail). See ADR-RECORDINGS-0005.
2. **Whisper-medium fallback under queue pressure** (saves ≈ 40 % GPU cost
   in burst windows). See ADR-RECORDINGS-0001 + degraded runbook.
3. **Per-tenant transcription opt-out** (saves Whisper + pyannote cost
   when tenant doesn't need transcript).
4. **CDN cache tuning** (lift cache hit > 90 % to reduce origin egress).
5. **Self-host CDN for pack-cn / pack-ksa** (avoids CloudFront where
   residency forbids; saves egress but trades for ops cost).
6. **Re-encode at lower max-bitrate for pack-default tenants** (saves
   storage + egress for non-4k content).

## Budget Alerts

| Trigger | Action |
|---|---|
| Pack monthly spend > 110 % of allocated | page ops-finops + axis-recordings |
| Pack monthly spend > 130 % of allocated | engage tenant for capacity review |
| Whisper GPU cost > 35 % of total | re-evaluate Whisper-medium fallback threshold |
| CDN egress > 30 % of total | engage `runbooks/playback-cdn-cache-cascade.md` |

## References

- ADR-RECORDINGS-0001, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005.
- `capacity-model.md`.
- foundry-runtime pricing surface (cross-µservice).
