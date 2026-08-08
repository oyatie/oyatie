---
doc_class: Runbook
title: Per-viewer watermark key rotation
microservice: recordings
severity: "Sev-3 (planned) / Sev-2 (if compromise suspected)"
status: Accepted
owner_team: ops-security + axis-recordings + cloud-secrets
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0004-playback-and-cdn-strategy.md
doc_status: published
---

# Runbook: Per-viewer watermark key rotation

## Purpose

Rotate the per-tenant watermark seed used to derive per-viewer dynamic +
steganographic watermarks. Rotation occurs (a) on schedule every 30 days,
(b) on suspected compromise (Sev-2), (c) on share-link HMAC leak (Sev-2).

## Symptoms (compromise case)

- Anomalous signed-URL usage pattern detected.
- Leaked recording identified externally with watermark traceable to a
  watermark key whose KMS access has been suspicious.

## Procedure (scheduled rotation)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Generate new 256-bit per-tenant watermark seed in OpenBao: `secret/recordings/<tenant>/watermark-seed-v<N+1>` | cloud-secrets | ≤ 5 min |
| 2 | Update tenant config: new watermark version is the default for new playback sessions | server | atomic |
| 3 | Old version retained for verification of legacy leaked-recording attribution | server | indefinite (audit data class) |
| 4 | Audit-chain seal: `WatermarkRotated` event emitted | server | ≤ 1 s |
| 5 | Playback worker picks up new seed within next refresh cycle (5 min) | server | ≤ 5 min |

## Procedure (compromise rotation — Sev-2)

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page ops-security + axis-recordings + council-privacy | on-call | immediate |
| 2 | Generate new seed + immediately switch tenant config | cloud-secrets + ops-security | ≤ 5 min |
| 3 | Force-invalidate all in-flight signed URLs for the tenant; refuse `start_playback_session` for sessions older than now | axis-recordings | ≤ 5 min |
| 4 | CDN purge for the tenant's playback URLs | ops-sre | ≤ 10 min |
| 5 | If a leaked recording is identified: trace via the old-key's watermark → identify leaker; engage tenant + counsel | ops-security + council-privacy | ≤ 24h |
| 6 | Audit-chain seal: `WatermarkRotated` + `WatermarkCompromiseInvestigated` events | server | ≤ 5 min |
| 7 | Customer notification per pack | council-privacy | ≤ 24h |

## Watermark Strategy (per ADR-RECORDINGS-0004)

- **Visible watermark**: tenant logo + viewer email + playback timestamp;
  overlaid by ffmpeg + watermark filter at playback time.
- **Invisible (steganographic) watermark**: HMAC-derived per-viewer bit
  pattern embedded in DCT coefficients; survives screen-capture re-encode
  at moderate bitrate; detector µservice (future ADR) verifies leak
  attribution.
- **Key derivation**: per-viewer key = HMAC-SHA256(tenant-watermark-seed,
  viewer_ref || recording_id || session_id).

## Verification

- New watermark seed in OpenBao.
- Old seed retained in audit data class.
- `WatermarkRotated` audit-chain event sealed.
- Sample playback test confirms new watermark applies.

## Postmortem Triggers

- Any compromise-rotation (Sev-2).
- Any successful leak-attribution.
- Any KMS-shred of the old watermark seed before legal-hold review (must
  retain for any open hold).

## References

- ADR-RECORDINGS-0004.
- OpenBao rotation procedure.
- `policy/cedar/tenant-scope.cedar`.
- HMAC-SHA256 (FIPS 198-1).
