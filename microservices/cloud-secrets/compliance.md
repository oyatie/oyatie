---
doc_class: Compliance
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: ops-legal + council-privacy + ops-security
deciders: ops-legal, ops-security, council-privacy, council-architecture, axis-cloud-secrets
review_cadence: annually + on every regulation change + on every audit cycle
related_artifacts:
  - microservices/cloud-secrets/threat-model.md
  - microservices/cloud-secrets/dpia.md
  - microservices/cloud-secrets/policy/data-residency.md
  - microservices/cloud-secrets/policy/secret-isolation.md
doc_status: published
---

# Compliance: cloud-secrets µservice

This document maps cloud-secrets controls to legal + regulatory frameworks across active and conditional packs. Each citation is concrete and links to the artifact that satisfies it.

## Framework Index

- §1: KR PIPA (pack-kr) — Art. 29 + Enforcement Decree
- §2: HIPAA (pack-us-healthcare) — §164.312(a)(2)(iv) encryption + §164.308 + §164.316
- §3: GDPR (pack-eu) — Arts. 25 + 28 + 30 + 32 + 33 + 35
- §4: PCI-DSS v4.0 (pack-us + pack-kr when payment) — §3.5 + §3.6 + §3.7 + §8.6 + §10.5
- §5: SOC 2 Type 2 — CC6.1-8 + CC7.1-5 + A1.1-2
- §6: ISO 27001:2022 — A.5 + A.8 controls applicable
- §7: NIST SP 800-57 (Key Management — General)
- §8: FIPS 140-3 (Cryptographic Module Validation)
- §9: LGPD (pack-br) — Art. 46 + 48 + 50
- §10: APPI (pack-jp) — Art. 23
- §11: PDPA + MAS-TRM (pack-sg) — §24 + §9
- §12: Privacy Act + APRA-CPS 234 (pack-au) — APP 11 + §29-36
- §13: DPDPA + RBI (pack-in) — §8 + §6.4
- §14: UAE PDPL (pack-ae) — Art. 20
- §15: KSA PDPL + SAMA + NCA ECC (pack-ksa) — Art. 19 + §4.3.4 + ECC-1:2018
- §16: NIS2 + eIDAS (pack-eu critical-entities) — Art. 21(2)(h)

## §1: KR PIPA (pack-kr)

| Article | Requirement | Control | Evidence |
|---|---|---|---|
| Art. 23 | Sensitive personal data — explicit consent + extra safeguards | tenant_id treated as sensitive; salted-hash; explicit tenant DPA consent for processing | `policy/data-residency.md` §"pack-kr"; `dpia.md` §"R-01" |
| Art. 23-2 | Cross-border transfer of sensitive data — forbidden without explicit consent | cross-pack replication forbidden; SCC does not authorise sensitive-data KEK transfer | `policy/data-residency.md` "Cross-Pack Replication Policy" |
| Art. 28 | Storage period limitation | rotation cadence + cryptographic-erasure on tenant offboard | `policy/secret-isolation.md` §"TI-04"; `policy/data-residency.md` "DSR + Tenant Offboard Cascade" |
| Art. 29 | 안전성 확보조치 (safety control measures) | encryption (HSM-backed KEK + AES-256-GCM), access control (per-tenant + per-µservice scope), audit (audit-chain Ed25519-sealed) | `threat-model.md`; all mitigations |
| Enforcement Decree Art. 30 | Audit retention ≥ 1y | audit-chain ≥ 3y default (pack-kr); KR-FSS ≥ 5y | `policy/data-residency.md` "Retention by Jurisdiction" |
| Art. 33 | 개인정보 영향평가 (PIA) | full DPIA | `dpia.md` |
| Art. 36 | Right to deletion | DSR cascade | `policy/data-residency.md` "DSR + Tenant Offboard Cascade" |
| PIPC Notice 2020-7 | Overseas-transfer notification | pack-kr residency guarantee in tenant DPA | `dpia.md` §"R-06"; tenant DPA template |

## §2: HIPAA (pack-us-healthcare)

