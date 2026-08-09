---
doc_class: Runbook
title: Playback CDN cache cascade failure
microservice: recordings
severity: "Sev-2 (cache hit < 70 %) escalating to Sev-1 if origin overloaded"
status: Accepted
owner_team: ops-sre-reliability + axis-recordings
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0004-playback-and-cdn-strategy.md
  - microservices/recordings/multi-region.md
doc_status: published
---

# Runbook: Playback CDN cache cascade failure

## Purpose

Recover from a CDN cold-cache + high-QPS cascade that overloads the S3-hot
origin and degrades playback-start latency past SLO.

## Symptoms

- `recordings.playback-start.latency` p99 > 1s (warm-target was 400ms).
- CDN cache hit rate < 70 %.
- S3-hot 5xx rate > 1 %.
- Playback session creation failure rate > 1 %.

## Diagnosis

1. Identify cause: cold cache after CDN config change? popular recording
   suddenly viral? regional CDN edge outage?
2. Check CDN edge logs by pack region.
3. Check S3-hot origin load.
4. Identify whether one tenant's recordings dominate the traffic.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page ops-sre + axis-recordings | on-call | immediate |
| 2 | Activate origin shielding: route all cold-misses through a regional shield bucket | ops-sre | ≤ 5 min |
| 3 | Pre-warm CDN for the top-N popular recordings by reading the playback heat-map (Valkey `popular-recordings`) | ops-sre | ≤ 10 min |
| 4 | If a single tenant is dominating: engage per-tenant playback rate-limit at the CDN edge | ops-sre | ≤ 5 min |
| 5 | Activate degraded mode: HLS-low-bitrate-only (480p) for non-paid tenants | axis-recordings | ≤ 5 min |
| 6 | If CDN edge outage in a region: route to DR-pair pack region per `multi-region.md` (subject to residency — only within-pack) | ops-sre | ≤ 10 min |
| 7 | Monitor cache hit rate recovery; once back above 80 % sustained 30 min, lift degraded mode + rate limits | ops-sre | ≤ 1h |

## CDN Cache Strategy (per ADR-RECORDINGS-0004)

- Cache key: `<pack>/<tenant_id>/<recording_id>/<bitrate>/<segment_index>`
  + signed-URL ID (so per-viewer watermark variants don't share cache keys).
- TTL: 24h for static segments; 1h for manifests.
- Purge: invalidation on redaction-overlay change.
- Per-pack origin shield: regional Lambda@Edge selects the in-pack origin.

## Verification

- CDN cache hit rate ≥ 90 % sustained 30 min.
- S3-hot 5xx rate < 0.1 %.
- Playback-start p99 ≤ 400ms warm / 1s cold.

## Postmortem Triggers

- Any Sev-1 escalation (origin overload).
- Any tenant impacted > 30 min.
- Any cross-pack routing during the incident (residency review).

## References

- ADR-RECORDINGS-0004.
- RFC 8216 (HLS).
- `multi-region.md`.
- `slos/playback-start-latency.openslo.yaml`.
