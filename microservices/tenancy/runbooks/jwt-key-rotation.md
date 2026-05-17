---
doc_class: Runbook
title: JWT signing-key rotation (routine + emergency)
microservice: tenancy
severity: "Sev-3 (routine) / Sev-1 (emergency on suspected compromise)"
status: Accepted
owner_team: ops-security + axis-tenancy
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-04 JWT compromise; FM-13 secret leak)
  - microservices/tenancy/threat-model.md (T-S-01)
  - microservices/tenancy/policy/rls-isolation.md (Invariant JWT-04)
  - microservices/tenancy/incident-response.md
doc_status: published
---

# Runbook: JWT signing-key rotation

## Purpose

JWT signing keys are the load-bearing identity primitive: every µservice's tenant validation depends on the public-key fingerprint advertised by tenancy. Routine rotation (30d) is the primary defence; emergency rotation (immediate) on suspected compromise is the secondary.

## Trigger

ONE of:
- **Routine**: OpenBao rotation cron fires (30d cadence per pack per environment).
- **Emergency**: secret-scanner detects leaked key material in commit / log (FM-13).
- **Emergency**: OpenBao audit-log anomaly detected (unusual signing-key access pattern; potential compromise).
- **Emergency**: pen-test reveals JWT forgery succeeded (T-S-01).
- **Manual**: ops-security initiates rotation on policy-driven schedule change.

## Severity

- Routine 30d rotation: Sev-3 (operational; not an incident; standard ops).
- Emergency on suspected compromise: **Sev-1** (security breach risk).

## Routine rotation (normal)

| Step | Action | Time budget |
|---|---|---|
| 1 | OpenBao rotation cron fires; generates new Ed25519 keypair; binds to (pack, env). | ≤ 5 s |
| 2 | Old keypair retained as `previous` for verification grace (30d). | – |
| 3 | tenancy-isolation-policy-worker queries OpenBao for new key; computes fingerprint. | ≤ 5 s |
| 4 | Worker emits `JwtSigningKeyRotated{pack, env, prev_fingerprint, new_fingerprint, rotated_at, kid}` Workflow event. | ≤ 5 s |
| 5 | Every µservice's JwtVerifier subscriber receives event; refreshes public-key cache to include the new fingerprint (keeping the old for grace period). | ≤ 30 s |
| 6 | tenancy-isolation-policy-rest's JwtIssuer adopts new key for new issuances (older in-flight tokens still verify against `previous` fingerprint). | ≤ 1 min cluster-wide |
| 7 | Audit-chain seal: signed envelope. | ≤ 1 s |
| 8 | LEAN check `oya-governance-jwt-key-fingerprint-advertised` validates the rotation event was emitted. | – |

## Emergency rotation (suspected compromise)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security + axis-tenancy SME; open `#inc-sec-<id>`. | ≤ 3 min |
| 2 | Confirm scope: which pack/env signing key suspected? Single pack vs all? | ≤ 5 min |
| 3 | OpenBao operator (2-person rule + JIT elevation): generate new keypair IMMEDIATELY; mark old key `revoked` (not just `previous`; revoked keys are not accepted by verifiers). | ≤ 5 min |
| 4 | tenancy-isolation-policy-worker emits `JwtSigningKeyRotated` Workflow event with `emergency=true` and `revoke_previous=true` flags. | ≤ 5 s |
| 5 | Verifier subscribers refresh: cache update + immediate revocation of `previous` key from cache. | ≤ 30 s cluster-wide |
| 6 | **All in-flight JWTs signed by revoked key are invalidated**: customers re-authenticate. Tenant operators notified per `incident-response.md` Sev-1 template. | ≤ 15 min total |
| 7 | Forensic trace: how was key exposed? OpenBao audit-log review + repo scan + log review. | hours |
| 8 | Breach-notification chain per `compliance.md` per pack timelines (GDPR 72h; KR PIPA 72h; HIPAA 60d if PHI-affecting). | per SLA |
| 9 | Audit-chain seal: emergency-rotation envelope (separate event type for auditability). | ≤ 1 s |

