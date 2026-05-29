---
doc_class: Runbook
title: Federation bridge degraded (ActivityPub)
microservice: social
severity: "Sev-2 (Professional-tier degradation) / Sev-1 (Personal-tier leak attempt)"
status: Accepted
owner_team: axis-social + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-14, FM-20)
  - microservices/social/threat-model.md (T-S-03, T-I-08, T-D-07)
  - microservices/social/policy/dual-context-isolation.md (DCI-08)
  - microservices/social/decisions/ADR-SOC-0004-federation-posture.md
doc_status: published
---

# Runbook: Federation bridge degraded (FM-14 + FM-20)

## Trigger

Any of:
- `social_personal_tier_federation_attempt_total` > 0 (compile-time invariant means this should be unreachable; runtime guard fires — Sev-1).
- `social_federation_peer_spam_rate` > threshold (untrusted peer ingestion — Sev-2).
- HTTP Signature verification failures from a previously-trusted peer (compromise signal — Sev-2).
- Federation outbox queue depth > 100k (delivery lag — Sev-3).
- Federation inbox flood (peer DDoS — Sev-2).

## Severity

- Personal-tier-leak attempt: **Sev-1** (regulatory + privacy breach signal).
- Peer compromise / ingestion of malicious content: **Sev-2**.
- Outbox delivery lag: **Sev-3**.

---

## Path A: Personal-tier federation leak attempt (Sev-1)

This MUST be unreachable per compile-time invariant DCI-08; if the metric fires, this is a critical regression.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security + council-privacy immediately | immediate |
| 2 | Verify metric not a false-positive: inspect `social_personal_tier_federation_attempt_total` audit-chain seal | ≤ 5 min |
| 3 | Halt federation-gateway egress globally (kill switch via Cedar) | ≤ 5 min |
| 4 | Verify no Personal-tier activity actually egressed: check peer-side delivery logs + outbox audit | ≤ 15 min |
| 5 | If leak confirmed: declare data breach; GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 clocks may start | ≤ 5 min |
| 6 | Forensic snapshot of `federation-gateway` worker state, code SHA, deployment manifests | ≤ 10 min |
| 7 | Engineering: identify regression in compile-time invariant; emergency LEAN-lane investigation | days |
| 8 | Restore federation egress only after compile-time invariant verified | hours |

---

## Path B: Federation peer compromise / spam (Sev-2)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `social_federation_peer_spam_rate` by peer; identify offending peer | ≤ 3 min |
| 2 | Verify HTTP Signature anomalies in peer's recent deliveries | ≤ 5 min |
| 3 | Remove peer from allowlist (Cedar policy update via Helm rollout) | ≤ 5 min |
| 4 | Quarantine ingested content from compromised peer (last 24h or longer per pattern) | ≤ 15 min |
| 5 | Engage ops-security; if peer is a cooperative homeserver: notify peer admin | ≤ 30 min |
| 6 | Re-evaluate peer allowlist policy; tighten signature-verification thresholds if needed | hours |

---

## Path C: Federation inbox flood (Sev-2; DoS)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect inbox `req/sec` by peer; identify flooding source | ≤ 3 min |
| 2 | Apply per-peer rate limit (default 1k/min; reduce to 100/min during flood) | ≤ 5 min |
| 3 | If flood from a single peer continues: remove from allowlist | ≤ 5 min |
| 4 | Engage ops-security; investigate whether DDoS or legitimate burst | ≤ 30 min |

---

## Path D: Federation outbox delivery lag (Sev-3)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect outbox queue depth by `peer_id` | ≤ 3 min |
| 2 | Identify slow peer(s); check peer-side response latency | ≤ 5 min |
| 3 | Scale federation-gateway outbox workers (HPA or manual) | ≤ 5 min |
| 4 | If specific peer is consistently slow: increase per-peer parallelism; coordinate with peer admin | ≤ 15 min |
| 5 | If queue lag persists > 1h: pause outbox to affected peer; tenant notified | ≤ 30 min |

---

## Path E: Federation peer resync (planned operational)

Triggered by `federation_peer_resync_requested` event. See `backfill-replay.md` §"Federation Replay (ActivityPub outbox re-sync)" — the same runbook covers the resync ceremony.

---

## Diagnosis

| Hypothesis | Signal | Action |
|---|---|---|
| Personal-tier compile-time invariant regression | `social_personal_tier_federation_attempt_total` > 0 | Sev-1 path A |
| Peer compromise (rogue HTTP Signatures) | sig-verify failures + spam-rate spike | Sev-2 path B |
| Peer compromise (malicious content) | post-moderation classifier spike on peer-sourced posts | Sev-2 path B |
| Peer DDoS | inbox rate >> baseline; single-peer dominant | Sev-2 path C |
| Slow peer (legitimate) | outbox depth without inbox spike; peer-side latency reported | Sev-3 path D |
| Network partition oyatie ↔ peer | both inbox + outbox stall to single peer | Sev-3; coordinate with cloud-iac |

## Recovery Verification

- For path A: `social_personal_tier_federation_attempt_total` = 0 sustained; compile-time invariant re-verified.
- For path B: removed peer no longer in allowlist; no further spam from peer.
- For path C: inbox req/sec back to baseline.
- For path D: outbox depth back to ≤ 1000.

## Postmortem Triggers

- Path A: Sev-1 postmortem within 5 business days; council-privacy + ops-security sign-off; regulatory notifications per `incident-response.md`.
- Path B/C: Sev-2 postmortem; engage peer admin if cooperative.
- Path D: Sev-3 postmortem; capacity review.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | GDPR Art. 33 personal-data breach clock; EU DSA Art. 24 transparency-report update on Path-A leak; cross-border transfer (SCC) compliance review |
| pack-kr | KR PIPA Art. 28 cross-border consent breach review on Path-A leak; Art. 34 personal-data leakage notification |
| pack-us-healthcare | Federation OFF by default; any leak triggers HIPAA §164.412 breach-notification |
| pack-uk | UK Online Safety Act 2023 reporting where federation peers carry illegal content |
| pack-au | AU Online Safety Act 2021 reporting where federation peers carry harmful content |

## References

- ADR-SOC-0004 (federation posture).
- `microservices/social/failure-modes.md` FM-14, FM-20.
- `microservices/social/threat-model.md` T-S-03, T-I-08, T-D-07.
- `microservices/social/policy/dual-context-isolation.md` DCI-08.
- `microservices/social/policy/public-read.cedar` (federation inbox Cedar).
- `microservices/social/backfill-replay.md` §"Federation Replay".
- ActivityPub W3C Rec 2018.
- RFC 9421 HTTP Signatures.
