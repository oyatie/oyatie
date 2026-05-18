---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security + ops-legal
deciders: council-privacy, ops-security, axis-mail, council-architecture, ops-compliance, ops-legal
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/mail/threat-model.md
  - microservices/mail/dpia.md
  - microservices/mail/policy/dual-context-isolation.md
  - microservices/mail/policy/data-residency.md
  - microservices/mail/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (mail µservice)

## Purpose

Canonical control-to-framework mapping for the mail µservice. Tells an external auditor (SOC 2 / ISO 27001 / GDPR DPA / KR PIPC / HIPAA OCR / KR-FSS / ANPD / etc.) exactly which control implementation satisfies which framework clause + evidence artifact pointer. Continuous-compliance lane (`oya-governance-compliance-evidence-recency`) keeps the matrix machine-verifiable.

## Enforced Frameworks (every µservice; every pack)

### SOC 2 Type 2 (2017 TSC + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | Integrity and ethical values | Signed-commit policy + CODEOWNERS | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/mail/CODEOWNERS` |
| CC1.5 | Accountability for performance | Per-µservice SLO targets + on-call | `PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `runbooks/oncall-rotation.md` (cross-ref) |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` §Escalation |
| CC2.3 | Communication with external parties | DPA + BAA + tenant onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification | Annual threat-model + DPIA | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519; four-eyes for sensitive ops | `policy/dual-context-isolation.md` §"Four-Eyes Legal Hold" |
| CC3.4 | Significant change risk | PR review + LEAN lanes | branch-protection.yaml |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes (mail-specific + cross-cutting) | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Postgres RLS + KMS DEK + signed commits | `policy/*.cedar` |
| CC5.3 | Policy + procedure deployment | Per-µservice runbooks + standards | `runbooks/*` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `auditor-scope.cedar`, `ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant API keys + SPIFFE | `threat-model.md` §"Actors" |
| CC6.3 | Adds/removes access | OpenBao access lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Postgres RLS + Mimir multi-tenancy + reserved namespaces | `policy/dual-context-isolation.md` |
| CC6.7 | Information transmission + disposal | TLS 1.3 + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype + weekly CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postfix + per-tenant rate limits + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts + abuse-classifier | observability µservice integration |
| CC7.4 | Incident response | Severity-classified response | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | branch-protection.yaml |
| CC9.1 | Risk mitigation | Multi-region DR + automated rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + employee mail notice cascade |
| P2 | Choice and consent | Tenant onboarding consent + per-user persona election |
| P3 | Collection | OTel SDK PII redactor + minimum-necessary handoff redaction |
| P4 | Use, retention, disposal | Retention matrix + DSR cascade + soft-delete grace |
| P5 | Access | Tenant operators read own; DSR access cascade; auditor JIT scope |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | Schema validation + audit-chain integrity |
| P8 | Monitoring and enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | TLS 1.3 + MTA-STS + cross-pack-replication-forbidden | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + per-context-pillar | `policy/dual-context-isolation.md` |
| A.5.17 | Authentication information | OpenBao secret rotation (30d / 90d cadences) | OpenBao audit |
| A.5.18 | Access rights | RBAC via Terraform; per-tenant compliance-officer scope | `iac/terraform/cedar-rbac.tf` |
| A.5.23 | Cloud security | OCI HIPAA-eligible regions for pack-us-healthcare | `policy/data-residency.md` |
| A.5.24 | Incident planning | Incident-response playbook | `incident-response.md` |
| A.5.25 | Incident assessment | Severity classification | `incident-response.md` §Severity |
| A.5.26 | Incident response | Severity-driven runbooks | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from incidents | Postmortem template + ADR successor-IP | `runbooks/postmortem-template.md` (cross-ref) |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for business continuity | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal, statutory | This document + per-pack overlays | `compliance.md` |
| A.5.33 | Protection of records | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy + PII protection | DPIA + DSR cascade + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access | JIT via OpenBao; 2-person rule for sensitive ops | OpenBao audit |
| A.8.3 | Information access restriction | Postgres RLS + Cedar | `policy/dual-context-isolation.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | branch-protection.yaml |
| A.8.5 | Secure authentication | OIDC + MFA + SPIFFE | `threat-model.md` §Actors |
| A.8.7 | Protection against malware | Trivy + Grype + Cosign + Rspamd inbound | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | OTel SDK PII redactor + handoff redactor | OTel redaction |
| A.8.12 | Data leakage prevention | Cross-context refusal + Postgres RLS + DLP | `policy/dual-context-isolation.md` |
| A.8.14 | Redundancy | HA Postfix + Postgres replication ≥ 3 + S3 RF-3 | `multi-region.md` |
| A.8.15 | Logging | Audit-chain + structured logs | ADR-0028 |
| A.8.16 | Monitoring activities | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Network security | NetworkPolicy + Istio mTLS | cloud-k8s |
| A.8.21 | Network services security | Same | Same |
| A.8.23 | Web filtering | WAF + OWASP CRS at REST ingress | cloud-iac |
| A.8.24 | Cryptography | TLS 1.3 + Ed25519 + AES-256-GCM + KMS DEK envelope | ADR-0028 + `policy/encryption.md` (cross-ref) |
| A.8.25 | Secure development lifecycle | LEAN + PR review + spec-driven-development | `docs/standards/*` |
| A.8.26 | Application security | OpenAPI schema + Cedar + LEAN | `contracts/openapi/mail.yaml` |
| A.8.27 | Secure architecture | Clean architecture per ADR-0056 + ADR-0105 | ADR-0056 + ADR-0105 |
| A.8.28 | Secure coding | Cedar fuzz + cargo clippy + cargo deny | LEAN lanes |
| A.8.32 | Change management | PR review + LEAN | branch-protection.yaml |
| A.8.33 | Test information | Synthetic data only in dev/staging | `docs/standards/testing.md` |
| A.8.34 | Audit testing protection | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership cascade | DPA |
| 5(1)(b) | Purpose limitation | Purpose declared in DPIA §2.4 | DPIA |
| 5(1)(c) | Data minimisation | OTel SDK redactor + handoff minimisation + minimum-necessary search | OTel + handoff design |
| 5(1)(d) | Accuracy | Schema validation + audit-chain | LEAN |
| 5(1)(e) | Storage limitation | Retention matrix | `policy/data-residency.md` |
| 5(1)(f) | Integrity + confidentiality | Per-tenant DEK + dual-context + RLS | `policy/dual-context-isolation.md` |
| 5(2) | Accountability | This + DPIA + ROPA | `legal/ropa.md` |
| 6 | Lawful basis | Art. 6(1)(b) contract + 6(1)(c) legal-obligation + 6(1)(f) legitimate-interest | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) for PHI (pack-us-healthcare); explicit consent for KR PIPA sensitive | DPIA §4 |
| 13 + 14 | Information to data subjects | Tenant notice + employee notice cascade | DPA |
| 17 | Right to erasure | DSR cascade per `policy/data-residency.md` | DSR runner |
| 22 | Automated decision-making | Mail-to-Workflow handoff requires explicit consent OR policy basis; not solely-automated | DPIA §6 R-06 |
| 25 | Privacy by design + default | Dual-context invariant + per-tenant DEK + encrypted-token search | `policy/dual-context-isolation.md` + `data-residency.md` |
| 28 | Processor terms | DPA template | `legal/dpa-template.md` |
| 30 | Records of processing | ROPA register | `legal/ropa.md` |
| 32 | Security of processing | Threat-model + isolation + Cedar + KMS | `threat-model.md` |
| 33 | Breach notification (72h) | Incident-response procedure | `incident-response.md` §"Regulatory notifications" |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | Not triggered (residual ≤ Medium) | DPIA §7 |
| 44–46 | Cross-border transfers | SCC-only; Schrems-II supplementary | `legal/transfer-register.md` |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법 + KR-FSS)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.1 policy | Annual ISMS-P policy review | This document annual review |
| KR-ISMS-P §2.2 risk management | Annual risk assessment | `threat-model.md` + `dpia.md` |
| KR-ISMS-P §2.3 HR security | Background check + onboarding training | `docs/standards/onboarding.md` |
| KR-ISMS-P §2.4 physical | Inherited from OCI ap-seoul-1 | Oracle attestation |
| KR-ISMS-P §2.5 access control | OpenBao JIT + Cedar | `policy/*.cedar` |
| KR-ISMS-P §2.6 encryption | TLS 1.3 + AES-256-GCM + KMS DEK | ISO A.8.24 |
| KR-ISMS-P §2.7 system | LEAN lanes + supply-chain | governance µservice |
| KR-ISMS-P §2.8 operations | Runbooks + incident-response | `runbooks/*` + `incident-response.md` |
| KR-ISMS-P §2.9 incident management | Sev 1/2 reporting per KR PIPA Art. 34 (72h to PIPC) | `incident-response.md` §pack-kr |
| KR-ISMS-P §2.10 PII processing | DPIA + DSR + retention | `dpia.md` + `data-residency.md` |
| KR-ISMS-P §2.11 sub-processor | Sub-processor list + DPA cascade | `legal/sub-processors.md` |
| KR-ISMS-P §2.12 violation management | Audit-chain tampering detection | ADR-0028 |
| KR PIPA Art. 3 | Collection minimization | OTel + handoff redactor |
| KR PIPA Art. 15 + 17 | Consent for collection + use | Tenant onboarding consent |
| KR PIPA Art. 18 | Use limitation | Purpose declared in DPIA |
| KR PIPA Art. 22-2 | Sensitive PII protections | `policy/dual-context-isolation.md` + KMS-in-KR |
| KR PIPA Art. 23 + 23-2 | Sensitive data + cross-border | `data-residency.md` |
| KR PIPA Art. 24 | RRN protection | redactor strips; no RRN in mailbox metadata |
| KR PIPA Art. 28 | Storage limitation | Retention matrix |
| KR PIPA Art. 29 | Technical safeguards | Mapped in `threat-model.md` |
| KR PIPA Art. 29-2 | Encryption requirement | A.8.24 |
| KR PIPA Art. 33 | DPIA + DPO + impact assessment | `dpia.md` + council-privacy chair |
| KR PIPA Art. 33-2 | DPO appointment notification | council-privacy chair registered with PIPC |
| KR PIPA Art. 34 | Breach notification (72h to PIPC + 72h to data subjects) | `incident-response.md` |
| KR 전자문서법 Art. 5 | Electronic document integrity | Ed25519 audit-chain seals |
| KR 전자문서법 Art. 6 | Electronic document storage | S3 immutable + audit-chain |
| KR 전자문서법 Art. 7 | Electronic document verification | Audit-chain Merkle proofs |
| KR-FSS 전자금융감독규정 | Mail retention 5y + KMS-in-KR + KR-resident operators | `data-residency.md` pack-kr overlay |