| Citation | Requirement | Control | Evidence |
|---|---|---|---|
| §164.308(a)(1)(ii)(D) | Information system activity review | audit-chain queryable by ops-security + auditors | `policy/auditor-scope.cedar` |
| §164.308(a)(3) | Workforce security | per-µservice SPIFFE scope; 4-eye break-glass | `threat-model.md` T-E-02 |
| §164.308(a)(4) | Information access management | per-tenant namespace + per-µservice scope | `policy/secret-isolation.md` |
| §164.308(a)(5)(ii)(D) | Password management | no passwords; OIDC + JIT short-lived; HSM-backed signing | `threat-model.md` |
| §164.312(a)(2)(iv) | Encryption + decryption | HSM-backed KEK; AES-256-GCM at rest | `policy/data-residency.md` "KEK Lifecycle"; `threat-model.md` T-I-04 |
| §164.312(b) | Audit controls | audit-emitter → audit-chain (Ed25519 + Merkle) | `threat-model.md` T-R-01 |
| §164.312(e)(2)(ii) | Encryption in transit | mTLS everywhere; TLS 1.3 | `iac/helm/openbao/values.yaml`; SDK configuration |
| §164.314 | Organizational — BAA | tenant BAA template | `legal/baa-template.md` (Slice D) |
| §164.316(b)(2) | Documentation retention 6y | audit-chain 6y retention in pack-us-healthcare | `policy/data-residency.md` |
| §164.530(j) | Retention 6y | per above | per above |

## §3: GDPR (pack-eu)

| Article | Requirement | Control | Evidence |
|---|---|---|---|
| Art. 5(1)(c) | Data minimisation | only salted-hash tenant_id + SPIFFE id + path-hash carried | `dpia.md` §"2.2 Data Inventory" |
| Art. 5(1)(f) | Integrity + confidentiality | HSM + mTLS + Ed25519-sealed audit | `threat-model.md` |
| Art. 25 | Data protection by design + default | default-deny Cedar; salted-hash; LEAN-A11 BLOCKER | `policy/*.cedar`; `policy/secret-isolation.md` |
| Art. 28 | Processor obligations | tenant DPA + sub-processor enumeration | `legal/{dpa-template,sub-processors}.md` |
| Art. 30 | Records of processing | ROPA at `legal/ropa.md` | `legal/ropa.md` (Slice D) |
| Art. 32(1)(a) | Pseudonymisation + encryption | salted-hash tenant_id + HSM-KEK-encrypted at rest | per above |
| Art. 32(1)(b) | Confidentiality + integrity + availability + resilience | threat-model mitigations | `threat-model.md` |
| Art. 33 | Breach notification within 72h | Sev-1 incident → tenant + DPA within 72h | `incident-response.md` |
| Art. 35 | DPIA | this DPIA | `dpia.md` |
| Arts. 44-50 | Transfer mechanisms | per-pack residency; SCC does not authorise KEK transfer | `policy/data-residency.md` |

## §4: PCI-DSS v4.0 (pack-us + tenants with payment data)

| Requirement | Control | Evidence |
|---|---|---|
| §3.5.1 | Render PAN unreadable (encryption) | applicable to consumers; cloud-secrets provides KEK + DEK lifecycle | `policy/data-residency.md` "KEK Lifecycle" |
| §3.5.2 | Strong cryptography | AES-256-GCM + RSA-4096 + ECDSA P-384 + Ed25519 | crypto choices in `threat-model.md` |
| §3.6.1 | Documented key management | this document + rotation policy + `runbooks/hsm-key-rotation.md` | per above |
| §3.6.4 | Key rotation per cryptoperiod | rotation scheduler enforces; cascade-rotation | `IP-010` |
| §3.6.7 | Key compromise procedure | revoke + cascade-rotate within ≤5s | `runbooks/secret-leak-detected.md` |
| §3.7 | Key management lifecycle | per `runbooks/hsm-key-rotation.md` | per above |
| §8.6 | Strong cryptography for credentials | HSM-backed signing | per `threat-model.md` |
| §10.2 | Audit log of access | audit-emitter + audit-chain | `threat-model.md` T-R-01 |
| §10.5.1 | Audit retention ≥ 1y; 3mo immediately available | audit-chain retention ≥ 1y; hot-storage 3mo | `policy/data-residency.md` |

