---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: kr-pipa
authoritative_source: 개인정보 보호법 (Personal Information Protection Act, Act No. 19234, fully revised 2023) + 전자서명법 (Digital Signature Act, fully revised 2020)
related_adrs: [ADR-0251, ADR-0244, ADR-0247, ADR-0263]
date: 2026-05-21
---

# KR-PIPA Pack Overlay — CLM

KR-PIPA (Personal Information Protection Act, 개인정보 보호법) is South Korea's primary data-protection regime, comparable in scope to GDPR but with stricter consent and cross-border transfer rules. The Digital Signature Act (전자서명법) governs electronic signature evidence in Korean jurisdiction. CSAP (Cloud Security Assurance Program) governs cloud-service procurement for the public sector.

## Active triggers

The `kr-pipa` pack is **mandatory** when any of:

- `contract.signatory.residency == "KR"` or `contract.counterparty.principal_residency == "KR"`.
- `contract.governing_law` references Korean law.
- `tenant.declared_jurisdictions` includes KR.
- `tenant.entity_type` is a Korean legal entity (정보통신서비스 제공자, 신용정보회사, 의료기관, etc.).

## Article 32 — explicit consent (P0 LEGAL — L-009)

PIPA Article 32 (개인정보 처리방침의 수립 및 공개) and Article 15 (개인정보의 수집·이용) require explicit affirmative consent for PII processing, including:

- Mandatory consent items (필수 동의): cannot proceed without consent.
- Optional consent items (선택 동의): cannot be bundled with mandatory items; separate checkbox required.
- Sensitive information (민감정보 per Article 23): additional explicit consent.
- Unique identification numbers (고유식별정보 per Article 24): higher restriction; almost always paper-form consent.
- Cross-border transfer (per Article 28): explicit consent listing destination country, recipient, purpose, retention period.

CLM enforces Article 32 by:

- Splitting consent capture into mandatory and optional checkboxes at the data layer.
- Storing each consent item as a distinct `consent_record` (per `legal-dimensions/gdpr-article-7-consent-records.md` extended schema).
- Refusing to bundle optional consent items into the mandatory bundle.

## Article 39 — damages and liability

Article 39 imposes statutory damages up to KRW 5M per data subject for negligent breach, up to KRW 30M for intent. CLM enforces by:

- Tenant-scoped Cedar default-deny.
- HSM-backed encryption for sensitive information.
- Audit-chain seal for every PII access event.

## Cross-border transfer (Article 28)

Article 28 of PIPA (effective 2024-03-15 per the 2023 revision) requires:

- Explicit consent listing destination country, recipient, purpose, retention period (Article 28(1)).
- Or one of the alternative legal bases: BCR-equivalent corporate rules, certification, adequacy decision by PIPC, contractual safeguards (Article 28(2)-(5)).

CLM contracts that involve cross-border transfer of Korean PII are auto-classified `cross_border_transfer = true` and require:

- A signed Standard Contractual Clauses (SCC) per PIPC's standard form (issued 2022-07-15) attached as a `kr_pipa_scc` sub-contract.
- Audit-chain seal of the SCC attestation.

## Digital Signature Act — 전자서명법

The 2020 revision retired the previous concept of 공인전자서명 (certified electronic signature, formerly the sole legally-equivalent-to-handwritten signature) and adopted technology neutrality. Two signature evidence levels apply:

- **일반전자서명** (general electronic signature): any electronic data attached for signing purpose.
- **인증전자서명** (certified electronic signature): issued by a Certified Electronic Signature Service (전자서명인증사업자) accredited under Article 7.

CLM supports both via the canonical signature envelope:

- 일반전자서명 maps to AES envelope.
- 인증전자서명 maps to QES envelope with a Korean Certified TSP certificate (e.g. KICA, KTNET, KOSCOM, SignKorea, Yessign).

## Time-Stamp Authority

For 인증전자서명, the time-stamp must be issued by a KISA (Korea Internet & Security Agency)-rooted TSA. Approved TSAs include:

