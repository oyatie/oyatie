---
doc_class: Runbook
title: Lobby bypass incident
microservice: meet
severity: "Sev-1 (security breach)"
status: Accepted
owner_team: ops-security + axis-meet + council-privacy
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-05)
  - microservices/meet/threat-model.md (T-I-04)
  - microservices/meet/policy/meeting-scope.cedar
  - microservices/meet/dashboards/recording-pipeline.json
doc_status: published
---

# Runbook: Lobby bypass incident (meet)

## Trigger

Any of:
- `oya_meet_lobby_bypass_attempt_total` > 0 with `result=succeeded`.
- Tenant escalation: unauthorized participant in meeting.
- ops-security alert: unusual LiveKit publisher token issued without lobby_approved bit.
- Synthetic chaos drill detects bypass path.

## Severity

**Sev-1 (always)**. Lobby bypass = unauthorized participant in a meeting = potential confidentiality breach. Regulatory clocks may start:
- GDPR Art. 33 (72h) if EU data subjects in meeting.
- KR PIPA Art. 34 (≤ 24h) if KR data subjects.
- HIPAA §164.412 if PHI exposed.
- SEC Rule 17a-4 supervisor notification if pack-us-financial recorded comms.

## Immediate Mitigation (≤ 5 min block; ≤ 15 min containment)

| Step | Action | Time |
|---|---|---|
| 1 | Block meet-rest token-redemption path globally (emergency Cedar policy push: deny all `Action::"redeem_lobby_token"`) | ≤ 5 min |
| 2 | Force-disconnect bypass participant (LiveKit room-evict; audit-chain seal) | ≤ 5 min |
| 3 | Quarantine affected meeting instance: writes blocked; reads only by ops-security under JIT elevation | ≤ 5 min |
| 4 | Surface in-meeting host banner: "Unauthorized participant detected and removed; meeting can continue" | ≤ 5 min |
| 5 | Engage ops-security + council-privacy; declare breach-suspect | ≤ 10 min |
| 6 | Identify scope: how many meetings affected? what's exposure timeline? | ≤ 30 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Cedar policy bug allows bypass | new Cedar policy version + bypass coincides | review recent policy changes |
| LiveKit access-token forgery | LiveKit token with lobby_approved=true issued without server-side approval | check OpenBao audit; verify token-issuance code path |
| JWT algorithm confusion | tokens signed with HS256 instead of Ed25519 | verify JWT verify code path enforces algorithm explicitly |
| Replay attack (old approved token reused) | timestamp-replay pattern; jti reuse | verify nonce + jti uniqueness enforcement |
| Side-channel via meeting-instance state mutation | direct Postgres mutation bypassing service | check Postgres audit log; RLS-bypass query |

## Investigation Procedure

1. Pull LiveKit access-log for the affected meeting-instance: list all token issuances + redemption attempts.
2. Cross-reference with Postgres participant-log: which redemptions matched server-side approval records?
3. Identify the gap: which redemption resulted in publisher capabilities without matching approval record?
4. Reproduce locally: synthetic chaos in staging cluster.
5. Patch identified vulnerability; emergency-rotate Cedar policy + LiveKit access-key.
6. Forensics: pull any recordings/transcripts of the affected meeting (post-incident); review for what unauthorized participant could have seen/heard.

## Regulatory Notification (Sev-1)

| Pack | Trigger | Window | Recipient |
|---|---|---|---|
| pack-eu | EU data subject in affected meeting | 72h GDPR Art. 33 | Lead supervisory authority |
| pack-kr | KR data subject | ≤ 24h KR PIPA Art. 34 | KR PIPC |
| pack-us-healthcare | PHI in meeting | 60d HIPAA §164.412 (per affected individual) | HHS + affected individuals |
| pack-us-financial | Recorded supervised-comms affected | "promptly" | Tenant FINRA supervisor |
| pack-eu (MiFID II) | Investment-firm recorded comms | "without undue delay" | Tenant compliance + competent authority |

CommsLead drafts notification text from `legal/breach-notification-templates.md`.

## Recovery Verification

- Lobby bypass attempt rate back to 0 sustained for ≥ 24h.
- Cedar policy patched + LEAN-lane `oya-check-cedar-coverage` green.
- LiveKit access-token forgery vector closed; verified via penetration drill.
- All affected meetings reviewed; no further unauthorized access found.

## Postmortem (5 business days)

| Step | Owner | Deadline |
|---|---|---|
| Blameless postmortem meeting | IC + ops-security + council-privacy | within 2 business days |
| Root-cause analysis | SME | within 3 business days |
| Action items (Cedar policy review, LiveKit access-key audit, Lobby evaluation test coverage) | All | meeting end |
| Tracker created | IC | meeting end |
| Re-review at 30/60/90 days | IC | rolling |
| External communication (if breach confirmed) | CommsLead + ExecSponsor | per pack notification clock |

## Chaos Drill (quarterly)

| Step | Owner |
|---|---|
| Synthetic guest attempts lobby bypass with crafted token | ops-security red team |
| Verify detection at meet-rest token-redemption | ops-sre-reliability |
| Verify LiveKit refuses publish without lobby_approved | axis-meet |
| Verify alerts fire + on-call paged | ops-sre-reliability |
| Verify Cedar policy patch path reversible | ops-security |

## References

- ADR-MEET-0003 (E2E meetings have different lobby semantics).
- `microservices/meet/policy/meeting-scope.cedar`.
- `microservices/meet/threat-model.md` T-I-04, T-E-05.
- GDPR Art. 33/34; KR PIPA Art. 34; HIPAA §164.412; SEC Rule 17a-4; FINRA 4530.
- NIST SP 800-61 Incident Handling Guide.