## §5: SOC 2 Type 2

| Trust Service Criterion | Requirement | Control | Evidence |
|---|---|---|---|
| CC6.1 | Logical + physical access controls | Cedar default-deny + SPIFFE + HSM physical isolation | `policy/*.cedar`; `threat-model.md` |
| CC6.2 | Authentication | OIDC + MFA + JIT + SPIFFE for workloads | per above |
| CC6.3 | Authorization | per-tenant + per-µservice scope | `policy/secret-isolation.md` |
| CC6.6 | Transmission of confidential info | TLS 1.3 + mTLS | `iac/helm/openbao/values.yaml` |
| CC6.7 | Disposal of confidential info | cryptographic-erasure on offboard | `policy/data-residency.md` "DSR" |
| CC6.8 | Anti-malware + integrity | container scanning (Trivy) + signed images (cosign) | `iac/helm/*/values.yaml` (cosign) |
| CC7.1 | System monitoring | audit-chain + observability SLO | per above |
| CC7.2 | Anomalous activity | audit-emit `cross_*_attempt`; alarms | `threat-model.md` mitigations |
| CC7.3 | Security incidents | `incident-response.md` Sev ladder | per above |
| CC7.4 | Incident communication | Sev-1 → tenant ≤72h, regulator ≤24h | per above |
| CC7.5 | Recovery + restoration | runbooks + drills | `runbooks/*` |
| CC8.1 | Change management | PR-review + LEAN gates + reviewer-agent | per governance µservice |
| A1.1 | Capacity | `capacity-model.md` | per above |
| A1.2 | Availability | SLOs in `microservices/cloud-secrets/slos/*` | per above |

## §6: ISO 27001:2022 (Annex A controls applicable)

| Control | Title | Control | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | quarterly threat-model review + LEAN-A11 pattern updates | `threat-model.md` review cadence |
| A.5.10 | Acceptable use of info + assets | tenant DPA + operator-acceptable-use docs | `legal/dpa-template.md` |
| A.5.14 | Information transfer | mTLS + TLS 1.3 + signed audit events | per threat-model |
| A.5.15 | Access control | Cedar default-deny + OpenBao policy + SPIFFE | per policy artifacts |
| A.5.16 | Identity management | OIDC + SPIRE issuance | per threat-model |
| A.5.17 | Authentication information | rotation per ISO cadence; HSM-backed | `policy/data-residency.md` "KEK Lifecycle" |
| A.5.18 | Access rights | per-µservice scope; revocation push | per above |
| A.5.19-23 | Supplier relationships | sub-processor enumeration; HSM vendor SLA | `legal/sub-processors.md` |
| A.5.26 | Response to incidents | `incident-response.md` | per above |
| A.5.28 | Collection of evidence | audit-chain Merkle + Ed25519 non-repudiation | per above |
| A.5.30 | ICT readiness for business continuity | `multi-region.md`; DR drills | per multi-region |
| A.5.31-33 | Legal + compliance | this document + DPIA | per above |
| A.8.2 | Privileged access rights | 4-eye break-glass; JIT elevation | `threat-model.md` T-E-02 |
| A.8.3 | Information access restriction | per-tenant + per-µservice | per policy |
| A.8.5 | Secure authentication | mTLS + OIDC + MFA | per above |
| A.8.7 | Protection against malware | Trivy scans; cosign | per above |
| A.8.10 | Information deletion | DSR cascade | per data-residency |
| A.8.11 | Data masking | salted-hash tenant_id; `Secret<T>` newtype | per threat-model |
| A.8.12 | Data leakage prevention | LEAN-A11 BLOCKER + Loki redaction | `policy/secret-isolation.md` |
| A.8.13 | Information backup | encrypted Postgres backups | `iac/helm/postgres/values.yaml` |
| A.8.15 | Logging | audit-chain | per above |
| A.8.16 | Monitoring | observability SLOs + alarms | per above |
| A.8.20 | Networks security | mTLS + NetworkPolicy | `iac/kustomize/base/` |
| A.8.21 | Security of network services | per above | |
| A.8.23 | Web filtering | Envoy / Istio gateway with WAF | per `iac` |
| A.8.24 | Use of cryptography | HSM + AES-256-GCM + Ed25519 | per data-residency "KEK Lifecycle" |
| A.8.25-28 | Secure development lifecycle | LEAN gates + reviewer-agent + threat-model | per governance |
| A.8.30 | Outsourced development | sub-processor diligence | `legal/sub-processors.md` |

