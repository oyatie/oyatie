---
doc_class: Runbook
title: DRM key rotation
microservice: shorts
severity: "Sev-3 (routine 90d rotation) / Sev-1 (forced rotation on compromise indicator)"
status: Accepted
owner_team: axis-shorts + cloud-secrets + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-15, FM-27)
  - microservices/shorts/threat-model.md (T-I-10, T-E-06)
  - microservices/shorts/decisions/ADR-SHORTS-0004-drm-substrate-tenant-tier.md
doc_status: published
---

# Runbook: DRM key rotation (FM-15 + FM-27)

## Trigger

- Scheduled: 90d rotation cadence (Widevine + FairPlay + PlayReady root-key rotation).
- Scheduled: 7d rotation cadence (per-content key rotation).
- Forced: `oya_shorts_drm_key_compromise_indicator_total` > 0 → immediate rotation + revocation list.
- Vendor escalation: Widevine / FairPlay / PlayReady security advisory.

## Severity

Sev-3 for scheduled rotation.
Sev-1 for forced rotation (compromise indicator).

## Routine 90d Rotation (Sev-3)

### Widevine

| Step | Action | Time |
|---|---|---|
| 1 | Coordinate maintenance window with cloud-secrets (HSM access) | T-7d |
| 2 | Generate new Widevine root key in OpenBao HSM | T-3d |
| 3 | Begin issuing new per-content keys derived from new root | T-1d |
| 4 | Existing licenses continue with old root for grace period (24h) | T-0 |
| 5 | After grace period: old root marked retired; new root primary | T+1d |
| 6 | Audit-chain seal: `DrmKeyRotationCompleted{key_system: widevine, root_version: N→N+1}` | T+1d |
| 7 | Vendor (Google Widevine) coordination via SecureStop API | T+2d |

### FairPlay

Same procedure with Apple FairPlay key-server API.

### PlayReady

Same procedure with Microsoft PlayReady DRM-server API.

### Per-Content Key Rotation (7d)

Automated worker rotates per-content keys every 7d:
1. New per-content key derived from current root.
2. New keys distributed to active CDN POPs.
3. Old key marked expiring; existing licenses use old key during grace window.
4. After 7d: old key revoked; new licenses use new key.
5. Audit-chain seal per video: `DrmPerContentKeyRotated{video_id, key_version: N→N+1}`.

## Forced Rotation (Sev-1 path)

If compromise indicator triggers:

| Step | Action | Time |
|---|---|---|
| 1 | Sev-1 declared; ops-security + cloud-secrets + axis-shorts war-room | ≤ 5 min |
| 2 | Identify scope: which key-system (W/FP/PR); which per-content keys; which tenants | ≤ 10 min |
| 3 | Immediate: add affected per-content keys to revocation list (in-pack distribution) | ≤ 15 min |
| 4 | Generate emergency new root key in OpenBao HSM | ≤ 30 min |
| 5 | Distribute new root to all active key-servers in pack (multi-AZ HA cluster) | ≤ 30 min |
| 6 | Re-issue licenses for affected videos with new root | continuous |
| 7 | Engage vendor (Google / Apple / Microsoft) for root-key validity review | ≤ 1h |
| 8 | If root-key compromised at vendor level: full HSM rebuild; multi-week recovery | escalate |
| 9 | Notify affected tenants (Premium-tier with DRM); estimate user impact | ≤ 30 min |
| 10 | Postmortem with cloud-secrets + ops-security + ADR-SHORTS-0004 review | ≤ 5d |

## License Issuance Overload (FM-27)

If license issuance rate > 80% of key-server capacity:

1. Trigger HA cluster scale-up via Terraform (per-pack DRM key-server cluster).
2. Per-tenant rate limit at gateway to smooth spikes.
3. Client SDK exponential backoff on license-acquisition retry.
4. Verify DRM-protected content playback latency p95 ≤ 150ms.

## Verification

- `oya_shorts_drm_license_issuance_duration_p95` ≤ 150ms.
- `oya_shorts_drm_key_rotation_failure_total` rate = 0 for ≥ 24h.
- All per-content keys within rotation window (7d cap).
- Root keys within rotation window (90d cap).
- Audit-chain seal present for every rotation event.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Scheduled rotation deadline approaching | rotation-cadence metric > 80d (root) or > 6d (content) | trigger rotation per schedule |
| Compromise indicator (key leak) | external threat intel; vendor advisory; license-server unusual access | Sev-1 forced rotation |
| Vendor service outage | Widevine / FairPlay / PlayReady endpoint unreachable | vendor TAM escalation; in-pack key-server falls back to local HSM operations |
| OpenBao access misconfiguration | HSM access denied | cloud-secrets team engagement |
| HSM hardware failure | HSM cluster member offline | failover to active replica; replace failed unit |

## References

- `microservices/shorts/failure-modes.md` FM-15, FM-27.
- `microservices/shorts/threat-model.md` T-I-10, T-E-06.
- `microservices/shorts/decisions/ADR-SHORTS-0004`.
- Widevine SecureStop API.
- FairPlay key-server API (Apple Developer).
- PlayReady DRM-server API (Microsoft).
- W3C EME 2017.
- OpenBao HSM docs.
