---
doc_class: DPIA
template_id: TPL-DPIA
microservice: cloud-secrets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cloud-secrets
deciders: council-privacy, ops-security, axis-cloud-secrets, ops-legal
methodology: GDPR Art. 35 (DPIA) + KR PIPA Art. 33 (개인정보 영향평가) + DPDPA 2023 §10
related_adrs: [ADR-0028, ADR-0117, ADR-0131]
review_cadence: annually + on every change touching personal data flow
related_artifacts:
  - secrets/threat-model.md
  - secrets/policy/data-residency.md
  - secrets/policy/secret-isolation.md
doc_status: published
---

# DPIA: cloud-secrets µservice

## 1. Purpose + Scope

The `cloud-secrets` µservice manages the cryptographic material and credentials used by every other oyatie µservice. It is itself **not** the primary processor of tenant end-user personal data — that role lies with each consuming µservice. However, cloud-secrets processes:

- **Tenant identifiers** (used as OpenBao namespace paths; classified `SENSITIVE_PIPA_ART23` per KR PIPA Art. 23 due to re-identification potential when combined with other oyatie metadata).
- **Secret metadata** (path, version, last-rotation timestamp; classified `BEHAVIORAL_TENANT_PRODUCT`).
- **Access audit events** (who accessed which secret when; classified `AUDIT` + may contain `PII_QUASI_IDENTIFIER` via SPIFFE-id-of-human-operator).
- **encryption-key BYOK material from tenants** (tenant-supplied KEK; ADR-0251 §D-10; classified `SECRET` and treated as tenant-supplied processor data per Art. 28).

This DPIA is required because:
- GDPR Art. 35(3)(a): "systematic and extensive evaluation" of natural persons — partial, via operator audit trails.
- GDPR Art. 35(3)(b): processing of sensitive data — yes, via SPIFFE-id-of-human-operator embedded in audit.
- KR PIPA Art. 33(1)(3): processing systems handling > 50 000 data subjects' sensitive information — applicable at scale.
- DPDPA 2023 §10: high-volume processor.

## 2. Data Flows + Data Subject Categories

### 2.1 Data Subject Categories

| Category | How processed | Lawful basis |
|---|---|---|
| Tenant operators (humans authorised by tenant to manage secrets) | OIDC identifier + audit on every action | GDPR Art. 6(1)(b) contract; KR PIPA Art. 15(1)(2) contract |
| oyatie-side operators (ops-security, ops-sre) | OIDC + employment record + audit | GDPR Art. 6(1)(f) legitimate interest (operating the service); KR PIPA Art. 15(1)(6) |
| Tenant end-users | **never directly processed** by cloud-secrets; their data is encrypted by DEKs that cloud-secrets manages, but cloud-secrets never sees plaintext end-user data | n/a (no direct processing) |

### 2.2 Data Inventory

| Data category | Class | Source | Purpose | Recipient | Retention | Pack-specific notes |
|---|---|---|---|---|---|---|
| OIDC subject id (operator) | `PII_IDENTIFYING` | OIDC IdP | audit + access decision | OpenBao policy eval + audit-chain | per pack audit retention | pack-eu pseudonymised in audit (Art. 25) |
| Tenant id (salted-hash) | `SENSITIVE_PIPA_ART23` | tenancy µservice | namespace path + audit | OpenBao + audit-chain | per tenant lifecycle | pack-kr: Art. 23 sensitive |
| SPIFFE workload id | `INTERNAL_ONLY` | SPIRE | mTLS authn | OpenBao | 24h | — |
| Secret path | `INTERNAL_ONLY` (path text); `SENSITIVE_PIPA_ART23` when combined with tenant id | µservice config | resolution | OpenBao | per rotation | — |
| Resolved secret value | `SECRET` (transient) | OpenBao KV | runtime use | consumer process memory (TTL ≤60s) | not retained | — |
| Audit event payload | `AUDIT` + may embed `PII_QUASI_IDENTIFIER` | OpenBao audit-device | audit | audit-chain | per pack | pack-us-hc: 6y HIPAA; pack-kr: ≥1y |
| BYOK material (tenant-supplied KEK) | `SECRET` (sub-class `TENANT_BYOK`) | tenant upload | wrap tenant DEKs | OpenBao + HSM | per tenant DPA | pack-kr/eu/us-hc: regulated handling |
| HSM attestation report | `AUDIT` | HSM | compliance evidence | audit-chain | 7y | — |
| Rotation policy | `INTERNAL_ONLY` | ops-security via PR | scheduling | OpenBao + git | append-only history | — |
| Cache hit/miss telemetry | `BEHAVIORAL_TENANT_PRODUCT` | resolver SDK | observability | `observability` µservice (Mimir) | 30d | — |

