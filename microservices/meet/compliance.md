---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-meet, council-architecture, ops-compliance
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0126, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/meet/threat-model.md
  - microservices/meet/dpia.md
  - microservices/meet/policy/data-residency.md
  - microservices/meet/policy/recording-consent.md
  - microservices/meet/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (meet µservice)

## Purpose

Canonical control-to-framework mapping for the meet µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / SEC / FINRA / MiFID II reviewer / EU AI Act conformity assessor) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact.

## Enforced Frameworks (every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.5 | Accountability for performance | Per-µservice SLO + on-call | `slos/*.openslo.yaml` + `incident-response.md` |
| CC3.1 | Risk identification | Threat model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per ADR + IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; four-eyes recording disclosure | `policy/recording-consent.md` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLO | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 + audit-chain µservice |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT | `policy/*.cedar` |
| CC6.2 | Authn + authz | Per-tenant API keys + SPIFFE + LiveKit JWT scope | `policy/tenant-scope.cedar` |
| CC6.3 | Access lifecycle | OpenBao adds/removes + audit | OpenBao audit log |
| CC6.6 | Logical access controls | Postgres RLS + Cedar + Lobby evaluation | `threat-model.md` T-I-01 mitigation |
| CC6.7 | Transmission + disposal | mTLS + SRTP in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §DSR |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE; ffmpeg + LiveKit + Whisper CVE feed | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + per-tenant rate limits + HPA + LiveKit StatefulSet | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability via observability µservice | `slos/` + `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates | observability promotion gate per ADR-0130 |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA (LiveKit/Whisper/SRS/OBS) | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + in-meeting recording banner |
| P2 | Choice + consent | Recording-consent modal + per-participant AI-summary opt-in + E2E opt-in tier |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade with face-blur/voice-mask |
| P5 | Access | Tenant operators read own data; participants read their own attendance |
| P6 | Disclosure to third parties | Sub-processor list + transfer register + RTMP-egress allow-list |
| P7 | Quality | Audit-chain integrity + four-eyes recording disclosure |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Threat-model review cadence; threat-intel feeds | `threat-model.md` |
| A.5.10 | Acceptable use | Internal AUP + onboarding | `docs/standards/onboarding.md` |
| A.5.14 | Info transfer | mTLS + SRTP + KMS + signed audit-chain | `threat-model.md` Trust Boundary 3 |
| A.5.15 | Access control | Cedar fragments + OIDC + MFA + LiveKit token scope | `policy/*.cedar` |
| A.5.17 | Authentication info | OpenBao secret lifecycle + rotation | OpenBao audit log |
| A.5.18 | Access rights | Per-meeting ACL + four-eyes recording disclosure | `policy/tenant-scope.cedar` |
| A.5.23 | Cloud-service security | Multi-region + DR posture | `multi-region.md` |
| A.5.26 | Incident response | Severity-classified IR; postmortems | `incident-response.md` |
| A.5.30 | ICT readiness for BCDR | DR pair + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal + statutory | Per-pack regulatory cross-mapping below | this doc |
| A.5.34 | Privacy + PII protection | Data-class taxonomy + DSR cascade + four-eyes | `policy/data-residency.md` §DSR |
| A.8.2 | Privileged access rights | JIT elevation; two-person rule for admin ops | OpenBao audit |
| A.8.3 | Info access restriction | Cedar + RLS + per-tenant key bindings | `threat-model.md` T-S-01 mitigation |
| A.8.5 | Secure authentication | OIDC + MFA; mTLS internal; LiveKit JWT verify Ed25519 | `policy/tenant-scope.cedar` |
| A.8.7 | Protection against malware | ffmpeg gVisor sandbox; image-scan on every release | `runbooks/sfu-degraded.md` |
| A.8.11 | Data masking | Span redactor; transcript redactor; search-result Cedar filter | `policy/redaction-phi.md` (pack-us-healthcare) |
| A.8.12 | Data leakage prevention | DLP via PII detectors + cardinality limits + egress NetworkPolicy + RTMP allow-list | `threat-model.md` T-I-08 mitigation |
| A.8.20 | Networks security | Service mesh + mTLS + NetworkPolicy | k8s NetworkPolicy review |
| A.8.21 | Network services | TLS termination + WAF + DDoS + WebRTC ICE | ingress configuration |
| A.8.23 | Web filtering | n/a (server-side service) | – |
| A.8.25 | Secure development lifecycle | LEAN gates + multispectrum review | `evidence/multispectrum/` |
| A.8.27 | Application security | OWASP API Top 10 + OWASP ASVS v4 level 2; cargo audit | `threat-model.md` |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + `cargo deny` | CI lanes |
| A.8.32 | Change management | PR + LEAN + branch-protection | branch-protection.yaml |
| A.8.34 | Audit findings remediation | Audit-finding tracker per engagement | ops-compliance |

### GDPR

| Article | Implementation | Evidence |
|---|---|---|
| Art. 5 (principles) | Data-class taxonomy + minimisation + retention | `policy/data-residency.md` |
| Art. 6 (lawful basis) | Per-class lawful-basis declared in `dpia.md` §2.2 | `dpia.md` |
| Art. 9 (special-category) | Pack-us-healthcare BAA + KR PIPA Art. 23 consent + biometric Art. 9(2)(a) | `legal/baa-template.md` |
| Art. 13/14 (transparency) | Tenant onboarding notice; recording-consent modal; joint-controllership clause | `legal/dpa-template.md` + `policy/recording-consent.md` |
| Art. 17 (erasure) | DSR cascade with face-blur/voice-mask | `policy/data-residency.md` §DSR |
| Art. 22 (automated decisions) | AI summary classified non-binding; participant opt-out; meaningful information per Art. 22(3) | ADR-MEET-0006 |
| Art. 25 (privacy-by-design) | Recording-consent invariant; redactor; Cedar; E2E mode | `policy/recording-consent.md` |
| Art. 28 (processor) | Per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | Audit-chain ledger | audit-chain µservice |
| Art. 32 (security) | Every mitigation in `threat-model.md` | `threat-model.md` |
| Art. 33 (breach notification) | IR playbook; 72h GDPR clock | `incident-response.md` |
| Art. 35 (DPIA) | This DPIA satisfies | `dpia.md` |
| Art. 44–50 (transfers) | Pack-pinning; SCC required for cross-border | `policy/data-residency.md` |

### EU AI Act (Regulation 2024/1689)

| Article | Implementation | Evidence |
|---|---|---|
| Art. 13 (transparency) | Captions/summary labelled "AI-generated"; system-card-equivalent published | `capabilities/T1-assist.yaml` + ADR-MEET-0006 |
| Art. 50 (AI-generated content) | Per-output labelling; tenant-admin must surface in privacy notice | `capabilities/*.yaml` |
| Risk-class: minimal-risk (T0 suggest), low-risk (transcription, summary), medium-risk (live translation across language barriers per Art. 50(4)) | per-capability classification | ADR-MEET-0006 |
| Art. 26 (deployer obligations) | Tenant attests deployment-context at first-enable | `legal/dpa-template.md` |

### SEC Rule 17a-4(f) + FINRA Rule 4511 + SEC Rule 17a-3 (pack-us-financial)

| Clause | Implementation |
|---|---|
| 17a-4(f) WORM | S3 Object Lock on recording bucket; content_hash sealed; 3-7y retention |
| 17a-4(f) tamper-evident | Audit-chain Ed25519 over recording_id + content_hash |
| 17a-4(b)(4) easily-accessible | Recording manifest queryable by message_id + date + participant |
| FINRA 4511 supervisory review | Four-eyes disclosure path; supervisor entitlement Cedar-gated |
| SEC 17a-3 book-and-records | Per-meeting attendance log + recording manifest preserved |

### MiFID II (pack-eu — investment firm communications)

| Clause | Implementation |
|---|---|
| Art. 16(7) recording of telephone + electronic comms | Recording enabled for tenant-attested investment-firm; 5-7y retention floor |
| RTS 6 tamper-evident | Audit-chain Ed25519 + content_hash |
| Art. 35 audit access | Auditor scope Cedar policy time-boxed |

### HIPAA (pack-us-healthcare)

| HIPAA clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | DPIA + threat-model |
| §164.308(a)(4)(ii)(B) (access authorization) | Cedar + OIDC + MFA |
| §164.310 (physical safeguards) | OCI-managed datacenter (BAA-eligible) |
| §164.312(a)(1) (access control) | Cedar + Postgres RLS + four-eyes |
| §164.312(b) (audit controls) | Audit-chain ≥ 6y retention |
| §164.312(c)(1) (integrity) | content-hash + audit-chain |
| §164.312(e)(1) (transmission security) | mTLS + SRTP + KMS |
| §164.502(b) (minimum-necessary) | Transcript redactor + search redaction |
| §164.514 (de-identification) | Safe Harbour 18-identifier redactor |
| BAA template | `legal/baa-template.md` |

### KR PIPA (pack-kr)

| KR clause | Implementation |
|---|---|
| KR PIPA Art. 15 (recording consent) | Modal consent banner at join + audit-chain participant_consent_acknowledged |
| KR PIPA Art. 17 (cross-border consent) | Pack-pinning; cross-border requires explicit consent |
| KR PIPA Art. 22-2 (sensitive consent) | pack-kr sensitive recordings require additional consent flow |
| KR PIPA Art. 23 (sensitive data) | Encryption + Cedar entitlement + four-eyes for disclosure |
| KR PIPA Art. 28 (processor) | Tenant DPA |
| KR PIPA Art. 29 (technical safeguards) | All `threat-model.md` mitigations map to Art. 29 controls |
| KR PIPA Art. 29-2 (KR-specific) | Audit log retention ≥ 1 year |
| KR-ISMS-P §2.5 (personnel) | Two-person rule + JIT elevation |
| KR-ISMS-P §2.7 (access control) | Cedar |
| KR 정보통신망법 §49 (intercept) | Server-side admin recording-disclosure only via four-eyes |
| KR 전자문서법 Art. 5 (integrity) | Audit-chain Ed25519 seal |

### pack-eu additional (ePrivacy + eIDAS + AVMS Directive)

| Clause | Implementation |
|---|---|
| ePrivacy Directive 2002/58/EC Art. 5 | Confidentiality of communications via Cedar + RLS + E2E mode |
| ePrivacy Directive Art. 5(3) | Embedded analytics gated; default off |
| eIDAS 910/2014 | Ed25519 audit-chain seals = AdES on transcripts |
| AVMS Directive 2010/13/EU | When meet hosts public webinar broadcasts as AV-media-on-demand, content classification minima apply |
| NIS2 2022/2555 (when thresholds engaged) | IR playbook 24h/72h/1mo timelines |

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack-overlay `regional-packs/<pack>/meet-compliance-overlay.md`.

## Continuous Compliance Evidence

CI lane `oya-governance-compliance-evidence-recency --microservice meet` evaluates every 24h:

- All policy/*.cedar files lint clean.
- All Helm charts pass `helm lint`.
- All OpenSLO manifests pass schema validation.
- All runbooks have a `last_drill_date` within 90 days.
- All threat-model rows have a re-review date within 90 days for residual ≥ M.
- All DPIA rows have a re-review date within 365 days.
- Per-tenant DPA + BAA signed status reflected in compliance dashboard.
- EU AI Act per-capability risk-class document present and current per ADR-MEET-0006.

Output: `microservices/meet/evidence/compliance-evidence-<unix_ts>.json`.

## References

- `microservices/meet/threat-model.md`.
- `microservices/meet/dpia.md`.
- `microservices/meet/policy/data-residency.md`.
- `microservices/meet/policy/recording-consent.md`.
- `microservices/observability/compliance.md` (shape reference).
- ADR-0028 (Bominal) + ADR-0008 + ADR-0126 + ADR-0130 + ADR-0131.
- ADR-MEET-0001..0006.
- Standards cited inline above.
