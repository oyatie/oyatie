---
doc_class: Runbook
title: Session Storm — session-store overload mitigation
microservice: application
severity: "Sev-2"
status: Accepted
owner_team: ops-security + axis-application + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-06)
  - microservices/application/incident-response.md
  - microservices/application/threat-model.md (D-01, D-02)
doc_status: published
---

# Runbook: Session Storm

## Trigger

Valkey memory > 80 % for ≥ 3 min OR
`oya_application_session_create_rate` > 10× rolling baseline OR
`oya_application_auth_rate_limit_block_total` spikes (credential-stuffing
signal).

## Severity

**Sev-2** — degraded but functional; existing sessions OK; new sign-ins
may fail intermittently if not mitigated.

## Pre-checks

1. Identify storm class:
   - **Credential stuffing**: `oya_application_auth_rate_limit_block_total{reason="rate_limit"}` is high; many distinct source IPs.
   - **Viral sign-in (e.g., post-OAuth-callback)**: high but concentrated source IPs; success rate normal.
   - **Cookie replay attack**: many sessions with same fingerprint; investigate via auth-chain.
   - **Legitimate spike** (new tenant onboarding; product launch).
2. Confirm Valkey Sentinel/Cluster status: which node memory pressure? Are evictions occurring?
3. Confirm HPA on auth-gateway and session-store scaling.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Sev-2; assign IC + ops-security SME | ≤ 5 min |
| 2 | Pre-checks | ≤ 5 min |
| 3 | If credential-stuffing: engage WAF challenge mode: `cargo run -p oya-dev-cli -- application waf raise --pack <pack> --mode challenge` — Cloudflare bot management / OCI WAF puts JavaScript challenge in front of sign-in path | ≤ 5 min |
| 4 | Raise per-IP rate limit: `cargo run -p oya-dev-cli -- application auth rate-limit set --pack <pack> --per-ip-per-min 10 --per-user-per-min 5` (down from defaults 60/30) | ≤ 2 min |
| 5 | Scale auth-gateway HPA min: `kubectl scale deployment/oya-application-auth-gateway-rest --replicas=8` (override min until storm subsides) | ≤ 2 min |
| 6 | If Valkey memory pressure: trigger cluster expansion (add a shard pair) `kubectl scale statefulset/oya-application-valkey-cluster --replicas=8` (from 6 → 8) | ≤ 10 min |
| 7 | Tighten session-store eviction policy temporarily: `cargo run -p oya-dev-cli -- application session set-eviction-aggressive --pack <pack>` (kicks in if memory still pressured) | ≤ 2 min |
| 8 | Monitor `oya_application_session_create_rate` falling; `oya_application_valkey_memory_used_ratio < 0.7` | ≤ 30 min |
| 9 | When storm subsides: revert WAF + rate-limit; auth-gateway HPA returns to normal min | ≤ 1 h after subsided |
| 10 | CommsLead: tenant comm if user-visible impact (sign-in retries seen) | ≤ 30 min |
| 11 | Postmortem; action: was rate-limit baseline too generous? | ≤ 5 BDs |

## Defense-in-depth

The Application Shell is already hardened against this scenario; runbook
mitigations are escalations beyond the baseline. Baseline already in place:

- Per-IP rate-limit 60/min on auth path.
- Per-user rate-limit 30/min.
- Valkey: 3-master + 3-replica cluster with `allkeys-lru` eviction.
- Auth-gateway HPA: min 4, max 50 in production.
- Cloudflare / OCI WAF: rate-shape + bot management baseline.
- OWASP CRS: enabled with credential-stuffing pattern.

## Verification

- `oya_application_session_create_rate` returns to ≤ 2× baseline for ≥ 15 min.
- `oya_application_valkey_memory_used_ratio < 0.7`.
- `oya_application_auth_signin_success_rate > 99 %` over recent 5 min.
- No alarm on `oya_application_session_evicted_unexpected_total`.

## Forensic capture

If credential-stuffing or replay is confirmed:
- Capture sample requests via Tempo trace sampling at 100 % for 1 hr.
- Engage ops-security forensic for attacker IP / fingerprint analysis.
- File breach assessment if any account compromise confirmed.

## References

- `failure-modes.md` FM-06.
- `threat-model.md` D-01 (auth flood), D-02 (session-store DOS).
- OWASP ASVS V11 (business logic).
