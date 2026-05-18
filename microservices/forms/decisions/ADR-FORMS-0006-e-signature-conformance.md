---
id: ADR-FORMS-0006
title: E-signature conformance — eIDAS XAdES/PAdES/CAdES profiles; tenant-tier mapping (SES / AES / QES)
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + council-legal-compliance + ops-security
deciders: council-architecture, axis-forms, ops-security, council-legal-compliance, axis-fintech
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0140, ADR-FORMS-0001, ADR-FORMS-0003]
related_specs: [/specs/microservices/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-09 + AC-12
  - microservices/forms/policy/data-residency.md
  - microservices/forms/threat-model.md §"T-R-04"
  - microservices/forms/compliance.md §"5. eIDAS"
doc_status: published
---

# ADR-FORMS-0006: E-signature — eIDAS-conformant XAdES/PAdES/CAdES; tenant tier maps to SES / AES / QES class

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Many of Forms' competitive use cases involve a signed envelope: contract acceptance, patient consent (HIPAA + Art. 9), employee onboarding, vendor agreements. The signature must be non-repudiable, archive-grade, and legally recognised across jurisdictions.

EU **Regulation 910/2014 (eIDAS)** defines three classes of e-signature with progressively stronger non-repudiation:

1. **SES — Simple Electronic Signature** (Art. 3(10)): "data in electronic form which is attached to or logically associated with other data in electronic form and which is used by the signatory to sign". No specific technical requirements. Admissible as evidence (Art. 25(1)) but easier to repudiate.
2. **AES — Advanced Electronic Signature** (Art. 3(11), Annex II): linked to the signatory uniquely, capable of identifying the signatory, created using signature-creation data under signatory's sole control, linked such that subsequent change detectable. Crypto-anchored (typically PKCS#7 with signer cert).
3. **QES — Qualified Electronic Signature** (Art. 3(12)): AES PLUS based on a qualified certificate issued by a qualified trust service provider (QTSP) AND created using a qualified signature-creation device (QSCD). Equivalent to handwritten signature across the EU (Art. 25(2)).

ETSI standards define wire formats:
- **XAdES** (ETSI EN 319 132) — XML-based signature; for XML documents.
- **PAdES** (ETSI EN 319 142) — PDF-based signature; for PDF documents (most common for tenant-generated form-export envelopes).
- **CAdES** (ETSI EN 319 122) — CMS-based signature; for binary documents.

Each format supports archival profiles (-LT, -LTA) that embed validation data + long-term-signature time-stamps for decades-long verifiability.

Non-EU jurisdictions:
- **KR**: Electronic Signature Act + Electronic Document Act recognise advanced + qualified equivalents via KISA-certified CAs.
- **US** (ESIGN Act + UETA): broad recognition; tier-specific protections via state notary law.
- **JP**: Act on Electronic Signatures and Certification Services — recognises advanced-equivalent.
- **AU**: Electronic Transactions Act — broad recognition; specific exclusions (wills, statutory declarations).
- **IN**: IT Act 2000 — Digital Signature Certificate (Class 2 + Class 3) recognised; eIDAS-QES-equivalent via Class 3.
- **BR**: ICP-Brasil — ICP-Brasil signature equivalent to QES.

oyatie Forms must support all three eIDAS classes for pack-eu tenants and the per-pack equivalents elsewhere.

## Decision

Adopt **eIDAS-conformant signing pipeline** with **tenant-tier class mapping** and **multi-format wire output**.

### Class mapping per tenant tier

| Tenant tier | Default e-signature class | Available classes (tenant choice) |
|---|---|---|
| Free / Tier-1 | SES (click-to-sign + audit-chain seal) | SES |
| Tier-2 (standard) | SES | SES |
| Tier-3 (business) | AES default | SES, AES |
| Tier-D (developer / professional) | AES default | SES, AES |
| Tier-E (enterprise) | AES default; QES on per-form opt-in | SES, AES, QES |
| Tier-G (government / regulated) | QES default | SES, AES, QES |

Tenant cannot upgrade above their tier without explicit entitlement (`forms.esign.qes` Cedar entitlement gates QES).

### Wire format

- Form-export PDF → **PAdES-LTA** (long-term archival).
- Form-export XML → **XAdES-LTA**.
- Form-export binary attachment → **CAdES-LTA**.
- The signed envelope includes:
  - Form spec at submission time (`form.v1` per ADR-FORMS-0001; `schema_hash`).
  - Submitter response (full).
  - Signature creation timestamp (RFC 3161 TSA, qualified for QES).
  - Validation data (OCSP / CRL).
  - Long-term archival time-stamp (-LTA profile).
- All ciphertext fields (PII) decrypted in-memory at envelope generation time; envelope itself can be re-encrypted at tenant option.

### Certificate authority

- **QES**: tenant's chosen QTSP from the EU Trusted List + pack-specific equivalents (KISA-certified for pack-kr; ICP-Brasil for pack-br; Class 3 DSC issuer for pack-in; etc.). Tenant onboards the QSCD reference (USB token / smartcard / cloud-HSM).
- **AES**: oyatie-operated CA (pack-resident) per ISO 27001 A.5.34 cryptographic key management; or tenant-supplied AES CA.
- **SES**: no CA; HMAC-style attestation + audit-chain seal.

### Pack-specific overlays

- **pack-eu**: QES via EU Trusted List QTSP; default tier-E+; eIDAS legal equivalence Art. 25(2).
- **pack-kr**: Advanced/Qualified via KISA-certified CA; pack-overlay maps QES → KISA-qualified.
- **pack-us-healthcare**: HIPAA + state notary law; PAdES-LTA + tenant-state-attorney attestation reference.
- **pack-br**: ICP-Brasil for QES equivalent.
- **pack-in**: DSC Class 3 for QES equivalent.
- **(other packs)**: per per-pack overlay file.

### Archival

Signed envelopes archived per `policy/data-residency.md` retention table:
- pack-us-healthcare: ≥ 7y (HIPAA + state notary).
- pack-eu: per tenant DPA (bounded; min 2y).
- pack-kr: 5y default (commercial code).
- (others): per pack.

LTA profile re-stamps every 5 years to extend validity.

## Alternatives Considered

### Alternative A — SES-only (click-to-sign + audit-chain seal)

Implement only the simple class; rely on audit-chain Ed25519 seal as the "advanced" tier.

- **Pros**: simplest implementation; lowest cost; no QTSP integrations needed; oyatie's Ed25519 seal is cryptographically strong.
- **Cons**: cannot offer eIDAS QES Art. 25(2) legal equivalence in pack-eu; loses enterprise + government tenants who require QES for binding contracts; weak cross-jurisdictional recognition.
- **Rejected reason**: leaves the Tier-E+ + Tier-G market on the table. Forms competes with DocuSign + Adobe Sign for envelope-signing use cases; QES is the differentiator at the top tier.

### Alternative B — QES-only

Implement only QES (highest tier).

- **Pros**: maximum legal weight; simplest tier mapping (one class).
- **Cons**: every tenant must onboard a QSCD or qualified-cloud-signing reference; massive UX friction for Tier-2 tenants who don't need it; cost-pass-through ($0.20+ per signed envelope) prices out small tenants.
- **Rejected reason**: tier mapping serves the market: small tenants get SES (cheap, sufficient for most), enterprises get QES (high cost OK, legal weight required).

### Alternative C — PAdES-only wire format

Only support PAdES (PDF-based) signatures.

- **Pros**: most common use case; simpler implementation.
- **Cons**: tenants exporting XML or binary attachments lack signature wrapping; cross-jurisdictional XAdES requirements (e.g., some EU public-sector tenders) unmet.
- **Rejected reason**: enterprise tenants regularly mix formats; multi-format support is a moderate incremental investment.

### Alternative D — Third-party signing-as-a-service (DocuSign / Adobe Sign integration)

Integrate with DocuSign or Adobe Sign as the signing layer.

- **Pros**: shortest time-to-market; outsource the QTSP complexity.
- **Cons**: signing layer becomes an external dependency (residency + sub-processor entry per pack); per-envelope cost-pass-through; tenant relationship with the third party.
- **Rejected reason**: signature is a core trust primitive of Forms; outsourcing creates a perpetual sub-processor + residency story we don't want to maintain. Adopting eIDAS natively gives us the trust primitive in-product.

### Alternative E — Implement signature as a thin Ed25519 audit-chain extension (no eIDAS conformance)

Use the audit-chain Ed25519 seal as the signature; no eIDAS / XAdES / PAdES / CAdES wire format.

- **Pros**: minimal incremental work given existing audit-chain.
- **Cons**: no legal recognition outside oyatie ecosystem; envelope cannot be verified by external parties without oyatie's keys + tooling; fails eIDAS QES Art. 25(2) requirements.
- **Rejected reason**: signature must be portable + externally verifiable; eIDAS conformance is the trust property here.

## Consequences

### Architectural

- The `oya-forms-esign-domain` kernel exposes `Sign(envelope, signer_class, signing_credential_ref) -> SignedEnvelope` and `Verify(SignedEnvelope) -> VerificationResult`.
- Per-class implementations: `SES` (Ed25519 + audit-chain), `AES` (PKCS#7 with oyatie-CA or tenant-CA), `QES` (PAdES/XAdES/CAdES via tenant's QTSP integration).
- The export-worker invokes the esign-domain on tenant-requested signed exports.
- The form-renderer Leptos-WASM includes signature-capture UI (signature pad for SES; QSCD redirect flow for AES/QES).

### Downstream µservices

1. **tenancy**: per-tenant signature-class entitlement; QSCD reference storage.
2. **fintech** (Tier-G): payment field's payment-receipt envelope may include AES signature.
3. **audit-chain**: every signed envelope's hash sealed into chain.
4. **drive**: signed envelopes archived in tenant's drive root or oyatie-managed archive bucket per tenant preference.
5. **ontology**: `SignedEnvelope` entity with `signature_class`, `signer_identifier_hash`, `signed_at`, `tsa_timestamp`, `qsca_certificate_chain_hash`.

### SLOs and CI lanes affected

- `oya-forms-esign-conformance` — verify signed envelope against EU Trusted List + per-pack equivalents.
- `oya-forms-esign-pades-lta-archival` — periodic re-stamp drill.
- `oya-forms-esign-tier-mapping-conformance` — Cedar gate prevents tier-leak.

### Compliance + audit

- eIDAS Art. 25 legal recognition: SES (Art. 25(1)) + QES (Art. 25(2)) preserved.
- HIPAA: PAdES-LTA envelopes for patient consent; 7y archival.
- KR Electronic Signature Act: KISA-certified CA chain.
- US ESIGN + UETA: SES default sufficient; state-attorney attestation reference for state notary law.

### Risk register

- **Risk**: EU Trusted List update mid-archival; cert revocation. **Mitigation**: PAdES-LTA includes long-term validation material; re-stamp every 5 years.
- **Risk**: Tenant QTSP outage during signing. **Mitigation**: signing flow is async with retry; tenant comms; fallback to AES class if tenant pre-approved.
- **Risk**: PostQuantum signature migration. **Mitigation**: ADR supersession path; ETSI is publishing PQ-eIDAS draft.
- **Risk**: Cross-jurisdictional non-recognition. **Mitigation**: per-pack overlay maps QES to local equivalent; tenant comms make clear which jurisdictions accept which class.

## References

- Regulation (EU) 910/2014 (eIDAS) — Official Journal of the European Union, 28 August 2014.
- ETSI EN 319 122 (CAdES) — `etsi.org/`.
- ETSI EN 319 132 (XAdES).
- ETSI EN 319 142 (PAdES).
- ETSI TS 119 612 (EU Trusted Lists).
- IETF RFC 3161 (Time-Stamp Protocol).
- IETF RFC 5652 (Cryptographic Message Syntax / PKCS#7).
- KR Electronic Signature Act + KISA accreditation list — `kisa.or.kr/`.
- ICP-Brasil — `iti.gov.br/icp-brasil`.
- India IT Act 2000 + Controller of Certifying Authorities (CCA) Class 3 DSC list.
- US ESIGN Act of 2000; UETA.
- AU Electronic Transactions Act 1999.
- JP Act on Electronic Signatures and Certification Services 2001.
- `microservices/forms/PRD.md` FR-09 + AC-12.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/threat-model.md` T-R-04.
- ADR-FORMS-0001 (form.v1 schema_hash; sealed envelope).
- ADR-FORMS-0003 (PII encryption; envelope decryption boundary).
- ADR-0140 Cedar entitlement gate.
- ADR-0131 per-microservice flat layout.
