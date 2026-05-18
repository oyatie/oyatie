---
doc_class: Runbook
title: CDN cache invalidation cascade
microservice: shorts
severity: "Sev-2 (cascade) / Sev-3 (per-POP failure)"
status: Accepted
owner_team: ops-sre-reliability + axis-shorts
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-06, FM-07)
  - microservices/shorts/multi-region.md (CDN topology)
  - microservices/shorts/threat-model.md (T-D-04)
doc_status: published
---

# Runbook: CDN cache invalidation cascade (FM-06 + FM-07)

## Trigger

- `oya_shorts_cdn_invalidation_rate` spike (>10x baseline).
- `oya_shorts_cdn_cache_hit_ratio` < 70 % sustained 15min.
- Mass-takedown event triggers global purge storm.
- Cloudflare health-check failure for in-pack POP.
- Tenant-reported: video playback degradation (high latency, buffering).

## Severity

Sev-3: single POP failure (auto-failover to nearest healthy POP).
Sev-2: multi-POP failure OR cache-hit-ratio collapse OR sustained > 30 min.
Sev-1: pack-wide CDN outage (all in-pack POPs down).

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Check Cloudflare status: `wrangler` / API for in-pack POP health | ≤ 2 min |
| 2 | Check cache-hit ratio by region: `oya_shorts_cdn_cache_hit_ratio` by `pop_region` | ≤ 3 min |
| 3 | Identify cascade origin: which invalidation triggered? mass-takedown? DMCA storm? misconfig? | ≤ 5 min |
| 4 | If cache-key-based invalidation: prefer over full-purge (avoid future storms) | ≤ 5 min |
| 5 | If full-purge in flight: pause additional purges; let cache rebuild lazy | ≤ 5 min |
| 6 | Verify S3 origin serving: degrade gracefully when CDN cold | ≤ 5 min |
| 7 | If all in-pack POPs down: failover to S3 origin direct (degraded latency but functional) | ≤ 10 min |

## Sev-1 Pack-Wide CDN Outage

If all in-pack POPs unreachable for ≥ 15 min:

1. Engage Cloudflare TAM via support; escalate to P1 ticket.
2. Communicate to affected tenants via status page.
3. Serve direct-from-S3 origin (degraded latency 3-5x; cost +20x but functional).
4. If sustained: consider activating DR-pair pack CDN failover.
5. Watch for cascading failures: feed-load latency may spike; transcode-queue may grow.
6. Postmortem with Cloudflare + ops-sre-reliability + cloud-k8s.

## Mass-Takedown Cascade

When mass DMCA-takedown or moderation-rollback triggers cache invalidation:

1. Use **cache-key-based invalidation** (sub-second TTL update on per-video manifest) over **full-purge**.
2. Per-video CDN purge: `wrangler purge --tag video-{video_id}`.
3. Rate-limit CDN purge API: max 100 purges/sec; queue overflow to batch.
4. Manifest TTL ≤ 15min ensures stale content expires naturally even without explicit purge.
5. For DRM-protected content: key rotation force-revokes existing licenses; purge becomes belt-and-suspenders.

## CDN POP Failover Architecture

Per `multi-region.md`:
- Cloudflare auto-routes to nearest healthy POP within pack region (default).
- If all in-pack POPs down: degrade to in-pack S3 origin via Cloudflare Workers fallback.
- Cross-pack CDN failover requires data-residency review (forbidden by default; pack-pinning).

## Recovery Verification

- `oya_shorts_cdn_cache_hit_ratio` ≥ 90 % for ≥ 30 min.
- `oya_shorts_video_start_latency_p95` ≤ 400ms.
- `oya_shorts_cdn_invalidation_rate` returns to baseline.
- No active alerts on CDN path.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Single POP outage | per-POP health check fails; cache-hit drops in single region | Cloudflare auto-failover; verify; raise ticket if sustained |
| Mass-takedown cascade | invalidation rate spike correlates with takedown event | use cache-key invalidation; rate-limit purge API |
| Cloudflare control-plane outage | API errors; multi-POP failure | Cloudflare TAM escalation; serve from origin |
| Misconfigured cache-control headers | new deploy correlates; cache-hit collapses | rollback Helm; investigate misconfig |
| Origin-S3 latency spike | origin fetch time elevated | investigate S3-side; CDN serves stale during recovery |

## Postmortem Triggers

- If Cloudflare-side outage: vendor-engagement record + multi-vendor CDN consideration.
- If cascade pattern from mass-takedown: cache-key-invalidation primary path review.
- If misconfig: deploy review + canary policy.

## References

- `microservices/shorts/failure-modes.md` FM-06, FM-07.
- `microservices/shorts/multi-region.md` §CDN topology.
- `microservices/shorts/threat-model.md` T-D-04.
- Cloudflare R2 + Workers docs.
- HLS RFC 8216 (manifest TTL semantics).
