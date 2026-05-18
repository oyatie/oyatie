---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-social, council-architecture, ops-compliance
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/social/threat-model.md
  - microservices/social/dpia.md
  - microservices/social/policy/dual-context-isolation.md
  - microservices/social/policy/data-residency.md
  - microservices/social/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (social µservice)

## Purpose

Canonical control-to-framework mapping for the social µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / EU DSA Coordinator / EU AI Act notified body) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact.

## Enforced Frameworks (every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.5 | Accountability for performance | Per-µservice SLO + on-call | `slos/social-feed-render-latency.openslo.yaml` + `incident-response.md` |
| CC3.1 | Risk identification | Threat model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per ADR + IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; four-eyes disclosure; sybil-detector | `policy/dual-context-isolation.md` |
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
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts + sybil + trending poisoning | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates | observability promotion gate per ADR-0139 |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` (Slice B) |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + EU AI Act Art. 50 transparency label |
| P2 | Choice + consent | OpenBao tenant-resolver onboarding consent + age-gate + child-consent flow |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own data |
| P6 | Disclosure to third parties | Sub-processor list + transfer register + federation peer allowlist |
| P7 | Quality | Audit-chain integrity + four-eyes disclosure + appeal workflow |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Threat-model review cadence; threat-intel feeds | `threat-model.md` |
| A.5.10 | Acceptable use | Internal AUP + onboarding | `docs/standards/onboarding.md` |
| A.5.14 | Info transfer | mTLS + KMS + signed audit-chain + signed ActivityPub federation | `threat-model.md` Trust Boundary 3 + 9 |
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
| A.8.11 | Data masking | Span redactor; media preview redactor; search-result Cedar filter | `policy/redaction-phi.md` (pack-us-healthcare) |
| A.8.12 | Data leakage prevention | DLP via PII detectors + cardinality limits + LEAN coverage; federation-personal-tier-refused | `threat-model.md` T-I-08 mitigation |
| A.8.20 | Networks security | Service mesh + mTLS + NetworkPolicy | k8s NetworkPolicy review |
| A.8.21 | Network services | TLS termination + WAF + DDoS | ingress configuration |
| A.8.23 | Web filtering | n/a (server-side service) | – |
| A.8.25 | Secure development lifecycle | LEAN gates + multispectrum review | `evidence/multispectrum/` |
| A.8.27 | Application security | OWASP API Top 10 mitigations; cargo audit | `threat-model.md` |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + `cargo deny` | CI lanes |
| A.8.32 | Change management | PR + LEAN + branch-protection | branch-protection.yaml |
| A.8.34 | Audit findings remediation | Audit-finding tracker per engagement | ops-compliance |

### GDPR

| Article | Implementation | Evidence |
|---|---|---|
| Art. 5 (principles) | Data-class taxonomy + minimisation + retention | `policy/data-residency.md` |
| Art. 6 (lawful basis) | Per-class lawful-basis declared in `dpia.md` §2.2 | `dpia.md` |
| Art. 7 (consent) | Tenant onboarding + per-user signup consent | `legal/dpa-template.md` (Slice B) |
| Art. 8 (child consent) | `age-verification` BC + per-pack threshold | `policy/age-gate.md` (Slice B) |
| Art. 9 (special-category) | Pack-us-healthcare BAA + KR PIPA Art. 23 consent | `legal/baa-template.md` |
| Art. 13/14 (transparency) | Tenant onboarding notice; joint-controllership clause; EU AI Act Art. 50 label | `legal/dpa-template.md` |
| Art. 17 (erasure) | DSR cascade | `policy/data-residency.md` §DSR |
| Art. 22 (automated decisions) | Algorithmic-ranking transparency + appeal workflow; opt-out to chronological | `capabilities/T2-auto.yaml` |
| Art. 25 (privacy-by-design) | Dual-context invariant; redactor; Cedar | `policy/dual-context-isolation.md` |
| Art. 28 (processor) | Per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | Audit-chain ledger | audit-chain µservice |
| Art. 32 (security) | Every mitigation in `threat-model.md` | `threat-model.md` |
| Art. 33 (breach notification) | IR playbook; 72h GDPR clock | `incident-response.md` |
| Art. 35 (DPIA) | This DPIA satisfies | `dpia.md` |
| Art. 44–50 (transfers) | Pack-pinning; SCC required for cross-border; federation peer allowlist | `policy/data-residency.md` |

### EU DSA (Digital Services Act 2065/2022)

| Article | Implementation | Evidence |
|---|---|---|
| Art. 14 (terms-of-service transparency) | Per-tenant ToS + plain-language summary | tenant onboarding |
| Art. 16 (notice-and-action) | `AbuseReportFiled` Workflow event + `runbooks/abuse-report-backlog-drain.md` | content-moderation BC |
| Art. 17 (Statement of Reasons) | Every moderation verdict emits a Statement of Reasons + audit-chain seal | `capabilities/T2-auto.yaml` evidence pipeline |
| Art. 20 (internal complaint-handling) | `Appeal` workflow; ≤ 7d resolution | content-moderation BC |
| Art. 23 (out-of-court dispute settlement) | Pointer to certified dispute body; tenant-pack overlay | `legal/dsa-overlay-eu.md` (Slice B) |
| Art. 24 (transparency reports) | Per-tenant transparency log; quarterly export | `compliance.md` Continuous Evidence |
| Art. 27 (recommender system transparency) | Algorithmic feed explanation API + user-controllable chronological override | `capabilities/T2-auto.yaml` |
| Art. 28 (online protection of minors) | `age-verification` BC + minor-protection policy | per pack |

### EU AI Act 2024/1689

| Article | Implementation | Evidence |
|---|---|---|
| Art. 9 (risk-management system) | Per-classifier risk register + mitigation tracker | `capabilities/T2-auto.yaml` |
| Art. 10 (data + governance) | Training-dataset SHA + bias-audit + golden-set eval | foundry-runtime evidence pipeline |
| Art. 11 (technical documentation) | Per-classifier model card + ADR-SOC-0003 | foundry-runtime |
| Art. 13 (transparency to users) | Per-verdict `eu_ai_act_label: ai_generated_assessment` + appeal pointer | `capabilities/T2-auto.yaml` |
| Art. 14 (human oversight) | Manual reviewer in appeal workflow; mandatory human review for verdicts above confidence threshold | content-moderation BC |
| Art. 15 (accuracy + robustness + cybersecurity) | Per-release golden-set eval + adversarial robustness eval | foundry-runtime |
| Art. 50 (transparency obligation) | UI label "AI-assessed" on every classifier verdict | client SDK |
| Art. 52 (codes of conduct) | Voluntary code-of-conduct alignment | ops-compliance |

## Per-Pack Overlays

### pack-kr

| KR clause | Implementation |
|---|---|
| KR PIPA Art. 8 (child consent) | `age-verification` minor flow + parental consent attestation |
| KR PIPA Art. 15 (collection consent) | Tenant onboarding + per-user signup |
| KR PIPA Art. 17 (cross-border consent) | Pack-pinning; cross-border requires explicit consent |
| KR PIPA Art. 22-2 (sensitive consent) | pack-kr sensitive profiles require additional consent flow |
| KR PIPA Art. 23 (sensitive data) | Encryption + Cedar entitlement + four-eyes for disclosure |
| KR PIPA Art. 28 (processor) | Tenant DPA |
| KR PIPA Art. 29 (technical safeguards) | All `threat-model.md` mitigations map to Art. 29 controls |
| KR PIPA Art. 29-2 (KR-specific) | Audit log retention ≥ 1 year |
| KR-ISMS-P §2.5 (personnel) | Two-person rule + JIT elevation |
| KR-ISMS-P §2.7 (access control) | Cedar |
| KR 정보통신망법 §49 (intercept) | Server-side admin reads only via four-eyes |
| KR 청소년 보호법 | Minor-protection routing in `age-verification` |
| KR 전자문서법 Art. 5 (integrity) | Audit-chain Ed25519 seal |

### pack-us-healthcare

| HIPAA clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | DPIA + threat-model |
| §164.308(a)(4)(ii)(B) (access authorization) | Cedar + OIDC + MFA |
| §164.310 (physical safeguards) | OCI-managed datacenter (BAA-eligible) |
| §164.312(a)(1) (access control) | Cedar + Postgres RLS + four-eyes |
| §164.312(b) (audit controls) | Audit-chain ≥ 6y retention |
| §164.312(c)(1) (integrity) | content-hash + audit-chain |
| §164.312(e)(1) (transmission security) | mTLS + KMS |
| §164.502(b) (minimum-necessary) | Media preview + search redaction |
| §164.514 (de-identification) | PHI redactor pre-index |
| BAA template | `legal/baa-template.md` |
| US COPPA 15 USC §6501 | `age-verification` BC; minor-account flow |

### pack-eu

| Clause | Implementation |
|---|---|
| GDPR Art. 8 + 25 + 32 | Child consent + dual-context invariant; mitigations table |
| GDPR Art. 22 + 35 | Algorithmic ranking transparency + DPIA |
| ePrivacy Directive Art. 5(3) | Confidentiality via Cedar + RLS |
| NIS2 2022/2555 (when thresholds engaged) | IR playbook 24h/72h/1mo timelines |
| eIDAS 910/2014 | Ed25519 audit-chain seals = AdES |
| EU DSA Arts. 14, 16, 17, 20, 23, 24, 27, 28 | Notice-and-action + Statement of Reasons + appeal + transparency report + minor protection |
| EU AI Act Arts. 9, 10, 11, 13, 14, 15, 50, 52 | Risk-management + data governance + transparency + human oversight + accuracy + per-verdict label |
| UK Online Safety Act 2023 (UK-located tenants) | Ofcom illegal-content duty; safety-by-design report |

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack-overlay `regional-packs/<pack>/social-compliance-overlay.md`.

## Continuous Compliance Evidence

CI lane `oya-governance-compliance-evidence-recency --microservice social` evaluates every 24h:

- All policy/*.cedar files lint clean.
- All Helm charts pass `helm lint`.
- All OpenSLO manifests pass schema validation.
- All runbooks have a `last_drill_date` within 90 days.
- All threat-model rows have a re-review date within 90 days for residual ≥ M.
- All DPIA rows have a re-review date within 365 days.
- Per-tenant DPA + BAA signed status reflected in compliance dashboard.
- Per-classifier-version EU AI Act golden-set eval recorded; per-release model card sealed.
- EU DSA transparency report per-tenant exportable (quarterly).
- Pack-aware age-gate test exit 0 per pack.

Output: `microservices/social/evidence/compliance-evidence-<unix_ts>.json`.

## References

- `microservices/social/threat-model.md`.
- `microservices/social/dpia.md`.
- `microservices/social/policy/dual-context-isolation.md`.
- `microservices/social/policy/data-residency.md`.
- `microservices/observability/compliance.md` (shape reference).
- `microservices/messenger/compliance.md` (sibling reference; dual-context posture).
- ADR-0028 (Bominal) + ADR-0008 + ADR-0135 + ADR-0139 + ADR-0131 + ADR-0140.
- EU DSA 2065/2022; EU AI Act 2024/1689; UK Online Safety Act 2023.
