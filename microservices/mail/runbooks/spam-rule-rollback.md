---
doc_class: Runbook
title: Spam classifier rule rollback (false-positive surge or model drift)
microservice: mail
severity: "Sev-2 (false-positive > 1%) / Sev-1 (false-positive > 10% or PII-rule misfire)"
status: Accepted
owner_team: axis-mail + ops-deliverability + axis-trust-safety
date: 2026-05-17
related_artifacts:
  - microservices/mail/failure-modes.md (FM-SP-01 false-positive surge, FM-SP-02 ML model drift)
  - microservices/mail/threat-model.md (T-D-03 spam-rule poisoning)
  - microservices/mail/contracts/asyncapi.yaml §"AbuseVerdict"
  - microservices/mail/capabilities/T1-assist.yaml (smart classifier as T1)
  - ADR-0133 cross-tenant mail-server pattern
  - EU AI Act Annex III §3 (employment/HR communication classification — high-risk where applicable)
doc_status: published
---

# Runbook: Spam classifier rule rollback

## Trigger

ANY of:

1. `oya_mail_inbound_spam_false_positive_rate_5m` > 1% sustained ≥ 10 min (page).
2. User-reported `not-spam` feedback rate > 500/h system-wide (`oya_mail_user_feedback_not_spam_total[1h]`).
3. Specific tenant complaint > 10 false-positives in 1h.
4. ML model emission drift detected (PSI > 0.25 between live distribution and training baseline).
5. Spam rule deployment within last 30 min preceded the FP surge.
6. PII / business-critical patterns being flagged (e.g., legitimate calendar invites quarantined; medical-record emails quarantined under HIPAA pack).

## Severity

| Condition | Severity |
|---|---|
| FP rate 1-10%; single tenant | Sev-2 |
| FP rate > 10% across tenants | Sev-1 |
| HIPAA-pack PHI mail being quarantined (false-positive on medical content) | Sev-1 (HIPAA §164.312 access integrity at risk) |
| Phishing/abuse false-NEGATIVE surge (spam being delivered as ham) | Sev-2 (deliverability + tenant safety) |
| Model-drift detected without acute FP | Sev-3 |

## Pre-checks

| # | Check | Command / source |
|---|---|---|
| 1 | Identify recent classifier changes: rule deployments + model versions | `git log -p microservices/mail/src/crates/oya-mail-inbound-smtp-adapter/src/abuse/rules/ --since=24h` |
| 2 | Live FP/FN rates | `oya_mail_inbound_spam_false_positive_rate_5m`, `oya_mail_inbound_spam_false_negative_rate_5m` |
| 3 | Which rule? | `oya_mail_inbound_spam_rule_fire_total{rule_id=...}` topk(10) over recent window |
| 4 | Which model version? | `oya_mail_spam_classifier_model_version` gauge |
| 5 | Tenant skew: single-tenant or systemic? | breakdown by `tenant_id` |
| 6 | User feedback skew | `oya_mail_user_feedback_not_spam_total[1h]` by `tenant_id` |
| 7 | EU AI Act Annex III applicability: is this an HR / employment / educational mail surface where classification is "high-risk"? | check tenant `data_class_overlay` + pack |

## Recovery Path A — Rule-based misfire (Rspamd or local rule)

Cause: A newly deployed deterministic rule pattern (regex/SPF/DMARC/blocklist) is over-matching legitimate mail.

| Step | Action | Time |
|---|---|---|
| 1 | Identify offending rule from `oya_mail_inbound_spam_rule_fire_total` topk + cross-ref recent commits in `src/crates/oya-mail-inbound-smtp-adapter/src/abuse/rules/` | ≤ 5 min |
| 2 | Disable rule WITHOUT redeploy: `kubectl exec -n mail <pod> -- oya-mail-cli abuse rule-disable --rule-id=<id> --reason="<rfc>"`. The disable propagates via in-memory config; effective ≤ 60s. | ≤ 5 min |
| 3 | Verify FP rate trending down: `oya_mail_inbound_spam_false_positive_rate_5m` falling | ≤ 15 min |
| 4 | Release affected mail from quarantine for ALL tenants impacted: `oya-mail-cli quarantine release-by-rule --rule-id=<id> --within=2h`. Audit-emit `SpamQuarantineReleasedBulk`. | ≤ 10 min |
| 5 | Notify users: per-mailbox notification "We released N messages from your spam folder that were incorrectly classified between <start> and <end>". | ≤ 30 min |
| 6 | Open Issue to fix the rule properly (regex too broad? bad blocklist source? competing legitimate signal?) | ≤ 1 h to triage |
| 7 | Rolling-deploy the fix once tested | ≤ 4 h |

## Recovery Path B — ML model regression (drifted or bad training data)

