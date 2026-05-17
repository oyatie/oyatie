---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-audit-chain, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0003, ADR-0117, ADR-0123, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/audit-chain/threat-model.md
  - microservices/audit-chain/dpia.md
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/policy/data-residency.md
  - microservices/audit-chain/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (audit-chain µservice)

## Purpose

Canonical control-to-framework mapping for audit-chain. Because this µservice is the evidence backbone, **almost every SOC 2 CC4.x / ISO 27001 A.8.15 / GDPR Art. 30 / HIPAA §164.312(b) / KR PIPA Art. 29 control across every other µservice ultimately points here** for its evidence emission. This document tells external auditors which audit-chain artifact satisfies which framework clause for every µservice.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC + 2022 Points of Focus)

| TSC | Control objective | Implementation in audit-chain | Evidence |
|---|---|---|---|
| CC4.1 Internal monitoring | Audit-chain emits SLI; verification correctness lane | `microservices/audit-chain/slos/` + `oya-check-verification-correctness` lane | self-SLO + lane history |
| CC4.2 Deficiency communication | `VerificationFailed` events emit on tamper detection | `runbooks/signature-verification-failure.md` | event stream |
| CC6.1 Logical access | Cedar policy + SPIFFE binding | `policy/tenant-scope.cedar` + `policy/ci-scope.cedar` + `policy/auditor-scope.cedar` | policy artifacts |
| CC6.2 Authentication + authorization | OIDC + SPIFFE + per-tenant API keys | `policy/seal-integrity.md` §"Signing call authenticated by SPIFFE" | OpenBao audit log |
| CC6.6 Logical access control | Three layers: Cedar + Postgres role + HSM IAM | `policy/seal-integrity.md` §"HSM Signing Policy" | layered policy artifacts |
| CC6.7 Information transmission + disposal | mTLS + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" | DSR runner output |
| CC6.8 Vulnerability management | `cargo deny` + Trivy + Grype | governance µservice's supply-chain spec | CI lane history |
| CC7.1 System operations | HA emission + DR-pair sealing + autoscaling | `capacity-model.md` + `multi-region.md` | deployed state |
| CC7.2 Monitoring system inputs | Self-SLO + OnCall paging | `failure-modes.md` | SLO history |
| CC7.4 Incident response | Severity-classified response per `incident-response.md` | runbooks | postmortem evidence |
| CC8.1 Change management | Signed commits + LEAN lanes + CODEOWNERS | branch-protection.yaml + governance µservice | branch-protection state |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation in audit-chain | Evidence |
|---|---|---|---|
| A.5.14 Information transfer | mTLS + cross-pack-replication-forbidden | `policy/data-residency.md` | enforcement at emission |
| A.5.15 Access control | Cedar + Postgres role separation + HSM partition isolation | `policy/*.cedar` + `policy/seal-integrity.md` | policy artifacts |
| A.5.17 Authentication information | OpenBao + SPIFFE + 24h-cert rotation | `policy/seal-integrity.md` §"SI-09" | OpenBao audit log |
| A.5.27 Learning from incidents | Postmortems published per `incident-response.md` | `evidence/postmortems/` | postmortem corpus |
| A.5.28 Collection of evidence | THIS MICROSERVICE IS THE IMPLEMENTATION | self | every SealRecord |
| A.5.33 Protection of records | S3 Object Lock Compliance mode + Merkle integrity | `policy/seal-integrity.md` §"SI-04" + §"FM-SI-03" | S3 bucket policy |
| A.8.2 Privileged access rights | JIT elevation + 2-person rule | `policy/ci-scope.cedar` §"PERMIT 5" + §"FORBID 2-person bypass" | OpenBao audit |
| A.8.3 Information access restriction | Cedar default-deny | `policy/tenant-scope.cedar` | policy artifact |
| A.8.5 Secure authentication | SPIFFE-bound mTLS for every call | `policy/seal-integrity.md` §"SI-09" | SPIFFE attestation log |
| A.8.7 Protection against malware | Trivy + Grype + signed images | governance µservice | container scan history |
| A.8.11 Data masking | Caller-redaction contract per Bominal ADR-0003 | source µservices redact; audit-chain treats opaque | source-µservice SDK redactor |
| A.8.12 Data leakage prevention | Cross-tenant query refusal (Cedar) + audit-chain payload-class enforcement | `policy/tenant-scope.cedar` | LEAN lane |
| A.8.15 Logging | THIS MICROSERVICE IS THE IMPLEMENTATION | self | every emission |
| A.8.16 Monitoring activities | Self-SLO + cross-channel root validator | `slos/` + `oya:audit_chain_root_cross_channel_match` recording rule | SLO + recording rules |
| A.8.24 Use of cryptography | TLS 1.3 + Ed25519 (HSM-backed) + AES-256-GCM at rest | `policy/seal-integrity.md` §"SI-07" + §"SI-08" | crypto inventory |
| A.8.25 Secure development life cycle | LEAN lanes + spec-driven-development | governance µservice | lane history |
| A.8.28 Secure coding | `cargo clippy` + `cargo deny` + crypto-crate version-pin | governance µservice | CI history |
| A.8.34 Protection during audit testing | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` | auditor audit-of-audits |

### GDPR

| Art. | Requirement | Implementation in audit-chain | Evidence |
|---|---|---|---|
| 5(1)(d) Accuracy | Chain integrity per `policy/seal-integrity.md` | self | SealRecord |
| 5(1)(e) Storage limitation | Per-pack retention matrix | `policy/data-residency.md` §"Retention" | retention-cascade output |
| 5(1)(f) Integrity + confidentiality | Ed25519 + Merkle + Cedar | `policy/seal-integrity.md` | SealRecord + Cedar audit |
| 5(2) Accountability | Every state change has an audit-chain record | self | every emission |
| 17 Right to erasure | DSR cascade with chain-preservation | `policy/data-residency.md` §"DSR Cascade" | DSR audit |
| 25 Privacy by design + default | Default-deny Cedar + caller-redaction + pack-pinning | `policy/seal-integrity.md` + `policy/data-residency.md` | architecture |
| 28 Processor terms | DPA template + per-tenant audit access | `legal/dpa-template.md` | tenant DPAs on file |
| 30 Records of processing | THIS MICROSERVICE IS THE PLATFORM-WIDE REGISTER | self | every emission |
| 32 Security of processing | Threat-model mitigations | `threat-model.md` | mitigation cross-mapping |
| 33 Breach notification | 72h chain per `incident-response.md` | runbooks | incident audit trail |
| 35 DPIA | `dpia.md` | self | DPIA |
| 44–46 Cross-border transfers | SCC-only export bundle; pack-pinning default | `policy/data-residency.md` §"Exception" | transfer register |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.5 인적보안 | Access control | Cedar + JIT + 2-person rule |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + Ed25519 + AES-256 | `policy/seal-integrity.md` |
| KR-ISMS-P §2.9 사고관리 | Sev-1/2 reporting to PIPC within 72h | `incident-response.md` |
| KR-ISMS-P §2.12 위반관리 | Tamper detection per `policy/seal-integrity.md` §"SI-13..SI-14" | self |
| KR PIPA Art. 28 | Retention limitation | retention-cascade |
| KR PIPA Art. 29 | Technical safeguards | mapped in `threat-model.md` |
| KR PIPA Art. 29-2 | Encryption | Ed25519 + AES-256 |
| KR PIPA Art. 33 | DPIA | `dpia.md` |
| KR PIPA Art. 34 | Breach notification (72h to PIPC + 72h to subjects) | `incident-response.md` |
| KR PIPA Art. 36 | Right to erasure | DSR cascade |
| **KR 전자문서법 Art. 5** | Electronic document integrity | **Ed25519 + Merkle + WORM** — load-bearing |
| **KR 전자문서법 Art. 6** | Electronic document storage | S3 WORM + 3y retention default |
| **KR 전자문서법 Art. 7** | Electronic document verification | verification SDK |

### pack-us-healthcare (HIPAA)

| 45 CFR | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk management | Mitigations in `dpia.md` §6 |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar `auditor-scope.cedar` |
| §164.308(a)(6) Incident procedures | `incident-response.md` |
| §164.312(a)(1) Access control | Cedar + SPIFFE |
| **§164.312(b) Audit controls** | **THIS MICROSERVICE IS THE IMPLEMENTATION** |
| **§164.312(c)(1) Integrity** | Ed25519 + Merkle + WORM |
| §164.312(d) Person/entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) Transmission security | TLS 1.3 |
| §164.314(a)(1) Business associate contracts | BAA template |
| **§164.316(b)(2) 6y retention** | retention-cascade enforces |
| §164.502(a) TPO permitted uses | Operations purpose only |
| §164.404 Notification | `incident-response.md` |

### pack-eu (GDPR + EDPB + eIDAS + NIS2)

- EDPB Guidelines 4/2019 Art. 25: pseudonymisation + pack-pinning + default-deny Cedar.
- EDPB Guidelines 9/2022 breach: 72h chain in `incident-response.md`.
- EDPB Recommendations 01/2020 (post-Schrems II): pseudonymisation + EU-pack KMS keys for SSE; supplementary measures `legal/schrems-supplementary-measures.md`.
- **eIDAS 910/2014 Art. 26 (AdES)**: HSM-Ed25519 satisfies AdES.
- NIS2 (2022/2555): incident-reporting timelines integrated.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose | DPIA §2.4 |
| APPI Art. 20 | Security measures | threat-model |
| APPI Art. 24 | Cross-border restrictions | pack-pinning |
| APPI Art. 26-2 | Breach notification | `incident-response.md` |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/audit-chain-compliance-overlay.md`. Structure mirrors above:
- pack-sg: PDPA + MAS Notice 644 (finance ≥ 5y retention).
- pack-au: Privacy Act APP + APRA-CPS 234 (finance ≥ 7y retention).
- pack-in: DPDPA 2023 + RBI Master Direction (finance ≥ 7y retention).
- pack-br: LGPD + BACEN Res. 4.893/2021 (finance ≥ 5y).
- pack-ae: UAE PDPL Federal Decree-Law 45/2021.
- pack-ksa: KSA PDPL + SAMA Cybersecurity Framework 2017 (finance ≥ 10y retention).

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency` (cross-cutting)

Refuses merges on evidence > 90d old without refresh stamp. Forces quarterly re-validation.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence (config snapshot, lane-run output, audit-chain seal).
- `microservices/audit-chain/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence.

