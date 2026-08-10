---
doc_class: Runbook
title: DLP quarantine release (review + release flow for held outbound mail)
microservice: mail
severity: "Sev-3 (single quarantine) / Sev-2 (tenant queue stall) / Sev-1 (PHI/PII regulator-scope leak averted)"
status: Accepted
owner_team: axis-mail + ops-deliverability + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/mail/threat-model.md (T-I-03 DLP-class data exfiltration via outbound, T-E-02 DLP-rule poisoning)
  - microservices/mail/dpia.md (R-04 DLP outbound-scanning)
  - microservices/mail/policy/data-residency.md
  - microservices/mail/contracts/openapi.yaml §"/v1/dlp/quarantine"
  - comms/mail/capabilities/T2-auto.yaml (DLP enforcement as T2)
  - ADR-0140 (retired per ADR-0145) Cedar policy enforcement
doc_status: published
---

# Runbook: DLP quarantine release

## Purpose

The DLP (data-loss-prevention) BC scans outbound mail for tenant-defined sensitive patterns (PHI/PII/PCI/IP/regulatory codes). Matches are held in tenant-scoped quarantine until reviewed. This runbook covers the review-and-release flow + the failure modes around it (false-positive surge; reviewer queue stall; regulator-scope leak averted).

DLP scan is a `T2-auto` capability (per `capabilities/T2-auto.yaml`) — autonomous block by default, with explicit reviewer override.

## Trigger

| Trigger | Severity | Reviewer |
|---|---|---|
| User sends an outbound mail; DLP rule matches; quarantine held | Sev-3 (planned) | tenant DLP reviewer |
| Tenant quarantine queue depth > 50 messages OR oldest > 2 h | Sev-2 | tenant DLP reviewer + axis-mail oncall |
| Phantom-PHI: HIPAA-pack tenant tries to send PHI to non-BAA recipient | Sev-1 | council-privacy + tenant compliance officer (mandatory four-eyes per `policy/dual-context-isolation.md`) |
| DLP rule appears mis-tuned (FP rate > 5%) | Sev-2 | axis-trust-safety + tenant DLP reviewer |
| Adversary attempts release-without-review bypass | Sev-1 | ops-security |

## Pre-checks

| # | Check | Command / source |
|---|---|---|
| 1 | Confirm quarantine record exists + status: `held` | `oya-mail-cli dlp quarantine show --quarantine-id=<id>` |
| 2 | Reviewer identity + Cedar entitlement | `tenant has DlpReviewer` entitlement; checked at API layer |
| 3 | Match rule + match severity | from quarantine record metadata |
| 4 | Pack residency + applicable regulator (HIPAA / GDPR / PCI / PIPA) | from tenant pack overlay |
| 5 | Four-eyes requirement: PHI-class or PCI-class match requires two reviewers per `policy/data-residency.md` overlay | per pack |
| 6 | Sender + recipient context | from envelope; flag if recipient is external (non-tenant-domain) |

## Steps — Standard release (single message, non-PHI)

| Step | Action | Time |
|---|---|---|
| 1 | Tenant DLP reviewer opens quarantine record in tenant DLP UI (REST: `GET /v1/dlp/quarantine/{id}`) | ≤ 1 min |
| 2 | Reviewer inspects message content (Cedar `read_quarantined_message` permit) + match rationale (which rule + match span) | ≤ 5 min |
| 3 | Decision: `release` (send), `redact-and-release` (mask matched span; send), `discard` (delete; do not send), `escalate` (forward to council-privacy / compliance officer for second review) | ≤ 5 min |
| 4 | On `release`: `POST /v1/dlp/quarantine/{id}/release` with reviewer identity + Ed25519 signature; envelope emitted to outbound-smtp queue; audit-emit `DlpQuarantineReleased{quarantine_id, reviewer_id, decision, decided_at}` | ≤ 2 min |
| 5 | On `redact-and-release`: reviewer provides redaction spans (UTF-8 byte-range list); DLP engine generates redacted body; emits `DlpQuarantineRedactedAndReleased{...,redaction_spans}`; original body archived for 1y under reviewer + compliance officer Cedar read-only scope | ≤ 5 min |
| 6 | On `discard`: `POST /v1/dlp/quarantine/{id}/discard`; emit `DlpQuarantineDiscarded`; sender notified per `dpia.md` R-04 | ≤ 2 min |
| 7 | On `escalate`: forwarded to escalation queue; original quarantine remains `held`; track via `MailDlpEscalated` | ≤ 1 min |

## Steps — PHI / PCI / regulatory four-eyes release (Sev-1 or pack-mandatory)