Cause: Newly deployed classifier model version has worse FP/FN trade-off than prior; PSI shift detected; user feedback surging.

| Step | Action | Time |
|---|---|---|
| 1 | Roll back model version to prior: `kubectl exec <pod> -- oya-mail-cli classifier rollback-model --to-version=<prior>`. Propagates via OCI registry pull + pod rolling-restart. | ≤ 10 min |
| 2 | Verify model version live: `oya_mail_spam_classifier_model_version` = prior version | ≤ 5 min |
| 3 | Release quarantined mail from the affected window (Path A Step 4) | ≤ 10 min |
| 4 | Engage axis-trust-safety + foundry-runtime team to root-cause the model regression — was training data poisoned? was feature distribution shift undetected? | ≤ 1 d |
| 5 | EU AI Act compliance gate: re-train + re-validate per `/specs/ai-act-conformance.json` BEFORE re-deploying the regressed model line; if HR/employment surface, conformity assessment required | per validation |
| 6 | Tenant notification per pack-eu / pack-us (CAN-SPAM Act §5(a)(5) labeling does NOT apply to spam filters; this is internal-classification correction). | per ops-comms |

## Recovery Path C — Rule poisoning (T-D-03; adversarial input)

Cause: External actor influenced the classifier via crafted feedback (vote-bombing not-spam to train future model) or by gaming a blocklist source.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security + axis-trust-safety | immediate |
| 2 | Suspend user-feedback ingestion into the training pipeline: `oya-mail-cli classifier suspend-feedback-pipeline` | ≤ 5 min |
| 3 | Run Path A or Path B for live mitigation | per FP surge |
| 4 | Forensic: identify the poisoning signature; quarantine the contributing tenant or user(s) per Cedar policy `forbid (principal, action == Action::"submit_feedback", resource) when {principal in suspended_users};` | ≤ 4 h |
| 5 | Re-train model on cleaned baseline; gate re-deploy through EU AI Act validation if applicable | per validation |
| 6 | Update threat-model T-D-03 with new attack signature | post-incident |

## Recovery Path D — Cross-tenant rule scope leak

Cause: A rule added for tenant X accidentally affects all tenants (T-I-04 cross-tenant signal leak).

| Step | Action | Time |
|---|---|---|
| 1 | Disable rule globally (Path A Step 2) | ≤ 5 min |
| 2 | Engage ops-security: this is a Cedar policy authoring failure; review tenant scope assertion in rule deploy pipeline | immediate |
| 3 | Audit: did any tenant's mail get classified by another tenant's rule? Emit `mail_cross_tenant_classifier_leak_total` | ≤ 1 h |
| 4 | Per pack notification (GDPR Art. 33 if EU-pack tenant affected; KR PIPA Art. 34 if KR-pack) | per pack |
| 5 | Fix: rules must declare `tenant_scope: ["<tenant-id>"]` OR `tenant_scope: ["*"]` (explicit) at authoring time; CI lane refuses missing field | within 1 wk |

## Verification

After completion:
- `oya_mail_inbound_spam_false_positive_rate_5m` < 0.1% for ≥ 30 min.
- `oya_mail_inbound_spam_false_negative_rate_5m` within target (< 1%).
- `oya_mail_spam_classifier_model_version` matches expected (rolled-back) version.
- User feedback rate normalised.
- No tenant complaints in last 1h.
- Audit-chain seals on every rule-disable + quarantine-release.
- Postmortem assigned within 5 business days.

## Post-incident updates

- Tune `oya-mail-inbound-spam-canary-cohort` lane: every new rule + model must pass shadow-mode evaluation for ≥ 24h before promotion.
- Per `competitor-parity-matrix.md` update: how does this incident affect parity claims vs Gmail/Proton/Outlook?
- Per EU AI Act: maintain post-market monitoring log per Art. 72 if classifier is in scope for tenant.
- Re-test spam-detection p99 ≤ 100ms (per `slos/spam-classification-latency.openslo.yaml`).

## References

- RFC 7208 (SPF), RFC 7489 (DMARC), RFC 8617 (ARC) — context for header-based rules
- Rspamd documentation — `https://rspamd.com/doc/`
- SpamAssassin docs — `https://spamassassin.apache.org/`
- EU AI Act (Regulation (EU) 2024/1689) Art. 72 (post-market monitoring); Annex III §3 (high-risk classifier domains)
- M3AAWG Anti-Phishing Best Practices — `https://www.m3aawg.org`
- CAN-SPAM Act 15 USC §7701 et seq (labeling does not bind classifiers)
- `microservices/mail/failure-modes.md` FM-SP-01, FM-SP-02
- `microservices/mail/threat-model.md` T-D-03, T-I-04
- `microservices/mail/capabilities/T1-assist.yaml` (autonomy tier; classifier rationale)