### Per-framework continuous runs

- Daily: SOC 2 CC4.x + CC7.x; ISO 27001 A.8.15 + A.8.16.
- Weekly: CC8.x; A.5.27.
- Monthly: CC3.x; A.5.7; key-rotation status.
- Quarterly: full matrix re-validation.
- Annually: full external auditor re-attestation.

### Audit evidence delivery

External auditors receive a frozen evidence pack scoped to (tenant, framework, engagement-window) signed by pack HSM key; auditor JIT token (per `policy/auditor-scope.cedar`) scopes; every read audit-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency --microservice audit-chain` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 + ISO 27001:2022 audit results in `evidence/audits/`.

## References

- `microservices/audit-chain/threat-model.md`.
- `microservices/audit-chain/dpia.md`.
- `microservices/audit-chain/policy/*`.
- `microservices/audit-chain/incident-response.md`.
- Bominal ADR-0028 + ADR-0003.
- ADR-0117 + ADR-0123 + ADR-0131 + ADR-0140.
- SOC 2 + ISO 27001:2022 + GDPR + KR PIPA + KR 전자문서법 + HIPAA + APPI + PDPA + Privacy Act 1988 + DPDPA 2023 + LGPD + UAE PDPL + KSA PDPL + SAMA.
- eIDAS 910/2014.