### pack-us-healthcare (HIPAA Privacy + Security + Breach Notification Rules)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | Annual risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk management | Mitigations in §6 DPIA | DPIA |
| §164.308(a)(3) Workforce security | Background checks + JIT elevation | OpenBao + ops-security |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar + per-tenant scope + four-eyes | `policy/*.cedar` |
| §164.308(a)(5) Security awareness + training | Onboarding + annual refresher | `docs/standards/onboarding.md` |
| §164.308(a)(6) Security incident procedures | `incident-response.md` | Incident response |
| §164.308(a)(7) Contingency plan | `multi-region.md` + `runbooks/*` | DR plan |
| §164.310(a) Facility access controls | OCI HIPAA-eligible regions | Oracle attestation |
| §164.312(a)(1) Access control | Postgres RLS + Cedar + dual-context | `policy/dual-context-isolation.md` |
| §164.312(b) Audit controls | Audit-chain emission ≥ 6y | ADR-0028 |
| §164.312(c)(1) Integrity | Ed25519 + content-addressable blob digest | `threat-model.md` T-T-02, T-T-06 |
| §164.312(d) Person/entity authentication | OIDC + MFA + SPIFFE + SCRAM-SHA-256 | `threat-model.md` Actors |
| §164.312(e)(1) Transmission security | TLS 1.3 + STARTTLS + MTA-STS | A.8.24 + `threat-model.md` T-T-01 |
| §164.314(a)(1) BAA | BAA template | `legal/baa-template.md` |
| §164.316(a)+(b)(2) Policies + procedures + 6y retention | `compliance.md` + retention | This file + `data-residency.md` |
| §164.502(a) Permitted uses (TPO) | Purpose limited to operations | DPIA §2.4 |
| §164.502(b) Minimum necessary | Handoff redactor + dual-context invariant | `policy/dual-context-isolation.md` |
| §164.514 De-identification | Pseudonymisation + dual-context | `policy/dual-context-isolation.md` |
| §164.404 Notification to individuals (60d) | Breach response chain | `incident-response.md` |
| §164.406 Notification to media (1000+) | Comms templates | `incident-response.md` |
| §164.408 Notification to HHS | OCR reporting | `incident-response.md` |
| HITECH §13402 | Breach notification timeline | `incident-response.md` |