### 2.3 Data Flow Diagram (DFD)

```text
[Tenant operator OIDC]
        │ (OIDC + MFA)
        ▼
[Application Shell] ──→ [tenancy µservice] ──→ [cloud-secrets per-tenant-namespace-controller]
                                                       │
                                                       ▼
                                                [OpenBao per-tenant namespace]
                                                       │
            ┌──────────────────────────────────────────┤
            ▼                                          ▼
    [audit-emitter] ──→ [audit-chain]      [key-rotation-scheduler]
                                                       │
                                                       ▼
                                              [hsm-integration] ──→ [HSM partition]
                                                       │
                                                       ▼
                                       [revocation push → consumers' SDKs]
```

## 3. Risks + Mitigations (privacy axis)

| # | Risk | Likelihood | Impact (DS) | Mitigation | Residual |
|---|---|---|---|---|---|
| R-01 | Tenant identifier re-identification via audit + ontology cross-link | M | H (PIPA Art. 23 sensitive) | salted-hash tenant_id with per-pack salt; salt rotated annually; raw id only in OpenBao tenant-resolver | L |
| R-02 | Operator OIDC subject id linked to access patterns reveals operator behaviour | M | M | audit retention bounded to legal minimum + 30d engineering buffer; pseudonymisation in pack-eu | L |
| R-03 | Tenant denies access to own audit log (Art. 15 violation) | L | M | tenant audit export API in `audit-chain`; SLA 30d (GDPR), 30d (KR), 15d (BR), 30d (US-HC accounting-of-disclosures) | L |
| R-04 | Secret path leakage in error messages reveals tenant business structure | M | M | errors return opaque codes; never echo path or value; debug logs scrubbed at SDK | L |
| R-05 | encryption-key BYOK material persists after tenant offboard (ADR-0251 §D-10) | L | H (Art. 17 erasure violation) | tenant deprovisioning: 30d-grace soft-delete + cryptographic-erasure of DEKs (KEK destruction renders DEKs unrecoverable); audit-chain seal | L |
| R-06 | Cross-pack data movement of audit events | L | H (Art. 44 transfer violation) | per-pack audit-chain instance; forbidden by Cedar `pack-routing.cedar`; quarterly drill | L |
| R-07 | HSM vendor's home jurisdiction conflicts with pack residency | M | H | per-pack HSM vendor: pack-kr Thales Luna (KR-resident); pack-eu OCI Cloud-HSM (EU-resident); pack-us-hc OCI Cloud-HSM (US HIPAA-eligible region); never extra-pack | L |
| R-08 | DSR (right-to-erasure) on tenant deprovision: data older than retention may already be deleted | L | L | documented in DPA; tenant notified | L (accepted) |
| R-09 | Subject access request: operator OIDC id linked to audit events of access | M | M | data subject can request own access log via tenancy µservice DSR cascade; not via cloud-secrets directly | L |
| R-10 | Salted-hash collision reveals two tenants share namespace | L | M | sha256 with 16-byte salt; collision probability negligible at < 10^10 tenants | L |
| R-11 | Cross-border misroute during BCDR exercise | L | H | BCDR drills are intra-pack only (au-melbourne-1 ↔ au-sydney-1, eu-frankfurt-1 ↔ eu-amsterdam-1); cross-pack BCDR forbidden | L |
| R-12 | Insider misuse: ops-security reads break-glass without justification | M | H | 4-eye approval + quarterly review + tenancy-notification on tenant-scope break-glass | L |

