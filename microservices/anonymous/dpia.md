---
doc_class: DPIA
template_id: TPL-DPIA
microservice: anonymous
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-anonymous
deciders: council-privacy, ops-security, axis-anonymous, council-architecture
methodology: GDPR Art. 35 DPIA template (EDPB Guidelines 4/2019) + ICO DPIA framework + CNIL PIA software
related_adrs: [ADR-0008, ADR-0117, ADR-0135, ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0003, ADR-ANON-0004, ADR-ANON-0005, ADR-ANON-0006, ADR-ANON-0007]
related_specs: [/specs/microservices/anonymous.json]
review_cadence: annually + on substrate change + on every new pack onboarding
doc_status: published
---

# DPIA: anonymous µservice

## Step 1 — Necessity of the DPIA

GDPR Art. 35 + EDPB Guidelines 4/2019 require a DPIA for processing likely to result in high risk to the rights and freedoms of natural persons. The anonymous µservice processes (a) personal data via affinity-attestation linkage, (b) pseudonymous data (GDPR Art. 11 + Recital 26), (c) behavioural content data, (d) systematic monitoring (content-moderation classifier — EU AI Act limited risk), (e) sensitive data scenarios (CSAM-suspect content; minor users; legal-process disclosure). **DPIA mandatory.**

## Step 2 — Processing description

| Aspect | Description |
|---|---|
| Nature | Pseudonymous-and-affinity-bound posting (Sidechat / YikYak / Whisper / Blind-class) with cryptographic identity-correlation refusal |
| Scope | All 11 oyatie regulatory packs; multi-tenant; per-tenant retention tier 30/60/90 days |
| Context | End-users post under per-channel-rotated handles bound to a verified affinity (employer/edu/geo/workspace). Platform CANNOT link post → user except via court order. |
| Purpose | Enable anonymous community discussion with structural privacy guarantees that competitors cannot offer |
| Data categories | (a) PII_IDENTIFYING at affinity-attestation issuance (real-name → blinded credential issuance; one-way blinded); (b) BEHAVIORAL_USER_CONTENT (post body); (c) BEHAVIORAL_TENANT_PRODUCT (vote, comment, hashtag); (d) AUDIT (audit-chain records); (e) blinded credential commitments (INTERNAL_ONLY cryptographic) |
| Data subjects | End-users (B2C-anonymous tier); workforce-users (Blind-class employer-bound); university-students (Sidechat-class); geo-residents (YikYak-class hyperlocal) |
| Recipients | (1) The platform itself (post + vote storage; aggregate metrics); (2) Content moderators (via abuse-report flow + classifier verdict); (3) Law enforcement (via legal-process disclosure ONLY); (4) Affinity-IdP issuer at attestation time (verifies real identity; receives nothing about subsequent posts) |
| Transfers | Per-pack residency pinned per `policy/data-residency.md`; cross-border transfer per pack policy (GDPR Arts. 44-50; KR PIPA Art. 28; APPI Art. 24) |
| Retention | Default 30 days; tenant-selectable 30 / 60 / 90; hard-delete + audit-chain tombstone within p99 ≤ 5s |
| Automated decisions | EU AI Act limited-risk content-moderation classifier (auto-hide above confidence threshold; appeal-workflow opens) — Art. 50 transparency obligation; NOT Art. 22 individual-significant-effect (users are anonymous) |

## Step 3 — Lawful basis (GDPR Art. 6 + Art. 9 where relevant)

