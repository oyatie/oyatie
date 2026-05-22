---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-002
authoritative_source: eIDAS Articles 25-26-28 + ETSI EN 319 102-1 family
related_packs: [eidas, esign, kr-pipa, sec-17a-4]
date: 2026-05-21
---

# Signature Envelope — Canonical Model

This document specifies the canonical signature envelope used across all jurisdiction packs. Per ADR-CLM-001 §C5, the envelope must satisfy AES requirements at minimum (eIDAS Article 26 + ESIGN Act intent capture); QES (eIDAS Article 28) and certified electronic signature (KR 전자서명법) layer additional evidence on top of the same envelope.

## Envelope shape

```
SignatureEnvelope {
  envelope_id: UUIDv7,
  contract_id: ContractId,
  contract_version_hash: BLAKE3Hash,               // hash of the immutable contract artefact at sign time
  signature_level: SignatureLevel,                 // SES | AES | QES | KR_CERTIFIED
  envelope_format: EnvelopeFormat,                 // CAdES | XAdES | PAdES | DETACHED_BLAKE3
  hash_algorithm: HashAlgorithm,                   // SHA-256 default; SHA-384 | SHA-512 | SHA-3-256 | BLAKE3
  signer_attestation: SignerAttestation,
  signer_certificate_chain: [X509Certificate],
  signing_key_custody: KeyCustody,
  intent_evidence: IntentEvidence,
  timestamp_authority_signature: TSASignature,
  archive_timestamp: TSASignature?,                // for AdES-B-LTA
  envelope_blob: Vec<u8>,                          // the serialised envelope
  envelope_blob_hash: BLAKE3Hash,
  pack_overlays: [PackId],                         // active packs at sign time
  audit_event_id: AuditEventId,
}

enum SignatureLevel {
  SES,                                             // eIDAS Art. 25 + ESIGN basic
  AES,                                             // eIDAS Art. 26
  QES,                                             // eIDAS Art. 28
  KR_CERTIFIED,                                    // 인증전자서명
  JP_NINTEI,                                       // 認定認証業務
}

enum EnvelopeFormat {
  CAdES_B,         // ETSI EN 319 122-1, Basic
  CAdES_B_T,       // + Timestamp
  CAdES_B_LT,      // + Long Term
  CAdES_B_LTA,     // + Long Term Archive
  XAdES_B,         // ETSI EN 319 132-1, Basic
  XAdES_B_T,
  XAdES_B_LT,
  XAdES_B_LTA,
  PAdES_B,         // ETSI EN 319 142-1, Basic (PDF)
  PAdES_B_T,
  PAdES_B_LT,
  PAdES_B_LTA,
  Detached_BLAKE3, // pure detached signature; for non-AdES backwards compat
}

enum HashAlgorithm {
  SHA256, SHA384, SHA512, SHA3_256, SHA3_512, BLAKE3,
}

struct SignerAttestation {
  signatory_legal_name: String,                    // declared full legal name
  signatory_principal_id: PrincipalId,             // from identity µservice
  signatory_role_attestation: RoleAttestation?,    // e.g. "Chief Legal Officer of LEI 213800XYZ"
  authentication_ladder: [AuthenticationFactor],   // WebAuthn / eID / mobile-ID / etc.
  signature_jurisdiction: CountryCode,
  signature_location_attestation: GeoAttestation?, // optional, when required by contract
  signature_attestation_timestamp: Timestamp<RFC3339>,
}

enum AuthenticationFactor {
  WebAuthnFIDO2 { authenticator_aaguid: UUID, user_verified: bool, user_present: bool },
  EIDSmartCard { issuer_id: WalletIssuerId, certificate_serial: String },
  MobileQualifiedID { issuer_id: WalletIssuerId, attestation_id: WalletAttestationId },
  EUDIWallet { issuer_id: WalletIssuerId, pid_attestation_id: WalletAttestationId },
  HardwareToken { token_serial: String, attestation_certificate: X509Certificate },
  Password { mfa_method: MFAMethod },             // insufficient for AES alone
  OAuth2Federated { issuer: OAuth2Issuer, subject_claim: String },
}

enum KeyCustody {
  SoftwareKey { storage: SoftwareKeyStorage },    // SES only
  HSMHosted {
    hsm_id: HSMIdentifier,
    fips_140_level: u8,                            // 2 or 3
    eidas_qscd_certified: bool,                    // for QES
    common_criteria_eal: u8?,                      // for QES
    key_attestation: HSMAttestation,
    custody_mode: CustodyMode,                     // platform_default | byok | byok_required_by_pack
  },
  WalletResident {                                 // EUDI Wallet, KR mobile-ID
    wallet_id: WalletIdentifier,
    custody_proof: WalletAttestation,
  },
}

struct IntentEvidence {
  document_full_text_displayed: bool,              // ESIGN clear-and-conspicuous
  intent_statement_displayed: String,              // "By clicking Sign, I confirm..."
  intent_statement_hash: BLAKE3Hash,
  display_artefact_id: ArtefactId,                 // screenshot or DOM snapshot
  declared_signatory_name: String,                 // typed by signatory
  network_attestation: NetworkAttestation,         // IP, user-agent, ASN
  click_timestamp: Timestamp<RFC3339>,
  click_to_sign_duration_ms: u64,
  esign_consumer_disclosure_id: ConsumerDisclosureEvidenceId?,  // if counterparty is consumer
}
```