### pack-eu (GDPR Arts. cited + EDPB Guidelines + eIDAS + NIS2 + ePrivacy)

- EDPB Guidelines 4/2019 (Art. 25 by design): satisfied per `dpia.md` §6 + `policy/dual-context-isolation.md`.
- EDPB Guidelines 9/2022 (breach notification): integrated in `incident-response.md`.
- EDPB Recommendations 01/2020 (post-Schrems II): supplementary measures in `legal/schrems-supplementary-measures.md`.
- eIDAS 910/2014 Art. 26 (AdES): Ed25519 audit-chain seals satisfy AdES.
- NIS2 (2022/2555): 24h + 72h + 1mo timelines in `incident-response.md`.
- ePrivacy Directive Art. 5: e-mail confidentiality preserved via dual-context + per-tenant DEK + TLS 1.3.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA §2.4 |
| APPI Art. 20 | Security control measures | `policy/dual-context-isolation.md` + `threat-model.md` |
| APPI Art. 21 | Supervision of employees + entrustees | `legal/sub-processors.md` |
| APPI Art. 23 | Third-party provision restrictions | DPA + cross-border SCCs |
| APPI Art. 24 | Cross-border transfer restrictions | `data-residency.md` JP-pinning |
| APPI Art. 26-2 | Data breach reporting (PPC + subjects) | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | Tenant DPA consent capture |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/mail-compliance-overlay.md` carry full local-law citation matrix:
- pack-sg: PDPA 2012 + MAS Notice 644
- pack-au: Privacy Act 1988 APP 1–13 + APRA-CPS 234
- pack-in: DPDPA 2023 + RBI Master Direction on IT Outsourcing 2023
- pack-br: LGPD + BACEN Res. 4.893/2021
- pack-ae: UAE PDPL Federal Decree-Law 45/2021
- pack-ksa: KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity Framework 2017

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any cited evidence > 90 days without refresh stamp. Forces quarterly re-validation.

### Evidence emission

For every framework × control:
- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence
- `microservices/mail/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset

