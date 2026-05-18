---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-network, council-architecture, ops-compliance
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0133, ADR-0134, ADR-0140]
related_artifacts:
  - microservices/network/threat-model.md
  - microservices/network/dpia.md
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/policy/data-residency.md
  - microservices/network/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (network µservice)

## Purpose

Canonical control-to-framework mapping for the `network` Professional µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / EU AI Act notified body / EU DSA Coordinator / EEOC examiner / NYC DCWP Local Law 144 auditor / KR PIPC / KR Labor Standards inspector / HIPAA OCR / DPDP Board India / LGPD ANPD / UAE Data Office / SDAIA) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact. The employment-context posture of `network` makes the *labor + anti-discrimination + automated-decision* family central; this mapping calls those out explicitly.

## Enforced Frameworks (every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.5 | Accountability for performance | Per-µservice SLO + on-call | `slos/feed-render-latency.openslo.yaml` + `incident-response.md` |
| CC3.1 | Risk identification | Threat model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per ADR + IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; four-eyes disclosure; sybil-and-spam-detector on connection-request | `policy/professional-context-isolation.md` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLO | `/registry/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 + audit-chain µservice |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT | `policy/*.cedar` |
| CC6.2 | Authn + authz | Per-tenant API keys + SPIFFE | `policy/tenant-scope.cedar` |
| CC6.3 | Access lifecycle | OpenBao adds/removes + audit | OpenBao audit log |
| CC6.6 | Logical access controls | Postgres RLS + Cedar + reserved tenants | `threat-model.md` T-I-01 mitigation |
| CC6.7 | Transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §DSR |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE; ImageMagick + ffmpeg CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + per-tenant rate limits + HPA | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability via observability µservice | `slos/` + `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts + bias-audit drift | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates | observability promotion gate per ADR-0139 |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` (Slice B) |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + EU AI Act Art. 50 transparency label + NYC LL144 candidate notice |
| P2 | Choice + consent | OpenBao tenant-resolver onboarding consent + GDPR Art. 22 opt-out flow |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation; EMPLOYMENT_RECORD class added per ADR-NET-0001 |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade including endorsement-chain revocation |
| P5 | Access | Tenant operators read own data |
| P6 | Disclosure to third parties | Sub-processor list + transfer register + ATS-handoff allowlist + employer-confirm chain |
| P7 | Quality | Audit-chain integrity + four-eyes disclosure + appeal workflow + endorsement-revocation flow |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Threat-model review cadence; threat-intel feeds | `threat-model.md` |
| A.5.10 | Acceptable use | Internal AUP + onboarding | `docs/standards/onboarding.md` |
| A.5.14 | Info transfer | mTLS + KMS + signed audit-chain + signed Ed25519 endorsement signatures | `threat-model.md` Trust Boundary 3 + 8 |
| A.5.15 | Access control | Cedar fragments + OIDC + MFA | `policy/*.cedar` |
| A.5.17 | Authentication info | OpenBao secret lifecycle + rotation | OpenBao audit log |
| A.5.18 | Access rights | Per-resource Cedar + four-eyes for disclosure | `policy/tenant-scope.cedar` |
| A.5.23 | Cloud-service security | Multi-region + DR posture | `multi-region.md` |
| A.5.26 | Incident response | Severity-classified IR; postmortems | `incident-response.md` |
| A.5.30 | ICT readiness for BCDR | DR pair + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal + statutory | Per-pack regulatory cross-mapping below | this doc |
| A.5.34 | Privacy + PII protection | Data-class taxonomy + DSR cascade + four-eyes | `policy/data-residency.md` §DSR |
| A.8.2 | Privileged access rights | JIT elevation; two-person rule for admin ops | OpenBao audit |
| A.8.3 | Info access restriction | Cedar + RLS + per-tenant key bindings | `threat-model.md` T-S-01 mitigation |
| A.8.5 | Secure authentication | OIDC + MFA + OAuth 2.1; mTLS internal | `policy/tenant-scope.cedar` |
| A.8.7 | Protection against malware | OPSWAT / ClamAV media scan + quarantine; ImageMagick + ffmpeg sandboxed (gVisor / Kata) | `threat-model.md` T-E-05 + media-malware runbook |
| A.8.11 | Data masking | Span redactor; profile-export redactor; search-result Cedar filter | `policy/data-residency.md` |
| A.8.12 | Data leakage prevention | DLP via PII detectors + cardinality limits + LEAN coverage; Professional-context-isolation enforced | `threat-model.md` T-I-08 mitigation |
| A.8.20 | Networks security | Service mesh + mTLS + NetworkPolicy | k8s NetworkPolicy review |
| A.8.21 | Network services | TLS termination + WAF + DDoS | ingress configuration |
| A.8.25 | Secure development lifecycle | LEAN gates + multispectrum review | `evidence/multispectrum/` |
| A.8.27 | Application security | OWASP ASVS v4; cargo audit | `threat-model.md` |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + `cargo deny` | CI lanes |
| A.8.32 | Change management | PR + LEAN + branch-protection | branch-protection.yaml |
| A.8.34 | Audit findings remediation | Audit-finding tracker per engagement | ops-compliance |

### ISO 30414:2018 (Human Capital Reporting)

| Clause | Implementation | Evidence |
|---|---|---|
| §4.3 (workforce demographics) | Per-tenant aggregate-only export; per-individual disclosure forbidden | `salary-insights-stub` BC + `policy/professional-context-isolation.md` |
| §4.4 (compensation + benefits) | Salary-insights aggregate band reporting; per-individual disclosure forbidden | `salary-insights-stub` |
| §4.5 (recruitment + mobility) | Recruiter-stub bias-audit cadence; jobs-handoff event integrity | ADR-NET-0002 + `dashboards/recommender-fairness-and-bias.json` |
| §4.7 (organizational culture) | Abuse-reporting + harassment category (KR 직장 갑질) coverage | `runbooks/endorsement-storm-throttle.md` (paired with content-moderation-rollback) |

### GDPR

| Article | Implementation | Evidence |
|---|---|---|
| Art. 5 (principles) | Data-class taxonomy + minimisation + retention | `policy/data-residency.md` |
| Art. 6 (lawful basis) | Per-class lawful-basis declared in `dpia.md` §2.2 | `dpia.md` |
| Art. 7 (consent) | Tenant onboarding + per-user signup consent + recruiter-stub explicit opt-in | `legal/dpa-template.md` (Slice B) |
| Art. 9 (special-category) | Pack-us-healthcare BAA + EEOC + KR PIPA Art. 23 consent for sensitive workforce data | `legal/baa-template.md` |
| Art. 13/14 (transparency) | Tenant onboarding notice; joint-controllership clause; EU AI Act Art. 50 label; NYC LL144 candidate notice | `legal/dpa-template.md` |
| Art. 17 (erasure) | DSR cascade including endorsement-chain revocation (per ADR-NET-0005); profile-export pre-delete | `policy/data-residency.md` §DSR |
| Art. 20 (portability) | vCard 4.0 (RFC 6350) + JSON Resume + GDPR-Art-20 portable JSON export per ADR-NET-0006 | `runbooks/profile-export-vcard-corruption.md` |
| Art. 21 (right to object) | Per-user opt-out of recommender + recruiter ranker | `capabilities/T2-auto.yaml` |
| Art. 22 (automated decisions) | Algorithmic-ranking transparency + appeal workflow; opt-out to chronological; mandatory human review for recruiter-search ranker, jobs-ranking, endorsement aggregation (per ADR-NET-0002) | `capabilities/T2-auto.yaml` + ADR-NET-0002 |
| Art. 25 (privacy-by-design) | Professional-context invariant; redactor; Cedar | `policy/professional-context-isolation.md` |
| Art. 28 (processor) | Per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | Audit-chain ledger | audit-chain µservice |
| Art. 32 (security) | Every mitigation in `threat-model.md` | `threat-model.md` |
| Art. 33 (breach notification) | IR playbook; 72h GDPR clock | `incident-response.md` |
| Art. 35 (DPIA) | This DPIA satisfies + recruiter-stub Art. 35(3)(a) high-risk DPIA | `dpia.md` |
| Art. 44–50 (transfers) | Pack-pinning; SCC required for cross-border; no federation in P01 | `policy/data-residency.md` |

### EU AI Act 2024/1689

| Article | Implementation | Evidence |
|---|---|---|
| Annex III §4 (employment + workers management) | HIGH-RISK classification applied to (a) recruiter-search ranker, (b) jobs-ranking, (c) endorsement-aggregation, (d) people-you-may-know recommender used for employment intent. Conformity assessment per Art. 43 prior to substantial-use deployment. | ADR-NET-0002 + foundry-runtime model card |
| Art. 9 (risk-management system) | Per-classifier risk register + mitigation tracker | `capabilities/T2-auto.yaml` |
| Art. 10 (data + governance) | Training-dataset SHA + bias-audit + golden-set eval; 4/5-rule statistical check per EEOC UGESP | foundry-runtime evidence pipeline |
| Art. 11 (technical documentation) | Per-classifier model card + ADR-NET-0002 | foundry-runtime |
| Art. 13 (transparency to users) | Per-decision `eu_ai_act_label: ai_generated_assessment` + appeal pointer | `capabilities/T2-auto.yaml` |
| Art. 14 (human oversight) | Manual reviewer in appeal workflow; mandatory human review for materially-impacting employment decisions | `runbooks/recruiter-classifier-rollback.md` |
| Art. 15 (accuracy + robustness + cybersecurity) | Per-release golden-set eval + adversarial robustness eval | foundry-runtime |
| Art. 27 (fundamental-rights impact assessment) | FRIA executed for recruiter-stub activation; coverage in dpia.md §FRIA | `dpia.md` |
| Art. 50 (transparency obligation) | UI label "AI-assessed" on every classifier/ranker decision | client SDK |
| Art. 52 (codes of conduct) | Voluntary code-of-conduct alignment | ops-compliance |
| Art. 72 (post-deployment monitoring) | Continuous bias-audit + drift-detector; mandatory quarterly review | `dashboards/recommender-fairness-and-bias.json` |
| Art. 73 (serious-incident reporting) | Notification to market surveillance authority within 15d for serious incidents | `incident-response.md` |

### EU Equal Treatment Directives (2000/43/EC + 2000/78/EC) + EU Pay Transparency Directive 2023/970

| Article | Implementation | Evidence |
|---|---|---|
| Dir. 2000/43/EC (racial equality) | Disparity ratio per protected group monitored on recruiter-stub ranker + jobs-ranking | `dashboards/recommender-fairness-and-bias.json` |
| Dir. 2000/78/EC (employment equality — age, disability, sexual orientation, religion) | Same coverage; per-feature audit when feature can serve as proxy | ADR-NET-0002 |
| Dir. 2023/970 (pay transparency) | Salary-insights stub: aggregate-only, per-individual disclosure forbidden; tenant opt-in only | ADR-NET-0006 + `salary-insights-stub` BC |

### UK Equality Act 2010 + ICO ADM Guidance

| Clause | Implementation |
|---|---|
| §13 (direct discrimination) | Disparate-treatment monitor on every employment-context ranker decision |
| §19 (indirect discrimination) | Disparate-impact (4/5 rule) statistical monitor |
| ICO ADM Code 2024 §6 (meaningful information about logic) | `getRankingExplanation` SDK helper + per-decision contributing-signals output |

### US Title VII Civil Rights Act 1964 + ADA + ADEA + EEOC UGESP 1978

| Clause | Implementation | Evidence |
|---|---|---|
| Title VII §703 (employment discrimination) | Recruiter-stub default OFF; activation requires bias audit | ADR-NET-0002 |
| ADA Title I (disability discrimination) | Accessibility-captions BC ships WCAG 2.2 Level AA; recruiter feature must not penalise assistive-tech use | `accessibility-captions` BC |
| ADEA (age discrimination) | Age cannot enter recommender feature vector as input or proxy; bias-audit per release verifies | ADR-NET-0002 |
| EEOC UGESP 29 CFR §1607 (Uniform Guidelines on Employee Selection Procedures) | 4/5-rule statistical disparity ratio published per release; record-keeping retention 2y | `dashboards/recommender-fairness-and-bias.json` |

### NYC AI Hiring Law (Local Law 144-2021)

| Clause | Implementation | Evidence |
|---|---|---|
| §20-870 (bias audit) | Annual independent bias audit when recruiter-stub activated for NYC tenant | `runbooks/recruiter-classifier-rollback.md` + ADR-NET-0002 |
| §20-871 (candidate notice) | 10-business-day prior notice published to candidate via tenant-onboarded UI | SDK helper |
| §20-872 (public summary) | Summary of bias-audit results published per tenant under DCWP rules | tenant onboarding |

### CA AB-331 (Automated Decision Tools) + CO SB 24-205 (Colorado AI Act)

| Clause | Implementation |
|---|---|
| CA AB-331 §22756 (deployer impact assessment) | FRIA template + per-tenant impact assessment when recruiter-stub activated for CA tenant |
| CA AB-331 §22756.3 (consumer notice) | Consumer notice + opt-out path |
| CO SB 24-205 §6-1-1701 (deployer notice + risk-management policy) | Risk-management policy + algorithmic-discrimination prevention duty operative when CO tenant activates recruiter-stub |

### EU DSA (Digital Services Act 2065/2022)

| Article | Implementation | Evidence |
|---|---|---|
| Art. 14 (terms-of-service transparency) | Per-tenant ToS + plain-language summary | tenant onboarding |
| Art. 16 (notice-and-action) | `AbuseReportFiled` Workflow event + abuse-reporting BC | `runbooks/endorsement-storm-throttle.md` (paired pattern) |
| Art. 17 (Statement of Reasons) | Every moderation verdict emits a Statement of Reasons + audit-chain seal | `capabilities/T2-auto.yaml` evidence pipeline |
| Art. 20 (internal complaint-handling) | `Appeal` workflow; ≤ 7d resolution | abuse-reporting BC |
| Art. 24 (transparency reports) | Per-tenant transparency log; quarterly export | `compliance.md` Continuous Evidence |
| Art. 27 (recommender system transparency) | Algorithmic feed + recruiter explanation API + user-controllable chronological override | `capabilities/T2-auto.yaml` |
| Art. 28 (online protection of minors) | minor-account never surfaces in recruiter or salary insights; minor cannot receive InMail from unconnected adult | per pack |

## Per-Pack Overlays

### pack-kr

| KR clause | Implementation |
|---|---|
| KR PIPA Art. 8 (child consent) | minor flow gated at `professional-profile` BC |
| KR PIPA Art. 15 (collection consent) | Tenant onboarding + per-user signup |
| KR PIPA Art. 17 (cross-border consent) | Pack-pinning; cross-border requires explicit consent |
| KR PIPA Art. 22-2 (sensitive consent) | pack-kr sensitive profile fields require additional consent flow |
| KR PIPA Art. 23 (sensitive data) | Encryption + Cedar entitlement + four-eyes for disclosure |
| KR PIPA Art. 28 (processor) | Tenant DPA |
| KR PIPA Art. 29 (technical safeguards) | All `threat-model.md` mitigations |
| KR PIPA Art. 29-2 (KR-specific automated-decision opt-out) | per-end-user opt-out toggle on recruiter + recommender |
| KR-ISMS-P §2.5 (personnel) | Two-person rule + JIT elevation |
| KR 근로기준법 (Labor Standards Act) | Work-record retention floor 3y; honoured per `policy/data-residency.md` |
| KR 직장 갑질 protections (workplace harassment) | dedicated `harassment-workplace` abuse-report category; routing per content-moderation BC |
| KR 통신비밀보호법 (Communications Secrecy Act) | InMail intercept only via four-eyes; covered |
| KR 정보통신망법 §49 (intercept) | Server-side admin reads only via four-eyes |
| KR 청소년 보호법 | Minor protections — minor never surfaces in recruiter-stub or salary insights |
| KR 전자문서법 Art. 5 (integrity) | Audit-chain Ed25519 seal |

### pack-eu

| Clause | Implementation |
|---|---|
| GDPR Art. 9 + 22 + 25 + 32 + 35 | Per ADR-NET-0002; recruiter-stub Art. 22 + 35 high-risk DPIA executed prior to activation |
| ePrivacy Directive Art. 5(3) | Confidentiality via Cedar + RLS |
| NIS2 2022/2555 (when thresholds engaged) | IR playbook 24h/72h/1mo timelines |
| eIDAS 910/2014 | Ed25519 audit-chain seals = AdES; endorsement signatures eIDAS-AdES-eligible (ADR-NET-0005) |
| EU DSA Arts. 14, 16, 17, 20, 24, 27, 28 | Per row above |
| EU AI Act Annex III §4 + Arts. 9-15, 27, 50, 52, 72, 73 | Per ADR-NET-0002 |
| EU Pay Transparency Directive 2023/970 | Salary-insights aggregate-only; per-individual disclosure forbidden |
| UK Equality Act 2010 (for UK-located tenants) | Per row above |
| UK Online Safety Act 2023 (UK-located tenants) | Ofcom illegal-content duty; safety-by-design report |

### pack-us

| Clause | Implementation |
|---|---|
| Title VII + ADA + ADEA | Per row above |
| EEOC UGESP 29 CFR §1607 | 4/5-rule statistical disparity per release |
| NYC LL144-2021 | annual bias audit + candidate notice |
| CA AB-331 | FRIA + opt-out + consumer notice |
| CO SB 24-205 | risk-management policy + algorithmic-discrimination prevention duty |
| Illinois HB 3773 (2024 — AI in employment) | per-tenant disclosure when AI used for employment decisions |
| CCPA + CPRA | per-user data-rights cascade; aligned with GDPR Art. 17 + 20 + 21 |

### pack-us-healthcare (HIPAA — when health-context profile surfaces)

| HIPAA clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | DPIA + threat-model |
| §164.308(a)(4)(ii)(B) (access authorization) | Cedar + OIDC + MFA |
| §164.310 (physical safeguards) | OCI-managed datacenter (BAA-eligible) |
| §164.312(a)(1) (access control) | Cedar + Postgres RLS + four-eyes |
| §164.312(b) (audit controls) | Audit-chain ≥ 6y retention |
| §164.312(c)(1) (integrity) | content-hash + audit-chain |
| §164.312(e)(1) (transmission security) | mTLS + KMS |
| §164.502(b) (minimum-necessary) | Recommender excluded from PHI accounts by default |
| §164.514 (de-identification) | PHI redactor pre-index |
| BAA template | `legal/baa-template.md` |

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

| Pack | Major clauses |
|---|---|
| pack-jp | APPI Art. 22-2 + 個人情報保護法 + 労働基準法 (work-record floor) + 労働契約法 |
| pack-sg | PDPA + PDPC employment guidance + Fair Consideration Framework (MOM) |
| pack-au | Australian Privacy Act 1988 + AHRC AI guidance + Fair Work Act 2009 |
| pack-in | DPDPA 2023 + Industrial Disputes Act 1947 + Equal Remuneration Act 1976 |
| pack-br | LGPD + CLT (Consolidação das Leis do Trabalho) |
| pack-ae | UAE PDPL + Federal Decree-Law 33/2021 (Labour Law) |
| pack-ksa | KSA PDPL + Labor Law (Royal Decree M/51) |

Per pack-overlay `regional-packs/<pack>/network-compliance-overlay.md`.

## Continuous Compliance Evidence

CI lane `oya-governance-compliance-evidence-recency --microservice network` evaluates every 24h:

- All `policy/*.cedar` files lint clean.
- All Helm charts pass `helm lint`.
- All OpenSLO manifests pass schema validation.
- All runbooks have a `last_drill_date` within 90 days.
- All threat-model rows have a re-review date within 90 days for residual ≥ M.
- All DPIA rows have a re-review date within 365 days.
- Per-tenant DPA + BAA signed status reflected in compliance dashboard.
- Per-classifier-version EU AI Act golden-set eval recorded; per-release model card sealed.
- EEOC UGESP 4/5-rule statistical disparity ratio recorded per release.
- NYC LL144 annual bias-audit timestamp within rolling 12mo window when NYC tenant active.
- EU DSA transparency report per-tenant exportable (quarterly).
- Pack-aware age-gate test exit 0 per pack.

Output: `microservices/network/evidence/compliance-evidence-<unix_ts>.json`.

## References

- `microservices/network/threat-model.md`.
- `microservices/network/dpia.md`.
- `microservices/network/policy/professional-context-isolation.md`.
- `microservices/network/policy/data-residency.md`.
- `microservices/observability/compliance.md` (shape reference).
- `microservices/social/compliance.md` (sibling reference; dual-context posture inherited).
- ADR-0028 (Bominal) + ADR-0008 + ADR-0135 + ADR-0139 + ADR-0131 + ADR-0140.
- EU DSA 2065/2022; EU AI Act 2024/1689; EU Pay Transparency Directive 2023/970.
- US Title VII + ADA + ADEA + EEOC UGESP 29 CFR §1607; NYC LL144-2021; CA AB-331; CO SB 24-205; IL HB 3773.
- ISO 30414:2018 (Human Capital Reporting); SLSA L3; NIST SSDF; OWASP ASVS v4; CIS K8s.