## Envelope format selection rule

| Document MIME type | Default envelope |
|---|---|
| `application/pdf` | PAdES-B-LTA |
| `application/xml` | XAdES-B-LTA |
| `application/json` | XAdES-B-LTA over a canonicalised XML wrapper |
| `application/vnd.openxmlformats-officedocument.wordprocessingml.document` (.docx) | PAdES-B-LTA after PDF conversion; or CAdES-B-LTA over the .docx blob |
| Any other | CAdES-B-LTA |
| Plain text / Markdown contract bodies | CAdES-B-LTA |

Tenant may override the default per contract type or per jurisdiction pack.

## Hash algorithm selection rule

Default SHA-256. Tenant may upgrade per contract:

- SHA-384 / SHA-512: when contract terms reference a stronger hash.
- SHA-3-256 / SHA-3-512: when tenant requires NIST FIPS 202 compliance.
- BLAKE3: only for SES envelopes; not yet accepted by major TSA providers under ETSI EN 319 312.

## QES additional gates (eIDAS Article 28)

For `signature_level = QES`, the envelope must additionally satisfy:

- `signing_key_custody = HSMHosted { fips_140_level >= 3, eidas_qscd_certified = true }`.
- `signer_certificate_chain[0]` issued by a QTSP listed on the EU LOTL.
- `timestamp_authority_signature` from a Trust List TSA (LOTL-qualified).
- `archive_timestamp` present (AdES-B-LTA mandatory).

## Korean Certified additional gates (전자서명법)

For `signature_level = KR_CERTIFIED`, the envelope must additionally satisfy:

- `signer_certificate_chain[0]` issued by a Korean Certified Electronic Signature Service (전자서명인증사업자) accredited under Article 7 (KICA, KTNET, KOSCOM, SignKorea, Yessign).
- `timestamp_authority_signature` from a KISA-rooted TSA.

## SES → AES → QES progressive signing

A single contract may carry signatures at different levels per signatory. The signature_packet aggregates all signatures and reports the lowest level achieved as the contract's effective signature level. Subsequent counterparty signatures may upgrade the level.

## Provenance

Every signature envelope cross-references:

- The contract version (immutable, content-addressed).
- The signer's identity attestation.
- The active packs at sign time.
- The Cedar policy decision that permitted the seal.

Tampering with any of these breaks the hash chain and is detectable by validators.

## Rust implementation sketch

The canonical envelope encoder/decoder lives in `crates/oya-clm-signature-envelope-kernel/`. Dependencies:

- `rasn` (Rust pure ASN.1 / DER for CAdES/XAdES/PAdES container parsing).
- `pkcs7` / `cms` (CMS construction).
- `ring` or `rustcrypto/signature` (cryptographic primitives, FIPS mode where applicable).
- `x509-parser` (certificate chain validation).
- `tsp-client` (RFC 3161 client) — custom implementation given upstream sparseness.
- `pqcrypto` (post-quantum hybrid signatures, optional under PQC pack).

The kernel is pure (no I/O); adapters in `crates/oya-clm-signature-envelope-adapter-*` integrate with provider HSMs, EUDI Wallets, KISA TSAs, etc.

## Audit event

`oya.contract.lifecycle.management.signature.envelope.sealed` with dimensions:

- tenant_id, tenant_class, contract_id, envelope_id
- signature_level, envelope_format, hash_algorithm
- signatory_principal_id, signature_jurisdiction
- active_packs, audit_event_id

## Standards references

- ETSI EN 319 102-1 — Procedures for Creation and Validation of AdES Digital Signatures.
- ETSI EN 319 122-1 / 132-1 / 142-1 — CAdES/XAdES/PAdES profiles.
- ETSI TS 119 312 — Cryptographic Suites.
- ETSI TS 119 612 — Trusted Lists.
- RFC 3161 — Time-Stamp Protocol.
- RFC 5652 — Cryptographic Message Syntax (CMS).
- ISO 32000-2 — PDF 2.0.