Per-framework runs:
- Daily: CC4.x + CC7.x; A.8.15 + A.8.16
- Weekly: CC8.x; A.5.27
- Monthly: CC3.x; A.5.7
- Quarterly: full matrix re-validation
- Annually: external auditor re-attestation

### Audit evidence delivery

Auditors receive frozen evidence pack per `docs/templates/evidence-pack-template.md`; auditor JIT token (per `policy/auditor-scope.cedar`); engagement window bounded; every read audit-chain-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2: external auditor sign-off at `evidence/audits/soc2/<year>-type2-report.pdf`.
- Annual ISO 27001:2022: analogous.
- Per-pack audit cadences per local law.

## References

- `microservices/mail/threat-model.md`.
- `microservices/mail/dpia.md`.
- `microservices/mail/policy/{dual-context-isolation, data-residency}.md`.
- `microservices/mail/policy/*.cedar`.
- `microservices/mail/incident-response.md`.
- ADR-0008 (data-use-boundary); ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0123 (HG-MAIL); ADR-0135 (Connect dissolution); ADR-0131 (per-µservice layout); ADR-0140 (Cedar).
- SOC 2 Type 2: TSC 2017 + 2022 Points of Focus — `aicpa.org`.
- ISO/IEC 27001:2022 + ISO/IEC 27002:2022 — `iso.org`.
- GDPR — `gdpr-info.eu`; EDPB Guidelines — `edpb.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`.
- HIPAA — `hhs.gov/hipaa`.
- APPI — `ppc.go.jp`.
- PDPA (SG) — `pdpc.gov.sg`.
- Privacy Act 1988 (AU) — `oaic.gov.au`.
- DPDPA 2023 (IN) — `meity.gov.in`.
- LGPD (BR) — `gov.br/anpd`.
- UAE PDPL — `mohre.gov.ae`.
- KSA PDPL — `sdaia.gov.sa`.
