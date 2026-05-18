---
doc_class: IncidentResponse
title: Incident Response Playbook
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + axis-shorts + ops-legal
deciders: ops-sre-reliability, ops-security, axis-shorts, council-architecture, council-privacy, ops-legal
related_adrs: [ADR-0008, ADR-0028, ADR-0123, ADR-0126, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/shorts/threat-model.md
  - microservices/shorts/failure-modes.md
  - microservices/shorts/runbooks/
review_cadence: quarterly + on every Sev-1 or Sev-2 incident
doc_status: published
---

# Incident Response Playbook (shorts µservice)

## Purpose

Define classification, escalation, communication, regulatory-notification, and postmortem procedures for shorts incidents. Aligned with NIST SP 800-61 Rev. 2 + ICS / NIMS structure + EU DSA Art. 24 (transparency) + EU AI Act Art. 73 (serious-incident reporting) + GDPR Art. 33 (72h breach notification) + KR PIPC PIPA Art. 34 (24h+72h) + HIPAA §164.404-410 + DMCA repeat-infringer obligations.

## Severity Classification

| Sev | Definition | Detection | Examples |
|---|---|---|---|
| **Sev-1** | Hero-product is broken at scale OR security invariant breached OR regulatory deadline approaching | Auto-page on burn-rate or correctness invariant violation | Personal-tier video leaked via federation; DRM key compromise; mass false-positive moderation; pack-wide outage; cross-tenant video leak; minor-protection bypass; DMCA repeat-infringer policy failure |
| **Sev-2** | User-visible degradation; latency / availability budget heavy burn; reversible | Burn-rate fast-1h alert; classifier drift | Feed-load p95 above SLO; transcode queue backup; CDN POP outage; notification fanout delay; fingerprint matcher slow |
| **Sev-3** | Internal-only; user-invisible; planned-work-like | Slow-burn alert; ticket-burn | Cache hit-ratio low (still functional); Meilisearch index lag |
| **Sev-4** | Tracking-only; no immediate action | scheduled review | Trend compute interval extension; sound-of-the-week stale by 1h |

## Roles (ICS-inspired)

| Role | Owner during Sev-1 | Owner during Sev-2 |
|---|---|---|
| **Incident Commander** | ops-sre-reliability primary on-call | axis-shorts on-call |
| **Comms Lead** | gtm-customer-success on-call | gtm-customer-success on-call |
| **Tech Lead** | axis-shorts senior on-call | axis-shorts on-call |
| **Security Lead** | ops-security on-call | ops-security on-call only if security-touched |
| **Privacy Lead** | council-privacy on-call | council-privacy if PII / minor-protection touched |
| **Legal Lead** | ops-legal on-call | ops-legal if DMCA / DSA / DSAR / regulatory-notification needed |
| **Scribe** | rotating | rotating |

## Activation

### Auto-activation triggers

- Any Sev-1 alert fires (Prometheus `severity=critical`).
- Burn-rate fast-1h (14.4x) sustained > 5min on hero-product SLO.
- Correctness SLO `shorts-content-policy-enforcement-correctness` non-zero violation.
- Cross-context routing detected (`oya_shorts_dual_context_denied_total` > 0).
- Personal-tier federation attempt (`oya_shorts_personal_tier_federation_attempt_total` > 0).
- DRM key compromise indicator (`oya_shorts_drm_key_rotation_failure_total` > 0 with revoke-pending).
- Pack residency violation (`oya_shorts_pack_residency_violation_total` > 0).
- Minor-protection bypass attempt (`oya_shorts_minor_protection_bypass_attempt_total` > 0).
- DMCA repeat-infringer policy enforcement gap (audit dashboard red).

### Manual activation

- Any engineer can declare Sev-1 / Sev-2 via PagerDuty / Slack.
- Tenant-reported "service down" → check status; escalate if widespread.
- External notification: DPA / Ofcom / DPC / DMCA agent contact → Sev-1.

## Communication

### Internal

| Channel | Purpose | Sev |
|---|---|---|
| `#oya-shorts-incidents` Slack | Real-time coordination | Sev-1, Sev-2 |
| Zoom war-room | Voice for complex coordination | Sev-1 |
| Status page draft channel | Comms-Lead drafts external comms | Sev-1, Sev-2 |
| Incident ticket | Ground-truth timeline | All |

### External

| Audience | Channel | Cadence | Sev |
|---|---|---|---|
| Tenants | Status page + per-tenant Slack/email | Initial within 15min + update every 30min | Sev-1, Sev-2 |
| End-users | Tenant-mediated | Tenant decides | depends |
| Regulator (when triggered) | DPC / DPA / EU DSA / EU AI Act notified body / KR PIPC / HIPAA OCR / UK Ofcom / AU eSafety / US Copyright Office | per regulation | Sev-1 |

## Regulatory Notification Timelines

| Regulation | Trigger | Deadline | Owner |
|---|---|---|---|
| GDPR Art. 33 | Personal-data breach likely to result in risk | 72h to DPA | council-privacy + ops-legal |
| GDPR Art. 34 | High risk to data subjects | "without undue delay" to subjects | council-privacy + tenant-via-tenant-DPA |
| KR PIPA Art. 34 | Personal-data breach | 24h to PIPC + 72h to subjects | council-privacy + ops-legal |
| HIPAA §164.404-410 | Unsecured PHI breach | 60d to OCR + subjects + media (if > 500 subjects) | council-privacy + ops-legal + tenant-CE |
| EU DSA Art. 24 | Quarterly transparency report | quarterly | council-privacy + tenant-publishes |
| EU AI Act Art. 73 | Serious incident (mass false-positive moderation; classifier malfunction harming fundamental rights) | 15 days to market-surveillance authority | axis-shorts + council-privacy + ops-legal |
| EU AVMSD Art. 28b | Minor-protection failure (if egregious) | per Member State law | ops-legal + tenant-of-tenant |
| UK Online Safety Act 2023 | Illegal-content duty failure | per Ofcom direction | ops-legal |
| AU Online Safety Act 2021 | BOSE non-compliance | per Commissioner direction | ops-legal |
| LGPD Art. 48 | Personal-data breach with relevant risk | per ANPD direction | council-privacy + ops-legal |
| Singapore PDPA | Notifiable data breach | 72h to PDPC + affected | council-privacy + ops-legal |
| US state-level | Per-state breach laws (CCPA, NY SHIELD, etc.) | per-state | council-privacy + ops-legal |
| DMCA §512(c)(2) | Designated-agent contact failure | best-effort within 24h | ops-legal |
| DMCA §512(g) | Counter-notice processing | 10-14d business cycle | ops-legal |

## Common Incident Playbooks

### Sev-1: Cross-tenant video leak detected

1. IC declared; Sev-1 page; war-room.
2. Identify scope: which tenants, which videos, which time window.
3. Disable affected feed-render endpoints + CDN keys (cordon).
4. Engage council-privacy + ops-security; identify if RLS misconfig or Cedar bug.
5. Apply hotfix; verify cordoned endpoint behavior.
6. Begin Art. 33 / Art. 34 / KR PIPA Art. 34 / HIPAA breach-notification clock evaluation.
7. Postmortem ≤ 5 business days.

### Sev-1: Personal-tier video federation attempt

1. IC declared; Sev-1 page; war-room.
2. Identify if compile-time invariant was bypassed (would indicate Rust compiler bug or unsafe code) vs runtime guard caught it.
3. If unreachable (compile error path): verify metric is from runtime guard catching what should be impossible — investigate.
4. If reachable: HALT all federation-gateway egress; engage council-architecture + ops-security.
5. Forensic audit of any actual egress + recall via federation peer outbox-undo if available.
6. Postmortem with parallel ADR-0126 + ADR-SHORTS-* review.

### Sev-1: DRM key compromise indicator

1. IC declared; Sev-1 page; war-room with ops-security + cloud-secrets.
2. Identify scope: which key-system (Widevine / FairPlay / PlayReady); which per-content keys.
3. Trigger immediate key rotation per `runbooks/drm-key-rotation.md`.
4. Add affected per-content keys to revocation list.
5. Notify affected tenants (Premium-tier with DRM); estimate user impact.
6. Engage Widevine / FairPlay / PlayReady vendor for root-key validity.
7. If root-key compromised: full HSM rebuild; multi-week recovery.
8. Postmortem with cloud-secrets + ops-security.

### Sev-1: Mass false-positive moderation event

1. IC declared; Sev-1 page; war-room.
2. Identify classifier version + verdict pattern.
3. Trigger `runbooks/moderation-classifier-rollback.md` to previous golden-set-passing version.
4. Restore auto-hidden videos affected by false-positive verdicts via reversal worker.
5. Engage council-privacy + ops-legal: EU AI Act Art. 73 serious-incident assessment (15-day clock).
6. Per-affected-creator notification (via tenant-of-tenant).
7. Postmortem with axis-foundry-runtime model card review.

### Sev-1: Forged DMCA copyright-claim storm

1. IC declared; Sev-1 page; war-room.
2. Identify claimant pattern; rate-limit aggressively.
3. Engage ops-legal: DMCA §512(f) misrepresentation review.
4. Reverse auto-hides for verified-false claims.
5. Per-affected-creator counter-notice support.
6. If claimant verified malicious: ops-legal pursues §512(f) damages remedy.
7. Postmortem with ops-legal + content-moderation BC review.

### Sev-1: Minor-protection bypass attempt

1. IC declared; Sev-1 page; war-room.
2. Identify scope: which minor accounts, which protections bypassed.
3. Cordon affected accounts; restore parental-controls + chronological-only default.
4. Engage council-privacy: GDPR Art. 8 / COPPA / KR 청소년 보호법 / CA AB-2273 / UT SMRA breach evaluation.
5. Notify parents (via tenant-of-tenant DPA).
6. Postmortem with `parental-controls` + `age-gate` BC review.

### Sev-1: Pack residency violation (cross-pack video replication)

1. IC declared; Sev-1 page; war-room.
2. Identify which pack and which videos crossed.
3. Cordon the cross-pack data plane; halt replication.
4. council-privacy: GDPR Arts. 44-50 + KR PIPA Art. 28 + LGPD Art. 33 evaluation.
5. Move the violating data back to source-pack via DSR-cascade analog.
6. Postmortem with cell µservice + multi-region.md review.

### Sev-2: Transcode queue backup

1. Follow `runbooks/transcode-queue-backup.md`.
2. Communicate ETA to affected creators (tenant-mediated).
3. Promote to Sev-1 if sustained > 30 min affecting > 30 % of tenants.

### Sev-2: CDN POP failure

1. Cloudflare auto-failover to nearest healthy POP.
2. Status page update.
3. Promote to Sev-1 if all in-pack POPs down.

## Postmortem Requirements

| Sev | Postmortem deadline | Author | Reviewers |
|---|---|---|---|
| Sev-1 | 5 business days | Incident Commander + Tech Lead | council-architecture, ops-security, council-privacy, ops-legal, axis-shorts lead |
| Sev-2 | 10 business days | Tech Lead | axis-shorts lead, ops-sre-reliability |
| Sev-3+ | Bi-weekly aggregate | ops-sre-reliability | axis-shorts lead |

Required postmortem sections: Timeline, Root Cause, Contributing Factors, What Went Well, What Went Poorly, Action Items, Customer Impact, Regulatory Notification Status.

Stored at `microservices/shorts/evidence/postmortems/PM-<unix_ts>.md`.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Sev-1 cross-tenant leak tabletop | Quarterly | 2026-Q3 (scheduled) |
| Sev-1 DRM key rotation drill | Per rotation cadence (90d) | rolling |
| Sev-1 classifier-rollback drill | Quarterly | 2026-Q3 (scheduled) |
| Sev-1 pack-residency cordon drill | Annually | 2026-Q4 (scheduled) |
| Sev-1 DMCA forged-claim-storm tabletop | Quarterly | 2026-Q3 (scheduled) |
| Sev-1 minor-protection bypass tabletop | Quarterly | 2026-Q3 (scheduled) |
| Sev-2 transcode queue backup drill | Quarterly | 2026-Q3 (scheduled) |
| Sev-2 CDN POP failover drill | Quarterly | 2026-Q3 (scheduled) |

## References

- NIST SP 800-61 Rev. 2.
- ICS / NIMS structure.
- GDPR Art. 33, 34.
- KR PIPA Art. 34.
- HIPAA §164.404-410.
- EU DSA Regulation 2022/2065 Art. 24.
- EU AI Act Regulation 2024/1689 Art. 73.
- EU AVMSD 2018/1808 Art. 28b.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021.
- LGPD Art. 48.
- DMCA Title II 17 USC §512.
- `microservices/shorts/threat-model.md`.
- `microservices/shorts/failure-modes.md`.
- `microservices/shorts/runbooks/`.
- `microservices/social/incident-response.md` (sibling pattern).