- KISA Time-Stamping Authority.
- KICA (Korea Information Certificate Authority).
- KOSCOM TSA.
- KTNET TSA.

The µservice maintains a KISA-rooted Trust List and validates each Korean QES against the current list.

## Sovereign-cell residency

PIPA Article 28-2 (effective 2024-03-15) imposes data-localisation requirements for certain regulated sectors (telecom, financial services, healthcare, public services). When `jurisdiction_pack = kr-pipa-sovereign`:

- Primary store in a Korean cell (Seoul preferred; Busan secondary) — see `iac/oci-guest/seoul/` or `iac/on-prem/kr/`.
- Cross-region replication restricted to other Korean cells.
- Outbound transfer to non-Korean cells gated by Article 28 consent + SCC.
- Encryption-at-rest with KISA-approved cipher suites (typically ARIA-256 or AES-256 + SEED-128 fallback).

## CSAP — Cloud Security Assurance Program

For public-sector tenants (정부기관, 공공기관, 지방자치단체), CSAP certification is required. CSAP has three levels: 상 (high), 중 (medium), 하 (low). CLM's CSAP overlay activates the corresponding cell-eligibility gates (CSAP-certified cells only).

## Retention overlay

| Document class | Retention | Source |
|---|---|---|
| Executed contract with KR PII | 5 years from contract termination (default); per-sector overrides apply | PIPA Article 21 + Commercial Code Article 33 |
| Consent record | Until consent withdrawal + 3 years | PIPA Article 21 |
| Cross-border transfer SCC | Until transfer ends + 5 years | PIPA Article 28 |
| Sensitive information processing | Until purpose ends + immediate destruction unless legal hold | PIPA Article 21(2) |

## Cedar gate fragment

```cedar
forbid (
  principal,
  action == Action::"CrossBorderTransfer",
  resource is Contract
) when {
  resource.active_packs.contains("kr-pipa") &&
  (resource.scc_kr_pipa == null ||
   resource.cross_border_consent == null)
};

forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.active_packs.contains("kr-pipa") &&
  resource.required_signature_level == "certified_electronic_signature" &&
  (resource.signer_certificate.kisa_qualified == false ||
   resource.tsa.kisa_qualified == false)
};
```

## Tenant-class composition

- `tenant_class=demo_trial`: PIPA general electronic signature only; certified electronic signature gated off.
- `tenant_class=paid + jurisdiction_pack=kr-pipa-sovereign`: certified electronic signature + sovereign-cell residency.

## Composition with other packs

- `kr-pipa` + `gdpr`: dual-pack tenants (KR-EU cross-border) must satisfy both; KR Article 28 typically stricter than GDPR Chapter V.
- `kr-pipa` + `eidas`: cross-jurisdiction signatures dual-envelope (eIDAS PAdES on EU side + Korean certified envelope on KR side).
- `kr-pipa` + `hipaa-baa`: rare; Korean healthcare contracts use KR Medical Service Act overlay (의료법) instead.

## Evidence on activation

Activation of the `kr-pipa` pack emits:

- `oya.contract.lifecycle.management.pack.kr-pipa.activated` audit event with the tenant's KISA-rooted TSA preference and Korean cell home.
- Cedar policy compilation with Article 28 + Article 32 gates enabled.
- KISA TSA preference order recorded.
- Cross-emit to `audit-chain` to mark the tenant as KR-PIPA-in-scope.

## Standards references

- 개인정보 보호법 (Act No. 19234, fully revised 2023-09-15).
- 전자서명법 (Act No. 17354, fully revised 2020-06-09).
- 신용정보의 이용 및 보호에 관한 법률 (Credit Information Use and Protection Act).
- 의료법 (Medical Service Act).
- KISA 전자서명 인증서비스 (Certified Electronic Signature Service) accreditation guidelines.
- CSAP (Cloud Security Assurance Program) 인증 기준.
- PIPC Standard Contractual Clauses for Cross-Border Transfer (2022-07-15).