## 4. Compliance Mapping

### 4.1 GDPR (pack-eu)

| Article | Requirement | How met |
|---|---|---|
| Art. 5(1)(c) | Data minimisation | Only tenant_id (salted-hash) + SPIFFE id + secret-path-hash carried in audit; no end-user PII |
| Art. 5(1)(f) | Integrity + confidentiality | Per-pack OpenBao + HSM + LUKS + mTLS + Ed25519-sealed audit |
| Art. 25 | Data protection by design + default | Default-deny Cedar; salted-hash tenant_id; LEAN-A11 BLOCKER |
| Art. 28 | Processor obligations | Per-tenant DPA + encryption-key BYOK processor model (ADR-0251 §D-10); sub-processors enumerated |
| Art. 30 | Records of processing | This DPIA + `microservices/cloud-secrets/legal/ropa.md` (Slice D) |
| Art. 32 | Security of processing | Per `threat-model.md`; encryption + access control + audit + resilience |
| Art. 33 | Breach notification | Sev-1 incident on raw-secret-leak; tenant notification within 72h |
| Art. 35 | DPIA | this document |

### 4.2 KR PIPA (pack-kr)

| Article | Requirement | How met |
|---|---|---|
| Art. 23 | Sensitive data | tenant_id treated as sensitive; explicit consent in tenant DPA |
| Art. 28 | Storage period limitation | per rotation policy; cryptographic-erasure on deprovision |
| Art. 29 | Safety control measures (안전성 확보조치) | per `threat-model.md` mitigations matrix |
| Art. 33 | 개인정보 영향평가 (PIA) | this document |
| Art. 36 | Right to deletion | DSR cascade documented |
| Enforcement Decree Art. 30 | Audit retention ≥1y | audit-chain retention ≥3y (pack-kr default); KR-FSS ≥5y |

### 4.3 HIPAA (pack-us-healthcare)

| Citation | Requirement | How met |
|---|---|---|
| §164.312(a)(2)(iv) | Encryption + decryption | HSM-backed KEK; AES-256-GCM at rest; TLS 1.3 in transit |
| §164.312(e)(2)(ii) | Encryption in transit | mTLS everywhere; TLS 1.3 minimum |
| §164.308(a)(4) | Information access management | Per-µservice scope + per-tenant namespace |
| §164.308(a)(1)(ii)(D) | Audit controls | audit-chain Merkle + Ed25519 + 6y retention |
| §164.316(b)(2) | Documentation retention | 6y on policy + this DPIA + audit |

### 4.4 DPDPA 2023 (pack-in)

| Section | Requirement | How met |
|---|---|---|
| §8(5) | Reasonable security safeguards | per `threat-model.md` |
| §10 | High-volume processor obligations | this DPIA + appointment of DPO |
| §12 | Right to erasure | DSR cascade |

### 4.5 PCI-DSS v4.0 (pack-us pack-us-healthcare pack-kr with payment data)

| Requirement | How met |
|---|---|
| §3.5 (PAN encryption) | not applicable directly (no PAN in this µservice); applies to consumers' use of cloud-secrets to wrap PAN DEKs |
| §3.6 (Key management) | per `threat-model.md` + HSM + rotation scheduler |
| §3.7 (Key management lifecycle) | rotation policies + cascade-rotation |
| §8.6 (Strong cryptography for credentials) | AES-256-GCM + Ed25519 + RSA-4096 / ECDSA P-384 |
| §10.5 (Audit retention) | audit-chain ≥1y, 3mo immediately available |

### 4.6 LGPD (pack-br)

| Article | Requirement | How met |
|---|---|---|
| Art. 6 | Principles | minimisation + necessity per data inventory |
| Art. 46 | Security measures | per threat-model |
| Art. 48 | Breach notification | 24h ANPD per LGPD timeline |
| Art. 50 | Security practices | per threat-model |

