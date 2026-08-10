---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: eidas
authoritative_source: Regulation (EU) 910/2014 (eIDAS)
related_adrs: [ADR-0251, ADR-0244, ADR-0247, ADR-0263]
date: 2026-05-21
---

# eIDAS Pack Overlay — CLM

eIDAS (Regulation (EU) 910/2014, as amended by Regulation (EU) 2024/1183 eIDAS 2.0) is the EU electronic identification + trust services regime. CLM must produce evidence at three signature levels: Simple Electronic Signature (SES) per Article 25; Advanced Electronic Signature (AES) per Article 26; Qualified Electronic Signature (QES) per Article 28.

## Active triggers

The `eidas` pack is **mandatory** when any of the following hold:

- `contract.signatory.signature_jurisdiction ∈ EU_27 ∪ EEA_3 ∪ UK_transitional`.
- `contract.governing_law` references an EU member state legal system.
- `tenant.declared_jurisdictions` includes any EU member state.

## Signature level matrix

| Level | Article | Required evidence | Required at |
|---|---|---|---|
| SES (Simple) | Art. 25 | Electronic data attached to other electronic data for signing | Default fallback when AES/QES not required by contract terms |
| AES (Advanced) | Art. 26 | (a) Unique link to signatory; (b) capable of identifying signatory; (c) created with means under signatory's sole control; (d) linked to data such that subsequent change is detectable | Default for commercial B2B contracts |
| QES (Qualified) | Art. 28 | AES + qualified certificate from Trust List provider + qualified signature creation device (QSCD) | Required when contract terms or law mandate (e.g. real estate transfers, public-procurement contracts) |

## AES envelope (Article 26)

The canonical AES envelope is defined in `legal-dimensions/signature-envelope-canonical.md`. The eIDAS-specific subset:

- **Container**: CAdES-B-LTA (CMS Advanced Electronic Signature with Long-Term Archive timestamp), XAdES-B-LTA, or PAdES-B-LTA depending on document MIME type.
  - PDF → PAdES-B-LTA per ETSI EN 319 142-1.
  - XML → XAdES-B-LTA per ETSI EN 319 132-1.
  - Everything else → CAdES-B-LTA per ETSI EN 319 122-1.
- **Hash algorithm**: SHA-256 default; SHA-384 / SHA-512 / SHA-3 / BLAKE3 selectable per tenant.
- **Signer certificate path**: full X.509 chain to a Trust List root retained in the signature artefact.
- **Timestamp**: RFC 3161 timestamp from a Trust List TSA included in the AdES-B-T archive layer.

## QES requirements (Article 28)

### Qualified certificate

Must be issued by a Qualified Trust Service Provider (QTSP) listed on the EU LOTL (List of Trusted Lists). The µservice ingests the LOTL on a 6-hour cadence via the `kms` substrate adapter and validates each signature against the current LOTL state.

### QSCD (Qualified Signature Creation Device)

Approved devices include:

- **Thales Luna 7 A790** (Common Criteria EAL4+ certified, FIPS 140-3 Level 3).
- **Utimaco SecurityServer Se Gen2** (Common Criteria EAL4+, FIPS 140-3 Level 3).
- **Entrust nShield XC** (Common Criteria EAL4+, FIPS 140-3 Level 3).
- **AWS CloudHSM** (FIPS 140-2 Level 3; eIDAS QSCD-equivalence subject to per-jurisdiction QTSP attestation).
- **OCI Vault HSM** (FIPS 140-2 Level 3; eIDAS QSCD-equivalence subject to per-jurisdiction QTSP attestation).
- **Azure Key Vault Managed HSM** (FIPS 140-2 Level 3; same caveat as AWS/OCI).

### Trust List TSA

Time-Stamp Authorities for the AdES-B-LTA archive layer must themselves be on the LOTL. Examples:

- SwissSign Time Stamping Authority.
- Trustpro Qualified TSA.
- D-Trust Qualified TSA (Germany).
- Certigna TSA (France).
- KIR S.A. Qualified TSA (Poland).

The µservice maintains a per-tenant TSA preference order and falls over within the LOTL on TSA outage.

