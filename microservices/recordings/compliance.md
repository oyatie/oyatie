---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-compliance + axis-recordings + council-privacy
related_adrs: [ADR-0117, ADR-0126, ADR-0131, ADR-0133, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0006]
doc_status: published
---

# Compliance Matrix: recordings µservice

## Frameworks honoured

| Framework | Scope | Activation |
|---|---|---|
| SOC 2 Type 2 (CC6, CC7, CC8) | every pack | always-on |
| ISO/IEC 27001:2022 (Annex A controls per threat-model) | every pack | always-on |
| ISO/IEC 27037:2012 | digital-evidence preservation | always-on for ediscovery + legal-hold |
| NIST SP 800-86 | forensic-integrity | always-on |
| NIST SP 800-218 (SSDF) | secure SDLC | always-on |
| SLSA L3 | build provenance | always-on |
| OWASP ASVS v4.0.3 | application security | always-on |
| CIS Kubernetes Benchmark v1.9 | K8s hardening | always-on |
| eIDAS | qualified signatures on export bundles | conditional per tenant |
| EBU R128 | audio loudness | always-on |
| ITU-T G.107 | audio quality (E-model) | always-on |
| SMPTE-TT | timed-text | always-on |
| GDPR (Arts. 5/6/9/13/14/17/22/25/30/32/33/35/44-50) | pack-eu | always-on for EU tenants |
| ePrivacy Directive 2002/58 Art. 5(3) | pack-eu | always-on |
| EU AI Act (Arts. 13/27/50/Annex III) | pack-eu | always-on for AI capabilities |
| NIS2 Directive 2022/2555 | pack-eu | conditional (threshold) |
| HIPAA 45 CFR §§164.308/312/316/502/514/530 | pack-us-healthcare | BAA-conditional |
| HITECH Act | pack-us-healthcare | BAA-conditional |
| SEC Rule 17a-4(f) | pack-us-financial | always-on for SEC-regulated tenants |
| FINRA Rule 4511 | pack-us-financial | always-on for FINRA-regulated tenants |
| MiFID II Art. 16(7) | pack-eu (financial-services) | conditional |
| CFTC Rule 1.31 | pack-us-financial | conditional |
| FRCP Rule 26(f)/34 | every pack | always-on for ediscovery |
| Sedona Conference | every pack | always-on for ediscovery |
| KR PIPA Arts. 15/17/22-2/23/28/29 | pack-kr | always-on for KR tenants |
| KR-ISMS-P | pack-kr | tenant-conditional |
| KR 전자문서법 (Electronic Document Act) | pack-kr | always-on |
| KR 통신비밀보호법 (Wiretap Act) | pack-kr | always-on (recording-consent gate) |
| APPI (Japan) | pack-jp | always-on |
| PDPA 2012 (Singapore) | pack-sg | always-on |
| Privacy Act 1988 + TIA Act + Surveillance Devices Act | pack-au | always-on |
| DPDPA 2023 (India) | pack-in | always-on |
| LGPD (Brazil) | pack-br | always-on |
| UAE PDPL | pack-ae | always-on |
| KSA PDPL + SAMA | pack-ksa | always-on |

## Per-Article Control Mapping

### GDPR

| Article | Control surface | Evidence |
|---|---|---|
| Art. 5(1)(a) lawfulness | Cedar policies + recording-consent banner | `policy/cedar/*.cedar` + ingest-consent flag |
| Art. 5(1)(b) purpose-limitation | per-pack overlay | `policy/data-residency.md` |
| Art. 5(1)(c) data-minimisation | redaction overlay | ADR-RECORDINGS-0003 |
| Art. 5(1)(d) accuracy | content_hash + audit-chain seal | `tests/e2e/audit-chain-content-hash.rs` |
| Art. 5(1)(e) storage-limitation | retention purge + KMS-shred | ADR-RECORDINGS-0002 |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK envelope encryption | Bominal ADR-0111 |
| Art. 5(2) accountability | audit-chain Ed25519 seals | Bominal ADR-0028 |
| Art. 6 lawfulness | per-purpose legal basis | `dpia.md` |
| Art. 9 special-category | diarization opt-in | DPIA R-01 |
| Art. 13/14 transparency | producer-side recording-consent banner | meet / messenger |
| Art. 17 right-to-erasure | DSR cascade | `runbooks/dsr-cascade.md` (referenced from common pack) |
| Art. 22 automated-decision | auto-summary Annex III gate | ADR-RECORDINGS-0006 |
| Art. 25 by-design | redaction overlay; default-deny Cedar | always-on |
| Art. 30 record-of-processing | ROP entry per pack | `legal/transfer-register.md` |
| Art. 32 security | encryption + access control | `threat-model.md` |
| Art. 33 breach-notification | breach-detector emits to ops-compliance | always-on |
| Art. 35 DPIA | this set of docs | `dpia.md` |
| Arts. 44-50 transfer | residency pinning | ADR-0117 |