## Pre-checks (recovery)

1. Verify rotation event delivered: query Workflow event log for the rotation event id.
2. Verify verifier pubkey cache update: each µservice has a `oya_jwt_verifier_cache_fingerprint` metric; new fingerprint should appear within 30s of event.
3. Verify JwtIssuer adopted new key: new issuance carries new `kid` header.
4. For emergency: verify the old key is unreachable from OpenBao (deleted from `previous` mount).

## Recovery Path A — Routine rotation incomplete (some µservice missed event)

Cause: Workflow event consumer down; pubkey cache stale.

| Step | Action |
|---|---|
| 1 | Identify lagging µservice via metric: `oya_jwt_verifier_cache_fingerprint{microservice=<>}` shows stale fingerprint. |
| 2 | Engage µservice's on-call; verify event consumer health. |
| 3 | If consumer down: restart pods; backlog drains. |
| 4 | If consumer healthy but cache stale: invoke per-µservice cache-refresh endpoint (admin-only). |
| 5 | Once cache refresh complete, verifier accepts JWTs signed by new key. |

## Recovery Path B — Emergency rotation: re-auth flood overwhelms validate path

Cause: every µservice's customers re-authenticate simultaneously when keys revoked; validate-path RPS spike.

| Step | Action |
|---|---|
| 1 | Pre-anticipated by capacity-model.md; HPA scales validate-rest from 3 → up to 100 replicas per cell. |
| 2 | If overload persists: temporary per-tenant rate-limit tightening to spread re-auth across 5-10min window. |
| 3 | Tenant operator notified to expect ≤ 10min auth-token re-issuance window. |

## Recovery Path C — Old-key residual verification (in-flight tokens)

Routine rotation: 30d grace where old fingerprint remains in verifier caches. If a workload µservice's JwtVerifier somehow purges old fingerprint early:

| Step | Action |
|---|---|
| 1 | Identify: customer reports "I'm being challenged to re-auth, but my token is < 1h old." |
| 2 | Verify the µservice's verifier cache has the previous fingerprint (should be there for 30d grace). |
| 3 | If cache prematurely purged: restore from per-µservice cache-load script (the cache is just a Valkey-or-in-memory lookup). |
| 4 | Postmortem: cache-eviction logic too aggressive? Tighten grace period enforcement. |

## Verification

After rotation:
- New fingerprint visible in `oya_jwt_active_fingerprint_*` metric.
- All verifiers (every µservice) have cached the new fingerprint.
- JwtIssuer issues new tokens with new `kid`.
- For routine: old fingerprint retained 30d; in-flight tokens verify.
- For emergency: old fingerprint immediately revoked; in-flight tokens rejected.
- Audit-chain seal log captures the rotation event.

## On-Call Notes (rotation-related)

Per `incident-response.md` On-Call Rotation:
- Routine rotation is monitored by axis-tenancy on-call SME; alarms only if propagation lag > 5 min.
- Emergency rotation pages ops-security + axis-tenancy SME + DPO + ExecSponsor.
- Rotation drill: quarterly synthetic rotation; verify Workflow event propagation + verifier cache refresh.

## Post-incident updates

- Routine: log to evidence/rotation-history; no postmortem unless propagation lag occurred.
- Emergency: postmortem within 5 business days; breach-notification per pack SLA; harden secret-management discipline; LEAN check tightening if secret-scanner gap exposed.

## References

- `microservices/tenancy/failure-modes.md` FM-04 + FM-13.
- `microservices/tenancy/threat-model.md` T-S-01.
- `microservices/tenancy/policy/rls-isolation.md` Invariant JWT-01..JWT-04.
- `microservices/tenancy/incident-response.md`.
- `microservices/tenancy/compliance.md` §"Regulatory Notifications".
- OpenBao docs — `openbao.org`.
- OWASP API Top 10 (2023) #2 (Broken Authentication).
- RFC 7518 (JOSE alg=EdDSA).
