---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: gdpr
authoritative_source: Regulation (EU) 2016/679 (GDPR)
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# GDPR Pack Overlay — CLM

GDPR (Regulation (EU) 2016/679) is the controlling EU data-protection regime for any contract that processes personal data of EU data subjects, regardless of where the controller is established.

## Active triggers

The `gdpr` pack is **mandatory** when any of the following hold for a `(tenant, contract)` tuple:

- `counterparty.principal_residency ∈ EEA_states` (28 EU + 3 EEA member states + UK transitional).
- `contract.signatory.residency ∈ EEA_states`.
- `contract.body` contains personal data (GDPR Article 4(1) definition) of an EU data subject.
- `tenant.declared_jurisdictions` includes any EEA state.

## Enforced behaviour

### Article 5 — processing principles

- **Purpose limitation**: every contract carries a declared `processing_purpose` field; clause analysis cannot infer a secondary purpose without re-collecting consent.
- **Data minimization**: counterparty PII fields collected at intake are restricted to the minimum required to execute and enforce the contract (name, role, business-contact email, signatory authority).
- **Storage limitation**: see retention overlay below.
- **Accuracy**: counterparty MDM (per `counterparty-mdm/counterparty-mdm.md`) maintains current legal-entity name; outdated records flagged within 30 days of detected change.
- **Integrity and confidentiality**: AES-256-GCM at rest, TLS 1.3 + ECH in transit (ADR-0253).
- **Accountability**: every state transition emits an audit-chain event sealed with the tenant's signing key (ADR-0263).

### Article 6 — lawful basis

Each contract carries a `lawful_basis ∈ {contract_performance, legitimate_interest, consent, legal_obligation, vital_interests, public_task}` declaration. The default for a commercial contract is `contract_performance` per Article 6(1)(b); explicit consent capture is required only for PII processing that exceeds contract performance (e.g. marketing opt-in clauses).

### Article 7 — consent records (P0 LEGAL — see `legal-dimensions/gdpr-article-7-consent-records.md`)

When `lawful_basis = consent`, the µservice persists a consent record with the following minimum fields:

- `consent_id` (immutable UUID v7).
- `data_subject_principal_id` (foreign key to identity µservice).
- `purpose_text` (verbatim, in the data subject's chosen language).
- `consent_given_at` (RFC 3339 with tenant-home-cell time zone).
- `consent_mechanism` ∈ `{clickwrap_with_separate_checkbox, e_signature_with_consent_clause, written_signature_with_consent_clause, oral_consent_recorded}`.
- `withdrawal_endpoint` (HTTP URL the data subject can use to retract consent without disproportionate effort).
- `evidence_hash` (BLAKE3 of the consent artefact bundle).
- `audit_event_id` (cross-reference to audit-chain emission).

Consent withdrawal is processed within 72 hours and triggers a re-evaluation of any obligation that depended on the withdrawn lawful basis.

### Article 17 — right to erasure

CLM contracts are subject to erasure requests **except** where Article 17(3) exemptions apply (legal claims, public-interest archival, regulatory obligation). The exemption applies broadly to executed contracts (legal-claims preservation). Erasure of pre-signature drafts and counterparty PII not material to contract enforcement is honoured within 30 days.

### Article 25 — data protection by design

- Default `data_classification = PII_QUASI` for all contract bodies.
- Counterparty MDM master records carry `data_classification = PII_DIRECT` only for explicit signatory identification fields.
- Tenant-scoped projection (per ADR-0244) prevents cross-tenant leak by construction.

### Article 28 — processor obligations

When the tenant uses Oyatie as a data processor for personal data in contracts, the Oyatie Data Processing Addendum (DPA) is applied automatically. The DPA itself is a `dpa` contract type in the contract type taxonomy; tenants execute it at onboarding.

### Article 32 — security of processing

- AES-256-GCM at rest, RFC 8439 ChaCha20-Poly1305 fallback on platforms lacking AES hardware acceleration.
- TLS 1.3 floor; ECH where terminated; PQC hybrid (X25519+ML-KEM-768) where negotiated.
- HSM-resident signing keys for QES (see `packs/eidas/README.md`).
- All `policy/*.cedar` evaluations are atomic; partial evaluation results never reach storage.
- Breach detection cross-emits to the `detection` substrate via ADR-0263.

### Article 33-34 — breach notification

Personal-data breach detection triggers a 72-hour clock to notify the relevant supervisory authority and (where the breach is high-risk to data subjects) the data subjects directly. The µservice does not deliver these notifications directly; it cross-emits to the `governance` µservice which holds the regulator-contact registry.

### Article 35 — DPIA

CLM is high-risk-eligible under Article 35(3)(b) when contracts contain Article 9 special-category personal data (health, religion, ethnicity, sexual orientation, biometric, genetic). The DPIA is maintained in `dpia.md`; activation of the `hipaa-baa` pack or the `health-records` data class automatically refreshes the DPIA review trigger.

## Retention overlay

| Contract class | Default GDPR retention | Source |
|---|---|---|
| Executed commercial contract | 6 years from termination | Limitation Act 1980 (UK) + Article 17(3)(e) legal-claims preservation |
| Executed contract with personal data | 6 years from termination, then anonymization | Article 5(1)(e) storage limitation |
| Pre-signature draft (>90 days idle) | 90 days from last edit | Article 5(1)(e) minimization |
| Consent record | Duration of consent + 3 years post-withdrawal | Article 7(1) demonstration evidence |
| Counterparty PII (signatory identity) | Duration of contract + 6 years post-termination | Article 17(3)(e) |

## Residency overlay

When `gdpr` is active and `deployment_context ∈ {oyatie-public-cloud, aws-guest, oci-guest}`:

- Primary store in an EEA cell (Frankfurt, Paris, Dublin, Stockholm) per the cell-eligibility map.
- Cross-region replication restricted to other EEA cells.
- Outbound transfer to non-EEA cells permitted only with SCCs (Standard Contractual Clauses 2021/914) recorded as a sub-contract of type `dpa-scc`.

When `deployment_context ∈ {on-prem, colo}`:

- Tenant-controlled residency; Oyatie provides the OpenTofu module (`iac/on-prem/` / `iac/colo/`) and verifies via attestation that the deployment is within EEA borders.

## Composition with other packs

- `gdpr` + `kr-pipa`: higher-restriction-wins; KR-PIPA cross-border-transfer rules typically stricter than GDPR Chapter V.
- `gdpr` + `hipaa-baa`: HIPAA covered-entity rules layered on top; the BAA contract type itself becomes a GDPR Article 28 DPA.
- `gdpr` + `sox-404`: SOX seven-year retention overrides GDPR six-year retention for SOX-relevant contracts (audit-relevant contracts of public companies).

## Cedar gate fragment

```cedar
forbid (
  principal,
  action == Action::"ContractDelete",
  resource is Contract
) when {
  resource.active_packs.contains("gdpr") &&
  resource.legal_hold_active == false &&
  resource.retention_remaining_days > 0
};

permit (
  principal,
  action == Action::"ConsentWithdraw",
  resource is ConsentRecord
) when {
  principal.principal_id == resource.data_subject_principal_id
};
```

(Full Cedar fragment in `policy/pack-gdpr.cedar` once policy module is split.)

## Evidence on activation

Activation of the `gdpr` pack on a tenant emits:

- `oya.contract.lifecycle.management.pack.gdpr.activated` audit event with the tenant's declared `data_protection_officer_contact`.
- A DPIA refresh task in workflow-engine.
- Cedar policy compilation against the tenant-scoped schema.
- A counterparty PII inventory snapshot (Article 30 record of processing activities).