| Step | Action | Time |
|---|---|---|
| 1 | First reviewer (tenant DLP reviewer) opens quarantine, inspects, votes `release-pending-second-reviewer` | ≤ 5 min |
| 2 | Second reviewer (council-privacy delegate or tenant compliance officer per pack) opens quarantine within 1h window; if window exceeded, first vote expires (re-vote required) | ≤ 1 h |
| 3 | If both votes `release`: Ed25519 co-sign sealed; envelope released; audit-emit `DlpQuarantineFourEyesReleased{quarantine_id, reviewer_a_id, reviewer_b_id}` | ≤ 2 min |
| 4 | If votes disagree: quarantine status becomes `disputed`; engage tenant ops-legal; ultimate decision binding by tenant compliance officer | per dispute |
| 5 | If PHI to non-BAA recipient (HIPAA pack): default action is `discard`; release requires council-privacy explicit ADR-shaped justification | per HIPAA |

## Steps — Bulk release (after false-positive surge fix)

Cause: A DLP rule was over-matching; FP fix deployed; backlog of incorrectly-quarantined messages needs release without per-message review.

| Step | Action | Time |
|---|---|---|
| 1 | Confirm fix is live (rule disabled or refined; `oya_mail_dlp_match_total{rule_id=<id>}` flat at 0 for ≥ 30 min) | ≤ 30 min |
| 2 | Operator authority: requires axis-mail oncall + council-privacy approval (NOT tenant-scoped action) | ≤ 30 min |
| 3 | Identify scope: `oya-mail-cli dlp quarantine list --rule-id=<id> --status=held --since=<window>`. Verify only messages within the FP window are affected. | ≤ 5 min |
| 4 | Apply bulk release: `oya-mail-cli dlp quarantine bulk-release --rule-id=<id> --since=<window> --reason="<rfc-link>"` | ≤ 10 min |
| 5 | Emit per-message `DlpQuarantineReleasedBulk` events (one per release, with bulk-job-id linkage) | automatic |
| 6 | Sender notification: each affected user gets a single summary email listing their released messages + reason | ≤ 1 h |
| 7 | Tenant admin notification: dashboard banner with bulk-release summary | ≤ 5 min |

## Steps — Reviewer queue stall

Cause: Quarantine queue growing because tenant reviewer is unavailable or rule volume exceeds review capacity.

| Step | Action | Time |
|---|---|---|
| 1 | Alert tenant admin: `oya-mail-cli dlp queue-stall-notify --tenant=<t>` (auto-fired on queue-depth threshold) | automatic |
| 2 | Tenant admin assigns additional reviewer or escalates per their SOP | per tenant |
| 3 | If queue > 200 messages OR > 24h oldest: axis-mail oncall reviews per `policy/data-residency.md` (with tenant compliance officer co-sign) | per tenant |
| 4 | Long-term: tune DLP rule sensitivity for the tenant; engage axis-trust-safety | per tenant |

## Steps — Cross-tenant DLP rule leak (T-I-04)

Cause: A DLP rule authored for tenant X is firing on tenant Y's mail.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security |
| 2 | Quarantine the affected rule globally; release any tenant Y matches (Path "Bulk release") |
| 3 | Audit: which Cedar tenant-scope assertion failed? — review CI lane `dlp-rule-tenant-scope-conformance` |
| 4 | Per pack: GDPR Art. 33 (EU pack); KR PIPA Art. 34; HIPAA §164.410 — notification if PHI surface affected |
| 5 | Postmortem within 24h |

## Verification

After release decisions:
- `oya_mail_dlp_quarantine_depth` ≤ target.
- `oya_mail_dlp_quarantine_age_p99_seconds` ≤ 7200 (2h target).
- All releases audit-chained; `oya-mail-cli audit verify --event=DlpQuarantineReleased --quarantine=<id>` returns ✅.
- Sender notified per workflow.
- Four-eyes seals present where required.
- `mail_dlp_release_without_review_total` = 0 (no bypass).
- If PHI: HIPAA audit log entry persisted ≥ 6y per §164.316(b)(2).

## Post-incident updates

- Tune DLP rules (per tenant + per pack); update `policy/data-residency.md` overlays as needed.
- Update `failure-modes.md` if new FM identified.
- Update `dpia.md` R-04 if scanning scope changed.
- EU AI Act compliance: if DLP uses ML classification, log under Art. 72 post-market monitoring.

## References

- HIPAA Security Rule §164.312, §164.502, §164.504(e), §164.316(b)(2), §164.410
- GDPR Arts. 5, 25, 32, 33; ePrivacy Directive 2002/58/EC Art. 5(3)
- PCI DSS v4.0 Req. 3 (protect stored PAN), Req. 4 (encrypt in transit)
- KR PIPA Art. 28, 29, 34
- CAN-SPAM Act 15 USC §7701 (sender-suppression; outbound)
- ADR-0140 Cedar policy enforcement
- `microservices/mail/threat-model.md` T-I-03, T-E-02
- `microservices/mail/dpia.md` R-04
- `comms/mail/capabilities/T2-auto.yaml`
- M3AAWG Outbound Best Common Practices — `https://www.m3aawg.org`