| Operation | Lawful basis | Justification |
|---|---|---|
| Affinity-IdP linkage (real-name → blinded credential at issuance) | Art. 6(1)(b) contract performance + Art. 6(1)(a) consent | User has signed up to tenant; tenant offers anonymous tier; user explicit consent at first attestation |
| Post creation (under blinded handle) | Art. 6(1)(b) | Contract performance under anonymous tier — Art. 11 applies (data not requiring identification; controller's obligations under Arts. 15-22 reduced) |
| Content-moderation classifier (limited-risk per EU AI Act) | Art. 6(1)(c) (legal obligation — UK OSA, AU OSA, DE NetzDG) + Art. 6(1)(f) (legitimate interest in safety) | Balancing test: anonymous tier without moderation creates safety harms (Burnbook precedent); LIA documented in Appendix A |
| Legal-process disclosure | Art. 6(1)(c) legal obligation | Court order + Cedar gate + dual-control |
| Audit-chain record | Art. 6(1)(c) legal obligation + Art. 6(1)(f) legitimate interest | Accountability under Art. 5(2) |
| Special-category data (Art. 9) | Not knowingly processed; if user posts SCD voluntarily into post body, Art. 9(2)(e) "manifestly made public by data subject" + content-moderation removal where harmful |

## Step 4 — Necessity and proportionality

| Principle | How satisfied |
|---|---|
| Lawfulness, fairness, transparency (Art. 5(1)(a)) | Pack-aware ToS + Art. 11 + Recital 26 explicit disclosure; UI advisory at signup explains the cryptographic blinding |
| Purpose limitation (Art. 5(1)(b)) | Posts retained ONLY for the configured retention tier; audit-chain retained for accountability + legal-process; legal-process disclosure only on court order |
| Data minimisation (Art. 5(1)(c)) | Platform stores blinded commitment only; affinity-IdP stores user identity; legal_process_disclosure_view JOIN is the ONLY bridge; minimisation is structural |
| Accuracy (Art. 5(1)(d)) | Posts immutable after edit-window; user can hard-delete |
| Storage limitation (Art. 5(1)(e)) | 30 / 60 / 90 day retention tier; hard-delete propagation p99 ≤ 5s |
| Integrity + confidentiality (Art. 5(1)(f)) | TLS 1.3 + at-rest encryption (tenant-DEK) + RLS + Cedar |
| Accountability (Art. 5(2)) | Audit-chain seal on every state change |

## Step 5 — Risk identification (per LINDDUN + GDPR risk catalogue)

| Risk-ID | Risk to data subject | Likelihood | Severity | Mitigation status |
|---|---|---|---|---|
| R-01 | De-anonymization via internal DB JOIN (insider) | Medium | Severe (anonymity defeated) | Mitigated via I1 + DB GRANT separation + LEAN lanes (threat-model T-I-01) |
| R-02 | De-anonymization via timing-correlation | Medium | High | Mitigated best-effort (T-I-02); client SDK advisory; metric coarsening |
| R-03 | De-anonymization via stylometry | Medium | High | Mitigated best-effort (T-I-03); per-channel rotation + user-facing advisory |
| R-04 | De-anonymization via IP-correlation | Medium | High | Mitigated best-effort; IP not retained > 15min; Tor advisory for highest assurance |
| R-05 | Affinity-attestation issuer collusion | Low | Severe | Mitigated via blind-signature (T-I-10); issuer never sees post |
| R-06 | Push-notification real-name leak | Medium | High | Mitigated (T-I-04); opaque-handle-only |
| R-07 | Server-side log accidentally captures user_id | Medium | Severe | Mitigated (T-I-06); structured log schema + LEAN lane |
| R-08 | Third-party analytics SDK behavioural-leak | Critical (without I4) | Severe | I4 hard refusal at build (T-L-05) |
| R-09 | Affinity small-cardinality de-anonymization | Medium | High | Mitigated via k-anonymity floor + ADR-ANON-0007 |
| R-10 | Legal-process disclosure misuse (single-approver) | Low | Severe | Mitigated via dual-control + Cedar gate (T-E-03) |
| R-11 | Audit-chain corruption (loss of accountability) | Low | Severe | Mitigated via Merkle + Ed25519 seal per ADR-0028 |
| R-12 | Minor (under-13) signup despite COPPA ban | Medium | Severe | Mitigated via age-gate (FR-14) + UI hard-refusal + parental-consent token where applicable |
| R-13 | EU AI Act Art. 50 transparency omission | Medium | High | Mitigated via classifier output schema + UI label |
| R-14 | Content-moderation classifier bias against minority affinity | Medium | High | Mitigated via per-release golden-set eval + 4/5-rule disparity audit per `capabilities/T2-auto.yaml` |
| R-15 | Cross-pack transfer without SCC | Low | Severe | Pack residency pinned; cross-pack refused at code level for Personal-tier |
| R-16 | Tenant-administrator extending retention beyond regulatory ceiling | Medium | High | Pack-policy ceiling hard-enforced |
| R-17 | NCMEC CyberTipline missed report on confirmed CSAM | Low | Severe (US 18 USC §2258A violation) | Reporter queue + 48h SLA + dedicated runbook Path E |
| R-18 | Anonymous-DM plaintext leak via misconfiguration | Low | Severe (I6 violated) | Mitigated via MLS + LEAN lane (T-I-11) |
| R-19 | Hard-delete propagation slips below 100% correctness | Medium | High | Mitigated via dedicated SLO `hard-delete-propagation-correctness` 100% target |
| R-20 | Legal-process user-notification suppressed without lawful gag order | Low | Severe | Mitigated via gag-order documentary evidence required + audit-chain seal + transparency-report numerical inclusion |

## Step 6 — Measures to address risks (cross-reference)

(See threat-model.md mitigation columns + risk catalogue above. Per-risk runbook + per-risk code path link below.)

| Risk-ID | Code measure | Policy measure | Runbook |
|---|---|---|---|
| R-01 | DB GRANT separation; LEAN lane `oya-check-blinding-column-isolation` | `policy/legal-process-disclosure.cedar` | `runbooks/anonymity-leak-incident-response.md` (P0) |
| R-02 | Per-session credential TTL | n/a | `runbooks/anonymity-leak-incident-response.md` |
| R-04 | IP retention ≤ 15min; LEAN lane `oya-check-ip-retention-bounds` | n/a | n/a |
| R-05 | Blind-signature library audit; KAT vectors in CI | n/a | n/a |
| R-06 | Notification schema review + LEAN lane | n/a | n/a |
| R-07 | Structured-log schema + LEAN lane `oya-check-log-schema-no-user-id` | n/a | n/a |
| R-08 | LEAN lane `oya-check-third-party-tracker-refused` | n/a | n/a |
| R-09 | k-anonymity floor enforcement (ADR-ANON-0007) | `policy/affinity-attestation-verification.md` | n/a |
| R-10 | Cedar dual-control + audit-chain seal | `policy/legal-process-disclosure.cedar` | `runbooks/legal-process-court-order-receipt.md` |
| R-11 | Bominal ADR-0028 Merkle + Ed25519 | n/a | (audit-chain µservice runbook) |
| R-12 | Age-gate hard refusal | n/a | n/a |
| R-13 | EU AI Act Art. 50 label in classifier output schema | n/a | n/a |
| R-14 | Golden-set eval per release + 4/5-rule audit | `capabilities/T2-auto.yaml` | `runbooks/abuse-classifier-rollback.md` |
| R-15 | Pack residency at code | `policy/data-residency.md` | n/a |
| R-16 | Pack ceiling enforced in retention worker | `policy/data-residency.md` | n/a |
| R-17 | NCMEC reporter queue + 48h SLA | n/a | `runbooks/legal-process-court-order-receipt.md` Path E |
| R-18 | MLS + LEAN lane `oya-check-e2e-no-plaintext-server-state` | n/a | n/a |
| R-19 | Hard-delete worker correctness SLO | n/a | `runbooks/hard-delete-tombstone-corruption.md` |
| R-20 | Gag-order doc + audit-chain seal | `policy/legal-process-disclosure.cedar` | `runbooks/legal-process-court-order-receipt.md` |

## Step 7 — Residual risk acceptance

After mitigations:

- **Residual high-risk items:** R-02, R-03, R-04 (timing / stylometry / IP correlation are partial mitigations only; threat actors at nation-state level may still de-anonymize through external correlation).
- **Residual acceptance:** council-privacy + ops-security accept residual high-risk items with user-facing advisory in client SDK explaining the bounds.
- **Documented Tor-as-recommendation:** the platform recommends Tor or equivalent network anonymity for users whose threat model includes nation-state correlation; the platform does NOT promise network-anonymity, only application-anonymity.

## Step 8 — Pack-specific addenda

### pack-eu

- GDPR Art. 35 DPIA mandatory; this document satisfies.
- Art. 11 + Recital 26 explicitly invoked — controller's obligations under Arts. 15-22 reduced where data not requiring identification.
- Art. 22 NOT triggered (users are anonymous; no individual significant-effect decision).
- EU DSA Arts. 14/16/17/20/24/27/28 obligations: per-tenant ToS disclosure; statement-of-reasons per verdict; appeal workflow 7-day SLA; transparency report quarterly.

### pack-kr

- KR PIPA Art. 24-2 alternative pseudonymous processing — this is the canonical use case; PIPC briefing required at first KR tenant onboarding.
- 통신비밀보호법 (Communications Secrecy Protection Act) Arts. 5, 9, 9-2 — anonymous-DM falls under secrecy-of-communications protection; legal-process disclosure requires court order under Art. 9.
- 청소년 보호법 (Youth Protection Act) — pack-kr age threshold 14; minor signup requires parental consent.

### pack-us

- First Amendment anonymous-speech doctrine (Talley v. California 1960, McIntyre v. Ohio 1995) protects anonymous posting; legal-process disclosure must overcome the Doe v. Cahill / Krinsky v. Doe heightened standard.
- Section 230 CDA protects platform from liability for user content.
- COPPA 15 USC §6501 + 16 CFR §312 — under-13 ban absolute.
- State anti-doxxing (CA Civ. Code §1708.7; NY CRL §50/§51; IL 720 ILCS 5/26.5) — posts that doxx others triggered for moderation regardless of platform speech protection.

### pack-uk

- UK OSA 2023 — Ofcom oversight; illegal-content priority detection mandatory; covered service category.
- UK IPA 2016 §57 — legal-process disclosure framework (interception, equipment interference, communications data acquisition).
- UK DPA 2018 + UK GDPR.

### pack-jp

- 通信の秘密 (Constitutional Art. 21 secrecy of communications) — anonymous-DM falls under constitutional protection; legal-process disclosure requires court order.
- APPI Arts. 17/18/20/21/23/24/26-2/27.

## Step 9 — Consultation

- Council-privacy review: 2026-05-17 (this document).
- Ops-security review: 2026-05-17.
- Council-architecture review: 2026-05-17.
- Per-pack DPA briefing at first tenant onboarding in each pack.
- ICO consultation for UK pack: scheduled per Art. 36 prior consultation if residual risk threshold exceeded.

## Appendix A — Legitimate Interest Assessment (LIA) for content-moderation classifier

| Test | Outcome |
|---|---|
| Purpose test | Lawful (Art. 6(1)(f)); legitimate interest in user safety + abuse prevention; aligned with UK OSA, AU OSA, DE NetzDG duties |
| Necessity test | Necessary to achieve safety at scale; manual-only moderation does not scale to feed-render volume |
| Balancing test | Limited-risk classifier; no Art. 22 trigger (anonymous users; no individual significant-effect decision); transparency obligation satisfied via Art. 50 label; appeal workflow per EU DSA Art. 20 within 7 days |
| Conclusion | LIA passes; legitimate interest is a lawful basis for content-moderation classifier processing |
