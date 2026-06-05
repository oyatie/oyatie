---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security + ops-legal
deciders: council-privacy, ops-security, axis-mail, council-architecture, ops-compliance, ops-legal
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0131, ADR-0140 (retired per ADR-0145)]
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
| A.5.18 | Access rights | RBAC via OpenTofu; per-tenant compliance-officer scope | `iac/terraform/cedar-rbac.tf` |
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

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/mail-compliance-overlay.md` carry full local-law citation matrix:
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
- ADR-0008 (data-use-boundary); ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0123 (HG-MAIL); ADR-0135 (dissolution); ADR-0131 (per-µservice layout); ADR-0140 (Cedar).
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

---



## §day-one-cert-readiness
This anchor is closed for `mail` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `mail` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +19 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `mail` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §minor-protection
This anchor is closed for `mail` against ADR-0292 §D-1: minor-user refusal, teen age-band and age-verification handling.

### Service-specific answer
- Minor exposure for `mail` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-age-band activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `mail` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `mail` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`, `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`; +19 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.mail.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `mail` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `mail` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +19 more.
- Example event class: `oya.mail.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §ml-model-lifecycle
This anchor is closed for `mail` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `False` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `mail.t0-suggest` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `T0-suggest` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-fairness-audit
This anchor is closed for `mail` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `mail` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `mail` never stores protected attributes solely to make a product feature easier.
- Example: `T0-suggest` abuse/risk score challenge rate is compared across locale, accessibility profile, age band, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `mail` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `mail` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.mail.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `mail` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`, `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`; +5 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `mail.dual_context_isolation` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `mail` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +16 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.mail` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/mail/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `mail` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`, `microservices/mail/iac/ech-config.yaml`, `microservices/mail/iac/edge-waf.yaml`, `microservices/mail/iac/helm/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `mail` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`, `microservices/mail/iac/ech-config.yaml`, `microservices/mail/iac/edge-waf.yaml`, `microservices/mail/iac/helm/Chart.yaml`; +19 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `mail` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `mail` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `mail` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/mail/catalog/oya-mail-anti-phishing-kernel.yaml`, `microservices/mail/catalog/oya-mail-dual-context-isolation-kernel.yaml`, `microservices/mail/catalog/oya-mail-imap-frontend-rest.yaml`, `microservices/mail/catalog/oya-mail-inbound-smtp-adapter-smtp.yaml`, `microservices/mail/catalog/oya-mail-inbound-smtp-app.yaml`, `microservices/mail/catalog/oya-mail-jmap-frontend-rest.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `mail` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `mail` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `mail` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- State/event surfaces carrying classification: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `mail`; owner `axis-mail`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dual-context-isolation`, `imap-frontend`, `inbound-smtp`, `legal-hold`, `mailbox-store`, `outbound-smtp`; +2 more.
- Capability records cited: `microservices/mail/capabilities/T0-suggest.yaml`, `microservices/mail/capabilities/T1-assist.yaml`, `microservices/mail/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar/policy artifacts cited: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- SLO and dashboard evidence: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/mail/contracts/asyncapi/mail-events.yaml`, `microservices/mail/contracts/openapi/mail.yaml`, `microservices/mail/contracts/proto/mail.proto`.
- Cedar binding: `microservices/mail/policy/abuse-defence.cedar`, `microservices/mail/policy/anti-phishing.cedar`, `microservices/mail/policy/auditor-scope.cedar`, `microservices/mail/policy/ci-scope.cedar`, `microservices/mail/policy/data-residency.md`, `microservices/mail/policy/dual-context-isolation.md`; +4 more.
- State/event binding: `mail.dual_context_isolation`, `mail.imap_frontend`, `mail.inbound_smtp`, `mail.legal_hold`, `mail.mailbox_store`, `mail.outbound_smtp`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/mail/slos/dual-context-correctness.openslo.yaml`, `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`, `microservices/mail/slos/inbound-receive-availability.openslo.yaml`, `microservices/mail/slos/inbox-open-latency.openslo.yaml`, `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`, `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/mail/runbooks/account-compromise-recovery.md`, `microservices/mail/runbooks/dkim-key-rotation.md`, `microservices/mail/runbooks/dlp-quarantine-release.md`, `microservices/mail/runbooks/dmarc-rollout-monitoring.md`, `microservices/mail/runbooks/e2e-encryption-key-recovery.md`, `microservices/mail/runbooks/mail-bot-score-recalibration.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `mail`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `mail`.
- `policy-engine` supplies the signed Cedar corpus while `mail` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `mail` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `mail`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `mail` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