## §7: NIST SP 800-57 Part 1 (Key Management — General)

Key management lifecycle aligned with NIST SP 800-57:

- **Pre-activation**: KEK generated in HSM ceremony with 4-eye witness; attestation captured.
- **Active**: rotation cadence per pack + cryptoperiod (KEK 1y, signing 90d, API 30d).
- **Suspended**: revoke + cascade-rotate; audit-chain seal.
- **Deactivated / Destroyed**: cryptographic erasure on tenant offboard; HSM partition destroy.
- **Compromised**: incident response per `incident-response.md` Sev-1.

## §8: FIPS 140-3 (Cryptographic Module Validation)

| Requirement | Control |
|---|---|
| Cryptographic module validation | OCI Cloud-HSM (FIPS 140-3 Level 3) + Thales Luna (FIPS 140-3 Level 3) |
| Approved algorithms | AES-256-GCM, RSA-4096, ECDSA P-384, Ed25519 — all FIPS-approved |
| Key generation in approved module | HSM-side generation only |
| Attestation | daily attestation report; failure pages |

## §9-15: Other pack-specific frameworks

Detailed mappings live in `regional-packs/<pack>/cloud-secrets-compliance-overlay.md` per pack activation. Summary:

| Pack | Lead framework | Key citations |
|---|---|---|
| pack-br | LGPD | Art. 46 + 48 + 50 |
| pack-jp | APPI | Art. 23 |
| pack-sg | PDPA + MAS-TRM | §24 + §9 |
| pack-au | Privacy Act + APRA-CPS 234 | APP 11 + §29-36 |
| pack-in | DPDPA + RBI | §8 + §6.4 |
| pack-ae | UAE PDPL | Art. 20 |
| pack-ksa | KSA PDPL + SAMA + NCA | Art. 19 + §4.3.4 + ECC-1:2018 |

## §16: NIS2 + eIDAS (pack-eu critical-entities)

| Citation | Requirement | Control |
|---|---|---|
| NIS2 Art. 21(2)(h) | Cryptography | per FIPS + threat-model |
| eIDAS 910/2014 Art. 24 | Qualified signature | HSM-backed Ed25519 supports qualified-signature workflows |

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate compliance-mapping --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Annual third-party audit:
- SOC 2 Type 2 (annual)
- ISO 27001:2022 (annual surveillance + tri-annual recertification)
- PCI-DSS QSA (annual for tenants with payment data)
- HIPAA security assessment (per BAA cadence)
- Pack-specific regulator inspections per pack activation

## References

- `microservices/cloud-secrets/threat-model.md`
- `microservices/cloud-secrets/dpia.md`
- `microservices/cloud-secrets/policy/data-residency.md`
- `microservices/cloud-secrets/policy/secret-isolation.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/runbooks/*.md`
- `microservices/cloud-secrets/legal/*.md` (Slice D)
- KR PIPA + Enforcement Decree
- HIPAA 45 CFR §164
- GDPR Regulation (EU) 2016/679
- PCI-DSS v4.0
- ISO 27001:2022 + ISO 27002:2022
- NIST SP 800-57 Part 1 Rev. 5
- FIPS 140-3
- NIS2 Directive (EU) 2022/2555
- eIDAS Regulation (EU) 910/2014
- LGPD Lei 13.709/2018
- APPI 2003 (as amended 2022)
- PDPA 2012 (SG) + MAS-TRM v2021
- Privacy Act 1988 (AU) + APRA-CPS 234
- DPDPA 2023 (IN) + RBI Master Direction
- UAE PDPL Federal Decree-Law No. 45/2021
- KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity 2017 + NCA ECC-1:2018