### Signer authentication

QES requires sole-control of the signing key. Acceptable authentication ladders for unlocking the HSM-resident key:

- WebAuthn FIDO2 (UV+UP) with a roaming authenticator (YubiKey 5C+, Feitian K9+, Token2 PIN+).
- eID smart card with PIN under PKCS#11.
- Mobile-based qualified electronic ID per Article 6a (mID, BankID, Smart-ID, NemID, ItsMe, Verimi, FranceConnect+).

Local biometrics alone are insufficient.

## eIDAS 2.0 wallet integration

Per Regulation (EU) 2024/1183, EU member states must provide European Digital Identity Wallets (EUDI Wallets) by 2026. The µservice exposes a `wallet-attest` endpoint that accepts:

- EUDI Wallet attribute attestations (issued under the ARF — Architecture and Reference Framework).
- PID (Person Identification Data) for signatory binding.
- QEAA (Qualified Electronic Attestations of Attributes) for role / authority binding (e.g. "this signatory has authority to sign on behalf of LEI 213800XYZ").

## Tenant-class composition

- `tenant_class=demo_trial`: AES only; QES not available (no Trust List TSA enrolment, no HSM provisioning, no eID authentication).
- `tenant_class=paid + billing_components=[per_seat]`: AES + QES via Oyatie-leased QSCD (platform_default credential mode).
- `tenant_class=paid + jurisdiction_pack=eu-eidas-qes`: AES + QES with BYOK HSM option (`provider_credential_modes.hsm_qes ∈ {platform_default, byok}`).
- Sovereign-cell tenants (e.g. `kr-pipa-sovereign` or per-member-state cells): QES with `byok_required_by_pack` HSM mode.

## Composition with other packs

- `eidas` + `gdpr`: signature artefacts include PII; retention follows the stricter pack. Article 17 erasure is suspended for signed artefacts under Article 17(3) legal-claims preservation.
- `eidas` + `kr-pipa`: cross-jurisdiction signatures (EU signatory + KR counterparty) trigger dual-pack evidence: eIDAS QES envelope on the EU side, KR 전자서명법 envelope on the KR side. Both envelopes attach to the same `signature_packet`.
- `eidas` + `sox-404`: archive retention extended to seven years.

## Cedar gate fragment

```cedar
forbid (
  principal,
  action == Action::"SignaturePacketSeal",
  resource is SignaturePacket
) when {
  resource.required_signature_level == "QES" &&
  (
    resource.signing_key.qscd_attestation == false ||
    resource.signer_certificate.lotl_qualified == false ||
    resource.tsa.lotl_qualified == false
  )
};
```

(Full Cedar fragment lives in `policy/pack-eidas.cedar` once policy module is split.)

## Evidence on activation

Activation of the `eidas` pack on a tenant emits:

- `oya.contract.lifecycle.management.pack.eidas.activated` audit event with the tenant's declared `signature_jurisdictions`.
- A LOTL snapshot pinned at activation time.
- Cedar policy compilation against the tenant-scoped schema with QES gates enabled.
- QSCD enrolment workflow if `provider_credential_modes.hsm_qes ∈ {byok, byok_required_by_pack}`.

## Standards references

- ETSI EN 319 102-1 — Electronic Signatures and Infrastructures (ESI); Procedures for Creation and Validation of AdES Digital Signatures.
- ETSI EN 319 122-1 — CAdES profiles.
- ETSI EN 319 132-1 — XAdES profiles.
- ETSI EN 319 142-1 — PAdES profiles.
- ETSI EN 319 412-2/3/5 — Certificate Profiles for Trust Service Providers.
- ETSI TS 119 312 — Cryptographic Suites.
- ETSI TS 119 612 — Trusted Lists.
- RFC 3161 — Internet X.509 Public Key Infrastructure Time-Stamp Protocol (TSP).
- RFC 5652 — Cryptographic Message Syntax (CMS).
- ISO 32000-2 — PDF 2.0 (PAdES dependency).
- Commission Implementing Decision (EU) 2015/1505 — LOTL technical specifications.