### 4.7 Other packs

Pack-specific compliance overlays at `regional-packs/<pack>/cloud-secrets-overlay.md` (per pack onboarding):

- pack-jp APPI Art. 23
- pack-sg PDPA §24 + MAS-TRM §9
- pack-au Privacy Act APP 11 + APRA-CPS 234 §29-36
- pack-ae UAE PDPL Art. 20
- pack-ksa PDPL Art. 19 + SAMA Cybersecurity §4.3.4 + NCA ECC-1:2018

## 5. Sub-Processors

| Sub-processor | Role | Pack | Lawful basis | Contract |
|---|---|---|---|---|
| Oracle (OCI Cloud-HSM) | HSM partition vendor | pack-eu, pack-us, pack-us-hc, pack-jp, pack-sg, pack-au, pack-in, pack-br, pack-ae, pack-ksa | Art. 28 sub-processor | OCI MSA + HSM addendum |
| Thales (Luna HSM) | HSM partition vendor | pack-kr (preferred for KR-FSS) | Art. 28 sub-processor | Thales HSM MSA |
| OpenBao project / LF Edge | OpenBao OSS upstream | all packs | OSS — no DP relationship | n/a (OSS) |
| OCI Object Storage | Postgres + audit backup storage | per pack | Art. 28 sub-processor | OCI MSA |

Tenant DPA enumerates sub-processors; tenants are notified of additions per Art. 28(2).

## 6. Data Subject Rights

| Right | How honoured |
|---|---|
| Art. 15 (access) — operator audit | tenant operator can request own access log via tenancy DSR cascade |
| Art. 16 (rectification) | not applicable (no rectifiable data; OIDC subject id is authoritative) |
| Art. 17 (erasure) | upon tenant deprovision: 30d grace + cryptographic-erasure of DEKs |
| Art. 18 (restriction) | namespace seal on dispute; audit retained |
| Art. 20 (portability) | not applicable (no portable data; tenant encryption-key BYOK is tenant-supplied per ADR-0251 §D-10) |
| Art. 21 (objection) | not applicable (no profiling) |
| Art. 22 (automated decision-making) | not applicable |

## 7. Approval Trail

| Reviewer | Role | Decision | Date |
|---|---|---|---|
| council-privacy lead | Privacy authority | Approved subject to annual review | 2026-05-17 |
| ops-security lead | Security authority | Approved subject to threat-model review cadence | 2026-05-17 |
| axis-cloud-secrets lead | Owner | Authored + signed | 2026-05-17 |
| ops-legal | Legal review | Approved for pack-kr / pack-eu / pack-us-hc; other packs reviewed per first-tenant onboarding | 2026-05-17 |
| DPO (pack-eu) | Data Protection Officer | Approved | pending pack-eu activation |

## 8. Review Cadence + Triggers

| Trigger | Action |
|---|---|
| Annual review | DPIA refresh + ROPA reconciliation |
| Sub-processor change | Tenant notification + DPIA update |
| Architecture change touching personal data flow | DPIA section update + reviewer-agent sign-off |
| Regulator inquiry | DPIA + threat-model + audit export within statutory window |
| Breach | DPIA reflection in post-mortem; mitigations strengthened |

## 9. References

- `secrets/threat-model.md`
- `secrets/policy/secret-isolation.md`
- `secrets/policy/data-residency.md`
- `microservices/cloud-secrets/legal/{ropa,sub-processors,dpa-template,baa-template,transfer-register}.md` (Slice D)
- ADR-0028 (audit-chain + data-class taxonomy)
- ADR-0131 (Cloud split)
- GDPR Arts. 5, 25, 28, 30, 32, 33, 35
- KR PIPA Arts. 15, 17, 23, 28, 29, 33, 36
- DPDPA 2023 §§ 6-12
- HIPAA 45 CFR §164.308, §164.312, §164.316
- PCI-DSS v4.0 §3.5, §3.6, §3.7, §8.6, §10.5
- LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48, 50
