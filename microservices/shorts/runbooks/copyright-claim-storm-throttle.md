---
doc_class: Runbook
title: Copyright-claim storm throttle
microservice: shorts
severity: "Sev-2 (storm) / Sev-1 (forged-claim-storm; DMCA §512(f) misrepresentation)"
status: Accepted
owner_team: ops-legal + ops-sre-reliability + axis-shorts
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-08)
  - microservices/shorts/threat-model.md (T-S-03, T-D-03, T-T-03)
  - microservices/shorts/compliance.md §DMCA
doc_status: published
---

# Runbook: Copyright-claim storm throttle (FM-08)

## Trigger

- `oya_shorts_copyright_claim_filings_per_sec` > per-claimant rate limit (default 100/hr; verified-business 1000/hr).
- `oya_shorts_copyright_claim_worker_queue_depth` > 1000.
- `oya_shorts_copyright_claim_false_positive_rate` > 5% on a single claimant.
- ops-legal manual page (claimant pattern looks bad-faith).

## Severity

Sev-2 default; escalate to Sev-1 if:
- Forged-claim pattern confirmed (DMCA §512(f) misrepresentation) → ops-legal pursues damages.
- Mass auto-hide cascading impact (creators affected > 1% of pack MAU).
- Pattern correlates with coordinated extortion.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect claim filings by claimant: `topk(10, oya_shorts_copyright_claim_filings_per_sec) by (claimant_ref)` | ≤ 3 min |
| 2 | Apply emergency per-claimant rate limit at gateway: `kubectl -n shorts patch configmap shorts-rate-limit` | ≤ 5 min |
| 3 | If single claimant > 70% of queue: cordon that claimant's filing endpoint pending ops-legal review | ≤ 5 min |
| 4 | Engage ops-legal on-call for DMCA §512(f) evaluation | ≤ 10 min |
| 5 | Communicate to affected creators (via tenant-of-tenant) about review delay | ≤ 15 min |
| 6 | Pause auto-hide on pending claims; require manual review for affected window | ≤ 10 min |

## Forged-Claim Detection (Sev-1 path)

If false-positive rate per claimant > 5%:

1. Engage ops-legal: DMCA §512(f) misrepresentation prima facie case.
2. Audit-chain seal review of all claims by this claimant (Ed25519-verified).
3. Verify perjury-attestation chain (§512(c)(3)(A)(vi)) for each claim.
4. If forged: reverse auto-hides; restore videos; emit `CopyrightClaimReversedDueToBadFaith` audit event.
5. Per-affected-creator notification + counter-notice support.
6. ops-legal pursues §512(f) damages remedy + claimant ban + repeat-claimant blocklist.
7. EU DSA Art. 16 transparency: incident emitted to per-tenant transparency log.
8. Postmortem with content-moderation + copyright-claim BC review.

## Per-Tenant Throttle Tuning

| Tenant tier | Default per-hour claim cap | Verified-business cap | Notes |
|---|---|---|---|
| Free | 10/hr | n/a | typically end-user; rare to file claim |
| Basic | 50/hr | 200/hr (business-verified) | per-tenant |
| Premium | 100/hr | 1000/hr | per-tenant |
| Enterprise | 500/hr | 5000/hr | per-tenant; SLA-backed |

## DMCA Compliance Throughout Throttle

- Throttling MUST NOT prevent legitimate claims from being filed within DMCA-conformant SLA (typically 14d response window from Safe Harbor).
- Throttle queues claims; does not reject; claims processed in order with priority queue per tier.
- Designated agent (ops-legal) notified via dashboard + email on queue depth.
- Per §512(c)(2): designated-agent contact updated with US Copyright Office; backup agent designated.

## Recovery Verification

- `oya_shorts_copyright_claim_worker_queue_depth` < 100 for ≥ 30 min.
- `oya_shorts_copyright_claim_false_positive_rate` < 1% per claimant for ≥ 7d.
- No active alerts on copyright-claim path.
- DMCA §512(c) SLA met for all in-flight claims.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Legitimate rights-holder bulk-files (e.g., movie release window) | concentrated claimant pattern; perjury-attestations consistent | accept; expand cap temporarily; ops-legal coordination with claimant |
| Coordinated forged-claim campaign | high false-positive rate; correlated targets | Sev-1; ops-legal §512(f) pursuit |
| Per-claimant config bug | single tenant misconfigured | fix tenant config; re-enable normal limits |
| Fingerprint corpus poisoning (T-T-03) | claims from licensor with mismatched corpus | cordon licensor namespace; ops-legal review |
| Bot-spammed claim submissions | non-human filing pattern | gateway-level bot detection; rate-limit by IP + claimant identity |

## Postmortem Triggers

- If forged-claim event confirmed: ops-legal pursues §512(f) remedy + claimant blocklist + transparency report disclosure per EU DSA Art. 24.
- If legitimate event sustained: capacity revision; per-claimant priority lanes.
- If false-positive rate elevated: classifier review (paired with content-moderation BC).

## References

- `microservices/shorts/failure-modes.md` FM-08.
- `microservices/shorts/threat-model.md` T-S-03, T-D-03, T-T-03.
- `microservices/shorts/compliance.md` §DMCA Title II Safe Harbor.
- DMCA Title II 17 USC §512(c)(3)(A)(vi), §512(f), §512(i)(1)(A).
- EU DSA Art. 16 (notice-and-action), Art. 24 (transparency report).
