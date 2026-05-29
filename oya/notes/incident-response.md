---
doc_class: IncidentResponseGuide
title: notes µservice — Incident Response Guide
microservice: notes
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-notes + council-privacy
review_cadence: quarterly
references:
  - NIST SP 800-61 Rev. 2 Computer Security Incident Handling Guide
  - GDPR Arts. 33, 34 breach notification
  - KR PIPA Art. 34 breach notification
  - HIPAA 45 CFR §164.400 et seq. breach notification
doc_status: published
---

# Incident Response — notes µservice

## Severity Definitions

| Sev | Definition | RTO | Pager |
|---|---|---|---|
| Sev-1 | Personal-tier E2E invariant breach OR cross-tenant data leak OR multi-pack outage | 15 min | axis-notes oncall + ops-security + council-privacy |
| Sev-2 | Single-pack availability < 95 % OR AI E2E-refusal CI lane regression OR share-link brute-force confirmed | 1 h | axis-notes oncall |
| Sev-3 | Single-BC degraded (search-index slow, graph-render slow) | 8 h | axis-notes oncall |
| Sev-4 | Planned change with known impact | 24 h | axis-notes |

## Incident Lifecycle

1. **Detect** — Prometheus alert / customer report / chaos-test.
2. **Triage** — oncall confirms severity within 5 min; opens incident channel.
3. **Contain** — kill-switch where applicable (AI assist off; share-link emit off; web-clipper off).
4. **Eradicate** — root-cause + fix forward; rollback if needed.
5. **Recover** — restore service; verify SLO + AC.
6. **Lessons-learned** — post-mortem within 5 business days; ledger entry.

## Sev-1 Playbooks

### IR-01 Personal-tier E2E invariant breach (suspected)

Triggered by: confirmed report that oyatie server-side decrypted Personal-tier note plaintext, OR `oya_notes_ai_call_blocked_e2e_total` increment that traces to a code path that *would have* called AI on Personal content (near-miss).

Actions:
1. Page council-privacy + ops-security.
2. Freeze deploy: `oya vcs deploy freeze --microservice notes`.
3. Disable AI assist tenant-wide: `kubectl patch configmap notes-feature-flags --patch '{"data":{"ai_assist_enabled":"false"}}'`.
4. Audit-chain query for any decrypt attempt: `oya audit query --microservice notes --since 24h --kind PersonalE2EDecryptAttempt`.
5. If any non-zero hits → notify affected users per GDPR Art. 34 within 72h.
6. Forensic capture of code path; council-privacy reviews; ADR-NOTES-0001 update if model failed.

### IR-02 Cross-tenant data leak

Triggered by: customer report or `oya_notes_dual_context_denied_total` increment with confirmed cross-tenant artefact.

Actions:
1. Page ops-security + axis-notes oncall.
2. Identify scope: which tenants, which notes, which BC.
3. Rotate compromised tokens: per-installation web-clipper, OIDC keys if needed.
4. Engage tenancy + audit-chain to verify scope.
5. Notify affected tenant DPOs per GDPR Art. 33 within 72h.
6. KR PIPC notification per PIPA Art. 34 within 24h if KR-tenant impact confirmed.

### IR-03 Multi-pack outage

Triggered by: `oya:current_verdict:by_microservice_env` red for ≥ 2 packs simultaneously.

Actions:
1. Page ops-sre-reliability.
2. Confirm whether cause is shared (control-plane / observability) or coincidental.
3. Per-pack failover: switch to DR-pair where pack has one.
4. Status page update within 15 min.
5. Customer comms within 30 min.

## Sev-2 Playbooks

### IR-04 AI E2E-refusal regression in CI

Triggered by: `oya-check-e2e-ai-refusal` lane red on `dev`.

Actions:
1. Page council-privacy.
2. Revert offending PR via `oya vcs revert --pr <n>`.
3. Block re-merge until invariant restored.
4. Add reproducer test to `tests/regression/ai-e2e-refusal/`.

### IR-05 Share-link brute-force confirmed

Triggered by: `oya_notes_share_link_brute_force_attempts_total > 100/min` per tenant.

Actions:
1. Rate-limit auto-engages.
2. Optional CAPTCHA at threshold.
3. Notify affected tenant via comms channel.
4. Audit any successful access; if compromised, revoke share-links.

### IR-06 Web-clipper invalid-token spike

Triggered by: `oya_notes_web_clipper_invalid_token_total > 50/min` per tenant.

Actions per `runbooks/web-clipper-degraded.md`:
1. Identify install (extension version + browser + OS).
2. Force rotation of installation tokens.
3. Notify affected users via in-app banner.

## Sev-3 Playbooks

Standard playbooks per `runbooks/sync-conflict-resolution.md`, `runbooks/tag-graph-corruption.md`, etc.

## Communications

| Audience | Channel | Trigger |
|---|---|---|
| Internal — engineering | #incident-notes channel | every Sev-1/2 |
| Internal — exec | Sev-1 email + page | every Sev-1 |
| Tenant DPO | per-tenant DPO email + status page | within GDPR/PIPA window |
| End-user (Personal) | in-product banner | when individual user impact confirmed |
| Regulator | per-pack DPA contact | when threshold triggered |
| Public | status page (status.oyatie.dev) | every Sev-1/2 |

## Forensic Evidence Capture

Within 24h of any Sev-1:
- Audit-chain segment for affected window (Ed25519-sealed).
- Postgres point-in-time snapshot.
- Workflow event log (24h window).
- Prometheus + Grafana dashboard exports.
- Logs (Alloy / Loki).
- Stored in `evidence/incidents/<ulid>/`.

## Post-Mortem Template

| Field | Content |
|---|---|
| Incident ID | ULID |
| Severity | Sev-N |
| Start | timestamp |
| End | timestamp |
| Detection method | metric / report / chaos |
| Customer impact | tenants, users, BCs affected |
| Root cause | technical + organisational |
| Action items | ledger entries with due-by |
| Lessons learned | systemic patterns; ADR if needed |
| Audit-chain evidence pointer | ULID + URL |

## References

- NIST SP 800-61 Rev. 2.
- GDPR Arts. 33, 34.
- KR PIPA Art. 34.
- HIPAA §164.400 et seq.
- `microservices/notes/runbooks/*`.
- `microservices/notes/threat-model.md`.