### HIPAA (pack-us-healthcare)

| 45 CFR Section | Control surface | Evidence |
|---|---|---|
| §164.308(a)(1) — security management | risk-analysis | `dpia.md` + `threat-model.md` |
| §164.308(a)(3) — workforce security | role-based access | tenancy µservice |
| §164.308(a)(4) — information-access management | Cedar policies | `policy/cedar/` |
| §164.308(a)(5) — security awareness + training | training curriculum | ops-security |
| §164.312(a)(1) — access control | mTLS + WebAuthn step-up | always-on |
| §164.312(b) — audit controls | audit-chain | always-on |
| §164.312(c)(1) — integrity | content_hash + Ed25519 seals | always-on |
| §164.312(d) — person + entity authentication | SPIFFE identity | always-on |
| §164.312(e)(1) — transmission security | TLS 1.3 + mTLS | always-on |
| §164.316 — policies + procedures | this doc + runbooks | always-on |
| §164.502(b) — minimum necessary | export-scope strict matching | ADR-RECORDINGS-0002 |
| §164.514 — de-identification (Safe Harbor) | redaction overlay matches 18 identifiers | ADR-RECORDINGS-0003 |
| §164.530(j) — 6-yr retention | pack-us-healthcare retention floor | ADR-RECORDINGS-0002 |

### SEC 17a-4(f) + FINRA 4511 + MiFID II 16(7) (pack-us-financial + pack-eu-financial)

| Rule | Control | Evidence |
|---|---|---|
| SEC 17a-4(f)(2) WORM | S3 object-lock + legal-hold-default-on | pack-us-financial overlay |
| SEC 17a-4(f)(3) accessibility | indexed retrieval + audit-chain | always-on |
| SEC 17a-4(b)(4) retention 3y / 6y | pack-default 3y; tenant-configurable to 7y | ADR-RECORDINGS-0002 |
| FINRA 4511 retention 6y | pack-default 6y | ADR-RECORDINGS-0002 |
| MiFID II Art. 16(7) — recording 5y + on-request to 7y | pack-default 5y; on-request extension to 7y | ADR-RECORDINGS-0002 |
| CFTC Rule 1.31 — recorded comms | aligned with SEC 17a-4 | ADR-RECORDINGS-0002 |

### EU AI Act

| Article | Control | Evidence |
|---|---|---|
| Art. 13 — technical documentation (high-risk) | per-capability `evidence_topic` | `capabilities/T2-auto.yaml` |
| Art. 27 — FRIA | DPIA section R-05 | `dpia.md` |
| Art. 50 — transparency | every transcription / summary / translate output labelled `ai-generated` | ADR-RECORDINGS-0006 |
| Annex III §4(a) — employment context | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |
| Annex III §6 — law-enforcement context | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |
| Annex III §8 — administration of justice | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |

### KR PIPA + 전자문서법 + 통신비밀보호법

| Article | Control | Evidence |
|---|---|---|
| PIPA Art. 15 — collection consent | recording-consent banner | producer side |
| PIPA Art. 17 — third-party transfer | residency pinning | ADR-0117 |
| PIPA Art. 22-2 — DPIA | this set of docs | `dpia.md` |
| PIPA Art. 23 — sensitive-info | diarization opt-in | DPIA R-01 |
| PIPA Art. 28 — technical security | encryption-at-rest + KMS | always-on |
| PIPA Art. 29 — admin security | runbook + procedures | always-on |
| 전자문서법 Art. 5 — electronic-doc retention with integrity | audit-chain Merkle seal | always-on |
| 전자문서법 Art. 6 — long-term preservation | tiered storage hot + cold | ADR-RECORDINGS-0005 |
| 통신비밀보호법 — recording-consent | ingest refuses without `consent_banner_confirmed: true` | ingest contract |

### KR-ISMS-P

| §2 control | Recordings surface |
|---|---|
| §2.1 policy + governance | this set of docs |
| §2.5 access control | Cedar policies |
| §2.7 system + service security | sandbox (gVisor) + LTS pinning |
| §2.10 incident handling | runbooks |
| §2.12 privacy | DPIA |

## Annual Audit Calendar

| Audit | Owner | Cadence |
|---|---|---|
| SOC 2 Type 2 | ops-compliance + external | annual |
| ISO 27001:2022 surveillance | ops-compliance + external | annual |
| HIPAA risk-analysis review | council-privacy + external | annual |
| GDPR DPIA review | council-privacy | annual |
| EU AI Act FRIA review | council-privacy + ops-compliance | annual + per high-risk activation |
| KR PIPA Art. 22-2 PIA | council-privacy + KR PIPC | per major change |
| SEC 17a-4 attestation | ops-compliance | per pack-us-financial tenant onboarding |

## References

- All frameworks listed above.
- `dpia.md`, `threat-model.md`, `policy/data-residency.md`.
- `decisions/ADR-RECORDINGS-0001..0007.md`.
